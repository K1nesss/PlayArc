//! Small key/value store for app state that must survive restarts
//! (e.g. the last successful Steam sync shown in the sidebar footer).

use rusqlite::{params, Connection, OptionalExtension};

pub const STEAM_LAST_SYNC_AT: &str = "steam.last_sync_at";
pub const STEAM_LAST_ERROR: &str = "steam.last_error";
pub const STEAM_LAST_ERROR_AT: &str = "steam.last_error_at";

pub fn get_meta(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    conn.query_row(
        "SELECT value FROM app_meta WHERE key = ?1",
        params![key],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map_err(|e| e.to_string())
}

pub fn set_meta(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO app_meta (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}

pub fn delete_meta(conn: &Connection, key: &str) -> Result<(), String> {
    conn.execute("DELETE FROM app_meta WHERE key = ?1", params![key])
        .map(|_| ())
        .map_err(|e| e.to_string())
}
