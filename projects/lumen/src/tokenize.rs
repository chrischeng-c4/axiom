//! Tokenizers for `text` fields.
//!
//! Each [`Analyzer`] variant maps to one tokenizer impl. Schemas pick
//! the analyzer at declaration time and a field is bound to it for life
//! (changing it requires a collection version bump + reindex).
//!
//! - `whitespace_lower` — lowercase + Unicode whitespace split. Default.
//!   Adequate for English; Chinese falls through as one big token.
//! - `jieba` — Chinese word segmentation. Feature-gated; falls back to
//!   `whitespace_lower` when the `jieba` feature is off.
//! - `ngram` — character N-grams (default 2..3). Useful for substring
//!   search on identifier-like fields.
//!
//! The output is a `Vec<String>` — duplicate tokens within one value are
//! preserved so callers can compute term frequency for BM25 later.

use crate::types::Analyzer;

/// Default n-gram window (inclusive on both sides).
pub const DEFAULT_NGRAM_MIN: usize = 2;
pub const DEFAULT_NGRAM_MAX: usize = 3;

/// Tokenize `text` with the chosen `analyzer`.
pub fn tokenize(text: &str, analyzer: Analyzer) -> Vec<String> {
    match analyzer {
        Analyzer::WhitespaceLower => whitespace_lower(text),
        Analyzer::Jieba => jieba(text),
        Analyzer::Ngram => ngram(text, DEFAULT_NGRAM_MIN, DEFAULT_NGRAM_MAX),
    }
}

fn whitespace_lower(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|t| {
            t.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|t| !t.is_empty())
        .collect()
}

#[cfg(feature = "jieba")]
fn jieba(text: &str) -> Vec<String> {
    use std::sync::OnceLock;
    static JIEBA: OnceLock<jieba_rs::Jieba> = OnceLock::new();
    let j = JIEBA.get_or_init(jieba_rs::Jieba::new);
    j.cut(text, false)
        .into_iter()
        .map(|t| t.to_lowercase())
        .filter(|t| !t.trim().is_empty())
        .collect()
}

#[cfg(not(feature = "jieba"))]
fn jieba(text: &str) -> Vec<String> {
    // No-feature fallback: treat the whole string as one token after
    // lowercasing. Honest: makes `match` over Chinese degenerate to an
    // exact-equality probe instead of silently dropping the field.
    let t = text.trim().to_lowercase();
    if t.is_empty() {
        vec![]
    } else {
        vec![t]
    }
}

fn ngram(text: &str, min: usize, max: usize) -> Vec<String> {
    let chars: Vec<char> = text
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(|c| c.to_lowercase())
        .collect();
    let mut out = Vec::new();
    for window in min..=max {
        if chars.len() < window {
            continue;
        }
        for start in 0..=chars.len() - window {
            out.push(chars[start..start + window].iter().collect());
        }
    }
    out
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

    #[test]
    fn jieba_fallback_when_no_feature() {
        let tokens = tokenize("中文測試", Analyzer::Jieba);
        #[cfg(not(feature = "jieba"))]
        assert_eq!(tokens, vec!["中文測試"]);
        #[cfg(feature = "jieba")]
        assert!(tokens.len() >= 1);
    }
}
