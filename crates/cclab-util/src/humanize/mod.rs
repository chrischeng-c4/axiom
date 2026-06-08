//! Humanize formatting utilities.
//!
//! Convert numbers, durations, and byte sizes into human-readable strings.
//!
//! # Modules
//!
//! - [`numbers`] - Number formatting (intcomma, intword, ordinal, apnumber)
//! - [`time_size`] - Time and size formatting (naturaltime, naturalsize, naturaldelta)

pub mod numbers;
pub mod time_size;

// Re-export all public functions for convenience
pub use numbers::{apnumber, intcomma, intcomma_f64, intword, ordinal};
pub use time_size::{naturaldelta, naturalsize, naturalsize_fmt, naturaltime, SizeFormat};
