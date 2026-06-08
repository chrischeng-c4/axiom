//! Integration tests for the segment module (Chinese text segmentation).
//!
//! These tests were extracted from the inline `#[cfg(test)]` modules in the segment source files.

#![cfg(feature = "segment")]

use cclab_text::segment::KeywordAlgorithm;
use cclab_text::segment::{ChineseConverter, JiebaSegmenter, KeywordExtractor, TokenizeMode};

// ============================================================================
// JiebaSegmenter tests (from segmenter.rs)
// ============================================================================

mod segmenter_tests {
    use super::*;

    #[test]
    fn test_precise_segmentation() {
        let seg = JiebaSegmenter::new();
        let tokens = seg.tokenize("我来到北京清华大学", TokenizeMode::Precise);

        let words: Vec<&str> = tokens.iter().map(|t| t.word.as_str()).collect();
        // Should segment into meaningful words
        assert!(words.contains(&"我"));
        assert!(words.contains(&"来到"));
        assert!(words.contains(&"北京"));
        assert!(words.contains(&"清华大学"));
    }

    #[test]
    fn test_token_offsets() {
        let seg = JiebaSegmenter::new();
        let tokens = seg.tokenize("我来到", TokenizeMode::Precise);

        // First token should be "我" with correct offsets
        assert_eq!(tokens[0].word, "我");
        assert_eq!(tokens[0].start, 0);
        assert_eq!(tokens[0].end, 3); // "我" is 3 bytes in UTF-8
    }

    #[test]
    fn test_pos_tagging() {
        let seg = JiebaSegmenter::new();
        let tokens = seg.tag("我来到北京");

        // Check that POS tags are assigned
        let tagged: Vec<_> = tokens.iter().filter(|t| t.pos.is_some()).collect();
        assert!(!tagged.is_empty());
    }

    #[test]
    fn test_full_mode() {
        let seg = JiebaSegmenter::new();
        let tokens = seg.tokenize("清华大学", TokenizeMode::Full);

        // Full mode should return more segments
        assert!(tokens.len() >= 1);
    }

    #[test]
    fn test_add_word() {
        let mut seg = JiebaSegmenter::new();
        seg.add_word("新增词", 1000, None);
        let tokens = seg.tokenize("这是新增词测试", TokenizeMode::Precise);
        let words: Vec<&str> = tokens.iter().map(|t| t.word.as_str()).collect();
        assert!(words.contains(&"新增词"));
    }

    #[test]
    fn test_cut() {
        let seg = JiebaSegmenter::new();
        let words = seg.cut("我来到北京");
        assert!(!words.is_empty());
        assert!(words.contains(&"北京".to_string()));
    }

    #[test]
    fn test_lcut() {
        let seg = JiebaSegmenter::new();
        let words = seg.lcut("我来到北京");
        assert!(!words.is_empty());
        assert!(words.contains(&"北京".to_string()));
    }

    #[test]
    fn test_cut_for_search() {
        let seg = JiebaSegmenter::new();
        let words = seg.cut_for_search("清华大学");
        // Search mode should produce more segments
        assert!(!words.is_empty());
    }

    #[test]
    fn test_tokenize_batch() {
        let seg = JiebaSegmenter::new();
        let texts = vec!["我来到北京", "清华大学"];
        let results = seg.tokenize_batch(&texts, TokenizeMode::Precise);
        assert_eq!(results.len(), 2);
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn test_tokenize_batch_parallel() {
        let seg = JiebaSegmenter::new();
        let texts = vec!["我来到北京", "清华大学", "台灣"];
        let results = seg.tokenize_batch_parallel(&texts, TokenizeMode::Precise);
        assert_eq!(results.len(), 3);
        assert!(!results[0].is_empty());
        assert!(!results[1].is_empty());
        assert!(!results[2].is_empty());
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn test_cut_batch_parallel() {
        let seg = JiebaSegmenter::new();
        let texts = vec!["我来到北京", "清华大学"];
        let results = seg.cut_batch_parallel(&texts);
        assert_eq!(results.len(), 2);
        assert!(!results[0].is_empty());
        assert!(!results[1].is_empty());
    }

    #[test]
    fn test_tag() {
        let seg = JiebaSegmenter::new();
        let tokens = seg.tag("我来到北京清华大学");
        assert!(!tokens.is_empty());
        // Tokens should have POS tags
        for token in &tokens {
            // POS tag may be empty for unknown words, but token.word should not be
            assert!(!token.word.is_empty());
        }
    }
}

// ============================================================================
// KeywordExtractor tests (from keyword.rs)
// ============================================================================

mod keyword_tests {
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

// ============================================================================
// ChineseConverter tests (from traditional.rs)
// ============================================================================

mod converter_tests {
    use super::*;

    #[test]
    fn test_to_traditional() {
        let converter = ChineseConverter::new();

        assert_eq!(converter.to_traditional("国"), "國");
        assert_eq!(converter.to_traditional("学"), "學");
        assert_eq!(converter.to_traditional("华"), "華");
        assert_eq!(converter.to_traditional("清华大学"), "清華大學");
    }

    #[test]
    fn test_to_simplified() {
        let converter = ChineseConverter::new();

        assert_eq!(converter.to_simplified("國"), "国");
        assert_eq!(converter.to_simplified("學"), "学");
        assert_eq!(converter.to_simplified("華"), "华");
        assert_eq!(converter.to_simplified("清華大學"), "清华大学");
    }

    #[test]
    fn test_round_trip() {
        let converter = ChineseConverter::new();
        let original = "我来到北京清华大学";
        let traditional = converter.to_traditional(original);
        let back = converter.to_simplified(&traditional);

        assert_eq!(original, back);
    }

    #[test]
    fn test_has_traditional() {
        let converter = ChineseConverter::new();

        assert!(converter.has_traditional("學習"));
        assert!(!converter.has_traditional("学习"));
    }

    #[test]
    fn test_has_simplified() {
        let converter = ChineseConverter::new();

        assert!(converter.has_simplified("学习"));
        assert!(!converter.has_simplified("學習"));
    }

    #[test]
    fn test_mixed_text() {
        let converter = ChineseConverter::new();
        let text = "Hello 世界";

        // Non-Chinese characters should pass through unchanged
        assert_eq!(converter.to_traditional(text), "Hello 世界");
    }
}
