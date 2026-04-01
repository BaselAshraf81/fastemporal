//! Tests ported from the TC39 Temporal proposal test suite.
//!
//! Sources:
//!   - proposal-temporal/polyfill/test/
//!   - temporal-test262-runner
//! (BSD-style / MIT-compatible licenses)
//!
//! Covers: PlainDate, PlainTime, PlainDateTime, ZonedDateTime, Duration.

use fastemporal::{Duration, PlainDate, PlainDateTime, PlainTime, ZonedDateTime};

// ─── PlainDate ────────────────────────────────────────────────────────────────

#[test]
fn plain_date_from_fields_valid() {
    let d = PlainDate::new(1976, 11, 18).unwrap();
    assert_eq!(d.year(), 1976);
    assert_eq!(d.month(), 11);
    assert_eq!(d.day(), 18);
}

#[test]
fn plain_date_from_fields_invalid_month() {
    assert!(PlainDate::new(2019, 13, 1).is_none());
    assert!(PlainDate::new(2019, 0, 1).is_none());
}

#[test]
fn plain_date_from_fields_invalid_day() {
    assert!(PlainDate::new(2019, 1, 0).is_none());
    assert!(PlainDate::new(2019, 1, 32).is_none());
    assert!(PlainDate::new(2019, 2, 29).is_none()); // 2019 not a leap year
}

#[test]
fn plain_date_leap_day() {
    assert!(PlainDate::new(2020, 2, 29).is_some()); // 2020 is leap
}

#[test]
fn plain_date_from_iso_basic() {
    let d = PlainDate::from_iso("1976-11-18").unwrap();
    assert_eq!((d.year(), d.month(), d.day()), (1976, 11, 18));
}

#[test]
fn plain_date_to_iso_string() {
    let d = PlainDate::new(1976, 11, 18).unwrap();
    assert_eq!(d.to_iso(), "1976-11-18");
    assert_eq!(d.to_string(), "1976-11-18");
}

#[test]
fn plain_date_add_days_basic() {
    let d = PlainDate::new(1976, 11, 18).unwrap().add_days(7);
    assert_eq!((d.year(), d.month(), d.day()), (1976, 11, 25));
}

#[test]
fn plain_date_add_days_crosses_month() {
    let d = PlainDate::new(2019, 10, 31).unwrap().add_days(1);
    assert_eq!((d.month(), d.day()), (11, 1));
}

#[test]
fn plain_date_add_days_crosses_year() {
    let d = PlainDate::new(2019, 12, 31).unwrap().add_days(1);
    assert_eq!((d.year(), d.month(), d.day()), (2020, 1, 1));
}

#[test]
fn plain_date_add_negative_days() {
    let d = PlainDate::new(2019, 11, 5).unwrap().add_days(-7);
    assert_eq!((d.month(), d.day()), (10, 29));
}

#[test]
fn plain_date_add_months_clamps() {
    let d = PlainDate::new(2019, 1, 31).unwrap().add_months(1);
    assert_eq!((d.month(), d.day()), (2, 28));
}

#[test]
fn plain_date_add_years_clamps_leap() {
    let d = PlainDate::new(2020, 2, 29).unwrap().add_years(1);
    assert_eq!((d.year(), d.month(), d.day()), (2021, 2, 28));
}

#[test]
fn plain_date_days_until_positive() {
    let a = PlainDate::new(2019, 11, 18).unwrap();
    let b = PlainDate::new(2019, 11, 1).unwrap();
    assert_eq!(a.days_until(b), 17);
}

#[test]
fn plain_date_days_until_negative() {
    let a = PlainDate::new(2019, 11, 1).unwrap();
    let b = PlainDate::new(2019, 11, 18).unwrap();
    assert_eq!(a.days_until(b), -17);
}

