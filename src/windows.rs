use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError},
};
use tauri::{Emitter, Runtime, WebviewWindow};
use windows_sys::Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        HiDpi::GetDpiForWindow,
        Input::KeyboardAndMouse::{TME_LEAVE, TME_NONCLIENT, TRACKMOUSEEVENT, TrackMouseEvent},
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::{
            CS_HREDRAW, CS_VREDRAW, CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect,
            HTMAXBUTTON, HWND_TOP, RegisterClassExW, SW_HIDE, SWP_NOACTIVATE, SWP_SHOWWINDOW,
            SetWindowPos, ShowWindow, WM_DPICHANGED, WM_NCDESTROY, WM_NCHITTEST, WM_NCLBUTTONDOWN,
            WM_NCLBUTTONUP, WM_NCMOUSELEAVE, WM_NCMOUSEMOVE, WM_SIZE, WNDCLASSEXW, WS_CHILD,
            WS_CLIPSIBLINGS, WS_VISIBLE,
        },
    },
};

const MAX_CONTROL_BAND_LOGICAL: u64 = 640;
const MAX_MAXIMIZE_HEIGHT_LOGICAL: u64 = 128;
const MAX_MAXIMIZE_WIDTH_LOGICAL: u64 = 256;
const MAX_TITLEBAR_BAND_LOGICAL: u64 = 128;
const SNAP_CLASS_NAME: &str = "TauriPluginWindowIntegrationSnapOverlay";
const SNAP_CLASS: [u16; SNAP_CLASS_NAME.len() + 1] = {
    let mut class = [0u16; SNAP_CLASS_NAME.len() + 1];
    let bytes = SNAP_CLASS_NAME.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        class[index] = bytes[index] as u16;
        index += 1;
    }
    class
};
const SUBCLASS_ID: usize = 1;

type SnapEmitter = Arc<dyn Fn(&str, bool) + Send + Sync>;

struct Entry {
    overlay: isize,
    emit: SnapEmitter,
    geometry: Geometry,
    hovering: bool,
    pressing: bool,
}

#[derive(Clone, Copy)]
struct Geometry {
    right_inset: i32,
    y: i32,
    width: u32,
    height: u32,
    dpi: u32,
}

pub fn accent_color() -> Option<String> {
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::Graphics::Dwm::DwmGetColorizationColor;
    use windows_sys::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_READ, REG_DWORD, RegCloseKey, RegOpenKeyExW, RegQueryValueExW,
    };

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    unsafe {
        let subkey = wide("Software\\Microsoft\\Windows\\DWM");
        let mut hkey: HKEY = std::mem::zeroed();
        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_READ, &mut hkey)
            == ERROR_SUCCESS
        {
            let value_name = wide("AccentColor");
            let mut data: u32 = 0;
            let mut data_len: u32 = size_of::<u32>() as u32;
            let mut value_type: u32 = 0;
            let status = RegQueryValueExW(
                hkey,
                value_name.as_ptr(),
                std::ptr::null_mut(),
                &mut value_type,
                &mut data as *mut u32 as *mut u8,
                &mut data_len,
            );
            RegCloseKey(hkey);
            if status == ERROR_SUCCESS && value_type == REG_DWORD {
                let r = (data & 0xFF) as u8;
                let g = ((data >> 8) & 0xFF) as u8;
                let b = ((data >> 16) & 0xFF) as u8;
                return Some(format!("#{r:02x}{g:02x}{b:02x}"));
            }
        }
        let mut colorization: u32 = 0;
        let mut opaque_blend = 0;
        if DwmGetColorizationColor(&mut colorization, &mut opaque_blend) < 0 {
            return None;
        }
        let r = ((colorization >> 16) & 0xFF) as u8;
        let g = ((colorization >> 8) & 0xFF) as u8;
        let b = (colorization & 0xFF) as u8;
        Some(format!("#{r:02x}{g:02x}{b:02x}"))
    }
}

