//! BM25Okapi implementation.

use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// BM25Okapi ranking algorithm.
///
/// This implementation follows the Okapi BM25 formula:
/// ```text
/// score(D, Q) = Σ IDF(qi) * (f(qi, D) * (k1 + 1)) / (f(qi, D) + k1 * (1 - b + b * |D| / avgdl))
/// ```
///
/// Where:
/// - `f(qi, D)` is the term frequency of query term qi in document D
/// - `|D|` is the document length
/// - `avgdl` is the average document length in the corpus
/// - `k1` and `b` are tuning parameters (defaults: k1=1.5, b=0.75)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BM25Okapi {
    /// Number of documents in corpus
    corpus_size: usize,
    /// Average document length
    avgdl: f64,
    /// IDF values for each term
    idf: HashMap<String, f64>,
    /// Document lengths
    doc_lengths: Vec<usize>,
    /// Term frequencies per document: doc_idx -> term -> count
    doc_freqs: Vec<HashMap<String, usize>>,
    /// BM25 parameter k1 (term frequency saturation)
    k1: f64,
    /// BM25 parameter b (document length normalization)
    b: f64,
}

impl BM25Okapi {
    /// Create a new BM25Okapi instance from a tokenized corpus.
    ///
    /// Each document should be pre-tokenized as a Vec<String>.
    pub fn new(corpus: Vec<Vec<String>>) -> Self {
        Self::with_params(corpus, 1.5, 0.75)
    }

    /// Create a BM25Okapi instance with custom k1 and b parameters.
    ///
    /// - `k1`: Term frequency saturation parameter (typically 1.2-2.0)
    /// - `b`: Document length normalization (0.0-1.0, typically 0.75)
    pub fn with_params(corpus: Vec<Vec<String>>, k1: f64, b: f64) -> Self {
        let corpus_size = corpus.len();
        if corpus_size == 0 {
            return Self {
                corpus_size: 0,
                avgdl: 0.0,
                idf: HashMap::new(),
                doc_lengths: vec![],
                doc_freqs: vec![],
                k1,
                b,
            };
        }

        // Calculate document lengths and term frequencies
        let mut doc_lengths = Vec::with_capacity(corpus_size);
        let mut doc_freqs = Vec::with_capacity(corpus_size);
        let mut df: HashMap<String, usize> = HashMap::new(); // Document frequency

        for doc in &corpus {
            let doc_len = doc.len();
            doc_lengths.push(doc_len);

            // Count term frequencies in this document
            let mut tf: HashMap<String, usize> = HashMap::new();
            for term in doc {
                *tf.entry(term.clone()).or_insert(0) += 1;
            }

            // Update document frequency (how many docs contain each term)
            for term in tf.keys() {
                *df.entry(term.clone()).or_insert(0) += 1;
            }

            doc_freqs.push(tf);
        }

        // Calculate average document length
        let total_len: usize = doc_lengths.iter().sum();
        let avgdl = total_len as f64 / corpus_size as f64;

        // Calculate IDF for each term
        // Using rank-bm25's formula: IDF(t) = ln((N - df(t) + 0.5) / (df(t) + 0.5))
        // With epsilon floor for negative values (when df > N/2)
        let mut idf = HashMap::new();
        let n = corpus_size as f64;
        let epsilon = 0.25; // rank-bm25 default epsilon
        for (term, doc_freq) in df {
            let df_t = doc_freq as f64;
            let idf_val = ((n - df_t + 0.5) / (df_t + 0.5)).ln();
            // Apply epsilon floor for negative IDF (very common terms)
            idf.insert(term, if idf_val < 0.0 { epsilon } else { idf_val });
        }

        Self {
            corpus_size,
            avgdl,
            idf,
            doc_lengths,
            doc_freqs,
            k1,
            b,
        }
    }

    /// Get BM25 scores for all documents given a query.
    ///
    /// Returns a vector of scores, one per document in corpus order.
    pub fn get_scores(&self, query: &[String]) -> Vec<f64> {
        if self.corpus_size == 0 {
            return vec![];
        }

        let mut scores = vec![0.0; self.corpus_size];

        // Guard against division by zero when avgdl is 0 (all empty docs)
        if self.avgdl == 0.0 {
            return scores;
        }

        for (doc_idx, doc_tf) in self.doc_freqs.iter().enumerate() {
            let doc_len = self.doc_lengths[doc_idx] as f64;

            for term in query {
                if let Some(&idf) = self.idf.get(term) {
                    let tf = *doc_tf.get(term).unwrap_or(&0) as f64;
                    if tf > 0.0 {
                        // BM25 scoring formula
                        let numerator = tf * (self.k1 + 1.0);
                        let denominator =
                            tf + self.k1 * (1.0 - self.b + self.b * doc_len / self.avgdl);
                        scores[doc_idx] += idf * numerator / denominator;
                    }
                }
            }
        }

        scores
    }

