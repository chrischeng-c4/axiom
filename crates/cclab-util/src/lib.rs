//! Cclab Utilities
//!
//! A collection of humanize formatting, iteration helpers, and caching utilities.
//!
//! # Modules
//!
//! - [`humanize`] - Human-readable formatting for numbers, time, and sizes
//! - [`iter`] - Iteration utilities (chunked, windowed, unique, partition, etc.)
//! - [`cache`] - LRU cache with optional TTL
//!
//! # Examples
//!
//! ```rust
//! use cclab_util::humanize;
//! use cclab_util::iter;
//! use cclab_util::cache::LruCache;
//!
//! // Humanize numbers
//! assert_eq!(humanize::intcomma(1_000_000), "1,000,000");
//! assert_eq!(humanize::intword(1_200_000), "1.2 million");
//! assert_eq!(humanize::ordinal(3), "3rd");
//! assert_eq!(humanize::apnumber(5), "five");
//!
//! // Humanize time and size
//! assert_eq!(humanize::naturaltime(3600.0), "an hour ago");
//! assert_eq!(humanize::naturalsize(1_048_576, true), "1.0 MB");
//!
//! // Iteration utilities
//! assert_eq!(iter::chunked(&[1, 2, 3, 4, 5], 2), vec![vec![1, 2], vec![3, 4], vec![5]]);
//! assert_eq!(iter::pairwise(&[1, 2, 3]), vec![(1, 2), (2, 3)]);
//!
//! // LRU Cache
//! let mut cache = LruCache::new(100);
//! cache.put("key", 42);
//! assert_eq!(cache.get(&"key"), Some(&42));
//! ```

pub mod cache;
pub mod humanize;
pub mod iter;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
