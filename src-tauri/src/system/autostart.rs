//! "Launch at startup" (Settings › General), via tauri-plugin-autostart.
//!
//! On Windows this is the per-user `Run` registry entry. Launches from it carry
//! `--autostart`, so the app can start quietly in the tray.

use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use crate::system::logging::log_warn;

pub const AUTOSTART_ARG: &str = "--autostart";

/// Make the OS entry match the setting. Enabling always rewrites the entry so it
/// points at the current executable (e.g. after an update or the Velaris rename).
pub fn apply(app: &AppHandle, enabled: bool) {
    let manager = app.autolaunch();
    let result = if enabled {
        manager.enable()
    } else if manager.is_enabled().unwrap_or(false) {
        manager.disable()
    } else {
        return;
    };
    if let Err(error) = result {
        log_warn(format!("Failed to update launch at startup: {}", error));
    }
}

/// Whether this process was started by the OS at sign-in.
pub fn launched_at_startup() -> bool {
    std::env::args().any(|arg| arg == AUTOSTART_ARG)
}
