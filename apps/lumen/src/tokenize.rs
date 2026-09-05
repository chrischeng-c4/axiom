// CODEGEN-BEGIN
//! Tokenizers for `text` fields.
//!
//! Each [`Analyzer`] variant maps to one tokenizer impl. Schemas pick
//! the analyzer at declaration time and a field is bound to it for life
//! (changing it requires a collection version bump + reindex).
//!
//! - `whitespace_lower` — lowercase + Unicode whitespace split. Default.
//!   Adequate for English; Chinese falls through as one big token.
//! - `jieba` — Chinese word segmentation when the `jieba` feature is on.
//!   With the feature **off** it is not whitespace splitting: the fallback is a
//!   CJK-bigram tokenizer. Input is cut into maximal runs of CJK characters
//!   (Han, Hiragana, Katakana, Hangul syllables — a char-range test, no
//!   segmenter) versus everything else. A CJK run of N chars emits N-1
//!   overlapping bigrams (`北京大學` → `北京`, `京大`, `大學`), matching Lucene
//!   `CJKBigramFilter`; a lone CJK char emits itself as a unigram rather than
//!   being dropped. Non-CJK runs go through `whitespace_lower` unchanged, so
//!   `lumen 搜尋引擎` keeps its `lumen` token. Scan order is preserved.
//!
//!   **Documents indexed under the pre-#1975 fallback need a reindex.** That
//!   fallback emitted the whole string as one token, so old postings carry
//!   whole-value terms that no bigram query will ever match. This is a
//!   degraded-mode caveat, not migration machinery — nothing detects or
//!   rewrites those postings.
//! - `ngram` — character N-grams (default 2..3). Useful for substring
//!   search on identifier-like fields.
//!
//! The output is a `Vec<String>` — duplicate tokens within one value are
//! preserved so callers can compute term frequency for BM25 later.

use std::borrow::Cow;

use crate::types::Analyzer;

/// Default n-gram window (inclusive on both sides).
pub const DEFAULT_NGRAM_MIN: usize = index_text::DEFAULT_NGRAM_MIN;
pub const DEFAULT_NGRAM_MAX: usize = index_text::DEFAULT_NGRAM_MAX;

/// Tokenize `text` with the chosen `analyzer`.
pub fn tokenize(text: &str, analyzer: Analyzer) -> Vec<String> {
    index_text::tokenize(text, shared_analyzer(analyzer))
}

pub(crate) fn for_whitespace_lower_cow<'a>(
    text: &'a str,
    mut emit: impl FnMut(Cow<'a, str>),
) -> u32 {
    index_text::for_whitespace_lower_cow(text, |token| emit(token))
}

fn shared_analyzer(analyzer: Analyzer) -> index_text::Analyzer {
    match analyzer {
        Analyzer::WhitespaceLower => index_text::Analyzer::WhitespaceLower,
        Analyzer::Jieba => index_text::Analyzer::Jieba,
        Analyzer::Ngram => index_text::Analyzer::Ngram,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace_lower_english() {
        assert_eq!(
            tokenize("Hello, World!", Analyzer::WhitespaceLower),
            vec!["hello", "world"]
        );
    }

    #[test]
    fn whitespace_lower_skips_empty() {
        assert_eq!(
            tokenize("   ", Analyzer::WhitespaceLower),
            Vec::<String>::new()
        );
    }

    #[test]
    fn ngram_basic() {
        let tokens = tokenize("abcd", Analyzer::Ngram);
        // bigrams: ab bc cd ; trigrams: abc bcd
        assert_eq!(tokens, vec!["ab", "bc", "cd", "abc", "bcd"]);
    }

    #[test]
    fn ngram_too_short_skipped() {
        assert!(tokenize("a", Analyzer::Ngram).is_empty());
    }

    // <HANDWRITE gap="missing-generator:unit-test" tracker="#1975" reason="unit-test section in tokenize.rs is hand-written pending codegen support">
    #[test]
    fn jieba_fallback_when_no_feature() {
        let tokens = tokenize("北京大學", Analyzer::Jieba);
        #[cfg(not(feature = "jieba"))]
        assert_eq!(tokens, vec!["北京", "京大", "大學"]);
        #[cfg(feature = "jieba")]
        assert!(tokens.len() >= 1);
    }

    #[test]
    fn jieba_fallback_mixed_text() {
        let tokens = tokenize("lumen 搜尋引擎", Analyzer::Jieba);
        #[cfg(not(feature = "jieba"))]
        {
            // Should contain "lumen" from the non-CJK run, plus CJK bigrams from 搜尋引擎
            assert!(tokens.contains(&"lumen".to_string()));
            assert!(tokens.contains(&"搜尋".to_string()));
            assert!(tokens.contains(&"尋引".to_string()));
            assert!(tokens.contains(&"引擎".to_string()));
            // Should NOT have the whole-string token
            assert!(!tokens.contains(&"lumen 搜尋引擎".to_string()));
            assert!(!tokens.contains(&"搜尋引擎".to_string()));
        }
        #[cfg(feature = "jieba")]
        assert!(tokens.len() >= 1);
    }

    #[test]
    fn jieba_fallback_single_cjk_char() {
        let tokens = tokenize("中", Analyzer::Jieba);
        #[cfg(not(feature = "jieba"))]
        assert_eq!(tokens, vec!["中"]);
        #[cfg(feature = "jieba")]
        assert_eq!(tokens, vec!["中"]);
    }

    #[test]
    fn jieba_fallback_empty() {
        let tokens = tokenize("", Analyzer::Jieba);
        assert_eq!(tokens, Vec::<String>::new());
    }

    #[test]
    fn jieba_fallback_whitespace() {
        let tokens = tokenize("   ", Analyzer::Jieba);
        assert_eq!(tokens, Vec::<String>::new());
    }
    // </HANDWRITE>
}
// CODEGEN-END
