//! Tests ported from the Luxon test suite
//! (`luxon-master/test/datetime/` and related).
//!
//! Source: https://github.com/moment/luxon (MIT License)
//! Ported to Rust #[test] functions by fastemporal.

use fastemporal::{Duration, ZonedDateTime};

// ─── Helper ───────────────────────────────────────────────────────────────────

fn dt(s: &str) -> ZonedDateTime {
    ZonedDateTime::from_iso(s).unwrap_or_else(|e| panic!("parse '{s}': {e}"))
}

fn ymd(y: i32, m: u8, d: u8) -> ZonedDateTime {
    let s = format!("{y:04}-{m:02}-{d:02}T00:00:00Z");
    dt(&s)
}

#[allow(dead_code)]
fn ymdh(y: i32, m: u8, d: u8, h: u8) -> ZonedDateTime {
    let s = format!("{y:04}-{m:02}-{d:02}T{h:02}:00:00Z");
    dt(&s)
}

#[allow(dead_code)]
fn ymdhms(y: i32, m: u8, d: u8, h: u8, min: u8, s: u8) -> ZonedDateTime {
    let iso = format!("{y:04}-{m:02}-{d:02}T{h:02}:{min:02}:{s:02}Z");
    dt(&iso)
}

// ─── DateTime.now() ───────────────────────────────────────────────────────────

#[test]
fn now_has_todays_date() {
    let now = ZonedDateTime::now();
    let sys = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Should be within 2 seconds
    assert!((now.unix_seconds() - sys as i64).abs() < 2);
}

// ─── DateTime.fromISO ─────────────────────────────────────────────────────────

#[test]
fn from_iso_date_only() {
    let z = dt("2025-03-15");
    assert_eq!((z.year(), z.month(), z.day()), (2025, 3, 15));
    assert_eq!((z.hour(), z.minute(), z.second()), (0, 0, 0));
}

#[test]
fn from_iso_utc() {
    let z = dt("2025-06-07T14:32:00Z");
    assert_eq!((z.year(), z.month(), z.day()), (2025, 6, 7));
    assert_eq!((z.hour(), z.minute(), z.second()), (14, 32, 0));
    assert_eq!(z.offset_seconds(), 0);
}

#[test]
fn from_iso_with_millis() {
    let z = dt("2025-06-07T14:32:00.123Z");
    assert_eq!(z.millisecond(), 123);
}

#[test]
fn from_iso_with_nanos() {
    let z = dt("2025-06-07T14:32:00.123456789Z");
    assert_eq!(z.nanosecond(), 123_456_789);
}

#[test]
fn from_iso_negative_offset() {
    let z = dt("2025-06-07T14:32:00.000-04:00");
    assert_eq!(z.offset_seconds(), -4 * 3600);
    // UTC instant is 14:32 + 4h = 18:32Z
    let utc = dt("2025-06-07T18:32:00Z");
    assert_eq!(z.unix_seconds(), utc.unix_seconds());
}

#[test]
fn from_iso_positive_offset() {
    let z = dt("2025-06-07T14:32:00+05:30");
    assert_eq!(z.offset_seconds(), 5 * 3600 + 30 * 60);
}

#[test]
fn from_iso_no_time() {
    let z = dt("2025-06-07T14:32");
    assert_eq!((z.hour(), z.minute(), z.second()), (14, 32, 0));
}

#[test]
fn from_iso_invalid_returns_err() {
    assert!(ZonedDateTime::from_iso("not-a-date").is_err());
    assert!(ZonedDateTime::from_iso("2025-13-01T00:00:00Z").is_err());
    assert!(ZonedDateTime::from_iso("2025-02-30T00:00:00Z").is_err());
}

#[test]
fn from_iso_leap_day_valid() {
    assert!(ZonedDateTime::from_iso("2024-02-29T00:00:00Z").is_ok());
    assert!(ZonedDateTime::from_iso("2023-02-29T00:00:00Z").is_err());
}

// ─── #plus({ years }) ─────────────────────────────────────────────────────────

#[test]
fn plus_years_1() {
    let z = ymdhms(2010, 2, 3, 4, 5, 6).plus(Duration::years(1));
    assert_eq!(z.year(), 2011);
}

#[test]
fn plus_quarter_1() {
    // +3 months = quarter
    let z = ymdhms(2010, 2, 3, 4, 5, 6).plus(Duration::months(3));
    assert_eq!(z.month(), 5);
}

#[test]
fn plus_months_1_end_of_month() {
    let z = dt("2018-01-31T10:00:00Z").plus(Duration::months(1));
    assert_eq!(z.day(), 28);
    assert_eq!(z.month(), 2);
}