    /// Get the top N documents for a query.
    ///
    /// Returns indices of documents sorted by score (highest first).
    pub fn get_top_n_indices(&self, query: &[String], n: usize) -> Vec<usize> {
        let scores = self.get_scores(query);

        let mut indexed_scores: Vec<(usize, f64)> = scores.into_iter().enumerate().collect();
        // Use total_cmp for stable sorting
        indexed_scores.sort_by(|a, b| b.1.total_cmp(&a.1));

        indexed_scores
            .into_iter()
            .take(n)
            .map(|(idx, _)| idx)
            .collect()
    }

    /// Get the top N documents from a list of documents.
    ///
    /// This is the rank-bm25 compatible API.
    ///
    /// # Panics
    /// Panics if `documents.len() != corpus_size` (rank-bm25 compatibility).
    pub fn get_top_n<T: Clone>(&self, query: &[String], documents: &[T], n: usize) -> Vec<T> {
        assert_eq!(
            documents.len(),
            self.corpus_size,
            "documents length ({}) must match corpus size ({})",
            documents.len(),
            self.corpus_size
        );

        let indices = self.get_top_n_indices(query, n);

        indices
            .into_iter()
            .filter_map(|idx| documents.get(idx).cloned())
            .collect()
    }

    /// Get the top N documents with their scores.
    ///
    /// # Panics
    /// Panics if `documents.len() != corpus_size` (rank-bm25 compatibility).
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
        // Use total_cmp for stable sorting with potential NaN (though we guard against it now)
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

    /// Get average document length.
    pub fn avgdl(&self) -> f64 {
        self.avgdl
    }

    /// Get IDF value for a term.
    pub fn get_idf(&self, term: &str) -> Option<f64> {
        self.idf.get(term).copied()
    }

    /// Get document lengths for all documents.
    pub fn doc_lengths(&self) -> &[usize] {
        &self.doc_lengths
    }

    /// Get document length for a specific document.
    pub fn get_doc_len(&self, doc_idx: usize) -> Option<usize> {
        self.doc_lengths.get(doc_idx).copied()
    }

    // === Incremental Updates ===

    /// Add a document to the index.
    ///
    /// This updates the corpus statistics, IDF values, and avgdl.
    pub fn add_document(&mut self, doc: Vec<String>) {
        let doc_len = doc.len();

        // Calculate term frequencies for new document
        let mut tf: HashMap<String, usize> = HashMap::new();
        for term in &doc {
            *tf.entry(term.clone()).or_insert(0) += 1;
        }

        // Track which terms need IDF recalculation
        let new_terms: Vec<String> = tf.keys().cloned().collect();

        // Update corpus state
        self.doc_lengths.push(doc_len);
        self.doc_freqs.push(tf);
        self.corpus_size += 1;

        // Recalculate avgdl
        let total_len: usize = self.doc_lengths.iter().sum();
        self.avgdl = total_len as f64 / self.corpus_size as f64;

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

        // Remove document data
        self.doc_lengths.remove(doc_idx);
        self.doc_freqs.remove(doc_idx);
        self.corpus_size -= 1;

        if self.corpus_size == 0 {
            self.avgdl = 0.0;
            self.idf.clear();
        } else {
            // Recalculate avgdl
            let total_len: usize = self.doc_lengths.iter().sum();
            self.avgdl = total_len as f64 / self.corpus_size as f64;

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
        let epsilon = 0.25;

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
                let df_t = df as f64;
                let idf_val = ((n - df_t + 0.5) / (df_t + 0.5)).ln();
                self.idf
                    .insert(term.clone(), if idf_val < 0.0 { epsilon } else { idf_val });
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
            vec!["hello".into(), "rust".into(), "world".into()],
        ]
    }

    #[test]
    fn test_bm25_creation() {
        let corpus = create_test_corpus();
        let bm25 = BM25Okapi::new(corpus);

        assert_eq!(bm25.corpus_size(), 4);
        assert!(bm25.avgdl() > 0.0);
    }

