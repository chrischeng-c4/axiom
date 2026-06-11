//! Go symbol extraction visitor methods
//!
//! Extracts symbols from Go AST nodes:
//! - Package declarations (package_clause) → Module
//! - Functions (function_declaration) → Function
//! - Methods (method_declaration) → Function
//! - Types (type_declaration → type_spec) → Class/Struct/Interface
//! - Constants (const_declaration) → Const
//! - Variables (var_declaration) → Variable
//! - Imports (import_declaration) → Import

use crate::diagnostic::Range;
use crate::syntax::ParsedFile;

use super::{SymbolKind, SymbolTableBuilder, TypeInfo};

impl SymbolTableBuilder {
    pub(crate) fn visit_go_node(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        if node.is_error() || node.is_missing() {
            return;
        }

        match node.kind() {
            "package_clause" => {
                self.visit_go_package(node, file);
                return;
            }
            "function_declaration" => {
                self.visit_go_function(node, file);
                return;
            }
            "method_declaration" => {
                self.visit_go_method(node, file);
                return;
            }
            "type_declaration" => {
                self.visit_go_type_declaration(node, file);
                return;
            }
            "const_declaration" => {
                self.visit_go_const_declaration(node, file);
                return;
            }
            "var_declaration" => {
                self.visit_go_var_declaration(node, file);
                return;
            }
            "import_declaration" => {
                self.visit_go_import_declaration(node, file);
                return;
            }
            _ => {}
        }

        // Recurse children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if !child.is_error() && !child.is_missing() {
                self.visit_go_node(&child, file);
            }
        }
    }

    /// Extract package declaration
    fn visit_go_package(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "package_identifier" {
                let name = file.node_text(&child).to_string();
                let location = Range::from_node(&child);
                self.table.add_symbol(
                    name,
                    SymbolKind::Module,
                    location,
                    None,
                    None,
                    self.current_scope,
                );
                return;
            }
        }
    }

    /// Extract function declarations
    fn visit_go_function(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let name_node = node.child_by_field_name("name");
        let name = name_node
            .map(|n| file.node_text(&n).to_string())
            .unwrap_or_default();
        let location = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();

        let return_type = self.extract_go_return_type(node, file);
        let doc = extract_go_doc_comment(node, file);

        self.table.add_symbol(
            name,
            SymbolKind::Function,
            location,
            return_type,
            doc,
            self.current_scope,
        );

        // Enter function scope for parameters and body
        self.push_scope();

        if let Some(params) = node.child_by_field_name("parameters") {
            self.visit_go_parameters(&params, file);
        }

        if let Some(body) = node.child_by_field_name("body") {
            self.visit_go_node(&body, file);
        }

        self.pop_scope();
    }

    /// Extract method declarations
    fn visit_go_method(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let name_node = node.child_by_field_name("name");
        let name = name_node
            .map(|n| file.node_text(&n).to_string())
            .unwrap_or_default();
        let location = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();

        let return_type = self.extract_go_return_type(node, file);
        let doc = extract_go_doc_comment(node, file);

        self.table.add_symbol(
            name,
            SymbolKind::Function,
            location,
            return_type,
            doc,
            self.current_scope,
        );

        // Enter method scope
        self.push_scope();

        // Visit receiver as parameter
        if let Some(receiver) = node.child_by_field_name("receiver") {
            self.visit_go_parameters(&receiver, file);
        }

        if let Some(params) = node.child_by_field_name("parameters") {
            self.visit_go_parameters(&params, file);
        }

        if let Some(body) = node.child_by_field_name("body") {
            self.visit_go_node(&body, file);
        }

        self.pop_scope();
    }

    /// Extract type declarations (struct, interface, type alias)
    fn visit_go_type_declaration(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_spec" {
                self.visit_go_type_spec(&child, file);
            }
        }
    }

    fn visit_go_type_spec(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let name_node = node.child_by_field_name("name");
        let name = name_node
            .map(|n| file.node_text(&n).to_string())
            .unwrap_or_default();
        let location = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();

        let doc = extract_go_doc_comment(node, file).or_else(|| {
            // type_spec is inside type_declaration, try parent
            node.parent().and_then(|p| extract_go_doc_comment(&p, file))
        });

        // Determine kind from the type expression
        let type_node = node.child_by_field_name("type");
        let kind = match type_node.as_ref().map(|n| n.kind()) {
            Some("struct_type") => SymbolKind::Struct,
            Some("interface_type") => SymbolKind::Interface,
            _ => SymbolKind::Class, // type alias or other
        };

        self.table
            .add_symbol(name, kind, location, None, doc, self.current_scope);

        // Enter type scope for fields/methods
        if let Some(type_node) = type_node {
            self.push_scope();
            match type_node.kind() {
                "struct_type" => {
                    self.visit_go_struct_fields(&type_node, file);
                }
                "interface_type" => {
                    self.visit_go_interface_methods(&type_node, file);
                }
                _ => {}
            }
            self.pop_scope();
        }
    }

    fn visit_go_struct_fields(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        if let Some(field_list) = node.child_by_field_name("body") {
            let mut cursor = field_list.walk();
            for child in field_list.children(&mut cursor) {
                if child.kind() == "field_declaration" {
                    let name_node = child.child_by_field_name("name");
                    let name = name_node
                        .map(|n| file.node_text(&n).to_string())
                        .unwrap_or_default();
                    let location = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();

                    let type_info = child
                        .child_by_field_name("type")
                        .map(|n| parse_go_type(file.node_text(&n)));

                    if !name.is_empty() {
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
        }
    }

    fn visit_go_interface_methods(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "method_spec" {
                let name_node = child.child_by_field_name("name");
                let name = name_node
                    .map(|n| file.node_text(&n).to_string())
                    .unwrap_or_default();
                let location = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();

                if !name.is_empty() {
                    self.table.add_symbol(
                        name,
                        SymbolKind::Function,
                        location,
                        None,
                        None,
                        self.current_scope,
                    );
                }
            }
        }
    }

    /// Extract const declarations
    fn visit_go_const_declaration(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "const_spec" {
                let name_node = child.child_by_field_name("name");
                let name = name_node
                    .map(|n| file.node_text(&n).to_string())
                    .unwrap_or_default();
                let location = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();

                let type_info = child
                    .child_by_field_name("type")
                    .map(|n| parse_go_type(file.node_text(&n)));

                let doc = extract_go_doc_comment(&child, file)
                    .or_else(|| extract_go_doc_comment(node, file));

                if !name.is_empty() {
                    self.table.add_symbol(
                        name,
                        SymbolKind::Const,
                        location,
                        type_info,
                        doc,
                        self.current_scope,
                    );
                }
            }
        }
    }

    /// Extract var declarations
    fn visit_go_var_declaration(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "var_spec" {
                let name_node = child.child_by_field_name("name");
                let name = name_node
                    .map(|n| file.node_text(&n).to_string())
                    .unwrap_or_default();
                let location = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();

                let type_info = child
                    .child_by_field_name("type")
                    .map(|n| parse_go_type(file.node_text(&n)));

                if !name.is_empty() {
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
    }

    /// Extract import declarations
    fn visit_go_import_declaration(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "import_spec" {
                let path_node = child.child_by_field_name("path");
                if let Some(path) = path_node {
                    let path_text = file.node_text(&path).trim_matches('"').to_string();
                    // Use alias if present, otherwise last segment of path
                    let name = child
                        .child_by_field_name("name")
                        .map(|n| file.node_text(&n).to_string())
                        .unwrap_or_else(|| {
                            path_text
                                .rsplit('/')
                                .next()
                                .unwrap_or(&path_text)
                                .to_string()
                        });

                    if name != "_" && name != "." {
                        self.table.add_symbol(
                            name,
                            SymbolKind::Import,
                            Range::from_node(&path),
                            None,
                            None,
                            self.current_scope,
                        );
                    }
                }
            }
        }
    }

    /// Extract function parameters
    fn visit_go_parameters(&mut self, params: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let mut cursor = params.walk();
        for child in params.children(&mut cursor) {
            if child.kind() == "parameter_declaration" {
                let name_node = child.child_by_field_name("name");
                let name = name_node
                    .map(|n| file.node_text(&n).to_string())
                    .unwrap_or_default();
                let location = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();

                let type_info = child
                    .child_by_field_name("type")
                    .map(|n| parse_go_type(file.node_text(&n)));

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
        }
    }

    /// Extract return type from function/method result field
    fn extract_go_return_type(
        &self,
        node: &tree_sitter::Node<'_>,
        file: &ParsedFile,
    ) -> Option<TypeInfo> {
        node.child_by_field_name("result")
            .map(|n| parse_go_type(file.node_text(&n)))
    }
}

/// Extract doc comment from preceding sibling (Go uses // comments)
fn extract_go_doc_comment(node: &tree_sitter::Node<'_>, file: &ParsedFile) -> Option<String> {
    let mut doc_lines = Vec::new();
    let mut sibling = node.prev_sibling();

    while let Some(sib) = sibling {
        if sib.kind() == "comment" {
            let text = file.node_text(&sib);
            if let Some(doc_text) = text.strip_prefix("//") {
                doc_lines.push(doc_text.strip_prefix(' ').unwrap_or(doc_text).to_string());
            } else {
                break;
            }
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

/// Parse Go type string into TypeInfo
fn parse_go_type(type_str: &str) -> TypeInfo {
    let type_str = type_str.trim();

    if type_str.is_empty() {
        return TypeInfo::Unknown;
    }

    // Handle pointer types
    if let Some(inner) = type_str.strip_prefix('*') {
        return TypeInfo::Reference(Box::new(parse_go_type(inner)));
    }

    // Handle slice types
    if let Some(inner) = type_str.strip_prefix("[]") {
        return TypeInfo::List(Box::new(parse_go_type(inner)));
    }

    // Handle map types
    if type_str.starts_with("map[") {
        if let Some(bracket_end) = type_str.find(']') {
            let key = &type_str[4..bracket_end];
            let value = &type_str[bracket_end + 1..];
            return TypeInfo::Dict(Box::new(parse_go_type(key)), Box::new(parse_go_type(value)));
        }
    }

    // Handle Go primitives
    match type_str {
        "int" | "int8" | "int16" | "int32" | "int64" | "uint" | "uint8" | "uint16" | "uint32"
        | "uint64" | "uintptr" | "float32" | "float64" | "complex64" | "complex128" | "bool"
        | "byte" | "rune" | "string" => TypeInfo::Primitive(type_str.to_string()),
        "error" => TypeInfo::Named("error".to_string()),
        _ => TypeInfo::Named(type_str.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::super::SymbolTableBuilder;
    use crate::syntax::{Language, MultiParser};

    #[test]
    fn test_go_function_and_types() {
        let source = r#"
package main

import "fmt"

// Add adds two integers.
func Add(a int, b int) int {
    return a + b
}

// Server represents an HTTP server.
type Server struct {
    Host string
    Port int
}

// Handler is the request handler interface.
type Handler interface {
    ServeHTTP(w ResponseWriter, r *Request)
}

const MaxRetries = 3

var defaultTimeout int = 30
"#;
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Go).unwrap();
        let table = SymbolTableBuilder::new().build_go(&parsed);
        let symbols = table.all_symbols();

        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(
            names.contains(&"main"),
            "missing package 'main', got: {:?}",
            names
        );
        assert!(
            names.contains(&"Add"),
            "missing function 'Add', got: {:?}",
            names
        );
        assert!(
            names.contains(&"Server"),
            "missing struct 'Server', got: {:?}",
            names
        );
        assert!(
            names.contains(&"Handler"),
            "missing interface 'Handler', got: {:?}",
            names
        );
        assert!(
            names.contains(&"MaxRetries"),
            "missing const 'MaxRetries', got: {:?}",
            names
        );
        assert!(
            names.contains(&"defaultTimeout"),
            "missing var 'defaultTimeout', got: {:?}",
            names
        );
        assert!(
            names.contains(&"fmt"),
            "missing import 'fmt', got: {:?}",
            names
        );

        // Check doc comment on Add
        let add_fn = symbols.iter().find(|s| s.name == "Add").unwrap();
        assert_eq!(add_fn.doc.as_deref(), Some("Add adds two integers."));

        // Check struct kind
        let server = symbols.iter().find(|s| s.name == "Server").unwrap();
        assert_eq!(server.kind, super::super::SymbolKind::Struct);

        // Check interface kind
        let handler = symbols.iter().find(|s| s.name == "Handler").unwrap();
        assert_eq!(handler.kind, super::super::SymbolKind::Interface);

        // Check const kind
        let max_retries = symbols.iter().find(|s| s.name == "MaxRetries").unwrap();
        assert_eq!(max_retries.kind, super::super::SymbolKind::Const);
    }

    #[test]
    fn test_go_method_declaration() {
        let source = r#"
package main

type MyStruct struct {
    Value int
}

// Process does something useful.
func (s *MyStruct) Process() error {
    return nil
}
"#;
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Go).unwrap();
        let table = SymbolTableBuilder::new().build_go(&parsed);
        let symbols = table.all_symbols();

        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(
            names.contains(&"Process"),
            "missing method 'Process', got: {:?}",
            names
        );

        let process = symbols.iter().find(|s| s.name == "Process").unwrap();
        assert_eq!(process.kind, super::super::SymbolKind::Function);
        assert_eq!(
            process.doc.as_deref(),
            Some("Process does something useful.")
        );
    }

    #[test]
    fn test_go_type_parsing() {
        assert_eq!(
            super::parse_go_type("int"),
            super::super::TypeInfo::Primitive("int".to_string())
        );
        assert_eq!(
            super::parse_go_type("string"),
            super::super::TypeInfo::Primitive("string".to_string())
        );
    }
}
