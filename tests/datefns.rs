//! Tests ported from the date-fns test suite.
//!
//! Source: https://github.com/date-fns/date-fns (MIT License)
//! Functions covered: addDays, subDays, differenceInDays, format, parseISO,
//!   addMonths, addYears, startOfDay, endOfDay, startOfMonth, endOfMonth,
//!   startOfYear, endOfYear, isLeapYear, getDaysInMonth.

use fastemporal::{Duration, PlainDate, ZonedDateTime};

fn dt(s: &str) -> ZonedDateTime {
    ZonedDateTime::from_iso(s).unwrap_or_else(|e| panic!("parse '{s}': {e}"))
}

// ─── addDays ─────────────────────────────────────────────────────────────────

#[test]
fn add_days_basic() {
    // new Date(2014, 8, 1) + 10 days = Sep 11
    let d = dt("2014-09-01T00:00:00Z").plus(Duration::days(10));
    assert_eq!((d.year(), d.month(), d.day()), (2014, 9, 11));
}

#[test]
fn add_days_zero() {
    let d = dt("2014-09-01T00:00:00Z").plus(Duration::days(0));
    assert_eq!((d.year(), d.month(), d.day()), (2014, 9, 1));
}

#[test]
fn add_days_negative() {
    let d = dt("2014-09-11T00:00:00Z").plus(Duration::days(-10));
    assert_eq!((d.year(), d.month(), d.day()), (2014, 9, 1));
}

#[test]
fn add_days_crosses_year() {
    let d = dt("2014-12-31T00:00:00Z").plus(Duration::days(1));
    assert_eq!((d.year(), d.month(), d.day()), (2015, 1, 1));
}

#[test]
fn add_days_leap() {
    // 2012-02-28 + 1 = 2012-02-29 (leap)
    let d = dt("2012-02-28T00:00:00Z").plus(Duration::days(1));
    assert_eq!((d.month(), d.day()), (2, 29));
}

#[test]
fn add_days_past_leap() {
    // 2012-02-29 + 1 = 2012-03-01
    let d = dt("2012-02-29T00:00:00Z").plus(Duration::days(1));
    assert_eq!((d.month(), d.day()), (3, 1));
}

// ─── subDays ─────────────────────────────────────────────────────────────────

#[test]
fn sub_days_basic() {
    let d = dt("2014-09-11T00:00:00Z").minus(Duration::days(10));
    assert_eq!((d.year(), d.month(), d.day()), (2014, 9, 1));
}

#[test]
fn sub_days_crosses_month() {
    let d = dt("2014-09-01T00:00:00Z").minus(Duration::days(1));
    assert_eq!((d.month(), d.day()), (8, 31));
}

// ─── differenceInDays ────────────────────────────────────────────────────────

#[test]
fn difference_in_days_positive() {
    let a = dt("2014-09-11T00:00:00Z");
    let b = dt("2014-09-01T00:00:00Z");
    assert_eq!(a.diff(b, "days").unwrap().num_days(), 10);
}

#[test]
fn difference_in_days_negative() {
    let a = dt("2014-09-01T00:00:00Z");
    let b = dt("2014-09-11T00:00:00Z");
    assert_eq!(a.diff(b, "days").unwrap().num_days(), -10);
}

#[test]
fn difference_in_days_zero() {
    let a = dt("2014-09-01T00:00:00Z");
    let b = dt("2014-09-01T00:00:00Z");
    assert_eq!(a.diff(b, "days").unwrap().num_days(), 0);
}

#[test]
fn difference_in_days_same_month_different_time() {
    // Less than a full day difference in wall clock, still 0 days
    let a = dt("2014-09-05T06:00:00Z");
    let b = dt("2014-09-04T06:00:00Z");
    assert_eq!(a.diff(b, "days").unwrap().num_days(), 1);
}

#[test]
fn difference_in_days_across_year() {
    let a = dt("2014-01-01T00:00:00Z");
    let b = dt("2013-01-01T00:00:00Z");
    assert_eq!(a.diff(b, "days").unwrap().num_days(), 365);
}

#[test]
fn difference_in_days_across_leap_year() {
    let a = dt("2013-01-01T00:00:00Z");
    let b = dt("2012-01-01T00:00:00Z");
    // 2012 is a leap year: 366 days
    assert_eq!(a.diff(b, "days").unwrap().num_days(), 366);
}

