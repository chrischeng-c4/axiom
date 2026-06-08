//! Fuzzy string matching module.
//!
//! Provides string distance algorithms and fuzzy search capabilities:
//! - Levenshtein distance
//! - Damerau-Levenshtein distance
//! - Jaro-Winkler similarity
//! - Hamming distance
//! - Fuzzy search with configurable thresholds

mod distance;
mod search;

pub use distance::{
    damerau_levenshtein, hamming, jaro_winkler, levenshtein, levenshtein_normalized,
};
pub use search::{extract, extract_one, fuzzy_search, FuzzyMatch, FuzzyMetric, FuzzySearcher};
