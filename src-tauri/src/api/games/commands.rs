use rusqlite::{params, Transaction};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

use crate::api::common::accent::ensure_accent_color;
use crate::core::process_watcher::request_process_match_index_refresh;
use crate::core::steam_scanner::steam_scan_print;
use crate::storage::db::get_db_connection;

type GameDetailRow = (
    i32,
    Option<i32>,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
    i64,
    i64,
    i64,
    i64,
    Option<i64>,
    i64,
);

#[derive(Serialize)]
pub struct GameDetailSession {
    pub session_id: i32,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub duration_seconds: i64,
    pub formatted: String,
}

#[derive(Serialize)]
pub struct GameDetail {
    pub id: i32,
    pub appid: Option<i32>,
    pub name: String,
    pub install_path: Option<String>,
    pub cover_path: Option<String>,
    pub hero_path: Option<String>,
    pub is_installed: bool,
    pub is_hidden: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub total_playtime_seconds: i64,
    pub total_playtime_formatted: String,
    pub last_played_at: Option<i64>,
    pub session_count: i64,
    pub average_session_seconds: i64,
    pub average_session_formatted: String,
    pub recent_sessions: Vec<GameDetailSession>,
    /// '#rrggbb' from the artwork for the Hero tint (§5); computed once, then cached.
    pub accent_color: Option<String>,
}

#[derive(Serialize)]
pub struct IgnoredGameItem {
    pub appid: i32,
    pub name: String,
    pub ignored_at: i64,
    pub reason: Option<String>,
}

#[derive(Serialize)]
pub struct HiddenGameItem {
    pub id: i32,
    pub appid: Option<i32>,
    pub name: String,
    pub updated_at: i64,
}

#[derive(Deserialize)]
pub struct CreateManualGameRequest {
    pub name: String,
    pub executable_path: String,
    pub icon_path: Option<String>,
    pub cover_path: Option<String>,
    pub hero_path: Option<String>,
}

