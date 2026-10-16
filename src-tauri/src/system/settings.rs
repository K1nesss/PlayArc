use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

use tauri::AppHandle;

const APP_DATA_DIR_NAME: &str = "PlayArc";
const SETTINGS_FILE_NAME: &str = "settings.json";
const DEFAULT_IDLE_SCAN_INTERVAL_SECONDS: u32 = 15;
const DEFAULT_ACTIVE_SCAN_INTERVAL_SECONDS: u32 = 3;

static PROCESS_SCAN_IDLE_INTERVAL_SECONDS: AtomicU32 =
    AtomicU32::new(DEFAULT_IDLE_SCAN_INTERVAL_SECONDS);
static PROCESS_SCAN_ACTIVE_INTERVAL_SECONDS: AtomicU32 =
    AtomicU32::new(DEFAULT_ACTIVE_SCAN_INTERVAL_SECONDS);
static SETTINGS_RUNTIME_GENERATION: AtomicU64 = AtomicU64::new(0);
static KEEP_RUNNING_IN_TRAY: AtomicBool = AtomicBool::new(true);
static UI_LANGUAGE_ENGLISH: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct AppSettings {
    pub theme_mode: String,
    pub language: String,
    pub api_key: String,
    pub steam64_id: String,
    pub launch_on_startup: bool,
    pub minimize_to_tray: bool,
    pub auto_start_monitor: bool,
    pub dashboard_refresh_seconds: u32,
    pub timeline_page_size: u32,
    pub analytics_default_range: u32,
    pub animations_enabled: bool,
    pub idle_scan_interval_seconds: u32,
    pub active_scan_interval_seconds: u32,
    // Redesign (DOIT.md §8). Fields must exist here or saves drop them.
    pub sidebar_transparency: bool,
    pub compact_game_list: bool,
    pub steam_auto_sync: bool,
    pub steam_sync_interval_minutes: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme_mode: "system".to_string(),
            language: "zh-CN".to_string(),
            api_key: String::new(),
            steam64_id: String::new(),
            launch_on_startup: false,
            minimize_to_tray: true,
            auto_start_monitor: true,
            dashboard_refresh_seconds: 5,
            timeline_page_size: 80,
            analytics_default_range: 7,
            animations_enabled: true,
            idle_scan_interval_seconds: DEFAULT_IDLE_SCAN_INTERVAL_SECONDS,
            active_scan_interval_seconds: DEFAULT_ACTIVE_SCAN_INTERVAL_SECONDS,
            sidebar_transparency: true,
            compact_game_list: false,
            steam_auto_sync: true,
            steam_sync_interval_minutes: 60,
        }
    }
}

#[tauri::command]
pub fn load_app_settings() -> Result<AppSettings, String> {
    let path = settings_path()?;
    if !path.exists() {
        let settings = AppSettings::default();
        apply_runtime_settings(&settings);
        return Ok(settings);
    }

    let payload = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read settings {}: {}", path.display(), e))?;
    let parsed: AppSettings = serde_json::from_str(&payload)
        .map_err(|e| format!("Failed to parse settings {}: {}", path.display(), e))?;

    let settings = normalize_settings(parsed);
    apply_runtime_settings(&settings);
    Ok(settings)
}

#[tauri::command]
pub fn save_app_settings(app: AppHandle, settings: AppSettings) -> Result<AppSettings, String> {
    let settings = normalize_settings(settings);
    let path = settings_path()?;
    let payload = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    fs::write(&path, payload)
        .map_err(|e| format!("Failed to write settings {}: {}", path.display(), e))?;
    apply_runtime_settings(&settings);
    crate::system::autostart::apply(&app, settings.launch_on_startup);
    Ok(settings)
}

pub fn hydrate_runtime_settings() -> Result<AppSettings, String> {
    load_app_settings()
}

pub fn process_scan_intervals() -> (u32, u32) {
    (
        PROCESS_SCAN_IDLE_INTERVAL_SECONDS.load(Ordering::Acquire),
        PROCESS_SCAN_ACTIVE_INTERVAL_SECONDS.load(Ordering::Acquire),
    )
}

