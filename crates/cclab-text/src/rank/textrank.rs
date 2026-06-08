//! TextRank keyword extraction algorithm.
//!
//! Implements the TextRank algorithm for unsupervised keyword extraction,
//! based on the PageRank graph-based ranking model.

use std::collections::{HashMap, HashSet};

/// A keyword extracted by TextRank with its score.
#[derive(Debug, Clone)]
pub struct Keyword {
    /// The keyword text.
    pub word: String,
    /// The TextRank score (higher = more important).
    pub score: f64,
}

/// TextRank keyword extractor.
///
/// Uses a graph-based approach where words are nodes and edges represent
/// co-occurrence within a sliding window.
#[derive(Debug, Clone)]
pub struct TextRank {
    /// Damping factor (probability of following a link vs random jump).
    damping: f64,
    /// Number of iterations for convergence.
    max_iterations: usize,
    /// Convergence threshold.
    convergence_threshold: f64,
    /// Co-occurrence window size.
    window_size: usize,
}

impl Default for TextRank {
    fn default() -> Self {
        Self {
            damping: 0.85,
            max_iterations: 100,
            convergence_threshold: 1e-6,
            window_size: 4,
        }
    }
}

impl TextRank {
    /// Create a new TextRank extractor with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the damping factor (0.0 to 1.0, default: 0.85).
    pub fn with_damping(mut self, damping: f64) -> Self {
        self.damping = damping.clamp(0.0, 1.0);
        self
    }

    /// Set the maximum iterations (default: 100).
    pub fn with_max_iterations(mut self, max_iter: usize) -> Self {
        self.max_iterations = max_iter;
        self
    }

    /// Set the convergence threshold (default: 1e-6).
    pub fn with_convergence_threshold(mut self, threshold: f64) -> Self {
        self.convergence_threshold = threshold;
        self
    }

    /// Set the co-occurrence window size (default: 4).
    pub fn with_window_size(mut self, size: usize) -> Self {
        self.window_size = size.max(2);
        self
    }

    /// Extract keywords from tokenized text.
    ///
    /// Returns keywords sorted by score (highest first).
    pub fn extract_keywords(&self, tokens: &[String], top_n: usize) -> Vec<Keyword> {
        if tokens.is_empty() {
            return Vec::new();
        }

        // Build co-occurrence graph
        let (graph, words) = self.build_graph(tokens);

        if words.is_empty() {
            return Vec::new();
        }

        // Run PageRank
        let scores = self.pagerank(&graph, words.len());

        // Collect results
        let mut keywords: Vec<Keyword> = words
            .into_iter()
            .enumerate()
            .map(|(i, word)| Keyword {
                word,
                score: scores[i],
            })
            .collect();

        keywords.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        keywords.truncate(top_n);
        keywords
    }

    /// Build the co-occurrence graph from tokens.
    ///
    /// Returns adjacency matrix and list of unique words.
    fn build_graph(&self, tokens: &[String]) -> (Vec<Vec<f64>>, Vec<String>) {
        // Collect unique words and assign indices
        let unique: Vec<String> = {
            let mut seen = HashSet::new();
            tokens
                .iter()
                .filter(|t| seen.insert(t.as_str()))
                .cloned()
                .collect()
        };

        let word_to_idx: HashMap<&str, usize> = unique
            .iter()
            .enumerate()
            .map(|(i, w)| (w.as_str(), i))
            .collect();

        let n = unique.len();
        let mut graph = vec![vec![0.0f64; n]; n];

        // Build co-occurrence edges using sliding window
        for i in 0..tokens.len() {
            let end = (i + self.window_size).min(tokens.len());
            for j in (i + 1)..end {
                if let (Some(&idx_i), Some(&idx_j)) = (
                    word_to_idx.get(tokens[i].as_str()),
                    word_to_idx.get(tokens[j].as_str()),
                ) {
                    if idx_i != idx_j {
                        graph[idx_i][idx_j] += 1.0;
                        graph[idx_j][idx_i] += 1.0;
                    }
                }
            }
        }

        (graph, unique)
    }

    /// Run the PageRank algorithm on the adjacency matrix.
    fn pagerank(&self, graph: &[Vec<f64>], n: usize) -> Vec<f64> {
        let mut scores = vec![1.0 / n as f64; n];

        // Precompute out-degree for each node
        let out_degrees: Vec<f64> = graph.iter().map(|row| row.iter().sum::<f64>()).collect();

        for _ in 0..self.max_iterations {
            let mut new_scores = vec![0.0f64; n];
            let mut max_diff = 0.0f64;

            for i in 0..n {
                let mut rank_sum = 0.0;
                for j in 0..n {
                    if graph[j][i] > 0.0 && out_degrees[j] > 0.0 {
                        rank_sum += graph[j][i] * scores[j] / out_degrees[j];
                    }
                }
                new_scores[i] = (1.0 - self.damping) / n as f64 + self.damping * rank_sum;
                max_diff = max_diff.max((new_scores[i] - scores[i]).abs());
            }

            scores = new_scores;

            if max_diff < self.convergence_threshold {
                break;
            }
        }

        scores
    }
}

/// Convenience function: extract top N keywords from tokenized text.
pub fn extract_keywords(tokens: &[String], top_n: usize) -> Vec<Keyword> {
    TextRank::new().extract_keywords(tokens, top_n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tokens() -> Vec<String> {
        "the quick brown fox jumps over the lazy dog the fox is quick"
            .split_whitespace()
            .map(String::from)
            .collect()
    }

    #[test]
    fn test_extract_keywords() {
        let tokens = sample_tokens();
        let keywords = extract_keywords(&tokens, 3);
        assert_eq!(keywords.len(), 3);
        // All scores should be positive
        for kw in &keywords {
            assert!(kw.score > 0.0);
        }
    }

    #[test]
    fn test_keyword_ordering() {
        let tokens = sample_tokens();
        let keywords = extract_keywords(&tokens, 5);
        // Should be sorted by score descending
        for i in 1..keywords.len() {
            assert!(keywords[i - 1].score >= keywords[i].score);
        }
    }

    #[test]
    fn test_empty_input() {
        let keywords = extract_keywords(&[], 5);
        assert!(keywords.is_empty());
    }

    #[test]
    fn test_custom_params() {
        let tokens = sample_tokens();
        let tr = TextRank::new()
            .with_damping(0.9)
            .with_window_size(3)
            .with_max_iterations(50);
        let keywords = tr.extract_keywords(&tokens, 3);
        assert!(!keywords.is_empty());
    }
}
