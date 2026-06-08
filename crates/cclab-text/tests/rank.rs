//! Integration tests for the rank module (BM25/TF-IDF ranking algorithms).
//!
//! These tests were extracted from the inline `#[cfg(test)]` modules in the rank source files.

#![cfg(feature = "rank")]

use cclab_text::rank::{BM25Okapi, DefaultTokenizer, TfIdf, Tokenizer, WhitespaceTokenizer};

// ============================================================================
// BM25Okapi tests (from bm25.rs)
// ============================================================================

mod bm25_tests {
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

// ============================================================================
// TfIdf tests (from tfidf.rs)
// ============================================================================

mod tfidf_tests {
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

// ============================================================================
// Tokenizer tests (from tokenizer.rs)
// ============================================================================

mod tokenizer_tests {
    use super::*;

    #[test]
    fn test_default_tokenizer() {
        let tokenizer = DefaultTokenizer::new();
        let tokens = tokenizer.tokenize("Hello, World!");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_default_tokenizer_no_lowercase() {
        let tokenizer = DefaultTokenizer::with_lowercase(false);
        let tokens = tokenizer.tokenize("Hello World");
        assert_eq!(tokens, vec!["Hello", "World"]);
    }

    #[test]
    fn test_whitespace_tokenizer() {
        let tokenizer = WhitespaceTokenizer;
        let tokens = tokenizer.tokenize("Hello World");
        assert_eq!(tokens, vec!["hello", "world"]);
    }
}