pub fn apply_window_vibrancy<R: Runtime>(
    window: &WebviewWindow<R>,
    dark: bool,
) -> Result<(), String> {
    let mica_error = match window_vibrancy::apply_mica(window, Some(dark)) {
        Ok(()) => return Ok(()),
        Err(error) => error,
    };
    let tint = if dark {
        (20, 20, 20, 160)
    } else {
        (240, 240, 240, 160)
    };
    window_vibrancy::apply_blur(window, Some(tint))
        .map_err(|blur_error| format!("mica failed ({mica_error}); blur failed ({blur_error})"))
}

pub fn install_or_update_snap_layout<R: Runtime>(
    window: &WebviewWindow<R>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    titlebar_height: u32,
    control_band_width: u32,
) -> Result<(), String> {
    let hwnd = window_hwnd(window)?;
    if width == 0 || height == 0 || x < 0 || y < 0 {
        return Err(
            "snap maximize-button geometry must be a non-empty client rectangle".to_string(),
        );
    }
    let mut rect = unsafe { std::mem::zeroed() };
    if unsafe { GetClientRect(hwnd as HWND, &mut rect) } == 0 {
        return Err(format!(
            "GetClientRect failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    let right = i64::from(x) + i64::from(width);
    if right > i64::from(rect.right) || i64::from(y) + i64::from(height) > i64::from(rect.bottom) {
        return Err(
            "snap maximize-button geometry lies outside the window client area".to_string(),
        );
    }
    let dpi = unsafe { GetDpiForWindow(hwnd as HWND) };
    if dpi == 0 {
        return Err(format!(
            "GetDpiForWindow failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    let geometry = Geometry {
        right_inset: (i64::from(rect.right) - right) as i32,
        y,
        width,
        height,
        dpi,
    };
    let logical = |value: u64| value.saturating_mul(96).div_ceil(u64::from(dpi));
    if logical(u64::from(titlebar_height)) == 0
        || logical(u64::from(titlebar_height)) > MAX_TITLEBAR_BAND_LOGICAL
        || logical(u64::from(control_band_width)) == 0
        || logical(u64::from(control_band_width)) > MAX_CONTROL_BAND_LOGICAL
        || logical(u64::from(geometry.width)) == 0
        || logical(u64::from(geometry.width)) > MAX_MAXIMIZE_WIDTH_LOGICAL
        || logical(u64::from(geometry.height)) == 0
        || logical(u64::from(geometry.height)) > MAX_MAXIMIZE_HEIGHT_LOGICAL
        || u64::try_from(i64::from(rect.right) - right)
            .unwrap_or(u64::MAX)
            .saturating_add(u64::from(geometry.width))
            > u64::from(control_band_width)
        || u64::from(geometry.height) > u64::from(titlebar_height)
        || u64::try_from(geometry.y)
            .unwrap_or(u64::MAX)
            .saturating_add(u64::from(geometry.height))
            > u64::from(titlebar_height)
    {
        return Err(
            "snap maximize-button geometry is outside the trusted top-right caption region"
                .to_string(),
        );
    }
    let installed = registry()
        .get_mut(&hwnd)
        .map(|entry| entry.geometry = geometry)
        .is_some();
    if installed {
        return update_overlay_position(hwnd);
    }
    static REGISTRATION: OnceLock<Result<isize, String>> = OnceLock::new();
    let instance = REGISTRATION
        .get_or_init(|| unsafe {
            let instance = GetModuleHandleW(std::ptr::null()) as HINSTANCE;
            if instance.is_null() {
                return Err(format!(
                    "GetModuleHandleW failed: {}",
                    std::io::Error::last_os_error()
                ));
            }
            if RegisterClassExW(&WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(overlay_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: instance,
                hIcon: std::ptr::null_mut(),
                hCursor: std::ptr::null_mut(),
                hbrBackground: std::ptr::null_mut(),
                lpszMenuName: std::ptr::null(),
                lpszClassName: SNAP_CLASS.as_ptr(),
                hIconSm: std::ptr::null_mut(),
            }) == 0
            {
                return Err(format!(
                    "RegisterClassExW failed: {}",
                    std::io::Error::last_os_error()
                ));
            }
            Ok(instance as isize)
        })
        .as_ref()
        .copied()
        .map(|instance| instance as HINSTANCE)
        .map_err(|error| error.clone())?;
    let overlay = unsafe {
        CreateWindowExW(
            0,
            SNAP_CLASS.as_ptr(),
            SNAP_CLASS.as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS,
            0,
            0,
            0,
            0,
            hwnd as HWND,
            std::ptr::null_mut(),
            instance,
            std::ptr::null_mut(),
        )
    };
    if overlay.is_null() {
        return Err(format!(
            "CreateWindowExW failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    if unsafe { SetWindowSubclass(hwnd as HWND, Some(parent_subclass_proc), SUBCLASS_ID, 0) } == 0 {
        unsafe { DestroyWindow(overlay) };
        return Err("SetWindowSubclass failed for the snap overlay".to_string());
    }
    let window = window.clone();
    registry().insert(
        hwnd,
        Entry {
            overlay: overlay as isize,
            emit: Arc::new(move |event: &str, value: bool| {
                let _ = window.emit_to(window.label(), event, value);
            }),
            geometry,
            hovering: false,
            pressing: false,
        },
    );
    update_overlay_position(hwnd)
}

pub fn uninstall_snap_layout<R: Runtime>(window: &WebviewWindow<R>) -> Result<(), String> {
    let hwnd = window_hwnd(window)?;
    let Some(entry) = registry().remove(&hwnd) else {
        return Ok(());
    };
    unsafe {
        ShowWindow(entry.overlay as HWND, SW_HIDE);
        DestroyWindow(entry.overlay as HWND);
        RemoveWindowSubclass(hwnd as HWND, Some(parent_subclass_proc), SUBCLASS_ID);
    }
    Ok(())
}

fn emit_snap_event(hwnd: HWND, event: &str, value: bool) {
    let Some(emit) = parent_for_overlay(hwnd)
        .and_then(|parent| registry().get(&parent).map(|entry| entry.emit.clone()))
    else {
        return;
    };
    emit(event, value);
}

unsafe extern "system" fn overlay_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        match msg {
            WM_NCHITTEST
                if registry()
                    .values()
                    .any(|entry| entry.overlay == hwnd as isize) =>
            {
                return HTMAXBUTTON as LRESULT;
            }
            WM_NCMOUSEMOVE => {
                let Some(parent) = parent_for_overlay(hwnd) else {
                    return 0;
                };
                let entered = {
                    let mut state = registry();
                    let Some(entry) = state.get_mut(&parent) else {
                        return 0;
                    };
                    let entered = !entry.hovering;
                    entry.hovering = true;
                    entered
                };
                if entered {
                    emit_snap_event(hwnd, "plugin:window-integration:snap-hover", true);
                    let mut track = TRACKMOUSEEVENT {
                        cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                        dwFlags: TME_LEAVE | TME_NONCLIENT,
                        hwndTrack: hwnd,
                        dwHoverTime: 0,
                    };
                    unsafe { TrackMouseEvent(&mut track) };
                }
                return 0;
            }
            WM_NCMOUSELEAVE => {
                if let Some(parent) = parent_for_overlay(hwnd)
                    && let Some(entry) = registry().get_mut(&parent)
                {
                    entry.hovering = false;
                    entry.pressing = false;
                }
                emit_snap_event(hwnd, "plugin:window-integration:snap-hover", false);
                return 0;
            }
            WM_NCLBUTTONDOWN => {
                if let Some(parent) = parent_for_overlay(hwnd)
                    && let Some(entry) = registry().get_mut(&parent)
                {
                    entry.pressing = true;
                }
                emit_snap_event(hwnd, "plugin:window-integration:snap-press", true);
                return 0;
            }
            WM_NCLBUTTONUP => {
                if let Some(parent) = parent_for_overlay(hwnd)
                    && let Some(entry) = registry().get_mut(&parent)
                {
                    entry.pressing = false;
                }
                emit_snap_event(hwnd, "plugin:window-integration:snap-press", false);
                return 0;
            }
            WM_NCDESTROY => {
                if let Some(parent) = parent_for_overlay(hwnd) {
                    registry().remove(&parent);
                }
            }
            _ => {}
        }
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }))
    .unwrap_or_else(|_| unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) })
}

fn parent_for_overlay(overlay: HWND) -> Option<isize> {
    registry()
        .iter()
        .find_map(|(parent, entry)| (entry.overlay == overlay as isize).then_some(*parent))
}

unsafe extern "system" fn parent_subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _subclass_id: usize,
    _ref_data: usize,
) -> LRESULT {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        match msg {
            WM_DPICHANGED => {
                let new_dpi = (wparam & 0xFFFF) as u32;
                if let Some(entry) = registry().get_mut(&(hwnd as isize))
                    && new_dpi != 0
                    && entry.geometry.dpi != 0
                {
                    let old_dpi = i64::from(entry.geometry.dpi);
                    let scale = |value: i64| (value * i64::from(new_dpi) + old_dpi / 2) / old_dpi;
                    entry.geometry = Geometry {
                        right_inset: scale(i64::from(entry.geometry.right_inset)) as i32,
                        y: scale(i64::from(entry.geometry.y)) as i32,
                        width: scale(i64::from(entry.geometry.width)) as u32,
                        height: scale(i64::from(entry.geometry.height)) as u32,
                        dpi: new_dpi,
                    };
                }
            }
            WM_NCDESTROY => {
                registry().remove(&(hwnd as isize));
                unsafe { RemoveWindowSubclass(hwnd, Some(parent_subclass_proc), SUBCLASS_ID) };
            }
            _ => {}
        }
        let result = unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
        if matches!(msg, WM_SIZE | WM_DPICHANGED) {
            let _ = update_overlay_position(hwnd as isize);
        }
        result
    }))
    .unwrap_or_else(|_| unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) })
}