#[test]
fn plain_date_weekday_values() {
    // Known weekdays:
    // 1976-11-18 = Thursday = ISO 4
    assert_eq!(PlainDate::new(1976, 11, 18).unwrap().weekday(), 4);
    // 2021-09-13 = Monday = ISO 1
    assert_eq!(PlainDate::new(2021, 9, 13).unwrap().weekday(), 1);
    // 2021-09-19 = Sunday = ISO 7
    assert_eq!(PlainDate::new(2021, 9, 19).unwrap().weekday(), 7);
}

#[test]
fn plain_date_day_of_year() {
    assert_eq!(PlainDate::new(2019, 1, 1).unwrap().day_of_year(), 1);
    assert_eq!(PlainDate::new(2019, 12, 31).unwrap().day_of_year(), 365);
    assert_eq!(PlainDate::new(2020, 12, 31).unwrap().day_of_year(), 366);
}

#[test]
fn plain_date_in_leap_year() {
    assert!(PlainDate::new(2020, 1, 1).unwrap().in_leap_year());
    assert!(!PlainDate::new(2019, 1, 1).unwrap().in_leap_year());
    assert!(PlainDate::new(2000, 1, 1).unwrap().in_leap_year());
    assert!(!PlainDate::new(1900, 1, 1).unwrap().in_leap_year());
}

#[test]
fn plain_date_days_in_month() {
    assert_eq!(PlainDate::new(2019, 2, 1).unwrap().days_in_month(), 28);
    assert_eq!(PlainDate::new(2020, 2, 1).unwrap().days_in_month(), 29);
    assert_eq!(PlainDate::new(2019, 1, 1).unwrap().days_in_month(), 31);
    assert_eq!(PlainDate::new(2019, 4, 1).unwrap().days_in_month(), 30);
}

#[test]
fn plain_date_ordering() {
    let a = PlainDate::new(2019, 1, 1).unwrap();
    let b = PlainDate::new(2019, 1, 2).unwrap();
    assert!(a < b);
    assert!(b > a);
    assert_eq!(a, a);
}

// ─── PlainTime ────────────────────────────────────────────────────────────────

#[test]
fn plain_time_from_fields() {
    let t = PlainTime::new(15, 23, 30, 123_456_789).unwrap();
    assert_eq!(t.hour(), 15);
    assert_eq!(t.minute(), 23);
    assert_eq!(t.second(), 30);
    assert_eq!(t.nanosecond(), 123_456_789);
    assert_eq!(t.millisecond(), 123);
    assert_eq!(t.microsecond(), 456);
}

#[test]
fn plain_time_invalid_hour() {
    assert!(PlainTime::new(24, 0, 0, 0).is_none());
}

#[test]
fn plain_time_invalid_minute() {
    assert!(PlainTime::new(0, 60, 0, 0).is_none());
}

#[test]
fn plain_time_invalid_second() {
    assert!(PlainTime::new(0, 0, 60, 0).is_none());
}

#[test]
fn plain_time_invalid_nanosecond() {
    assert!(PlainTime::new(0, 0, 0, 1_000_000_000).is_none());
}

#[test]
fn plain_time_midnight() {
    let t = PlainTime::MIDNIGHT;
    assert_eq!((t.hour(), t.minute(), t.second(), t.nanosecond()), (0, 0, 0, 0));
}

#[test]
fn plain_time_to_iso_no_frac() {
    let t = PlainTime::new(15, 23, 30, 0).unwrap();
    assert_eq!(t.to_iso(), "15:23:30");
}

#[test]
fn plain_time_to_iso_with_nanos() {
    let t = PlainTime::new(15, 23, 30, 123_456_789).unwrap();
    assert_eq!(t.to_iso(), "15:23:30.123456789");
}

#[test]
fn plain_time_from_iso() {
    let t = PlainTime::from_iso("15:23:30").unwrap();
    assert_eq!((t.hour(), t.minute(), t.second()), (15, 23, 30));
}

#[test]
fn plain_time_total_nanoseconds() {
    let t = PlainTime::new(1, 0, 0, 0).unwrap();
    assert_eq!(t.total_nanoseconds(), 3_600_000_000_000);
}

