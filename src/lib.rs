mod commands;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use tauri::{
    Runtime,
    plugin::{Builder, TauriPlugin},
};

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("window-integration")
        .invoke_handler(tauri::generate_handler![
            commands::clear_snap_geometry,
            commands::get_accent_color,
            commands::get_button_layout,
            commands::get_titlebar_icons,
            commands::update_snap_geometry,
            commands::update_vibrancy,
        ])
        .on_window_ready(|_window| {
            #[cfg(target_os = "macos")]
            macos::apply_window_vibrancy(&_window);
        })
        .build()
}