/// "Keep running in the tray": closing the window hides it instead of quitting.
pub fn keep_running_in_tray() -> bool {
    KEEP_RUNNING_IN_TRAY.load(Ordering::Acquire)
}

/// Language for native UI (tray menu) — the web UI handles its own.
pub fn ui_is_english() -> bool {
    UI_LANGUAGE_ENGLISH.load(Ordering::Acquire)
}

pub fn settings_runtime_generation() -> u64 {
    SETTINGS_RUNTIME_GENERATION.load(Ordering::Acquire)
}

fn normalize_settings(mut settings: AppSettings) -> AppSettings {
    if !matches!(settings.theme_mode.as_str(), "light" | "dark" | "system") {
        settings.theme_mode = AppSettings::default().theme_mode;
    }

    if !matches!(settings.language.as_str(), "zh-CN" | "en-US") {
        settings.language = AppSettings::default().language;
    }

    if !matches!(settings.dashboard_refresh_seconds, 2 | 5 | 10 | 30) {
        settings.dashboard_refresh_seconds = AppSettings::default().dashboard_refresh_seconds;
    }

    if !matches!(settings.timeline_page_size, 50 | 80 | 120 | 200) {
        settings.timeline_page_size = AppSettings::default().timeline_page_size;
    }

    // 0 = all time. "Today" (1) moved to Home, so it maps to 7 days.
    if settings.analytics_default_range == 1 {
        settings.analytics_default_range = 7;
    }
    if !matches!(settings.analytics_default_range, 0 | 7 | 30 | 90 | 365) {
        settings.analytics_default_range = AppSettings::default().analytics_default_range;
    }

    if !matches!(settings.steam_sync_interval_minutes, 30 | 60 | 180 | 360) {
        settings.steam_sync_interval_minutes = AppSettings::default().steam_sync_interval_minutes;
    }

    if !matches!(settings.idle_scan_interval_seconds, 5 | 10 | 15 | 30) {
        settings.idle_scan_interval_seconds = AppSettings::default().idle_scan_interval_seconds;
    }

    if !matches!(settings.active_scan_interval_seconds, 2 | 3 | 5 | 10) {
        settings.active_scan_interval_seconds = AppSettings::default().active_scan_interval_seconds;
    }

    settings.api_key = settings.api_key.trim().to_string();
    settings.steam64_id = settings.steam64_id.trim().to_string();
    settings
}

fn apply_runtime_settings(settings: &AppSettings) {
    PROCESS_SCAN_IDLE_INTERVAL_SECONDS
        .store(settings.idle_scan_interval_seconds, Ordering::Release);
    PROCESS_SCAN_ACTIVE_INTERVAL_SECONDS
        .store(settings.active_scan_interval_seconds, Ordering::Release);
    KEEP_RUNNING_IN_TRAY.store(settings.minimize_to_tray, Ordering::Release);
    UI_LANGUAGE_ENGLISH.store(settings.language == "en-US", Ordering::Release);
    SETTINGS_RUNTIME_GENERATION.fetch_add(1, Ordering::Release);
}

fn settings_path() -> Result<PathBuf, String> {
    let base_dir = resolve_app_data_dir()?;
    let app_dir = base_dir.join(APP_DATA_DIR_NAME);
    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;
    Ok(app_dir.join(SETTINGS_FILE_NAME))
}

fn resolve_app_data_dir() -> Result<PathBuf, String> {
    if cfg!(target_os = "windows") {
        if let Some(local_app_data) = env::var_os("LOCALAPPDATA") {
            return Ok(PathBuf::from(local_app_data));
        }
        if let Some(app_data) = env::var_os("APPDATA") {
            return Ok(PathBuf::from(app_data));
        }
    }

    if cfg!(target_os = "macos") {
        if let Some(home) = env::var_os("HOME") {
            return Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support"));
        }
    }

    if let Some(home) = env::var_os("HOME") {
        return Ok(PathBuf::from(home).join(".local").join("share"));
    }

    env::current_dir().map_err(|e| format!("Failed to resolve current directory: {}", e))
}
