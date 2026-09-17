//! Streaming form of index-text's feature-off Jieba fallback.
//!
//! The fallback is intentionally not a dictionary segmenter. It emits CJK
//! bigrams (or a singleton for a one-scalar run), then delegates non-CJK runs
//! to the shared whitespace/lowercase tokenizer. It keeps only one pending CJK
//! scalar and an eight-byte UTF-8 buffer; an owned lowercase non-CJK token is
//! the existing shared tokenizer's bounded per-token workspace.

use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum JiebaFallbackStreamError<E> {
    Callback(E),
    TokenCountOverflow,
}

impl<E: fmt::Display> fmt::Display for JiebaFallbackStreamError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Callback(error) => write!(formatter, "jieba fallback callback: {error}"),
            Self::TokenCountOverflow => {
                formatter.write_str("jieba fallback document length exceeds u32")
            }
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for JiebaFallbackStreamError<E> {}

/// Emit the exact feature-off `index_text::Analyzer::Jieba` tokens without
/// collecting all input scalars or tokens. The callback must not retain `token`.
pub(crate) fn stream_fallback_jieba<E>(
    input: &str,
    mut emit: impl FnMut(&str) -> Result<(), E>,
) -> Result<u32, JiebaFallbackStreamError<E>> {
    let text = input.trim();
    if text.is_empty() {
        return Ok(0);
    }

    let mut count = 0u32;
    let mut non_cjk_start = 0usize;
    let mut cjk_previous = None;
    let mut cjk_len = 0usize;

    for (offset, character) in text.char_indices() {
        if is_cjk_char(character) {
            if cjk_previous.is_none() {
                emit_non_cjk(&text[non_cjk_start..offset], &mut count, &mut emit)?;
                cjk_previous = Some(character);
                cjk_len = 1;
            } else {
                emit_cjk_pair(
                    cjk_previous.expect("CJK run has a first scalar"),
                    character,
                    &mut count,
                    &mut emit,
                )?;
                cjk_previous = Some(character);
                cjk_len += 1;
            }
        } else if let Some(singleton) = cjk_previous.take() {
            if cjk_len == 1 {
                emit_cjk_singleton(singleton, &mut count, &mut emit)?;
            }
            cjk_len = 0;
            non_cjk_start = offset;
        }
    }

    if let Some(singleton) = cjk_previous {
        if cjk_len == 1 {
            emit_cjk_singleton(singleton, &mut count, &mut emit)?;
        }
    } else {
        emit_non_cjk(&text[non_cjk_start..], &mut count, &mut emit)?;
    }
    Ok(count)
}

fn emit_non_cjk<E>(
    run: &str,
    count: &mut u32,
    emit: &mut impl FnMut(&str) -> Result<(), E>,
) -> Result<(), JiebaFallbackStreamError<E>> {
    let mut failure = None;
    crate::tokenize::for_whitespace_lower_cow(run, |token| {
        if failure.is_some() {
            return;
        }
        match count.checked_add(1) {
            Some(next) => *count = next,
            None => {
                failure = Some(NonCjkFailure::Overflow);
                return;
            }
        }
        if let Err(error) = emit(token.as_ref()) {
            failure = Some(NonCjkFailure::Callback(error));
        }
    });
    match failure {
        None => Ok(()),
        Some(NonCjkFailure::Callback(error)) => Err(JiebaFallbackStreamError::Callback(error)),
        Some(NonCjkFailure::Overflow) => Err(JiebaFallbackStreamError::TokenCountOverflow),
    }
}

enum NonCjkFailure<E> {
    Callback(E),
    Overflow,
}

fn emit_cjk_singleton<E>(
    scalar: char,
    count: &mut u32,
    emit: &mut impl FnMut(&str) -> Result<(), E>,
) -> Result<(), JiebaFallbackStreamError<E>> {
    let mut utf8 = [0u8; 4];
    let bytes = scalar.encode_utf8(&mut utf8).len();
    emit_token(
        std::str::from_utf8(&utf8[..bytes]).expect("encoded scalar is UTF-8"),
        count,
        emit,
    )
}

fn emit_cjk_pair<E>(
    first: char,
    second: char,
    count: &mut u32,
    emit: &mut impl FnMut(&str) -> Result<(), E>,
) -> Result<(), JiebaFallbackStreamError<E>> {
    let mut utf8 = [0u8; 8];
    let first_bytes = first.encode_utf8(&mut utf8).len();
    let second_bytes = second.encode_utf8(&mut utf8[first_bytes..]).len();
    emit_token(
        std::str::from_utf8(&utf8[..first_bytes + second_bytes])
            .expect("encoded scalars are UTF-8"),
        count,
        emit,
    )
}

