use chrono::{Datelike, Local, TimeZone, Timelike};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::MdcraftApp;

const AUTO_WIKI_SYNC_TRIGGER_HOUR: u32 = 7;
const AUTO_WIKI_SYNC_TRIGGER_MINUTE: u32 = 40;

pub(super) fn now_unix_seconds() -> Option<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

fn local_timestamp_to_day_and_minute(unix_seconds: u64) -> Option<(i32, u32, u32)> {
    let date_time = Local.timestamp_opt(unix_seconds as i64, 0).single()?;
    let minute_of_day = date_time.hour() * 60 + date_time.minute();
    Some((date_time.year(), date_time.ordinal(), minute_of_day))
}

pub(super) fn has_reached_auto_sync_window(unix_seconds: u64) -> bool {
    let Some((_, _, minute_of_day)) = local_timestamp_to_day_and_minute(unix_seconds) else {
        return false;
    };

    let trigger_minute = AUTO_WIKI_SYNC_TRIGGER_HOUR * 60 + AUTO_WIKI_SYNC_TRIGGER_MINUTE;
    minute_of_day >= trigger_minute
}

fn is_same_local_day(left_unix_seconds: u64, right_unix_seconds: u64) -> bool {
    let Some((left_year, left_ordinal, _)) = local_timestamp_to_day_and_minute(left_unix_seconds)
    else {
        return false;
    };
    let Some((right_year, right_ordinal, _)) =
        local_timestamp_to_day_and_minute(right_unix_seconds)
    else {
        return false;
    };

    left_year == right_year && left_ordinal == right_ordinal
}

pub(super) fn did_sync_today_after_window(last_sync_unix_seconds: u64, now_unix_seconds: u64) -> bool {
    if !is_same_local_day(last_sync_unix_seconds, now_unix_seconds) {
        return false;
    }

    has_reached_auto_sync_window(last_sync_unix_seconds)
}

pub(super) fn should_start_auto_wiki_sync(app: &MdcraftApp, now_unix_seconds: u64) -> bool {
    if !has_reached_auto_sync_window(now_unix_seconds) {
        return false;
    }

    let Some(last_sync) = app.wiki_last_sync_unix_seconds else {
        return true;
    };

    !did_sync_today_after_window(last_sync, now_unix_seconds)
}
