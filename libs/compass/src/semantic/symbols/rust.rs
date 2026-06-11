//! Rust symbol extraction visitor methods
//!
//! Implements R1-R7 from spec rust-symbol-analysis:
//! - R1: Extract Rust Functions (function_item)
//! - R2: Extract Rust Structs (struct_item)
//! - R3: Extract Rust Traits (trait_item)
//! - R4: Extract Rust Impls (impl_item)
//! - R5: Extract Rust Constants (const_item, static_item)
//! - R6: Extract Rust Doc Comments (///, //!)
//! - R7: Parse Rust Types into TypeInfo

use crate::diagnostic::Range;
use crate::syntax::ParsedFile;

use super::{SymbolKind, SymbolTableBuilder, TypeInfo};

impl SymbolTableBuilder {
    pub(crate) fn visit_rust_node(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        if node.is_error() || node.is_missing() {
            return;
        }

        match node.kind() {
            "function_item" | "function_signature_item" => {
                self.visit_rust_function(node, file);
                return;
            }
            "struct_item" => {
                self.visit_rust_struct(node, file);
                return;
            }
            "enum_item" => {
                self.visit_rust_enum(node, file);
                return;
            }
            "trait_item" => {
                self.visit_rust_trait(node, file);
                return;
            }
            "impl_item" => {
                self.visit_rust_impl(node, file);
                return;
            }
            "const_item" => {
                self.visit_rust_const(node, file, SymbolKind::Const);
                return;
            }
            "static_item" => {
                self.visit_rust_const(node, file, SymbolKind::Static);
                return;
            }
            "type_item" => {
                self.visit_rust_type_alias(node, file);
                return;
            }
            "mod_item" => {
                self.visit_rust_mod(node, file);
                return;
            }
            "use_declaration" => {
                self.visit_rust_use(node, file);
                return;
            }
            "macro_definition" => {
                self.visit_rust_macro(node, file);
                return;
            }
            _ => {}
        }

        // Recurse children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if !child.is_error() && !child.is_missing() {
                self.visit_rust_node(&child, file);
            }
        }
    }

    /// R1: Extract function definitions
    fn visit_rust_function(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let name_node = node.child_by_field_name("name");
        let name = name_node
            .map(|n| file.node_text(&n).to_string())
            .unwrap_or_default();
        let location = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();

        // R7: Parse return type
        let return_type = node
            .child_by_field_name("return_type")
            .map(|n| TypeInfo::from_rust_type(file.node_text(&n)));

        // R6: Extract doc comments
        let doc = extract_rust_doc_comments(node, file);

        self.table.add_symbol(
            name,
            SymbolKind::Function,
            location,
            return_type,
            doc,
            self.current_scope,
        );

        // Enter function scope for parameters
        self.push_scope();

        if let Some(params) = node.child_by_field_name("parameters") {
            self.visit_rust_parameters(&params, file);
        }

        // Visit body for nested items
        if let Some(body) = node.child_by_field_name("body") {
            self.visit_rust_node(&body, file);
        }

        self.pop_scope();
    }

    /// R2: Extract struct definitions
    fn visit_rust_struct(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let name_node = node.child_by_field_name("name");
        let name = name_node
            .map(|n| file.node_text(&n).to_string())
            .unwrap_or_default();
        let location = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();

        let doc = extract_rust_doc_comments(node, file);

        self.table.add_symbol(
            name,
            SymbolKind::Struct,
            location,
            None,
            doc,
            self.current_scope,
        );

        // Enter struct scope for fields
        self.push_scope();

        if let Some(body) = node.child_by_field_name("body") {
            self.visit_rust_struct_fields(&body, file);
        }

        self.pop_scope();
    }

    fn visit_rust_struct_fields(&mut self, body: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            if child.kind() == "field_declaration" {
                let name_node = child.child_by_field_name("name");
                let name = name_node
                    .map(|n| file.node_text(&n).to_string())
                    .unwrap_or_default();
                let location = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();

                let type_info = child
                    .child_by_field_name("type")
                    .map(|n| TypeInfo::from_rust_type(file.node_text(&n)));

                self.table.add_symbol(
                    name,
                    SymbolKind::Variable,
                    location,
                    type_info,
                    None,
                    self.current_scope,
                );
            }
        }
    }

    /// Extract enum definitions
    fn visit_rust_enum(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let name_node = node.child_by_field_name("name");
        let name = name_node
            .map(|n| file.node_text(&n).to_string())
            .unwrap_or_default();
        let location = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();

        let doc = extract_rust_doc_comments(node, file);

        self.table.add_symbol(
            name,
            SymbolKind::Enum,
            location,
            None,
            doc,
            self.current_scope,
        );

        // Enter enum scope for variants
        self.push_scope();

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "enum_variant" {
                    let vname_node = child.child_by_field_name("name");
                    let vname = vname_node
                        .map(|n| file.node_text(&n).to_string())
                        .unwrap_or_default();
                    let vlocation = vname_node.map(|n| Range::from_node(&n)).unwrap_or_default();

                    self.table.add_symbol(
                        vname,
                        SymbolKind::EnumMember,
                        vlocation,
                        None,
                        None,
                        self.current_scope,
                    );
                }
            }
        }

        self.pop_scope();
    }

    /// R3: Extract trait definitions
    fn visit_rust_trait(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let name_node = node.child_by_field_name("name");
        let name = name_node
            .map(|n| file.node_text(&n).to_string())
            .unwrap_or_default();
        let location = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();

        let doc = extract_rust_doc_comments(node, file);

        self.table.add_symbol(
            name,
            SymbolKind::Trait,
            location,
            None,
            doc,
            self.current_scope,
        );

        // Enter trait scope for methods
        self.push_scope();

        if let Some(body) = node.child_by_field_name("body") {
            self.visit_rust_node(&body, file);
        }

        self.pop_scope();
    }

    /// R4: Extract impl blocks
    fn visit_rust_impl(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        // Build impl name: "TypeName" or "TraitName for TypeName"
        let name = build_impl_name(node, file);
        let location = node
            .child_by_field_name("type")
            .map(|n| Range::from_node(&n))
            .unwrap_or_else(|| Range::from_node(node));

        let doc = extract_rust_doc_comments(node, file);

        self.table.add_symbol(
            name,
            SymbolKind::Impl,
            location,
            None,
            doc,
            self.current_scope,
        );

        // Enter impl scope for methods
        self.push_scope();

        if let Some(body) = node.child_by_field_name("body") {
            self.visit_rust_node(&body, file);
        }

        self.pop_scope();
    }

    /// R5: Extract const/static items
    fn visit_rust_const(
        &mut self,
        node: &tree_sitter::Node<'_>,
        file: &ParsedFile,
        kind: SymbolKind,
    ) {
        let name_node = node.child_by_field_name("name");
        let name = name_node
            .map(|n| file.node_text(&n).to_string())
            .unwrap_or_default();
        let location = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();

        let type_info = node
            .child_by_field_name("type")
            .map(|n| TypeInfo::from_rust_type(file.node_text(&n)));

        let doc = extract_rust_doc_comments(node, file);

        self.table
            .add_symbol(name, kind, location, type_info, doc, self.current_scope);
    }

    /// Extract type alias definitions
    fn visit_rust_type_alias(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let name_node = node.child_by_field_name("name");
        let name = name_node
            .map(|n| file.node_text(&n).to_string())
            .unwrap_or_default();
        let location = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();

        let doc = extract_rust_doc_comments(node, file);

        self.table.add_symbol(
            name,
            SymbolKind::TypeAlias,
            location,
            None,
            doc,
            self.current_scope,
        );
    }

    /// Extract module declarations
    fn visit_rust_mod(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let name_node = node.child_by_field_name("name");
        let name = name_node
            .map(|n| file.node_text(&n).to_string())
            .unwrap_or_default();
        let location = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();

        let doc = extract_rust_doc_comments(node, file);

        self.table.add_symbol(
            name,
            SymbolKind::Module,
            location,
            None,
            doc,
            self.current_scope,
        );

        // If inline module (has body), enter scope and visit body
        if let Some(body) = node.child_by_field_name("body") {
            self.push_scope();
            self.visit_rust_node(&body, file);
            self.pop_scope();
        }
    }

    /// Extract use declarations
    fn visit_rust_use(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        // Extract the argument (use path)
        let arg = node.child_by_field_name("argument");
        if let Some(arg_node) = arg {
            let text = file.node_text(&arg_node).to_string();
            // Extract the last segment as the imported name
            let import_name = extract_use_name(&text);
            if !import_name.is_empty() {
                self.table.add_symbol(
                    import_name,
                    SymbolKind::Import,
                    Range::from_node(&arg_node),
                    None,
                    None,
                    self.current_scope,
                );
            }
        }
    }

    /// Extract macro definitions
    fn visit_rust_macro(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let name_node = node.child_by_field_name("name");
        let name = name_node
            .map(|n| file.node_text(&n).to_string())
            .unwrap_or_default();
        let location = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();

        let doc = extract_rust_doc_comments(node, file);

        self.table.add_symbol(
            name,
            SymbolKind::Macro,
            location,
            None,
            doc,
            self.current_scope,
        );
    }

    /// Extract function parameters
    fn visit_rust_parameters(&mut self, params: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let mut cursor = params.walk();
        for child in params.children(&mut cursor) {
            match child.kind() {
                "parameter" => {
                    // Regular parameter: pattern: type
                    let name_node = child.child_by_field_name("pattern");
                    let name = name_node
                        .map(|n| file.node_text(&n).to_string())
                        .unwrap_or_default();
                    let location = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();

                    let type_info = child
                        .child_by_field_name("type")
                        .map(|n| TypeInfo::from_rust_type(file.node_text(&n)));

                    if !name.is_empty() {
                        self.table.add_symbol(
                            name,
                            SymbolKind::Parameter,
                            location,
                            type_info,
                            None,
                            self.current_scope,
                        );
                    }
                }
                "self_parameter" => {
                    self.table.add_symbol(
                        "self".to_string(),
                        SymbolKind::Parameter,
                        Range::from_node(&child),
                        None,
                        None,
                        self.current_scope,
                    );
                }
                _ => {}
            }
        }
    }
}

