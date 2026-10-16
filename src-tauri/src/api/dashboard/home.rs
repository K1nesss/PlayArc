//! `home_snapshot` — the Home page's "now / today / most recent" data (DOIT.md §3).
//!
//! While a game runs: the game (with artwork + accent color), its session start,
//! and today's totals. When idle: the last finished session, so Home is never blank.

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use crate::api::common::accent::ensure_accent_color;
use crate::api::common::sessions::{clip, load_sessions};
use crate::api::common::time::{date_start, local_date, next_day, now_unix};
use crate::storage::db::get_db_connection;

#[derive(Serialize)]
pub struct HomeGame {
    pub game_id: i64,
    pub name: String,
    pub cover_path: Option<String>,
    pub hero_path: Option<String>,
    pub icon_path: Option<String>,
    /// '#rrggbb' extracted from the artwork, for the Hero tint (§3).
    pub accent_color: Option<String>,
}

#[derive(Serialize)]
pub struct NowPlaying {
    pub session_id: i64,
    pub game: HomeGame,
    pub start_time: i64,
    pub elapsed_seconds: i64,
}

#[derive(Serialize)]
pub struct LastSession {
    pub session_id: i64,
    pub game: HomeGame,
    pub start_time: i64,
    pub end_time: i64,
    pub duration_seconds: i64,
}

#[derive(Serialize)]
pub struct TodaySummary {
    pub seconds: i64,
    /// Sessions that overlap today (an active one included).
    pub sessions: i64,
}

#[derive(Serialize)]
pub struct HomeSnapshot {
    pub now: i64,
    pub now_playing: Option<NowPlaying>,
    pub today: TodaySummary,
    pub last_session: Option<LastSession>,
}

#[tauri::command]
pub fn home_snapshot() -> Result<HomeSnapshot, String> {
    let conn = get_db_connection()?;
    let now = now_unix();

    let today = local_date(now);
    let today_start = date_start(today);
    let today_end = date_start(next_day(today));
    let today_sessions = load_sessions(&conn, today_start, today_end, now, None)?;
    let today_seconds = today_sessions
        .iter()
        .filter_map(|session| clip(session, today_start, today_end))
        .map(|(start, end)| end - start)
        .sum();

    let now_playing = conn
        .query_row(
            "SELECT id, game_id, start_time FROM game_sessions
             WHERE end_time IS NULL ORDER BY start_time DESC LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .optional()
        .map_err(|e| e.to_string())?
        .map(
            |(session_id, game_id, start_time)| -> Result<NowPlaying, String> {
                Ok(NowPlaying {
                    session_id,
                    game: load_home_game(&conn, game_id)?,
                    start_time,
                    elapsed_seconds: (now - start_time).max(0),
                })
            },
        )
        .transpose()?;

    let last_session = conn
        .query_row(
            "SELECT id, game_id, start_time, end_time,
                    COALESCE(duration_seconds, end_time - start_time)
             FROM game_sessions
             WHERE end_time IS NOT NULL
             ORDER BY end_time DESC LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            },
        )
        .optional()
        .map_err(|e| e.to_string())?
        .map(
            |(session_id, game_id, start_time, end_time, duration)| -> Result<LastSession, String> {
                Ok(LastSession {
                    session_id,
                    game: load_home_game(&conn, game_id)?,
                    start_time,
                    end_time,
                    duration_seconds: duration.max(0),
                })
            },
        )
        .transpose()?;

    Ok(HomeSnapshot {
        now,
        now_playing,
        today: TodaySummary {
            seconds: today_seconds,
            sessions: today_sessions.len() as i64,
        },
        last_session,
    })
}

pub fn load_home_game(conn: &Connection, game_id: i64) -> Result<HomeGame, String> {
    let (name, cover_path, hero_path, icon_path, accent): (
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = conn
        .query_row(
            "SELECT name, cover_path, hero_path, icon_path, accent_color FROM games WHERE id = ?1",
            params![game_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .map_err(|e| e.to_string())?;

    let accent_color = ensure_accent_color(
        conn,
        game_id,
        accent,
        hero_path.as_deref(),
        cover_path.as_deref(),
    );

    Ok(HomeGame {
        game_id,
        name,
        cover_path,
        hero_path,
        icon_path,
        accent_color,
    })
}
