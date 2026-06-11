//! Python symbol extraction visitor methods

use crate::diagnostic::Range;
use crate::syntax::ParsedFile;

use super::{SymbolKind, SymbolTableBuilder, TypeInfo};

impl SymbolTableBuilder {
    pub(crate) fn visit_python_node(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        // Error recovery: skip ERROR nodes but continue processing siblings
        if node.is_error() || node.is_missing() {
            return;
        }

        match node.kind() {
            "function_definition" | "async_function_definition" => {
                self.visit_python_function(node, file);
                return;
            }
            "class_definition" => {
                self.visit_python_class(node, file);
                return;
            }
            "assignment" => {
                self.visit_python_assignment(node, file);
            }
            "identifier" => {
                // This is a reference to a symbol
                let name = file.node_text(node);
                if let Some(symbols) = self.table.by_name.get(name) {
                    if let Some(&id) = symbols.last() {
                        self.table.add_reference(id, Range::from_node(node));
                    }
                }
                return;
            }
            _ => {}
        }

        // Visit children (with error recovery - skip error nodes)
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            // Skip ERROR nodes at any level
            if !child.is_error() && !child.is_missing() {
                self.visit_python_node(&child, file);
            }
        }
    }

    fn visit_python_function(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        // Get function name
        let name_node = node.child_by_field_name("name");
        let name = name_node
            .map(|n| file.node_text(&n).to_string())
            .unwrap_or_default();
        let location = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();

        // Get return type annotation
        let return_type = node.child_by_field_name("return_type").map(|n| {
            let type_str = file.node_text(&n);
            TypeInfo::from_python_annotation(type_str)
        });

        // Get docstring
        let doc = self.extract_python_docstring(node, file);

        // Add function symbol
        self.table.add_symbol(
            name,
            SymbolKind::Function,
            location,
            return_type,
            doc,
            self.current_scope,
        );

        // Enter function scope
        self.push_scope();

        // Process parameters
        if let Some(params) = node.child_by_field_name("parameters") {
            self.visit_python_parameters(&params, file);
        }

        // Process body
        if let Some(body) = node.child_by_field_name("body") {
            self.visit_python_node(&body, file);
        }

        self.pop_scope();
    }

    fn visit_python_class(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let name_node = node.child_by_field_name("name");
        let name = name_node
            .map(|n| file.node_text(&n).to_string())
            .unwrap_or_default();
        let location = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();

        let doc = self.extract_python_docstring(node, file);

        self.table.add_symbol(
            name,
            SymbolKind::Class,
            location,
            None,
            doc,
            self.current_scope,
        );

        // Enter class scope
        self.push_scope();

        if let Some(body) = node.child_by_field_name("body") {
            self.visit_python_node(&body, file);
        }

        self.pop_scope();
    }

    fn visit_python_assignment(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        if let Some(left) = node.child_by_field_name("left") {
            if left.kind() == "identifier" {
                let name = file.node_text(&left).to_string();
                let location = Range::from_node(&left);

                // Try to get type annotation
                let type_info = node
                    .child_by_field_name("type")
                    .map(|n| TypeInfo::from_python_annotation(file.node_text(&n)));

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

        // Visit right side for references
        if let Some(right) = node.child_by_field_name("right") {
            self.visit_python_node(&right, file);
        }
    }

    fn visit_python_parameters(&mut self, params: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let mut cursor = params.walk();
        for child in params.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    let name = file.node_text(&child).to_string();
                    self.table.add_symbol(
                        name,
                        SymbolKind::Parameter,
                        Range::from_node(&child),
                        None,
                        None,
                        self.current_scope,
                    );
                }
                "typed_parameter" | "typed_default_parameter" | "default_parameter" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = file.node_text(&name_node).to_string();
                        let type_info = child
                            .child_by_field_name("type")
                            .map(|n| TypeInfo::from_python_annotation(file.node_text(&n)));

                        self.table.add_symbol(
                            name,
                            SymbolKind::Parameter,
                            Range::from_node(&name_node),
                            type_info,
                            None,
                            self.current_scope,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    fn extract_python_docstring(
        &self,
        node: &tree_sitter::Node<'_>,
        file: &ParsedFile,
    ) -> Option<String> {
        let body = node.child_by_field_name("body")?;
        let mut cursor = body.walk();
        let first_child = body.children(&mut cursor).next()?;

        if first_child.kind() == "expression_statement" {
            if let Some(expr) = first_child.child(0) {
                if expr.kind() == "string" {
                    let text = file.node_text(&expr);
                    // Strip quotes
                    let doc = text
                        .trim_start_matches("\"\"\"")
                        .trim_start_matches("'''")
                        .trim_end_matches("\"\"\"")
                        .trim_end_matches("'''")
                        .trim();
                    return Some(doc.to_string());
                }
            }
        }
        None
    }
}