fn registry() -> MutexGuard<'static, HashMap<isize, Entry>> {
    static REGISTRY: OnceLock<Mutex<HashMap<isize, Entry>>> = OnceLock::new();
    REGISTRY
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

fn update_overlay_position(hwnd: isize) -> Result<(), String> {
    let Some((overlay, geometry)) = registry()
        .get(&hwnd)
        .map(|entry| (entry.overlay, entry.geometry))
    else {
        return Ok(());
    };
    let mut rect = unsafe { std::mem::zeroed() };
    if unsafe { GetClientRect(hwnd as HWND, &mut rect) } == 0 {
        return Err(format!(
            "GetClientRect failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    let x = rect.right - geometry.right_inset - geometry.width as i32;
    if x < 0 || rect.bottom < geometry.y + geometry.height as i32 {
        unsafe { ShowWindow(overlay as HWND, SW_HIDE) };
        return Ok(());
    }
    if unsafe {
        SetWindowPos(
            overlay as HWND,
            HWND_TOP,
            x,
            geometry.y,
            geometry.width as i32,
            geometry.height as i32,
            SWP_NOACTIVATE | SWP_SHOWWINDOW,
        )
    } == 0
    {
        return Err(format!(
            "SetWindowPos failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

fn window_hwnd<R: Runtime>(window: &WebviewWindow<R>) -> Result<isize, String> {
    match window
        .window_handle()
        .map_err(|error| error.to_string())?
        .as_raw()
    {
        RawWindowHandle::Win32(handle) => Ok(handle.hwnd.get()),
        _ => Err("native snap overlay requires a Win32 window handle".to_string()),
    }
}
