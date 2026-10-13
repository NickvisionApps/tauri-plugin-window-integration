# @nickvisionapps/plugin-window-integration

JS/TS bindings for the [`tauri-plugin-window-integration`](https://github.com/NickvisionApps/tauri-plugin-window-integration) Tauri plugin.

Platform-specific functions (`getButtonLayout`, `getTitlebarIcons`, `updateVibrancy`, `updateSnapGeometry`, `clearSnapGeometry`) resolve to a harmless default (`null` or nothing) when called on a platform that doesn't support them — your app code never needs to branch on the platform itself.

```ts
import {
  getAccentColor,
  updateVibrancy,
} from "@nickvisionapps/plugin-window-integration";

const accentColor = await getAccentColor();
await updateVibrancy(true); // no-op outside Windows
```