fn emit_token<E>(
    token: &str,
    count: &mut u32,
    emit: &mut impl FnMut(&str) -> Result<(), E>,
) -> Result<(), JiebaFallbackStreamError<E>> {
    *count = count
        .checked_add(1)
        .ok_or(JiebaFallbackStreamError::TokenCountOverflow)?;
    emit(token).map_err(JiebaFallbackStreamError::Callback)
}

pub(crate) fn is_cjk_char(character: char) -> bool {
    let code = character as u32;
    (0x4E00..=0x9FFF).contains(&code)
        || (0x3400..=0x4DBF).contains(&code)
        || (0x3040..=0x309F).contains(&code)
        || (0x30A0..=0x30FF).contains(&code)
        || (0xAC00..=0xD7A3).contains(&code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Analyzer;

    fn streamed(input: &str) -> Vec<String> {
        let mut out = Vec::new();
        stream_fallback_jieba(input, |token| {
            out.push(token.to_owned());
            Ok::<_, ()>(())
        })
        .unwrap();
        out
    }

    #[test]
    fn matches_shipped_fallback_for_mixed_trimmed_repeated_and_unicode_input() {
        let corpus = [
            "",
            " \t\n ",
            "北京大學",
            "中",
            "lumen 搜尋引擎",
            "  hello, ΣΟΣ! 中文 Straße  ",
            "中a文",
            "ＡＢＣ\tカタカナ",
            "İSTANBUL\u{3000}한글",
            "中中中",
        ];
        for input in corpus {
            let expected = crate::tokenize::tokenize(input, Analyzer::Jieba);
            let got = streamed(input);
            assert_eq!(got, expected, "input: {input:?}");
        }
    }

    #[test]
    fn every_fallback_cjk_range_edge_is_a_singleton_or_exact_adjacent_bigram() {
        let edges = [
            '\u{3400}', '\u{4DBF}', '\u{4E00}', '\u{9FFF}', '\u{3040}', '\u{309F}', '\u{30A0}',
            '\u{30FF}', '\u{AC00}', '\u{D7A3}',
        ];
        for edge in edges {
            assert_eq!(streamed(&edge.to_string()), vec![edge.to_string()]);
        }
        let run: String = edges.into_iter().collect();
        assert_eq!(
            streamed(&run),
            crate::tokenize::tokenize(&run, Analyzer::Jieba)
        );
    }

    #[test]
    fn immediately_outside_each_cjk_interval_stays_in_the_non_cjk_run() {
        let outside = [
            '\u{33FF}', '\u{4DC0}', '\u{4DFF}', '\u{A000}', '\u{303F}', '\u{3100}', '\u{ABFF}',
            '\u{D7A4}',
        ];
        for scalar in outside {
            assert!(!is_cjk_char(scalar));
            let input = format!("中{scalar}文");
            assert_eq!(
                streamed(&input),
                crate::tokenize::tokenize(&input, Analyzer::Jieba)
            );
        }
    }

    #[test]
    fn long_non_cjk_runs_match_shared_contextual_lowercase_without_output_collection() {
        let expected = crate::tokenize::tokenize("ΣΟΣ", Analyzer::Jieba);
        let repeats = 50_000usize;
        let input = " ΣΟΣ ".repeat(repeats);
        let mut seen = 0usize;
        let count = stream_fallback_jieba(&input, |token| {
            assert_eq!(token, expected[seen % expected.len()]);
            seen += 1;
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(seen, repeats * expected.len());
        assert_eq!(usize::try_from(count).unwrap(), seen);
    }

    #[test]
    fn callback_stops_without_collecting_later_cjk_tokens() {
        let mut calls = 0usize;
        let error = stream_fallback_jieba("北京大學搜尋", |_| {
            calls += 1;
            if calls == 3 {
                Err("stop")
            } else {
                Ok(())
            }
        });
        assert_eq!(error, Err(JiebaFallbackStreamError::Callback("stop")));
        assert_eq!(calls, 3);
    }

    #[test]
    fn long_cjk_run_makes_bounded_forward_progress() {
        let input = "中".repeat(100_000);
        let mut seen = 0u64;
        let count = stream_fallback_jieba(&input, |_| {
            seen += 1;
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(u64::from(count), 99_999);
        assert_eq!(seen, 99_999);
    }
}
