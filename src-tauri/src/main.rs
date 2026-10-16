// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod core;
mod storage;
mod system;

use api::analytics::analytics_overview;
use api::dashboard::{current_game_name, home_snapshot};
use api::games::{
    clear_play_history, create_manual_game, delete_game, delete_game_session, game_detail,
    games_library, hidden_games_list, hide_game, ignored_games_list, restore_hidden_game,
    restore_ignored_game,
};
use api::steam::{steam_sync_status, sync_all_steam_game_media, sync_steam_owned_game_icons};
use api::timeline::{timeline_days, timeline_months};
use core::file_watcher::start_file_watcher;
use core::process_watcher::start_process_monitor;
use core::steam_scanner::steam_scan_print;
use steamlocate::SteamDir;
use storage::db::{export_database, import_database, init_database, print_database_tables};
use system::controls::{window_close, window_minimize, window_toggle_maximize};
use system::logging::{
    cleanup_old_logs_once_per_day, log_error, log_info, log_warn, open_app_log_file,
};
use system::material::{build_main_window, window_apply_material, window_supports_mica};
use system::settings::{load_app_settings, save_app_settings};
use system::window_state::{
    capture_window_state, current_monitor_key, ensure_window_fits_current_monitor,
    restore_webview_window_state, save_window_state, WindowState,
};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

#[tauri::command]
fn hello() -> String {
    "Hello from Rust".into()
}

/// Tray menu text in the app's language (Settings › General › Language).
fn tray_text(zh: &'static str, en: &'static str) -> &'static str {
    if system::settings::ui_is_english() {
        en
    } else {
        zh
    }
}

fn tray_now_playing_label() -> String {
    match current_game_name() {
        Ok(Some(name)) => format!("{}{}", tray_text("正在游玩：", "Now playing: "), name),
        Ok(None) => tray_text("当前没有在玩游戏", "Not playing anything").to_string(),
        Err(_) => tray_text("无法读取当前游戏", "Couldn’t read the current game").to_string(),
    }
}

