---
id: projects-lumen-src-tokenize-rs
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
fill_sections: [overview, source, changes]
---

# Standardized apps/lumen/src/tokenize.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/lumen/src/tokenize.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DEFAULT_NGRAM_MAX` | apps/lumen/src/tokenize.rs | constant | pub | 25 |  |
| `DEFAULT_NGRAM_MIN` | apps/lumen/src/tokenize.rs | constant | pub | 24 |  |
| `for_whitespace_lower` | apps/lumen/src/tokenize.rs | function | pub | 42 | for_whitespace_lower(text: &str, mut emit: impl FnMut(String)) -> u32 |
| `for_whitespace_lower_cow` | apps/lumen/src/tokenize.rs | function | pub | 47 | for_whitespace_lower_cow(     mut text: &'a str,     mut emit: impl FnMut(Cow<'a, str>), ) -> u32 |
| `is_cjk_char` | apps/lumen/src/tokenize.rs | function | (private) | 145 | is_cjk_char(c: char) -> bool |
| `tokenize` | apps/lumen/src/tokenize.rs | function | pub | 29 | tokenize(text: &str, analyzer: Analyzer) -> Vec<String> |
## Source
<!-- type: rust-source-unit lang: rust -->


```rust
// SPEC-MANAGED: apps/lumen/tech-design/semantic/source/apps-lumen-src-tokenize-rs.md#rust-source-unit
// CODEGEN-BEGIN
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

use std::borrow::Cow;

use crate::types::Analyzer;

/// Default n-gram window (inclusive on both sides).
pub const DEFAULT_NGRAM_MIN: usize = 2;
pub const DEFAULT_NGRAM_MAX: usize = 3;

/// Tokenize `text` with the chosen `analyzer`.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-tokenize-rs.md#source
pub fn tokenize(text: &str, analyzer: Analyzer) -> Vec<String> {
    match analyzer {
        Analyzer::WhitespaceLower => {
            let mut out = Vec::new();
            for_whitespace_lower(text, |tok| out.push(tok));
            out
        }
        Analyzer::Jieba => jieba(text),
        Analyzer::Ngram => ngram(text, DEFAULT_NGRAM_MIN, DEFAULT_NGRAM_MAX),
    }
}

/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-tokenize-rs.md#source
pub(crate) fn for_whitespace_lower(text: &str, mut emit: impl FnMut(String)) -> u32 {
    for_whitespace_lower_cow(text, |tok| emit(tok.into_owned()))
}

/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-tokenize-rs.md#source
pub(crate) fn for_whitespace_lower_cow<'a>(
    mut text: &'a str,
    mut emit: impl FnMut(Cow<'a, str>),
) -> u32 {
    let mut emitted = 0u32;
    while !text.is_empty() {
        let trimmed_start = text.trim_start();
        if trimmed_start.is_empty() {
            break;
        }
        let skipped = text.len() - trimmed_start.len();
        text = &text[skipped..];
        let end = text.find(char::is_whitespace).unwrap_or(text.len());
        let raw = &text[..end];
        text = &text[end..];
        let token = raw.trim_matches(|c: char| !c.is_alphanumeric());
        if token.is_empty() {
            continue;
        }
        emitted += 1;
        if token
            .as_bytes()
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        {
            emit(Cow::Borrowed(token));
        } else {
            emit(Cow::Owned(token.to_lowercase()));
        }
    }
    emitted
}

// <HANDWRITE gap="missing-generator:logic" tracker="#1975" reason="logic section in tokenize.rs is hand-written pending codegen support">
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
    // No-feature fallback: CJK-bigram tokenizer.
    // Split input into maximal runs of CJK characters vs non-CJK characters.
    // CJK runs emit overlapping 2-char bigrams (or a single unigram for length-1 runs).
    // Non-CJK runs are tokenized via the existing for_whitespace_lower path.
    // Output preserves scan order (CJK and non-CJK tokens interleaved as they appear).

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return vec![];
    }

    let mut tokens = Vec::new();
    let chars: Vec<char> = trimmed.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        let is_cjk = is_cjk_char(c);

        // Consume a maximal run of same type (CJK or non-CJK)
        let run_start = i;
        while i < chars.len() && is_cjk_char(chars[i]) == is_cjk {
            i += 1;
        }

        let run: String = chars[run_start..i].iter().collect();

        if is_cjk {
            // CJK run: emit overlapping bigrams (or unigram for length 1)
            let run_chars: Vec<char> = run.chars().collect();
            if run_chars.len() == 1 {
                tokens.push(run_chars[0].to_string());
            } else {
                for j in 0..run_chars.len() - 1 {
                    let bigram: String = run_chars[j..j + 2].iter().collect();
                    tokens.push(bigram);
                }
            }
        } else {
            // Non-CJK run: tokenize via existing for_whitespace_lower path
            for_whitespace_lower(&run, |tok| tokens.push(tok));
        }
    }

    tokens
}

/// Check if a Unicode character is in a CJK range.
/// Covers Han (CJK Unified Ideographs + Ext A), Hiragana, Katakana, Hangul syllables.
fn is_cjk_char(c: char) -> bool {
    let code = c as u32;
    // Han: U+4E00..U+9FFF (CJK Unified Ideographs)
    (code >= 0x4E00 && code <= 0x9FFF)
        // Han extension A: U+3400..U+4DBF
        || (code >= 0x3400 && code <= 0x4DBF)
        // Hiragana: U+3040..U+309F
        || (code >= 0x3040 && code <= 0x309F)
        // Katakana: U+30A0..U+30FF
        || (code >= 0x30A0 && code <= 0x30FF)
        // Hangul syllables: U+AC00..U+D7A3
        || (code >= 0xAC00 && code <= 0xD7A3)
}
// </HANDWRITE>

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
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/src/tokenize.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `apps/lumen/src/tokenize.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```
