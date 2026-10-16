//! System tray icon and what its right click opens.
//!
//! Two styles, chosen in Settings → General (`tray_style`):
//! - `"panel"` (default): a small borderless window (`/tray` route) drawn with
//!   the app's own theme, placed next to the tray icon.
//! - `"native"`: the system menu. Also the fallback if the panel window can't be
//!   created.
//!
//! Both are a short list of actions — no status or other information — handled
//! by [`run_action`]. Steam sync and opening Settings are done by the main
//! window's UI: the tray emits `tray-sync` / `tray-navigate`, which
//! `+layout.svelte` acts on.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{
    App, AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, Rect, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder, Window, WindowEvent, Wry,
};

use crate::system::logging::log_warn;
use crate::system::settings::{load_app_settings, ui_is_english, update_app_settings};

const TRAY_ID: &str = "main";
/// Window label of the tray panel (also listed in `capabilities/default.json`).
pub const PANEL_LABEL: &str = "tray-panel";
/// Logical width of the panel window, including the transparent margin the
/// menu's shadow is drawn in (`src/routes/tray/+page.svelte`).
const PANEL_WIDTH: f64 = 232.0;
const PANEL_INITIAL_HEIGHT: f64 = 240.0;

/// Whether right click opens the panel (true) or the native menu (false).
static PANEL_MODE: AtomicBool = AtomicBool::new(false);
/// When the panel last hid itself because it lost focus. Clicking the tray icon
/// while the panel is open first blurs it; this stops that same click reopening it.
static PANEL_HIDDEN_AT: Mutex<Option<Instant>> = Mutex::new(None);

fn text(zh: &'static str, en: &'static str) -> &'static str {
    if ui_is_english() {
        en
    } else {
        zh
    }
}

/* ------------------------------------------------------------------ */
/* Native menu                                                        */
/* ------------------------------------------------------------------ */

struct NativeMenu {
    menu: Menu<Wry>,
    open: MenuItem<Wry>,
    sync: MenuItem<Wry>,
    autostart: CheckMenuItem<Wry>,
    settings: MenuItem<Wry>,
    quit: MenuItem<Wry>,
}

impl NativeMenu {
    fn build(app: &App) -> tauri::Result<Self> {
        let separator = || PredefinedMenuItem::separator(app);
        let open = MenuItem::with_id(app, "tray_open", "", true, None::<&str>)?;
        let sync = MenuItem::with_id(app, "tray_sync", "", true, None::<&str>)?;
        let autostart =
            CheckMenuItem::with_id(app, "tray_autostart", "", true, false, None::<&str>)?;
        let settings = MenuItem::with_id(app, "tray_settings", "", true, None::<&str>)?;
        let quit = MenuItem::with_id(app, "tray_quit", "", true, None::<&str>)?;

        let menu = Menu::with_items(
            app,
            &[
                &open,
                &sync,
                &separator()?,
                &autostart,
                &settings,
                &separator()?,
                &quit,
            ],
        )?;

        Ok(Self {
            menu,
            open,
            sync,
            autostart,
            settings,
            quit,
        })
    }

    /// Bring labels (language) and states up to date. Called right before the menu opens.
    fn refresh(&self) {
        let _ = self.open.set_text(text("显示主窗口", "Show window"));
        let _ = self
            .sync
            .set_text(text("同步 Steam 数据", "Sync Steam data"));
        let _ = self
            .autostart
            .set_text(text("开机启动", "Launch at startup"));
        let _ = self.settings.set_text(text("设置", "Settings"));
        let _ = self.quit.set_text(text("退出", "Quit"));

        let settings = load_app_settings().unwrap_or_default();
        let _ = self.autostart.set_checked(settings.launch_on_startup);
        // Syncing needs a Steam API key and Steam ID.
        let _ = self
            .sync
            .set_enabled(!settings.api_key.is_empty() && !settings.steam64_id.is_empty());
    }
}

