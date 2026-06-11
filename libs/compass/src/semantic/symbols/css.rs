//! CSS symbol extraction (tree-sitter)
//!
//! Extracts symbols from CSS stylesheets:
//! - Class selectors (.name) as Class
//! - ID selectors (#name) as Variable
//! - Custom properties (--var-name) as Variable
//! - @keyframes names as Function
//! - @media descriptors as Label

use super::{SymbolKind, SymbolTableBuilder};
use crate::diagnostic::Range;
use crate::syntax::ParsedFile;

impl SymbolTableBuilder {
    /// Walk CSS AST to extract symbols
    pub(crate) fn visit_css_node(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        if node.is_error() || node.is_missing() {
            return;
        }

        match node.kind() {
            "class_selector" => {
                // .class-name — the class_name child holds the name
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = file.node_text(&name_node);
                    if !name.is_empty() {
                        self.table.add_symbol(
                            name.to_string(),
                            SymbolKind::Class,
                            Range::from_node(&name_node),
                            None,
                            Some("CSS class selector".to_string()),
                            self.current_scope,
                        );
                    }
                } else {
                    // Fallback: extract from text after the dot
                    let text = file.node_text(node);
                    if let Some(name) = text.strip_prefix('.') {
                        if !name.is_empty() {
                            self.table.add_symbol(
                                name.to_string(),
                                SymbolKind::Class,
                                Range::from_node(node),
                                None,
                                Some("CSS class selector".to_string()),
                                self.current_scope,
                            );
                        }
                    }
                }
            }
            "id_selector" => {
                // #id-name — the id child or text after #
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = file.node_text(&name_node);
                    if !name.is_empty() {
                        self.table.add_symbol(
                            name.to_string(),
                            SymbolKind::Variable,
                            Range::from_node(&name_node),
                            None,
                            Some("CSS ID selector".to_string()),
                            self.current_scope,
                        );
                    }
                } else {
                    let text = file.node_text(node);
                    if let Some(name) = text.strip_prefix('#') {
                        if !name.is_empty() {
                            self.table.add_symbol(
                                name.to_string(),
                                SymbolKind::Variable,
                                Range::from_node(node),
                                None,
                                Some("CSS ID selector".to_string()),
                                self.current_scope,
                            );
                        }
                    }
                }
            }
            "declaration" => {
                // Check for custom properties (--var-name: value)
                self.extract_css_custom_property(node, file);
            }
            "keyframes_statement" => {
                // @keyframes animation-name { ... }
                self.extract_css_keyframes(node, file);
            }
            "media_statement" => {
                // @media descriptor
                self.extract_css_media(node, file);
            }
            _ => {}
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if !child.is_error() && !child.is_missing() {
                self.visit_css_node(&child, file);
            }
        }
    }

    /// Extract custom property (CSS variable) from a declaration node
    fn extract_css_custom_property(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        // declaration has a property_name child
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "property_name" {
                let prop = file.node_text(&child);
                if prop.starts_with("--") {
                    self.table.add_symbol(
                        prop.to_string(),
                        SymbolKind::Variable,
                        Range::from_node(&child),
                        None,
                        Some("CSS custom property".to_string()),
                        self.current_scope,
                    );
                }
                break;
            }
        }
    }

    /// Extract @keyframes name
    fn extract_css_keyframes(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = file.node_text(&name_node).trim();
            if !name.is_empty() {
                self.table.add_symbol(
                    name.to_string(),
                    SymbolKind::Function,
                    Range::from_node(&name_node),
                    None,
                    Some("CSS @keyframes animation".to_string()),
                    self.current_scope,
                );
            }
        } else {
            // Fallback: second child is often the name
            let mut cursor = node.walk();
            let children: Vec<_> = node.children(&mut cursor).collect();
            for child in &children {
                if child.kind() == "keyframes_name" || child.kind() == "identifier" {
                    let name = file.node_text(child).trim();
                    if !name.is_empty() {
                        self.table.add_symbol(
                            name.to_string(),
                            SymbolKind::Function,
                            Range::from_node(child),
                            None,
                            Some("CSS @keyframes animation".to_string()),
                            self.current_scope,
                        );
                    }
                    break;
                }
            }
        }
    }

    /// Extract @media descriptor as label
    fn extract_css_media(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        // Collect the feature query text between @media and {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            // tree-sitter-css: the query part is typically a "feature_query" or similar
            if matches!(
                child.kind(),
                "feature_query"
                    | "media_query_list"
                    | "parenthesized_query"
                    | "binary_query"
                    | "unary_query"
                    | "keyword_query"
            ) {
                let desc = file.node_text(&child).trim();
                if !desc.is_empty() {
                    self.table.add_symbol(
                        desc.to_string(),
                        SymbolKind::Label,
                        Range::from_node(&child),
                        None,
                        Some("CSS @media query".to_string()),
                        self.current_scope,
                    );
                }
                break;
            }
        }
    }

    /// Build symbol table for a CSS file
    pub fn build_css(mut self, file: &ParsedFile) -> super::SymbolTable {
        self.visit_css_node(&file.root_node(), file);
        self.table
    }
}

#[cfg(test)]
mod tests {
    use super::super::{SymbolKind, SymbolTableBuilder};
    use crate::syntax::{Language, MultiParser};

    fn build(source: &str) -> super::super::SymbolTable {
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Css).unwrap();
        SymbolTableBuilder::new().build_css(&parsed)
    }

    #[test]
    fn test_css_class_selectors() {
        let table = build(".container { width: 100%; }\n.flex { display: flex; }");
        let classes: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Class)
            .map(|s| s.name.as_str())
            .collect();
        assert!(classes.contains(&"container"), "got: {:?}", classes);
        assert!(classes.contains(&"flex"), "got: {:?}", classes);
    }

    #[test]
    fn test_css_id_selectors() {
        let table = build("#main { color: red; }\n#sidebar { width: 300px; }");
        let ids: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| {
                s.kind == SymbolKind::Variable && s.doc.as_deref() == Some("CSS ID selector")
            })
            .map(|s| s.name.as_str())
            .collect();
        assert!(ids.contains(&"main"), "got: {:?}", ids);
        assert!(ids.contains(&"sidebar"), "got: {:?}", ids);
    }

    #[test]
    fn test_css_custom_properties() {
        let table = build(":root { --primary-color: blue; --font-size: 16px; }");
        let vars: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.doc.as_deref() == Some("CSS custom property"))
            .map(|s| s.name.as_str())
            .collect();
        assert!(vars.contains(&"--primary-color"), "got: {:?}", vars);
        assert!(vars.contains(&"--font-size"), "got: {:?}", vars);
    }
}
