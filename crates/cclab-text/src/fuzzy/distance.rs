//! String distance algorithms.
//!
//! Provides implementations of:
//! - Levenshtein distance
//! - Damerau-Levenshtein distance (transpositions)
//! - Jaro-Winkler similarity
//! - Hamming distance

use std::cmp::min;

/// Compute the Levenshtein edit distance between two strings.
///
/// The Levenshtein distance is the minimum number of single-character edits
/// (insertions, deletions, substitutions) required to change one string into another.
///
/// # Examples
/// ```
/// use cclab_text::fuzzy::levenshtein;
/// assert_eq!(levenshtein("kitten", "sitting"), 3);
/// assert_eq!(levenshtein("", "abc"), 3);
/// assert_eq!(levenshtein("abc", "abc"), 0);
/// ```
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    // Use two-row optimization for O(min(m,n)) space
    let mut prev = vec![0usize; b_len + 1];
    let mut curr = vec![0usize; b_len + 1];

    for j in 0..=b_len {
        prev[j] = j;
    }

    for i in 1..=a_len {
        curr[0] = i;
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr[j] = min(min(curr[j - 1] + 1, prev[j] + 1), prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b_len]
}

/// Compute the normalized Levenshtein similarity (0.0 to 1.0).
///
/// Returns 1.0 for identical strings and 0.0 for completely different strings.
pub fn levenshtein_normalized(a: &str, b: &str) -> f64 {
    let max_len = a.chars().count().max(b.chars().count());
    if max_len == 0 {
        return 1.0;
    }
    1.0 - (levenshtein(a, b) as f64 / max_len as f64)
}

/// Compute the Damerau-Levenshtein distance between two strings.
///
/// Like Levenshtein but also allows transpositions of two adjacent characters
/// as a single edit operation.
///
/// # Examples
/// ```
/// use cclab_text::fuzzy::damerau_levenshtein;
/// assert_eq!(damerau_levenshtein("ca", "abc"), 3);
/// assert_eq!(damerau_levenshtein("ab", "ba"), 1); // transposition
/// ```
pub fn damerau_levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    // Full matrix needed for transposition lookback
    let mut d = vec![vec![0usize; b_len + 1]; a_len + 1];

    for i in 0..=a_len {
        d[i][0] = i;
    }
    for j in 0..=b_len {
        d[0][j] = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };

            d[i][j] = min(
                min(d[i - 1][j] + 1, d[i][j - 1] + 1),
                d[i - 1][j - 1] + cost,
            );

            // Check for transposition
            if i > 1
                && j > 1
                && a_chars[i - 1] == b_chars[j - 2]
                && a_chars[i - 2] == b_chars[j - 1]
            {
                d[i][j] = min(d[i][j], d[i - 2][j - 2] + cost);
            }
        }
    }

    d[a_len][b_len]
}

/// Compute the Jaro similarity between two strings.
///
/// Returns a value between 0.0 (no similarity) and 1.0 (identical).
fn jaro(a: &str, b: &str) -> f64 {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 && b_len == 0 {
        return 1.0;
    }
    if a_len == 0 || b_len == 0 {
        return 0.0;
    }

    let match_distance = (a_len.max(b_len) / 2).saturating_sub(1);

    let mut a_matches = vec![false; a_len];
    let mut b_matches = vec![false; b_len];

    let mut matches = 0usize;
    let mut transpositions = 0usize;

    // Find matching characters
    for i in 0..a_len {
        let start = i.saturating_sub(match_distance);
        let end = min(i + match_distance + 1, b_len);

        for j in start..end {
            if b_matches[j] || a_chars[i] != b_chars[j] {
                continue;
            }
            a_matches[i] = true;
            b_matches[j] = true;
            matches += 1;
            break;
        }
    }

    if matches == 0 {
        return 0.0;
    }

    // Count transpositions
    let mut k = 0;
    for i in 0..a_len {
        if !a_matches[i] {
            continue;
        }
        while !b_matches[k] {
            k += 1;
        }
        if a_chars[i] != b_chars[k] {
            transpositions += 1;
        }
        k += 1;
    }

    let m = matches as f64;
    (m / a_len as f64 + m / b_len as f64 + (m - transpositions as f64 / 2.0) / m) / 3.0
}

/// Compute the Jaro-Winkler similarity between two strings.
///
/// Extends Jaro similarity by giving a boost to strings that share a common prefix.
///
/// # Arguments
/// * `a` - First string
/// * `b` - Second string
/// * `prefix_weight` - Weight for the common prefix bonus (default: 0.1, max: 0.25)
///
/// # Examples
/// ```
/// use cclab_text::fuzzy::jaro_winkler;
/// let sim = jaro_winkler("martha", "marhta", 0.1);
/// assert!(sim > 0.96);
/// ```
pub fn jaro_winkler(a: &str, b: &str, prefix_weight: f64) -> f64 {
    let jaro_sim = jaro(a, b);

    // Find common prefix length (max 4)
    let prefix_len = a
        .chars()
        .zip(b.chars())
        .take(4)
        .take_while(|(ca, cb)| ca == cb)
        .count();

    let weight = prefix_weight.min(0.25);
    jaro_sim + (prefix_len as f64 * weight * (1.0 - jaro_sim))
}

/// Compute the Hamming distance between two strings.
///
/// The Hamming distance is the number of positions at which the corresponding
/// characters are different. Strings must be the same length.
///
/// # Returns
/// `None` if strings have different lengths.
///
/// # Examples
/// ```
/// use cclab_text::fuzzy::hamming;
/// assert_eq!(hamming("karolin", "kathrin"), Some(3));
/// assert_eq!(hamming("abc", "abcd"), None);
/// ```
pub fn hamming(a: &str, b: &str) -> Option<usize> {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    if a_chars.len() != b_chars.len() {
        return None;
    }

    Some(
        a_chars
            .iter()
            .zip(b_chars.iter())
            .filter(|(ca, cb)| ca != cb)
            .count(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("flaw", "lawn"), 2);
    }

    #[test]
    fn test_levenshtein_normalized() {
        assert!((levenshtein_normalized("abc", "abc") - 1.0).abs() < f64::EPSILON);
        assert!((levenshtein_normalized("", "") - 1.0).abs() < f64::EPSILON);
        assert!(levenshtein_normalized("abc", "xyz") < 0.01);
    }

    #[test]
    fn test_damerau_levenshtein() {
        assert_eq!(damerau_levenshtein("ab", "ba"), 1);
        assert_eq!(damerau_levenshtein("abc", "abc"), 0);
        assert_eq!(damerau_levenshtein("", "abc"), 3);
        assert_eq!(damerau_levenshtein("ca", "abc"), 3);
    }

    #[test]
    fn test_jaro_winkler() {
        let sim = jaro_winkler("martha", "marhta", 0.1);
        assert!(sim > 0.96);
        assert!((jaro_winkler("", "", 0.1) - 1.0).abs() < f64::EPSILON);
        assert!(jaro_winkler("abc", "xyz", 0.1) < 0.1);
    }

    #[test]
    fn test_hamming() {
        assert_eq!(hamming("karolin", "kathrin"), Some(3));
        assert_eq!(hamming("1011101", "1001001"), Some(2));
        assert_eq!(hamming("abc", "abc"), Some(0));
        assert_eq!(hamming("abc", "abcd"), None);
    }
}