/* ------------------------------------------------------------------ */
/* Actions (shared by the menu and the panel)                         */
/* ------------------------------------------------------------------ */

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Show the window and ask the UI to open a page (only `settings` from the tray).
fn navigate(app: &AppHandle, page: &str) {
    show_main_window(app);
    let _ = app.emit("tray-navigate", page);
}

/// `open`, `settings`, `sync`, `autostart`, `quit`, `hide` (panel only).
fn run_action(app: &AppHandle, action: &str) {
    // Everything except toggles and sync leaves the panel: hide it first.
    if !matches!(action, "sync" | "autostart") {
        hide_panel(app);
    }
    match action {
        "hide" => {}
        "open" => show_main_window(app),
        "settings" => navigate(app, "settings"),
        // The main window's UI runs the sync (it owns progress and error states)
        // and answers with `tray-sync-result`.
        "sync" => {
            let _ = app.emit("tray-sync", ());
        }
        "autostart" => {
            match update_app_settings(app, |settings| {
                settings.launch_on_startup = !settings.launch_on_startup
            }) {
                // Keep an open Settings page (and the panel) in step.
                Ok(saved) => {
                    let _ = app.emit("settings-updated", saved);
                }
                Err(error) => log_warn(format!(
                    "Tray: couldn’t change launch at startup: {}",
                    error
                )),
            }
        }
        "quit" => app.exit(0),
        _ => {}
    }
}

/// Panel command: `invoke('tray_panel_action', { action })`.
#[tauri::command]
pub fn tray_panel_action(app: AppHandle, action: String) {
    run_action(&app, &action);
}

/// Panel command: the page reports its height (logical px) so the window fits it.
#[tauri::command]
pub fn tray_panel_resize(window: WebviewWindow, height: f64) {
    if window.label() == PANEL_LABEL && height.is_finite() {
        let _ = window.set_size(LogicalSize::new(PANEL_WIDTH, height.clamp(120.0, 720.0)));
    }
}

/* ------------------------------------------------------------------ */
/* Panel window                                                       */
/* ------------------------------------------------------------------ */

fn build_panel(app: &App) -> tauri::Result<WebviewWindow> {
    #[allow(unused_mut)]
    let mut builder = WebviewWindowBuilder::new(app, PANEL_LABEL, WebviewUrl::App("tray".into()))
        .title("PlayArc")
        .inner_size(PANEL_WIDTH, PANEL_INITIAL_HEIGHT)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .decorations(false)
        .shadow(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .focused(false)
        .visible(false);
    // The card's rounded corners and shadow are drawn by the page on a transparent window.
    #[cfg(not(target_os = "macos"))]
    {
        builder = builder.transparent(true);
    }
    builder.build()
}

fn hide_panel(app: &AppHandle) {
    if let Some(panel) = app.get_webview_window(PANEL_LABEL) {
        if panel.is_visible().unwrap_or(false) {
            let _ = panel.hide();
        }
    }
}

/// Window events for the panel (routed here from `main.rs`): it closes when it
/// loses focus, and Alt+F4 hides it instead of destroying it.
pub fn on_panel_event(window: &Window, event: &WindowEvent) {
    match event {
        WindowEvent::Focused(false) => {
            if window.is_visible().unwrap_or(false) {
                let _ = window.hide();
                if let Ok(mut hidden_at) = PANEL_HIDDEN_AT.lock() {
                    *hidden_at = Some(Instant::now());
                }
            }
        }
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            let _ = window.hide();
        }
        _ => {}
    }
}

/// Axis-aligned rectangle in physical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Area {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Area {
    fn right(&self) -> f64 {
        self.x + self.w
    }
    fn bottom(&self) -> f64 {
        self.y + self.h
    }
}

