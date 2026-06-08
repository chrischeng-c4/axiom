//! Trie data structure for dictionary lookup.

use std::collections::HashMap;

/// A node in the Trie.
#[derive(Debug, Clone, Default)]
struct TrieNode {
    children: HashMap<char, TrieNode>,
    freq: Option<u64>,
    tag: Option<String>,
}

/// Trie for efficient prefix lookup.
#[derive(Debug, Clone, Default)]
pub struct Trie {
    root: TrieNode,
    total_freq: u64,
}

impl Trie {
    pub fn new() -> Self {
        Self {
            root: TrieNode::default(),
            total_freq: 0,
        }
    }

    /// Insert a word with frequency and optional POS tag.
    pub fn insert(&mut self, word: &str, freq: u64, tag: Option<&str>) {
        let mut node = &mut self.root;
        for ch in word.chars() {
            node = node.children.entry(ch).or_default();
        }
        node.freq = Some(freq);
        node.tag = tag.map(|s| s.to_string());
        self.total_freq += freq;
    }

    /// Get frequency and tag for a word.
    pub fn get(&self, word: &str) -> Option<(u64, Option<&str>)> {
        let mut node = &self.root;
        for ch in word.chars() {
            node = node.children.get(&ch)?;
        }
        node.freq.map(|f| (f, node.tag.as_deref()))
    }

    /// Get frequency for a word, defaulting to 0.
    pub fn freq(&self, word: &str) -> u64 {
        self.get(word).map(|(f, _)| f).unwrap_or(0)
    }

    /// Check if word exists in dictionary.
    pub fn contains(&self, word: &str) -> bool {
        self.get(word).is_some()
    }

    /// Get all prefixes of the text that exist in the trie.
    /// Returns (end_position, frequency) pairs.
    pub fn prefixes(&self, text: &str) -> Vec<(usize, u64)> {
        let mut results = Vec::new();
        let mut node = &self.root;
        let mut pos = 0;

        for ch in text.chars() {
            match node.children.get(&ch) {
                Some(child) => {
                    node = child;
                    pos += ch.len_utf8();
                    if let Some(freq) = node.freq {
                        results.push((pos, freq));
                    }
                }
                None => break,
            }
        }

        results
    }

    /// Total frequency sum in dictionary.
    pub fn total_freq(&self) -> u64 {
        self.total_freq
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trie_basic() {
        let mut trie = Trie::new();
        trie.insert("北京", 100, Some("ns"));
        trie.insert("北", 50, None);

        assert!(trie.contains("北京"));
        assert!(trie.contains("北"));
        assert!(!trie.contains("京"));

        assert_eq!(trie.freq("北京"), 100);
        assert_eq!(trie.freq("北"), 50);
    }

    #[test]
    fn test_trie_prefixes() {
        let mut trie = Trie::new();
        trie.insert("清", 10, None);
        trie.insert("清华", 50, None);
        trie.insert("清华大学", 100, None);

        let prefixes = trie.prefixes("清华大学附近");
        assert_eq!(prefixes.len(), 3);
    }

    #[test]
    fn test_trie_get() {
        let mut trie = Trie::new();
        trie.insert("北京", 100, Some("ns"));

        let result = trie.get("北京");
        assert!(result.is_some());
        let (freq, tag) = result.unwrap();
        assert_eq!(freq, 100);
        assert_eq!(tag, Some("ns"));

        let missing = trie.get("上海");
        assert!(missing.is_none());
    }

    #[test]
    fn test_trie_total_freq() {
        let mut trie = Trie::new();
        trie.insert("北京", 100, None);
        trie.insert("上海", 50, None);
        assert_eq!(trie.total_freq(), 150);
    }
}
