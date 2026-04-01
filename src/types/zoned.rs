//! A date-time anchored to a specific IANA timezone, the primary workhorse of
//! `fastemporal`.  Mirrors Luxon's `DateTime` and the Temporal `ZonedDateTime`.
//!
//! Internally stored as a Unix nanosecond timestamp plus a cached UTC offset
//! and a stack-allocated IANA timezone name (see [`TzName`]).  All
//! accessor, arithmetic, and comparison operations are zero-allocation.
//! Only [`to_iso`] and [`format`] allocate (they produce an owned `String`).
//!
//! # Examples
//! ```no_run
//! use fastemporal::{ZonedDateTime, Duration};
//!
//! let dt = ZonedDateTime::now()
//!     .plus(Duration::days(7))
//!     .in_timezone("America/New_York").unwrap();
//!
//! println!("{}", dt.to_iso());
//! ```
//!
//! [`TzName`]: crate::tz::TzName
//! [`to_iso`]: ZonedDateTime::to_iso
//! [`format`]: ZonedDateTime::format
#![allow(missing_docs)]
use crate::calendar::{
    add_months_ym, civil_from_days, days_from_civil, days_in_month, local_fields,
    ts_from_fields, weekday_from_days, NANOS_PER_DAY, NANOS_PER_SEC,
};
use crate::error::{Error, Result};
use crate::format::strftime::{format_dt, FormatCtx};
use crate::tz::{resolve_offset, local_to_utc, TzName};
use crate::types::duration::Duration;

/// String-based unit accepted by [`ZonedDateTime::start_of`],
/// [`ZonedDateTime::end_of`], and [`ZonedDateTime::diff`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Nanosecond,
    Microsecond,
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

impl Unit {
    /// Parse a unit name (plural or singular).
    ///
    /// # Examples
    /// ```
    /// use fastemporal::Unit;
    /// assert_eq!(Unit::parse("days"), Some(Unit::Day));
    /// assert_eq!(Unit::parse("year"), Some(Unit::Year));
    /// ```
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "year" | "years" => Some(Unit::Year),
            "quarter" | "quarters" => Some(Unit::Quarter),
            "month" | "months" => Some(Unit::Month),
            "week" | "weeks" => Some(Unit::Week),
            "day" | "days" => Some(Unit::Day),
            "hour" | "hours" => Some(Unit::Hour),
            "minute" | "minutes" => Some(Unit::Minute),
            "second" | "seconds" => Some(Unit::Second),
            "millisecond" | "milliseconds" => Some(Unit::Millisecond),
            "microsecond" | "microseconds" => Some(Unit::Microsecond),
            "nanosecond" | "nanoseconds" => Some(Unit::Nanosecond),
            _ => None,
        }
    }
}

pub use Unit as TimeUnit;

// ─── ZonedDateTime ────────────────────────────────────────────────────────────

/// A date-time with a timezone.  `Copy`-able and stack-allocated (80 bytes).
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ZonedDateTime {
    /// Unix timestamp in nanoseconds (signed; negative = before 1970).
    ts_nanos: i64,
    /// UTC offset in seconds (cached from the tz database; updated on each
    /// arithmetic operation that can change the wall-clock date).
    offset_secs: i32,
    /// Stack-allocated IANA timezone name.
    tz: TzName,
}

impl ZonedDateTime {
    // ─── Constructors ─────────────────────────────────────────────────────────

