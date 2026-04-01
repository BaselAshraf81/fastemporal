/// A fixed-size, `Copy`-able IANA timezone name that fits on the stack.
///
/// The longest valid IANA timezone name currently is 32 bytes
/// (`America/Argentina/ComodRivadavia`).  We reserve 48 bytes to be safe.
///
/// # Examples
/// ```
/// use fastemporal::tz::TzName;
/// let tz = TzName::new("America/New_York").unwrap();
/// assert_eq!(tz.as_str(), "America/New_York");
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TzName {
    data: [u8; 48],
    len: u8,
}

impl TzName {
    /// The UTC timezone sentinel.
    pub const UTC: Self = {
        let mut data = [0u8; 48];
        data[0] = b'U';
        data[1] = b'T';
        data[2] = b'C';
        Self { data, len: 3 }
    };

    /// Construct from a string slice, returning `None` if the name is longer
    /// than 47 bytes.
    pub fn new(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        if bytes.len() > 47 {
            return None;
        }
        let mut data = [0u8; 48];
        data[..bytes.len()].copy_from_slice(bytes);
        Some(Self {
            data,
            len: bytes.len() as u8,
        })
    }

    /// Returns the timezone name as a `&str`.
    #[inline]
    pub fn as_str(&self) -> &str {
        // SAFETY: constructed only from valid UTF-8 slices.
        unsafe { core::str::from_utf8_unchecked(&self.data[..self.len as usize]) }
    }

    /// Returns `true` if this represents UTC.
    #[inline]
    pub fn is_utc(&self) -> bool {
        self.as_str() == "UTC"
    }
}

impl core::fmt::Debug for TzName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TzName({:?})", self.as_str())
    }
}

impl core::fmt::Display for TzName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for TzName {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for TzName {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        TzName::new(s).ok_or_else(|| serde::de::Error::custom("timezone name too long"))
    }
}

// ─── Resolution ──────────────────────────────────────────────────────────────

use crate::error::{Error, Result};

/// Returns `(utc_offset_seconds, is_dst)` for the given IANA timezone name at
/// the given Unix timestamp (in **seconds**).
///
/// When the `tz-embedded` or `tz-system` feature is not enabled this always
/// returns `Ok((0, false))` (i.e., UTC).
pub fn resolve_offset(tz: &TzName, unix_secs: i64) -> Result<(i32, bool)> {
    if tz.is_utc() {
        return Ok((0, false));
    }

    #[cfg(any(feature = "tz-embedded", feature = "tz-system"))]
    {
        use jiff::tz::{Dst, TimeZone};
        use jiff::Timestamp;

        let timezone = TimeZone::get(tz.as_str())
            .map_err(|_| Error::InvalidTimezone(tz.as_str().to_string()))?;
        let ts = Timestamp::from_second(unix_secs)
            .map_err(|_| Error::Overflow)?;
        let info = timezone.to_offset_info(ts);
        let offset_secs: i32 = info.offset().seconds();
        let is_dst = info.dst() == Dst::Yes;
        return Ok((offset_secs, is_dst));
    }

    #[allow(unreachable_code)]
    Ok((0, false))
}

/// Converts a local (wall-clock) datetime to a UTC nanosecond timestamp,
/// resolving DST ambiguity with "prefer earlier transition" semantics
/// (matches Luxon's default).
///
/// Returns `(ts_nanos, offset_secs)`.
#[allow(clippy::too_many_arguments)]
pub fn local_to_utc(
    tz: &TzName,
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    nanosecond: u32,
) -> Result<(i64, i32)> {
    use crate::calendar::{days_from_civil, NANOS_PER_DAY, NANOS_PER_SEC};

    if tz.is_utc() {
        let ts = crate::calendar::ts_from_fields(year, month, day, hour, minute, second, nanosecond, 0);
        return Ok((ts, 0));
    }

    #[cfg(any(feature = "tz-embedded", feature = "tz-system"))]
    {
        use jiff::tz::TimeZone;
        use jiff::civil::DateTime as JiffDateTime;

        let timezone = TimeZone::get(tz.as_str())
            .map_err(|_| Error::InvalidTimezone(tz.as_str().to_string()))?;

        let jdt = JiffDateTime::new(
            year as i16,
            month as i8,
            day as i8,
            hour as i8,
            minute as i8,
            second as i8,
            nanosecond as i32,
        )
        .map_err(|_| Error::Overflow)?;

        // Use compatible() which picks the earlier (pre-fold) time on ambiguity
        // and pushes forward (post-gap) on a DST gap — matching Luxon semantics.
        let ts = timezone
            .to_ambiguous_timestamp(jdt)
            .compatible()
            .map_err(|_| Error::Overflow)?;

        let offset_secs: i32 = timezone.to_offset(ts).seconds();
        // Convert jiff Timestamp (seconds + subseconds) to nanoseconds i64
        let ts_nanos = ts.as_second() * NANOS_PER_SEC
            + ts.subsec_nanosecond() as i64;
        return Ok((ts_nanos, offset_secs));
    }

    // Fallback when no TZ feature: treat as UTC
    #[allow(unreachable_code)]
    {
        let days = days_from_civil(year, month, day);
        let ts = days * NANOS_PER_DAY
            + hour as i64 * 3600 * NANOS_PER_SEC
            + minute as i64 * 60 * NANOS_PER_SEC
            + second as i64 * NANOS_PER_SEC
            + nanosecond as i64;
        Ok((ts, 0))
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tzname_utc_roundtrip() {
        assert_eq!(TzName::UTC.as_str(), "UTC");
        assert!(TzName::UTC.is_utc());
    }

    #[test]
    fn tzname_new() {
        let tz = TzName::new("America/New_York").unwrap();
        assert_eq!(tz.as_str(), "America/New_York");
    }

    #[test]
    fn resolve_utc_always_zero() {
        let (off, dst) = resolve_offset(&TzName::UTC, 0).unwrap();
        assert_eq!(off, 0);
        assert!(!dst);
    }

    #[cfg(any(feature = "tz-embedded", feature = "tz-system"))]
    #[test]
    fn resolve_new_york_winter() {
        // 2025-01-01T00:00:00Z — New York is UTC-5 (EST) in January
        let tz = TzName::new("America/New_York").unwrap();
        let (off, dst) = resolve_offset(&tz, 1_735_689_600).unwrap();
        assert_eq!(off, -5 * 3600);
        assert!(!dst);
    }

    #[cfg(any(feature = "tz-embedded", feature = "tz-system"))]
    #[test]
    fn resolve_new_york_summer() {
        // 2025-07-01T00:00:00Z — New York is UTC-4 (EDT) in July
        let tz = TzName::new("America/New_York").unwrap();
        let (off, dst) = resolve_offset(&tz, 1_751_328_000).unwrap();
        assert_eq!(off, -4 * 3600);
        assert!(dst);
    }

    #[cfg(any(feature = "tz-embedded", feature = "tz-system"))]
    #[test]
    fn local_to_utc_new_york() {
        let tz = TzName::new("America/New_York").unwrap();
        // 2025-01-01T00:00:00 local NY = 2025-01-01T05:00:00Z
        let (ts_nanos, off) = local_to_utc(&tz, 2025, 1, 1, 0, 0, 0, 0).unwrap();
        assert_eq!(off, -5 * 3600);
        // Expected UTC nanos: 1735689600 * 1e9 + 5*3600 * 1e9
        let expected = 1_735_689_600_i64 * 1_000_000_000 + 5 * 3600 * 1_000_000_000_i64;
        assert_eq!(ts_nanos, expected);
    }
}
