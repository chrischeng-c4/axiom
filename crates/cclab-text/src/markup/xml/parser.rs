//! XML parser with namespace support.

use crate::markup::dom::{Document, NodeId};
use std::collections::HashMap;

/// Namespace scope for tracking xmlns declarations.
#[derive(Debug, Clone, Default)]
struct NamespaceScope {
    /// Default namespace (xmlns="...").
    default_ns: Option<String>,
    /// Prefixed namespaces (xmlns:prefix="...").
    prefixes: HashMap<String, String>,
}

/// XML token types.
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum XmlToken {
    /// Processing instruction <?...?>.
    ProcessingInstruction { target: String, data: String },
    /// Start tag with attributes.
    StartTag {
        name: String,
        prefix: Option<String>,
        attrs: HashMap<String, String>,
        self_closing: bool,
    },
    /// End tag.
    EndTag {
        name: String,
        prefix: Option<String>,
    },
    /// Text content.
    Text(String),
    /// CDATA section.
    CData(String),
    /// Comment.
    Comment(String),
    /// DOCTYPE declaration.
    DocType(String),
}

/// XML tokenizer.
struct XmlTokenizer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> XmlTokenizer<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn tokenize(mut self) -> Vec<XmlToken> {
        let mut tokens = Vec::new();
        while self.pos < self.input.len() {
            if let Some(token) = self.next_token() {
                tokens.push(token);
            }
        }
        tokens
    }

    fn next_token(&mut self) -> Option<XmlToken> {
        if self.pos >= self.input.len() {
            return None;
        }

        // Skip XML declaration (treated as PI)
        if self.starts_with("<?xml") {
            return Some(self.read_processing_instruction());
        }

        if self.starts_with("<?") {
            return Some(self.read_processing_instruction());
        }

        if self.starts_with("<!DOCTYPE") || self.starts_with("<!doctype") {
            return Some(self.read_doctype());
        }

        if self.starts_with("<![CDATA[") {
            return Some(self.read_cdata());
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

    fn current_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap_or('\0')
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.current_char().is_whitespace() {
            self.pos += 1;
        }
    }

    fn read_processing_instruction(&mut self) -> XmlToken {
        self.pos += 2; // Skip "<?"
        let start = self.pos;

        // Read target
        while self.pos < self.input.len() {
            let c = self.current_char();
            if c.is_whitespace() || self.starts_with("?>") {
                break;
            }
            self.pos += 1;
        }
        let target = self.input[start..self.pos].to_string();

        self.skip_whitespace();

        // Read data
        let data_start = self.pos;
        while self.pos < self.input.len() && !self.starts_with("?>") {
            self.pos += 1;
        }
        let data = self.input[data_start..self.pos].trim().to_string();

        if self.starts_with("?>") {
            self.pos += 2;
        }

        XmlToken::ProcessingInstruction { target, data }
    }

    fn read_doctype(&mut self) -> XmlToken {
        self.pos += 9; // Skip "<!DOCTYPE"
        let start = self.pos;

        let mut depth = 0;
        while self.pos < self.input.len() {
            let c = self.current_char();
            if c == '[' {
                depth += 1;
            } else if c == ']' {
                depth -= 1;
            } else if c == '>' && depth == 0 {
                break;
            }
            self.pos += 1;
        }

        let content = self.input[start..self.pos].trim().to_string();
        self.pos += 1; // Skip ">"

        XmlToken::DocType(content)
    }

    fn read_cdata(&mut self) -> XmlToken {
        self.pos += 9; // Skip "<![CDATA["
        let start = self.pos;

        while self.pos < self.input.len() && !self.starts_with("]]>") {
            self.pos += 1;
        }

        let content = self.input[start..self.pos].to_string();
        self.pos += 3; // Skip "]]>"

        XmlToken::CData(content)
    }

    fn read_comment(&mut self) -> XmlToken {
        self.pos += 4; // Skip "<!--"
        let start = self.pos;

        while self.pos < self.input.len() && !self.starts_with("-->") {
            self.pos += 1;
        }

        let content = self.input[start..self.pos].to_string();
        self.pos += 3; // Skip "-->"

        XmlToken::Comment(content)
    }

    fn read_end_tag(&mut self) -> XmlToken {
        self.pos += 2; // Skip "</"
        let start = self.pos;

        while self.pos < self.input.len() {
            let c = self.current_char();
            if c == '>' || c.is_whitespace() {
                break;
            }
            self.pos += 1;
        }

        let full_name = &self.input[start..self.pos];
        let (prefix, name) = split_qname(full_name);

        // Skip to closing >
        while self.pos < self.input.len() && !self.starts_with(">") {
            self.pos += 1;
        }
        self.pos += 1;

        XmlToken::EndTag {
            name: name.to_string(),
            prefix: prefix.map(String::from),
        }
    }

    fn read_start_tag(&mut self) -> XmlToken {
        self.pos += 1; // Skip "<"
        let start = self.pos;

        // Read tag name (including prefix)
        while self.pos < self.input.len() {
            let c = self.current_char();
            if c == '>' || c == '/' || c.is_whitespace() {
                break;
            }
            self.pos += 1;
        }

        let full_name = &self.input[start..self.pos];
        let (prefix, name) = split_qname(full_name);

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

        XmlToken::StartTag {
            name: name.to_string(),
            prefix: prefix.map(String::from),
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

            let name = self.input[name_start..self.pos].to_string();
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
            decode_xml_entities(&value)
        } else {
            // Unquoted value (not valid XML but handle gracefully)
            let start = self.pos;
            while self.pos < self.input.len() {
                let c = self.current_char();
                if c.is_whitespace() || c == '>' || c == '/' {
                    break;
                }
                self.pos += 1;
            }
            decode_xml_entities(&self.input[start..self.pos])
        }
    }

    fn read_text(&mut self) -> XmlToken {
        let start = self.pos;

        while self.pos < self.input.len() && !self.starts_with("<") {
            self.pos += 1;
        }

        let text = decode_xml_entities(&self.input[start..self.pos]);
        XmlToken::Text(text)
    }
}

