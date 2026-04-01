//! A wall-clock time without a date or timezone, mirroring the Temporal
//! `PlainTime` type.
//!
//! # Examples
//! ```
//! use fastemporal::PlainTime;
//!
//! let t = PlainTime::new(14, 32, 5, 0).unwrap();
//! assert_eq!(t.hour(), 14);
//! assert_eq!(t.minute(), 32);
//! assert_eq!(t.second(), 5);
//! ```
#![allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlainTime {
    hour: u8,
    minute: u8,
    second: u8,
    nanosecond: u32,
}

impl PlainTime {
    /// Construct a `PlainTime` from its components.  `nanosecond` must be in
    /// `0..1_000_000_000`.
    ///
    /// Returns `None` if any field is out of range.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::PlainTime;
    ///
    /// assert!(PlainTime::new(23, 59, 59, 999_999_999).is_some());
    /// assert!(PlainTime::new(24, 0, 0, 0).is_none());
    /// ```
    pub fn new(hour: u8, minute: u8, second: u8, nanosecond: u32) -> Option<Self> {
        if hour > 23 || minute > 59 || second > 59 || nanosecond >= 1_000_000_000 {
            return None;
        }
        Some(Self { hour, minute, second, nanosecond })
    }

    /// Parse from an ISO 8601 time string (`HH:MM`, `HH:MM:SS`, or
    /// `HH:MM:SS.sss…`).
    ///
    /// # Examples
    /// ```
    /// use fastemporal::PlainTime;
    ///
    /// let t = PlainTime::from_iso("14:32:05").unwrap();
    /// assert_eq!(t.hour(), 14);
    /// ```
    pub fn from_iso(s: &str) -> crate::error::Result<Self> {
        // Prepend a dummy date to reuse the ISO parser
        let full = format!("2000-01-01T{s}");
        let f = crate::parsing::parse_iso(&full)?;
        PlainTime::new(f.hour, f.minute, f.second, f.nanosecond)
            .ok_or_else(|| crate::error::Error::Parse("invalid time".into()))
    }

    /// The midnight value `00:00:00.000000000`.
    pub const MIDNIGHT: Self = Self { hour: 0, minute: 0, second: 0, nanosecond: 0 };

    pub const fn hour(&self) -> u8       { self.hour }
    pub const fn minute(&self) -> u8     { self.minute }
    pub const fn second(&self) -> u8     { self.second }
    pub const fn nanosecond(&self) -> u32 { self.nanosecond }
    pub fn millisecond(&self) -> u32     { self.nanosecond / 1_000_000 }
    pub fn microsecond(&self) -> u32     { (self.nanosecond / 1_000) % 1_000 }

    /// Total nanoseconds since midnight.
    pub fn total_nanoseconds(&self) -> u64 {
        self.hour as u64 * 3_600_000_000_000
            + self.minute as u64 * 60_000_000_000
            + self.second as u64 * 1_000_000_000
            + self.nanosecond as u64
    }

    /// ISO 8601 string (`HH:MM:SS` or `HH:MM:SS.sssssssss`).
    ///
    /// # Examples
    /// ```
    /// use fastemporal::PlainTime;
    ///
    /// assert_eq!(
    ///     PlainTime::new(14, 32, 5, 0).unwrap().to_iso(),
    ///     "14:32:05"
    /// );
    /// assert_eq!(
    ///     PlainTime::new(14, 32, 5, 123_000_000).unwrap().to_iso(),
    ///     "14:32:05.123000000"
    /// );
    /// ```
    pub fn to_iso(&self) -> String {
        if self.nanosecond == 0 {
            format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
        } else {
            format!("{:02}:{:02}:{:02}.{:09}", self.hour, self.minute, self.second, self.nanosecond)
        }
    }
}

impl core::fmt::Display for PlainTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.to_iso())
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_valid() {
        assert!(PlainTime::new(0, 0, 0, 0).is_some());
        assert!(PlainTime::new(23, 59, 59, 999_999_999).is_some());
    }

    #[test]
    fn new_invalid() {
        assert!(PlainTime::new(24, 0, 0, 0).is_none());
        assert!(PlainTime::new(0, 60, 0, 0).is_none());
        assert!(PlainTime::new(0, 0, 60, 0).is_none());
        assert!(PlainTime::new(0, 0, 0, 1_000_000_000).is_none());
    }

    #[test]
    fn to_iso_no_frac() {
        assert_eq!(
            PlainTime::new(14, 32, 5, 0).unwrap().to_iso(),
            "14:32:05"
        );
    }

    #[test]
    fn to_iso_with_millis() {
        assert_eq!(
            PlainTime::new(14, 32, 5, 123_000_000).unwrap().to_iso(),
            "14:32:05.123000000"
        );
    }

    #[test]
    fn from_iso_round_trip() {
        let t = PlainTime::from_iso("14:32:05").unwrap();
        assert_eq!(t.hour(), 14);
        assert_eq!(t.minute(), 32);
        assert_eq!(t.second(), 5);
    }
}