/// R6: Extract doc comments from preceding siblings
fn extract_rust_doc_comments(node: &tree_sitter::Node<'_>, file: &ParsedFile) -> Option<String> {
    let mut doc_lines = Vec::new();
    let mut sibling = node.prev_sibling();

    // Walk backwards through preceding siblings collecting doc comments
    while let Some(sib) = sibling {
        let kind = sib.kind();
        if kind == "line_comment" {
            let text = file.node_text(&sib);
            if let Some(doc_text) = text.strip_prefix("///") {
                doc_lines.push(doc_text.strip_prefix(' ').unwrap_or(doc_text).to_string());
            } else if text.starts_with("//!") {
                // Inner doc comment — stop collecting outer docs
                break;
            } else {
                // Regular comment — stop
                break;
            }
        } else if kind == "attribute_item" || kind == "inner_attribute_item" {
            // Skip attributes between doc comments and item
            sibling = sib.prev_sibling();
            continue;
        } else {
            break;
        }
        sibling = sib.prev_sibling();
    }

    if doc_lines.is_empty() {
        return None;
    }

    // Reverse because we collected backwards
    doc_lines.reverse();
    Some(doc_lines.join("\n").trim().to_string())
}

/// Build impl name from node: "TypeName" or "TraitName for TypeName"
fn build_impl_name(node: &tree_sitter::Node<'_>, file: &ParsedFile) -> String {
    let type_name = node
        .child_by_field_name("type")
        .map(|n| file.node_text(&n).to_string())
        .unwrap_or_default();

    let trait_name = node
        .child_by_field_name("trait")
        .map(|n| file.node_text(&n).to_string());

    match trait_name {
        Some(t) => format!("{} for {}", t, type_name),
        None => type_name,
    }
}

