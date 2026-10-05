# tauri-plugin-window-integration

Native window-chrome integration for Tauri 2 desktop apps: OS accent color, window vibrancy, Windows 11 snap-layout geometry, and Linux titlebar button layout/icons.

## Commands

Platform-specific commands only exist (compile and get registered) on their own platform — calling one from the wrong platform fails with Tauri's standard "command not found" IPC error. Use the `@nickvisionapps/plugin-window-integration` JS bindings instead of raw `invoke()` where possible; they check the platform for you and resolve to a harmless default on unsupported platforms, so client code never needs its own platform checks.

| Invoke string | Platform | Description |
| --- | --- | --- |
| `plugin:window-integration\|get_accent_color` | macOS, Windows, Linux | Returns the current OS accent color as `#rrggbb`, or `null`. |
| `plugin:window-integration\|update_vibrancy` | Windows only | Applies mica/acrylic/blur vibrancy for the given `dark` flag. |
| `plugin:window-integration\|get_button_layout` | Linux only | Returns the desktop's titlebar button order (GTK, falling back to KDE), or `null`. |
| `plugin:window-integration\|get_titlebar_icons` | Linux only | Returns a map of titlebar button icons as base64 data URIs, or `null`. |
| `plugin:window-integration\|update_snap_geometry` | Windows only | Registers/updates the on-screen geometry of a custom maximize button so Windows 11's snap-layout flyout appears over it. |
| `plugin:window-integration\|clear_snap_geometry` | Windows only | Removes the snap-layout overlay for a window. |

macOS vibrancy is applied automatically to every window as it's created — no setup call needed. Windows vibrancy doesn't self-track light/dark, so call `update_vibrancy` whenever your app's resolved theme changes.

The Windows snap-overlay also emits plain (unnamespaced) `snap-hover`/`snap-press` boolean events on the window while the maximize button is hovered/pressed — harmless to listen for on any platform, since they simply never fire outside Windows.

## Required consumer setup

1. Add `"window-integration:default"` to your app's `capabilities/*.json`.
2. Register the plugin: `.plugin(tauri_plugin_window_integration::init())`.
3. On macOS, your own `tauri.conf.json` window config must set `"transparent": true` and `"macOSPrivateApi": true` — `window-vibrancy` requires both, and a plugin has no way to set them on your behalf.

## JS/TS bindings

See `guest-js/` for the typed wrapper package `@nickvisionapps/plugin-window-integration`.