#[test]
fn plain_time_ordering() {
    let a = PlainTime::new(10, 0, 0, 0).unwrap();
    let b = PlainTime::new(11, 0, 0, 0).unwrap();
    assert!(a < b);
}

// ─── PlainDateTime ───────────────────────────────────────────────────────────

#[test]
fn plain_datetime_from_fields() {
    let dt = PlainDateTime::new(1976, 11, 18, 15, 23, 30, 123_456_789).unwrap();
    assert_eq!(dt.year(), 1976);
    assert_eq!(dt.month(), 11);
    assert_eq!(dt.day(), 18);
    assert_eq!(dt.hour(), 15);
    assert_eq!(dt.minute(), 23);
    assert_eq!(dt.second(), 30);
    assert_eq!(dt.nanosecond(), 123_456_789);
}

#[test]
fn plain_datetime_invalid_fields() {
    assert!(PlainDateTime::new(2019, 13, 1, 0, 0, 0, 0).is_none());
    assert!(PlainDateTime::new(2019, 1, 1, 24, 0, 0, 0).is_none());
}

#[test]
fn plain_datetime_from_iso() {
    let dt = PlainDateTime::from_iso("1976-11-18T15:23:30").unwrap();
    assert_eq!((dt.year(), dt.month(), dt.day()), (1976, 11, 18));
    assert_eq!((dt.hour(), dt.minute(), dt.second()), (15, 23, 30));
}

#[test]
fn plain_datetime_to_iso_no_frac() {
    let dt = PlainDateTime::new(1976, 11, 18, 15, 23, 30, 0).unwrap();
    assert_eq!(dt.to_iso(), "1976-11-18T15:23:30");
}

#[test]
fn plain_datetime_to_iso_with_nanos() {
    let dt = PlainDateTime::new(1976, 11, 18, 15, 23, 30, 123_456_789).unwrap();
    assert!(dt.to_iso().contains(".123456789"), "{}", dt.to_iso());
}

#[test]
fn plain_datetime_add_days_crosses_year() {
    let dt = PlainDateTime::new(2019, 12, 31, 12, 0, 0, 0).unwrap().add_days(1);
    assert_eq!((dt.year(), dt.month(), dt.day()), (2020, 1, 1));
}

#[test]
fn plain_datetime_date_component() {
    let dt = PlainDateTime::new(1976, 11, 18, 15, 23, 30, 0).unwrap();
    let d = dt.date();
    assert_eq!((d.year(), d.month(), d.day()), (1976, 11, 18));
}

#[test]
fn plain_datetime_time_component() {
    let dt = PlainDateTime::new(1976, 11, 18, 15, 23, 30, 0).unwrap();
    let t = dt.time();
    assert_eq!((t.hour(), t.minute(), t.second()), (15, 23, 30));
}

// ─── Duration ────────────────────────────────────────────────────────────────

#[test]
fn duration_zero_is_zero() {
    assert!(Duration::zero().is_zero());
}

#[test]
fn duration_fields() {
    let d = Duration::builder()
        .years(1).months(2).weeks(3).days(4)
        .hours(5).minutes(6).seconds(7).millis(8).micros(9).nanos(10)
        .build();
    assert_eq!(d.num_years(), 1);
    assert_eq!(d.num_months(), 2);
    assert_eq!(d.num_weeks(), 3);
    assert_eq!(d.num_days(), 4);
    assert_eq!(d.num_hours(), 5);
    assert_eq!(d.num_minutes(), 6);
    assert_eq!(d.num_seconds(), 7);
    assert_eq!(d.num_milliseconds(), 8);
    assert_eq!(d.num_microseconds(), 9);
    assert_eq!(d.num_nanoseconds(), 10);
}

#[test]
fn duration_negate() {
    let d = Duration::builder().years(1).months(-2).days(3).build().negate();
    assert_eq!(d.num_years(), -1);
    assert_eq!(d.num_months(), 2);
    assert_eq!(d.num_days(), -3);
}

#[test]
fn duration_neg_operator() {
    let d = -Duration::days(5);
    assert_eq!(d.num_days(), -5);
}