/// Where the panel goes: against the taskbar, centred on the tray icon, like
/// Windows' own flyouts. The taskbar edge is wherever the icon sits outside the
/// work area. If the icon is inside it (e.g. the hidden-icons overflow), the
/// panel opens above the cursor (below if there's no room). Always kept on screen.
fn place_panel(work: Area, icon: Area, cursor: (f64, f64), panel: (f64, f64)) -> (f64, f64) {
    let (pw, ph) = panel;
    let (cx, cy) = (icon.x + icon.w / 2.0, icon.y + icon.h / 2.0);
    let known_icon = icon.w > 0.0 && icon.h > 0.0;

    let (x, y) = if known_icon && icon.y >= work.bottom() - 1.0 {
        (cx - pw / 2.0, work.bottom() - ph) // taskbar at the bottom
    } else if known_icon && icon.bottom() <= work.y + 1.0 {
        (cx - pw / 2.0, work.y) // top
    } else if known_icon && icon.right() <= work.x + 1.0 {
        (work.x, cy - ph / 2.0) // left
    } else if known_icon && icon.x >= work.right() - 1.0 {
        (work.right() - pw, cy - ph / 2.0) // right
    } else {
        let above = cursor.1 - ph;
        (
            cursor.0 - pw / 2.0,
            if above >= work.y { above } else { cursor.1 },
        )
    };

    let clamp = |value: f64, min: f64, max: f64| value.min(max).max(min);
    (
        clamp(x, work.x, work.right() - pw).round(),
        clamp(y, work.y, work.bottom() - ph).round(),
    )
}

fn toggle_panel(app: &AppHandle, rect: Rect, cursor: PhysicalPosition<f64>) {
    let Some(panel) = app.get_webview_window(PANEL_LABEL) else {
        return;
    };
    if panel.is_visible().unwrap_or(false) {
        let _ = panel.hide();
        return;
    }
    let just_hidden = PANEL_HIDDEN_AT
        .lock()
        .ok()
        .and_then(|hidden_at| *hidden_at)
        .is_some_and(|at| at.elapsed() < Duration::from_millis(300));
    if just_hidden {
        return; // This click is what closed it.
    }

    let monitor = app
        .monitor_from_point(cursor.x, cursor.y)
        .ok()
        .flatten()
        .or_else(|| app.primary_monitor().ok().flatten());
    if let Some(monitor) = monitor {
        let scale = monitor.scale_factor();
        let work = monitor.work_area();
        let work = Area {
            x: work.position.x as f64,
            y: work.position.y as f64,
            w: work.size.width as f64,
            h: work.size.height as f64,
        };
        let icon_position = rect.position.to_physical::<f64>(scale);
        let icon_size = rect.size.to_physical::<f64>(scale);
        let icon = Area {
            x: icon_position.x,
            y: icon_position.y,
            w: icon_size.width,
            h: icon_size.height,
        };
        // Size at the target monitor's scale (the panel may be on another monitor now).
        let size = panel
            .inner_size()
            .ok()
            .map(|size| size.to_logical::<f64>(panel.scale_factor().unwrap_or(scale)))
            .map(|size| (size.width * scale, size.height * scale))
            .unwrap_or((PANEL_WIDTH * scale, PANEL_INITIAL_HEIGHT * scale));
        let (x, y) = place_panel(work, icon, (cursor.x, cursor.y), size);
        let _ = panel.set_position(PhysicalPosition::new(x, y));
    }

    // The page refreshes its status, settings and theme before it's seen.
    let _ = panel.emit_to(PANEL_LABEL, "tray-panel-open", ());
    let _ = panel.show();
    let _ = panel.set_focus();
}

/* ------------------------------------------------------------------ */
/* Tray icon                                                          */
/* ------------------------------------------------------------------ */

/// Switch between the panel and the native menu (`tray_style` setting).
pub fn apply_style(app: &AppHandle, style: &str) {
    let panel_available = app.get_webview_window(PANEL_LABEL).is_some();
    let use_panel = style != "native" && panel_available;
    PANEL_MODE.store(use_panel, Ordering::Release);

    let (Some(tray), Some(native)) = (app.tray_by_id(TRAY_ID), app.try_state::<NativeMenu>())
    else {
        return;
    };
    // With a menu attached, the system opens it on right click; without one, we open the panel.
    let result = if use_panel {
        hide_panel(app);
        tray.set_menu(None::<Menu<Wry>>)
    } else {
        hide_panel(app);
        tray.set_menu(Some(native.menu.clone()))
    };
    if let Err(error) = result {
        log_warn(format!("Tray: couldn’t switch menu style: {}", error));
    }
}

