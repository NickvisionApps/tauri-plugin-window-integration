use directories::BaseDirs;
use std::{
    collections::BTreeMap,
    sync::{Mutex, PoisonError},
};

const GNOME_ACCENT_COLORS: [(&str, &str); 9] = [
    ("blue", "#3584e4"),
    ("teal", "#2190a4"),
    ("green", "#3a944a"),
    ("yellow", "#c88800"),
    ("orange", "#ed5b00"),
    ("red", "#e62d42"),
    ("pink", "#d56199"),
    ("purple", "#9141ac"),
    ("slate", "#6f8396"),
];
const TITLEBAR_ICON_SIZE: i32 = 16;
const TITLEBAR_ICON_CANDIDATES: [(&str, &[&str]); 4] = [
    ("minimize", &["window-minimize-symbolic", "window-minimize"]),
    ("maximize", &["window-maximize-symbolic", "window-maximize"]),
    ("restore", &["window-restore-symbolic", "window-restore"]),
    ("close", &["window-close-symbolic", "window-close"]),
];

pub fn accent_color() -> Option<String> {
    let gnome = || {
        std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "accent-color"])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| {
                let name = String::from_utf8_lossy(&output.stdout);
                let name = name.trim().trim_matches('\'');
                GNOME_ACCENT_COLORS
                    .iter()
                    .find(|(candidate, _)| *candidate == name)
                    .map(|(_, hex)| hex.to_string())
            })
    };
    let kde = || {
        let contents =
            std::fs::read_to_string(BaseDirs::new()?.config_dir().join("kdeglobals")).ok()?;
        let mut in_section = false;
        for line in contents.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_section = line == "[General]";
                continue;
            }
            if !in_section {
                continue;
            }
            if let Some(value) = line.strip_prefix("AccentColor=") {
                let parts: Vec<&str> = value.split(',').collect();
                if let [r, g, b] = parts[..] {
                    let (r, g, b) = (
                        r.trim().parse::<u8>().ok()?,
                        g.trim().parse::<u8>().ok()?,
                        b.trim().parse::<u8>().ok()?,
                    );
                    return Some(format!("#{r:02x}{g:02x}{b:02x}"));
                }
            }
        }
        None
    };
    if is_kde() {
        kde().or_else(gnome)
    } else {
        gnome().or_else(kde)
    }
}

pub fn button_layout() -> Option<String> {
    use gtk::prelude::*;
    let gtk_layout = || {
        gtk::Settings::default()?
            .gtk_decoration_layout()
            .map(|value| value.to_string())
    };
    let kde_layout = || {
        let contents = std::fs::read_to_string(BaseDirs::new()?.config_dir().join("kwinrc")).ok();
        if contents.is_none() && !is_kde() {
            return None;
        }
        let contents = contents.unwrap_or_default();
        let mut in_section = false;
        let mut custom = false;
        let mut left = None;
        let mut right = None;
        for line in contents.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_section = line == "[org.kde.kdecoration2]";
                continue;
            }
            if !in_section {
                continue;
            }
            if let Some(value) = line.strip_prefix("CustomButtonPositions=") {
                custom = value.trim().eq_ignore_ascii_case("true");
            } else if let Some(value) = line.strip_prefix("ButtonsOnLeft=") {
                left = Some(value.trim().to_string());
            } else if let Some(value) = line.strip_prefix("ButtonsOnRight=") {
                right = Some(value.trim().to_string());
            }
        }
        let (left, right) = if custom {
            (
                left.unwrap_or_else(|| "MS".to_string()),
                right.unwrap_or_else(|| "HIAX".to_string()),
            )
        } else {
            ("MS".to_string(), "HIAX".to_string())
        };
        Some(format!(
            "{}:{}",
            translate_kde_codes(&left),
            translate_kde_codes(&right)
        ))
    };
    if is_kde() {
        kde_layout().or_else(gtk_layout)
    } else {
        gtk_layout().or_else(kde_layout)
    }
}

pub fn titlebar_icons<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
) -> Option<BTreeMap<&'static str, String>> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use gtk::prelude::*;
    static CACHE: Mutex<Option<(String, BTreeMap<&'static str, String>)>> = Mutex::new(None);
    let gtk_window = window.gtk_window().ok()?;
    let theme = gtk::IconTheme::for_screen(&GtkWindowExt::screen(&gtk_window)?)?;
    let style_context = gtk_window.style_context();
    let scale = gtk_window.scale_factor().clamp(1, 4);
    let settings = gtk::Settings::default()?;
    let cache_key = format!(
        "{:?}|{:?}|{}|{}",
        settings.gtk_icon_theme_name(),
        settings.gtk_theme_name(),
        settings.is_gtk_application_prefer_dark_theme(),
        scale
    );
    if let Some((key, icons)) = CACHE
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        && *key == cache_key
    {
        return Some(icons.clone());
    }
    let mut icons = BTreeMap::new();
    for (control, candidates) in TITLEBAR_ICON_CANDIDATES {
        for candidate in candidates {
            let Some(icon) = theme.lookup_icon_for_scale(
                candidate,
                TITLEBAR_ICON_SIZE,
                scale,
                gtk::IconLookupFlags::FORCE_SIZE,
            ) else {
                continue;
            };
            let Ok((pixbuf, _)) = icon.load_symbolic_for_context(&style_context) else {
                continue;
            };
            let Ok(png) = pixbuf.save_to_bufferv("png", &[]) else {
                continue;
            };
            icons.insert(
                control,
                format!("data:image/png;base64,{}", STANDARD.encode(png)),
            );
            break;
        }
    }
    if icons.is_empty() {
        return None;
    }
    *CACHE.lock().unwrap_or_else(PoisonError::into_inner) = Some((cache_key, icons.clone()));
    Some(icons)
}

fn translate_kde_codes(codes: &str) -> String {
    codes
        .chars()
        .filter_map(|code| match code {
            'I' => Some("minimize"),
            'A' => Some("maximize"),
            'X' => Some("close"),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn is_kde() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP").is_ok_and(|desktop| {
        desktop
            .split(':')
            .any(|part| part.eq_ignore_ascii_case("KDE"))
    })
}
