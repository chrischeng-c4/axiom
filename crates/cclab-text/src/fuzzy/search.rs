//! Fuzzy search with configurable thresholds.
//!
//! Provides fuzzy string matching and search over collections of strings.

use super::distance::{jaro_winkler, levenshtein_normalized};

/// A match result from fuzzy search.
#[derive(Debug, Clone)]
pub struct FuzzyMatch {
    /// The matched string.
    pub text: String,
    /// The similarity score (0.0 to 1.0).
    pub score: f64,
    /// The index in the original collection.
    pub index: usize,
}

/// Distance metric to use for fuzzy matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuzzyMetric {
    /// Normalized Levenshtein similarity.
    Levenshtein,
    /// Jaro-Winkler similarity.
    JaroWinkler,
}

/// Fuzzy searcher that finds best matches from a collection.
#[derive(Debug, Clone)]
pub struct FuzzySearcher {
    items: Vec<String>,
    metric: FuzzyMetric,
    threshold: f64,
    prefix_weight: f64,
}

impl FuzzySearcher {
    /// Create a new fuzzy searcher with default settings.
    ///
    /// Defaults: Levenshtein metric, threshold 0.6.
    pub fn new(items: Vec<String>) -> Self {
        Self {
            items,
            metric: FuzzyMetric::Levenshtein,
            threshold: 0.6,
            prefix_weight: 0.1,
        }
    }

    /// Set the distance metric.
    pub fn with_metric(mut self, metric: FuzzyMetric) -> Self {
        self.metric = metric;
        self
    }

    /// Set the minimum similarity threshold (0.0 to 1.0).
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Set the prefix weight for Jaro-Winkler (0.0 to 0.25).
    pub fn with_prefix_weight(mut self, weight: f64) -> Self {
        self.prefix_weight = weight.clamp(0.0, 0.25);
        self
    }

    /// Compute similarity between two strings using the configured metric.
    fn similarity(&self, a: &str, b: &str) -> f64 {
        match self.metric {
            FuzzyMetric::Levenshtein => levenshtein_normalized(a, b),
            FuzzyMetric::JaroWinkler => jaro_winkler(a, b, self.prefix_weight),
        }
    }

    /// Search for the best matches to a query string.
    ///
    /// Returns matches sorted by score (highest first), filtered by threshold.
    pub fn search(&self, query: &str) -> Vec<FuzzyMatch> {
        let query_lower = query.to_lowercase();
        let mut matches: Vec<FuzzyMatch> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                let item_lower = item.to_lowercase();
                let score = self.similarity(&query_lower, &item_lower);
                if score >= self.threshold {
                    Some(FuzzyMatch {
                        text: item.clone(),
                        score,
                        index,
                    })
                } else {
                    None
                }
            })
            .collect();

        matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        matches
    }

    /// Search and return only the top N matches.
    pub fn search_top_n(&self, query: &str, n: usize) -> Vec<FuzzyMatch> {
        let mut results = self.search(query);
        results.truncate(n);
        results
    }

    /// Find the single best match.
    pub fn best_match(&self, query: &str) -> Option<FuzzyMatch> {
        self.search(query).into_iter().next()
    }

    /// Add an item to the search collection.
    pub fn add_item(&mut self, item: String) {
        self.items.push(item);
    }

    /// Get the number of items in the collection.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Convenience function: find fuzzy matches for a query in a list of candidates.
///
/// Uses Levenshtein distance with the given threshold.
pub fn fuzzy_search(query: &str, candidates: &[&str], threshold: f64) -> Vec<FuzzyMatch> {
    let items: Vec<String> = candidates.iter().map(|s| s.to_string()).collect();
    FuzzySearcher::new(items)
        .with_threshold(threshold)
        .search(query)
}

/// Extract the best match from candidates (similar to `fuzzywuzzy.extractOne`).
pub fn extract_one(query: &str, candidates: &[&str]) -> Option<FuzzyMatch> {
    let items: Vec<String> = candidates.iter().map(|s| s.to_string()).collect();
    FuzzySearcher::new(items)
        .with_threshold(0.0)
        .best_match(query)
}

/// Extract top N matches from candidates (similar to `fuzzywuzzy.extract`).
pub fn extract(query: &str, candidates: &[&str], limit: usize) -> Vec<FuzzyMatch> {
    let items: Vec<String> = candidates.iter().map(|s| s.to_string()).collect();
    FuzzySearcher::new(items)
        .with_threshold(0.0)
        .search_top_n(query, limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_search_basic() {
        let candidates = vec!["apple", "application", "banana", "apartment"];
        let results = fuzzy_search("app", &candidates, 0.3);
        assert!(!results.is_empty());
        // "apple" and "application" should rank higher than "banana"
        assert!(results[0].text == "apple" || results[0].text == "application");
    }

    #[test]
    fn test_extract_one() {
        let candidates = vec!["Los Angeles", "New York", "San Francisco"];
        let result = extract_one("los angeles", &candidates).unwrap();
        assert_eq!(result.text, "Los Angeles");
    }

    #[test]
    fn test_extract_top_n() {
        let candidates = vec!["apple", "application", "apply", "banana"];
        let results = extract("app", &candidates, 3);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_fuzzy_searcher_jaro_winkler() {
        let items = vec![
            "martha".to_string(),
            "marhta".to_string(),
            "xyz".to_string(),
        ];
        let searcher = FuzzySearcher::new(items)
            .with_metric(FuzzyMetric::JaroWinkler)
            .with_threshold(0.8);
        let results = searcher.search("martha");
        assert!(results.len() >= 1);
        assert_eq!(results[0].text, "martha");
    }

    #[test]
    fn test_empty_searcher() {
        let searcher = FuzzySearcher::new(vec![]);
        assert!(searcher.is_empty());
        assert!(searcher.search("test").is_empty());
    }
}
