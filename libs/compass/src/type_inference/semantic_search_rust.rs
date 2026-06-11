//! Rust-specific semantic search support
//!
//! Adds Rust language support to the unified semantic search API.

use std::collections::HashMap;
use std::path::PathBuf;

use tree_sitter::Node;

use super::mutable_ast::Span;
use super::semantic_search::{CallSite, MatchKind, SearchMatch};

/// Rust semantic search provider
pub struct RustSearchProvider;

impl RustSearchProvider {
    pub fn new() -> Self {
        Self
    }

    /// Extract docstrings from Rust code (/// and //! comments)
    pub fn extract_docstrings(&self, root: &Node, source: &str) -> HashMap<String, String> {
        let mut docstrings = HashMap::new();
        self.visit_for_docstrings(root, source, &mut docstrings);
        docstrings
    }

    fn visit_for_docstrings(
        &self,
        node: &Node,
        source: &str,
        docstrings: &mut HashMap<String, String>,
    ) {
        match node.kind() {
            "function_item" | "struct_item" | "enum_item" | "trait_item" | "impl_item" => {
                // Get name
                if let Some(name_node) = node.child_by_field_name("name") {
                    let symbol_name = &source[name_node.start_byte()..name_node.end_byte()];

                    // Look for doc comments before this node
                    if let Some(doc) = self.extract_rust_doc_comment(node, source) {
                        docstrings.insert(symbol_name.to_string(), doc);
                    }
                }
            }
            _ => {}
        }

        // Visit children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_for_docstrings(&child, source, docstrings);
        }
    }

    /// Extract Rust doc comment (/// or //!) before a node
    fn extract_rust_doc_comment(&self, node: &Node, source: &str) -> Option<String> {
        let mut comments = Vec::new();
        let mut prev = node.prev_sibling();

        while let Some(sibling) = prev {
            if sibling.kind() == "line_comment" {
                let text = &source[sibling.start_byte()..sibling.end_byte()];
                if text.starts_with("///") || text.starts_with("//!") {
                    // Doc comment - prepend (we're going backwards)
                    let cleaned = text
                        .trim_start_matches("///")
                        .trim_start_matches("//!")
                        .trim();
                    comments.insert(0, cleaned.to_string());
                } else {
                    // Regular comment - stop
                    break;
                }
            } else if sibling.kind() == "block_comment" {
                let text = &source[sibling.start_byte()..sibling.end_byte()];
                if text.starts_with("/**") || text.starts_with("/*!") {
                    // Block doc comment
                    let cleaned = self.clean_rust_block_comment(text);
                    comments.insert(0, cleaned);
                }
                break;
            } else {
                break;
            }
            prev = sibling.prev_sibling();
        }

        if comments.is_empty() {
            None
        } else {
            Some(comments.join("\n"))
        }
    }

    /// Clean Rust block comment (/** ... */ or /*! ... */)
    fn clean_rust_block_comment(&self, raw: &str) -> String {
        let trimmed = raw.trim();

        // Remove /** or /*! and */
        let inner = if trimmed.starts_with("/**") || trimmed.starts_with("/*!") {
            &trimmed[3..trimmed.len() - 2]
        } else {
            trimmed
        };

        // Clean each line - remove leading * and whitespace
        inner
            .lines()
            .map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with('*') {
                    trimmed[1..].trim()
                } else {
                    trimmed
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    }

    /// Build call graph for Rust code
    pub fn build_call_graph(&self, root: &Node, source: &str, file: &PathBuf) -> Vec<CallSite> {
        let mut call_sites = Vec::new();
        let mut current_function: Option<String> = None;
        self.visit_for_calls(root, source, &mut current_function, file, &mut call_sites);
        call_sites
    }

    fn visit_for_calls(
        &self,
        node: &Node,
        source: &str,
        current_function: &mut Option<String>,
        file: &PathBuf,
        call_sites: &mut Vec<CallSite>,
    ) {
        match node.kind() {
            "function_item" => {
                // Enter function context
                if let Some(name_node) = node.child_by_field_name("name") {
                    let func_name = &source[name_node.start_byte()..name_node.end_byte()];
                    let prev_function = current_function.clone();
                    *current_function = Some(func_name.to_string());

                    // Visit body
                    if let Some(body) = node.child_by_field_name("body") {
                        self.visit_for_calls(&body, source, current_function, file, call_sites);
                    }

                    // Restore previous function context
                    *current_function = prev_function;
                    return;
                }
            }
            "impl_item" => {
                // For impl blocks, visit methods
                if let Some(body) = node.child_by_field_name("body") {
                    let mut cursor = body.walk();
                    for child in body.children(&mut cursor) {
                        self.visit_for_calls(&child, source, current_function, file, call_sites);
                    }
                }
                return;
            }
            "call_expression" => {
                // Extract callee
                if let Some(func_node) = node.child_by_field_name("function") {
                    let callee_name = self.extract_rust_callee(&func_node, source);

                    if let Some(ref caller) = current_function {
                        call_sites.push(CallSite {
                            file: file.clone(),
                            span: Span {
                                start: node.start_byte(),
                                end: node.end_byte(),
                                start_line: node.start_position().row,
                                start_col: node.start_position().column,
                                end_line: node.end_position().row,
                                end_col: node.end_position().column,
                            },
                            callee: callee_name,
                            caller: caller.clone(),
                        });
                    }
                }
            }
            "method_call_expression" => {
                // Extract method name (last identifier before arguments)
                if let Some(name_node) = node.child_by_field_name("name") {
                    let method_name = &source[name_node.start_byte()..name_node.end_byte()];

                    if let Some(ref caller) = current_function {
                        call_sites.push(CallSite {
                            file: file.clone(),
                            span: Span {
                                start: node.start_byte(),
                                end: node.end_byte(),
                                start_line: node.start_position().row,
                                start_col: node.start_position().column,
                                end_line: node.end_position().row,
                                end_col: node.end_position().column,
                            },
                            callee: method_name.to_string(),
                            caller: caller.clone(),
                        });
                    }
                }
            }
            _ => {}
        }

        // Visit children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_for_calls(&child, source, current_function, file, call_sites);
        }
    }

    /// Extract Rust callee name from call expression
    fn extract_rust_callee(&self, node: &Node, source: &str) -> String {
        match node.kind() {
            "identifier" => source[node.start_byte()..node.end_byte()].to_string(),
            "scoped_identifier" => {
                // For path::to::function, get just the function name
                if let Some(name) = node.child_by_field_name("name") {
                    source[name.start_byte()..name.end_byte()].to_string()
                } else {
                    source[node.start_byte()..node.end_byte()].to_string()
                }
            }
            "field_expression" => {
                // For obj.field, get the field name
                if let Some(field) = node.child_by_field_name("field") {
                    source[field.start_byte()..field.end_byte()].to_string()
                } else {
                    source[node.start_byte()..node.end_byte()].to_string()
                }
            }
            _ => source[node.start_byte()..node.end_byte()].to_string(),
        }
    }

    /// Find usages of a symbol in Rust code
    pub fn find_usages(
        &self,
        root: &Node,
        source: &str,
        symbol: &str,
        file: &PathBuf,
    ) -> Vec<SearchMatch> {
        let mut matches = Vec::new();
        self.visit_for_usages(root, source, symbol, file, &mut matches);
        matches
    }

    fn visit_for_usages(
        &self,
        node: &Node,
        source: &str,
        symbol: &str,
        file: &PathBuf,
        matches: &mut Vec<SearchMatch>,
    ) {
        match node.kind() {
            "identifier" => {
                let text = &source[node.start_byte()..node.end_byte()];
                if text == symbol {
                    matches.push(SearchMatch {
                        file: file.clone(),
                        span: Span {
                            start: node.start_byte(),
                            end: node.end_byte(),
                            start_line: node.start_position().row,
                            start_col: node.start_position().column,
                            end_line: node.end_position().row,
                            end_col: node.end_position().column,
                        },
                        symbol: Some(symbol.to_string()),
                        kind: self.classify_rust_usage(node),
                        score: 1.0,
                        context: None,
                    });
                }
            }
            "type_identifier" => {
                let text = &source[node.start_byte()..node.end_byte()];
                if text == symbol {
                    matches.push(SearchMatch {
                        file: file.clone(),
                        span: Span {
                            start: node.start_byte(),
                            end: node.end_byte(),
                            start_line: node.start_position().row,
                            start_col: node.start_position().column,
                            end_line: node.end_position().row,
                            end_col: node.end_position().column,
                        },
                        symbol: Some(symbol.to_string()),
                        kind: MatchKind::TypeAnnotation,
                        score: 1.0,
                        context: None,
                    });
                }
            }
            _ => {}
        }

        // Visit children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_for_usages(&child, source, symbol, file, matches);
        }
    }

    /// Classify Rust usage context
    fn classify_rust_usage(&self, node: &Node) -> MatchKind {
        if let Some(parent) = node.parent() {
            match parent.kind() {
                "function_item" => MatchKind::FunctionDef,
                "struct_item" | "enum_item" | "trait_item" => MatchKind::ClassDef,
                "call_expression" | "method_call_expression" => MatchKind::Call,
                "let_declaration" => MatchKind::VariableAssignment,
                "use_declaration" => MatchKind::Import,
                "type_identifier" | "generic_type" => MatchKind::TypeAnnotation,
                _ => MatchKind::VariableAssignment,
            }
        } else {
            MatchKind::VariableAssignment
        }
    }

    /// Find trait implementations
    pub fn find_implementations(
        &self,
        root: &Node,
        source: &str,
        trait_name: &str,
        file: &PathBuf,
    ) -> Vec<SearchMatch> {
        let mut matches = Vec::new();
        self.visit_for_impls(root, source, trait_name, file, &mut matches);
        matches
    }

    fn visit_for_impls(
        &self,
        node: &Node,
        source: &str,
        trait_name: &str,
        file: &PathBuf,
        matches: &mut Vec<SearchMatch>,
    ) {
        if node.kind() == "impl_item" {
            // Check if this is an impl for the specified trait
            if let Some(trait_node) = node.child_by_field_name("trait") {
                let impl_trait = &source[trait_node.start_byte()..trait_node.end_byte()];
                // Handle generic traits like Trait<T>
                let base_trait = impl_trait.split('<').next().unwrap_or(impl_trait);

                if base_trait == trait_name {
                    // Get the implementing type
                    let type_name = node
                        .child_by_field_name("type")
                        .map(|t| source[t.start_byte()..t.end_byte()].to_string());

                    matches.push(SearchMatch {
                        file: file.clone(),
                        span: Span {
                            start: node.start_byte(),
                            end: node.end_byte(),
                            start_line: node.start_position().row,
                            start_col: node.start_position().column,
                            end_line: node.end_position().row,
                            end_col: node.end_position().column,
                        },
                        symbol: type_name,
                        kind: MatchKind::ClassDef,
                        score: 1.0,
                        context: None,
                    });
                }
            }
        }

        // Visit children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_for_impls(&child, source, trait_name, file, matches);
        }
    }
}