fn main() {
    // Before anything opens the data folder (logs, settings, database).
    system::rename_migration::migrate_legacy_data_dir();
    if let Err(e) = cleanup_old_logs_once_per_day() {
        eprintln!("Error cleaning application logs: {}", e);
    }
    log_info("PlayArc starting");
    // 初始化数据库
    if let Err(e) = init_database() {
        println!("Error initializing database: {}", e);
        log_error(format!("Error initializing database: {}", e));
    } else {
        log_info("Database initialized");
    }

    // 程序启动时自动执行 Steam 扫描
    println!("=== Auto Steam Scan on Startup ===");
    log_info("Auto Steam scan on startup started");
    if let Err(e) = steam_scan_print() {
        println!("Error during auto steam scan: {}", e);
        log_error(format!("Error during auto Steam scan: {}", e));
    } else {
        log_info("Auto Steam scan on startup completed");
    }

    // 程序启动时自动启动进程监控服务（常驻）
    println!("=== Starting Process Monitor Service ===");
    if let Err(e) = start_process_monitor() {
        println!("Error starting process monitor: {}", e);
        log_error(format!("Error starting process monitor: {}", e));
    } else {
        log_info("Process monitor service started");
    }

    // 程序启动时启动文件监听服务
    println!("=== Starting File Watcher Service ===");
    if let Ok(steam_dir) = SteamDir::locate() {
        if let Some(steam_path) = steam_dir.path().to_str() {
            start_file_watcher(steam_path.to_string());
            log_info(format!(
                "File watcher started for Steam path: {}",
                steam_path
            ));
        } else {
            println!("Failed to convert Steam path to string");
            log_warn("Failed to convert Steam path to string");
        }
    } else {
        println!("Failed to locate Steam installation");
        log_warn("Failed to locate Steam installation for file watcher");
    }

    let last_monitor_key = std::sync::Arc::new(std::sync::Mutex::new(None::<String>));
    let pending_window_state = std::sync::Arc::new(std::sync::Mutex::new(None::<WindowState>));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![system::autostart::AUTOSTART_ARG]),
        ))
        .setup(|app| {
            let settings = load_app_settings().unwrap_or_default();
            // Keep the OS "launch at startup" entry in sync with the setting.
            system::autostart::apply(app.handle(), settings.launch_on_startup);

            let now_playing_item = MenuItem::with_id(
                app,
                "tray_now_playing",
                tray_now_playing_label(),
                false,
                None::<&str>,
            )?;
            let show_label = tray_text("显示主窗口", "Show PlayArc");
            let quit_label = tray_text("退出", "Quit");

            let show_item = MenuItem::with_id(app, "tray_show", show_label, true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "tray_quit", quit_label, true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&now_playing_item, &show_item, &quit_item])?;

            let icon = app.default_window_icon().cloned();

            let mut tray_builder = TrayIconBuilder::new()
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event({
                    let now_playing_item = now_playing_item.clone();
                    let show_item = show_item.clone();
                    let quit_item = quit_item.clone();
                    move |tray, event| match event {
                        TrayIconEvent::DoubleClick { .. } => {
                            if let Some(window) = tray.app_handle().get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        TrayIconEvent::Click {
                            button: MouseButton::Right,
                            button_state: MouseButtonState::Down,
                            ..
                        } => {
                            // Refresh before the menu opens (language may have changed).
                            let _ = now_playing_item.set_text(tray_now_playing_label());
                            let _ = show_item.set_text(tray_text("显示主窗口", "Show PlayArc"));
                            let _ = quit_item.set_text(tray_text("退出", "Quit"));
                        }
                        _ => {}
                    }
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "tray_show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "tray_quit" => {
                        app.exit(0);
                    }
                    _ => {}
                });

            if let Some(icon) = icon {
                tray_builder = tray_builder.icon(icon);
            }

            let _tray = tray_builder.build(app)?;

            // Started by Windows at sign-in: stay in the tray if the user keeps the app there.
            let start_hidden =
                system::autostart::launched_at_startup() && settings.minimize_to_tray;
            let window = build_main_window(app, !start_hidden)?;
            restore_webview_window_state(&window);

            Ok(())
        })
        .on_window_event({
            let last_monitor_key = last_monitor_key.clone();
            let pending_window_state = pending_window_state.clone();
            move |window, event| match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    if let Some(state) = capture_window_state(window) {
                        save_window_state(&state);
                    } else if let Ok(guard) = pending_window_state.lock() {
                        if let Some(state) = guard.as_ref() {
                            save_window_state(state);
                        }
                    }
                    if system::settings::keep_running_in_tray() {
                        api.prevent_close();
                        let _ = window.hide();
                    } else {
                        window.app_handle().exit(0);
                    }
                }
                tauri::WindowEvent::Moved(_) | tauri::WindowEvent::ScaleFactorChanged { .. } => {
                    let next_key = current_monitor_key(window);
                    let monitor_changed = if let Ok(mut previous_key) = last_monitor_key.lock() {
                        if *previous_key != next_key {
                            *previous_key = next_key;
                            true
                        } else {
                            false
                        }
                    } else {
                        true
                    };

                    if monitor_changed {
                        ensure_window_fits_current_monitor(window);
                    }

                    if let Some(state) = capture_window_state(window) {
                        if let Ok(mut pending) = pending_window_state.lock() {
                            *pending = Some(state);
                        }
                    }
                }
                tauri::WindowEvent::Resized(_) => {
                    if let Some(state) = capture_window_state(window) {
                        if let Ok(mut pending) = pending_window_state.lock() {
                            *pending = Some(state);
                        }
                    }
                }
                tauri::WindowEvent::Focused(false) => {
                    if let Some(state) = capture_window_state(window) {
                        save_window_state(&state);
                    } else if let Ok(guard) = pending_window_state.lock() {
                        if let Some(state) = guard.as_ref() {
                            save_window_state(state);
                        }
                    }
                }
                _ => {}
            }
        })
        // 注册命令
        .invoke_handler(tauri::generate_handler![
            hello,
            window_minimize,
            window_toggle_maximize,
            window_close,
            window_apply_material,
            window_supports_mica,
            init_database,
            print_database_tables,
            export_database,
            import_database,
            open_app_log_file,
            load_app_settings,
            save_app_settings,
            create_manual_game,
            game_detail,
            hide_game,
            hidden_games_list,
            restore_hidden_game,
            delete_game,
            delete_game_session,
            ignored_games_list,
            restore_ignored_game,
            sync_steam_owned_game_icons,
            sync_all_steam_game_media,
            // Redesign (Stage 4)
            analytics_overview,
            home_snapshot,
            games_library,
            timeline_months,
            timeline_days,
            steam_sync_status,
            clear_play_history
        ])
        // 启动 App
        .run(tauri::generate_context!())
        // 错误处理
        .expect("error while running tauri application");
}
