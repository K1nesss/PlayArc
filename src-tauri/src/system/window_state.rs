use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use tauri::{LogicalPosition, LogicalSize, Monitor, Position, Size, WebviewWindow, Window};

const APP_DATA_DIR_NAME: &str = "PlayArc";
const STATE_FILE_NAME: &str = "window-state.json";
/// DOIT.md §12 minimum window size. Lowered automatically on screens whose work
/// area is smaller (e.g. 1366×768 at 125–150% scaling), so the window always fits.
const MIN_WIDTH: f64 = 960.0;
const MIN_HEIGHT: f64 = 640.0;
const SCREEN_MARGIN: f64 = 24.0;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

/// The minimum size for a monitor: §12's 960×640, or the work area if smaller.
pub fn min_size_for_monitor(monitor: &Monitor) -> LogicalSize<f64> {
    let work = logical_work_area(monitor);
    LogicalSize::new(
        MIN_WIDTH.min((work.width - SCREEN_MARGIN).max(360.0)),
        MIN_HEIGHT.min((work.height - SCREEN_MARGIN).max(320.0)),
    )
}

pub fn restore_webview_window_state(window: &WebviewWindow) {
    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten());
    if let Some(monitor) = monitor {
        let _ = window.set_min_size(Some(Size::Logical(min_size_for_monitor(&monitor))));
    }

    if let Some(state) = read_window_state() {
        apply_webview_window_state(window, state);
    } else {
        ensure_webview_window_fits_current_monitor(window);
    }
}

pub fn capture_window_state(window: &Window) -> Option<WindowState> {
    let maximized = window.is_maximized().unwrap_or(false);
    let position = window.outer_position().ok()?;
    let size = window.inner_size().ok()?;
    let scale_factor = window.scale_factor().unwrap_or(1.0).max(1.0);

    Some(WindowState {
        x: (position.x as f64 / scale_factor).round() as i32,
        y: (position.y as f64 / scale_factor).round() as i32,
        width: (size.width as f64 / scale_factor).round() as u32,
        height: (size.height as f64 / scale_factor).round() as u32,
        maximized,
    })
}

pub fn save_window_state(state: &WindowState) {
    let path = match window_state_path() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Failed to resolve window state path: {}", error);
            return;
        }
    };

    match serde_json::to_string_pretty(state) {
        Ok(payload) => {
            if let Err(error) = fs::write(&path, payload) {
                eprintln!("Failed to write window state {}: {}", path.display(), error);
            }
        }
        Err(error) => eprintln!("Failed to serialize window state: {}", error),
    }
}

pub fn current_monitor_key(window: &Window) -> Option<String> {
    window
        .current_monitor()
        .ok()
        .flatten()
        .map(|monitor| monitor_key(&monitor))
}

pub fn ensure_window_fits_current_monitor(window: &Window) {
    if window.is_fullscreen().unwrap_or(false) {
        return;
    }

    let monitor = match window.current_monitor() {
        Ok(Some(monitor)) => monitor,
        _ => return,
    };

    let _ = window.set_min_size(Some(Size::Logical(min_size_for_monitor(&monitor))));

    let maximized = window.is_maximized().unwrap_or(false);
    if maximized {
        return;
    }

    if let Some(state) = capture_window_state(window) {
        let next = fit_state_to_monitor(state, &monitor);
        let _ = window.set_size(Size::Logical(LogicalSize::new(
            next.width as f64,
            next.height as f64,
        )));
        let _ = window.set_position(Position::Logical(LogicalPosition::new(
            next.x as f64,
            next.y as f64,
        )));
    }
}

fn read_window_state() -> Option<WindowState> {
    let path = window_state_path().ok()?;
    let payload = fs::read_to_string(path).ok()?;
    serde_json::from_str::<WindowState>(&payload).ok()
}

fn apply_webview_window_state(window: &WebviewWindow, state: WindowState) {
    if window.is_fullscreen().unwrap_or(false) {
        return;
    }

    let monitor = choose_monitor_for_state(
        window.available_monitors().ok(),
        window.current_monitor().ok().flatten(),
        &state,
    );

    let Some(monitor) = monitor else {
        return;
    };

    let maximized = state.maximized;
    let next = fit_state_to_monitor(state, &monitor);
    let _ = window.unmaximize();
    let _ = window.set_size(Size::Logical(LogicalSize::new(
        next.width as f64,
        next.height as f64,
    )));
    let _ = window.set_position(Position::Logical(LogicalPosition::new(
        next.x as f64,
        next.y as f64,
    )));

    if maximized {
        let _ = window.maximize();
    }
}

