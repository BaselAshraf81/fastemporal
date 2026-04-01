//! A date and time without a timezone, mirroring the Temporal `PlainDateTime`
//! type.
//!
//! # Examples
//! ```
//! use fastemporal::PlainDateTime;
//!
//! let dt = PlainDateTime::new(2025, 6, 7, 14, 32, 0, 0).unwrap();
//! assert_eq!(dt.year(), 2025);
//! assert_eq!(dt.hour(), 14);
//! ```
#![allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlainDateTime {
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    nanosecond: u32,
}

impl PlainDateTime {
    /// Construct a `PlainDateTime` from its components.
    ///
    /// Returns `None` if any field is out of range.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::PlainDateTime;
    ///
    /// assert!(PlainDateTime::new(2025, 6, 7, 14, 32, 0, 0).is_some());
    /// assert!(PlainDateTime::new(2025, 13, 1, 0, 0, 0, 0).is_none());
    /// ```
    pub fn new(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
    ) -> Option<Self> {
        use crate::calendar::days_in_month;
        if !(1..=12).contains(&month) { return None; }
        if day < 1 || day > days_in_month(year, month) { return None; }
        if hour > 23 || minute > 59 || second > 59 || nanosecond >= 1_000_000_000 {
            return None;
        }
        Some(Self { year, month, day, hour, minute, second, nanosecond })
    }

    /// Parse from an ISO 8601 datetime string.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::PlainDateTime;
    ///
    /// let dt = PlainDateTime::from_iso("2025-06-07T14:32:00").unwrap();
    /// assert_eq!(dt.year(), 2025);
    /// ```
    pub fn from_iso(s: &str) -> crate::error::Result<Self> {
        let f = crate::parsing::parse_iso(s)?;
        PlainDateTime::new(f.year, f.month, f.day, f.hour, f.minute, f.second, f.nanosecond)
            .ok_or_else(|| crate::error::Error::Parse("invalid datetime".into()))
    }

    pub const fn year(&self) -> i32       { self.year }
    pub const fn month(&self) -> u8       { self.month }
    pub const fn day(&self) -> u8         { self.day }
    pub const fn hour(&self) -> u8        { self.hour }
    pub const fn minute(&self) -> u8      { self.minute }
    pub const fn second(&self) -> u8      { self.second }
    pub const fn nanosecond(&self) -> u32 { self.nanosecond }
    pub fn millisecond(&self) -> u32      { self.nanosecond / 1_000_000 }

    /// The date component.
    pub fn date(&self) -> crate::PlainDate {
        crate::PlainDate::new(self.year, self.month, self.day).unwrap()
    }

    /// The time component.
    pub fn time(&self) -> crate::PlainTime {
        crate::PlainTime::new(self.hour, self.minute, self.second, self.nanosecond).unwrap()
    }

    /// Add `n` days (calendar addition).
    pub fn add_days(self, n: i32) -> Self {
        use crate::calendar::{days_from_civil, civil_from_days};
        let days = days_from_civil(self.year, self.month, self.day) + n as i64;
        let (y, m, d) = civil_from_days(days);
        Self { year: y, month: m, day: d, ..self }
    }

    /// ISO 8601 string.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::PlainDateTime;
    ///
    /// let dt = PlainDateTime::new(2025, 6, 7, 14, 32, 0, 0).unwrap();
    /// assert_eq!(dt.to_iso(), "2025-06-07T14:32:00");
    /// ```
    pub fn to_iso(&self) -> String {
        if self.nanosecond == 0 {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
                self.year, self.month, self.day,
                self.hour, self.minute, self.second
            )
        } else {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09}",
                self.year, self.month, self.day,
                self.hour, self.minute, self.second, self.nanosecond
            )
        }
    }
}

impl core::fmt::Display for PlainDateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.to_iso())
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let dt = PlainDateTime::from_iso("2025-06-07T14:32:05.123456789").unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.nanosecond(), 123_456_789);
        assert!(dt.to_iso().starts_with("2025-06-07T14:32:05"));
    }

    #[test]
    fn add_days_crosses_year() {
        let dt = PlainDateTime::new(2025, 12, 31, 12, 0, 0, 0).unwrap().add_days(1);
        assert_eq!((dt.year(), dt.month(), dt.day()), (2026, 1, 1));
    }
}