    /// Returns the current time in UTC.
    ///
    /// # Examples
    /// ```no_run
    /// use fastemporal::ZonedDateTime;
    /// let now = ZonedDateTime::now();
    /// assert_eq!(now.timezone(), "UTC");
    /// ```
    pub fn now() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;
        Self {
            ts_nanos: nanos,
            offset_secs: 0,
            tz: TzName::UTC,
        }
    }

    /// Parse from an ISO 8601 string.  When a timezone name is embedded in
    /// brackets (`[America/New_York]`) it takes precedence over the numeric
    /// offset for future arithmetic; both are recorded.
    ///
    /// # Errors
    /// Returns [`Error::Parse`] for malformed input, or
    /// [`Error::InvalidTimezone`] for unrecognised IANA names.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::ZonedDateTime;
    ///
    /// let dt = ZonedDateTime::from_iso("2025-06-07T14:32:00Z").unwrap();
    /// assert_eq!(dt.year(), 2025);
    /// assert_eq!(dt.hour(), 14);
    /// assert_eq!(dt.offset_seconds(), 0);
    ///
    /// let dt2 = ZonedDateTime::from_iso("2025-06-07T14:32:00-04:00").unwrap();
    /// assert_eq!(dt2.offset_seconds(), -4 * 3600);
    /// ```
    pub fn from_iso(s: &str) -> Result<Self> {
        let f = crate::parsing::parse_iso(s)?;

        // Determine timezone name
        let tz = if let Some(ref tz_str) = f.tz_name {
            TzName::new(tz_str.as_str())
                .ok_or_else(|| Error::InvalidTimezone(tz_str.as_str().to_string()))?
        } else {
            TzName::UTC
        };

        // Determine UTC timestamp
        let (ts_nanos, offset_secs) = if let Some(off) = f.offset_secs {
            // We have a numeric offset: reconstruct UTC directly.
            let ts = ts_from_fields(f.year, f.month, f.day, f.hour, f.minute, f.second, f.nanosecond, off);
            // If we also have a tz name, re-resolve the actual offset at that UTC time.
            let actual_off = if tz.is_utc() {
                off
            } else {
                resolve_offset(&tz, ts / NANOS_PER_SEC)
                    .map(|(o, _)| o)
                    .unwrap_or(off)
            };
            (ts, actual_off)
        } else {
            // Floating datetime — treat as local wall-clock in the given tz (or UTC).
            local_to_utc(&tz, f.year, f.month, f.day, f.hour, f.minute, f.second, f.nanosecond)?
        };

        Ok(Self { ts_nanos, offset_secs, tz })
    }

    /// Build from a raw Unix nanosecond timestamp, resolving the offset for
    /// the given timezone name.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::ZonedDateTime;
    ///
    /// // 2025-01-01T00:00:00Z
    /// let dt = ZonedDateTime::from_unix_nanos(1_735_689_600_000_000_000, "UTC").unwrap();
    /// assert_eq!(dt.year(), 2025);
    /// ```
    pub fn from_unix_nanos(ts_nanos: i64, tz_name: &str) -> Result<Self> {
        let tz = TzName::new(tz_name)
            .ok_or_else(|| Error::InvalidTimezone(tz_name.to_string()))?;
        let (offset_secs, _) = resolve_offset(&tz, ts_nanos / NANOS_PER_SEC)?;
        Ok(Self { ts_nanos, offset_secs, tz })
    }

    // ─── Field accessors ─────────────────────────────────────────────────────

    /// Returns the year in the local timezone.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::ZonedDateTime;
    /// let dt = ZonedDateTime::from_iso("2025-06-07T14:32:00Z").unwrap();
    /// assert_eq!(dt.year(), 2025);
    /// ```
    pub fn year(&self) -> i32   { local_fields(self.ts_nanos, self.offset_secs).0 }

    /// Returns the month (1–12) in the local timezone.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::ZonedDateTime;
    /// let dt = ZonedDateTime::from_iso("2025-06-07T14:32:00Z").unwrap();
    /// assert_eq!(dt.month(), 6);
    /// ```
    pub fn month(&self) -> u8   { local_fields(self.ts_nanos, self.offset_secs).1 }

    /// Returns the day (1–31) in the local timezone.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::ZonedDateTime;
    /// let dt = ZonedDateTime::from_iso("2025-06-07T14:32:00Z").unwrap();
    /// assert_eq!(dt.day(), 7);
    /// ```
    pub fn day(&self) -> u8     { local_fields(self.ts_nanos, self.offset_secs).2 }

    /// Returns the hour (0–23) in the local timezone.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::ZonedDateTime;
    /// let dt = ZonedDateTime::from_iso("2025-06-07T14:32:00Z").unwrap();
    /// assert_eq!(dt.hour(), 14);
    /// ```
    pub fn hour(&self) -> u8    { local_fields(self.ts_nanos, self.offset_secs).3 }

    /// Returns the minute (0–59).
    ///
    /// # Examples
    /// ```
    /// use fastemporal::ZonedDateTime;
    /// let dt = ZonedDateTime::from_iso("2025-06-07T14:32:00Z").unwrap();
    /// assert_eq!(dt.minute(), 32);
    /// ```
    pub fn minute(&self) -> u8  { local_fields(self.ts_nanos, self.offset_secs).4 }

    /// Returns the second (0–59).
    ///
    /// # Examples
    /// ```
    /// use fastemporal::ZonedDateTime;
    /// let dt = ZonedDateTime::from_iso("2025-06-07T14:32:00Z").unwrap();
    /// assert_eq!(dt.second(), 0);
    /// ```
    pub fn second(&self) -> u8  { local_fields(self.ts_nanos, self.offset_secs).5 }

    /// Returns the sub-second component in nanoseconds (0–999_999_999).
    ///
    /// # Examples
    /// ```
    /// use fastemporal::ZonedDateTime;
    /// let dt = ZonedDateTime::from_iso("2025-06-07T14:32:00.123Z").unwrap();
    /// assert_eq!(dt.nanosecond(), 123_000_000);
    /// ```
    pub fn nanosecond(&self) -> u32 { local_fields(self.ts_nanos, self.offset_secs).6 }

    /// Returns the millisecond component (0–999).
    pub fn millisecond(&self) -> u32 { self.nanosecond() / 1_000_000 }

    /// Returns the ISO weekday: 1 = Monday … 7 = Sunday.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::ZonedDateTime;
    /// // 2025-01-01 is a Wednesday (ISO 3)
    /// let dt = ZonedDateTime::from_iso("2025-01-01T00:00:00Z").unwrap();
    /// assert_eq!(dt.weekday(), 3);
    /// ```
    pub fn weekday(&self) -> u8 {
        let local = self.ts_nanos + self.offset_secs as i64 * NANOS_PER_SEC;
        let days = local.div_euclid(NANOS_PER_DAY);
        weekday_from_days(days)
    }

    /// Returns the UTC offset in seconds.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::ZonedDateTime;
    /// let dt = ZonedDateTime::from_iso("2025-01-01T00:00:00-05:00").unwrap();
    /// assert_eq!(dt.offset_seconds(), -5 * 3600);
    /// ```
    pub const fn offset_seconds(&self) -> i32 { self.offset_secs }

    /// Returns the IANA timezone name.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::ZonedDateTime;
    /// let dt = ZonedDateTime::from_iso("2025-01-01T00:00:00Z").unwrap();
    /// assert_eq!(dt.timezone(), "UTC");
    /// ```
    pub fn timezone(&self) -> &str { self.tz.as_str() }

    /// Raw Unix timestamp in nanoseconds.
    pub const fn unix_nanos(&self) -> i64 { self.ts_nanos }

    /// Unix timestamp in whole seconds.
    pub const fn unix_seconds(&self) -> i64 { self.ts_nanos / NANOS_PER_SEC }

    /// Unix timestamp in whole milliseconds.
    pub const fn unix_millis(&self) -> i64 { self.ts_nanos / 1_000_000 }

    // ─── Arithmetic ───────────────────────────────────────────────────────────

    /// Add a [`Duration`] and return the resulting [`ZonedDateTime`].
    ///
    /// - **Calendar units** (years, months, weeks, days) are applied in
    ///   wall-clock space, preserving the time-of-day across DST transitions
    ///   (Luxon semantics).
    /// - **Clock units** (hours, minutes, seconds, …) are applied to the Unix
    ///   timestamp directly, so 24 hours may land on a different wall-clock hour
    ///   after a DST transition.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::{ZonedDateTime, Duration};
    ///
    /// let dt = ZonedDateTime::from_iso("2025-01-01T12:00:00Z").unwrap();
    /// let later = dt.plus(Duration::days(7));
    /// assert_eq!(later.day(), 8);
    /// assert_eq!(later.hour(), 12);
    /// ```
    pub fn plus(self, dur: Duration) -> Self {
        // 1. Apply clock units to timestamp directly.
        let new_ts = self.ts_nanos + dur.clock_nanos();

        if !dur.has_calendar_units() {
            // Pure clock addition: re-resolve offset and return.
            let (off, _) = resolve_offset(&self.tz, new_ts / NANOS_PER_SEC).unwrap_or((self.offset_secs, false));
            return Self { ts_nanos: new_ts, offset_secs: off, tz: self.tz };
        }

        // 2. Calendar addition: operate in wall-clock space.
        let (y, m, d, h, min, s, ns) = local_fields(new_ts, self.offset_secs);

        // Add years
        let ny = y + dur.num_years();
        // Add months
        let (ny, nm) = add_months_ym(ny, m, dur.num_months());
        // Clamp day (e.g., Jan 31 + 1 month = Feb 28)
        let nd = d.min(days_in_month(ny, nm));
        // Add days + weeks
        let epoch_day = days_from_civil(ny, nm, nd) + dur.total_days() as i64;
        let (fy, fm, fd) = civil_from_days(epoch_day);

        // 3. Reconstruct UTC timestamp from new local datetime.
        match local_to_utc(&self.tz, fy, fm, fd, h, min, s, ns) {
            Ok((ts, off)) => Self { ts_nanos: ts, offset_secs: off, tz: self.tz },
            Err(_) => {
                // Fallback: naive offset arithmetic (should be unreachable for valid tz).
                let ts = ts_from_fields(fy, fm, fd, h, min, s, ns, self.offset_secs);
                Self { ts_nanos: ts, offset_secs: self.offset_secs, tz: self.tz }
            }
        }
    }

    /// Subtract a [`Duration`] and return the resulting [`ZonedDateTime`].
    ///
    /// Equivalent to `self.plus(-dur)`.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::{ZonedDateTime, Duration};
    ///
    /// let dt = ZonedDateTime::from_iso("2025-01-08T12:00:00Z").unwrap();
    /// let earlier = dt.minus(Duration::days(7));
    /// assert_eq!(earlier.day(), 1);
    /// ```
    pub fn minus(self, dur: Duration) -> Self {
        self.plus(dur.negate())
    }

    // ─── Timezone conversion ──────────────────────────────────────────────────

    /// Convert to a different IANA timezone, preserving the instant.
    ///
    /// # Errors
    /// Returns [`Error::InvalidTimezone`] if `tz_name` is not recognised.
    ///
    /// # Examples
    /// ```
    /// # #[cfg(feature = "tz-embedded")]
    /// # {
    /// use fastemporal::ZonedDateTime;
    ///
    /// let utc = ZonedDateTime::from_iso("2025-01-01T05:00:00Z").unwrap();
    /// let ny  = utc.in_timezone("America/New_York").unwrap();
    /// // UTC-5 in January
    /// assert_eq!(ny.hour(), 0);
    /// assert_eq!(ny.timezone(), "America/New_York");
    /// # }
    /// ```
    pub fn in_timezone(self, tz_name: &str) -> Result<Self> {
        let tz = TzName::new(tz_name)
            .ok_or_else(|| Error::InvalidTimezone(tz_name.to_string()))?;
        let (offset_secs, _) = resolve_offset(&tz, self.ts_nanos / NANOS_PER_SEC)?;
        Ok(Self { ts_nanos: self.ts_nanos, offset_secs, tz })
    }

    // ─── start_of / end_of ───────────────────────────────────────────────────

    /// Move to the start of the given calendar unit.
    ///
    /// # Errors
    /// Returns [`Error::InvalidUnit`] for unknown unit strings.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::ZonedDateTime;
    ///
    /// let dt = ZonedDateTime::from_iso("2025-06-07T14:32:05Z").unwrap();
    /// let sob = dt.start_of("day").unwrap();
    /// assert_eq!((sob.hour(), sob.minute(), sob.second()), (0, 0, 0));
    ///
    /// let som = dt.start_of("month").unwrap();
    /// assert_eq!(som.day(), 1);
    /// ```
    pub fn start_of(self, unit: &str) -> Result<Self> {
        let u = Unit::parse(unit).ok_or_else(|| Error::InvalidUnit(unit.to_string()))?;
        let (y, m, d, h, min, s, _ns) = local_fields(self.ts_nanos, self.offset_secs);
        let (ny, nm, nd, nh, nmin, ns_val): (i32, u8, u8, u8, u8, u8) = match u {
            Unit::Year    => (y, 1, 1, 0, 0, 0),
            Unit::Quarter => {
                let q_start_month = ((m - 1) / 3) * 3 + 1;
                (y, q_start_month, 1, 0, 0, 0)
            }
            Unit::Month   => (y, m, 1, 0, 0, 0),
            Unit::Week    => {
                // ISO week: Monday is day 1
                let wd = self.weekday(); // 1=Mon … 7=Sun
                let day_offset = wd as i32 - 1;
                let epoch = days_from_civil(y, m, d) - day_offset as i64;
                let (wy, wm, wd2) = civil_from_days(epoch);
                (wy, wm, wd2, 0, 0, 0)
            }
            Unit::Day     => (y, m, d, 0, 0, 0),
            Unit::Hour    => (y, m, d, h, 0, 0),
            Unit::Minute  => (y, m, d, h, min, 0),
            Unit::Second  => (y, m, d, h, min, s),
            // Sub-second units: truncate nanoseconds / microseconds / millis
            Unit::Millisecond | Unit::Microsecond | Unit::Nanosecond => {
                let divisor: i64 = match u {
                    Unit::Millisecond => 1_000_000,
                    Unit::Microsecond => 1_000,
                    Unit::Nanosecond  => 1,
                    _ => unreachable!(),
                };
                let new_ts = (self.ts_nanos / divisor) * divisor;
                let (off, _) = resolve_offset(&self.tz, new_ts / NANOS_PER_SEC)
                    .unwrap_or((self.offset_secs, false));
                return Ok(Self { ts_nanos: new_ts, offset_secs: off, tz: self.tz });
            }
        };
        let (ts, off) = local_to_utc(&self.tz, ny, nm, nd, nh, nmin, ns_val, 0)?;
        Ok(Self { ts_nanos: ts, offset_secs: off, tz: self.tz })
    }

    /// Move to the end of the given calendar unit (last nanosecond).
    ///
    /// # Errors
    /// Returns [`Error::InvalidUnit`] for unknown unit strings.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::ZonedDateTime;
    ///
    /// let dt = ZonedDateTime::from_iso("2025-06-07T14:32:05Z").unwrap();
    /// let eod = dt.end_of("day").unwrap();
    /// assert_eq!((eod.hour(), eod.minute(), eod.second()), (23, 59, 59));
    /// assert_eq!(eod.nanosecond(), 999_999_999);
    /// ```
    pub fn end_of(self, unit: &str) -> Result<Self> {
        let start_of_next = self.start_of(unit)?.advance_one_unit(unit)?;
        // end = start_of_next - 1 nanosecond
        let ts = start_of_next.ts_nanos - 1;
        let (off, _) = resolve_offset(&self.tz, ts / NANOS_PER_SEC)
            .unwrap_or((self.offset_secs, false));
        Ok(Self { ts_nanos: ts, offset_secs: off, tz: self.tz })
    }

    /// Advance by exactly one `unit` (used internally by `end_of`).
    fn advance_one_unit(self, unit: &str) -> Result<Self> {
        let u = Unit::parse(unit).ok_or_else(|| Error::InvalidUnit(unit.to_string()))?;
        let dur = match u {
            Unit::Year        => Duration::builder().years(1).build(),
            Unit::Quarter     => Duration::builder().months(3).build(),
            Unit::Month       => Duration::builder().months(1).build(),
            Unit::Week        => Duration::builder().weeks(1).build(),
            Unit::Day         => Duration::days(1),
            Unit::Hour        => Duration::from_hours(1),
            Unit::Minute      => Duration::from_minutes(1),
            Unit::Second      => Duration::from_seconds(1),
            Unit::Millisecond => Duration::from_millis(1),
            Unit::Microsecond => Duration::from_nanos(1_000),
            Unit::Nanosecond  => Duration::from_nanos(1),
        };
        Ok(self.plus(dur))
    }

    // ─── diff ─────────────────────────────────────────────────────────────────

    /// Compute the difference between `self` and `other` in the given unit.
    ///
    /// Returns a [`Duration`] with the relevant field set.  The sign is
    /// positive when `self` is after `other`.
    ///
    /// # Errors
    /// Returns [`Error::InvalidUnit`] for unknown unit strings.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::{ZonedDateTime, Duration};
    ///
    /// let a = ZonedDateTime::from_iso("2025-01-10T00:00:00Z").unwrap();
    /// let b = ZonedDateTime::from_iso("2025-01-01T00:00:00Z").unwrap();
    /// let diff = a.diff(b, "days").unwrap();
    /// assert_eq!(diff.num_days(), 9);
    /// ```
    pub fn diff(self, other: ZonedDateTime, unit: &str) -> Result<Duration> {
        let u = Unit::parse(unit).ok_or_else(|| Error::InvalidUnit(unit.to_string()))?;
        let delta_nanos = self.ts_nanos - other.ts_nanos;

        let dur = match u {
            Unit::Nanosecond  => Duration::from_nanos(delta_nanos.clamp(i32::MIN as i64, i32::MAX as i64) as i32),
            Unit::Microsecond => Duration::from_nanos((delta_nanos / 1_000).clamp(i32::MIN as i64, i32::MAX as i64) as i32),
            Unit::Millisecond => Duration::from_millis((delta_nanos / 1_000_000).clamp(i32::MIN as i64, i32::MAX as i64) as i32),
            Unit::Second      => Duration::from_seconds((delta_nanos / NANOS_PER_SEC).clamp(i32::MIN as i64, i32::MAX as i64) as i32),
            Unit::Minute      => Duration::from_minutes((delta_nanos / (60 * NANOS_PER_SEC)).clamp(i32::MIN as i64, i32::MAX as i64) as i32),
            Unit::Hour        => Duration::from_hours((delta_nanos / (3600 * NANOS_PER_SEC)).clamp(i32::MIN as i64, i32::MAX as i64) as i32),
            Unit::Day => {
                let a_day = (self.ts_nanos + self.offset_secs as i64 * NANOS_PER_SEC).div_euclid(NANOS_PER_DAY);
                let b_day = (other.ts_nanos + other.offset_secs as i64 * NANOS_PER_SEC).div_euclid(NANOS_PER_DAY);
                Duration::days((a_day - b_day) as i32)
            }
            Unit::Week => {
                let a_day = (self.ts_nanos + self.offset_secs as i64 * NANOS_PER_SEC).div_euclid(NANOS_PER_DAY);
                let b_day = (other.ts_nanos + other.offset_secs as i64 * NANOS_PER_SEC).div_euclid(NANOS_PER_DAY);
                Duration::from_weeks(((a_day - b_day) / 7) as i32)
            }
            Unit::Month | Unit::Quarter => {
                let (ay, am, _ad, _ah, _amin, _as_, _) = local_fields(self.ts_nanos, self.offset_secs);
                let (by, bm, _bd, _bh, _bmin, _bs, _) = local_fields(other.ts_nanos, other.offset_secs);
                let months = (ay as i64 * 12 + am as i64) - (by as i64 * 12 + bm as i64);
                if u == Unit::Quarter {
                    Duration::builder().months((months / 3) as i32).build()
                } else {
                    Duration::builder().months(months as i32).build()
                }
            }
            Unit::Year => {
                let (ay, _, _, _, _, _, _) = local_fields(self.ts_nanos, self.offset_secs);
                let (by, _, _, _, _, _, _) = local_fields(other.ts_nanos, other.offset_secs);
                Duration::builder().years(ay - by).build()
            }
        };
        Ok(dur)
    }

    // ─── Formatting ───────────────────────────────────────────────────────────

    /// Format the datetime using a strftime-style or Luxon-style format string.
    ///
    /// See [`format_dt`] for a full token reference.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::ZonedDateTime;
    ///
    /// let dt = ZonedDateTime::from_iso("2025-06-07T14:32:05Z").unwrap();
    /// assert_eq!(dt.format("%Y-%m-%d"), "2025-06-07");
    /// assert_eq!(dt.format("HH:mm:ss"), "14:32:05");
    /// ```
    pub fn format(&self, fmt: &str) -> String {
        let (y, m, d, h, min, s, ns) = local_fields(self.ts_nanos, self.offset_secs);
        let local = self.ts_nanos + self.offset_secs as i64 * NANOS_PER_SEC;
        let days = local.div_euclid(NANOS_PER_DAY);
        let ctx = FormatCtx {
            year: y, month: m, day: d,
            hour: h, minute: min, second: s, nanosecond: ns,
            weekday_iso: weekday_from_days(days),
            offset_secs: self.offset_secs,
            tz_abbr: "",
        };
        format_dt(fmt, &ctx)
    }

    /// Return an ISO 8601 string with timezone offset and optional IANA name.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::ZonedDateTime;
    ///
    /// let dt = ZonedDateTime::from_iso("2025-06-07T14:32:00Z").unwrap();
    /// assert_eq!(dt.to_iso(), "2025-06-07T14:32:00.000+00:00");
    /// ```
    pub fn to_iso(&self) -> String {
        // Zero-intermediate-allocation ISO 8601 builder.
        // Format: YYYY-MM-DDTHH:MM:SS.mmm±HH:MM[TzName]
        // Max length: 4+1+2+1+2+1+2+1+2+1+2+1+3+1+2+1+2 + 2+48 = ~85 chars
        let (y, m, d, h, min, s, ns) = local_fields(self.ts_nanos, self.offset_secs);
        let millis = ns / 1_000_000;
        let off = self.offset_secs;
        let (sign, abs) = if off < 0 { ('-', (-off) as u32) } else { ('+', off as u32) };
        let oh = (abs / 3600) as u8;
        let om = ((abs % 3600) / 60) as u8;

        // Write into a fixed stack buffer; only one String allocation at the end.
        let mut buf = [0u8; 88];
        let mut pos = 0usize;

        #[inline(always)]
        fn w2(buf: &mut [u8], pos: &mut usize, v: u8) {
            buf[*pos]     = b'0' + v / 10;
            buf[*pos + 1] = b'0' + v % 10;
            *pos += 2;
        }
        #[inline(always)]
        fn w4(buf: &mut [u8], pos: &mut usize, v: i32) {
            let v = v.unsigned_abs();
            buf[*pos]     = b'0' + (v / 1000 % 10) as u8;
            buf[*pos + 1] = b'0' + (v / 100  % 10) as u8;
            buf[*pos + 2] = b'0' + (v / 10   % 10) as u8;
            buf[*pos + 3] = b'0' + (v         % 10) as u8;
            *pos += 4;
        }
        #[inline(always)]
        fn w3(buf: &mut [u8], pos: &mut usize, v: u32) {
            buf[*pos]     = b'0' + (v / 100 % 10) as u8;
            buf[*pos + 1] = b'0' + (v / 10  % 10) as u8;
            buf[*pos + 2] = b'0' + (v        % 10) as u8;
            *pos += 3;
        }

        if y < 0 { buf[pos] = b'-'; pos += 1; }
        w4(&mut buf, &mut pos, y.abs());
        buf[pos] = b'-'; pos += 1;
        w2(&mut buf, &mut pos, m);
        buf[pos] = b'-'; pos += 1;
        w2(&mut buf, &mut pos, d);
        buf[pos] = b'T'; pos += 1;
        w2(&mut buf, &mut pos, h);
        buf[pos] = b':'; pos += 1;
        w2(&mut buf, &mut pos, min);
        buf[pos] = b':'; pos += 1;
        w2(&mut buf, &mut pos, s);
        buf[pos] = b'.'; pos += 1;
        w3(&mut buf, &mut pos, millis);
        buf[pos] = sign as u8; pos += 1;
        w2(&mut buf, &mut pos, oh);
        buf[pos] = b':'; pos += 1;
        w2(&mut buf, &mut pos, om);

        // Append [TzName] if not UTC
        if !self.tz.is_utc() {
            buf[pos] = b'['; pos += 1;
            let tz_bytes = self.tz.as_str().as_bytes();
            buf[pos..pos + tz_bytes.len()].copy_from_slice(tz_bytes);
            pos += tz_bytes.len();
            buf[pos] = b']'; pos += 1;
        }

        // SAFETY: we only wrote ASCII digits, punctuation, and valid UTF-8 tz name bytes.
        unsafe { String::from_utf8_unchecked(buf[..pos].to_vec()) }
    }

    /// Convert to a [`PlainDate`].
    pub fn to_plain_date(&self) -> crate::PlainDate {
        let (y, m, d, _, _, _, _) = local_fields(self.ts_nanos, self.offset_secs);
        crate::PlainDate::new(y, m, d).unwrap()
    }

    /// Convert to a [`PlainTime`].
    pub fn to_plain_time(&self) -> crate::PlainTime {
        let (_, _, _, h, min, s, ns) = local_fields(self.ts_nanos, self.offset_secs);
        crate::PlainTime::new(h, min, s, ns).unwrap()
    }

    /// Convert to a [`PlainDateTime`].
    pub fn to_plain_datetime(&self) -> crate::PlainDateTime {
        let (y, m, d, h, min, s, ns) = local_fields(self.ts_nanos, self.offset_secs);
        crate::PlainDateTime::new(y, m, d, h, min, s, ns).unwrap()
    }
}