#[test]
fn duration_display_years() {
    assert_eq!(Duration::years(3).to_string(), "P3Y");
}

#[test]
fn duration_display_months() {
    assert_eq!(Duration::months(2).to_string(), "P2M");
}

#[test]
fn duration_display_weeks() {
    assert_eq!(Duration::weeks(1).to_string(), "P1W");
}

#[test]
fn duration_display_days() {
    assert_eq!(Duration::days(7).to_string(), "P7D");
}

#[test]
fn duration_display_hours() {
    assert_eq!(Duration::hours(2).to_string(), "PT2H");
}

#[test]
fn duration_display_zero() {
    assert_eq!(Duration::zero().to_string(), "PT0S");
}

#[test]
fn duration_clock_nanos() {
    let d = Duration::builder().hours(1).minutes(30).seconds(45).build();
    let expected = 1 * 3_600_000_000_000_i64
        + 30 * 60_000_000_000_i64
        + 45 * 1_000_000_000_i64;
    assert_eq!(d.num_hours() as i64 * 3_600_000_000_000 + d.num_minutes() as i64 * 60_000_000_000 + d.num_seconds() as i64 * 1_000_000_000, expected);
}

// ─── ZonedDateTime (Temporal semantics) ──────────────────────────────────────

#[test]
fn zdt_from_iso_with_offset_and_tz() {
    let z = ZonedDateTime::from_iso("2020-01-01T00:00:00+00:00[UTC]").unwrap();
    assert_eq!(z.year(), 2020);
    assert_eq!(z.timezone(), "UTC");
}

#[test]
fn zdt_unix_epoch() {
    let z = ZonedDateTime::from_iso("1970-01-01T00:00:00Z").unwrap();
    assert_eq!(z.unix_seconds(), 0);
    assert_eq!(z.unix_nanos(), 0);
}

#[test]
fn zdt_unix_millis() {
    let z = ZonedDateTime::from_iso("1970-01-01T00:00:00.001Z").unwrap();
    assert_eq!(z.unix_millis(), 1);
}

#[test]
fn zdt_negative_epoch() {
    // 1969-12-31T23:59:59Z = -1 second
    let z = ZonedDateTime::from_iso("1969-12-31T23:59:59Z").unwrap();
    assert_eq!(z.unix_seconds(), -1);
}

#[test]
fn zdt_plus_years() {
    let z = ZonedDateTime::from_iso("2019-11-18T15:23:30Z").unwrap()
        .plus(Duration::years(1));
    assert_eq!(z.year(), 2020);
    assert_eq!((z.month(), z.day()), (11, 18));
}

#[test]
fn zdt_plus_months_clamps() {
    let z = ZonedDateTime::from_iso("2019-01-31T15:00:00Z").unwrap()
        .plus(Duration::months(1));
    assert_eq!((z.month(), z.day()), (2, 28));
}

#[test]
fn zdt_minus_hours() {
    let z = ZonedDateTime::from_iso("2020-01-01T05:00:00Z").unwrap()
        .minus(Duration::hours(5));
    assert_eq!((z.year(), z.month(), z.day()), (2020, 1, 1));
    assert_eq!(z.hour(), 0);
}

#[test]
fn zdt_diff_days() {
    let a = ZonedDateTime::from_iso("2020-02-01T00:00:00Z").unwrap();
    let b = ZonedDateTime::from_iso("2020-01-01T00:00:00Z").unwrap();
    assert_eq!(a.diff(b, "days").unwrap().num_days(), 31);
}

#[test]
fn zdt_diff_hours() {
    let a = ZonedDateTime::from_iso("2020-01-01T06:00:00Z").unwrap();
    let b = ZonedDateTime::from_iso("2020-01-01T00:00:00Z").unwrap();
    assert_eq!(a.diff(b, "hours").unwrap().num_hours(), 6);
}

