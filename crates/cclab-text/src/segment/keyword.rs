//! Keyword extraction (TF-IDF, TextRank).

use super::segmenter::{JiebaSegmenter, TokenizeMode};
use std::collections::{HashMap, HashSet};

/// A keyword with its weight.
#[derive(Debug, Clone, PartialEq)]
pub struct Keyword {
    pub word: String,
    pub weight: f64,
}

impl Keyword {
    pub fn new(word: impl Into<String>, weight: f64) -> Self {
        Self {
            word: word.into(),
            weight,
        }
    }
}

/// Keyword extraction algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeywordAlgorithm {
    /// TF-IDF based extraction.
    #[default]
    TfIdf,
    /// TextRank graph-based extraction.
    TextRank,
}

/// TF-IDF keyword extractor.
pub struct KeywordExtractor {
    segmenter: JiebaSegmenter,
    /// IDF values from corpus (simplified)
    idf: HashMap<String, f64>,
    /// Default IDF for unknown words
    default_idf: f64,
    /// Stop words to exclude
    stop_words: Vec<String>,
}

impl Default for KeywordExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl KeywordExtractor {
    /// Create a new keyword extractor.
    pub fn new() -> Self {
        let stop_words = vec![
            "的", "了", "在", "是", "我", "有", "和", "就", "不", "人", "都", "一", "一个", "上",
            "也", "很", "到", "说", "要", "去", "你", "会", "着", "没有", "看", "好", "自己", "这",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        // Simplified IDF values (would be computed from corpus in production)
        let mut idf = HashMap::new();
        idf.insert("清华大学".to_string(), 8.5);
        idf.insert("北京".to_string(), 5.2);
        idf.insert("大学".to_string(), 4.8);
        idf.insert("学习".to_string(), 4.5);
        idf.insert("工作".to_string(), 4.2);
        idf.insert("技术".to_string(), 5.8);
        idf.insert("系统".to_string(), 5.5);
        idf.insert("数据".to_string(), 6.2);
        idf.insert("网络".to_string(), 5.9);

        Self {
            segmenter: JiebaSegmenter::new(),
            idf,
            default_idf: 6.0,
            stop_words,
        }
    }

    /// Extract top-K keywords from text.
    pub fn extract(&self, text: &str, top_k: usize) -> Vec<Keyword> {
        let tokens = self.segmenter.tokenize(text, TokenizeMode::Precise);

        // Calculate term frequency
        let mut tf: HashMap<String, usize> = HashMap::new();
        let mut total = 0;

        for token in &tokens {
            let word = &token.word;
            // Skip single characters and stop words
            if word.chars().count() < 2 || self.stop_words.contains(word) {
                continue;
            }
            *tf.entry(word.clone()).or_insert(0) += 1;
            total += 1;
        }

        if total == 0 {
            return vec![];
        }

        // Calculate TF-IDF scores
        let mut keywords: Vec<Keyword> = tf
            .into_iter()
            .map(|(word, count)| {
                let tf_val = count as f64 / total as f64;
                let idf_val = self.idf.get(&word).copied().unwrap_or(self.default_idf);
                Keyword::new(word, tf_val * idf_val)
            })
            .collect();

        // Sort by weight descending
        keywords.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());

        // Return top K
        keywords.truncate(top_k);
        keywords
    }

    /// Add IDF value for a word.
    pub fn add_idf(&mut self, word: &str, idf: f64) {
        self.idf.insert(word.to_string(), idf);
    }

    /// Add a stop word.
    pub fn add_stop_word(&mut self, word: &str) {
        self.stop_words.push(word.to_string());
    }

    /// Extract keywords using specified algorithm.
    pub fn extract_with_algorithm(
        &self,
        text: &str,
        top_k: usize,
        algorithm: KeywordAlgorithm,
    ) -> Vec<Keyword> {
        match algorithm {
            KeywordAlgorithm::TfIdf => self.extract(text, top_k),
            KeywordAlgorithm::TextRank => self.extract_textrank(text, top_k),
        }
    }

    /// Extract keywords using TextRank algorithm.
    ///
    /// TextRank is a graph-based ranking algorithm that considers word co-occurrence.
    pub fn extract_textrank(&self, text: &str, top_k: usize) -> Vec<Keyword> {
        self.extract_textrank_with_window(text, top_k, 5)
    }

