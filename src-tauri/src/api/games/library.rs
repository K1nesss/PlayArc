//! `games_library` — the Games page list (DOIT.md §4).
//!
//! There is no row cap (large Steam libraries are not
//! truncated), sorting is chosen by the page, and `total` is the library size
//! ("143 games") independent of the search box.

use rusqlite::params;
use serde::Serialize;

use crate::storage::db::get_db_connection;

#[derive(Serialize)]
pub struct LibraryGame {
    pub id: i64,
    pub appid: Option<i64>,
    pub name: String,
    pub cover_path: Option<String>,
    pub icon_path: Option<String>,
    pub hero_path: Option<String>,
    /// Cached artwork color if already computed (computed lazily on detail/home).
    pub accent_color: Option<String>,
    pub is_installed: bool,
    pub total_playtime_seconds: i64,
    pub last_played_at: Option<i64>,
    pub session_count: i64,
}

#[derive(Serialize)]
pub struct GamesLibrary {
    /// Visible (non-hidden) games in the library, ignoring search/filters.
    pub total: i64,
    pub items: Vec<LibraryGame>,
}

/// sort: "recent" (default) | "playtime" | "name" | "sessions"
#[tauri::command]
pub fn games_library(
    search: Option<String>,
    installed_only: Option<bool>,
    sort: Option<String>,
) -> Result<GamesLibrary, String> {
    let conn = get_db_connection()?;
    let search = search
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let installed = installed_only.and_then(|only| only.then_some(1_i64));

    let order = match sort.as_deref() {
        Some("playtime") => "total_playtime_seconds DESC, g.name COLLATE NOCASE ASC",
        Some("name") => "g.name COLLATE NOCASE ASC",
        Some("sessions") => "session_count DESC, total_playtime_seconds DESC",
        _ => "gs.last_played_at IS NULL, gs.last_played_at DESC, g.name COLLATE NOCASE ASC",
    };

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM games WHERE is_hidden = 0",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let sql = format!(
        "SELECT g.id, g.appid, g.name, g.cover_path, g.icon_path, g.hero_path, g.accent_color,
                g.is_installed,
                COALESCE(gs.total_playtime_seconds, 0) AS total_playtime_seconds,
                gs.last_played_at,
                (SELECT COUNT(1) FROM game_sessions s WHERE s.game_id = g.id) AS session_count
         FROM games g
         LEFT JOIN game_stats gs ON gs.game_id = g.id
         WHERE g.is_hidden = 0
           AND (?1 IS NULL OR g.name LIKE '%' || ?1 || '%')
           AND (?2 IS NULL OR g.is_installed = ?2)
         ORDER BY {}",
        order
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let items = stmt
        .query_map(params![search, installed], |row| {
            let accent: Option<String> = row.get(6)?;
            Ok(LibraryGame {
                id: row.get(0)?,
                appid: row.get(1)?,
                name: row.get(2)?,
                cover_path: row.get(3)?,
                icon_path: row.get(4)?,
                hero_path: row.get(5)?,
                accent_color: accent.filter(|value| !value.is_empty()),
                is_installed: row.get::<_, i64>(7)? == 1,
                total_playtime_seconds: row.get(8)?,
                last_played_at: row.get(9)?,
                session_count: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(GamesLibrary { total, items })
}
