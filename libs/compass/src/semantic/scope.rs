//! Scope analysis for Python code
//!
//! Tracks variable definitions and usages across scopes to detect:
//! - Unused variables (PY103)
//! - Undefined names (PY105)
//! - Variable redeclaration (PY106)

use crate::diagnostic::Range;
use crate::syntax::ParsedFile;
use std::collections::HashMap;

/// Kind of symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Variable,
    Parameter,
    Function,
    Class,
    Import,
    Global,
    Nonlocal,
}

/// A symbol in a scope
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub defined_at: Range,
    pub used: bool,
    pub assigned_multiple: bool,
}

/// Kind of scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Module,
    Function,
    Class,
    Comprehension,
    Lambda,
}

/// A scope containing symbols
#[derive(Debug)]
pub struct Scope {
    pub kind: ScopeKind,
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<usize>, // Index of parent scope
}

impl Scope {
    pub fn new(kind: ScopeKind, parent: Option<usize>) -> Self {
        Self {
            kind,
            symbols: HashMap::new(),
            parent,
        }
    }

    pub fn define(&mut self, name: String, kind: SymbolKind, range: Range) {
        if let Some(existing) = self.symbols.get_mut(&name) {
            existing.assigned_multiple = true;
        } else {
            self.symbols.insert(
                name.clone(),
                Symbol {
                    name,
                    kind,
                    defined_at: range,
                    used: false,
                    assigned_multiple: false,
                },
            );
        }
    }

    pub fn mark_used(&mut self, name: &str) -> bool {
        if let Some(symbol) = self.symbols.get_mut(name) {
            symbol.used = true;
            true
        } else {
            false
        }
    }
}

/// Scope analyzer for Python
pub struct ScopeAnalyzer {
    scopes: Vec<Scope>,
    current_scope: usize,
}