impl core::fmt::Debug for ZonedDateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ZonedDateTime({})", self.to_iso())
    }
}

impl core::fmt::Display for ZonedDateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.to_iso())
    }
}

impl PartialOrd for ZonedDateTime {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ZonedDateTime {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.ts_nanos.cmp(&other.ts_nanos)
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn dt(s: &str) -> ZonedDateTime {
        ZonedDateTime::from_iso(s).expect(s)
    }

    #[test]
    fn parse_utc() {
        let z = dt("2025-06-07T14:32:00Z");
        assert_eq!((z.year(), z.month(), z.day()), (2025, 6, 7));
        assert_eq!((z.hour(), z.minute(), z.second()), (14, 32, 0));
        assert_eq!(z.offset_seconds(), 0);
    }

    #[test]
    fn parse_with_offset() {
        let z = dt("2025-06-07T10:32:00-04:00");
        // UTC is 14:32
        assert_eq!(z.unix_seconds(), dt("2025-06-07T14:32:00Z").unix_seconds());
        assert_eq!(z.offset_seconds(), -4 * 3600);
    }

    #[test]
    fn parse_millis() {
        let z = dt("2025-06-07T14:32:00.123Z");
        assert_eq!(z.millisecond(), 123);
    }

    #[test]
    fn plus_days() {
        let z = dt("2025-01-01T12:00:00Z").plus(Duration::days(7));
        assert_eq!((z.year(), z.month(), z.day()), (2025, 1, 8));
        assert_eq!(z.hour(), 12);
    }

