//! HTML symbol extraction (tree-sitter)
//!
//! Extracts symbols from HTML documents:
//! - Element IDs (id="...") as Variable
//! - Class names (class="...") as Class
//! - Form names (name="...") as Variable
//! - Anchor hrefs as references
//! - Meta tag names as Label

use super::{SymbolKind, SymbolTableBuilder};
use crate::diagnostic::Range;
use crate::syntax::ParsedFile;

impl SymbolTableBuilder {
    /// Walk HTML AST to extract symbols
    pub(crate) fn visit_html_node(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        if node.is_error() || node.is_missing() {
            return;
        }

        match node.kind() {
            "element" | "self_closing_tag" => {
                self.extract_html_element(node, file);
                // Continue walking children for nested elements
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if !child.is_error() && !child.is_missing() {
                        self.visit_html_node(&child, file);
                    }
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if !child.is_error() && !child.is_missing() {
                        self.visit_html_node(&child, file);
                    }
                }
            }
        }
    }

    /// Extract symbols from an HTML element node
    fn extract_html_element(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let tag_name = html_tag_name(node, file);

        // Find the start_tag or self_closing_tag child to get attributes
        let attr_parent = find_attr_parent(node);
        let attr_parent = match attr_parent {
            Some(p) => p,
            None => return,
        };

        let mut cursor = attr_parent.walk();
        for child in attr_parent.children(&mut cursor) {
            if child.kind() != "attribute" {
                continue;
            }
            let (attr_name, attr_value, value_range) = match extract_attr(&child, file) {
                Some(v) => v,
                None => continue,
            };

            match attr_name.as_str() {
                "id" => {
                    let clean = strip_quotes(&attr_value);
                    if !clean.is_empty() {
                        self.table.add_symbol(
                            clean.to_string(),
                            SymbolKind::Variable,
                            value_range,
                            None,
                            Some(format!("HTML element ID on <{}>", tag_name)),
                            self.current_scope,
                        );
                    }
                }
                "class" => {
                    let clean = strip_quotes(&attr_value);
                    for class_name in clean.split_whitespace() {
                        if !class_name.is_empty() {
                            self.table.add_symbol(
                                class_name.to_string(),
                                SymbolKind::Class,
                                value_range,
                                None,
                                Some(format!("CSS class on <{}>", tag_name)),
                                self.current_scope,
                            );
                        }
                    }
                }
                "name" if tag_name.eq_ignore_ascii_case("form") => {
                    let clean = strip_quotes(&attr_value);
                    if !clean.is_empty() {
                        self.table.add_symbol(
                            clean.to_string(),
                            SymbolKind::Variable,
                            value_range,
                            None,
                            Some("HTML form name".to_string()),
                            self.current_scope,
                        );
                    }
                }
                "href" if tag_name.eq_ignore_ascii_case("a") => {
                    let clean = strip_quotes(&attr_value);
                    if !clean.is_empty() && !clean.starts_with('#') && clean != "javascript:void(0)"
                    {
                        // Record as a reference (not a definition)
                        // We use add_symbol for visibility in symbol table
                        self.table.add_symbol(
                            clean.to_string(),
                            SymbolKind::Variable,
                            value_range,
                            None,
                            Some("anchor href reference".to_string()),
                            self.current_scope,
                        );
                    }
                }
                "name" if tag_name.eq_ignore_ascii_case("meta") => {
                    let clean = strip_quotes(&attr_value);
                    if !clean.is_empty() {
                        self.table.add_symbol(
                            clean.to_string(),
                            SymbolKind::Label,
                            value_range,
                            None,
                            Some("meta tag name".to_string()),
                            self.current_scope,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    /// Build symbol table for an HTML file
    pub fn build_html(mut self, file: &ParsedFile) -> super::SymbolTable {
        self.visit_html_node(&file.root_node(), file);
        self.table
    }
}

/// Get the tag name from an element or self_closing_tag node
fn html_tag_name<'a>(node: &tree_sitter::Node<'a>, file: &'a ParsedFile) -> &'a str {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if matches!(child.kind(), "start_tag" | "self_closing_tag") {
            let mut ic = child.walk();
            for inner in child.children(&mut ic) {
                if inner.kind() == "tag_name" {
                    return file.node_text(&inner);
                }
            }
        }
        if child.kind() == "tag_name" {
            return file.node_text(&child);
        }
    }
    ""
}

/// Find the start_tag or self_closing_tag child that holds attributes
fn find_attr_parent<'a>(node: &'a tree_sitter::Node<'a>) -> Option<tree_sitter::Node<'a>> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if matches!(child.kind(), "start_tag" | "self_closing_tag") {
            return Some(child);
        }
    }
    // Node itself might be a self_closing_tag
    if node.kind() == "self_closing_tag" {
        return Some(*node);
    }
    None
}

/// Extract attribute name, value text, and value range from an attribute node
fn extract_attr<'a>(
    node: &tree_sitter::Node<'a>,
    file: &'a ParsedFile,
) -> Option<(String, String, Range)> {
    let mut name = String::new();
    let mut value = String::new();
    let mut value_range = Range::default();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "attribute_name" {
            name = file.node_text(&child).to_lowercase();
        }
        if matches!(child.kind(), "quoted_attribute_value" | "attribute_value") {
            value = file.node_text(&child).to_string();
            value_range = Range::from_node(&child);
        }
    }
    if name.is_empty() {
        return None;
    }
    Some((name, value, value_range))
}

/// Strip surrounding quotes from an attribute value
fn strip_quotes(s: &str) -> &str {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::super::{SymbolKind, SymbolTableBuilder};
    use crate::syntax::{Language, MultiParser};

    fn build(source: &str) -> super::super::SymbolTable {
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Html).unwrap();
        SymbolTableBuilder::new().build_html(&parsed)
    }

    #[test]
    fn test_html_ids() {
        let table = build(r#"<div id="main"><span id="title">Hi</span></div>"#);
        let ids: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| {
                s.kind == SymbolKind::Variable
                    && s.doc.as_deref() == Some("HTML element ID on <div>")
                    || s.doc.as_deref() == Some("HTML element ID on <span>")
            })
            .map(|s| s.name.as_str())
            .collect();
        assert!(ids.contains(&"main"), "got: {:?}", ids);
        assert!(ids.contains(&"title"), "got: {:?}", ids);
    }

    #[test]
    fn test_html_classes() {
        let table = build(r#"<div class="container flex"><p class="text-lg">Hi</p></div>"#);
        let classes: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Class)
            .map(|s| s.name.as_str())
            .collect();
        assert!(classes.contains(&"container"), "got: {:?}", classes);
        assert!(classes.contains(&"flex"), "got: {:?}", classes);
        assert!(classes.contains(&"text-lg"), "got: {:?}", classes);
    }

    #[test]
    fn test_html_meta_name() {
        let table = build(r#"<meta name="description" content="A page">"#);
        let labels: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Label)
            .map(|s| s.name.as_str())
            .collect();
        assert!(labels.contains(&"description"), "got: {:?}", labels);
    }

    #[test]
    fn test_html_form_name() {
        let table = build(r#"<form name="login"><input type="text"></form>"#);
        let names: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.doc.as_deref() == Some("HTML form name"))
            .map(|s| s.name.as_str())
            .collect();
        assert!(names.contains(&"login"), "got: {:?}", names);
    }
}
