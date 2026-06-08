//! TF-IDF implementation.

use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// TF-IDF (Term Frequency-Inverse Document Frequency) ranking.
///
/// A simpler alternative to BM25 that computes:
/// ```text
/// score(D, Q) = Σ TF(term, D) * IDF(term)
/// ```
///
/// Where:
/// - `TF(term, D)` = term frequency in document D
/// - `IDF(term)` = log(N / df(term)) where N is corpus size, df is document frequency
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TfIdf {
    /// Number of documents in corpus
    corpus_size: usize,
    /// IDF values for each term
    idf: HashMap<String, f64>,
    /// Term frequencies per document: doc_idx -> term -> count
    doc_freqs: Vec<HashMap<String, usize>>,
    /// Whether to use log-normalized TF
    use_log_tf: bool,
}

impl TfIdf {
    /// Create a new TF-IDF instance from a tokenized corpus.
    pub fn new(corpus: Vec<Vec<String>>) -> Self {
        Self::with_options(corpus, false)
    }

    /// Create a TF-IDF instance with log-normalized TF.
    ///
    /// Log-normalized TF: 1 + log(tf) if tf > 0, else 0
    pub fn with_log_tf(corpus: Vec<Vec<String>>) -> Self {
        Self::with_options(corpus, true)
    }

    /// Create a TF-IDF instance with options.
    fn with_options(corpus: Vec<Vec<String>>, use_log_tf: bool) -> Self {
        let corpus_size = corpus.len();
        if corpus_size == 0 {
            return Self {
                corpus_size: 0,
                idf: HashMap::new(),
                doc_freqs: vec![],
                use_log_tf,
            };
        }

        // Calculate term frequencies and document frequencies
        let mut doc_freqs = Vec::with_capacity(corpus_size);
        let mut df: HashMap<String, usize> = HashMap::new();

        for doc in &corpus {
            let mut tf: HashMap<String, usize> = HashMap::new();
            for term in doc {
                *tf.entry(term.clone()).or_insert(0) += 1;
            }

            for term in tf.keys() {
                *df.entry(term.clone()).or_insert(0) += 1;
            }

            doc_freqs.push(tf);
        }

        // Calculate IDF: log(N / df(term))
        let mut idf = HashMap::new();
        let n = corpus_size as f64;
        for (term, doc_freq) in df {
            let idf_val = (n / doc_freq as f64).ln();
            idf.insert(term, idf_val);
        }

        Self {
            corpus_size,
            idf,
            doc_freqs,
            use_log_tf,
        }
    }

    /// Get TF-IDF scores for all documents given a query.
    pub fn get_scores(&self, query: &[String]) -> Vec<f64> {
        if self.corpus_size == 0 {
            return vec![];
        }

        let mut scores = vec![0.0; self.corpus_size];

        for (doc_idx, doc_tf) in self.doc_freqs.iter().enumerate() {
            for term in query {
                if let Some(&idf) = self.idf.get(term) {
                    let raw_tf = *doc_tf.get(term).unwrap_or(&0) as f64;
                    if raw_tf > 0.0 {
                        let tf = if self.use_log_tf {
                            1.0 + raw_tf.ln()
                        } else {
                            raw_tf
                        };
                        scores[doc_idx] += tf * idf;
                    }
                }
            }
        }

        scores
    }

    /// Get the top N document indices for a query.
    pub fn get_top_n_indices(&self, query: &[String], n: usize) -> Vec<usize> {
        let scores = self.get_scores(query);

        let mut indexed_scores: Vec<(usize, f64)> = scores.into_iter().enumerate().collect();
        indexed_scores.sort_by(|a, b| b.1.total_cmp(&a.1));

        indexed_scores
            .into_iter()
            .take(n)
            .map(|(idx, _)| idx)
            .collect()
    }

    /// Get the top N documents from a list.
    ///
    /// # Panics
    /// Panics if `documents.len() != corpus_size`.
    pub fn get_top_n<T: Clone>(&self, query: &[String], documents: &[T], n: usize) -> Vec<T> {
        assert_eq!(
            documents.len(),
            self.corpus_size,
            "documents length ({}) must match corpus size ({})",
            documents.len(),
            self.corpus_size
        );

        self.get_top_n_indices(query, n)
            .into_iter()
            .filter_map(|idx| documents.get(idx).cloned())
            .collect()
    }