    #[test]
    fn plus_months_clamp() {
        // Jan 31 + 1 month = Feb 28 (2025 is not a leap year)
        let z = dt("2025-01-31T10:00:00Z").plus(Duration::builder().months(1).build());
        assert_eq!((z.month(), z.day()), (2, 28));
    }

    #[test]
    fn plus_months_leap() {
        // Jan 31 + 1 month in 2016 (leap year) = Feb 29
        let z = dt("2016-01-31T10:00:00Z").plus(Duration::builder().months(1).build());
        assert_eq!((z.month(), z.day()), (2, 29));
    }

    #[test]
    fn plus_months_13() {
        // Jan 31 2015 + 13 months = Feb 29 2016
        let z = dt("2015-01-31T10:00:00Z").plus(Duration::builder().months(13).build());
        assert_eq!(z.day(), 29);
        assert_eq!(z.month(), 2);
        assert_eq!(z.year(), 2016);
    }

    #[test]
    fn minus_days() {
        let z = dt("2025-01-08T12:00:00Z").minus(Duration::days(7));
        assert_eq!(z.day(), 1);
    }

    #[test]
    fn diff_days() {
        let a = dt("2025-01-10T00:00:00Z");
        let b = dt("2025-01-01T00:00:00Z");
        assert_eq!(a.diff(b, "days").unwrap().num_days(), 9);
    }

