//! Month-based timeline (DOIT.md §6).
//!
//! `timeline_months` lists months that have sessions (for the "Sep 2026 ▾" picker).
//! `timeline_days` returns one month grouped by local day. Day totals are computed
//! from the full day's sessions (after filters), so a header like
//! "Sep 28 · Monday — 3h 24m · 3 sessions" is always right — unlike the old
//! offset-paged list, which could split a day across pages.
//!
//! Sessions are grouped by the day they *started*; durations are full session
//! lengths, matching the rows shown under each header.

use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::BTreeMap;

use crate::api::common::time::{day_key, local_date, month_bounds, month_key, now_unix};
use crate::storage::db::get_db_connection;

#[derive(Serialize)]
pub struct TimelineMonth {
    /// "YYYY-MM"
    pub month: String,
    pub sessions: i64,
    pub seconds: i64,
}

#[derive(Serialize, Clone)]
pub struct TimelineEntry {
    pub session_id: i64,
    pub game_id: i64,
    pub appid: Option<i64>,
    pub game_name: String,
    pub icon_path: Option<String>,
    pub cover_path: Option<String>,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub duration_seconds: i64,
    pub is_active: bool,
}

#[derive(Serialize)]
pub struct TimelineDay {
    /// "YYYY-MM-DD"
    pub day: String,
    /// Local midnight, for formatting the header on the frontend.
    pub day_start: i64,
    pub total_seconds: i64,
    pub session_count: i64,
    pub sessions: Vec<TimelineEntry>,
}

#[derive(Serialize)]
pub struct TimelineMonthPage {
    pub month: String,
    pub total_seconds: i64,
    pub session_count: i64,
    /// Newest day first.
    pub days: Vec<TimelineDay>,
}

#[tauri::command]
pub fn timeline_months(
    game_id: Option<i64>,
    query: Option<String>,
) -> Result<Vec<TimelineMonth>, String> {
    let conn = get_db_connection()?;
    let entries = load_entries(&conn, now_unix(), None, game_id, query.as_deref())?;

    let mut months: BTreeMap<String, (i64, i64)> = BTreeMap::new();
    for entry in &entries {
        let item = months
            .entry(month_key(local_date(entry.start_time)))
            .or_insert((0, 0));
        item.0 += 1;
        item.1 += entry.duration_seconds;
    }

    Ok(months
        .into_iter()
        .rev()
        .map(|(month, (sessions, seconds))| TimelineMonth {
            month,
            sessions,
            seconds,
        })
        .collect())
}

#[tauri::command]
pub fn timeline_days(
    month: String,
    game_id: Option<i64>,
    query: Option<String>,
) -> Result<TimelineMonthPage, String> {
    let conn = get_db_connection()?;
    let bounds = month_bounds(&month).ok_or_else(|| format!("Invalid month: {}", month))?;
    let entries = load_entries(&conn, now_unix(), Some(bounds), game_id, query.as_deref())?;
    Ok(group_by_day(month, entries))
}

fn group_by_day(month: String, entries: Vec<TimelineEntry>) -> TimelineMonthPage {
    let mut days: BTreeMap<chrono::NaiveDate, Vec<TimelineEntry>> = BTreeMap::new();
    for entry in entries {
        days.entry(local_date(entry.start_time))
            .or_default()
            .push(entry);
    }

    let days: Vec<TimelineDay> = days
        .into_iter()
        .rev()
        .map(|(date, mut sessions)| {
            sessions.sort_by_key(|s| std::cmp::Reverse(s.start_time));
            TimelineDay {
                day: day_key(date),
                day_start: crate::api::common::time::date_start(date),
                total_seconds: sessions.iter().map(|s| s.duration_seconds).sum(),
                session_count: sessions.len() as i64,
                sessions,
            }
        })
        .collect();

    TimelineMonthPage {
        month,
        total_seconds: days.iter().map(|d| d.total_seconds).sum(),
        session_count: days.iter().map(|d| d.session_count).sum(),
        days,
    }
}

fn load_entries(
    conn: &Connection,
    now: i64,
    bounds: Option<(i64, i64)>,
    game_id: Option<i64>,
    query: Option<&str>,
) -> Result<Vec<TimelineEntry>, String> {
    let query = query.map(str::trim).filter(|q| !q.is_empty());
    let (start, end) = bounds.unwrap_or((i64::MIN, i64::MAX));

    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.game_id, g.appid, g.name, g.icon_path, g.cover_path,
                    s.start_time, s.end_time,
                    COALESCE(s.duration_seconds, COALESCE(s.end_time, ?1) - s.start_time)
             FROM game_sessions s
             JOIN games g ON g.id = s.game_id
             WHERE s.start_time >= ?2 AND s.start_time < ?3
               AND (?4 IS NULL OR s.game_id = ?4)
               AND (?5 IS NULL OR g.name LIKE '%' || ?5 || '%')
             ORDER BY s.start_time DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![now, start, end, game_id, query], |row| {
            let end_time: Option<i64> = row.get(7)?;
            Ok(TimelineEntry {
                session_id: row.get(0)?,
                game_id: row.get(1)?,
                appid: row.get(2)?,
                game_name: row.get(3)?,
                icon_path: row.get(4)?,
                cover_path: row.get(5)?,
                start_time: row.get(6)?,
                end_time,
                duration_seconds: row.get::<_, i64>(8)?.max(0),
                is_active: end_time.is_none(),
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::common::time::date_start;
    use chrono::NaiveDate;

    #[test]
    fn groups_month_by_day_with_full_day_totals() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("../../storage/schema.sql"))
            .unwrap();
        conn.execute(
            "INSERT INTO games (id, name, created_at, updated_at) VALUES (1, 'Elden Ring', 0, 0), (2, 'Hades', 0, 0)",
            [],
        )
        .unwrap();
        let day = |m, d| date_start(NaiveDate::from_ymd_opt(2026, m, d).unwrap());
        let add = |game: i64, start: i64, minutes: i64| {
            conn.execute(
                "INSERT INTO game_sessions (game_id, start_time, end_time, duration_seconds) VALUES (?1, ?2, ?3, ?4)",
                params![game, start, start + minutes * 60, minutes * 60],
            )
            .unwrap();
        };
        add(1, day(9, 28) + 21 * 3600, 92);
        add(2, day(9, 28) + 9 * 3600, 30);
        add(1, day(9, 27) + 20 * 3600, 60);
        add(1, day(10, 1) + 20 * 3600, 45); // other month

        let bounds = month_bounds("2026-09").unwrap();
        let page = group_by_day(
            "2026-09".into(),
            load_entries(&conn, 0, Some(bounds), None, None).unwrap(),
        );
        assert_eq!(page.days.len(), 2);
        assert_eq!(page.days[0].day, "2026-09-28");
        assert_eq!(page.days[0].session_count, 2);
        assert_eq!(page.days[0].total_seconds, 122 * 60);
        assert_eq!(page.days[0].sessions[0].game_name, "Elden Ring"); // newest first
        assert_eq!(page.session_count, 3);

        let filtered = load_entries(&conn, 0, Some(bounds), None, Some("hades")).unwrap();
        assert_eq!(filtered.len(), 1);
        assert!(month_bounds("2026-13").is_none());

        // months list, newest first
        let all = load_entries(&conn, 0, None, None, None).unwrap();
        let months: std::collections::BTreeSet<String> = all
            .iter()
            .map(|e| month_key(local_date(e.start_time)))
            .collect();
        assert_eq!(months.len(), 2);
    }
}
