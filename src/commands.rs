#[cfg(target_os = "linux")]
use std::collections::BTreeMap;
#[cfg(any(target_os = "linux", target_os = "windows"))]
use tauri::WebviewWindow;
use tauri::{AppHandle, Runtime, command};

#[cfg(target_os = "windows")]
#[command]
pub fn clear_snap_geometry<R: Runtime>(window: WebviewWindow<R>) -> Result<(), String> {
    crate::windows::uninstall_snap_layout(&window)
}

#[command]
pub async fn get_accent_color<R: Runtime>(app: AppHandle<R>) -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        crate::macos::accent_color(&app).await
    }
    #[cfg(target_os = "windows")]
    {
        let _ = app;
        crate::windows::accent_color()
    }
    #[cfg(target_os = "linux")]
    {
        let _ = app;
        crate::linux::accent_color()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        let _ = app;
        None
    }
}

#[cfg(target_os = "linux")]
#[command]
pub fn get_button_layout() -> Option<String> {
    crate::linux::button_layout()
}

#[cfg(target_os = "linux")]
#[command]
pub fn get_titlebar_icons<R: Runtime>(
    window: WebviewWindow<R>,
) -> Option<BTreeMap<&'static str, String>> {
    crate::linux::titlebar_icons(&window)
}

#[cfg(target_os = "windows")]
#[command]
pub fn update_snap_geometry<R: Runtime>(
    window: WebviewWindow<R>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    titlebar_height: u32,
    control_band_width: u32,
) -> Result<(), String> {
    crate::windows::install_or_update_snap_layout(
        &window,
        x,
        y,
        width,
        height,
        titlebar_height,
        control_band_width,
    )
}

#[cfg(target_os = "windows")]
#[command]
pub fn update_vibrancy<R: Runtime>(window: WebviewWindow<R>, dark: bool) -> Result<(), String> {
    crate::windows::apply_window_vibrancy(&window, dark);
    Ok(())
}