fn ensure_webview_window_fits_current_monitor(window: &WebviewWindow) {
    if window.is_fullscreen().unwrap_or(false) || window.is_maximized().unwrap_or(false) {
        return;
    }

    let monitor = match window.current_monitor() {
        Ok(Some(monitor)) => monitor,
        Ok(None) => match window.primary_monitor() {
            Ok(Some(monitor)) => monitor,
            _ => return,
        },
        Err(_) => return,
    };

    let scale_factor = window.scale_factor().unwrap_or(1.0).max(1.0);
    let position = match window.outer_position() {
        Ok(position) => position,
        Err(_) => return,
    };
    let size = match window.inner_size() {
        Ok(size) => size,
        Err(_) => return,
    };

    let state = WindowState {
        x: (position.x as f64 / scale_factor).round() as i32,
        y: (position.y as f64 / scale_factor).round() as i32,
        width: (size.width as f64 / scale_factor).round() as u32,
        height: (size.height as f64 / scale_factor).round() as u32,
        maximized: false,
    };
    let next = fit_state_to_monitor(state, &monitor);

    let _ = window.set_size(Size::Logical(LogicalSize::new(
        next.width as f64,
        next.height as f64,
    )));
    let _ = window.set_position(Position::Logical(LogicalPosition::new(
        next.x as f64,
        next.y as f64,
    )));
}

fn choose_monitor_for_state(
    monitors: Option<Vec<Monitor>>,
    fallback: Option<Monitor>,
    state: &WindowState,
) -> Option<Monitor> {
    let monitors = monitors.unwrap_or_default();
    let center_x = state.x as f64 + state.width as f64 / 2.0;
    let center_y = state.y as f64 + state.height as f64 / 2.0;

    monitors
        .iter()
        .find(|monitor| {
            let work = logical_work_area(monitor);
            center_x >= work.x
                && center_x <= work.x + work.width
                && center_y >= work.y
                && center_y <= work.y + work.height
        })
        .cloned()
        .or(fallback)
        .or_else(|| monitors.first().cloned())
}

fn fit_state_to_monitor(state: WindowState, monitor: &Monitor) -> WindowState {
    let work = logical_work_area(monitor);
    let max_width = (work.width - SCREEN_MARGIN).max(360.0);
    let max_height = (work.height - SCREEN_MARGIN).max(320.0);
    let min_width = MIN_WIDTH.min(max_width);
    let min_height = MIN_HEIGHT.min(max_height);
    let width = (state.width as f64).clamp(min_width, max_width);
    let height = (state.height as f64).clamp(min_height, max_height);
    let max_x = work.x + work.width - width;
    let max_y = work.y + work.height - height;
    let x = (state.x as f64).clamp(work.x, max_x.max(work.x));
    let y = (state.y as f64).clamp(work.y, max_y.max(work.y));

    WindowState {
        x: x.round() as i32,
        y: y.round() as i32,
        width: width.round() as u32,
        height: height.round() as u32,
        maximized: state.maximized,
    }
}

struct LogicalWorkArea {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

fn logical_work_area(monitor: &Monitor) -> LogicalWorkArea {
    let scale_factor = monitor.scale_factor().max(1.0);
    let work = monitor.work_area();

    LogicalWorkArea {
        x: work.position.x as f64 / scale_factor,
        y: work.position.y as f64 / scale_factor,
        width: work.size.width as f64 / scale_factor,
        height: work.size.height as f64 / scale_factor,
    }
}

fn monitor_key(monitor: &Monitor) -> String {
    let position = monitor.position();
    let size = monitor.size();
    format!(
        "{}:{}:{}:{}:{:.3}",
        position.x,
        position.y,
        size.width,
        size.height,
        monitor.scale_factor()
    )
}

fn window_state_path() -> Result<PathBuf, String> {
    let base_dir = resolve_app_data_dir()?;
    let app_dir = base_dir.join(APP_DATA_DIR_NAME);
    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;
    Ok(app_dir.join(STATE_FILE_NAME))
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
