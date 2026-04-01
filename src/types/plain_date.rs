/// A calendar date without a time-of-day component or timezone, mirroring the
/// Temporal `PlainDate` type.
///
/// # Examples
/// ```
/// use fastemporal::PlainDate;
///
/// let d = PlainDate::new(2025, 6, 7).unwrap();
/// assert_eq!(d.year(), 2025);
/// assert_eq!(d.month(), 6);
/// assert_eq!(d.day(), 7);
///
/// let tomorrow = d.add_days(1);
/// assert_eq!(tomorrow.day(), 8);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlainDate {
    year: i32,
    month: u8,
    day: u8,
}

impl PlainDate {
    /// Construct a `PlainDate` from year, month (1-based), and day (1-based).
    ///
    /// Returns `None` if the combination is not a valid Gregorian date.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::PlainDate;
    ///
    /// assert!(PlainDate::new(2024, 2, 29).is_some()); // leap day
    /// assert!(PlainDate::new(2023, 2, 29).is_none()); // not a leap year
    /// ```
    pub fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        if !(1..=12).contains(&month) {
            return None;
        }
        let max = crate::calendar::days_in_month(year, month);
        if day < 1 || day > max {
            return None;
        }
        Some(Self { year, month, day })
    }

    /// Parse a date from an ISO 8601 string (`YYYY-MM-DD`).
    ///
    /// # Examples
    /// ```
    /// use fastemporal::PlainDate;
    ///
    /// let d = PlainDate::from_iso("2025-06-07").unwrap();
    /// assert_eq!(d.year(), 2025);
    /// ```
    pub fn from_iso(s: &str) -> crate::error::Result<Self> {
        let f = crate::parsing::parse_iso(s)?;
        PlainDate::new(f.year, f.month, f.day)
            .ok_or_else(|| crate::error::Error::Parse("invalid date".into()))
    }

    /// Returns the year.
    pub const fn year(&self) -> i32  { self.year }
    /// Returns the month (1–12).
    pub const fn month(&self) -> u8  { self.month }
    /// Returns the day (1–31).
    pub const fn day(&self) -> u8    { self.day }

    /// Returns the ISO weekday: 1 = Monday … 7 = Sunday.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::PlainDate;
    ///
    /// // 2025-01-01 is a Wednesday (ISO 3)
    /// assert_eq!(PlainDate::new(2025, 1, 1).unwrap().weekday(), 3);
    /// ```
    pub fn weekday(&self) -> u8 {
        let days = crate::calendar::days_from_civil(self.year, self.month, self.day);
        crate::calendar::weekday_from_days(days)
    }

    /// Returns the day-of-year (1-based).
    ///
    /// # Examples
    /// ```
    /// use fastemporal::PlainDate;
    ///
    /// assert_eq!(PlainDate::new(2025, 1, 1).unwrap().day_of_year(), 1);
    /// assert_eq!(PlainDate::new(2025, 12, 31).unwrap().day_of_year(), 365);
    /// ```
    pub fn day_of_year(&self) -> u16 {
        let start = crate::calendar::days_from_civil(self.year, 1, 1);
        let me    = crate::calendar::days_from_civil(self.year, self.month, self.day);
        (me - start + 1) as u16
    }

    /// Returns whether the year is a leap year.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::PlainDate;
    ///
    /// assert!(PlainDate::new(2024, 1, 1).unwrap().in_leap_year());
    /// assert!(!PlainDate::new(2025, 1, 1).unwrap().in_leap_year());
    /// ```
    pub const fn in_leap_year(&self) -> bool {
        crate::calendar::is_leap_year(self.year)
    }

    /// Returns the number of days in the month of this date.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::PlainDate;
    ///
    /// assert_eq!(PlainDate::new(2025, 2, 1).unwrap().days_in_month(), 28);
    /// assert_eq!(PlainDate::new(2024, 2, 1).unwrap().days_in_month(), 29);
    /// ```
    pub const fn days_in_month(&self) -> u8 {
        crate::calendar::days_in_month(self.year, self.month)
    }

    // ─── Arithmetic ──────────────────────────────────────────────────────────

    /// Add `n` calendar days.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::PlainDate;
    ///
    /// let d = PlainDate::new(2025, 1, 31).unwrap().add_days(1);
    /// assert_eq!((d.year(), d.month(), d.day()), (2025, 2, 1));
    /// ```
    pub fn add_days(self, n: i32) -> Self {
        let days = crate::calendar::days_from_civil(self.year, self.month, self.day) + n as i64;
        let (y, m, d) = crate::calendar::civil_from_days(days);
        Self { year: y, month: m, day: d }
    }

    /// Add `n` calendar months, clamping the day to the last day of the
    /// resulting month if necessary.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::PlainDate;
    ///
    /// // Jan 31 + 1 month = Feb 28 (not a leap year)
    /// let d = PlainDate::new(2025, 1, 31).unwrap().add_months(1);
    /// assert_eq!((d.year(), d.month(), d.day()), (2025, 2, 28));
    /// ```
    pub fn add_months(self, n: i32) -> Self {
        let (ny, nm) = crate::calendar::add_months_ym(self.year, self.month, n);
        let nd = self.day.min(crate::calendar::days_in_month(ny, nm));
        Self { year: ny, month: nm, day: nd }
    }

    /// Add `n` calendar years.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::PlainDate;
    ///
    /// // Feb 29 (leap) + 1 year = Feb 28 (non-leap)
    /// let d = PlainDate::new(2024, 2, 29).unwrap().add_years(1);
    /// assert_eq!((d.year(), d.month(), d.day()), (2025, 2, 28));
    /// ```
    pub fn add_years(self, n: i32) -> Self {
        let ny = self.year + n;
        let nd = self.day.min(crate::calendar::days_in_month(ny, self.month));
        Self { year: ny, month: self.month, day: nd }
    }

    /// Returns the number of days between `self` and `other` (signed; positive
    /// if `self` is after `other`).
    ///
    /// # Examples
    /// ```
    /// use fastemporal::PlainDate;
    ///
    /// let a = PlainDate::new(2025, 1, 10).unwrap();
    /// let b = PlainDate::new(2025, 1, 1).unwrap();
    /// assert_eq!(a.days_until(b), 9);
    /// ```
    pub fn days_until(self, other: PlainDate) -> i64 {
        let a = crate::calendar::days_from_civil(self.year, self.month, self.day);
        let b = crate::calendar::days_from_civil(other.year, other.month, other.day);
        a - b
    }

    /// ISO 8601 string representation.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::PlainDate;
    ///
    /// assert_eq!(PlainDate::new(2025, 6, 7).unwrap().to_iso(), "2025-06-07");
    /// ```
    pub fn to_iso(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl core::fmt::Display for PlainDate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_valid() {
        assert!(PlainDate::new(2025, 1, 31).is_some());
        assert!(PlainDate::new(2024, 2, 29).is_some());
    }

    #[test]
    fn new_invalid() {
        assert!(PlainDate::new(2025, 13, 1).is_none());
        assert!(PlainDate::new(2023, 2, 29).is_none());
        assert!(PlainDate::new(2025, 1, 0).is_none());
    }

    #[test]
    fn from_iso_round_trip() {
        let d = PlainDate::from_iso("2025-06-07").unwrap();
        assert_eq!(d.to_iso(), "2025-06-07");
    }

    #[test]
    fn add_days_crosses_month() {
        let d = PlainDate::new(2025, 1, 31).unwrap().add_days(1);
        assert_eq!((d.year(), d.month(), d.day()), (2025, 2, 1));
    }

    #[test]
    fn add_months_clamps() {
        let d = PlainDate::new(2025, 1, 31).unwrap().add_months(1);
        assert_eq!((d.year(), d.month(), d.day()), (2025, 2, 28));
    }

    #[test]
    fn add_years_leap_clamp() {
        let d = PlainDate::new(2024, 2, 29).unwrap().add_years(1);
        assert_eq!((d.year(), d.month(), d.day()), (2025, 2, 28));
    }

    #[test]
    fn weekday_known() {
        // 2025-01-01 = Wednesday = ISO 3
        assert_eq!(PlainDate::new(2025, 1, 1).unwrap().weekday(), 3);
        // 1970-01-01 = Thursday = ISO 4
        assert_eq!(PlainDate::new(1970, 1, 1).unwrap().weekday(), 4);
    }

    #[test]
    fn days_until() {
        let a = PlainDate::new(2025, 1, 10).unwrap();
        let b = PlainDate::new(2025, 1, 1).unwrap();
        assert_eq!(a.days_until(b), 9);
        assert_eq!(b.days_until(a), -9);
    }
}
