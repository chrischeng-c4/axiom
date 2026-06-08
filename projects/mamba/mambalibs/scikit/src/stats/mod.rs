//! Statistical functions module (scipy.stats equivalent).
//!
//! Provides probability distributions, hypothesis testing, and descriptive statistics.

mod descriptive;
mod distributions;
mod hypothesis;

pub use descriptive::*;
pub use distributions::*;
pub use hypothesis::*;
