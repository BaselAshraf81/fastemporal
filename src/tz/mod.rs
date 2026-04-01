//! IANA timezone resolution.
#![allow(missing_docs)]
pub mod resolve;

pub use resolve::TzName;

/// Re-export the high-level resolution function.
pub use resolve::resolve_offset;
pub use resolve::local_to_utc;
