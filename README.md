# tauri-plugin-window-integration

Native window-chrome integration for Tauri 2 desktop apps: OS accent color, window vibrancy, Windows 11 snap-layout geometry, and Linux titlebar button layout/icons.

## Commands

Every command is registered on every platform. Commands that don't apply to the current platform are harmless no-ops that resolve to `null` (or nothing), so client code never needs its own platform checks.

| Invoke string | Platform | Description |
| --- | --- | --- |
| `plugin:window-integration\|get_accent_color` | macOS, Windows, Linux | Returns the current OS accent color as `#rrggbb`, or `null`. |
| `plugin:window-integration\|update_vibrancy` | Windows only | Applies Mica (falling back to blur on systems without Mica) for the given `dark` flag, and returns an error if no effect could be applied. |
| `plugin:window-integration\|get_button_layout` | Linux only | Returns the desktop's titlebar button order (GTK, falling back to KDE), or `null`. |
| `plugin:window-integration\|get_titlebar_icons` | Linux only | Returns a map of titlebar button icons as base64 data URIs, or `null`. |
| `plugin:window-integration\|update_snap_geometry` | Windows only | Registers/updates the on-screen geometry of a custom maximize button so Windows 11's snap-layout flyout appears over it. |
| `plugin:window-integration\|clear_snap_geometry` | Windows only | Removes the snap-layout overlay for a window. |

macOS vibrancy is applied automatically to every window as it's created — no setup call needed. Windows vibrancy doesn't self-track light/dark, so call `update_vibrancy` whenever your app's resolved theme changes.

The Windows snap-overlay also emits `plugin:window-integration:snap-hover` / `plugin:window-integration:snap-press` boolean events, targeted only at the window that owns the maximize button, while it is hovered/pressed. They never fire outside Windows. The overlay stays anchored to the window's top-right edge across resizes and DPI changes.

## Required consumer setup

1. Add `"window-integration:default"` to your app's `capabilities/*.json`.
2. Register the plugin: `.plugin(tauri_plugin_window_integration::init())`.
3. On macOS, your own `tauri.conf.json` window config must set `"transparent": true` and `"macOSPrivateApi": true` — `window-vibrancy` requires both, and a plugin has no way to set them on your behalf.

## JS/TS bindings

See `guest-js/` for the typed wrapper package `@nickvisionapps/plugin-window-integration`.