    #[test]
    fn diff_hours() {
        let a = dt("2025-01-01T13:00:00Z");
        let b = dt("2025-01-01T05:00:00Z");
        assert_eq!(a.diff(b, "hours").unwrap().num_hours(), 8);
    }

    #[test]
    fn diff_milliseconds() {
        let a = dt("2017-01-01T00:00:00.012Z");
        let b = dt("2017-01-01T00:00:00.000Z");
        assert_eq!(a.diff(b, "milliseconds").unwrap().num_milliseconds(), 12);
    }

    #[test]
    fn start_of_day() {
        let z = dt("2025-06-07T14:32:05Z").start_of("day").unwrap();
        assert_eq!((z.hour(), z.minute(), z.second(), z.nanosecond()), (0, 0, 0, 0));
        assert_eq!((z.year(), z.month(), z.day()), (2025, 6, 7));
    }

    #[test]
    fn start_of_month() {
        let z = dt("2025-06-15T14:32:00Z").start_of("month").unwrap();
        assert_eq!(z.day(), 1);
        assert_eq!(z.hour(), 0);
    }

    #[test]
    fn start_of_year() {
        let z = dt("2025-06-15T14:32:00Z").start_of("year").unwrap();
        assert_eq!((z.month(), z.day()), (1, 1));
    }

