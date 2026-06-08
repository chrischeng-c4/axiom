//! Core segmentation logic with DAG-based path finding.

use super::dict::load_default_dict;
use super::hmm::HmmModel;
use super::trie::Trie;
use std::collections::HashMap;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Tokenization mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TokenizeMode {
    /// Precise mode: minimal segments, best for text analysis.
    #[default]
    Precise,
    /// Full mode: all possible word combinations.
    Full,
    /// Search mode: precise + granular segments for search indexing.
    Search,
}

/// A token with word, position, and optional POS tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub word: String,
    pub start: usize,
    pub end: usize,
    pub pos: Option<String>,
}

impl Token {
    pub fn new(word: impl Into<String>, start: usize, end: usize) -> Self {
        Self {
            word: word.into(),
            start,
            end,
            pos: None,
        }
    }

    pub fn with_pos(mut self, pos: impl Into<String>) -> Self {
        self.pos = Some(pos.into());
        self
    }
}

/// Chinese text segmenter using DAG + HMM.
pub struct JiebaSegmenter {
    trie: Trie,
    hmm: HmmModel,
}

impl Default for JiebaSegmenter {
    fn default() -> Self {
        Self::new()
    }
}

impl JiebaSegmenter {
    /// Create a new segmenter with default dictionary.
    pub fn new() -> Self {
        Self {
            trie: load_default_dict(),
            hmm: HmmModel::new(),
        }
    }

    /// Create a segmenter with custom dictionary.
    pub fn with_dict(trie: Trie) -> Self {
        Self {
            trie,
            hmm: HmmModel::new(),
        }
    }

    /// Add a word to the dictionary.
    pub fn add_word(&mut self, word: &str, freq: u64, tag: Option<&str>) {
        self.trie.insert(word, freq, tag);
    }

    /// Tokenize text into a vector of tokens.
    pub fn tokenize(&self, text: &str, mode: TokenizeMode) -> Vec<Token> {
        match mode {
            TokenizeMode::Precise => self.tokenize_precise(text),
            TokenizeMode::Full => self.tokenize_full(text),
            TokenizeMode::Search => self.tokenize_search(text),
        }
    }

    /// Precise segmentation (default mode).
    fn tokenize_precise(&self, text: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut byte_pos = 0;

        // Process text in chunks (Chinese vs non-Chinese)
        for segment in self.split_by_type(text) {
            let seg_len = segment.text().len();
            match segment {
                Segment::Chinese(s) => {
                    let seg_tokens = self.segment_chinese(&s, byte_pos);
                    tokens.extend(seg_tokens);
                }
                Segment::Other(s) => {
                    let end = byte_pos + s.len();
                    if !s.trim().is_empty() {
                        tokens.push(Token::new(s, byte_pos, end));
                    }
                }
            }
            byte_pos += seg_len;
        }

        tokens
    }

    /// Full segmentation: all possible words.
    fn tokenize_full(&self, text: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut byte_pos = 0;

        for segment in self.split_by_type(text) {
            let seg_len = segment.text().len();
            match segment {
                Segment::Chinese(s) => {
                    // Get all possible words from DAG
                    let chars: Vec<char> = s.chars().collect();
                    let mut char_pos = 0;
                    let mut current_byte = byte_pos;

                    while char_pos < chars.len() {
                        let remaining: String = chars[char_pos..].iter().collect();
                        let prefixes = self.trie.prefixes(&remaining);

                        if prefixes.is_empty() {
                            // Single character
                            let ch = chars[char_pos];
                            let ch_len = ch.len_utf8();
                            tokens.push(Token::new(
                                ch.to_string(),
                                current_byte,
                                current_byte + ch_len,
                            ));
                            current_byte += ch_len;
                            char_pos += 1;
                        } else {
                            // Add all matching prefixes
                            for (end_byte, _) in &prefixes {
                                let word: String = remaining[..*end_byte].to_string();
                                tokens.push(Token::new(
                                    word,
                                    current_byte,
                                    current_byte + end_byte,
                                ));
                            }
                            // Move by single character for full coverage
                            let ch_len = chars[char_pos].len_utf8();
                            current_byte += ch_len;
                            char_pos += 1;
                        }
                    }
                }
                Segment::Other(s) => {
                    let end = byte_pos + s.len();
                    if !s.trim().is_empty() {
                        tokens.push(Token::new(s, byte_pos, end));
                    }
                }
            }
            byte_pos += seg_len;
        }

        tokens
    }