#[test]
fn plus_months_1_leap_year_end() {
    let z = dt("2016-01-31T10:00:00Z").plus(Duration::months(1));
    assert_eq!(z.day(), 29);
    assert_eq!(z.month(), 2);
}

#[test]
fn plus_months_13_end_of_month() {
    let z = dt("2015-01-31T10:00:00Z").plus(Duration::months(13));
    assert_eq!(z.day(), 29);
    assert_eq!(z.month(), 2);
    assert_eq!(z.year(), 2016);
}

#[test]
fn plus_days_1_across_month() {
    let z = ymd(2025, 1, 31).plus(Duration::days(1));
    assert_eq!((z.month(), z.day()), (2, 1));
}

#[test]
fn plus_duration_builder() {
    // +1 day, +3 hours, +28 minutes
    let z = dt("2016-03-12T10:13:00Z")
        .plus(Duration::builder().days(1).hours(3).minutes(28).build());
    assert_eq!(z.day(), 13);
    assert_eq!(z.hour(), 13);
    assert_eq!(z.minute(), 41);
}

// ─── #minus ───────────────────────────────────────────────────────────────────

#[test]
fn minus_days_7() {
    let z = ymd(2025, 1, 8).minus(Duration::days(7));
    assert_eq!(z.day(), 1);
}

#[test]
fn minus_months_1() {
    let z = dt("2025-03-31T10:00:00Z").minus(Duration::months(1));
    assert_eq!(z.month(), 2);
    // Feb 2025 has 28 days
    assert_eq!(z.day(), 28);
}

// ─── #diff ────────────────────────────────────────────────────────────────────

#[test]
fn diff_default_milliseconds() {
    let a = dt("2017-01-01T00:00:00.012Z");
    let b = dt("2017-01-01T00:00:00.000Z");
    assert_eq!(a.diff(b, "milliseconds").unwrap().num_milliseconds(), 12);
}

#[test]
fn diff_years_0() {
    let a = ymd(2017, 1, 1);
    let b = ymd(2017, 1, 1);
    assert_eq!(a.diff(b, "years").unwrap().num_years(), 0);
}

#[test]
fn diff_years_1() {
    let a = ymd(2017, 1, 1);
    let b = ymd(2016, 1, 1);
    assert_eq!(a.diff(b, "years").unwrap().num_years(), 1);
}

#[test]
fn diff_months_1() {
    let a = dt("2016-06-28T00:00:00Z");
    let b = dt("2016-05-28T00:00:00Z");
    assert_eq!(a.diff(b, "months").unwrap().num_months(), 1);
}

#[test]
fn diff_days_3() {
    let a = dt("2016-06-28T00:00:00Z");
    let b = dt("2016-06-25T00:00:00Z");
    assert_eq!(a.diff(b, "days").unwrap().num_days(), 3);
}

#[test]
fn diff_days_4_across_month() {
    let a = dt("2016-06-01T00:00:00Z");
    let b = dt("2016-05-28T00:00:00Z");
    assert_eq!(a.diff(b, "days").unwrap().num_days(), 4);
}

#[test]
fn diff_weeks_4() {
    let a = dt("2016-06-29T00:00:00Z");
    let b = dt("2016-06-01T00:00:00Z");
    assert_eq!(a.diff(b, "weeks").unwrap().num_weeks(), 4);
}

#[test]
fn diff_weeks_2() {
    let a = dt("2016-03-03T00:00:00Z");
    let b = dt("2016-02-18T00:00:00Z");
    assert_eq!(a.diff(b, "weeks").unwrap().num_weeks(), 2);
}

#[test]
fn diff_hours_8() {
    let a = dt("2016-06-28T13:00:00Z");
    let b = dt("2016-06-28T05:00:00Z");
    assert_eq!(a.diff(b, "hours").unwrap().num_hours(), 8);
}

#[test]
fn diff_hours_fractional_as_days() {
    // 8h difference expressed in days = 1/3
    let a = dt("2016-06-28T13:00:00Z");
    let b = dt("2016-06-28T05:00:00Z");
    // diff in days rounds toward zero
    assert_eq!(a.diff(b, "days").unwrap().num_days(), 0);
}

#[test]
fn diff_hours_across_3_days_and_8() {
    let a = dt("2016-06-28T13:00:00Z");
    let b = dt("2016-06-25T05:00:00Z");
    assert_eq!(a.diff(b, "hours").unwrap().num_hours(), 24 * 3 + 8);
}

// ─── #startOf ─────────────────────────────────────────────────────────────────

#[test]
fn start_of_day() {
    let z = ymdhms(2025, 6, 7, 14, 32, 5).start_of("day").unwrap();
    assert_eq!((z.hour(), z.minute(), z.second(), z.nanosecond()), (0, 0, 0, 0));
    assert_eq!((z.year(), z.month(), z.day()), (2025, 6, 7));
}