/// Split a qualified name into (prefix, local_name).
fn split_qname(qname: &str) -> (Option<&str>, &str) {
    if let Some(colon_pos) = qname.find(':') {
        let prefix = &qname[..colon_pos];
        let local = &qname[colon_pos + 1..];
        (Some(prefix), local)
    } else {
        (None, qname)
    }
}

/// Decode XML entities.
fn decode_xml_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// XML parser with namespace support.
struct XmlParser {
    tokens: Vec<XmlToken>,
    pos: usize,
    doc: Document,
    stack: Vec<NodeId>,
    /// Stack of namespace scopes (one per element level).
    ns_scopes: Vec<NamespaceScope>,
}

impl XmlParser {
    fn new(tokens: Vec<XmlToken>) -> Self {
        let doc = Document::new();
        let root = doc.root();
        Self {
            tokens,
            pos: 0,
            doc,
            stack: vec![root],
            ns_scopes: vec![NamespaceScope::default()],
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
            XmlToken::ProcessingInstruction { .. } | XmlToken::DocType(_) => {
                // Skip for now
            }
            XmlToken::StartTag {
                name,
                prefix,
                attrs,
                self_closing,
            } => {
                // Create new namespace scope (clone from parent or create new)
                let mut scope = self.ns_scopes.last().cloned().unwrap_or_default();

                // Extract namespace declarations from attributes
                let mut regular_attrs = HashMap::new();
                for (key, value) in &attrs {
                    if key == "xmlns" {
                        scope.default_ns = Some(value.clone());
                    } else if let Some(ns_prefix) = key.strip_prefix("xmlns:") {
                        scope.prefixes.insert(ns_prefix.to_string(), value.clone());
                    } else {
                        regular_attrs.insert(key.clone(), value.clone());
                    }
                }

                self.ns_scopes.push(scope);

                // Resolve namespace for this element
                let namespace = self.resolve_namespace(prefix.as_deref());

                // Create element
                let element_id = self.doc.create_element(&name);
                if let Some(element) = self.doc.get_mut(element_id) {
                    element.namespace = namespace;
                    element.prefix = prefix.clone();
                    for (key, value) in regular_attrs {
                        element.set_attr(key, value);
                    }
                    // Store xmlns declarations as attributes for serialization
                    for (key, value) in &attrs {
                        if key.starts_with("xmlns") {
                            element.set_attr(key.clone(), value.clone());
                        }
                    }
                }

                // Append to parent
                let parent = *self.stack.last().unwrap_or(&self.doc.root());
                let _ = self.doc.append_child(parent, element_id);

                if !self_closing {
                    self.stack.push(element_id);
                } else {
                    self.ns_scopes.pop();
                }
            }
            XmlToken::EndTag { name, prefix } => {
                // Find matching open tag by local name AND prefix
                // This ensures we close the correct element in mixed-namespace documents
                if let Some(pos) = self.stack.iter().rposition(|&id| {
                    self.doc
                        .get(id)
                        .map(|n| {
                            let name_matches = n.tag().map(|t| t == name).unwrap_or(false);
                            let prefix_matches = n.prefix.as_deref() == prefix.as_deref();
                            name_matches && prefix_matches
                        })
                        .unwrap_or(false)
                }) {
                    // Pop namespace scopes for closed elements
                    let elements_to_close = self.stack.len() - pos;
                    for _ in 0..elements_to_close {
                        self.ns_scopes.pop();
                    }
                    self.stack.truncate(pos);
                }
            }
            XmlToken::Text(text) => {
                if text.trim().is_empty() && self.stack.len() <= 1 {
                    return;
                }
                let text_id = self.doc.create_text(&text);
                let parent = *self.stack.last().unwrap_or(&self.doc.root());
                let _ = self.doc.append_child(parent, text_id);
            }
            XmlToken::CData(text) => {
                // Treat CDATA as text
                let text_id = self.doc.create_text(&text);
                let parent = *self.stack.last().unwrap_or(&self.doc.root());
                let _ = self.doc.append_child(parent, text_id);
            }
            XmlToken::Comment(text) => {
                let comment_id = self.doc.create_comment(&text);
                let parent = *self.stack.last().unwrap_or(&self.doc.root());
                let _ = self.doc.append_child(parent, comment_id);
            }
        }
    }