pub fn create(app: &App) -> tauri::Result<()> {
    let native = NativeMenu::build(app)?;
    native.refresh();
    app.manage(native);

    if let Err(error) = build_panel(app) {
        // Fall back to the native menu for this run.
        log_warn(format!(
            "Tray panel unavailable, using the system menu: {}",
            error
        ));
    }

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("PlayArc")
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            let app = tray.app_handle();
            match event {
                // Left click opens the app.
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } => {
                    hide_panel(app);
                    show_main_window(app);
                }
                // Right click: native menu — refresh it before the system shows it.
                TrayIconEvent::Click {
                    button: MouseButton::Right,
                    button_state: MouseButtonState::Down,
                    ..
                } if !PANEL_MODE.load(Ordering::Acquire) => {
                    if let Some(native) = app.try_state::<NativeMenu>() {
                        native.refresh();
                    }
                }
                // Right click: panel — opens on release, like the system menu.
                TrayIconEvent::Click {
                    button: MouseButton::Right,
                    button_state: MouseButtonState::Up,
                    rect,
                    position,
                    ..
                } if PANEL_MODE.load(Ordering::Acquire) => toggle_panel(app, rect, position),
                _ => {}
            }
        })
        .on_menu_event(|app, event| {
            if let Some(action) = event.id.as_ref().strip_prefix("tray_") {
                run_action(app, action);
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }
    builder.build(app)?;

    let style = load_app_settings().unwrap_or_default().tray_style;
    apply_style(app.handle(), &style);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{place_panel, Area};

    const SCREEN: Area = Area {
        x: 0.0,
        y: 0.0,
        w: 1920.0,
        h: 1080.0,
    };
    const PANEL: (f64, f64) = (320.0, 400.0);

    fn icon(x: f64, y: f64) -> Area {
        Area {
            x,
            y,
            w: 24.0,
            h: 32.0,
        }
    }

    #[test]
    fn bottom_taskbar_sits_on_it_centred_on_the_icon() {
        let work = Area {
            h: 1032.0,
            ..SCREEN
        };
        let at = place_panel(work, icon(1700.0, 1040.0), (1710.0, 1050.0), PANEL);
        assert_eq!(at, (1552.0, 632.0));
    }

    #[test]
    fn icon_near_the_corner_stays_on_screen() {
        let work = Area {
            h: 1032.0,
            ..SCREEN
        };
        let at = place_panel(work, icon(1880.0, 1040.0), (1890.0, 1050.0), PANEL);
        assert_eq!(at, (1600.0, 632.0));
    }

    #[test]
    fn top_and_side_taskbars() {
        let top = Area {
            y: 48.0,
            h: 1032.0,
            ..SCREEN
        };
        assert_eq!(
            place_panel(top, icon(1700.0, 8.0), (1710.0, 20.0), PANEL),
            (1552.0, 48.0)
        );

        let left = Area {
            x: 60.0,
            w: 1860.0,
            ..SCREEN
        };
        assert_eq!(
            place_panel(left, icon(18.0, 900.0), (30.0, 910.0), PANEL),
            (60.0, 680.0)
        );

        let right = Area {
            w: 1860.0,
            ..SCREEN
        };
        assert_eq!(
            place_panel(right, icon(1878.0, 900.0), (1890.0, 910.0), PANEL),
            (1540.0, 680.0)
        );
    }

    #[test]
    fn overflow_or_unknown_icon_uses_the_cursor() {
        let work = Area {
            h: 1032.0,
            ..SCREEN
        };
        // Inside the hidden-icons flyout (within the work area).
        assert_eq!(
            place_panel(work, icon(1600.0, 960.0), (1610.0, 970.0), PANEL),
            (1450.0, 570.0)
        );
        // No icon rect: near the top of the screen there's no room above, so below.
        let none = Area {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
        };
        assert_eq!(
            place_panel(work, none, (300.0, 100.0), PANEL),
            (140.0, 100.0)
        );
    }
}