#[tauri::command]
pub fn game_detail(id: i64, recent_limit: Option<i64>) -> Result<GameDetail, String> {
    let conn = get_db_connection()?;
    let now = current_unix_seconds();
    let recent_limit = recent_limit.unwrap_or(20).clamp(1, 100);

    let mut game_stmt = conn
        .prepare(
            "SELECT g.id,
                    g.appid,
                    g.name,
                    g.install_path,
                    g.cover_path,
                    g.hero_path,
                    g.is_installed,
                    g.is_hidden,
                    g.created_at,
                    g.updated_at,
                    COALESCE(gs.total_playtime_seconds, 0) AS total_playtime_seconds,
                    gs.last_played_at,
                    COALESCE((SELECT COUNT(1) FROM game_sessions s WHERE s.game_id = g.id), 0) AS session_count
             FROM games g
             LEFT JOIN game_stats gs ON gs.game_id = g.id
             WHERE g.id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let (
        id,
        appid,
        name,
        install_path,
        cover_path,
        hero_path,
        is_installed,
        is_hidden,
        created_at,
        updated_at,
        total_playtime_seconds,
        last_played_at,
        session_count,
    ): GameDetailRow = game_stmt
        .query_row([id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
                row.get(9)?,
                row.get(10)?,
                row.get(11)?,
                row.get(12)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let average_session_seconds = if session_count > 0 {
        total_playtime_seconds / session_count
    } else {
        0
    };

    let mut sessions_stmt = conn
        .prepare(
            "SELECT id,
                    start_time,
                    end_time,
                    COALESCE(duration_seconds, COALESCE(end_time, ?2) - start_time) AS actual_duration
             FROM game_sessions
             WHERE game_id = ?1
             ORDER BY start_time DESC
             LIMIT ?3",
        )
        .map_err(|e| e.to_string())?;

    let sessions = sessions_stmt
        .query_map(params![id, now, recent_limit], |row| {
            let duration_seconds: i64 = row.get(3)?;
            Ok(GameDetailSession {
                session_id: row.get(0)?,
                start_time: row.get(1)?,
                end_time: row.get(2)?,
                duration_seconds,
                formatted: format_duration(duration_seconds),
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let cached_accent: Option<String> = conn
        .query_row(
            "SELECT accent_color FROM games WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let accent_color = ensure_accent_color(
        &conn,
        id as i64,
        cached_accent,
        hero_path.as_deref(),
        cover_path.as_deref(),
    );

    Ok(GameDetail {
        id,
        appid,
        name,
        install_path,
        cover_path,
        hero_path,
        is_installed: is_installed == 1,
        is_hidden: is_hidden == 1,
        created_at,
        updated_at,
        total_playtime_seconds,
        total_playtime_formatted: format_duration(total_playtime_seconds),
        last_played_at,
        session_count,
        average_session_seconds,
        average_session_formatted: format_duration(average_session_seconds),
        recent_sessions: sessions,
        accent_color,
    })
}

#[tauri::command]
pub fn hide_game(game_id: i64) -> Result<(), String> {
    let conn = get_db_connection()?;
    let changed = conn
        .execute(
            "UPDATE games SET is_hidden = 1, updated_at = strftime('%s', 'now') WHERE id = ?1",
            params![game_id],
        )
        .map_err(|e| e.to_string())?;

    if changed == 0 {
        return Err("Game not found".to_string());
    }

    request_process_match_index_refresh();
    Ok(())
}

#[tauri::command]
pub fn hidden_games_list() -> Result<Vec<HiddenGameItem>, String> {
    let conn = get_db_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, appid, name, updated_at
             FROM games
             WHERE is_hidden = 1
             ORDER BY updated_at DESC, name COLLATE NOCASE ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(HiddenGameItem {
                id: row.get(0)?,
                appid: row.get(1)?,
                name: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_hidden_game(game_id: i64) -> Result<(), String> {
    let conn = get_db_connection()?;
    let changed = conn
        .execute(
            "UPDATE games SET is_hidden = 0, updated_at = strftime('%s', 'now') WHERE id = ?1",
            params![game_id],
        )
        .map_err(|e| e.to_string())?;

    if changed == 0 {
        return Err("Hidden game not found".to_string());
    }

    request_process_match_index_refresh();
    Ok(())
}

#[tauri::command]
pub fn delete_game(game_id: i64) -> Result<(), String> {
    let mut conn = get_db_connection()?;
    let transaction = conn.transaction().map_err(|e| e.to_string())?;

    let (appid, name) = transaction
        .query_row(
            "SELECT appid, name FROM games WHERE id = ?1",
            params![game_id],
            |row| Ok((row.get::<_, Option<i32>>(0)?, row.get::<_, String>(1)?)),
        )
        .map_err(|_| "Game not found".to_string())?;

    if let Some(appid) = appid {
        transaction
            .execute(
                "INSERT INTO ignored_games (appid, name, ignored_at, reason)
                 VALUES (?1, ?2, strftime('%s', 'now'), 'deleted_by_user')
                 ON CONFLICT(appid) DO UPDATE SET
                 name = excluded.name,
                 ignored_at = excluded.ignored_at,
                 reason = excluded.reason",
                params![appid, name],
            )
            .map_err(|e| e.to_string())?;
    }

    transaction
        .execute(
            "DELETE FROM game_sessions WHERE game_id = ?1",
            params![game_id],
        )
        .map_err(|e| e.to_string())?;
    transaction
        .execute(
            "DELETE FROM game_stats WHERE game_id = ?1",
            params![game_id],
        )
        .map_err(|e| e.to_string())?;
    transaction
        .execute("DELETE FROM games WHERE id = ?1", params![game_id])
        .map_err(|e| e.to_string())?;

    transaction.commit().map_err(|e| e.to_string())?;
    request_process_match_index_refresh();
    Ok(())
}

#[tauri::command]
pub fn ignored_games_list() -> Result<Vec<IgnoredGameItem>, String> {
    let conn = get_db_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT appid, name, ignored_at, reason
             FROM ignored_games
             ORDER BY ignored_at DESC, name COLLATE NOCASE ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(IgnoredGameItem {
                appid: row.get(0)?,
                name: row.get(1)?,
                ignored_at: row.get(2)?,
                reason: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_ignored_game(app_id: i64) -> Result<(), String> {
    let conn = get_db_connection()?;
    let changed = conn
        .execute(
            "DELETE FROM ignored_games WHERE appid = ?1",
            params![app_id],
        )
        .map_err(|e| e.to_string())?;

    if changed == 0 {
        return Err("Ignored game not found".to_string());
    }

    steam_scan_print()
}

#[tauri::command]
pub fn create_manual_game(app: AppHandle, request: CreateManualGameRequest) -> Result<i64, String> {
    let name = request.name.trim();
    if name.is_empty() {
        return Err("Game name is required".to_string());
    }

    let executable_path = normalize_existing_file_path(&request.executable_path, "Executable")?;
    if executable_path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| !value.eq_ignore_ascii_case("exe"))
        .unwrap_or(true)
    {
        return Err("Please choose a .exe file".to_string());
    }

    let install_path = executable_path
        .parent()
        .map(|path| path.to_string_lossy().to_string())
        .ok_or_else(|| "Failed to resolve executable directory".to_string())?;

    let icon_path = match optional_existing_file_path(request.icon_path, "Icon")? {
        Some(path) => Some(copy_manual_asset(&app, &path, "icons")?),
        None => extract_executable_icon(&app, &executable_path)?,
    };
    let cover_path = optional_existing_file_path(request.cover_path, "Cover")?
        .map(|path| copy_manual_asset(&app, &path, "covers"))
        .transpose()?;
    let hero_path = optional_existing_file_path(request.hero_path, "Hero")?
        .map(|path| copy_manual_asset(&app, &path, "heroes"))
        .transpose()?;

    let conn = get_db_connection()?;
    let now = current_unix_seconds();
    let executable_path_string = executable_path.to_string_lossy().to_string();

    let existing_id = conn
        .query_row(
            "SELECT id FROM games WHERE LOWER(executable_path) = LOWER(?1) LIMIT 1",
            params![executable_path_string],
            |row| row.get::<_, i64>(0),
        )
        .ok();

    if let Some(game_id) = existing_id {
        conn.execute(
            "UPDATE games
             SET source = 'manual',
                 name = ?1,
                 install_path = ?2,
                 executable_path = ?3,
                 icon_path = COALESCE(?4, icon_path),
                 cover_path = COALESCE(?5, cover_path),
                 hero_path = COALESCE(?6, hero_path),
                 is_installed = 1,
                 is_hidden = 0,
                 updated_at = ?7
             WHERE id = ?8",
            params![
                name,
                install_path,
                executable_path_string,
                icon_path,
                cover_path,
                hero_path,
                now,
                game_id
            ],
        )
        .map_err(|e| e.to_string())?;
        request_process_match_index_refresh();
        return Ok(game_id);
    }

    conn.execute(
        "INSERT INTO games (
            appid,
            source,
            name,
            install_path,
            executable_path,
            icon_path,
            cover_path,
            hero_path,
            is_installed,
            is_hidden,
            created_at,
            updated_at
         )
         VALUES (NULL, 'manual', ?1, ?2, ?3, ?4, ?5, ?6, 1, 0, ?7, ?7)",
        params![
            name,
            install_path,
            executable_path_string,
            icon_path,
            cover_path,
            hero_path,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    request_process_match_index_refresh();
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn delete_game_session(session_id: i64) -> Result<(), String> {
    let mut conn = get_db_connection()?;
    let transaction = conn.transaction().map_err(|e| e.to_string())?;
    let (game_id, end_time) = transaction
        .query_row(
            "SELECT game_id, end_time FROM game_sessions WHERE id = ?1",
            params![session_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Option<i64>>(1)?)),
        )
        .map_err(|_| "Session not found".to_string())?;

    if end_time.is_none() {
        return Err("Cannot delete an active session. Stop the game first.".to_string());
    }

    transaction
        .execute(
            "DELETE FROM game_sessions WHERE id = ?1",
            params![session_id],
        )
        .map_err(|e| e.to_string())?;
    recompute_game_stats(&transaction, game_id)?;
    transaction.commit().map_err(|e| e.to_string())?;

    Ok(())
}

/// Delete all finished play sessions (DOIT.md §8 Data → Clear Local History).
/// The game library is kept; a session that is still running is kept too.
/// Returns the number of sessions deleted.
#[tauri::command]
pub fn clear_play_history() -> Result<i64, String> {
    let mut conn = get_db_connection()?;
    let transaction = conn.transaction().map_err(|e| e.to_string())?;

    let game_ids: Vec<i64> = {
        let mut stmt = transaction
            .prepare("SELECT DISTINCT game_id FROM game_sessions WHERE end_time IS NOT NULL")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    let deleted = transaction
        .execute("DELETE FROM game_sessions WHERE end_time IS NOT NULL", [])
        .map_err(|e| e.to_string())?;

    for game_id in game_ids {
        recompute_game_stats(&transaction, game_id)?;
    }

    transaction.commit().map_err(|e| e.to_string())?;
    Ok(deleted as i64)
}

fn recompute_game_stats(transaction: &Transaction<'_>, game_id: i64) -> Result<(), String> {
    let (total_playtime_seconds, last_played_at): (i64, Option<i64>) = transaction
        .query_row(
            "SELECT COALESCE(SUM(
                    CASE
                        WHEN duration_seconds IS NOT NULL THEN duration_seconds
                        WHEN end_time IS NOT NULL THEN end_time - start_time
                        ELSE 0
                    END
                ), 0),
                MAX(end_time)
             FROM game_sessions
             WHERE game_id = ?1",
            params![game_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    if total_playtime_seconds > 0 || last_played_at.is_some() {
        transaction
            .execute(
                "INSERT INTO game_stats (game_id, total_playtime_seconds, last_played_at)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(game_id) DO UPDATE SET
                 total_playtime_seconds = excluded.total_playtime_seconds,
                 last_played_at = excluded.last_played_at",
                params![game_id, total_playtime_seconds, last_played_at],
            )
            .map_err(|e| e.to_string())?;
    } else {
        transaction
            .execute(
                "DELETE FROM game_stats WHERE game_id = ?1",
                params![game_id],
            )
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn normalize_existing_file_path(path: &str, label: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path.trim());
    if !path.is_file() {
        return Err(format!("{} file not found", label));
    }

    path.canonicalize()
        .map_err(|e| format!("Failed to resolve {} path: {}", label.to_lowercase(), e))
}

fn optional_existing_file_path(
    path: Option<String>,
    label: &str,
) -> Result<Option<PathBuf>, String> {
    match path.map(|value| value.trim().to_string()) {
        Some(path) if !path.is_empty() => normalize_existing_file_path(&path, label).map(Some),
        _ => Ok(None),
    }
}

fn copy_manual_asset(app: &AppHandle, source: &Path, kind: &str) -> Result<String, String> {
    let app_cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("Failed to resolve app cache directory: {}", e))?;
    let target_dir = app_cache_dir.join("manual-assets").join(kind);
    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create manual asset directory: {}", e))?;

    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("bin")
        .to_lowercase();
    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .map(sanitize_filename)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "asset".to_string());
    let target_path = target_dir.join(format!("{}_{}.{}", current_unix_seconds(), stem, extension));

    fs::copy(source, &target_path).map_err(|e| format!("Failed to copy manual asset: {}", e))?;

    Ok(target_path.to_string_lossy().to_string())
}

fn extract_executable_icon(
    app: &AppHandle,
    executable_path: &Path,
) -> Result<Option<String>, String> {
    if !cfg!(target_os = "windows") {
        return Ok(None);
    }

    let app_cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("Failed to resolve app cache directory: {}", e))?;
    let target_dir = app_cache_dir.join("manual-assets").join("icons");
    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create manual icon directory: {}", e))?;

    let stem = executable_path
        .file_stem()
        .and_then(|value| value.to_str())
        .map(sanitize_filename)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "game".to_string());
    let target_path = target_dir.join(format!("{}_{}_icon.png", current_unix_seconds(), stem));

    let script = r#"
$src = $args[0]
$dst = $args[1]
Add-Type -AssemblyName System.Drawing
$icon = [System.Drawing.Icon]::ExtractAssociatedIcon($src)
if ($null -eq $icon) { exit 2 }
$bitmap = $icon.ToBitmap()
$bitmap.Save($dst, [System.Drawing.Imaging.ImageFormat]::Png)
$bitmap.Dispose()
$icon.Dispose()
"#;

    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .arg(executable_path)
        .arg(&target_path)
        .output()
        .map_err(|e| format!("Failed to extract executable icon: {}", e))?;

    if !output.status.success() || !target_path.is_file() {
        return Ok(None);
    }

    Ok(Some(target_path.to_string_lossy().to_string()))
}

fn sanitize_filename(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn current_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn format_duration(total_seconds: i64) -> String {
    let seconds = total_seconds.max(0);
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let remaining_seconds = seconds % 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, remaining_seconds)
    } else {
        format!("{}s", remaining_seconds)
    }
}