impl ScopeAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            scopes: Vec::new(),
            current_scope: 0,
        };
        // Create module scope
        analyzer.scopes.push(Scope::new(ScopeKind::Module, None));
        analyzer
    }

    /// Analyze a parsed Python file
    pub fn analyze(&mut self, file: &ParsedFile) {
        self.visit_node(&file.root_node(), file);
    }

    fn push_scope(&mut self, kind: ScopeKind) {
        let parent = Some(self.current_scope);
        self.scopes.push(Scope::new(kind, parent));
        self.current_scope = self.scopes.len() - 1;
    }

    fn pop_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current_scope].parent {
            self.current_scope = parent;
        }
    }

    fn current(&mut self) -> &mut Scope {
        &mut self.scopes[self.current_scope]
    }

    fn visit_node(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        match node.kind() {
            "function_definition" | "async_function_definition" => {
                // Define function name in current scope
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = file.node_text(&name_node).to_string();
                    self.current()
                        .define(name, SymbolKind::Function, Range::from_node(&name_node));
                }

                // Check if this is a stub function (body is just ... or pass)
                let is_stub = node
                    .child_by_field_name("body")
                    .map_or(false, |body| Self::is_stub_body(&body, file));

                // Create new scope for function body
                self.push_scope(ScopeKind::Function);

                // Process parameters (skip for stub functions)
                if !is_stub {
                    if let Some(params) = node.child_by_field_name("parameters") {
                        self.visit_parameters(&params, file);
                    }
                }

                // Visit body
                if let Some(body) = node.child_by_field_name("body") {
                    self.visit_children(&body, file);
                }

                self.pop_scope();
                return; // Don't visit children again
            }

            "class_definition" => {
                // Define class name in current scope
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = file.node_text(&name_node).to_string();
                    self.current()
                        .define(name, SymbolKind::Class, Range::from_node(&name_node));
                }

                // Create new scope for class body
                self.push_scope(ScopeKind::Class);

                // Visit body
                if let Some(body) = node.child_by_field_name("body") {
                    self.visit_children(&body, file);
                }

                self.pop_scope();
                return;
            }

            "lambda" => {
                self.push_scope(ScopeKind::Lambda);

                if let Some(params) = node.child_by_field_name("parameters") {
                    self.visit_parameters(&params, file);
                }

                if let Some(body) = node.child_by_field_name("body") {
                    self.visit_node(&body, file);
                }

                self.pop_scope();
                return;
            }

            "list_comprehension"
            | "set_comprehension"
            | "dictionary_comprehension"
            | "generator_expression" => {
                self.push_scope(ScopeKind::Comprehension);
                self.visit_children(node, file);
                self.pop_scope();
                return;
            }

            "for_in_clause" => {
                // Loop variable in comprehension
                if let Some(left) = node.child_by_field_name("left") {
                    self.define_pattern(&left, file);
                }
            }

            "assignment" | "augmented_assignment" => {
                // Define variables on left side
                if let Some(left) = node.child_by_field_name("left") {
                    self.define_pattern(&left, file);
                }
                // Mark uses on right side
                if let Some(right) = node.child_by_field_name("right") {
                    self.visit_node(&right, file);
                }
                return;
            }

            "for_statement" => {
                // Loop variable
                if let Some(left) = node.child_by_field_name("left") {
                    self.define_pattern(&left, file);
                }
            }

            "except_clause" => {
                // Exception variable: except Exception as e
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "as_pattern" {
                        if let Some(alias) = child.child_by_field_name("alias") {
                            let name = file.node_text(&alias).to_string();
                            self.current().define(
                                name,
                                SymbolKind::Variable,
                                Range::from_node(&alias),
                            );
                        }
                    }
                }
            }

            "with_item" => {
                // with open() as f
                if let Some(alias) = node.child_by_field_name("alias") {
                    self.define_pattern(&alias, file);
                }
            }

            "import_statement" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "dotted_name" {
                        let name = file.node_text(&child);
                        let base_name = name.split('.').next().unwrap_or(name);
                        self.current().define(
                            base_name.to_string(),
                            SymbolKind::Import,
                            Range::from_node(&child),
                        );
                    } else if child.kind() == "aliased_import" {
                        if let Some(alias) = child.child_by_field_name("alias") {
                            let name = file.node_text(&alias).to_string();
                            self.current().define(
                                name,
                                SymbolKind::Import,
                                Range::from_node(&alias),
                            );
                        } else if let Some(name_node) = child.child_by_field_name("name") {
                            let name = file.node_text(&name_node).to_string();
                            self.current().define(
                                name,
                                SymbolKind::Import,
                                Range::from_node(&name_node),
                            );
                        }
                    }
                }
                return;
            }

            "import_from_statement" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "dotted_name" {
                        // Skip module name, we want imported names
                        continue;
                    }
                    if child.kind() == "aliased_import" {
                        if let Some(alias) = child.child_by_field_name("alias") {
                            let name = file.node_text(&alias).to_string();
                            self.current().define(
                                name,
                                SymbolKind::Import,
                                Range::from_node(&alias),
                            );
                        } else if let Some(name_node) = child.child_by_field_name("name") {
                            let name = file.node_text(&name_node).to_string();
                            self.current().define(
                                name,
                                SymbolKind::Import,
                                Range::from_node(&name_node),
                            );
                        }
                    }
                }
                return;
            }

            "global_statement" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        let name = file.node_text(&child).to_string();
                        self.current()
                            .define(name, SymbolKind::Global, Range::from_node(&child));
                    }
                }
                return;
            }

            "nonlocal_statement" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        let name = file.node_text(&child).to_string();
                        self.current()
                            .define(name, SymbolKind::Nonlocal, Range::from_node(&child));
                    }
                }
                return;
            }

            "identifier" => {
                // This is a use of an identifier
                let name = file.node_text(node);
                self.mark_used_in_scope(name);
                return;
            }

            _ => {}
        }

        // Visit children
        self.visit_children(node, file);
    }

    fn visit_children(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(&child, file);
        }
    }

    fn visit_parameters(&mut self, params: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let mut cursor = params.walk();
        for child in params.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    let name = file.node_text(&child).to_string();
                    self.current()
                        .define(name, SymbolKind::Parameter, Range::from_node(&child));
                }
                "typed_parameter" | "typed_default_parameter" | "default_parameter" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = file.node_text(&name_node).to_string();
                        self.current().define(
                            name,
                            SymbolKind::Parameter,
                            Range::from_node(&name_node),
                        );
                    }
                }
                "list_splat_pattern" | "dictionary_splat_pattern" => {
                    let mut inner_cursor = child.walk();
                    for inner in child.children(&mut inner_cursor) {
                        if inner.kind() == "identifier" {
                            let name = file.node_text(&inner).to_string();
                            self.current().define(
                                name,
                                SymbolKind::Parameter,
                                Range::from_node(&inner),
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn define_pattern(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        match node.kind() {
            "identifier" => {
                let name = file.node_text(node).to_string();
                self.current()
                    .define(name, SymbolKind::Variable, Range::from_node(node));
            }
            "tuple_pattern" | "list_pattern" | "pattern_list" | "tuple" | "list" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.define_pattern(&child, file);
                }
            }
            "subscript" | "attribute" => {
                // x[0] = ... or x.y = ... - visit right side for uses
                // but don't define the subscript/attribute itself
            }
            _ => {}
        }
    }

    fn mark_used_in_scope(&mut self, name: &str) {
        // Try current scope first, then walk up
        let mut scope_idx = self.current_scope;
        loop {
            if self.scopes[scope_idx].mark_used(name) {
                return;
            }
            match self.scopes[scope_idx].parent {
                Some(parent) => scope_idx = parent,
                None => return,
            }
        }
    }

    /// Get all unused symbols (for PY103)
    pub fn unused_symbols(&self) -> Vec<&Symbol> {
        let mut unused = Vec::new();
        for scope in &self.scopes {
            for symbol in scope.symbols.values() {
                // Skip if used, or if it's a special name
                if symbol.used {
                    continue;
                }
                // Skip _ (intentionally unused)
                if symbol.name == "_" || symbol.name.starts_with('_') {
                    continue;
                }
                // Skip self/cls
                if symbol.name == "self" || symbol.name == "cls" {
                    continue;
                }
                // Only report unused variables and parameters
                if matches!(symbol.kind, SymbolKind::Variable | SymbolKind::Parameter) {
                    unused.push(symbol);
                }
            }
        }
        unused
    }

    /// Get symbols that were redeclared (for PY106)
    pub fn redeclared_symbols(&self) -> Vec<&Symbol> {
        let mut redeclared = Vec::new();
        for scope in &self.scopes {
            for symbol in scope.symbols.values() {
                if symbol.assigned_multiple && matches!(symbol.kind, SymbolKind::Variable) {
                    redeclared.push(symbol);
                }
            }
        }
        redeclared
    }

    /// Check if a function body is a stub (only contains ... or pass, optionally with docstring)
    fn is_stub_body(body: &tree_sitter::Node<'_>, file: &ParsedFile) -> bool {
        let mut cursor = body.walk();
        let children: Vec<_> = body.children(&mut cursor).collect();

        // Empty body
        if children.is_empty() {
            return true;
        }

        // Check each statement in the body
        let mut has_real_code = false;
        for child in &children {
            match child.kind() {
                // Skip docstrings
                "expression_statement" => {
                    if let Some(expr) = child.child(0) {
                        if expr.kind() == "string" {
                            // This is a docstring, continue checking
                            continue;
                        } else if expr.kind() == "ellipsis" {
                            // ... is a stub marker
                            continue;
                        }
                    }
                    has_real_code = true;
                }
                // pass is a stub marker
                "pass_statement" => continue,
                // raise NotImplementedError is also a stub pattern
                "raise_statement" => {
                    let text = file.node_text(child);
                    if text.contains("NotImplementedError") {
                        continue;
                    }
                    has_real_code = true;
                }
                // Comments are OK
                "comment" => continue,
                // Anything else is real code
                _ => {
                    has_real_code = true;
                }
            }
        }

        !has_real_code
    }
}

impl Default for ScopeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
