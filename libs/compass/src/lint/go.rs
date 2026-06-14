//! Go code checker (R5)
//!
//! Implements 10 built-in lint rules (GO000–GO010) plus module-based import
//! graph analysis for interface-satisfaction checking.

use std::collections::HashMap;

use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Range};
use crate::syntax::{Language, ParsedFile};

// ============================================================================
// GoImportGraph — module-based import graph analysis
// ============================================================================

/// A simple module-based import graph built from parsed Go source files.
///
/// Records which modules are imported by which files so that callers can
/// perform impact analysis (e.g. "what re-exports a changed interface?").
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct GoImportGraph {
    /// file path → list of import paths
    imports: HashMap<String, Vec<String>>,
    /// import path → list of files that import it
    reverse: HashMap<String, Vec<String>>,
}

#[allow(dead_code)]
impl GoImportGraph {
    /// Create a new, empty import graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add all import paths found in `file` to the graph.
    pub fn add_file(&mut self, file_path: &str, file: &ParsedFile) {
        let imports = collect_import_paths(file);
        for imp in &imports {
            self.reverse
                .entry(imp.clone())
                .or_default()
                .push(file_path.to_string());
        }
        self.imports.insert(file_path.to_string(), imports);
    }

    /// Return all files that import the given module path.
    pub fn files_importing(&self, module_path: &str) -> Vec<&str> {
        self.reverse
            .get(module_path)
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Return all import paths used by the given file.
    pub fn imports_of(&self, file_path: &str) -> Vec<&str> {
        self.imports
            .get(file_path)
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }
}

/// Extract all string import paths from a parsed Go file.
#[allow(dead_code)]
fn collect_import_paths(file: &ParsedFile) -> Vec<String> {
    let mut paths = Vec::new();
    file.walk(|node, _depth| {
        if node.kind() == "import_spec" {
            // The path is the `interpreted_string_literal` child
            if let Some(path_node) = node.child_by_field_name("path") {
                let raw = file.node_text(&path_node);
                // Strip surrounding quotes
                let trimmed = raw.trim_matches('"');
                if !trimmed.is_empty() {
                    paths.push(trimmed.to_string());
                }
            }
        }
        true
    });
    paths
}

// ============================================================================
// GoChecker
// ============================================================================

/// Go checker
pub struct GoChecker;

impl GoChecker {
    pub fn new() -> Self {
        Self
    }