    #[test]
    fn end_of_day() {
        let z = dt("2025-06-07T14:32:05Z").end_of("day").unwrap();
        assert_eq!((z.hour(), z.minute(), z.second()), (23, 59, 59));
        assert_eq!(z.nanosecond(), 999_999_999);
    }

    #[test]
    fn to_iso_format() {
        let z = dt("2025-06-07T14:32:00Z");
        assert!(z.to_iso().starts_with("2025-06-07T14:32:00.000+00:00"));
    }

    #[test]
    fn format_strftime() {
        let z = dt("2025-06-07T14:32:05Z");
        assert_eq!(z.format("%Y-%m-%d"), "2025-06-07");
        assert_eq!(z.format("%H:%M:%S"), "14:32:05");
    }

    #[test]
    fn format_luxon_style() {
        let z = dt("2025-06-07T14:32:05Z");
        assert_eq!(z.format("yyyy-MM-dd"), "2025-06-07");
        assert_eq!(z.format("HH:mm:ss"), "14:32:05");
    }

    #[test]
    fn weekday() {
        // 2025-01-01 = Wednesday = ISO 3
        assert_eq!(dt("2025-01-01T00:00:00Z").weekday(), 3);
        // 1970-01-01 = Thursday = ISO 4
        assert_eq!(dt("1970-01-01T00:00:00Z").weekday(), 4);
    }