    #[test]
    fn test_bm25_get_scores() {
        let corpus = create_test_corpus();
        let bm25 = BM25Okapi::new(corpus);

        let query = vec!["rust".into()];
        let scores = bm25.get_scores(&query);

        assert_eq!(scores.len(), 4);
        // Doc 0 has no "rust", should score 0
        assert_eq!(scores[0], 0.0);
        // Doc 1, 2, 3 have "rust", should score > 0
        assert!(scores[1] > 0.0);
        assert!(scores[2] > 0.0);
        assert!(scores[3] > 0.0);
    }

    #[test]
    fn test_bm25_get_top_n() {
        let corpus = create_test_corpus();
        let documents = vec!["doc0", "doc1", "doc2", "doc3"];
        let bm25 = BM25Okapi::new(corpus);

        let query = vec!["rust".into()];
        let top = bm25.get_top_n(&query, &documents, 2);

        assert_eq!(top.len(), 2);
        // Top results should be from docs with "rust"
        assert!(top.contains(&"doc1") || top.contains(&"doc2") || top.contains(&"doc3"));
    }

    #[test]
    fn test_bm25_empty_corpus() {
        let bm25 = BM25Okapi::new(vec![]);

        assert_eq!(bm25.corpus_size(), 0);
        let empty: Vec<f64> = vec![];
        assert_eq!(bm25.get_scores(&["test".into()]), empty);
    }

    #[test]
    fn test_bm25_idf() {
        let corpus = create_test_corpus();
        let bm25 = BM25Okapi::new(corpus);

        // "programming" appears in only 1 doc, should have higher IDF
        // "hello" appears in 3 docs, should have lower IDF
        let idf_programming = bm25.get_idf("programming").unwrap();
        let idf_hello = bm25.get_idf("hello").unwrap();

        assert!(idf_programming > idf_hello);
    }

    #[test]
    fn test_bm25_multi_term_query() {
        let corpus = create_test_corpus();
        let bm25 = BM25Okapi::new(corpus);

        let query = vec!["hello".into(), "rust".into()];
        let scores = bm25.get_scores(&query);

        // Doc 3 has both "hello" and "rust", should score highest
        let max_idx = scores
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap();

        // Doc 1 and 3 both have hello+rust, but doc 3 also has world
        // The exact winner depends on length normalization
        assert!(max_idx == 1 || max_idx == 3);
    }

    #[test]
    fn test_bm25_with_custom_params() {
        let corpus = create_test_corpus();
        let bm25_default = BM25Okapi::new(corpus.clone());
        let bm25_custom = BM25Okapi::with_params(corpus, 2.0, 0.5);

        let query = vec!["rust".into()];
        let scores_default = bm25_default.get_scores(&query);
        let scores_custom = bm25_custom.get_scores(&query);

        // Scores should differ with different parameters
        assert_ne!(scores_default, scores_custom);
    }