    /// Extract keywords using TextRank with custom window size.
    pub fn extract_textrank_with_window(
        &self,
        text: &str,
        top_k: usize,
        window_size: usize,
    ) -> Vec<Keyword> {
        let tokens = self.segmenter.tokenize(text, TokenizeMode::Precise);

        // Filter valid words
        let words: Vec<String> = tokens
            .iter()
            .map(|t| t.word.clone())
            .filter(|w| w.chars().count() >= 2 && !self.stop_words.contains(w))
            .collect();

        if words.is_empty() {
            return vec![];
        }

        // Build co-occurrence graph
        let mut graph: HashMap<String, HashMap<String, f64>> = HashMap::new();
        let unique_words: HashSet<String> = words.iter().cloned().collect();

        for word in &unique_words {
            graph.insert(word.clone(), HashMap::new());
        }

        // Add edges based on co-occurrence within window
        for i in 0..words.len() {
            for j in (i + 1)..std::cmp::min(i + window_size, words.len()) {
                let w1 = &words[i];
                let w2 = &words[j];
                if w1 != w2 {
                    *graph.get_mut(w1).unwrap().entry(w2.clone()).or_insert(0.0) += 1.0;
                    *graph.get_mut(w2).unwrap().entry(w1.clone()).or_insert(0.0) += 1.0;
                }
            }
        }

        // Run TextRank iterations
        let damping = 0.85;
        let max_iter = 50;
        let min_diff = 0.0001;

        let mut scores: HashMap<String, f64> =
            unique_words.iter().map(|w| (w.clone(), 1.0)).collect();

        for _ in 0..max_iter {
            let mut new_scores: HashMap<String, f64> = HashMap::new();
            let mut max_diff: f64 = 0.0;

            for word in &unique_words {
                let neighbors = graph.get(word).unwrap();
                let mut sum = 0.0;

                for (neighbor, weight) in neighbors {
                    let neighbor_neighbors = graph.get(neighbor).unwrap();
                    let out_sum: f64 = neighbor_neighbors.values().sum();
                    if out_sum > 0.0 {
                        sum += weight * scores.get(neighbor).unwrap_or(&1.0) / out_sum;
                    }
                }

                let new_score = (1.0 - damping) + damping * sum;
                let old_score = *scores.get(word).unwrap_or(&1.0);
                max_diff = max_diff.max((new_score - old_score).abs());
                new_scores.insert(word.clone(), new_score);
            }

            scores = new_scores;

            if max_diff < min_diff {
                break;
            }
        }

        // Sort and return top-k
        let mut keywords: Vec<Keyword> = scores
            .into_iter()
            .map(|(word, weight)| Keyword::new(word, weight))
            .collect();

        keywords.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
        keywords.truncate(top_k);
        keywords
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_extraction() {
        let extractor = KeywordExtractor::new();
        let keywords = extractor.extract("我来到北京清华大学", 2);

        // Should extract some keywords
        assert!(!keywords.is_empty());

        // Keywords should have positive weights
        for kw in &keywords {
            assert!(kw.weight > 0.0);
        }
    }

    #[test]
    fn test_keyword_with_weights() {
        let extractor = KeywordExtractor::new();
        let keywords = extractor.extract("清华大学是北京著名大学", 3);

        // Check that 清华大学 has high weight (high IDF)
        let qinghua = keywords.iter().find(|k| k.word == "清华大学");
        assert!(qinghua.is_some());
    }

    #[test]
    fn test_stop_words_filtered() {
        let extractor = KeywordExtractor::new();
        let keywords = extractor.extract("我的北京之旅", 5);

        // Stop words like "的" should not appear
        let has_stop = keywords.iter().any(|k| k.word == "的");
        assert!(!has_stop);
    }

    #[test]
    fn test_textrank_extraction() {
        let extractor = KeywordExtractor::new();
        let text = "清华大学是北京著名的大学，位于北京市海淀区";
        let keywords = extractor.extract_textrank(text, 3);

        // Should extract some keywords
        assert!(!keywords.is_empty());

        // Keywords should have positive weights
        for kw in &keywords {
            assert!(kw.weight > 0.0);
        }
    }

    #[test]
    fn test_textrank_vs_tfidf() {
        let extractor = KeywordExtractor::new();
        let text = "清华大学和北京大学都是著名大学";

        let tfidf_kw = extractor.extract_with_algorithm(text, 3, KeywordAlgorithm::TfIdf);
        let textrank_kw = extractor.extract_with_algorithm(text, 3, KeywordAlgorithm::TextRank);

        // Both should return results
        assert!(!tfidf_kw.is_empty());
        assert!(!textrank_kw.is_empty());
    }

    #[test]
    fn test_add_idf() {
        let mut extractor = KeywordExtractor::new();
        // Add a high IDF score for "北京"
        extractor.add_idf("北京", 100.0);
        let keywords = extractor.extract("我来到北京清华大学", 5);
        // "北京" should appear with high weight
        let beijing = keywords.iter().find(|k| k.word == "北京");
        assert!(beijing.is_some());
    }

    #[test]
    fn test_add_stop_word() {
        let mut extractor = KeywordExtractor::new();
        extractor.add_stop_word("北京");
        let keywords = extractor.extract("北京大学在北京", 5);
        // "北京" should be filtered as stop word
        let has_beijing = keywords.iter().any(|k| k.word == "北京");
        assert!(!has_beijing);
    }

    #[test]
    fn test_extract_textrank_with_window() {
        let extractor = KeywordExtractor::new();
        let text = "清华大学是北京著名的大学，位于北京市海淀区";
        let keywords = extractor.extract_textrank_with_window(text, 3, 3);
        assert!(!keywords.is_empty());
    }
}
