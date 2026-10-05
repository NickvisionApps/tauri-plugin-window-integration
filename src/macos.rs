use objc2::MainThreadMarker;
use objc2_app_kit::{NSColor, NSColorSpace};
use tauri::{AppHandle, Runtime, Window};
use window_vibrancy::{NSVisualEffectMaterial, NSVisualEffectState, apply_vibrancy};

pub async fn accent_color<R: Runtime>(app: &AppHandle<R>) -> Option<String> {
    let (tx, mut rx) = tauri::async_runtime::channel(1);
    app.run_on_main_thread(move || {
        let hex = MainThreadMarker::new().and_then(|_mtm| {
            let color = NSColor::controlAccentColor();
            let srgb = color.colorUsingColorSpace(&NSColorSpace::sRGBColorSpace())?;
            let r = (srgb.redComponent() * 255.0).round() as u8;
            let g = (srgb.greenComponent() * 255.0).round() as u8;
            let b = (srgb.blueComponent() * 255.0).round() as u8;
            Some(format!("#{r:02x}{g:02x}{b:02x}"))
        });
        let _ = tx.try_send(hex);
    })
    .ok()?;
    rx.recv().await?
}

pub fn apply_window_vibrancy<R: Runtime>(window: &Window<R>) {
    let _ = apply_vibrancy(
        window,
        NSVisualEffectMaterial::UnderWindowBackground,
        Some(NSVisualEffectState::FollowsWindowActiveState),
        None,
    );
}