    /// Search segmentation: precise + granular for indexing.
    fn tokenize_search(&self, text: &str) -> Vec<Token> {
        let precise = self.tokenize_precise(text);
        let mut tokens = Vec::new();

        for token in precise {
            // For long words, also add sub-segments
            if token.word.chars().count() > 2 {
                let chars: Vec<char> = token.word.chars().collect();
                let mut sub_byte = token.start;

                // Add bi-grams and tri-grams
                for i in 0..chars.len() {
                    for len in 2..=3 {
                        if i + len <= chars.len() {
                            let sub_word: String = chars[i..i + len].iter().collect();
                            if self.trie.contains(&sub_word) {
                                let sub_end = sub_byte + sub_word.len();
                                tokens.push(Token::new(sub_word, sub_byte, sub_end));
                            }
                        }
                    }
                    sub_byte += chars[i].len_utf8();
                }
            }
            tokens.push(token);
        }

        tokens
    }

    /// Segment Chinese text using DAG + dynamic programming.
    fn segment_chinese(&self, text: &str, base_offset: usize) -> Vec<Token> {
        let chars: Vec<char> = text.chars().collect();
        if chars.is_empty() {
            return vec![];
        }

        // Build DAG
        let dag = self.build_dag(text);

        // Calculate best path using dynamic programming
        let route = self.calc_route(&dag, text);

        // Extract tokens from route
        let mut tokens = Vec::new();
        let mut char_idx = 0;
        let mut byte_pos = base_offset;

        while char_idx < chars.len() {
            let end_idx = *route.get(&char_idx).unwrap_or(&char_idx);
            let word: String = chars[char_idx..=end_idx].iter().collect();
            let word_len = word.len();

            // Check if word is in dictionary
            if self.trie.contains(&word) {
                tokens.push(Token::new(word, byte_pos, byte_pos + word_len));
            } else {
                // Use HMM for unknown words
                let hmm_segments = self.hmm.segment(&word);
                let mut hmm_pos = byte_pos;
                for seg in hmm_segments {
                    let seg_len = seg.len();
                    tokens.push(Token::new(seg, hmm_pos, hmm_pos + seg_len));
                    hmm_pos += seg_len;
                }
            }

            byte_pos += word_len;
            char_idx = end_idx + 1;
        }

        tokens
    }

    /// Build DAG for Chinese text.
    /// Returns map: char_index -> vec of (end_char_index, freq)
    fn build_dag(&self, text: &str) -> HashMap<usize, Vec<(usize, u64)>> {
        let chars: Vec<char> = text.chars().collect();
        let mut dag: HashMap<usize, Vec<(usize, u64)>> = HashMap::new();

        for i in 0..chars.len() {
            let remaining: String = chars[i..].iter().collect();
            let prefixes = self.trie.prefixes(&remaining);

            let mut ends = Vec::new();
            if prefixes.is_empty() {
                // Single character as fallback
                ends.push((i, 1));
            } else {
                // Convert byte positions to char positions
                let mut byte_offset = 0;
                for (j, ch) in chars[i..].iter().enumerate() {
                    let ch_len = ch.len_utf8();
                    for &(end_byte, freq) in &prefixes {
                        if end_byte == byte_offset + ch_len {
                            ends.push((i + j, freq));
                        }
                    }
                    byte_offset += ch_len;
                }
            }

            if ends.is_empty() {
                ends.push((i, 1));
            }

            dag.insert(i, ends);
        }

        dag
    }