    /// Get the top N documents with their scores.
    pub fn get_top_n_with_scores<T: Clone>(
        &self,
        query: &[String],
        documents: &[T],
        n: usize,
    ) -> Vec<(T, f64)> {
        assert_eq!(
            documents.len(),
            self.corpus_size,
            "documents length ({}) must match corpus size ({})",
            documents.len(),
            self.corpus_size
        );

        let scores = self.get_scores(query);

        let mut indexed: Vec<(usize, f64)> = scores.into_iter().enumerate().collect();
        indexed.sort_by(|a, b| b.1.total_cmp(&a.1));

        indexed
            .into_iter()
            .take(n)
            .filter_map(|(idx, score)| documents.get(idx).cloned().map(|doc| (doc, score)))
            .collect()
    }

    /// Get corpus size.
    pub fn corpus_size(&self) -> usize {
        self.corpus_size
    }

    /// Get IDF value for a term.
    pub fn get_idf(&self, term: &str) -> Option<f64> {
        self.idf.get(term).copied()
    }

    // === Incremental Updates ===

    /// Add a document to the index.
    ///
    /// This updates the corpus statistics and IDF values.
    pub fn add_document(&mut self, doc: Vec<String>) {
        // Calculate term frequencies for new document
        let mut tf: HashMap<String, usize> = HashMap::new();
        for term in &doc {
            *tf.entry(term.clone()).or_insert(0) += 1;
        }

        // Update document frequency for new terms
        let new_terms: Vec<String> = tf.keys().cloned().collect();

        self.doc_freqs.push(tf);
        self.corpus_size += 1;

        // Recalculate IDF for affected terms
        self.recalculate_idf(&new_terms);
    }

    /// Remove a document from the index by index.
    ///
    /// Returns true if the document was removed, false if index out of bounds.
    pub fn remove_document(&mut self, doc_idx: usize) -> bool {
        if doc_idx >= self.corpus_size {
            return false;
        }

        // Get terms in the removed document
        let removed_terms: Vec<String> = self.doc_freqs[doc_idx].keys().cloned().collect();

        // Remove document
        self.doc_freqs.remove(doc_idx);
        self.corpus_size -= 1;

        if self.corpus_size == 0 {
            self.idf.clear();
        } else {
            // Recalculate IDF for affected terms
            self.recalculate_idf(&removed_terms);
        }

        true
    }