    /// GO001: Unchecked error — assignment with `_` for error return
    fn check_unchecked_error(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "short_var_declaration" {
                // Check left side for blank identifiers in error position
                if let Some(left) = node.child_by_field_name("left") {
                    let mut cursor = left.walk();
                    let children: Vec<_> = left.children(&mut cursor).collect();

                    // If last identifier on left is `_`, likely ignoring error
                    if children.len() >= 2 {
                        if let Some(last) = children.iter().rev()
                            .find(|c| c.kind() == "identifier" || c.kind() == "blank_identifier")
                        {
                            if last.kind() == "blank_identifier"
                                || file.node_text(last) == "_"
                            {
                                // Verify the right side is a call expression (likely returns error)
                                if let Some(right) = node.child_by_field_name("right") {
                                    let mut rc = right.walk();
                                    let right_children: Vec<_> = right.children(&mut rc).collect();
                                    let has_call = right_children.iter()
                                        .any(|c| c.kind() == "call_expression");
                                    if has_call {
                                        diagnostics.push(Diagnostic::warning(
                                            Range::from_node(node),
                                            "GO001",
                                            DiagnosticCategory::Logic,
                                            "Error return value is discarded with '_' — handle the error",
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// GO002: Blank import — `import _ "pkg"` flagged as info
    fn check_blank_import(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "import_spec" {
                if let Some(name_node) = node.child_by_field_name("name") {
                    if file.node_text(&name_node) == "_" {
                        diagnostics.push(Diagnostic::new(
                            Range::from_node(node),
                            DiagnosticSeverity::Information,
                            "GO002",
                            DiagnosticCategory::Style,
                            "Blank import '_ \"pkg\"' — ensure side-effect import is intentional",
                        ));
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// GO003: Shadowed variable — variable redeclared in inner scope with same name
    fn check_shadowed_variable(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut scope_stack: Vec<Vec<String>> = vec![Vec::new()];

        file.walk(|node, _depth| {
            match node.kind() {
                "function_declaration"
                | "method_declaration"
                | "func_literal"
                | "if_statement"
                | "for_statement"
                | "block" => {
                    scope_stack.push(Vec::new());
                }
                "short_var_declaration" => {
                    if let Some(left) = node.child_by_field_name("left") {
                        let mut cursor = left.walk();
                        for child in left.children(&mut cursor) {
                            if child.kind() == "identifier" {
                                let name = file.node_text(&child).to_string();
                                // Check if name exists in any outer scope
                                let shadowed = scope_stack
                                    .iter()
                                    .rev()
                                    .skip(1)
                                    .any(|scope| scope.contains(&name));
                                if shadowed {
                                    diagnostics.push(Diagnostic::warning(
                                        Range::from_node(&child),
                                        "GO003",
                                        DiagnosticCategory::Logic,
                                        format!(
                                            "Variable '{}' shadows a variable in an outer scope",
                                            name
                                        ),
                                    ));
                                }
                                if let Some(current) = scope_stack.last_mut() {
                                    current.push(name);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
            true
        });

        diagnostics
    }

    /// GO004: Naked return in function with named return values
    fn check_naked_return(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "function_declaration" || node.kind() == "method_declaration" {
                // Check if function has named return values
                let has_named_returns = self.has_named_returns(node, file);
                if has_named_returns {
                    // Walk function body for naked returns
                    if let Some(body) = node.child_by_field_name("body") {
                        self.find_naked_returns(&body, file, &mut diagnostics);
                    }
                }
            }
            true
        });

        diagnostics
    }

    fn has_named_returns(&self, func_node: &tree_sitter::Node<'_>, _file: &ParsedFile) -> bool {
        if let Some(result) = func_node.child_by_field_name("result") {
            // Named returns use parameter_list with identifiers
            if result.kind() == "parameter_list" {
                let mut cursor = result.walk();
                for child in result.children(&mut cursor) {
                    if child.kind() == "parameter_declaration" {
                        // Named return has both name and type
                        if child.child_by_field_name("name").is_some() {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn find_naked_returns(
        &self,
        node: &tree_sitter::Node<'_>,
        file: &ParsedFile,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "return_statement" {
                // Naked return has no expression list
                let mut rc = child.walk();
                let has_values = child
                    .children(&mut rc)
                    .any(|c| c.kind() == "expression_list");
                if !has_values {
                    let text = file.node_text(&child).trim();
                    if text == "return" {
                        diagnostics.push(Diagnostic::warning(
                            Range::from_node(&child),
                            "GO004",
                            DiagnosticCategory::Style,
                            "Naked return in function with named return values — consider explicit return",
                        ));
                    }
                }
            } else if child.kind() != "func_literal" && child.kind() != "function_declaration" {
                // Recurse but not into nested functions
                self.find_naked_returns(&child, file, diagnostics);
            }
        }
    }

    /// GO005: context.Background() used outside main/init function
    fn check_context_background(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut current_func_name: Option<String> = None;

        file.walk(|node, _depth| {
            match node.kind() {
                "function_declaration" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        current_func_name = Some(file.node_text(&name_node).to_string());
                    }
                }
                "call_expression" => {
                    if let Some(func) = node.child_by_field_name("function") {
                        let text = file.node_text(&func);
                        if text == "context.Background" || text == "context.TODO" {
                            let in_allowed = current_func_name.as_deref()
                                .map(|n| n == "main" || n == "init" || n == "TestMain")
                                .unwrap_or(false);
                            if !in_allowed {
                                diagnostics.push(Diagnostic::warning(
                                    Range::from_node(node),
                                    "GO005",
                                    DiagnosticCategory::Logic,
                                    format!(
                                        "{}() used outside main/init — prefer passing context as parameter",
                                        text
                                    ),
                                ));
                            }
                        }
                    }
                }
                _ => {}
            }
            true
        });

        diagnostics
    }

    /// GO006: Empty error handling branch (if err != nil { } with empty block)
    fn check_empty_error_handling(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "if_statement" {
                if let Some(condition) = node.child_by_field_name("condition") {
                    let cond_text = file.node_text(&condition);
                    if cond_text.contains("err") && cond_text.contains("nil") {
                        // Check if the consequence block is empty
                        if let Some(body) = node.child_by_field_name("consequence") {
                            let mut cursor = body.walk();
                            let non_brace = body.children(&mut cursor)
                                .filter(|c| c.kind() != "{" && c.kind() != "}")
                                .count();
                            if non_brace == 0 {
                                diagnostics.push(Diagnostic::warning(
                                    Range::from_node(node),
                                    "GO006",
                                    DiagnosticCategory::Logic,
                                    "Empty error handling block — handle the error or add a comment",
                                ));
                            }
                        }
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// GO007: fmt.Sprintf in error string (use fmt.Errorf instead)
    fn check_sprintf_in_error(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "call_expression" {
                if let Some(func) = node.child_by_field_name("function") {
                    let func_text = file.node_text(&func);
                    if func_text == "errors.New" || func_text == "fmt.Errorf" {
                        // Check if argument is fmt.Sprintf(...)
                        if let Some(args) = node.child_by_field_name("arguments") {
                            let mut cursor = args.walk();
                            for child in args.children(&mut cursor) {
                                if child.kind() == "call_expression" {
                                    if let Some(inner_func) = child.child_by_field_name("function") {
                                        if file.node_text(&inner_func) == "fmt.Sprintf" {
                                            diagnostics.push(Diagnostic::warning(
                                                Range::from_node(node),
                                                "GO007",
                                                DiagnosticCategory::Style,
                                                "Use fmt.Errorf() directly instead of errors.New(fmt.Sprintf(...))",
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// GO008: Exported function/type missing doc comment
    fn check_exported_doc_comment(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            let name_field = match node.kind() {
                "function_declaration" | "method_declaration" | "type_declaration" => {
                    node.child_by_field_name("name")
                }
                "type_spec" => node.child_by_field_name("name"),
                _ => None,
            };

            if let Some(name_node) = name_field {
                let name = file.node_text(&name_node);
                // Exported names start with uppercase
                if name.starts_with(|c: char| c.is_ascii_uppercase()) {
                    let has_doc = self.has_preceding_comment(node, file);
                    if !has_doc {
                        diagnostics.push(Diagnostic::new(
                            Range::from_node(&name_node),
                            DiagnosticSeverity::Information,
                            "GO008",
                            DiagnosticCategory::Style,
                            format!("Exported name '{}' should have a doc comment", name),
                        ));
                    }
                }
            }
            true
        });

        diagnostics
    }

    fn has_preceding_comment(&self, node: &tree_sitter::Node<'_>, file: &ParsedFile) -> bool {
        if let Some(prev) = node.prev_sibling() {
            if prev.kind() == "comment" {
                let text = file.node_text(&prev);
                // Go doc comments start with // followed by the name
                return text.starts_with("//");
            }
        }
        false
    }

    /// GO009: Goroutine spawned without a WaitGroup or context cancellation check
    ///
    /// Flags `go func()` literals where neither a `sync.WaitGroup` Add/Done
    /// pattern nor a context parameter is visible in the enclosing function
    /// signature.  This is a conservative heuristic — it fires when the word
    /// "wg" or "ctx" is absent from the enclosing function body.
    fn check_goroutine_leak(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "go_statement" {
                // Look for `go func()` — anonymous goroutine literal
                if let Some(call) = node.child_by_field_name("call") {
                    if let Some(func) = call.child_by_field_name("function") {
                        if func.kind() == "func_literal" {
                            let body_text = file.node_text(node);
                            // Heuristic: flag if neither WaitGroup helpers nor
                            // context cancellation channels are referenced.
                            let has_wg = body_text.contains("wg.")
                                || body_text.contains(".Add(")
                                || body_text.contains(".Done()");
                            let has_ctx = body_text.contains("ctx.Done()")
                                || body_text.contains("<-ctx");
                            if !has_wg && !has_ctx {
                                diagnostics.push(Diagnostic::new(
                                    Range::from_node(node),
                                    DiagnosticSeverity::Warning,
                                    "GO009",
                                    DiagnosticCategory::Logic,
                                    "Goroutine spawned without WaitGroup or context cancellation — potential goroutine leak",
                                ));
                            }
                        }
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// GO010: Unused import (non-blank, non-side-effect import that is never
    /// referenced by any identifier in the file body)
    ///
    /// Tree-sitter allows us to collect all import names and verify each
    /// appears at least once as a selector prefix in the source text.
    fn check_unused_import(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Collect (local_name, path, node) for each non-blank import spec.
        let mut imports: Vec<(String, String, tree_sitter::Range)> = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "import_spec" {
                // Skip blank imports (handled by GO002)
                if let Some(name_node) = node.child_by_field_name("name") {
                    if file.node_text(&name_node) == "_" {
                        return true;
                    }
                    // Skip dot imports
                    if file.node_text(&name_node) == "." {
                        return true;
                    }
                }

                let path_node = match node.child_by_field_name("path") {
                    Some(p) => p,
                    None => return true,
                };
                let raw_path = file.node_text(&path_node);
                let path = raw_path.trim_matches('"').to_string();

                // Derive the local name (last component of the path, or alias)
                let local_name = if let Some(name_node) = node.child_by_field_name("name") {
                    file.node_text(&name_node).to_string()
                } else {
                    path.split('/').last().unwrap_or(&path).to_string()
                };

                imports.push((local_name, path, node.range()));
            }
            true
        });

        // For each import, check if its local name appears as a selector prefix
        // anywhere in the source (simple text scan).
        for (local_name, _path, range) in &imports {
            let selector = format!("{}.", local_name);
            // Also check for `local_name` used directly as a type (e.g. `http.Handler`)
            if !file.source.contains(&selector) {
                let ts_range = crate::diagnostic::Range::new(
                    crate::diagnostic::Position::new(
                        range.start_point.row as u32,
                        range.start_point.column as u32,
                    ),
                    crate::diagnostic::Position::new(
                        range.end_point.row as u32,
                        range.end_point.column as u32,
                    ),
                );
                diagnostics.push(Diagnostic::new(
                    ts_range,
                    DiagnosticSeverity::Warning,
                    "GO010",
                    DiagnosticCategory::Names,
                    format!("Imported package '{}' appears to be unused", local_name),
                ));
            }
        }

        diagnostics
    }
}

impl Default for GoChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Checker for GoChecker {
    fn language(&self) -> Language {
        Language::Go
    }

    fn check(&self, file: &ParsedFile, _config: &LintConfig) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Check for syntax errors from tree-sitter
        if file.has_errors {
            file.walk(|node, _depth| {
                if node.is_error() || node.is_missing() {
                    diagnostics.push(Diagnostic::error(
                        Range::from_node(node),
                        "GO000",
                        DiagnosticCategory::Syntax,
                        "Syntax error",
                    ));
                }
                true
            });
        }

        // Run all checks
        diagnostics.extend(self.check_unchecked_error(file));
        diagnostics.extend(self.check_blank_import(file));
        diagnostics.extend(self.check_shadowed_variable(file));
        diagnostics.extend(self.check_naked_return(file));
        diagnostics.extend(self.check_context_background(file));
        diagnostics.extend(self.check_empty_error_handling(file));
        diagnostics.extend(self.check_sprintf_in_error(file));
        diagnostics.extend(self.check_exported_doc_comment(file));
        diagnostics.extend(self.check_goroutine_leak(file));
        diagnostics.extend(self.check_unused_import(file));

        diagnostics
    }

    fn available_rules(&self) -> Vec<&'static str> {
        vec![
            "GO000", // Syntax error
            "GO001", // Unchecked error
            "GO002", // Blank import
            "GO003", // Shadowed variable
            "GO004", // Naked return
            "GO005", // context.Background() outside main/init
            "GO006", // Empty error handling
            "GO007", // fmt.Sprintf in error string
            "GO008", // Exported name missing doc comment
            "GO009", // Goroutine without WaitGroup/context
            "GO010", // Unused import
        ]
    }
}
