//! Hand-rolled, zero-allocation ISO 8601 parser.
//!
//! Handles the subset of ISO 8601 / RFC 3339 used in practice:
//!
//! ```text
//! YYYY-MM-DD
//! YYYY-MM-DDTHH:MM
//! YYYY-MM-DDTHH:MM:SS
//! YYYY-MM-DDTHH:MM:SS.sss            (1-9 fractional digits)
//! YYYY-MM-DDTHH:MM:SS.sssZ
//! YYYY-MM-DDTHH:MM:SS.sss+HH:MM
//! YYYY-MM-DDTHH:MM:SS.sss-HH:MM
//! YYYY-MM-DDTHH:MM:SS[America/New_York]
//! YYYY-MM-DDTHH:MM:SS.sss+HH:MM[America/New_York]
//! ```
#![allow(missing_docs)]

use crate::error::{Error, Result};

/// Parsed result of [`parse_iso`].
#[derive(Debug, PartialEq, Eq)]
pub struct IsoFields {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    /// Sub-second component, always stored as **nanoseconds** (0–999_999_999).
    pub nanosecond: u32,
    /// UTC offset in seconds, `None` when no offset or timezone was present
    /// (i.e., a "floating" / local datetime like `2025-01-01T12:00`).
    pub offset_secs: Option<i32>,
    /// IANA timezone name embedded in brackets, e.g. `America/New_York`.
    pub tz_name: Option<heapless::TzStr>,
}

/// A tiny stack-allocated string (≤48 bytes) for the inline timezone name.
pub mod heapless {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TzStr {
        data: [u8; 48],
        len: u8,
    }
    impl TzStr {
        pub(super) fn from_bytes(b: &[u8]) -> Option<Self> {
            if b.len() > 47 {
                return None;
            }
            let mut data = [0u8; 48];
            data[..b.len()].copy_from_slice(b);
            Some(Self { data, len: b.len() as u8 })
        }
        pub fn as_str(&self) -> &str {
            unsafe { core::str::from_utf8_unchecked(&self.data[..self.len as usize]) }
        }
    }
    impl core::fmt::Display for TzStr {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str(self.as_str())
        }
    }
}

// ─── Parser state ─────────────────────────────────────────────────────────────

struct Parser<'a> {
    src: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    #[inline]
    fn new(s: &'a str) -> Self {
        Self { src: s.as_bytes(), pos: 0 }
    }

    #[inline]
    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    #[inline]
    fn eat(&mut self) -> Option<u8> {
        let b = self.src.get(self.pos).copied();
        if b.is_some() {
            self.pos += 1;
        }
        b
    }

    /// Consume `expected` byte or return an error.
    fn expect(&mut self, expected: u8) -> Result<()> {
        match self.eat() {
            Some(b) if b == expected => Ok(()),
            Some(b) => Err(Error::Parse(format!(
                "expected '{}' but got '{}' at pos {}",
                expected as char, b as char, self.pos - 1
            ))),
            None => Err(Error::Parse(format!(
                "expected '{}' but reached end of input",
                expected as char
            ))),
        }
    }

    /// Parse exactly `n` ASCII decimal digits and return their value.
    fn digits(&mut self, n: usize) -> Result<u64> {
        let mut v: u64 = 0;
        for _ in 0..n {
            match self.eat() {
                Some(b) if b.is_ascii_digit() => v = v * 10 + (b - b'0') as u64,
                Some(b) => {
                    return Err(Error::Parse(format!(
                        "expected digit but got '{}' at pos {}",
                        b as char,
                        self.pos - 1
                    )))
                }
                None => {
                    return Err(Error::Parse(
                        "unexpected end of input while parsing digits".into(),
                    ))
                }
            }
        }
        Ok(v)
    }

    /// Parse 1–9 ASCII digits (for fractional seconds).  Returns (value, count).
    fn frac_digits(&mut self) -> (u64, usize) {
        let mut v: u64 = 0;
        let mut n: usize = 0;
        while n < 9 {
            match self.peek() {
                Some(b) if b.is_ascii_digit() => {
                    self.pos += 1;
                    v = v * 10 + (b - b'0') as u64;
                    n += 1;
                }
                _ => break,
            }
        }
        (v, n)
    }

    fn remaining(&self) -> &[u8] {
        &self.src[self.pos..]
    }

    fn is_done(&self) -> bool {
        self.pos >= self.src.len()
    }
}

