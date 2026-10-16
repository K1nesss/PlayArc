use chrono::Local;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const APP_DATA_DIR_NAME: &str = "PlayArc";
const LOG_RETENTION_DAYS: u64 = 7;
const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

#[tauri::command]
pub fn open_app_log_file() -> Result<String, String> {
    cleanup_old_logs_once_per_day()?;

    let log_path = today_log_path()?;
    if !log_path.exists() {
        fs::write(&log_path, "").map_err(|e| format!("Failed to create log file: {}", e))?;
    }

    open_path(&log_path)?;
    Ok(log_path.to_string_lossy().to_string())
}

pub fn log_info(message: impl AsRef<str>) {
    append_log("INFO", "GENERAL", message.as_ref());
}

pub fn log_warn(message: impl AsRef<str>) {
    append_log("WARNING", "GENERAL", message.as_ref());
}

pub fn log_error(message: impl AsRef<str>) {
    append_log("ERROR", "GENERAL", message.as_ref());
}

pub fn log_info_event(event: impl AsRef<str>, message: impl AsRef<str>) {
    append_log("INFO", event.as_ref(), message.as_ref());
}

pub fn log_warn_event(event: impl AsRef<str>, message: impl AsRef<str>) {
    append_log("WARNING", event.as_ref(), message.as_ref());
}

pub fn log_error_event(event: impl AsRef<str>, message: impl AsRef<str>) {
    append_log("ERROR", event.as_ref(), message.as_ref());
}

pub fn cleanup_old_logs_once_per_day() -> Result<(), String> {
    let log_dir = get_log_dir()?;
    let today = current_unix_day();
    let marker_path = log_dir.join(".last-cleanup-day");

    if let Ok(marker) = fs::read_to_string(&marker_path) {
        if marker.trim() == today.to_string() {
            return Ok(());
        }
    }

    cleanup_old_logs(&log_dir)?;
    fs::write(marker_path, today.to_string())
        .map_err(|e| format!("Failed to write log cleanup marker: {}", e))?;
    Ok(())
}

fn append_log(level: &str, event: &str, message: &str) {
    if let Err(error) = cleanup_old_logs_once_per_day() {
        eprintln!("Failed to cleanup app logs: {}", error);
    }

    let path = match today_log_path() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Failed to resolve app log path: {}", error);
            return;
        }
    };

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f %:z");
    let sanitized = message.replace('\r', "\\r").replace('\n', "\\n");
    let sanitized_event = event.replace(['\r', '\n'], "_");
    let line = format!(
        "[{}][{}][{}] {}\n",
        timestamp, level, sanitized_event, sanitized
    );

    match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(mut file) => {
            if let Err(error) = file.write_all(line.as_bytes()) {
                eprintln!("Failed to write app log: {}", error);
            }
        }
        Err(error) => eprintln!("Failed to open app log {}: {}", path.display(), error),
    }
}

fn cleanup_old_logs(log_dir: &PathBuf) -> Result<(), String> {
    if !log_dir.exists() {
        return Ok(());
    }

    let now = SystemTime::now();
    let max_age = std::time::Duration::from_secs(LOG_RETENTION_DAYS * SECONDS_PER_DAY);

    for entry in
        fs::read_dir(log_dir).map_err(|e| format!("Failed to read log directory: {}", e))?
    {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("log") {
            continue;
        }

        let metadata = match fs::metadata(&path) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };

        let modified = match metadata.modified() {
            Ok(modified) => modified,
            Err(_) => continue,
        };

        if now.duration_since(modified).unwrap_or_default() > max_age {
            fs::remove_file(&path)
                .map_err(|e| format!("Failed to remove old log {}: {}", path.display(), e))?;
        }
    }

    Ok(())
}

fn open_path(path: &PathBuf) -> Result<(), String> {
    if cfg!(target_os = "windows") {
        Command::new("notepad.exe")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open log file: {}", e))?;
        return Ok(());
    }

    if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open log file: {}", e))?;
        return Ok(());
    }

    Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map_err(|e| format!("Failed to open log file: {}", e))?;
    Ok(())
}

fn today_log_path() -> Result<PathBuf, String> {
    Ok(get_log_dir()?.join(format!("playarc-{}.log", current_unix_day())))
}

fn get_log_dir() -> Result<PathBuf, String> {
    let base_dir = resolve_app_data_dir()?;
    let log_dir = base_dir.join(APP_DATA_DIR_NAME).join("logs");
    fs::create_dir_all(&log_dir).map_err(|e| format!("Failed to create log directory: {}", e))?;
    Ok(log_dir)
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

fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn current_unix_day() -> u64 {
    current_unix_seconds() / SECONDS_PER_DAY
}