    #[test]
    fn test_bm25_all_empty_documents() {
        // Corpus with only empty documents
        let corpus: Vec<Vec<String>> = vec![vec![], vec![], vec![]];
        let bm25 = BM25Okapi::new(corpus);

        assert_eq!(bm25.corpus_size(), 3);
        assert_eq!(bm25.avgdl(), 0.0);

        // Should not panic, should return all zeros
        let scores = bm25.get_scores(&["test".into()]);
        assert_eq!(scores, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_bm25_doc_lengths() {
        let corpus = create_test_corpus();
        let bm25 = BM25Okapi::new(corpus);

        assert_eq!(bm25.doc_lengths(), &[2, 2, 3, 3]);
        assert_eq!(bm25.get_doc_len(0), Some(2));
        assert_eq!(bm25.get_doc_len(3), Some(3));
        assert_eq!(bm25.get_doc_len(10), None);
    }

    #[test]
    #[should_panic(expected = "documents length")]
    fn test_bm25_get_top_n_length_mismatch() {
        let corpus = create_test_corpus();
        let bm25 = BM25Okapi::new(corpus);

        // Should panic: only 2 documents but corpus has 4
        let documents = vec!["doc0", "doc1"];
        let _ = bm25.get_top_n(&["rust".into()], &documents, 2);
    }

    #[test]
    fn test_bm25_add_document() {
        let corpus = create_test_corpus();
        let mut bm25 = BM25Okapi::new(corpus);

        assert_eq!(bm25.corpus_size(), 4);
        let old_avgdl = bm25.avgdl();

        bm25.add_document(vec!["new".into(), "document".into(), "here".into()]);

        assert_eq!(bm25.corpus_size(), 5);
        assert!(bm25.get_idf("new").is_some());
        assert!(bm25.get_idf("document").is_some());
        // avgdl should have changed
        assert_ne!(bm25.avgdl(), old_avgdl);
    }

    #[test]
    fn test_bm25_remove_document() {
        let corpus = create_test_corpus();
        let mut bm25 = BM25Okapi::new(corpus);

        // Remove doc 2 (has "programming", "language")
        assert!(bm25.remove_document(2));
        assert_eq!(bm25.corpus_size(), 3);

        // "programming" and "language" should be gone
        assert!(bm25.get_idf("programming").is_none());
        assert!(bm25.get_idf("language").is_none());

        // Other terms should still exist
        assert!(bm25.get_idf("hello").is_some());
        assert!(bm25.get_idf("rust").is_some());
    }

    #[test]
    fn test_bm25_remove_invalid_index() {
        let corpus = create_test_corpus();
        let mut bm25 = BM25Okapi::new(corpus);

        assert!(!bm25.remove_document(10));
        assert_eq!(bm25.corpus_size(), 4);
    }

    #[test]
    fn test_bm25_incremental_consistency() {
        // Test that incremental updates produce consistent results
        let corpus = vec![
            vec!["hello".into(), "world".into()],
            vec!["hello".into(), "rust".into()],
        ];

        let mut bm25 = BM25Okapi::new(corpus.clone());
        bm25.add_document(vec!["rust".into(), "programming".into()]);

        // Create fresh index with all 3 docs
        let full_corpus = vec![
            vec!["hello".into(), "world".into()],
            vec!["hello".into(), "rust".into()],
            vec!["rust".into(), "programming".into()],
        ];
        let bm25_full = BM25Okapi::new(full_corpus);

        // Both should have same corpus size and avgdl
        assert_eq!(bm25.corpus_size(), bm25_full.corpus_size());
        assert!((bm25.avgdl() - bm25_full.avgdl()).abs() < 0.001);

        // IDF values should match
        assert!((bm25.get_idf("rust").unwrap() - bm25_full.get_idf("rust").unwrap()).abs() < 0.001);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_bm25_serialization() {
        let corpus = create_test_corpus();
        let bm25 = BM25Okapi::new(corpus);

        // Serialize to JSON
        let json = serde_json::to_string(&bm25).unwrap();

        // Deserialize back
        let bm25_restored: BM25Okapi = serde_json::from_str(&json).unwrap();

        // Verify state is preserved
        assert_eq!(bm25.corpus_size(), bm25_restored.corpus_size());
        assert!((bm25.avgdl() - bm25_restored.avgdl()).abs() < 0.001);

        // Verify scoring works the same
        let query = vec!["rust".into()];
        let scores_orig = bm25.get_scores(&query);
        let scores_restored = bm25_restored.get_scores(&query);
        assert_eq!(scores_orig, scores_restored);
    }

    #[test]
    fn test_bm25_get_doc_len() {
        let corpus = create_test_corpus();
        let bm25 = BM25Okapi::new(corpus);

        // Doc 0 has 2 terms, Doc 1 has 2 terms, Doc 2 has 3 terms
        assert_eq!(bm25.get_doc_len(0), Some(2));
        assert_eq!(bm25.get_doc_len(1), Some(2));
        assert_eq!(bm25.get_doc_len(2), Some(3));
        assert_eq!(bm25.get_doc_len(10), None); // Out of bounds
    }

    #[test]
    fn test_bm25_get_top_n_with_scores() {
        let corpus = create_test_corpus();
        let bm25 = BM25Okapi::new(corpus);

        let documents = vec!["doc0", "doc1", "doc2", "doc3"];
        let query = vec!["rust".into()];
        let top_with_scores = bm25.get_top_n_with_scores(&query, &documents, 2);

        assert_eq!(top_with_scores.len(), 2);
        // Scores should be in descending order
        assert!(top_with_scores[0].1 >= top_with_scores[1].1);
    }

    #[test]
    fn test_bm25_get_top_n_indices() {
        let corpus = create_test_corpus();
        let bm25 = BM25Okapi::new(corpus);

        let query = vec!["rust".into()];
        let top_indices = bm25.get_top_n_indices(&query, 2);

        assert_eq!(top_indices.len(), 2);
        // Documents with "rust" should be at top
        assert!(top_indices.contains(&1) || top_indices.contains(&2));
    }
}