#[test]
fn start_of_month() {
    let z = ymdhms(2025, 6, 15, 14, 0, 0).start_of("month").unwrap();
    assert_eq!((z.day(), z.hour()), (1, 0));
}

#[test]
fn start_of_year() {
    let z = ymdhms(2025, 6, 15, 14, 0, 0).start_of("year").unwrap();
    assert_eq!((z.month(), z.day(), z.hour()), (1, 1, 0));
}

#[test]
fn start_of_quarter() {
    // Q2 starts in April
    let z = ymdhms(2025, 5, 15, 14, 0, 0).start_of("quarter").unwrap();
    assert_eq!((z.month(), z.day()), (4, 1));
}

#[test]
fn start_of_week_iso() {
    // 2025-06-07 = Saturday (ISO 6); week starts on Monday 2025-06-02
    let z = ymdhms(2025, 6, 7, 14, 0, 0).start_of("week").unwrap();
    assert_eq!((z.year(), z.month(), z.day()), (2025, 6, 2));
}

#[test]
fn start_of_hour() {
    let z = ymdhms(2025, 6, 7, 14, 32, 5).start_of("hour").unwrap();
    assert_eq!((z.minute(), z.second()), (0, 0));
    assert_eq!(z.hour(), 14);
}

#[test]
fn start_of_minute() {
    let z = ymdhms(2025, 6, 7, 14, 32, 5).start_of("minute").unwrap();
    assert_eq!(z.second(), 0);
    assert_eq!((z.hour(), z.minute()), (14, 32));
}

// ─── #endOf ───────────────────────────────────────────────────────────────────

#[test]
fn end_of_day() {
    let z = ymdhms(2025, 6, 7, 14, 32, 5).end_of("day").unwrap();
    assert_eq!((z.hour(), z.minute(), z.second()), (23, 59, 59));
    assert_eq!(z.nanosecond(), 999_999_999);
    assert_eq!((z.year(), z.month(), z.day()), (2025, 6, 7));
}

#[test]
fn end_of_month() {
    let z = ymd(2025, 6, 7).end_of("month").unwrap();
    assert_eq!((z.month(), z.day()), (6, 30));
    assert_eq!(z.hour(), 23);
}

#[test]
fn end_of_year() {
    let z = ymd(2025, 6, 7).end_of("year").unwrap();
    assert_eq!((z.month(), z.day()), (12, 31));
}

#[test]
fn end_of_month_feb_non_leap() {
    let z = ymd(2025, 2, 1).end_of("month").unwrap();
    assert_eq!(z.day(), 28);
}

#[test]
fn end_of_month_feb_leap() {
    let z = ymd(2024, 2, 1).end_of("month").unwrap();
    assert_eq!(z.day(), 29);
}

// ─── #format ─────────────────────────────────────────────────────────────────

#[test]
fn format_yyyy_mm_dd() {
    assert_eq!(dt("2025-06-07T14:32:00Z").format("yyyy-MM-dd"), "2025-06-07");
}

#[test]
fn format_hhmmss() {
    assert_eq!(dt("2025-06-07T14:32:05Z").format("HH:mm:ss"), "14:32:05");
}

#[test]
fn format_sss_milliseconds() {
    assert_eq!(dt("2025-06-07T14:32:00.123Z").format("SSS"), "123");
}

#[test]
fn format_strftime_percent_y() {
    assert_eq!(dt("2025-06-07T00:00:00Z").format("%Y"), "2025");
}

#[test]
fn format_strftime_iso() {
    assert_eq!(
        dt("2025-06-07T14:32:05Z").format("%Y-%m-%dT%H:%M:%S"),
        "2025-06-07T14:32:05"
    );
}

#[test]
fn format_month_name_full() {
    assert_eq!(dt("2025-06-07T00:00:00Z").format("MMMM"), "June");
}

#[test]
fn format_month_name_short() {
    assert_eq!(dt("2025-06-07T00:00:00Z").format("MMM"), "Jun");
}

#[test]
fn format_weekday_full() {
    // 2025-06-07 is Saturday
    assert_eq!(dt("2025-06-07T00:00:00Z").format("EEEE"), "Saturday");
}

#[test]
fn format_weekday_short() {
    assert_eq!(dt("2025-06-07T00:00:00Z").format("EEE"), "Sat");
}

#[test]
fn format_offset() {
    let z = dt("2025-06-07T14:32:00-04:00");
    assert_eq!(z.format("%Z"), "-04:00");
}

// ─── Accessors ────────────────────────────────────────────────────────────────

#[test]
fn accessor_year() {
    assert_eq!(dt("2025-06-07T14:32:00Z").year(), 2025);
}