// ─── addMonths ───────────────────────────────────────────────────────────────

#[test]
fn add_months_basic() {
    let d = dt("2014-09-01T00:00:00Z").plus(Duration::months(5));
    assert_eq!((d.year(), d.month()), (2015, 2));
}

#[test]
fn add_months_end_of_month_clamp() {
    // Jan 31 + 1 = Feb 28
    let d = dt("2014-01-31T00:00:00Z").plus(Duration::months(1));
    assert_eq!((d.month(), d.day()), (2, 28));
}

#[test]
fn add_months_leap_clamp() {
    // Jan 31 + 1 = Feb 29 in leap year
    let d = dt("2016-01-31T00:00:00Z").plus(Duration::months(1));
    assert_eq!((d.month(), d.day()), (2, 29));
}

#[test]
fn add_months_across_year() {
    let d = dt("2014-09-01T00:00:00Z").plus(Duration::months(13));
    assert_eq!((d.year(), d.month()), (2015, 10));
}

#[test]
fn add_months_negative() {
    let d = dt("2015-02-01T00:00:00Z").plus(Duration::months(-5));
    assert_eq!((d.year(), d.month()), (2014, 9));
}

// ─── addYears ────────────────────────────────────────────────────────────────

#[test]
fn add_years_basic() {
    let d = dt("2014-09-01T00:00:00Z").plus(Duration::years(5));
    assert_eq!(d.year(), 2019);
}

#[test]
fn add_years_leap_clamp() {
    // Feb 29 + 1 year = Feb 28
    let d = dt("2016-02-29T00:00:00Z").plus(Duration::years(1));
    assert_eq!((d.month(), d.day()), (2, 28));
}

#[test]
fn add_years_negative() {
    let d = dt("2014-09-01T00:00:00Z").plus(Duration::years(-5));
    assert_eq!(d.year(), 2009);
}

// ─── format ──────────────────────────────────────────────────────────────────

#[test]
fn format_date_iso() {
    assert_eq!(
        dt("2014-09-01T00:00:00Z").format("yyyy-MM-dd"),
        "2014-09-01"
    );
}

#[test]
fn format_full_datetime() {
    assert_eq!(
        dt("2014-09-01T14:30:05Z").format("yyyy-MM-dd HH:mm:ss"),
        "2014-09-01 14:30:05"
    );
}

#[test]
fn format_month_name() {
    assert_eq!(dt("2014-09-01T00:00:00Z").format("MMMM"), "September");
}

#[test]
fn format_short_month_name() {
    assert_eq!(dt("2014-09-01T00:00:00Z").format("MMM"), "Sep");
}

#[test]
fn format_weekday() {
    // 2014-09-01 = Monday
    assert_eq!(dt("2014-09-01T00:00:00Z").format("EEEE"), "Monday");
}

// ─── parseISO (via from_iso) ─────────────────────────────────────────────────

#[test]
fn parse_iso_basic_date() {
    let d = ZonedDateTime::from_iso("2014-09-01").unwrap();
    assert_eq!((d.year(), d.month(), d.day()), (2014, 9, 1));
}

#[test]
fn parse_iso_datetime_utc() {
    let d = ZonedDateTime::from_iso("2014-09-01T14:30:05Z").unwrap();
    assert_eq!((d.hour(), d.minute(), d.second()), (14, 30, 5));
}

#[test]
fn parse_iso_with_milliseconds() {
    let d = ZonedDateTime::from_iso("2014-09-01T14:30:05.123Z").unwrap();
    assert_eq!(d.millisecond(), 123);
}

#[test]
fn parse_iso_invalid() {
    assert!(ZonedDateTime::from_iso("not-a-date").is_err());
    assert!(ZonedDateTime::from_iso("").is_err());
    assert!(ZonedDateTime::from_iso("2014-13-01").is_err());
}

// ─── startOfDay / endOfDay ───────────────────────────────────────────────────

#[test]
fn start_of_day() {
    let d = dt("2014-09-01T14:30:05.123Z").start_of("day").unwrap();
    assert_eq!((d.hour(), d.minute(), d.second(), d.nanosecond()), (0, 0, 0, 0));
    assert_eq!((d.year(), d.month(), d.day()), (2014, 9, 1));
}

