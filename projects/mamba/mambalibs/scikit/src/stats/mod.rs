//! Statistical functions module (scipy.stats equivalent).
//!
//! Provides probability distributions, hypothesis testing, and descriptive statistics.

mod distributions;
mod hypothesis;
mod descriptive;

pub use distributions::*;
pub use hypothesis::*;
pub use descriptive::*;
