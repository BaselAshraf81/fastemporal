/// A calendar/time duration, compatible with the Temporal `Duration` type and
/// Luxon's duration objects.
///
/// Constructors (static methods) use the unit name directly (`Duration::days(7)`).
/// Getters use the `num_` prefix to avoid name conflicts (`d.num_days()`),
/// following the convention of the `chrono` crate.
///
/// # Examples
/// ```
/// use fastemporal::Duration;
///
/// let d = Duration::days(7);
/// assert_eq!(d.num_days(), 7);
///
/// let d2 = Duration::builder().years(1).months(2).days(3).build();
/// assert_eq!(d2.num_years(), 1);
/// assert_eq!(d2.num_months(), 2);
/// assert_eq!(d2.num_days(), 3);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Duration {
    pub(crate) yrs: i32,
    pub(crate) mos: i32,
    pub(crate) wks: i32,
    pub(crate) dys: i32,
    pub(crate) hrs: i32,
    pub(crate) mins: i32,
    pub(crate) secs: i32,
    pub(crate) millis: i32,
    pub(crate) micros: i32,
    pub(crate) nanos: i32,
}

// ─── Builder ──────────────────────────────────────────────────────────────────

/// Fluent builder for constructing a [`Duration`] with multiple fields.
///
/// Obtain via [`Duration::builder()`].
///
/// # Examples
/// ```
/// use fastemporal::Duration;
///
/// let d = Duration::builder().hours(2).minutes(30).build();
/// assert_eq!(d.num_hours(), 2);
/// assert_eq!(d.num_minutes(), 30);
/// ```
#[derive(Default, Clone, Copy)]
pub struct DurationBuilder(Duration);

impl DurationBuilder {
    /// Set the years field.
    pub const fn years(mut self, n: i32) -> Self   { self.0.yrs   = n; self }
    /// Set the months field.
    pub const fn months(mut self, n: i32) -> Self  { self.0.mos   = n; self }
    /// Set the weeks field.
    pub const fn weeks(mut self, n: i32) -> Self   { self.0.wks   = n; self }
    /// Set the days field.
    pub const fn days(mut self, n: i32) -> Self    { self.0.dys   = n; self }
    /// Set the hours field.
    pub const fn hours(mut self, n: i32) -> Self   { self.0.hrs   = n; self }
    /// Set the minutes field.
    pub const fn minutes(mut self, n: i32) -> Self { self.0.mins  = n; self }
    /// Set the seconds field.
    pub const fn seconds(mut self, n: i32) -> Self { self.0.secs  = n; self }
    /// Set the milliseconds field.
    pub const fn millis(mut self, n: i32) -> Self  { self.0.millis = n; self }
    /// Set the microseconds field.
    pub const fn micros(mut self, n: i32) -> Self  { self.0.micros = n; self }
    /// Set the nanoseconds field.
    pub const fn nanos(mut self, n: i32) -> Self   { self.0.nanos = n; self }
    /// Finish and return the [`Duration`].
    pub const fn build(self) -> Duration           { self.0 }
}

// ─── Duration impl ────────────────────────────────────────────────────────────

impl Duration {
    // ── Zero / builder ───────────────────────────────────────────────────────

    /// A duration of zero.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::Duration;
    /// assert!(Duration::zero().is_zero());
    /// ```
    pub const fn zero() -> Self {
        Self { yrs: 0, mos: 0, wks: 0, dys: 0, hrs: 0, mins: 0, secs: 0, millis: 0, micros: 0, nanos: 0 }
    }

    /// Start a fluent builder.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::Duration;
    /// let d = Duration::builder().days(7).hours(3).build();
    /// assert_eq!(d.num_days(), 7);
    /// ```
    pub const fn builder() -> DurationBuilder {
        DurationBuilder(Self::zero())
    }

    // ── Single-unit constructors ─────────────────────────────────────────────

    /// Duration of `n` years.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::Duration;
    /// assert_eq!(Duration::years(2).num_years(), 2);
    /// ```
    pub const fn years(n: i32) -> Self    { Self { yrs:   n, ..Self::zero() } }

    /// Duration of `n` months.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::Duration;
    /// assert_eq!(Duration::months(3).num_months(), 3);
    /// ```
    pub const fn months(n: i32) -> Self   { Self { mos:   n, ..Self::zero() } }

