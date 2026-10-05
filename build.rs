const COMMANDS: &[&str] = &[
    "clear_snap_geometry",
    "get_accent_color",
    "get_button_layout",
    "get_titlebar_icons",
    "update_snap_geometry",
    "update_vibrancy",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
