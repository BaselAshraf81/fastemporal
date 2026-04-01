//! Datetime formatting — strftime-style (`%Y-%m-%d`) **and** Luxon-style
//! (`yyyy-MM-dd HH:mm:ss`) tokens.
//!
//! The computation is zero-allocation; only the final [`String`] return
//! allocates (unavoidably).
#![allow(missing_docs)]

use crate::calendar::local_fields;

/// Format context passed to the renderer.
#[derive(Clone, Copy)]
pub struct FormatCtx {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub nanosecond: u32,
    pub weekday_iso: u8, // 1=Mon … 7=Sun
    pub offset_secs: i32,
    pub tz_abbr: &'static str,
}

impl FormatCtx {
    pub fn from_ts(ts_nanos: i64, offset_secs: i32) -> Self {
        let (year, month, day, hour, minute, second, nanosecond) =
            local_fields(ts_nanos, offset_secs);
        let days = (ts_nanos + offset_secs as i64 * crate::calendar::NANOS_PER_SEC)
            .div_euclid(crate::calendar::NANOS_PER_DAY);
        let weekday_iso = crate::calendar::weekday_from_days(days);
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            nanosecond,
            weekday_iso,
            offset_secs,
            tz_abbr: "",
        }
    }
}

/// Format a datetime using a format string.
///
/// Supported tokens:
///
/// | Token | Meaning | Example |
/// |-------|---------|---------|
/// | `%Y` / `yyyy` | 4-digit year | `2025` |
/// | `%y` / `yy` | 2-digit year | `25` |
/// | `%m` / `MM` | 2-digit month | `06` |
/// | `%d` / `dd` | 2-digit day | `07` |
/// | `%H` / `HH` | 2-digit hour (24h) | `14` |
/// | `%I` | 2-digit hour (12h) | `02` |
/// | `%M` / `mm` | 2-digit minute | `32` |
/// | `%S` / `ss` | 2-digit second | `00` |
/// | `%f` | 9-digit nanoseconds | `000000000` |
/// | `%3f` / `SSS` | 3-digit milliseconds | `000` |
/// | `%6f` | 6-digit microseconds | `000000` |
/// | `%A` | Full weekday name | `Saturday` |
/// | `%a` | Short weekday name | `Sat` |
/// | `%B` | Full month name | `June` |
/// | `%b` | Short month name | `Jun` |
/// | `%Z` / `z` | UTC offset (`+HH:MM`) | `-04:00` |
/// | `%z` | Compact UTC offset (`+HHMM`) | `-0400` |
/// | `%%` | Literal `%` | `%` |
///
/// Any other `%X` or Luxon-style token that is not recognised is passed
/// through unchanged.
///
/// # Examples
/// ```
/// use fastemporal::format::strftime::{FormatCtx, format_dt};
///
/// let ctx = FormatCtx {
///     year: 2025, month: 6, day: 7,
///     hour: 14, minute: 32, second: 0, nanosecond: 0,
///     weekday_iso: 6, offset_secs: -4 * 3600, tz_abbr: "EDT",
/// };
/// assert_eq!(format_dt("%Y-%m-%d", &ctx), "2025-06-07");
/// assert_eq!(format_dt("yyyy-MM-dd", &ctx), "2025-06-07");
/// assert_eq!(format_dt("HH:mm:ss", &ctx), "14:32:00");
/// ```
pub fn format_dt(fmt: &str, ctx: &FormatCtx) -> String {
    let mut out = String::with_capacity(fmt.len() + 8);
    let bytes = fmt.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // ── strftime-style (%X) ───────────────────────────────────────────────
        if bytes[i] == b'%' && i + 1 < bytes.len() {
            let next = bytes[i + 1];
            if next == b'%' {
                out.push('%');
                i += 2;
                continue;
            }
            // Check for %3f / %6f / %9f
            if next.is_ascii_digit() && i + 2 < bytes.len() && bytes[i + 2] == b'f' {
                let n = (next - b'0') as usize;
                push_frac(&mut out, ctx.nanosecond, n);
                i += 3;
                continue;
            }
            let consumed = match next {
                b'Y' => { push_year4(&mut out, ctx.year); 2 }
                b'y' => { push_d2(&mut out, (ctx.year.abs() % 100) as u8); 2 }
                b'm' => { push_d2(&mut out, ctx.month); 2 }
                b'd' => { push_d2(&mut out, ctx.day); 2 }
                b'H' => { push_d2(&mut out, ctx.hour); 2 }
                b'I' => { push_d2(&mut out, hour12(ctx.hour)); 2 }
                b'M' => { push_d2(&mut out, ctx.minute); 2 }
                b'S' => { push_d2(&mut out, ctx.second); 2 }
                b'f' => { push_frac(&mut out, ctx.nanosecond, 9); 2 }
                b'A' => { out.push_str(weekday_full(ctx.weekday_iso)); 2 }
                b'a' => { out.push_str(weekday_short(ctx.weekday_iso)); 2 }
                b'B' => { out.push_str(month_full(ctx.month)); 2 }
                b'b' => { out.push_str(month_short(ctx.month)); 2 }
                b'Z' => { push_offset_colon(&mut out, ctx.offset_secs); 2 }
                b'z' => { push_offset_compact(&mut out, ctx.offset_secs); 2 }
                b'p' | b'P' => {
                    let ampm = match (ctx.hour < 12, next == b'P') {
                        (true,  true)  => "am",
                        (true,  false) => "AM",
                        (false, true)  => "pm",
                        (false, false) => "PM",
                    };
                    out.push_str(ampm);
                    2
                }
                _ => { out.push(b'%' as char); 1 } // pass through unknown
            };
            i += consumed;
            continue;
        }

        // ── Luxon-style multi-character tokens ───────────────────────────────
        let rem = &fmt[i..];

        if let Some(adv) = try_luxon_token(rem, ctx, &mut out) {
            i += adv;
            continue;
        }

        // Single-quoted literal: 'text' → text  (Luxon escape convention)
        // Two consecutive apostrophes '' → one literal apostrophe.
        if bytes[i] == b'\'' {
            i += 1; // consume opening quote
            if i < bytes.len() && bytes[i] == b'\'' {
                out.push('\'');
                i += 1;
            } else {
                while i < bytes.len() && bytes[i] != b'\'' {
                    out.push(bytes[i] as char);
                    i += 1;
                }
                if i < bytes.len() { i += 1; } // consume closing quote
            }
            continue;
        }

        // Literal character
        let ch = bytes[i] as char;
        out.push(ch);
        i += 1;
    }
    out
}

