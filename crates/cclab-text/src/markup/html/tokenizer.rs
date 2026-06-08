//! HTML tokenizer.

use std::collections::HashMap;

/// HTML token types.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// DOCTYPE declaration.
    DocType(String),
    /// Start tag with attributes.
    StartTag {
        name: String,
        attrs: HashMap<String, String>,
        self_closing: bool,
    },
    /// End tag.
    EndTag(String),
    /// Text content.
    Text(String),
    /// Comment.
    Comment(String),
}

/// HTML tokenizer state.
pub struct Tokenizer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    /// Create a new tokenizer.
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    /// Tokenize the entire input.
    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while self.pos < self.input.len() {
            if let Some(token) = self.next_token() {
                tokens.push(token);
            }
        }

        tokens
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace_between_tags();

        if self.pos >= self.input.len() {
            return None;
        }

        if self.starts_with("<!DOCTYPE") || self.starts_with("<!doctype") {
            return Some(self.read_doctype());
        }

        if self.starts_with("<!--") {
            return Some(self.read_comment());
        }

        if self.starts_with("</") {
            return Some(self.read_end_tag());
        }

        if self.starts_with("<") {
            return Some(self.read_start_tag());
        }

        Some(self.read_text())
    }

    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    fn skip_whitespace_between_tags(&mut self) {
        // Only skip pure whitespace if we're between tags
        let remaining = &self.input[self.pos..];
        if remaining.starts_with(char::is_whitespace) {
            // Check if it's followed by a tag
            let trimmed = remaining.trim_start();
            if trimmed.starts_with('<') {
                self.pos = self.input.len() - trimmed.len();
            }
        }
    }

    fn read_doctype(&mut self) -> Token {
        self.pos += 9; // Skip "<!DOCTYPE"
        let start = self.pos;

        while self.pos < self.input.len() && !self.starts_with(">") {
            self.pos += 1;
        }

        let content = self.input[start..self.pos].trim().to_string();
        self.pos += 1; // Skip ">"

        Token::DocType(content)
    }

    fn read_comment(&mut self) -> Token {
        self.pos += 4; // Skip "<!--"
        let start = self.pos;

        while self.pos < self.input.len() && !self.starts_with("-->") {
            self.pos += 1;
        }

        let content = self.input[start..self.pos].to_string();
        self.pos += 3; // Skip "-->"

        Token::Comment(content)
    }

    fn read_end_tag(&mut self) -> Token {
        self.pos += 2; // Skip "</"
        let start = self.pos;

        while self.pos < self.input.len() {
            let c = self.current_char();
            if c == '>' || c.is_whitespace() {
                break;
            }
            self.pos += 1;
        }

        let name = self.input[start..self.pos].to_lowercase();

        // Skip to closing >
        while self.pos < self.input.len() && !self.starts_with(">") {
            self.pos += 1;
        }
        self.pos += 1;

        Token::EndTag(name)
    }

    fn read_start_tag(&mut self) -> Token {
        self.pos += 1; // Skip "<"
        let start = self.pos;

        // Read tag name
        while self.pos < self.input.len() {
            let c = self.current_char();
            if c == '>' || c == '/' || c.is_whitespace() {
                break;
            }
            self.pos += 1;
        }

        let name = self.input[start..self.pos].to_lowercase();

        // Read attributes
        let attrs = self.read_attributes();

        // Check for self-closing
        self.skip_whitespace();
        let self_closing = self.starts_with("/>");

        // Skip to end of tag
        while self.pos < self.input.len() && !self.starts_with(">") {
            self.pos += 1;
        }
        self.pos += 1;

        Token::StartTag {
            name,
            attrs,
            self_closing,
        }
    }

    fn read_attributes(&mut self) -> HashMap<String, String> {
        let mut attrs = HashMap::new();

        loop {
            self.skip_whitespace();

            if self.pos >= self.input.len() {
                break;
            }

            let c = self.current_char();
            if c == '>' || c == '/' {
                break;
            }

            // Read attribute name
            let name_start = self.pos;
            while self.pos < self.input.len() {
                let c = self.current_char();
                if c == '=' || c == '>' || c == '/' || c.is_whitespace() {
                    break;
                }
                self.pos += 1;
            }

            let name = self.input[name_start..self.pos].to_lowercase();
            if name.is_empty() {
                self.pos += 1;
                continue;
            }

            self.skip_whitespace();

            // Check for =
            let value = if self.pos < self.input.len() && self.current_char() == '=' {
                self.pos += 1; // Skip "="
                self.skip_whitespace();
                self.read_attribute_value()
            } else {
                // Boolean attribute
                String::new()
            };

            attrs.insert(name, value);
        }

        attrs
    }

    fn read_attribute_value(&mut self) -> String {
        if self.pos >= self.input.len() {
            return String::new();
        }

        let quote = self.current_char();
        if quote == '"' || quote == '\'' {
            self.pos += 1;
            let start = self.pos;

            while self.pos < self.input.len() && self.current_char() != quote {
                self.pos += 1;
            }

            let value = self.input[start..self.pos].to_string();
            self.pos += 1; // Skip closing quote
            decode_entities(&value)
        } else {
            // Unquoted value
            let start = self.pos;
            while self.pos < self.input.len() {
                let c = self.current_char();
                if c.is_whitespace() || c == '>' || c == '/' {
                    break;
                }
                self.pos += 1;
            }
            decode_entities(&self.input[start..self.pos])
        }
    }

    fn read_text(&mut self) -> Token {
        let start = self.pos;

        while self.pos < self.input.len() && !self.starts_with("<") {
            self.pos += 1;
        }

        let text = decode_entities(&self.input[start..self.pos]);
        Token::Text(text)
    }

    fn current_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap_or('\0')
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.current_char().is_whitespace() {
            self.pos += 1;
        }
    }
}

/// Decode HTML entities.
fn decode_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&nbsp;", "\u{00A0}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let html = "<div>Hello</div>";
        let tokens = Tokenizer::new(html).tokenize();

        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0], Token::StartTag { name, .. } if name == "div"));
        assert!(matches!(&tokens[1], Token::Text(t) if t == "Hello"));
        assert!(matches!(&tokens[2], Token::EndTag(n) if n == "div"));
    }

    #[test]
    fn test_tokenize_attributes() {
        let html = r#"<a href="https://example.com" class="link">Click</a>"#;
        let tokens = Tokenizer::new(html).tokenize();

        if let Token::StartTag { name, attrs, .. } = &tokens[0] {
            assert_eq!(name, "a");
            assert_eq!(attrs.get("href"), Some(&"https://example.com".to_string()));
            assert_eq!(attrs.get("class"), Some(&"link".to_string()));
        } else {
            panic!("Expected StartTag");
        }
    }

    #[test]
    fn test_tokenize_comment() {
        let html = "<!-- This is a comment --><p>Text</p>";
        let tokens = Tokenizer::new(html).tokenize();

        assert!(matches!(&tokens[0], Token::Comment(c) if c.contains("comment")));
    }

    #[test]
    fn test_tokenize_self_closing() {
        let html = "<br /><img src='test.png'/>";
        let tokens = Tokenizer::new(html).tokenize();

        assert!(matches!(
            &tokens[0],
            Token::StartTag {
                self_closing: true,
                ..
            }
        ));
        assert!(matches!(
            &tokens[1],
            Token::StartTag {
                self_closing: true,
                ..
            }
        ));
    }
}
