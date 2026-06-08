//! Tokenizer trait and default implementation.

/// Trait for tokenizing text into words.
///
/// Implement this trait to provide custom tokenization logic
/// (e.g., using jieba for Chinese text).
pub trait Tokenizer {
    /// Tokenize text into a vector of tokens.
    fn tokenize(&self, text: &str) -> Vec<String>;
}

/// Default tokenizer using whitespace and punctuation splitting.
#[derive(Debug, Clone, Default)]
pub struct DefaultTokenizer {
    lowercase: bool,
}

impl DefaultTokenizer {
    /// Create a new default tokenizer with lowercase enabled.
    pub fn new() -> Self {
        Self { lowercase: true }
    }

    /// Create a tokenizer with custom lowercase setting.
    pub fn with_lowercase(lowercase: bool) -> Self {
        Self { lowercase }
    }
}

impl Tokenizer for DefaultTokenizer {
    fn tokenize(&self, text: &str) -> Vec<String> {
        let text = if self.lowercase {
            text.to_lowercase()
        } else {
            text.to_string()
        };

        text.split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect()
    }
}

/// Simple whitespace tokenizer (no punctuation handling).
#[derive(Debug, Clone, Default)]
pub struct WhitespaceTokenizer;

impl Tokenizer for WhitespaceTokenizer {
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.split_whitespace().map(|s| s.to_lowercase()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_tokenizer() {
        let tokenizer = DefaultTokenizer::new();
        let tokens = tokenizer.tokenize("Hello, World!");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_default_tokenizer_no_lowercase() {
        let tokenizer = DefaultTokenizer::with_lowercase(false);
        let tokens = tokenizer.tokenize("Hello World");
        assert_eq!(tokens, vec!["Hello", "World"]);
    }

    #[test]
    fn test_whitespace_tokenizer() {
        let tokenizer = WhitespaceTokenizer;
        let tokens = tokenizer.tokenize("Hello World");
        assert_eq!(tokens, vec!["hello", "world"]);
    }
}
