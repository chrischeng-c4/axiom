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

# Standardized projects/lumen/src/tokenize.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/tokenize.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DEFAULT_NGRAM_MAX` | projects/lumen/src/tokenize.rs | constant | pub | 25 |  |
| `DEFAULT_NGRAM_MIN` | projects/lumen/src/tokenize.rs | constant | pub | 24 |  |
| `for_whitespace_lower` | projects/lumen/src/tokenize.rs | function | pub | 42 | for_whitespace_lower(text: &str, mut emit: impl FnMut(String)) -> u32 |
| `for_whitespace_lower_cow` | projects/lumen/src/tokenize.rs | function | pub | 47 | for_whitespace_lower_cow(     mut text: &'a str,     mut emit: impl FnMut(Cow<'a, str>), ) -> u32 |
| `tokenize` | projects/lumen/src/tokenize.rs | function | pub | 29 | tokenize(text: &str, analyzer: Analyzer) -> Vec<String> |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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

pub(crate) fn for_whitespace_lower(text: &str, mut emit: impl FnMut(String)) -> u32 {
    for_whitespace_lower_cow(text, |tok| emit(tok.into_owned()))
}

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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/tokenize.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/tokenize.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```