    /// Recalculate IDF values for specific terms.
    fn recalculate_idf(&mut self, terms: &[String]) {
        if self.corpus_size == 0 {
            return;
        }

        let n = self.corpus_size as f64;

        for term in terms {
            // Count documents containing this term
            let df: usize = self
                .doc_freqs
                .iter()
                .filter(|doc_tf| doc_tf.contains_key(term))
                .count();

            if df == 0 {
                self.idf.remove(term);
            } else {
                let idf_val = (n / df as f64).ln();
                self.idf.insert(term.clone(), idf_val);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_corpus() -> Vec<Vec<String>> {
        vec![
            vec!["hello".into(), "world".into()],
            vec!["hello".into(), "rust".into()],
            vec!["rust".into(), "programming".into(), "language".into()],
        ]
    }

    #[test]
    fn test_tfidf_creation() {
        let corpus = create_test_corpus();
        let tfidf = TfIdf::new(corpus);

        assert_eq!(tfidf.corpus_size(), 3);
    }

    #[test]
    fn test_tfidf_get_scores() {
        let corpus = create_test_corpus();
        let tfidf = TfIdf::new(corpus);

        let query = vec!["rust".into()];
        let scores = tfidf.get_scores(&query);

        assert_eq!(scores.len(), 3);
        assert_eq!(scores[0], 0.0); // No "rust" in doc 0
        assert!(scores[1] > 0.0); // Has "rust"
        assert!(scores[2] > 0.0); // Has "rust"
    }

    #[test]
    fn test_tfidf_idf_ranking() {
        let corpus = create_test_corpus();
        let tfidf = TfIdf::new(corpus);

        // "programming" appears in 1 doc, "hello" in 2 docs
        let idf_prog = tfidf.get_idf("programming").unwrap();
        let idf_hello = tfidf.get_idf("hello").unwrap();

        assert!(idf_prog > idf_hello);
    }

    #[test]
    fn test_tfidf_log_tf() {
        // Need a third document so IDF isn't zero for "rust"
        let corpus = vec![
            vec!["rust".into(), "rust".into(), "rust".into()], // tf=3
            vec!["rust".into()],                               // tf=1
            vec!["other".into(), "terms".into()],              // no "rust" - makes IDF non-zero
        ];

        let tfidf_raw = TfIdf::new(corpus.clone());
        let tfidf_log = TfIdf::with_log_tf(corpus);

        let query = vec!["rust".into()];
        let scores_raw = tfidf_raw.get_scores(&query);
        let scores_log = tfidf_log.get_scores(&query);

        // Both should have non-zero scores for docs 0 and 1
        assert!(scores_raw[0] > 0.0);
        assert!(scores_raw[1] > 0.0);

        // Raw TF: doc 0 should score 3x higher than doc 1
        assert!((scores_raw[0] / scores_raw[1] - 3.0).abs() < 0.001);

        // Log TF: difference should be smaller (log(1+3) / log(1+1) = ~1.58)
        let ratio = scores_log[0] / scores_log[1];
        assert!(ratio < 3.0);
        assert!(ratio > 1.0);
    }

    #[test]
    fn test_tfidf_add_document() {
        let corpus = create_test_corpus();
        let mut tfidf = TfIdf::new(corpus);

        assert_eq!(tfidf.corpus_size(), 3);

        tfidf.add_document(vec!["new".into(), "document".into()]);

        assert_eq!(tfidf.corpus_size(), 4);
        assert!(tfidf.get_idf("new").is_some());
        assert!(tfidf.get_idf("document").is_some());
    }

    #[test]
    fn test_tfidf_remove_document() {
        let corpus = create_test_corpus();
        let mut tfidf = TfIdf::new(corpus);

        // Remove doc 2 (has "programming", "language")
        assert!(tfidf.remove_document(2));
        assert_eq!(tfidf.corpus_size(), 2);

        // "programming" and "language" should be gone
        assert!(tfidf.get_idf("programming").is_none());
        assert!(tfidf.get_idf("language").is_none());

        // "hello" and "rust" should still exist
        assert!(tfidf.get_idf("hello").is_some());
        assert!(tfidf.get_idf("rust").is_some());
    }

    #[test]
    fn test_tfidf_remove_invalid_index() {
        let corpus = create_test_corpus();
        let mut tfidf = TfIdf::new(corpus);

        assert!(!tfidf.remove_document(10));
        assert_eq!(tfidf.corpus_size(), 3);
    }

    #[test]
    fn test_tfidf_empty_corpus() {
        let tfidf = TfIdf::new(vec![]);

        assert_eq!(tfidf.corpus_size(), 0);
        let empty: Vec<f64> = vec![];
        assert_eq!(tfidf.get_scores(&["test".into()]), empty);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_tfidf_serialization() {
        let corpus = create_test_corpus();
        let tfidf = TfIdf::new(corpus);

        // Serialize to JSON
        let json = serde_json::to_string(&tfidf).unwrap();

        // Deserialize back
        let tfidf_restored: TfIdf = serde_json::from_str(&json).unwrap();

        // Verify state is preserved
        assert_eq!(tfidf.corpus_size(), tfidf_restored.corpus_size());

        // Verify scoring works the same
        let query = vec!["rust".into()];
        let scores_orig = tfidf.get_scores(&query);
        let scores_restored = tfidf_restored.get_scores(&query);
        assert_eq!(scores_orig, scores_restored);
    }

    #[test]
    fn test_get_top_n_indices() {
        let corpus = create_test_corpus();
        let tfidf = TfIdf::new(corpus);

        let query = vec!["rust".into()];
        let top_indices = tfidf.get_top_n_indices(&query, 2);

        assert_eq!(top_indices.len(), 2);
        // Documents 1 and 2 have "rust", should be top
        assert!(top_indices.contains(&1) || top_indices.contains(&2));
    }

    #[test]
    fn test_get_top_n_with_scores() {
        let corpus = create_test_corpus();
        let tfidf = TfIdf::new(corpus.clone());

        let documents: Vec<&str> = vec!["doc0", "doc1", "doc2"];
        let query = vec!["rust".into()];
        let top_with_scores = tfidf.get_top_n_with_scores(&query, &documents, 2);

        assert_eq!(top_with_scores.len(), 2);
        // Scores should be in descending order
        assert!(top_with_scores[0].1 >= top_with_scores[1].1);
    }
}
