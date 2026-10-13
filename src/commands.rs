use std::collections::BTreeMap;
use tauri::{AppHandle, Runtime, WebviewWindow, command};

#[command]
pub fn clear_snap_geometry<R: Runtime>(window: WebviewWindow<R>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        crate::windows::uninstall_snap_layout(&window)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = window;
        Ok(())
    }
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
        tauri::async_runtime::spawn_blocking(crate::linux::accent_color)
            .await
            .ok()
            .flatten()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        let _ = app;
        None
    }
}

#[command]
pub fn get_button_layout() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        crate::linux::button_layout()
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

#[command]
pub fn get_titlebar_icons<R: Runtime>(
    window: WebviewWindow<R>,
) -> Option<BTreeMap<&'static str, String>> {
    #[cfg(target_os = "linux")]
    {
        crate::linux::titlebar_icons(&window)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = window;
        None
    }
}

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
    #[cfg(target_os = "windows")]
    {
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
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (
            window,
            x,
            y,
            width,
            height,
            titlebar_height,
            control_band_width,
        );
        Ok(())
    }
}

#[command]
pub fn update_vibrancy<R: Runtime>(window: WebviewWindow<R>, dark: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        crate::windows::apply_window_vibrancy(&window, dark)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (window, dark);
        Ok(())
    }
}