#[test]
fn accessor_month() {
    assert_eq!(dt("2025-06-07T14:32:00Z").month(), 6);
}

#[test]
fn accessor_day() {
    assert_eq!(dt("2025-06-07T14:32:00Z").day(), 7);
}

#[test]
fn accessor_hour() {
    assert_eq!(dt("2025-06-07T14:32:00Z").hour(), 14);
}

#[test]
fn accessor_minute() {
    assert_eq!(dt("2025-06-07T14:32:00Z").minute(), 32);
}

#[test]
fn accessor_second() {
    assert_eq!(dt("2025-06-07T14:32:05Z").second(), 5);
}

#[test]
fn accessor_weekday_wednesday() {
    // 2025-01-01 = Wednesday = ISO 3
    assert_eq!(dt("2025-01-01T00:00:00Z").weekday(), 3);
}

#[test]
fn accessor_weekday_thursday() {
    // 1970-01-01 = Thursday = ISO 4
    assert_eq!(dt("1970-01-01T00:00:00Z").weekday(), 4);
}

// ─── to_iso ───────────────────────────────────────────────────────────────────

#[test]
fn to_iso_utc() {
    let s = dt("2025-06-07T14:32:00Z").to_iso();
    assert!(s.starts_with("2025-06-07T14:32:00.000+00:00"), "got: {s}");
}

#[test]
fn to_iso_with_offset() {
    let z = dt("2025-06-07T14:32:00-04:00");
    let s = z.to_iso();
    assert!(s.contains("-04:00"), "got: {s}");
}

// ─── Timezone tests ──────────────────────────────────────────────────────────

#[cfg(any(feature = "tz-embedded", feature = "tz-system"))]
#[test]
fn in_timezone_utc_to_ny_winter() {
    // EST = UTC-5 in January
    let utc = dt("2025-01-01T05:00:00Z");
    let ny = utc.in_timezone("America/New_York").unwrap();
    assert_eq!(ny.hour(), 0);
    assert_eq!(ny.timezone(), "America/New_York");
    assert_eq!(ny.offset_seconds(), -5 * 3600);
}

#[cfg(any(feature = "tz-embedded", feature = "tz-system"))]
#[test]
fn in_timezone_utc_to_ny_summer() {
    // EDT = UTC-4 in July
    let utc = dt("2025-07-01T04:00:00Z");
    let ny = utc.in_timezone("America/New_York").unwrap();
    assert_eq!(ny.hour(), 0);
    assert_eq!(ny.offset_seconds(), -4 * 3600);
}

#[cfg(any(feature = "tz-embedded", feature = "tz-system"))]
#[test]
fn in_timezone_invalid_returns_err() {
    assert!(dt("2025-01-01T00:00:00Z")
        .in_timezone("Not/ATimezone")
        .is_err());
}

#[cfg(any(feature = "tz-embedded", feature = "tz-system"))]
#[test]
fn plus_days_preserves_wall_clock_across_dst_spring() {
    // 2016-03-12T10:00 Los Angeles + 1 day → 2016-03-13T10:00
    // (spring forward happened 2016-03-13 02:00 → 03:00)
    let z = ZonedDateTime::from_iso("2016-03-12T10:00:00-08:00[America/Los_Angeles]").unwrap();
    let later = z.plus(Duration::days(1));
    assert_eq!(later.day(), 13);
    assert_eq!(later.hour(), 10);
}

#[cfg(any(feature = "tz-embedded", feature = "tz-system"))]
#[test]
fn plus_hours_24_changes_wall_clock_across_dst() {
    // 2016-03-12T10:00 + 24 hours → 2016-03-13T11:00 (spring forward)
    let z = ZonedDateTime::from_iso("2016-03-12T10:00:00-08:00[America/Los_Angeles]").unwrap();
    let later = z.plus(Duration::hours(24));
    assert_eq!(later.day(), 13);
    assert_eq!(later.hour(), 11);
}

// ─── unix_millis / unix_seconds ───────────────────────────────────────────────

#[test]
fn unix_seconds_epoch() {
    assert_eq!(dt("1970-01-01T00:00:00Z").unix_seconds(), 0);
}

#[test]
fn unix_seconds_known() {
    // 2025-01-01T00:00:00Z
    assert_eq!(dt("2025-01-01T00:00:00Z").unix_seconds(), 1_735_689_600);
}

// ─── Ordering ─────────────────────────────────────────────────────────────────

#[test]
fn ordering() {
    let a = dt("2025-01-01T00:00:00Z");
    let b = dt("2025-01-02T00:00:00Z");
    assert!(a < b);
    assert!(b > a);
    assert_eq!(a, a);
}
