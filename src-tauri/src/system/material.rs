//! Window material (DOIT.md §9).
//!
//! Windows 11 (build 22000+) gets native Mica behind the sidebar and title bar.
//! Everything else — Windows 10, macOS, Linux, or Mica turned off — gets the
//! solid-color fallback, painted entirely by CSS.
//!
//! The window is only created transparent when Mica is supported (see
//! `build_main_window`), so older systems keep a normal opaque window with its
//! usual shadow and no risk of see-through areas.

use serde::Serialize;
use tauri::window::{Effect, EffectsBuilder};
use tauri::{Theme, WebviewWindow};

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WindowMaterial {
    Mica,
    Solid,
}

/// True on Windows 11 and later, where DWM provides the Mica backdrop.
pub fn supports_mica() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_version::OsVersion::current().build >= 22_000
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

/// Apply the material for the given theme. Called once at startup (with the
/// system theme) and again by the frontend whenever the app theme changes, so
/// Mica's light/dark tint always matches the app — not just the OS setting.
pub fn apply_material(window: &WebviewWindow, dark: bool, enabled: bool) -> WindowMaterial {
    let _ = window.set_theme(Some(if dark { Theme::Dark } else { Theme::Light }));

    if !supports_mica() {
        return WindowMaterial::Solid;
    }

    if !enabled {
        // The window is transparent; the frontend paints a solid background.
        let _ = window.set_effects(None);
        return WindowMaterial::Solid;
    }

    let effect = if dark {
        Effect::MicaDark
    } else {
        Effect::MicaLight
    };
    match window.set_effects(EffectsBuilder::new().effect(effect).build()) {
        Ok(()) => WindowMaterial::Mica,
        Err(error) => {
            eprintln!("Failed to apply Mica: {}", error);
            WindowMaterial::Solid
        }
    }
}

/// Frontend command: `invoke('window_apply_material', { dark, enabled })`.
/// Returns `"mica"` or `"solid"` so the UI can style the sidebar accordingly.
#[tauri::command]
pub fn window_apply_material(
    window: WebviewWindow,
    dark: bool,
    enabled: Option<bool>,
) -> WindowMaterial {
    apply_material(&window, dark, enabled.unwrap_or(true))
}

/// Create the main window from its `tauri.conf.json` entry (`"create": false`).
/// Transparent only where Mica can fill it; Mica is applied immediately with the
/// system theme so there is no white flash before the frontend loads.
/// `visible: false` creates the window hidden (start-up in the tray); show it later.
pub fn build_main_window(app: &tauri::App, visible: bool) -> tauri::Result<WebviewWindow> {
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "main")
        .cloned()
        .expect("tauri.conf.json must define the \"main\" window");

    #[allow(unused_mut)]
    let mut builder =
        tauri::WebviewWindowBuilder::from_config(app.handle(), &config)?.visible(visible);

    #[cfg(not(target_os = "macos"))]
    if supports_mica() {
        builder = builder.transparent(true);
    }

    let window = builder.build()?;
    let dark = matches!(window.theme(), Ok(Theme::Dark));
    apply_material(&window, dark, true);
    Ok(window)
}

/// Frontend command: whether Mica is available here (Windows 11+), so Settings
/// can explain why "Sidebar transparency" has no effect elsewhere.
#[tauri::command]
pub fn window_supports_mica() -> bool {
    supports_mica()
}