    fn resolve_namespace(&self, prefix: Option<&str>) -> Option<String> {
        let scope = match self.ns_scopes.last() {
            Some(s) => s,
            None => return None,
        };
        match prefix {
            None => scope.default_ns.clone(),
            Some(p) => scope.prefixes.get(p).cloned(),
        }
    }
}

/// Parse XML string into a Document with namespace support.
pub fn parse_xml_with_ns(xml: &str) -> Document {
    let tokens = XmlTokenizer::new(xml).tokenize();
    XmlParser::new(tokens).parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_xml() {
        let doc = parse_xml_with_ns("<root><item>A</item><item>B</item></root>");
        let items = doc.find_by_tag("item");
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_parse_with_default_namespace() {
        let xml = r#"<root xmlns="http://example.com"><item>A</item></root>"#;
        let doc = parse_xml_with_ns(xml);

        let root = doc.find_by_tag("root");
        assert_eq!(root.len(), 1);

        let node = doc.get(root[0]).unwrap();
        assert_eq!(node.namespace.as_deref(), Some("http://example.com"));
    }

    #[test]
    fn test_parse_with_prefixed_namespace() {
        let xml = r#"<ns:root xmlns:ns="http://example.com"><ns:item>A</ns:item></ns:root>"#;
        let doc = parse_xml_with_ns(xml);

        let root = doc.find_by_tag("root");
        assert_eq!(root.len(), 1);

        let node = doc.get(root[0]).unwrap();
        assert_eq!(node.namespace.as_deref(), Some("http://example.com"));
        assert_eq!(node.prefix.as_deref(), Some("ns"));
    }

    #[test]
    fn test_parse_mixed_namespaces() {
        let xml = r#"
            <root xmlns="http://default.com" xmlns:other="http://other.com">
                <item>Default NS</item>
                <other:item>Other NS</other:item>
            </root>
        "#;
        let doc = parse_xml_with_ns(xml);

        let items = doc.find_by_tag("item");
        assert_eq!(items.len(), 2);

        // First item has default namespace
        let item1 = doc.get(items[0]).unwrap();
        assert_eq!(item1.namespace.as_deref(), Some("http://default.com"));
        assert!(item1.prefix.is_none());

        // Second item has prefixed namespace
        let item2 = doc.get(items[1]).unwrap();
        assert_eq!(item2.namespace.as_deref(), Some("http://other.com"));
        assert_eq!(item2.prefix.as_deref(), Some("other"));
    }

    #[test]
    fn test_parse_cdata() {
        let xml = r#"<root><![CDATA[<not-a-tag>]]></root>"#;
        let doc = parse_xml_with_ns(xml);

        let root = doc.find_by_tag("root")[0];
        assert_eq!(doc.text_content(root), "<not-a-tag>");
    }

    #[test]
    fn test_parse_processing_instruction() {
        let xml = r#"<?xml version="1.0"?><root>content</root>"#;
        let doc = parse_xml_with_ns(xml);

        let root = doc.find_by_tag("root");
        assert_eq!(root.len(), 1);
    }

    #[test]
    fn test_parse_self_closing() {
        let xml = r#"<root><empty/><empty2 /></root>"#;
        let doc = parse_xml_with_ns(xml);

        assert_eq!(doc.find_by_tag("empty").len(), 1);
        assert_eq!(doc.find_by_tag("empty2").len(), 1);
    }

    #[test]
    fn test_prefix_sensitive_end_tags() {
        // Test that end tags correctly match by prefix, not just local name
        let xml = r#"<root xmlns:a="http://a.com" xmlns:b="http://b.com">
            <a:item>A content</a:item>
            <b:item>B content</b:item>
        </root>"#;
        let doc = parse_xml_with_ns(xml);

        let items = doc.find_by_tag("item");
        assert_eq!(items.len(), 2);

        // Verify correct namespace assignment
        let item1 = doc.get(items[0]).unwrap();
        let item2 = doc.get(items[1]).unwrap();

        assert_eq!(item1.namespace.as_deref(), Some("http://a.com"));
        assert_eq!(item1.prefix.as_deref(), Some("a"));
        assert_eq!(item2.namespace.as_deref(), Some("http://b.com"));
        assert_eq!(item2.prefix.as_deref(), Some("b"));
    }

    #[test]
    fn test_nested_same_local_name_different_prefix() {
        // Nested elements with same local name but different prefixes
        let xml = r#"<a:root xmlns:a="http://a.com" xmlns:b="http://b.com">
            <a:item>
                <b:item>nested</b:item>
            </a:item>
        </a:root>"#;
        let doc = parse_xml_with_ns(xml);

        let items = doc.find_by_tag("item");
        assert_eq!(items.len(), 2);

        // Find by namespace
        let a_items = doc.find_by_tag_ns("http://a.com", "item");
        let b_items = doc.find_by_tag_ns("http://b.com", "item");

        assert_eq!(a_items.len(), 1);
        assert_eq!(b_items.len(), 1);
    }
}