/// Extract the imported name from a use path
fn extract_use_name(path: &str) -> String {
    // Handle "as alias" rename
    if let Some((_, alias)) = path.rsplit_once(" as ") {
        return alias.trim().to_string();
    }
    // Handle glob imports
    if path.ends_with("::*") || path.contains('{') {
        return String::new();
    }
    // Take last segment
    path.rsplit("::").next().unwrap_or("").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::super::SymbolTableBuilder;
    use crate::syntax::{Language, MultiParser};

    #[test]
    fn test_rust_struct_and_impl() {
        let source = r#"
/// A simple struct
struct MyStruct {
    value: i32,
}

impl MyStruct {
    /// Creates a new instance
    fn new(value: i32) -> Self {
        MyStruct { value }
    }

    fn process(&self) -> i32 {
        self.value * 2
    }
}
"#;
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Rust).unwrap();
        let table = SymbolTableBuilder::new().build_rust(&parsed);
        let symbols = table.all_symbols();

        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(
            names.contains(&"MyStruct"),
            "missing MyStruct, got: {:?}",
            names
        );
        assert!(names.contains(&"value"), "missing field 'value'");
        assert!(names.contains(&"new"), "missing method 'new'");
        assert!(names.contains(&"process"), "missing method 'process'");

        // Check doc comment on struct
        let my_struct = symbols.iter().find(|s| s.name == "MyStruct").unwrap();
        assert_eq!(my_struct.doc.as_deref(), Some("A simple struct"));

        // Check doc comment on new()
        let new_fn = symbols.iter().find(|s| s.name == "new").unwrap();
        assert_eq!(new_fn.doc.as_deref(), Some("Creates a new instance"));
    }

    #[test]
    fn test_rust_function_with_return_type() {
        let source = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Rust).unwrap();
        let table = SymbolTableBuilder::new().build_rust(&parsed);
        let symbols = table.all_symbols();

        let add_fn = symbols.iter().find(|s| s.name == "add").unwrap();
        assert!(add_fn.type_info.is_some());

        // Check parameters
        let params: Vec<&str> = symbols
            .iter()
            .filter(|s| s.kind == super::super::SymbolKind::Parameter && s.name != "self")
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            params.contains(&"a"),
            "missing param 'a', got: {:?}",
            params
        );
        assert!(
            params.contains(&"b"),
            "missing param 'b', got: {:?}",
            params
        );
    }

    #[test]
    fn test_rust_trait() {
        let source = r#"
/// A display trait
trait Displayable {
    fn display(&self) -> String;
}
"#;
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Rust).unwrap();
        let table = SymbolTableBuilder::new().build_rust(&parsed);
        let symbols = table.all_symbols();

        let trait_sym = symbols.iter().find(|s| s.name == "Displayable").unwrap();
        assert_eq!(trait_sym.kind, super::super::SymbolKind::Trait);
        assert_eq!(trait_sym.doc.as_deref(), Some("A display trait"));

        let display_fn = symbols.iter().find(|s| s.name == "display");
        assert!(
            display_fn.is_some(),
            "missing 'display' method in trait, got: {:?}",
            symbols
                .iter()
                .map(|s| (&s.name, s.kind))
                .collect::<Vec<_>>()
        );
        assert_eq!(display_fn.unwrap().kind, super::super::SymbolKind::Function);
    }

    #[test]
    fn test_rust_enum() {
        let source = r#"
enum Color {
    Red,
    Green,
    Blue,
}
"#;
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Rust).unwrap();
        let table = SymbolTableBuilder::new().build_rust(&parsed);
        let symbols = table.all_symbols();

        let enum_sym = symbols.iter().find(|s| s.name == "Color").unwrap();
        assert_eq!(enum_sym.kind, super::super::SymbolKind::Enum);

        let variants: Vec<&str> = symbols
            .iter()
            .filter(|s| s.kind == super::super::SymbolKind::EnumMember)
            .map(|s| s.name.as_str())
            .collect();
        assert_eq!(variants.len(), 3);
        assert!(variants.contains(&"Red"));
        assert!(variants.contains(&"Green"));
        assert!(variants.contains(&"Blue"));
    }

    #[test]
    fn test_rust_const_and_static() {
        let source = r#"
const MAX_SIZE: usize = 100;
static COUNTER: i32 = 0;
"#;
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Rust).unwrap();
        let table = SymbolTableBuilder::new().build_rust(&parsed);
        let symbols = table.all_symbols();

        let max_size = symbols.iter().find(|s| s.name == "MAX_SIZE").unwrap();
        assert_eq!(max_size.kind, super::super::SymbolKind::Const);
        assert!(max_size.type_info.is_some());

        let counter = symbols.iter().find(|s| s.name == "COUNTER").unwrap();
        assert_eq!(counter.kind, super::super::SymbolKind::Static);
    }

    #[test]
    fn test_rust_mod_and_use() {
        let source = r#"
mod utils;
use std::collections::HashMap;
"#;
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Rust).unwrap();
        let table = SymbolTableBuilder::new().build_rust(&parsed);
        let symbols = table.all_symbols();

        let mod_sym = symbols.iter().find(|s| s.name == "utils").unwrap();
        assert_eq!(mod_sym.kind, super::super::SymbolKind::Module);

        let import_sym = symbols.iter().find(|s| s.name == "HashMap").unwrap();
        assert_eq!(import_sym.kind, super::super::SymbolKind::Import);
    }

    #[test]
    fn test_rust_macro_definition() {
        let source = r#"
macro_rules! my_macro {
    () => {};
}
"#;
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Rust).unwrap();
        let table = SymbolTableBuilder::new().build_rust(&parsed);
        let symbols = table.all_symbols();

        let mac = symbols.iter().find(|s| s.name == "my_macro").unwrap();
        assert_eq!(mac.kind, super::super::SymbolKind::Macro);
    }

    #[test]
    fn test_rust_error_recovery() {
        let source = r#"
fn valid_fn() -> i32 {
    42
}

fn broken( {
    0
}

struct ValidStruct {
    x: i32,
}
"#;
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Rust).unwrap();
        let table = SymbolTableBuilder::new().build_rust(&parsed);
        let symbols = table.all_symbols();

        // Should still extract valid symbols despite parse errors
        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(
            names.contains(&"valid_fn"),
            "missing valid_fn, got: {:?}",
            names
        );
        assert!(
            names.contains(&"ValidStruct"),
            "missing ValidStruct, got: {:?}",
            names
        );
    }

    #[test]
    fn test_rust_impl_trait_for_type() {
        let source = r#"
struct Foo;

trait Bar {
    fn do_thing(&self);
}

impl Bar for Foo {
    fn do_thing(&self) {}
}
"#;
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Rust).unwrap();
        let table = SymbolTableBuilder::new().build_rust(&parsed);
        let symbols = table.all_symbols();

        let impl_sym = symbols
            .iter()
            .find(|s| s.kind == super::super::SymbolKind::Impl)
            .unwrap();
        assert_eq!(impl_sym.name, "Bar for Foo");
    }

    #[test]
    fn test_rust_scoping() {
        let source = r#"
fn outer() {
    fn inner() {}
}
"#;
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Rust).unwrap();
        let table = SymbolTableBuilder::new().build_rust(&parsed);
        let symbols = table.all_symbols();

        let outer = symbols.iter().find(|s| s.name == "outer").unwrap();
        let inner = symbols.iter().find(|s| s.name == "inner").unwrap();
        assert_ne!(
            outer.scope_id, inner.scope_id,
            "inner should be in a different scope"
        );
    }
}