    #[test]
    fn plus_year() {
        let z = dt("2010-02-03T04:05:06.007Z").plus(Duration::builder().years(1).build());
        assert_eq!(z.year(), 2011);
    }

    #[test]
    fn plus_quarter() {
        let z = dt("2010-02-03T04:05:06.007Z").plus(Duration::builder().months(3).build());
        assert_eq!(z.month(), 5);
    }

    #[cfg(any(feature = "tz-embedded", feature = "tz-system"))]
    #[test]
    fn in_timezone_new_york() {
        // 2025-01-01T05:00:00Z = 2025-01-01T00:00:00 EST (UTC-5)
        let utc = dt("2025-01-01T05:00:00Z");
        let ny = utc.in_timezone("America/New_York").unwrap();
        assert_eq!(ny.hour(), 0);
        assert_eq!(ny.timezone(), "America/New_York");
    }

    #[cfg(any(feature = "tz-embedded", feature = "tz-system"))]
    #[test]
    fn plus_days_preserves_wall_clock_across_dst() {
        // 2016-03-12T10:00 LA time + 1 day = 2016-03-13T10:00 LA time
        // (spring forward happened on 2016-03-13 at 02:00 → 03:00)
        let z = ZonedDateTime::from_iso("2016-03-12T10:00:00-08:00[America/Los_Angeles]").unwrap();
        let later = z.plus(Duration::days(1));
        assert_eq!(later.day(), 13);
        assert_eq!(later.hour(), 10);
    }

    #[cfg(any(feature = "tz-embedded", feature = "tz-system"))]
    #[test]
    fn plus_hours_24_changes_wall_clock_across_dst() {
        // 2016-03-12T10:00 LA + 24 hours = 2016-03-13T11:00 LA (gained 1h)
        let z = ZonedDateTime::from_iso("2016-03-12T10:00:00-08:00[America/Los_Angeles]").unwrap();
        let later = z.plus(Duration::from_hours(24));
        assert_eq!(later.day(), 13);
        assert_eq!(later.hour(), 11);
    }
}