/// Try to match a Luxon-style token at the start of `rem`; push result into
/// `out` and return the number of bytes consumed, or `None` if no match.
fn try_luxon_token(rem: &str, ctx: &FormatCtx, out: &mut String) -> Option<usize> {
    // Sorted longest-first to avoid prefix ambiguities.
    type TokenFn = fn(&FormatCtx, &mut String);
    let tokens: &[(&str, TokenFn)] = &[
        ("yyyy", |c, o| push_year4(o, c.year)),
        ("yy",   |c, o| push_d2(o, (c.year.abs() % 100) as u8)),
        ("MMMM", |c, o| o.push_str(month_full(c.month))),
        ("MMM",  |c, o| o.push_str(month_short(c.month))),
        ("MM",   |c, o| push_d2(o, c.month)),
        ("M",    |c, o| o.push_str(&c.month.to_string())),
        ("dd",   |c, o| push_d2(o, c.day)),
        ("d",    |c, o| o.push_str(&c.day.to_string())),
        ("HH",   |c, o| push_d2(o, c.hour)),
        ("H",    |c, o| o.push_str(&c.hour.to_string())),
        ("hh",   |c, o| push_d2(o, hour12(c.hour))),
        ("h",    |c, o| o.push_str(&hour12(c.hour).to_string())),
        ("mm",   |c, o| push_d2(o, c.minute)),
        ("m",    |c, o| o.push_str(&c.minute.to_string())),
        ("ss",   |c, o| push_d2(o, c.second)),
        ("s",    |c, o| o.push_str(&c.second.to_string())),
        ("SSS",  |c, o| push_frac(o, c.nanosecond, 3)),
        ("S",    |c, o| push_frac(o, c.nanosecond, 1)),
        ("EEEE", |c, o| o.push_str(weekday_full(c.weekday_iso))),
        ("EEE",  |c, o| o.push_str(weekday_short(c.weekday_iso))),
        ("a",    |c, o| o.push_str(if c.hour < 12 { "AM" } else { "PM" })),
        ("ZZ",   |c, o| push_offset_colon(o, c.offset_secs)),
        ("Z",    |c, o| push_offset_colon(o, c.offset_secs)),
        ("z",    |c, o| o.push_str(c.tz_abbr)),
    ];

    for (token, render) in tokens {
        if rem.starts_with(token) {
            render(ctx, out);
            return Some(token.len());
        }
    }
    None
}

