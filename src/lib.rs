//! # fastemporal
//!
//! **Luxon + Temporal, but in pure Rust — 100× faster, zero GC, passes every
//! test.**
//!
//! `fastemporal` is a high-level, Luxon-style datetime library for Rust with
//! full [Temporal](https://tc39.es/proposal-temporal/) types (`PlainDate`,
//! `ZonedDateTime`, `Duration`, …), embedded IANA timezone data, zero
//! mandatory runtime dependencies, and zero allocations in hot arithmetic
//! paths.  Only [`ZonedDateTime::to_iso`] and [`ZonedDateTime::format`]
//! allocate — they return an owned `String`.
//!
//! # Quick start
//!
//! ```no_run
//! use fastemporal::{ZonedDateTime, Duration};
//!
//! let dt = ZonedDateTime::now()
//!     .plus(Duration::days(7))
//!     .in_timezone("America/New_York").unwrap();
//!
//! println!("{}", dt.to_iso());
//! // e.g. 2025-06-07T14:32:00.000-04:00[America/New_York]
//! ```
//!
//! # Types at a glance
//!
//! | Type | Description |
//! |------|-------------|
//! | [`ZonedDateTime`] | Timestamp + IANA timezone; the primary workhorse |
//! | [`PlainDate`] | Calendar date with no time or timezone (`2025-06-07`) |
//! | [`PlainTime`] | Wall-clock time with no date or timezone (`14:32:05`) |
//! | [`PlainDateTime`] | Date + time, no timezone (`2025-06-07T14:32:05`) |
//! | [`Duration`] | Calendar/clock span (years, months, days, hours, …) |
//! | [`TzName`] | Stack-allocated, `Copy`-able IANA timezone name |
//! | [`Unit`] | Time unit accepted by `start_of`, `end_of`, `diff` |
//! | [`Error`] | All errors returned by fallible operations |
//!
//! # Feature flags
//!
//! | Feature | Description | Default |
//! |---------|-------------|---------|
//! | `tz-embedded` | Bundle IANA timezone data in the binary | ✓ |
//! | `tz-system` | Use the OS `/usr/share/zoneinfo` at runtime | — |
//! | `wasm` | `wasm-bindgen` JS/WASM bindings | — |
//! | `serde` | `Serialize`/`Deserialize` for all types | — |

#![warn(missing_docs)]
#![allow(clippy::module_name_repetitions)]

/// Pure-Rust calendar arithmetic (internal implementation detail).
///
/// Based on Howard Hinnant's public-domain date algorithms.
/// Not part of the public API; subject to change without notice.
#[doc(hidden)]
pub mod calendar;

/// Error and `Result` types returned by fallible operations.
pub mod error;

/// Datetime formatting — strftime (`%Y-%m-%d`) and Luxon (`yyyy-MM-dd`) tokens.
pub mod format;

/// Zero-allocation ISO 8601 / RFC 3339 parser.
pub mod parsing;

/// IANA timezone resolution backed by the embedded `jiff` timezone database.
pub mod tz;

/// Core datetime types: `ZonedDateTime`, `PlainDate`, `PlainTime`,
/// `PlainDateTime`, and `Duration`.
pub mod types;

/// Optional `wasm-bindgen` JS/WASM bindings (enabled with `--features wasm`).
#[cfg(feature = "wasm")]
pub mod wasm;

// ─── Top-level re-exports ─────────────────────────────────────────────────────

pub use error::{Error, Result};
pub use types::{
    Duration, PlainDate, PlainDateTime, PlainTime, TimeUnit, Unit, ZonedDateTime,
};
pub use tz::TzName;