    /// Calculate optimal route using dynamic programming.
    /// Returns map: char_index -> best_end_char_index
    fn calc_route(
        &self,
        dag: &HashMap<usize, Vec<(usize, u64)>>,
        text: &str,
    ) -> HashMap<usize, usize> {
        let chars: Vec<char> = text.chars().collect();
        let n = chars.len();

        // route[i] = (log_prob, best_end_index)
        let mut route: HashMap<usize, (f64, usize)> = HashMap::new();
        let total_freq = self.trie.total_freq().max(1) as f64;

        // Initialize last position
        route.insert(n, (0.0, 0));

        // Backward DP
        for i in (0..n).rev() {
            if let Some(ends) = dag.get(&i) {
                let mut best = (f64::NEG_INFINITY, i);

                for &(end_idx, freq) in ends {
                    let prob = (freq as f64 / total_freq).ln();
                    let next_prob = route.get(&(end_idx + 1)).map(|r| r.0).unwrap_or(0.0);
                    let total = prob + next_prob;

                    if total > best.0 {
                        best = (total, end_idx);
                    }
                }

                route.insert(i, best);
            }
        }

        // Extract path
        route.into_iter().map(|(k, (_, v))| (k, v)).collect()
    }

    /// Split text into Chinese and non-Chinese segments.
    fn split_by_type(&self, text: &str) -> Vec<Segment> {
        let mut segments = Vec::new();
        let mut current = String::new();
        let mut is_chinese = false;

        for ch in text.chars() {
            let ch_is_chinese = is_chinese_char(ch);

            if current.is_empty() {
                is_chinese = ch_is_chinese;
                current.push(ch);
            } else if ch_is_chinese == is_chinese {
                current.push(ch);
            } else {
                if is_chinese {
                    segments.push(Segment::Chinese(current.clone()));
                } else {
                    segments.push(Segment::Other(current.clone()));
                }
                current.clear();
                current.push(ch);
                is_chinese = ch_is_chinese;
            }
        }

        if !current.is_empty() {
            if is_chinese {
                segments.push(Segment::Chinese(current));
            } else {
                segments.push(Segment::Other(current));
            }
        }

        segments
    }

    // === Jieba-compatible API aliases ===

    /// Cut text in precise mode (jieba-compatible alias).
    pub fn cut(&self, text: &str) -> Vec<String> {
        self.tokenize(text, TokenizeMode::Precise)
            .into_iter()
            .map(|t| t.word)
            .collect()
    }

    /// Cut for search indexing (jieba-compatible alias).
    pub fn cut_for_search(&self, text: &str) -> Vec<String> {
        self.tokenize(text, TokenizeMode::Search)
            .into_iter()
            .map(|t| t.word)
            .collect()
    }

    /// List cut - returns list of words (jieba-compatible alias).
    pub fn lcut(&self, text: &str) -> Vec<String> {
        self.tokenize(text, TokenizeMode::Precise)
            .into_iter()
            .map(|t| t.word)
            .collect()
    }

    /// POS tagging using dictionary tags.
    pub fn tag(&self, text: &str) -> Vec<Token> {
        let tokens = self.tokenize(text, TokenizeMode::Precise);

        tokens
            .into_iter()
            .map(|mut token| {
                if let Some((_, tag)) = self.trie.get(&token.word) {
                    token.pos = tag.map(|s| s.to_string());
                }
                token
            })
            .collect()
    }

    // === Batch Processing ===

    /// Tokenize multiple texts sequentially.
    pub fn tokenize_batch(&self, texts: &[&str], mode: TokenizeMode) -> Vec<Vec<Token>> {
        texts.iter().map(|text| self.tokenize(text, mode)).collect()
    }

    /// Tokenize multiple texts in parallel (requires `parallel` feature).
    #[cfg(feature = "parallel")]
    pub fn tokenize_batch_parallel(&self, texts: &[&str], mode: TokenizeMode) -> Vec<Vec<Token>> {
        texts
            .par_iter()
            .map(|text| self.tokenize(text, mode))
            .collect()
    }

    /// Cut multiple texts in parallel (requires `parallel` feature).
    #[cfg(feature = "parallel")]
    pub fn cut_batch_parallel(&self, texts: &[&str]) -> Vec<Vec<String>> {
        texts.par_iter().map(|text| self.cut(text)).collect()
    }
}

enum Segment {
    Chinese(String),
    Other(String),
}

impl Segment {
    fn text(&self) -> &str {
        match self {
            Segment::Chinese(s) | Segment::Other(s) => s,
        }
    }
}

