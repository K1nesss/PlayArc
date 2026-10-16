//! `analytics_overview` — everything the Analysis page (DOIT.md §7) and the Game
//! Detail page (§5, with `game_id`) need, in one call.
//!
//! Ranges: `days` = 7 / 30 / 90 / 365 …, or omitted for "All" (from the first
//! session). Optional `game_id` filters every figure to one game.
//!
//! Durations are seconds. No pre-formatted strings — the frontend formats.

use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::HashMap;

use crate::api::common::sessions::{
    clip, load_sessions, median, seconds_by_day, seconds_by_weekday_hour, Session,
};
use crate::api::common::time::{
    date_start, dates_between, day_key, local_date, next_day, now_unix,
};
use crate::storage::db::get_db_connection;

/// Session-length histogram buckets (§7): <30m, 30–60m, 1–2h, 2–3h, 3h+.
const HISTOGRAM_EDGES: [i64; 4] = [1_800, 3_600, 7_200, 10_800];
/// The activity calendar always covers the last 53 weeks.
const CALENDAR_DAYS: i64 = 371;

#[derive(Serialize)]
pub struct OverviewRange {
    /// Requested day count; `None` = all time.
    pub days: Option<i64>,
    pub start: i64,
    pub end: i64,
    /// Number of calendar days in the range (for daily averages).
    pub day_count: i64,
}

#[derive(Serialize)]
pub struct OverviewSummary {
    pub total_seconds: i64,
    pub daily_average_seconds: i64,
    pub session_count: i64,
    pub average_session_seconds: i64,
    pub median_session_seconds: i64,
    pub longest_session_seconds: i64,
    pub active_days: i64,
    /// All-time first / last play for the current filter (not just this range).
    pub first_played_at: Option<i64>,
    pub last_played_at: Option<i64>,
}

#[derive(Serialize)]
pub struct DayValue {
    pub day: String,
    pub seconds: i64,
}

#[derive(Serialize)]
pub struct HistogramBucket {
    /// Lower bound in seconds (inclusive).
    pub min_seconds: i64,
    /// Upper bound in seconds (exclusive); `None` for the open "3h+" bucket.
    pub max_seconds: Option<i64>,
    pub sessions: i64,
}

#[derive(Serialize)]
pub struct RankedGame {
    pub game_id: i64,
    pub name: String,
    pub seconds: i64,
    pub sessions: i64,
    /// Share of total playtime in range, 0–100 with one decimal.
    pub share: f64,
    pub last_played_at: Option<i64>,
}

#[derive(Serialize)]
pub struct PeakWindow {
    /// Monday = 0 … Sunday = 6.
    pub weekday: usize,
    pub start_hour: usize,
    /// Exclusive, may be 24.
    pub end_hour: usize,
    pub seconds: i64,
}

#[derive(Serialize)]
pub struct OverviewInsights {
    /// The 2-hour slot with the most play (§7 "Peak time").
    pub peak_window: Option<PeakWindow>,
    /// Monday = 0 … Sunday = 6.
    pub most_active_weekday: Option<usize>,
    pub longest_streak_days: i64,
    /// Consecutive days with play ending today (or yesterday if today is empty).
    pub current_streak_days: i64,
}

#[derive(Serialize)]
pub struct AnalyticsOverview {
    pub range: OverviewRange,
    pub game_id: Option<i64>,
    pub summary: OverviewSummary,
    pub daily: Vec<DayValue>,
    /// 7 rows (Monday-first) × 24 hours of seconds.
    pub weekday_hour: Vec<Vec<i64>>,
    pub session_histogram: Vec<HistogramBucket>,
    pub top_games: Vec<RankedGame>,
    pub insights: OverviewInsights,
    /// Last 53 weeks, independent of `days` (GitHub-style calendar).
    pub calendar: Vec<DayValue>,
}

#[tauri::command]
pub fn analytics_overview(
    days: Option<i64>,
    game_id: Option<i64>,
    top_limit: Option<i64>,
) -> Result<AnalyticsOverview, String> {
    let conn = get_db_connection()?;
    build_overview(&conn, now_unix(), days, game_id, top_limit.unwrap_or(10))
}