    /// Duration of `n` weeks.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::Duration;
    /// assert_eq!(Duration::weeks(2).num_weeks(), 2);
    /// ```
    pub const fn weeks(n: i32) -> Self    { Self { wks:   n, ..Self::zero() } }

    /// Duration of `n` days.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::Duration;
    /// assert_eq!(Duration::days(7).num_days(), 7);
    /// ```
    pub const fn days(n: i32) -> Self     { Self { dys:   n, ..Self::zero() } }

    /// Duration of `n` hours.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::Duration;
    /// assert_eq!(Duration::hours(3).num_hours(), 3);
    /// ```
    pub const fn hours(n: i32) -> Self    { Self { hrs:   n, ..Self::zero() } }

    /// Duration of `n` minutes.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::Duration;
    /// assert_eq!(Duration::minutes(45).num_minutes(), 45);
    /// ```
    pub const fn minutes(n: i32) -> Self  { Self { mins:  n, ..Self::zero() } }

    /// Duration of `n` seconds.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::Duration;
    /// assert_eq!(Duration::seconds(90).num_seconds(), 90);
    /// ```
    pub const fn seconds(n: i32) -> Self  { Self { secs:  n, ..Self::zero() } }

    /// Duration of `n` milliseconds.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::Duration;
    /// assert_eq!(Duration::millis(500).num_milliseconds(), 500);
    /// ```
    pub const fn millis(n: i32) -> Self   { Self { millis: n, ..Self::zero() } }

    /// Duration of `n` nanoseconds.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::Duration;
    /// assert_eq!(Duration::nanos(1000).num_nanoseconds(), 1000);
    /// ```
    pub const fn nanos(n: i32) -> Self    { Self { nanos: n, ..Self::zero() } }

    // Legacy aliases kept for backward compat with test code
    /// Alias for [`Duration::years`].
    pub const fn from_years(n: i32) -> Self   { Self::years(n) }
    /// Alias for [`Duration::months`].
    pub const fn from_months(n: i32) -> Self  { Self::months(n) }
    /// Alias for [`Duration::weeks`].
    pub const fn from_weeks(n: i32) -> Self   { Self::weeks(n) }
    /// Alias for [`Duration::days`].
    pub const fn from_days(n: i32) -> Self    { Self::days(n) }
    /// Alias for [`Duration::hours`].
    pub const fn from_hours(n: i32) -> Self   { Self::hours(n) }
    /// Alias for [`Duration::minutes`].
    pub const fn from_minutes(n: i32) -> Self { Self::minutes(n) }
    /// Alias for [`Duration::seconds`].
    pub const fn from_seconds(n: i32) -> Self { Self::seconds(n) }
    /// Alias for [`Duration::millis`].
    pub const fn from_millis(n: i32) -> Self  { Self::millis(n) }
    /// Alias for [`Duration::nanos`].
    pub const fn from_nanos(n: i32) -> Self   { Self::nanos(n) }

    // ── Getters (num_ prefix avoids conflict with static constructors) ────────

    /// Returns the years component.
    pub const fn num_years(&self) -> i32        { self.yrs }
    /// Returns the months component.
    pub const fn num_months(&self) -> i32       { self.mos }
    /// Returns the weeks component.
    pub const fn num_weeks(&self) -> i32        { self.wks }
    /// Returns the days component.
    pub const fn num_days(&self) -> i32         { self.dys }
    /// Returns the hours component.
    pub const fn num_hours(&self) -> i32        { self.hrs }
    /// Returns the minutes component.
    pub const fn num_minutes(&self) -> i32      { self.mins }
    /// Returns the seconds component.
    pub const fn num_seconds(&self) -> i32      { self.secs }
    /// Returns the milliseconds component.
    pub const fn num_milliseconds(&self) -> i32 { self.millis }
    /// Returns the microseconds component.
    pub const fn num_microseconds(&self) -> i32 { self.micros }
    /// Returns the nanoseconds component.
    pub const fn num_nanoseconds(&self) -> i32  { self.nanos }

    // ── Predicates / derived ─────────────────────────────────────────────────

