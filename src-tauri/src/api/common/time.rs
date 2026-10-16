//! Local calendar helpers for statistics.
//!
//! Uses the system time zone (chrono::Local), so day/hour boundaries are correct
//! for every user and across daylight-saving changes — unlike the fixed UTC+8
//! offset the older dashboard/analytics commands use.

use chrono::{DateTime, Datelike, Duration, Local, LocalResult, NaiveDate, TimeZone, Timelike};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

pub fn local_datetime(ts: i64) -> DateTime<Local> {
    match Local.timestamp_opt(ts, 0) {
        LocalResult::Single(dt) | LocalResult::Ambiguous(dt, _) => dt,
        LocalResult::None => Local::now(),
    }
}

pub fn local_date(ts: i64) -> NaiveDate {
    local_datetime(ts).date_naive()
}

/// Unix timestamp of local midnight at the start of `date`.
/// If midnight does not exist (DST gap), uses the first valid instant after it.
pub fn date_start(date: NaiveDate) -> i64 {
    let midnight = date.and_hms_opt(0, 0, 0).expect("valid midnight");
    match Local.from_local_datetime(&midnight) {
        LocalResult::Single(dt) => dt.timestamp(),
        LocalResult::Ambiguous(earliest, _) => earliest.timestamp(),
        LocalResult::None => {
            let one_am = date.and_hms_opt(1, 0, 0).expect("valid 01:00");
            Local
                .from_local_datetime(&one_am)
                .earliest()
                .map(|dt| dt.timestamp())
                .unwrap_or_else(|| midnight.and_utc().timestamp())
        }
    }
}

pub fn next_day(date: NaiveDate) -> NaiveDate {
    date.succ_opt().unwrap_or(date)
}

pub fn day_key(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

pub fn month_key(date: NaiveDate) -> String {
    date.format("%Y-%m").to_string()
}

/// Parse "YYYY-MM" into the [start, end) unix range of that local month.
pub fn month_bounds(month: &str) -> Option<(i64, i64)> {
    let first = NaiveDate::parse_from_str(&format!("{}-01", month), "%Y-%m-%d").ok()?;
    let next = if first.month() == 12 {
        NaiveDate::from_ymd_opt(first.year() + 1, 1, 1)?
    } else {
        NaiveDate::from_ymd_opt(first.year(), first.month() + 1, 1)?
    };
    Some((date_start(first), date_start(next)))
}

/// Monday = 0 … Sunday = 6.
pub fn weekday_index(ts: i64) -> usize {
    local_datetime(ts).weekday().num_days_from_monday() as usize
}

pub fn hour_of_day(ts: i64) -> usize {
    local_datetime(ts).hour() as usize
}

/// Start of the next local hour after `ts`.
pub fn next_hour_boundary(ts: i64) -> i64 {
    let dt = local_datetime(ts);
    let into_hour = (dt.minute() * 60 + dt.second()) as i64;
    ts - into_hour + 3_600
}

/// Inclusive list of dates from `first` to `last`.
pub fn dates_between(first: NaiveDate, last: NaiveDate) -> Vec<NaiveDate> {
    let mut out = Vec::new();
    let mut day = first;
    while day <= last {
        out.push(day);
        day += Duration::days(1);
    }
    out
}