pub fn build_overview(
    conn: &Connection,
    now: i64,
    days: Option<i64>,
    game_id: Option<i64>,
    top_limit: i64,
) -> Result<AnalyticsOverview, String> {
    let days = days
        .filter(|value| *value > 0)
        .map(|value| value.min(3_660));
    let top_limit = top_limit.clamp(1, 50);

    let today = local_date(now);
    let range_end = date_start(next_day(today));
    let (first_played_at, last_played_at) = play_bounds(conn, now, game_id)?;

    let first_day = match days {
        Some(count) => today - chrono::Duration::days(count - 1),
        None => first_played_at.map(local_date).unwrap_or(today).min(today),
    };
    let range_start = date_start(first_day);
    let range_dates = dates_between(first_day, today);
    let day_count = range_dates.len() as i64;

    let sessions = load_sessions(conn, range_start, range_end, now, game_id)?;

    // Daily totals (split at midnight) and the summary built from them.
    let by_day = seconds_by_day(&sessions, range_start, range_end);
    let daily: Vec<DayValue> = range_dates
        .iter()
        .map(|date| DayValue {
            day: day_key(*date),
            seconds: *by_day.get(date).unwrap_or(&0),
        })
        .collect();
    let total_seconds: i64 = daily.iter().map(|item| item.seconds).sum();
    let active_days = daily.iter().filter(|item| item.seconds > 0).count() as i64;

    // Per-session metrics use sessions that *started* in range, at full length.
    let started: Vec<&Session> = sessions
        .iter()
        .filter(|session| session.start >= range_start)
        .collect();
    let mut durations: Vec<i64> = started.iter().map(|session| session.duration()).collect();
    let session_count = durations.len() as i64;
    let longest_session_seconds = durations.iter().copied().max().unwrap_or(0);
    let average_session_seconds = if session_count > 0 {
        durations.iter().sum::<i64>() / session_count
    } else {
        0
    };
    let median_session_seconds = median(&mut durations);

    let grid = seconds_by_weekday_hour(&sessions, range_start, range_end);

    let mut histogram: Vec<HistogramBucket> = (0..=HISTOGRAM_EDGES.len())
        .map(|index| HistogramBucket {
            min_seconds: if index == 0 {
                0
            } else {
                HISTOGRAM_EDGES[index - 1]
            },
            max_seconds: HISTOGRAM_EDGES.get(index).copied(),
            sessions: 0,
        })
        .collect();
    for session in &started {
        let duration = session.duration();
        let index = HISTOGRAM_EDGES
            .iter()
            .position(|edge| duration < *edge)
            .unwrap_or(HISTOGRAM_EDGES.len());
        histogram[index].sessions += 1;
    }

    let top_games = rank_games(
        conn,
        &sessions,
        range_start,
        range_end,
        total_seconds,
        top_limit,
    )?;

    let values: Vec<i64> = daily.iter().map(|item| item.seconds).collect();
    let insights = OverviewInsights {
        peak_window: peak_window(&grid),
        most_active_weekday: most_active_weekday(&grid),
        longest_streak_days: longest_streak(&values),
        current_streak_days: current_streak(&values),
    };

    // Calendar: always the last 53 weeks.
    let calendar_first = today - chrono::Duration::days(CALENDAR_DAYS - 1);
    let calendar_start = date_start(calendar_first);
    let calendar_sessions = if calendar_start >= range_start {
        sessions.clone()
    } else {
        load_sessions(conn, calendar_start, range_end, now, game_id)?
    };
    let calendar_by_day = seconds_by_day(&calendar_sessions, calendar_start, range_end);
    let calendar = dates_between(calendar_first, today)
        .into_iter()
        .map(|date| DayValue {
            day: day_key(date),
            seconds: *calendar_by_day.get(&date).unwrap_or(&0),
        })
        .collect();

    Ok(AnalyticsOverview {
        range: OverviewRange {
            days,
            start: range_start,
            end: range_end,
            day_count,
        },
        game_id,
        summary: OverviewSummary {
            total_seconds,
            daily_average_seconds: if day_count > 0 {
                total_seconds / day_count
            } else {
                0
            },
            session_count,
            average_session_seconds,
            median_session_seconds,
            longest_session_seconds,
            active_days,
            first_played_at,
            last_played_at,
        },
        daily,
        weekday_hour: grid.iter().map(|row| row.to_vec()).collect(),
        session_histogram: histogram,
        top_games,
        insights,
        calendar,
    })
}