    /// Returns `true` if all fields are zero.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::Duration;
    /// assert!(Duration::zero().is_zero());
    /// assert!(!Duration::days(1).is_zero());
    /// ```
    pub const fn is_zero(&self) -> bool {
        self.yrs == 0 && self.mos == 0 && self.wks == 0 && self.dys == 0
            && self.hrs == 0 && self.mins == 0 && self.secs == 0
            && self.millis == 0 && self.micros == 0 && self.nanos == 0
    }

    /// Returns a negated copy of this duration.
    ///
    /// # Examples
    /// ```
    /// use fastemporal::Duration;
    /// assert_eq!(Duration::days(3).negate().num_days(), -3);
    /// ```
    pub const fn negate(self) -> Self {
        Self {
            yrs: -self.yrs, mos: -self.mos, wks: -self.wks, dys: -self.dys,
            hrs: -self.hrs, mins: -self.mins, secs: -self.secs,
            millis: -self.millis, micros: -self.micros, nanos: -self.nanos,
        }
    }

    pub(crate) const fn has_calendar_units(&self) -> bool {
        self.yrs != 0 || self.mos != 0 || self.wks != 0 || self.dys != 0
    }

    pub(crate) fn clock_nanos(&self) -> i64 {
        self.hrs as i64 * 3_600 * 1_000_000_000
            + self.mins as i64 * 60 * 1_000_000_000
            + self.secs as i64 * 1_000_000_000
            + self.millis as i64 * 1_000_000
            + self.micros as i64 * 1_000
            + self.nanos as i64
    }

    pub(crate) const fn total_days(&self) -> i32 {
        self.wks * 7 + self.dys
    }
}

impl core::ops::Neg for Duration {
    type Output = Self;
    fn neg(self) -> Self { self.negate() }
}

impl core::fmt::Display for Duration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "P")?;
        if self.yrs != 0  { write!(f, "{}Y", self.yrs)?; }
        if self.mos != 0  { write!(f, "{}M", self.mos)?; }
        if self.wks != 0  { write!(f, "{}W", self.wks)?; }
        if self.dys != 0  { write!(f, "{}D", self.dys)?; }
        let has_time = self.hrs != 0 || self.mins != 0 || self.secs != 0
            || self.millis != 0 || self.micros != 0 || self.nanos != 0;
        if has_time {
            write!(f, "T")?;
            if self.hrs != 0  { write!(f, "{}H", self.hrs)?; }
            if self.mins != 0 { write!(f, "{}M", self.mins)?; }
            let total_nanos = self.secs as i64 * 1_000_000_000
                + self.millis as i64 * 1_000_000
                + self.micros as i64 * 1_000
                + self.nanos as i64;
            if total_nanos != 0 {
                let s = total_nanos / 1_000_000_000;
                let frac = (total_nanos % 1_000_000_000).abs();
                if frac == 0 { write!(f, "{s}S")?; }
                else         { write!(f, "{s}.{frac:09}S")?; }
            }
        }
        if self.is_zero() { write!(f, "T0S")?; }
        Ok(())
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero() { assert!(Duration::zero().is_zero()); }

    #[test]
    fn days_constructor() {
        let d = Duration::days(7);
        assert_eq!(d.num_days(), 7);
        assert!(d.has_calendar_units());
    }

    #[test]
    fn builder() {
        let d = Duration::builder().years(1).months(2).days(3).hours(4).minutes(5).build();
        assert_eq!(d.num_years(), 1);
        assert_eq!(d.num_months(), 2);
        assert_eq!(d.num_days(), 3);
        assert_eq!(d.num_hours(), 4);
        assert_eq!(d.num_minutes(), 5);
    }

    #[test]
    fn negate() {
        assert_eq!(Duration::days(3).negate().num_days(), -3);
    }

    #[test]
    fn clock_nanos() {
        let d = Duration::builder().hours(1).minutes(30).build();
        assert_eq!(d.clock_nanos(), 90 * 60 * 1_000_000_000_i64);
    }

    #[test]
    fn display() {
        assert_eq!(Duration::days(7).to_string(), "P7D");
        assert_eq!(Duration::hours(2).to_string(), "PT2H");
        assert_eq!(Duration::zero().to_string(), "PT0S");
    }
}
