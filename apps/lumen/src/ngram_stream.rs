//! Allocation-bounded default n-gram streaming for a staged Text record.
//!
//! This deliberately preserves `index_text::ngram` order: every bigram first,
//! then every trigram. It rescans the borrowed input once per fixed width so it
//! never stores all normalized scalars or all emitted tokens.

use std::fmt;

use crate::tokenize::{DEFAULT_NGRAM_MAX, DEFAULT_NGRAM_MIN};

/// A callback failure is returned unchanged. `TokenCountOverflow` prevents the
/// future Text apply path from truncating the `u32` document length it stores.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum NgramStreamError<E> {
    Callback(E),
    TokenCountOverflow,
}

impl<E: fmt::Display> fmt::Display for NgramStreamError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Callback(error) => write!(f, "ngram callback: {error}"),
            Self::TokenCountOverflow => f.write_str("ngram document length exceeds u32"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for NgramStreamError<E> {}

/// Stream default n-grams without allocating normalized input or tokens.
///
/// The only working buffers are three Unicode scalar slots and twelve UTF-8
/// bytes, enough for the default maximum window of three scalars. The callback
/// borrows that UTF-8 buffer and must not retain the `&str`.
pub(crate) fn stream_default_ngrams<E>(
    text: &str,
    mut emit: impl FnMut(&str) -> Result<(), E>,
) -> Result<u32, NgramStreamError<E>> {
    debug_assert_eq!(DEFAULT_NGRAM_MIN, 2);
    debug_assert_eq!(DEFAULT_NGRAM_MAX, 3);
    let mut total = 0u32;
    for width in DEFAULT_NGRAM_MIN..=DEFAULT_NGRAM_MAX {
        stream_width(text, width, &mut total, &mut emit)?;
    }
    Ok(total)
}

fn stream_width<E>(
    text: &str,
    width: usize,
    total: &mut u32,
    emit: &mut impl FnMut(&str) -> Result<(), E>,
) -> Result<(), NgramStreamError<E>> {
    debug_assert!((DEFAULT_NGRAM_MIN..=DEFAULT_NGRAM_MAX).contains(&width));
    let mut chars = ['\0'; DEFAULT_NGRAM_MAX];
    let mut len = 0usize;

    for character in text.chars().filter(|character| !character.is_whitespace()) {
        for lowered in character.to_lowercase() {
            if len < width {
                chars[len] = lowered;
                len += 1;
            } else {
                chars.copy_within(1..width, 0);
                chars[width - 1] = lowered;
            }
            if len != width {
                continue;
            }

            let mut utf8 = [0u8; DEFAULT_NGRAM_MAX * 4];
            let mut bytes = 0usize;
            for scalar in &chars[..width] {
                let encoded = scalar.encode_utf8(&mut utf8[bytes..]);
                bytes += encoded.len();
            }
            // Check before changing the index: the stored document length is
            // `u32`, so an unrepresentable next token must not leave a partial
            // posting update behind.
            *total = total
                .checked_add(1)
                .ok_or(NgramStreamError::TokenCountOverflow)?;
            // Every byte came from `char::encode_utf8`, so this is valid UTF-8.
            let token = std::str::from_utf8(&utf8[..bytes]).expect("encoded chars are UTF-8");
            emit(token).map_err(NgramStreamError::Callback)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Analyzer;

    #[test]
    fn stream_matches_the_shared_tokenizer_for_unicode_and_whitespace() {
        let corpus = [
            "",
            "a",
            "abcd",
            "ab cd",
            "  a\tb\nc  ",
            "İstanbul Straße",
            "中 文 字",
            "ＡＢＣＤ",
            "💩a💩",
            "e\u{301} e\u{301}",
            "Σίσυφος",
        ];
        for input in corpus {
            let mut streamed = Vec::new();
            let count = stream_default_ngrams(input, |token| {
                streamed.push(token.to_owned());
                Ok::<_, ()>(())
            })
            .unwrap();
            let expected = crate::tokenize::tokenize(input, Analyzer::Ngram);
            assert_eq!(streamed, expected, "input: {input:?}");
            assert_eq!(usize::try_from(count).unwrap(), expected.len());
        }
    }

    #[test]
    fn long_input_uses_a_non_retaining_callback() {
        let input = "a".repeat(100_000);
        let mut seen = 0u64;
        let count = stream_default_ngrams(&input, |_| {
            seen += 1;
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(u64::from(count), (100_000 - 1 + 100_000 - 2) as u64);
        assert_eq!(seen, u64::from(count));
    }

    #[derive(Debug, PartialEq, Eq)]
    struct Stop;
    impl fmt::Display for Stop {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("stop")
        }
    }
    impl std::error::Error for Stop {}

    #[test]
    fn callback_error_stops_before_later_windows() {
        let mut calls = 0usize;
        let result = stream_default_ngrams("abcdef", |_| {
            calls += 1;
            if calls == 3 {
                Err(Stop)
            } else {
                Ok(())
            }
        });
        assert_eq!(result, Err(NgramStreamError::Callback(Stop)));
        assert_eq!(calls, 3);
    }
}