/// Check if a character is Chinese.
fn is_chinese_char(ch: char) -> bool {
    matches!(ch, '\u{4E00}'..='\u{9FFF}'   // CJK Unified Ideographs
        | '\u{3400}'..='\u{4DBF}'          // CJK Extension A
        | '\u{F900}'..='\u{FAFF}'          // CJK Compatibility Ideographs
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precise_segmentation() {
        let seg = JiebaSegmenter::new();
        let tokens = seg.tokenize("我来到北京清华大学", TokenizeMode::Precise);

        let words: Vec<&str> = tokens.iter().map(|t| t.word.as_str()).collect();
        // Should segment into meaningful words
        assert!(words.contains(&"我"));
        assert!(words.contains(&"来到"));
        assert!(words.contains(&"北京"));
        assert!(words.contains(&"清华大学"));
    }

    #[test]
    fn test_token_offsets() {
        let seg = JiebaSegmenter::new();
        let tokens = seg.tokenize("我来到", TokenizeMode::Precise);

        // First token should be "我" with correct offsets
        assert_eq!(tokens[0].word, "我");
        assert_eq!(tokens[0].start, 0);
        assert_eq!(tokens[0].end, 3); // "我" is 3 bytes in UTF-8
    }

    #[test]
    fn test_pos_tagging() {
        let seg = JiebaSegmenter::new();
        let tokens = seg.tag("我来到北京");

        // Check that POS tags are assigned
        let tagged: Vec<_> = tokens.iter().filter(|t| t.pos.is_some()).collect();
        assert!(!tagged.is_empty());
    }

    #[test]
    fn test_full_mode() {
        let seg = JiebaSegmenter::new();
        let tokens = seg.tokenize("清华大学", TokenizeMode::Full);

        // Full mode should return more segments
        assert!(tokens.len() >= 1);
    }

    #[test]
    fn test_add_word() {
        let mut seg = JiebaSegmenter::new();
        seg.add_word("新增词", 1000, None);
        let tokens = seg.tokenize("这是新增词测试", TokenizeMode::Precise);
        let words: Vec<&str> = tokens.iter().map(|t| t.word.as_str()).collect();
        assert!(words.contains(&"新增词"));
    }

    #[test]
    fn test_cut() {
        let seg = JiebaSegmenter::new();
        let words = seg.cut("我来到北京");
        assert!(!words.is_empty());
        assert!(words.contains(&"北京".to_string()));
    }

    #[test]
    fn test_lcut() {
        let seg = JiebaSegmenter::new();
        let words = seg.lcut("我来到北京");
        assert!(!words.is_empty());
        assert!(words.contains(&"北京".to_string()));
    }

    #[test]
    fn test_cut_for_search() {
        let seg = JiebaSegmenter::new();
        let words = seg.cut_for_search("清华大学");
        // Search mode should produce more segments
        assert!(!words.is_empty());
    }

    #[test]
    fn test_tokenize_batch() {
        let seg = JiebaSegmenter::new();
        let texts = vec!["我来到北京", "清华大学"];
        let results = seg.tokenize_batch(&texts, TokenizeMode::Precise);
        assert_eq!(results.len(), 2);
    }

    #[test]
    #[cfg(feature = "parallel")]
    fn test_tokenize_batch_parallel() {
        let seg = JiebaSegmenter::new();
        let texts = vec!["我来到北京", "清华大学", "台灣"];
        let results = seg.tokenize_batch_parallel(&texts, TokenizeMode::Precise);
        assert_eq!(results.len(), 3);
        assert!(!results[0].is_empty());
        assert!(!results[1].is_empty());
        assert!(!results[2].is_empty());
    }

    #[test]
    #[cfg(feature = "parallel")]
    fn test_cut_batch_parallel() {
        let seg = JiebaSegmenter::new();
        let texts = vec!["我来到北京", "清华大学"];
        let results = seg.cut_batch_parallel(&texts);
        assert_eq!(results.len(), 2);
        assert!(!results[0].is_empty());
        assert!(!results[1].is_empty());
    }

    #[test]
    fn test_tag() {
        let seg = JiebaSegmenter::new();
        let tokens = seg.tag("我来到北京清华大学");
        assert!(!tokens.is_empty());
        // Tokens should have POS tags
        for token in &tokens {
            // POS tag may be empty for unknown words, but token.word should not be
            assert!(!token.word.is_empty());
        }
    }
}