#[test]
fn zdt_diff_milliseconds() {
    let a = ZonedDateTime::from_iso("2020-01-01T00:00:00.500Z").unwrap();
    let b = ZonedDateTime::from_iso("2020-01-01T00:00:00.000Z").unwrap();
    assert_eq!(a.diff(b, "milliseconds").unwrap().num_milliseconds(), 500);
}

#[test]
fn zdt_start_of_day() {
    let z = ZonedDateTime::from_iso("2020-03-15T14:30:00Z").unwrap()
        .start_of("day").unwrap();
    assert_eq!((z.hour(), z.minute(), z.second()), (0, 0, 0));
    assert_eq!((z.year(), z.month(), z.day()), (2020, 3, 15));
}

#[test]
fn zdt_end_of_day() {
    let z = ZonedDateTime::from_iso("2020-03-15T14:30:00Z").unwrap()
        .end_of("day").unwrap();
    assert_eq!((z.hour(), z.minute(), z.second()), (23, 59, 59));
    assert_eq!(z.nanosecond(), 999_999_999);
}

#[test]
fn zdt_start_of_month() {
    let z = ZonedDateTime::from_iso("2020-03-15T00:00:00Z").unwrap()
        .start_of("month").unwrap();
    assert_eq!((z.month(), z.day()), (3, 1));
}

#[test]
fn zdt_to_plain_date() {
    let z = ZonedDateTime::from_iso("2020-03-15T14:30:00Z").unwrap();
    let d = z.to_plain_date();
    assert_eq!((d.year(), d.month(), d.day()), (2020, 3, 15));
}

#[test]
fn zdt_to_plain_time() {
    let z = ZonedDateTime::from_iso("2020-03-15T14:30:05Z").unwrap();
    let t = z.to_plain_time();
    assert_eq!((t.hour(), t.minute(), t.second()), (14, 30, 5));
}

#[test]
fn zdt_to_plain_datetime() {
    let z = ZonedDateTime::from_iso("2020-03-15T14:30:05Z").unwrap();
    let dt = z.to_plain_datetime();
    assert_eq!((dt.year(), dt.month(), dt.day()), (2020, 3, 15));
    assert_eq!((dt.hour(), dt.minute(), dt.second()), (14, 30, 5));
}

#[test]
fn zdt_format() {
    let z = ZonedDateTime::from_iso("1976-11-18T15:23:30Z").unwrap();
    assert_eq!(z.format("yyyy-MM-dd"), "1976-11-18");
    assert_eq!(z.format("HH:mm:ss"), "15:23:30");
}

#[test]
fn zdt_from_unix_nanos() {
    let z = ZonedDateTime::from_unix_nanos(0, "UTC").unwrap();
    assert_eq!(z.year(), 1970);
    assert_eq!((z.month(), z.day()), (1, 1));
}

#[test]
fn zdt_ordering_by_instant() {
    let a = ZonedDateTime::from_iso("2020-01-01T00:00:00Z").unwrap();
    let b = ZonedDateTime::from_iso("2020-01-02T00:00:00Z").unwrap();
    assert!(a < b);
    assert!(b > a);
    assert_eq!(a, a);
}

#[cfg(any(feature = "tz-embedded", feature = "tz-system"))]
#[test]
fn zdt_in_timezone_preserves_instant() {
    let utc = ZonedDateTime::from_iso("2020-01-01T12:00:00Z").unwrap();
    let ny = utc.in_timezone("America/New_York").unwrap();
    // The instant must be identical
    assert_eq!(utc.unix_seconds(), ny.unix_seconds());
    // The offset must be -5h in January
    assert_eq!(ny.offset_seconds(), -5 * 3600);
    assert_eq!(ny.hour(), 7); // 12 - 5
}

#[cfg(any(feature = "tz-embedded", feature = "tz-system"))]
#[test]
fn zdt_bracket_tz_parsed() {
    let z = ZonedDateTime::from_iso("2020-07-01T00:00:00-04:00[America/New_York]").unwrap();
    assert_eq!(z.timezone(), "America/New_York");
    // July = EDT = UTC-4
    assert_eq!(z.offset_seconds(), -4 * 3600);
}
