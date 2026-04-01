//! Error types for `fastemporal`.
/// The error type returned by fallible `fastemporal` operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// An ISO 8601 string could not be parsed.
    Parse(String),
    /// The requested IANA timezone name was not found.
    InvalidTimezone(String),
    /// An arithmetic operation overflowed the representable range.
    Overflow,
    /// A unit string (e.g. `"days"`, `"months"`) was not recognised.
    InvalidUnit(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Parse(msg) => write!(f, "parse error: {msg}"),
            Error::InvalidTimezone(name) => write!(f, "unknown timezone: {name}"),
            Error::Overflow => write!(f, "datetime arithmetic overflow"),
            Error::InvalidUnit(u) => write!(f, "unknown unit: {u}"),
        }
    }
}

impl std::error::Error for Error {}

/// Convenience alias used internally and re-exported.
pub type Result<T> = std::result::Result<T, Error>;
