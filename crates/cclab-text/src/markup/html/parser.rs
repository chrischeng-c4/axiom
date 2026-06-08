//! HTML parser - builds DOM from tokens.

use super::tokenizer::{Token, Tokenizer};
use crate::markup::dom::{Document, NodeId};

/// Parse HTML string into a Document.
pub fn parse_html(html: &str) -> Document {
    let tokens = Tokenizer::new(html).tokenize();
    Parser::new(tokens).parse()
}

/// HTML parser state.
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    doc: Document,
    /// Stack of open element IDs.
    stack: Vec<NodeId>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        let doc = Document::new();
        let root = doc.root();
        Self {
            tokens,
            pos: 0,
            doc,
            stack: vec![root],
        }
    }

    fn parse(mut self) -> Document {
        while self.pos < self.tokens.len() {
            self.process_token();
            self.pos += 1;
        }

        self.doc
    }

    fn process_token(&mut self) {
        let token = self.tokens[self.pos].clone();

        match token {
            Token::DocType(_) => {
                // Ignore DOCTYPE for now
            }
            Token::StartTag {
                name,
                attrs,
                self_closing,
            } => {
                let element_id = self.doc.create_element(&name);

                // Set attributes
                if let Some(element) = self.doc.get_mut(element_id) {
                    for (key, value) in attrs {
                        element.set_attr(key, value);
                    }
                }

                // Append to current parent
                let parent = *self.stack.last().unwrap_or(&self.doc.root());
                let _ = self.doc.append_child(parent, element_id);

                // Handle special cases
                if !self_closing && !is_void_element(&name) {
                    // Auto-close implied tags
                    self.auto_close_tags(&name);
                    self.stack.push(element_id);
                }
            }
            Token::EndTag(name) => {
                // Find matching open tag
                self.close_tag(&name);
            }
            Token::Text(text) => {
                // Skip whitespace-only text at root level
                if text.trim().is_empty() && self.stack.len() <= 1 {
                    return;
                }

                let text_id = self.doc.create_text(&text);
                let parent = *self.stack.last().unwrap_or(&self.doc.root());
                let _ = self.doc.append_child(parent, text_id);
            }
            Token::Comment(text) => {
                let comment_id = self.doc.create_comment(&text);
                let parent = *self.stack.last().unwrap_or(&self.doc.root());
                let _ = self.doc.append_child(parent, comment_id);
            }
        }
    }

    fn close_tag(&mut self, name: &str) {
        // Find the matching open tag in the stack
        let pos = self.stack.iter().rposition(|&id| {
            self.doc
                .get(id)
                .and_then(|n| n.tag())
                .map(|t| t.eq_ignore_ascii_case(name))
                .unwrap_or(false)
        });

        if let Some(pos) = pos {
            // Close all tags up to and including the matching one
            self.stack.truncate(pos);
        }
        // If no matching tag found, ignore (lenient parsing)
    }

    fn auto_close_tags(&mut self, new_tag: &str) {
        // Auto-close certain tags when specific new tags are opened
        let new_tag_lower = new_tag.to_lowercase();

        loop {
            let should_close = if let Some(&current_id) = self.stack.last() {
                if let Some(current) = self.doc.get(current_id) {
                    if let Some(current_tag) = current.tag() {
                        should_auto_close(current_tag, &new_tag_lower)
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };

            if should_close {
                self.stack.pop();
            } else {
                break;
            }
        }
    }
}

/// Check if a tag should auto-close when another tag is opened.
fn should_auto_close(current: &str, new: &str) -> bool {
    match current.to_lowercase().as_str() {
        "p" => matches!(
            new,
            "address"
                | "article"
                | "aside"
                | "blockquote"
                | "div"
                | "dl"
                | "fieldset"
                | "footer"
                | "form"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "header"
                | "hr"
                | "main"
                | "nav"
                | "ol"
                | "p"
                | "pre"
                | "section"
                | "table"
                | "ul"
        ),
        "li" => new == "li",
        "dt" | "dd" => new == "dt" || new == "dd",
        "tr" => new == "tr",
        "td" | "th" => new == "td" || new == "th" || new == "tr",
        "option" => new == "option" || new == "optgroup",
        "optgroup" => new == "optgroup",
        "thead" | "tbody" | "tfoot" => new == "thead" || new == "tbody" || new == "tfoot",
        _ => false,
    }
}

/// Check if a tag is void (self-closing).
fn is_void_element(tag: &str) -> bool {
    matches!(
        tag.to_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let doc = parse_html("<div>Hello</div>");
        let divs = doc.find_by_tag("div");
        assert_eq!(divs.len(), 1);
        assert_eq!(doc.text_content(divs[0]), "Hello");
    }

    #[test]
    fn test_parse_nested() {
        let doc = parse_html("<div><span>A</span><span>B</span></div>");
        let spans = doc.find_by_tag("span");
        assert_eq!(spans.len(), 2);
    }

    #[test]
    fn test_parse_attributes() {
        let doc = parse_html(r#"<div id="main" class="container">Content</div>"#);
        let div_id = doc.find_by_id("main").unwrap();
        let div = doc.get(div_id).unwrap();
        assert!(div.has_class("container"));
    }

    #[test]
    fn test_parse_unclosed_tags() {
        // Lenient parsing - unclosed tags should be handled
        let doc = parse_html("<div><p>First<p>Second</div>");
        let paragraphs = doc.find_by_tag("p");
        assert_eq!(paragraphs.len(), 2);
    }

    #[test]
    fn test_parse_void_elements() {
        let doc = parse_html("<div><br><hr><img src='test.png'></div>");
        assert_eq!(doc.find_by_tag("br").len(), 1);
        assert_eq!(doc.find_by_tag("hr").len(), 1);
        assert_eq!(doc.find_by_tag("img").len(), 1);
    }

    #[test]
    fn test_parse_malformed() {
        // Missing closing tags
        let doc = parse_html("<div><span>text");
        assert_eq!(doc.find_by_tag("div").len(), 1);
        assert_eq!(doc.find_by_tag("span").len(), 1);
    }
}