// ─── Public entry point ───────────────────────────────────────────────────────

/// Parse an ISO 8601 / RFC 3339 string into its component fields.
///
/// # Errors
/// Returns [`Error::Parse`] if the string does not conform to the supported
/// subset of ISO 8601.
///
/// # Examples
/// ```
/// use fastemporal::parsing::iso8601::parse_iso;
///
/// let f = parse_iso("2025-06-07T14:32:00.000-04:00").unwrap();
/// assert_eq!(f.year, 2025);
/// assert_eq!(f.month, 6);
/// assert_eq!(f.day, 7);
/// assert_eq!(f.hour, 14);
/// assert_eq!(f.minute, 32);
/// assert_eq!(f.second, 0);
/// assert_eq!(f.offset_secs, Some(-4 * 3600));
/// ```
pub fn parse_iso(s: &str) -> Result<IsoFields> {
    let mut p = Parser::new(s);

    // ── Date part ────────────────────────────────────────────────────────────
    let year = p.digits(4)? as i32;
    p.expect(b'-')?;
    let month = p.digits(2)? as u8;
    p.expect(b'-')?;
    let day = p.digits(2)? as u8;

    validate_date(year, month, day)?;

    // ── Optional time part ───────────────────────────────────────────────────
    let (hour, minute, second, nanosecond) = if matches!(p.peek(), Some(b'T') | Some(b't') | Some(b' ')) {
        p.eat(); // consume T/t/space
        let hour = p.digits(2)? as u8;
        p.expect(b':')?;
        let minute = p.digits(2)? as u8;
        let (second, nanosecond) = if p.peek() == Some(b':') {
            p.eat();
            let second = p.digits(2)? as u8;
            let nanosecond = if matches!(p.peek(), Some(b'.') | Some(b',')) {
                p.eat();
                let (v, n) = p.frac_digits();
                if n == 0 {
                    return Err(Error::Parse("expected fractional digits after '.'".into()));
                }
                // Normalise to nanoseconds
                let nanos = v * 10u64.pow((9 - n) as u32);
                nanos as u32
            } else {
                0
            };
            (second, nanosecond)
        } else {
            (0, 0)
        };
        validate_time(hour, minute, second)?;
        (hour, minute, second, nanosecond)
    } else {
        (0, 0, 0, 0)
    };

    // ── Optional offset / tz ─────────────────────────────────────────────────
    let (offset_secs, tz_name) = parse_offset_and_tz(&mut p)?;

    if !p.is_done() {
        return Err(Error::Parse(format!(
            "unexpected trailing input: {:?}",
            std::str::from_utf8(p.remaining()).unwrap_or("(invalid utf8)")
        )));
    }

    Ok(IsoFields {
        year,
        month,
        day,
        hour,
        minute,
        second,
        nanosecond,
        offset_secs,
        tz_name,
    })
}

fn parse_offset_and_tz(p: &mut Parser<'_>) -> Result<(Option<i32>, Option<heapless::TzStr>)> {
    let mut offset_secs: Option<i32> = None;
    let mut tz_name: Option<heapless::TzStr> = None;

    match p.peek() {
        Some(b'Z') | Some(b'z') => {
            p.eat();
            offset_secs = Some(0);
        }
        Some(b'+') | Some(b'-') => {
            let sign: i32 = if p.eat() == Some(b'+') { 1 } else { -1 };
            let oh = p.digits(2)? as i32;
            let om = if p.peek() == Some(b':') {
                p.eat();
                p.digits(2)? as i32
            } else if p.peek().is_some_and(|b| b.is_ascii_digit()) {
                p.digits(2)? as i32
            } else {
                0
            };
            offset_secs = Some(sign * (oh * 3600 + om * 60));
        }
        _ => {}
    }

    // Optional IANA timezone in brackets: [America/New_York]
    if p.peek() == Some(b'[') {
        p.eat();
        let start = p.pos;
        while p.peek().is_some_and(|b| b != b']') {
            p.eat();
        }
        let tz_bytes = &p.src[start..p.pos];
        p.expect(b']')?;
        tz_name = heapless::TzStr::from_bytes(tz_bytes)
            .ok_or_else(|| Error::Parse("timezone name too long".into()))
            .map(Some)?;
    }

    Ok((offset_secs, tz_name))
}