#[test]
fn end_of_day() {
    let d = dt("2014-09-01T14:30:05.123Z").end_of("day").unwrap();
    assert_eq!((d.hour(), d.minute(), d.second()), (23, 59, 59));
    assert_eq!(d.nanosecond(), 999_999_999);
}

// ─── startOfMonth / endOfMonth ───────────────────────────────────────────────

#[test]
fn start_of_month() {
    let d = dt("2014-09-15T12:00:00Z").start_of("month").unwrap();
    assert_eq!((d.month(), d.day(), d.hour()), (9, 1, 0));
}

#[test]
fn end_of_month_30_day() {
    let d = dt("2014-09-01T00:00:00Z").end_of("month").unwrap();
    assert_eq!((d.month(), d.day()), (9, 30));
}

#[test]
fn end_of_month_31_day() {
    let d = dt("2014-08-01T00:00:00Z").end_of("month").unwrap();
    assert_eq!((d.month(), d.day()), (8, 31));
}

#[test]
fn end_of_february_non_leap() {
    let d = dt("2014-02-01T00:00:00Z").end_of("month").unwrap();
    assert_eq!(d.day(), 28);
}

#[test]
fn end_of_february_leap() {
    let d = dt("2016-02-01T00:00:00Z").end_of("month").unwrap();
    assert_eq!(d.day(), 29);
}

// ─── startOfYear / endOfYear ─────────────────────────────────────────────────

#[test]
fn start_of_year() {
    let d = dt("2014-09-15T12:00:00Z").start_of("year").unwrap();
    assert_eq!((d.month(), d.day(), d.hour()), (1, 1, 0));
}

#[test]
fn end_of_year() {
    let d = dt("2014-09-15T12:00:00Z").end_of("year").unwrap();
    assert_eq!((d.month(), d.day()), (12, 31));
}

// ─── isLeapYear ──────────────────────────────────────────────────────────────

#[test]
fn is_leap_year_2016() {
    assert!(PlainDate::new(2016, 1, 1).unwrap().in_leap_year());
}

#[test]
fn is_not_leap_year_2014() {
    assert!(!PlainDate::new(2014, 1, 1).unwrap().in_leap_year());
}

#[test]
fn is_leap_year_2000() {
    assert!(PlainDate::new(2000, 1, 1).unwrap().in_leap_year());
}

#[test]
fn is_not_leap_year_1900() {
    assert!(!PlainDate::new(1900, 1, 1).unwrap().in_leap_year());
}

// ─── getDaysInMonth ──────────────────────────────────────────────────────────

#[test]
fn days_in_month_jan() {
    assert_eq!(PlainDate::new(2014, 1, 1).unwrap().days_in_month(), 31);
}

#[test]
fn days_in_month_feb_non_leap() {
    assert_eq!(PlainDate::new(2014, 2, 1).unwrap().days_in_month(), 28);
}

#[test]
fn days_in_month_feb_leap() {
    assert_eq!(PlainDate::new(2016, 2, 1).unwrap().days_in_month(), 29);
}

#[test]
fn days_in_month_april() {
    assert_eq!(PlainDate::new(2014, 4, 1).unwrap().days_in_month(), 30);
}

// ─── PlainDate arithmetic (date-fns functional style) ────────────────────────

#[test]
fn plain_date_add_days() {
    let d = PlainDate::new(2014, 9, 1).unwrap().add_days(10);
    assert_eq!((d.year(), d.month(), d.day()), (2014, 9, 11));
}

#[test]
fn plain_date_add_months_clamp() {
    let d = PlainDate::new(2014, 1, 31).unwrap().add_months(1);
    assert_eq!((d.month(), d.day()), (2, 28));
}

#[test]
fn plain_date_days_until() {
    let a = PlainDate::new(2014, 9, 11).unwrap();
    let b = PlainDate::new(2014, 9, 1).unwrap();
    assert_eq!(a.days_until(b), 10);
}

#[test]
fn plain_date_from_iso() {
    let d = PlainDate::from_iso("2014-09-01").unwrap();
    assert_eq!((d.year(), d.month(), d.day()), (2014, 9, 1));
}

#[test]
fn plain_date_to_iso() {
    assert_eq!(PlainDate::new(2014, 9, 1).unwrap().to_iso(), "2014-09-01");
}

#[test]
fn plain_date_weekday() {
    // 2014-09-01 = Monday = ISO 1
    assert_eq!(PlainDate::new(2014, 9, 1).unwrap().weekday(), 1);
}