fn play_bounds(
    conn: &Connection,
    now: i64,
    game_id: Option<i64>,
) -> Result<(Option<i64>, Option<i64>), String> {
    conn.query_row(
        "SELECT MIN(start_time), MAX(COALESCE(end_time, ?1))
         FROM game_sessions
         WHERE (?2 IS NULL OR game_id = ?2)",
        params![now, game_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .map_err(|e| e.to_string())
}

fn rank_games(
    conn: &Connection,
    sessions: &[Session],
    range_start: i64,
    range_end: i64,
    total_seconds: i64,
    limit: i64,
) -> Result<Vec<RankedGame>, String> {
    let mut seconds: HashMap<i64, i64> = HashMap::new();
    let mut counts: HashMap<i64, i64> = HashMap::new();
    for session in sessions {
        if let Some((start, end)) = clip(session, range_start, range_end) {
            *seconds.entry(session.game_id).or_insert(0) += end - start;
        }
        if session.start >= range_start {
            *counts.entry(session.game_id).or_insert(0) += 1;
        }
    }

    let mut ranked: Vec<(i64, i64)> = seconds.into_iter().filter(|(_, s)| *s > 0).collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    ranked.truncate(limit as usize);

    let mut stmt = conn
        .prepare(
            "SELECT g.name, gs.last_played_at
             FROM games g LEFT JOIN game_stats gs ON gs.game_id = g.id
             WHERE g.id = ?1",
        )
        .map_err(|e| e.to_string())?;

    ranked
        .into_iter()
        .map(|(game_id, game_seconds)| {
            let (name, last_played_at): (String, Option<i64>) = stmt
                .query_row(params![game_id], |row| Ok((row.get(0)?, row.get(1)?)))
                .map_err(|e| e.to_string())?;
            let share = if total_seconds > 0 {
                ((game_seconds as f64 / total_seconds as f64) * 1000.0).round() / 10.0
            } else {
                0.0
            };
            Ok(RankedGame {
                game_id,
                name,
                seconds: game_seconds,
                sessions: *counts.get(&game_id).unwrap_or(&0),
                share,
                last_played_at,
            })
        })
        .collect()
}

fn peak_window(grid: &[[i64; 24]; 7]) -> Option<PeakWindow> {
    let mut best: Option<PeakWindow> = None;
    for (weekday, row) in grid.iter().enumerate() {
        for start_hour in 0..23 {
            let seconds = row[start_hour] + row[start_hour + 1];
            if seconds > best.as_ref().map(|b| b.seconds).unwrap_or(0) {
                best = Some(PeakWindow {
                    weekday,
                    start_hour,
                    end_hour: start_hour + 2,
                    seconds,
                });
            }
        }
    }
    best
}

fn most_active_weekday(grid: &[[i64; 24]; 7]) -> Option<usize> {
    grid.iter()
        .map(|row| row.iter().sum::<i64>())
        .enumerate()
        .filter(|(_, seconds)| *seconds > 0)
        .max_by_key(|(_, seconds)| *seconds)
        .map(|(weekday, _)| weekday)
}

fn longest_streak(values: &[i64]) -> i64 {
    let mut longest = 0;
    let mut run = 0;
    for value in values {
        run = if *value > 0 { run + 1 } else { 0 };
        longest = longest.max(run);
    }
    longest
}

fn current_streak(values: &[i64]) -> i64 {
    let mut iter = values.iter().rev().peekable();
    // Today may not have play yet; the streak still counts up to yesterday.
    if iter.peek().is_some_and(|value| **value == 0) {
        iter.next();
    }
    iter.take_while(|value| **value > 0).count() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::common::time::date_start;
    use chrono::NaiveDate;

    fn memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("../../storage/schema.sql"))
            .unwrap();
        conn.execute(
            "INSERT INTO games (id, name, created_at, updated_at) VALUES (1, 'Alpha', 0, 0), (2, 'Beta', 0, 0)",
            [],
        )
        .unwrap();
        conn
    }

    fn add_session(conn: &Connection, game: i64, start: i64, minutes: i64) {
        conn.execute(
            "INSERT INTO game_sessions (game_id, start_time, end_time, duration_seconds) VALUES (?1, ?2, ?3, ?4)",
            params![game, start, start + minutes * 60, minutes * 60],
        )
        .unwrap();
    }

    #[test]
    fn splits_midnight_sessions_and_builds_summary() {
        let conn = memory_db();
        let today = NaiveDate::from_ymd_opt(2026, 9, 28).unwrap(); // a Monday
        let now = date_start(today) + 12 * 3600;
        let yesterday_2330 = date_start(today) - 30 * 60;

        add_session(&conn, 1, yesterday_2330, 100); // 23:30 → 01:10
        add_session(&conn, 2, date_start(today) + 9 * 3600, 20); // 09:00 – 09:20

        let overview = build_overview(&conn, now, Some(7), None, 10).unwrap();
        let last_two: Vec<i64> = overview
            .daily
            .iter()
            .rev()
            .take(2)
            .map(|d| d.seconds)
            .collect();
        assert_eq!(last_two, vec![70 * 60 + 20 * 60, 30 * 60]);
        assert_eq!(overview.summary.total_seconds, 120 * 60);
        assert_eq!(overview.summary.session_count, 2);
        assert_eq!(overview.summary.median_session_seconds, 60 * 60);
        assert_eq!(overview.summary.active_days, 2);
        assert_eq!(overview.insights.current_streak_days, 2);

        // Histogram: one <30m, one 1–2h.
        let counts: Vec<i64> = overview
            .session_histogram
            .iter()
            .map(|b| b.sessions)
            .collect();
        assert_eq!(counts, vec![1, 0, 1, 0, 0]);

        // Weekday×hour: Sunday 23h gets 30m, Monday 0h gets 60m, Monday 1h gets 10m.
        assert_eq!(overview.weekday_hour[6][23], 30 * 60);
        assert_eq!(overview.weekday_hour[0][0], 60 * 60);
        assert_eq!(overview.weekday_hour[0][1], 10 * 60);

        assert_eq!(overview.top_games[0].game_id, 1);
        assert_eq!(overview.top_games[0].share, 83.3);

        // Game filter.
        let beta = build_overview(&conn, now, Some(7), Some(2), 10).unwrap();
        assert_eq!(beta.summary.total_seconds, 20 * 60);
        assert_eq!(beta.calendar.len() as i64, CALENDAR_DAYS);

        // All time starts at the first session's day.
        let all = build_overview(&conn, now, None, None, 10).unwrap();
        assert_eq!(all.range.day_count, 2);
    }
}