impl Default for RustSearchProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_rust(code: &str) -> tree_sitter::Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();
        parser.parse(code, None).unwrap()
    }

    #[test]
    fn test_extract_rust_docstrings() {
        let code = r#"
/// This is a documented function
/// with multiple lines
fn documented_fn() {}

struct UndocumentedStruct {}

/// Documented struct
struct DocStruct {
    field: i32,
}
"#;
        let tree = parse_rust(code);
        let provider = RustSearchProvider::new();
        let docs = provider.extract_docstrings(&tree.root_node(), code);

        assert!(docs.contains_key("documented_fn"));
        assert!(docs
            .get("documented_fn")
            .unwrap()
            .contains("documented function"));
        assert!(docs.contains_key("DocStruct"));
        assert!(!docs.contains_key("UndocumentedStruct"));
    }

    #[test]
    fn test_rust_call_graph() {
        let code = r#"
fn caller() {
    callee();
    helper();
}

fn callee() {}
fn helper() {}
"#;
        let tree = parse_rust(code);
        let provider = RustSearchProvider::new();
        let file = PathBuf::from("test.rs");
        let calls = provider.build_call_graph(&tree.root_node(), code, &file);

        assert_eq!(calls.len(), 2);
        assert!(calls
            .iter()
            .any(|c| c.caller == "caller" && c.callee == "callee"));
        assert!(calls
            .iter()
            .any(|c| c.caller == "caller" && c.callee == "helper"));
    }

    #[test]
    fn test_find_rust_usages() {
        let code = r#"
fn foo() {
    let x = bar();
    bar();
}

fn bar() -> i32 { 42 }
"#;
        let tree = parse_rust(code);
        let provider = RustSearchProvider::new();
        let file = PathBuf::from("test.rs");
        let usages = provider.find_usages(&tree.root_node(), code, "bar", &file);

        // Should find: definition, two calls
        assert_eq!(usages.len(), 3);
    }

    #[test]
    fn test_find_trait_implementations() {
        let code = r#"
trait MyTrait {
    fn method(&self);
}

struct Foo;

impl MyTrait for Foo {
    fn method(&self) {}
}

struct Bar;

impl MyTrait for Bar {
    fn method(&self) {}
}

impl Clone for Foo {
    fn clone(&self) -> Self { Foo }
}
"#;
        let tree = parse_rust(code);
        let provider = RustSearchProvider::new();
        let file = PathBuf::from("test.rs");
        let impls = provider.find_implementations(&tree.root_node(), code, "MyTrait", &file);

        assert_eq!(impls.len(), 2);
        let symbols: Vec<_> = impls.iter().filter_map(|m| m.symbol.clone()).collect();
        assert!(symbols.contains(&"Foo".to_string()));
        assert!(symbols.contains(&"Bar".to_string()));
    }
}
