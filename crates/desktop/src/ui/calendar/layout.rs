use crate::app::ViewMode;
use chrono::{DateTime, Datelike, Days, Local, NaiveDate, TimeZone, Timelike, Utc};

pub const HEADER: f32 = 34.0;
pub const HOUR: f32 = 56.0;
pub const HEIGHT: f32 = HEADER + 24.0 * HOUR;
pub const GUTTER: f32 = 58.0;

pub fn dates(date: NaiveDate, mode: ViewMode) -> Vec<NaiveDate> {
    let (first, count) = match mode {
        ViewMode::Day => (date, 1),
        ViewMode::Week => (
            date - Days::new(date.weekday().num_days_from_monday().into()),
            7,
        ),
    };
    (0..count)
        .filter_map(|offset| first.checked_add_days(Days::new(offset)))
        .collect()
}

/// First valid local minute also covers zones that skip midnight during DST changes.
pub fn day_start(date: NaiveDate) -> Option<DateTime<Utc>> {
    (0..1440).find_map(|minute| {
        let time = date.and_hms_opt(minute / 60, minute % 60, 0)?;
        Local
            .from_local_datetime(&time)
            .earliest()
            .map(|value| value.with_timezone(&Utc))
    })
}

pub fn period(date: NaiveDate, mode: ViewMode) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    let dates = dates(date, mode);
    Some((
        day_start(*dates.first()?)?,
        day_start(dates.last()?.checked_add_days(Days::new(1))?)?,
    ))
}

pub fn minute(time: DateTime<Utc>) -> f32 {
    let local = time.with_timezone(&Local);
    (local.hour() * 60 + local.minute()) as f32 + local.second() as f32 / 60.0
}

pub fn y(minute: f32) -> f32 {
    HEADER + minute / 60.0 * HOUR
}
