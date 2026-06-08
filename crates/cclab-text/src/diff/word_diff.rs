//! Word-level diff algorithm.
//!
//! Computes differences at the word level for more granular change detection.

/// A word-level diff operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WordDiffOp {
    /// Word is the same in both texts.
    Equal(String),
    /// Word was inserted.
    Insert(String),
    /// Word was deleted.
    Delete(String),
}

/// Compute a word-level diff between two strings.
///
/// Splits text on whitespace boundaries and diffs the resulting words.
pub fn diff_words(old: &str, new: &str) -> Vec<WordDiffOp> {
    let old_words = tokenize_words(old);
    let new_words = tokenize_words(new);

    let lcs = compute_lcs(&old_words, &new_words);
    build_word_diff(&old_words, &new_words, &lcs)
}

/// Tokenize text into words while preserving whitespace info.
fn tokenize_words(text: &str) -> Vec<&str> {
    let mut words = Vec::new();
    let mut start = 0;
    let mut in_word = false;

    for (i, ch) in text.char_indices() {
        if ch.is_whitespace() {
            if in_word {
                words.push(&text[start..i]);
                in_word = false;
            }
            // Add whitespace as a token
            start = i;
            words.push(&text[i..i + ch.len_utf8()]);
        } else if !in_word {
            start = i;
            in_word = true;
        }
    }

    if in_word {
        words.push(&text[start..]);
    }

    words
}

/// Compute LCS table for word sequences.
fn compute_lcs<'a>(old: &[&'a str], new: &[&'a str]) -> Vec<Vec<usize>> {
    let m = old.len();
    let n = new.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if old[i - 1] == new[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    dp
}

/// Build word diff operations from LCS table.
fn build_word_diff(old: &[&str], new: &[&str], lcs: &[Vec<usize>]) -> Vec<WordDiffOp> {
    let mut ops = Vec::new();
    let mut i = old.len();
    let mut j = new.len();
    let mut trace = Vec::new();

    while i > 0 && j > 0 {
        if old[i - 1] == new[j - 1] {
            trace.push(WordDiffOp::Equal(old[i - 1].to_string()));
            i -= 1;
            j -= 1;
        } else if lcs[i - 1][j] >= lcs[i][j - 1] {
            trace.push(WordDiffOp::Delete(old[i - 1].to_string()));
            i -= 1;
        } else {
            trace.push(WordDiffOp::Insert(new[j - 1].to_string()));
            j -= 1;
        }
    }

    while i > 0 {
        trace.push(WordDiffOp::Delete(old[i - 1].to_string()));
        i -= 1;
    }

    while j > 0 {
        trace.push(WordDiffOp::Insert(new[j - 1].to_string()));
        j -= 1;
    }

    trace.reverse();
    ops.extend(trace);
    ops
}

/// Format word diff as inline markup.
///
/// Deleted words are wrapped in `[-...-]`, inserted words in `{+...+}`.
pub fn format_word_diff(ops: &[WordDiffOp]) -> String {
    let mut output = String::new();

    for op in ops {
        match op {
            WordDiffOp::Equal(word) => output.push_str(word),
            WordDiffOp::Delete(word) => {
                output.push_str("[-");
                output.push_str(word);
                output.push_str("-]");
            }
            WordDiffOp::Insert(word) => {
                output.push_str("{+");
                output.push_str(word);
                output.push_str("+}");
            }
        }
    }

    output
}

/// Format word diff as HTML.
///
/// Deleted words get `<del>` tags, inserted words get `<ins>` tags.
pub fn format_word_diff_html(ops: &[WordDiffOp]) -> String {
    let mut output = String::new();

    for op in ops {
        match op {
            WordDiffOp::Equal(word) => output.push_str(word),
            WordDiffOp::Delete(word) => {
                output.push_str("<del>");
                output.push_str(word);
                output.push_str("</del>");
            }
            WordDiffOp::Insert(word) => {
                output.push_str("<ins>");
                output.push_str(word);
                output.push_str("</ins>");
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_identical() {
        let ops = diff_words("hello world", "hello world");
        let changed = ops.iter().any(|op| !matches!(op, WordDiffOp::Equal(_)));
        assert!(!changed);
    }

    #[test]
    fn test_diff_insert_word() {
        let ops = diff_words("hello world", "hello beautiful world");
        assert!(ops
            .iter()
            .any(|op| matches!(op, WordDiffOp::Insert(w) if w == "beautiful")));
    }

    #[test]
    fn test_diff_delete_word() {
        let ops = diff_words("hello beautiful world", "hello world");
        assert!(ops
            .iter()
            .any(|op| matches!(op, WordDiffOp::Delete(w) if w == "beautiful")));
    }

    #[test]
    fn test_format_word_diff() {
        let ops = diff_words("the cat sat", "the dog sat");
        let formatted = format_word_diff(&ops);
        assert!(formatted.contains("[-cat-]") || formatted.contains("{+dog+}"));
    }

    #[test]
    fn test_format_word_diff_html() {
        let ops = diff_words("hello world", "hello there");
        let html = format_word_diff_html(&ops);
        assert!(html.contains("<del>") || html.contains("<ins>"));
    }
}