// ─── Validation helpers ────────────────────────────────────────────────────────

fn validate_date(year: i32, month: u8, day: u8) -> Result<()> {
    if !(1..=12).contains(&month) {
        return Err(Error::Parse(format!("invalid month: {month}")));
    }
    let max_day = crate::calendar::days_in_month(year, month);
    if day < 1 || day > max_day {
        return Err(Error::Parse(format!("invalid day {day} for {year}-{month:02}")));
    }
    Ok(())
}

fn validate_time(hour: u8, minute: u8, second: u8) -> Result<()> {
    if hour > 23 {
        return Err(Error::Parse(format!("invalid hour: {hour}")));
    }
    if minute > 59 {
        return Err(Error::Parse(format!("invalid minute: {minute}")));
    }
    if second > 60 {
        // allow leap second 60
        return Err(Error::Parse(format!("invalid second: {second}")));
    }
    Ok(())
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_only() {
        let f = parse_iso("2025-03-15").unwrap();
        assert_eq!((f.year, f.month, f.day), (2025, 3, 15));
        assert_eq!((f.hour, f.minute, f.second), (0, 0, 0));
        assert_eq!(f.offset_secs, None);
    }

    #[test]
    fn datetime_utc() {
        let f = parse_iso("2025-06-07T14:32:00Z").unwrap();
        assert_eq!(f.hour, 14);
        assert_eq!(f.minute, 32);
        assert_eq!(f.offset_secs, Some(0));
    }

    #[test]
    fn datetime_with_millis() {
        let f = parse_iso("2025-06-07T14:32:00.123Z").unwrap();
        assert_eq!(f.nanosecond, 123_000_000);
    }

    #[test]
    fn datetime_with_nanos() {
        let f = parse_iso("2025-06-07T14:32:00.123456789Z").unwrap();
        assert_eq!(f.nanosecond, 123_456_789);
    }

    #[test]
    fn datetime_negative_offset() {
        let f = parse_iso("2025-06-07T14:32:00.000-04:00").unwrap();
        assert_eq!(f.offset_secs, Some(-4 * 3600));
    }

    #[test]
    fn datetime_positive_offset() {
        let f = parse_iso("2025-06-07T14:32:00+05:30").unwrap();
        assert_eq!(f.offset_secs, Some(5 * 3600 + 30 * 60));
    }

    #[test]
    fn datetime_with_tz_bracket() {
        let f = parse_iso("2025-06-07T14:32:00-04:00[America/New_York]").unwrap();
        assert_eq!(f.offset_secs, Some(-4 * 3600));
        assert_eq!(f.tz_name.as_ref().unwrap().as_str(), "America/New_York");
    }

    #[test]
    fn datetime_no_seconds() {
        let f = parse_iso("2025-06-07T14:32").unwrap();
        assert_eq!((f.hour, f.minute, f.second), (14, 32, 0));
    }

    #[test]
    fn invalid_month() {
        assert!(parse_iso("2025-13-01").is_err());
    }

    #[test]
    fn invalid_day() {
        assert!(parse_iso("2025-02-30").is_err());
    }

    #[test]
    fn leap_day_valid() {
        assert!(parse_iso("2024-02-29").is_ok());
        assert!(parse_iso("2023-02-29").is_err());
    }

    #[test]
    fn floating_datetime() {
        // No offset → offset_secs should be None
        let f = parse_iso("2025-06-07T14:32:00").unwrap();
        assert_eq!(f.offset_secs, None);
    }
}