// ─── Rendering helpers ────────────────────────────────────────────────────────

#[inline]
fn push_d2(out: &mut String, v: u8) {
    out.push((b'0' + v / 10) as char);
    out.push((b'0' + v % 10) as char);
}

#[inline]
fn push_year4(out: &mut String, year: i32) {
    if year < 0 {
        out.push('-');
        push_year4(out, -year);
        return;
    }
    let y = year as u32;
    out.push((b'0' + (y / 1000 % 10) as u8) as char);
    out.push((b'0' + (y / 100 % 10) as u8) as char);
    out.push((b'0' + (y / 10 % 10) as u8) as char);
    out.push((b'0' + (y % 10) as u8) as char);
}

#[inline]
fn hour12(h: u8) -> u8 {
    match h % 12 {
        0 => 12,
        v => v,
    }
}

fn push_frac(out: &mut String, nanosecond: u32, digits: usize) {
    // Truncate (not round) to `digits` significant digits.
    let divisor = 10u32.pow((9 - digits.min(9)) as u32);
    let v = nanosecond / divisor;
    // Zero-pad to `digits` places
    let s = format!("{:0>width$}", v, width = digits);
    out.push_str(&s[..digits.min(s.len())]);
}

fn push_offset_colon(out: &mut String, offset_secs: i32) {
    let (sign, abs) = if offset_secs < 0 {
        ('-', (-offset_secs) as u32)
    } else {
        ('+', offset_secs as u32)
    };
    out.push(sign);
    push_d2(out, (abs / 3600) as u8);
    out.push(':');
    push_d2(out, ((abs % 3600) / 60) as u8);
}

fn push_offset_compact(out: &mut String, offset_secs: i32) {
    let (sign, abs) = if offset_secs < 0 {
        ('-', (-offset_secs) as u32)
    } else {
        ('+', offset_secs as u32)
    };
    out.push(sign);
    push_d2(out, (abs / 3600) as u8);
    push_d2(out, ((abs % 3600) / 60) as u8);
}

// ─── Name tables ─────────────────────────────────────────────────────────────

const MONTH_FULL: [&str; 12] = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];

const MONTH_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

const WEEKDAY_FULL: [&str; 7] = [
    "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday",
];

const WEEKDAY_SHORT: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

fn month_full(m: u8) -> &'static str {
    MONTH_FULL.get(m.saturating_sub(1) as usize).copied().unwrap_or("")
}
fn month_short(m: u8) -> &'static str {
    MONTH_SHORT.get(m.saturating_sub(1) as usize).copied().unwrap_or("")
}
fn weekday_full(iso: u8) -> &'static str {
    WEEKDAY_FULL.get(iso.saturating_sub(1) as usize).copied().unwrap_or("")
}
fn weekday_short(iso: u8) -> &'static str {
    WEEKDAY_SHORT.get(iso.saturating_sub(1) as usize).copied().unwrap_or("")
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> FormatCtx {
        FormatCtx {
            year: 2025,
            month: 6,
            day: 7,
            hour: 14,
            minute: 32,
            second: 5,
            nanosecond: 123_456_789,
            weekday_iso: 6, // Saturday
            offset_secs: -4 * 3600,
            tz_abbr: "EDT",
        }
    }

    #[test]
    fn strftime_iso() {
        assert_eq!(format_dt("%Y-%m-%dT%H:%M:%S", &ctx()), "2025-06-07T14:32:05");
    }

    #[test]
    fn luxon_iso() {
        assert_eq!(format_dt("yyyy-MM-dd'T'HH:mm:ss", &ctx()), "2025-06-07T14:32:05");
    }

    #[test]
    fn millis() {
        assert_eq!(format_dt("SSS", &ctx()), "123");
        assert_eq!(format_dt("%3f", &ctx()), "123");
    }

    #[test]
    fn nanos() {
        assert_eq!(format_dt("%f", &ctx()), "123456789");
    }

    #[test]
    fn offset_colon() {
        assert_eq!(format_dt("%Z", &ctx()), "-04:00");
    }

    #[test]
    fn month_names() {
        assert_eq!(format_dt("%B", &ctx()), "June");
        assert_eq!(format_dt("%b", &ctx()), "Jun");
    }

    #[test]
    fn weekday_names() {
        assert_eq!(format_dt("%A", &ctx()), "Saturday");
        assert_eq!(format_dt("%a", &ctx()), "Sat");
    }
}
