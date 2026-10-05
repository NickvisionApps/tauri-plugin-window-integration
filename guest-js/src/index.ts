import {invoke} from "@tauri-apps/api/core";
import {listen, type UnlistenFn} from "@tauri-apps/api/event";
import {platform} from "@tauri-apps/plugin-os";

export interface SnapGeometry {
  x: number;
  y: number;
  width: number;
  height: number;
  titlebarHeight: number;
  controlBandWidth: number;
}

export async function clearSnapGeometry(): Promise<void> {
  if (platform() !== "windows") {
    return;
  }
  await invoke("plugin:window-integration|clear_snap_geometry");
}

export async function getAccentColor(): Promise<string | null> {
  return await invoke("plugin:window-integration|get_accent_color");
}

export async function getButtonLayout(): Promise<string | null> {
  if (platform() !== "linux") {
    return null;
  }
  return await invoke("plugin:window-integration|get_button_layout");
}

export async function getTitlebarIcons(): Promise<Record<
  string,
  string
> | null> {
  if (platform() !== "linux") {
    return null;
  }
  return await invoke("plugin:window-integration|get_titlebar_icons");
}

export async function onSnapHover(
  callback: (hovering: boolean) => void,
): Promise<UnlistenFn> {
  return await listen<boolean>("snap-hover", (event) => {
    callback(event.payload);
  });
}

export async function onSnapPress(
  callback: (pressing: boolean) => void,
): Promise<UnlistenFn> {
  return await listen<boolean>("snap-press", (event) => {
    callback(event.payload);
  });
}

export async function updateSnapGeometry(
  geometry: SnapGeometry,
): Promise<void> {
  if (platform() !== "windows") {
    return;
  }
  await invoke("plugin:window-integration|update_snap_geometry", {...geometry});
}

export async function updateVibrancy(dark: boolean): Promise<void> {
  if (platform() !== "windows") {
    return;
  }
  await invoke("plugin:window-integration|update_vibrancy", {dark});
}
