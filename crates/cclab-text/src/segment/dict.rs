//! Dictionary management with embedded default dictionary.

use super::trie::Trie;

/// Embedded minimal dictionary for common Chinese words.
/// In production, this would be a much larger dictionary (100k+ entries).
const DEFAULT_DICT: &str = include_str!("../data/dict.txt");

/// Load the default embedded dictionary into a Trie.
pub fn load_default_dict() -> Trie {
    let mut trie = Trie::new();

    for line in DEFAULT_DICT.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let word = parts[0];
            if let Ok(freq) = parts[1].parse::<u64>() {
                let tag = parts.get(2).copied();
                trie.insert(word, freq, tag);
            }
        }
    }

    trie
}

/// Load a custom dictionary from text content.
#[allow(dead_code)]
pub fn load_dict_from_str(content: &str) -> Trie {
    let mut trie = Trie::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let word = parts[0];
            if let Ok(freq) = parts[1].parse::<u64>() {
                let tag = parts.get(2).copied();
                trie.insert(word, freq, tag);
            }
        }
    }

    trie
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default_dict() {
        let trie = load_default_dict();
        // Should have loaded some words
        assert!(trie.total_freq() > 0);
        // Check some common words exist
        assert!(trie.contains("北京"));
        assert!(trie.contains("清华大学"));
    }

    #[test]
    fn test_load_dict_from_str() {
        let content = "hello 100 n\nworld 200 n\n# comment\n\n測試 150 v";
        let trie = load_dict_from_str(content);

        assert!(trie.contains("hello"));
        assert!(trie.contains("world"));
        assert!(trie.contains("測試"));
        assert!(!trie.contains("comment"));

        assert_eq!(trie.freq("hello"), 100);
        assert_eq!(trie.freq("world"), 200);
        assert_eq!(trie.freq("測試"), 150);
    }
}
