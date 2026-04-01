#![allow(missing_docs)]
/// Pure calendar-math utilities.  No allocations, no I/O.
///
/// All algorithms are based on Howard Hinnant's public-domain
/// "chrono-Compatible Low-Level Date Algorithms" paper.

// ─── Leap-year / days-in-month ───────────────────────────────────────────────

/// Returns `true` if `year` is a proleptic Gregorian leap year.
#[inline]
pub(crate) const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Returns the number of days in `month` (1-based) of `year`.
#[inline]
pub(crate) const fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0, // caller must never pass an invalid month
    }
}

// ─── Days ↔ civil date (Hinnant algorithm) ───────────────────────────────────

/// Converts a count of days since the Unix epoch (1970-01-01 = day 0) into a
/// proleptic Gregorian `(year, month, day)` triple.  `month` and `day` are
/// 1-based.
pub(crate) const fn civil_from_days(z: i64) -> (i32, u8, u8) {
    let z = z + 719_468;
    let era: i64 = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u8, d as u8)
}

/// Converts a proleptic Gregorian `(year, month, day)` (1-based `month` and
/// `day`) into a count of days since the Unix epoch (1970-01-01 = day 0).
pub(crate) const fn days_from_civil(y: i32, m: u8, d: u8) -> i64 {
    let y: i64 = if m <= 2 { y as i64 - 1 } else { y as i64 };
    let era: i64 = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64; // [0, 399]
    let m_idx: u64 = if m > 2 { m as u64 - 3 } else { m as u64 + 9 }; // [0,11]
    let doy = (153 * m_idx + 2) / 5 + d as u64 - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + doe as i64 - 719_468
}

// ─── Weekday ─────────────────────────────────────────────────────────────────

/// Returns the ISO weekday for the given epoch-day count.
/// 1 = Monday … 7 = Sunday.
#[inline]
pub(crate) const fn weekday_from_days(z: i64) -> u8 {
    // 1970-01-01 was a Thursday (ISO 4)
    let d = z.rem_euclid(7) as u8;
    // shift so Monday = 0, then +1
    (d + 3) % 7 + 1
}

// ─── Nanosecond-level helpers ─────────────────────────────────────────────────

pub(crate) const NANOS_PER_SEC: i64 = 1_000_000_000;
pub(crate) const NANOS_PER_DAY: i64 = 86_400 * NANOS_PER_SEC;

/// Splits an absolute nanosecond timestamp (+ UTC offset in seconds) into
/// `(year, month, day, hour, minute, second, nanosecond)`.
pub(crate) fn local_fields(ts_nanos: i64, offset_secs: i32) -> (i32, u8, u8, u8, u8, u8, u32) {
    let local = ts_nanos + offset_secs as i64 * NANOS_PER_SEC;
    let days = local.div_euclid(NANOS_PER_DAY);
    let nanos_in_day = local.rem_euclid(NANOS_PER_DAY);
    let (y, m, d) = civil_from_days(days);
    let secs_in_day = (nanos_in_day / NANOS_PER_SEC) as u32;
    let h = (secs_in_day / 3_600) as u8;
    let min = ((secs_in_day % 3_600) / 60) as u8;
    let s = (secs_in_day % 60) as u8;
    let ns = (nanos_in_day % NANOS_PER_SEC) as u32;
    (y, m, d, h, min, s, ns)
}

/// Combines a calendar date/time into a UTC nanosecond timestamp, assuming
/// the given UTC offset.  No timezone database is consulted.
#[allow(clippy::too_many_arguments)]
pub(crate) const fn ts_from_fields(
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    nanosecond: u32,
    offset_secs: i32,
) -> i64 {
    let days = days_from_civil(year, month, day);
    let time_nanos = hour as i64 * 3_600 * NANOS_PER_SEC
        + minute as i64 * 60 * NANOS_PER_SEC
        + second as i64 * NANOS_PER_SEC
        + nanosecond as i64;
    days * NANOS_PER_DAY + time_nanos - offset_secs as i64 * NANOS_PER_SEC
}

/// Add `months` calendar months to `(year, month)`, returning the adjusted
/// pair (month stays in [1, 12]).
pub(crate) const fn add_months_ym(year: i32, month: u8, months: i32) -> (i32, u8) {
    let total_months = (year as i64) * 12 + (month as i64 - 1) + months as i64;
    let new_year = total_months.div_euclid(12) as i32;
    let new_month = (total_months.rem_euclid(12) + 1) as u8;
    (new_year, new_month)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_epoch() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(days_from_civil(1970, 1, 1), 0);
    }

    #[test]
    fn round_trip_various() {
        let cases: &[(i32, u8, u8)] = &[
            (2024, 3, 15),
            (2000, 1, 1),
            (1999, 12, 31),
            (2016, 2, 29), // leap day
            (1969, 12, 31),
            (1, 1, 1),
        ];
        for &(y, m, d) in cases {
            let days = days_from_civil(y, m, d);
            assert_eq!(civil_from_days(days), (y, m, d), "{y}-{m:02}-{d:02}");
        }
    }

    #[test]
    fn leap_year_checks() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2023));
    }

    #[test]
    fn days_in_month_checks() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2024, 1), 31);
        assert_eq!(days_in_month(2024, 4), 30);
    }

    #[test]
    fn weekday_epoch() {
        // 1970-01-01 = Thursday = ISO 4
        assert_eq!(weekday_from_days(0), 4);
    }

    #[test]
    fn add_months_wrap() {
        assert_eq!(add_months_ym(2023, 11, 3), (2024, 2));
        assert_eq!(add_months_ym(2024, 1, -1), (2023, 12));
    }
}
