//! Session loading and time bucketing shared by the statistics commands.
//!
//! Sessions are loaded once and bucketed in Rust. A session that crosses
//! midnight or an hour boundary is split, so a 23:30–01:10 session counts
//! 30 minutes toward the first day and 70 toward the next (DOIT.md review).

use rusqlite::{params, Connection};
use std::collections::BTreeMap;

use chrono::NaiveDate;

use super::time::{
    date_start, hour_of_day, local_date, next_day, next_hour_boundary, weekday_index,
};

#[derive(Clone, Debug)]
pub struct Session {
    pub game_id: i64,
    pub start: i64,
    /// Effective end (now for an active session).
    pub end: i64,
}

impl Session {
    pub fn duration(&self) -> i64 {
        (self.end - self.start).max(0)
    }
}

/// Sessions overlapping [range_start, range_end), optionally for one game.
pub fn load_sessions(
    conn: &Connection,
    range_start: i64,
    range_end: i64,
    now: i64,
    game_id: Option<i64>,
) -> Result<Vec<Session>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT game_id, start_time, end_time
             FROM game_sessions
             WHERE start_time < ?2
               AND COALESCE(end_time, ?3) > ?1
               AND (?4 IS NULL OR game_id = ?4)
             ORDER BY start_time ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![range_start, range_end, now, game_id], |row| {
            let start: i64 = row.get(1)?;
            let end: Option<i64> = row.get(2)?;
            Ok(Session {
                game_id: row.get(0)?,
                start,
                end: end.unwrap_or(now).max(start),
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

/// Clip a session to [range_start, range_end).
pub fn clip(session: &Session, range_start: i64, range_end: i64) -> Option<(i64, i64)> {
    let start = session.start.max(range_start);
    let end = session.end.min(range_end);
    (end > start).then_some((start, end))
}

/// Seconds per local day, split at midnight.
pub fn seconds_by_day(
    sessions: &[Session],
    range_start: i64,
    range_end: i64,
) -> BTreeMap<NaiveDate, i64> {
    let mut map = BTreeMap::new();
    for session in sessions {
        let Some((mut cursor, end)) = clip(session, range_start, range_end) else {
            continue;
        };
        while cursor < end {
            let day = local_date(cursor);
            let boundary = date_start(next_day(day)).max(cursor + 1);
            let segment_end = boundary.min(end);
            *map.entry(day).or_insert(0) += segment_end - cursor;
            cursor = segment_end;
        }
    }
    map
}

/// 7×24 matrix of seconds (Monday-first), split at hour boundaries.
pub fn seconds_by_weekday_hour(
    sessions: &[Session],
    range_start: i64,
    range_end: i64,
) -> [[i64; 24]; 7] {
    let mut grid = [[0_i64; 24]; 7];
    for session in sessions {
        let Some((mut cursor, end)) = clip(session, range_start, range_end) else {
            continue;
        };
        while cursor < end {
            let boundary = next_hour_boundary(cursor).max(cursor + 1);
            let segment_end = boundary.min(end);
            grid[weekday_index(cursor)][hour_of_day(cursor)] += segment_end - cursor;
            cursor = segment_end;
        }
    }
    grid
}

pub fn median(values: &mut [i64]) -> i64 {
    if values.is_empty() {
        return 0;
    }
    values.sort_unstable();
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[mid - 1] + values[mid]) / 2
    } else {
        values[mid]
    }
}
