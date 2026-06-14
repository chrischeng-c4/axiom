//! TypeScript code checker

use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Range, TextEdit};
use crate::syntax::{Language, ParsedFile};

/// TypeScript checker
pub struct TypeScriptChecker;

impl TypeScriptChecker {
    pub fn new() -> Self {
        Self
    }

    /// Check for non-null assertions (!)
    fn check_non_null_assertion(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "non_null_expression" {
                diagnostics.push(Diagnostic::warning(
                    Range::from_node(node),
                    "TS102",
                    DiagnosticCategory::Type,
                    "Non-null assertion (!) bypasses TypeScript's null checks",
                ));
            }
            true
        });

        diagnostics
    }

    /// Check for type assertions (as Type)
    fn check_type_assertion(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "as_expression" {
                diagnostics.push(Diagnostic::new(
                    Range::from_node(node),
                    crate::diagnostic::DiagnosticSeverity::Information,
                    "TS002",
                    DiagnosticCategory::Type,
                    "Type assertion may hide type errors",
                ));
            }
            true
        });

        diagnostics
    }

    /// Check for any type usage
    fn check_any_type(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "predefined_type" {
                let text = file.node_text(node);
                if text == "any" {
                    diagnostics.push(Diagnostic::warning(
                        Range::from_node(node),
                        "TS001",
                        DiagnosticCategory::Type,
                        "Avoid using 'any' type - use 'unknown' or a more specific type",
                    ));
                }
            }
            true
        });

        diagnostics
    }

    /// TS103: console.log statements left in code
    fn check_console_log(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "call_expression" {
                if let Some(func) = node.child_by_field_name("function") {
                    if func.kind() == "member_expression" {
                        let text = file.node_text(&func);
                        if text.starts_with("console.") {
                            diagnostics.push(Diagnostic::new(
                                Range::from_node(node),
                                crate::diagnostic::DiagnosticSeverity::Hint,
                                "TS103",
                                DiagnosticCategory::Style,
                                format!("'{}' statement should be removed in production", text),
                            ));
                        }
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// TS104: Prefer const over let when variable is never reassigned
    fn check_prefer_const(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        use std::collections::{HashMap, HashSet};

        let mut diagnostics = Vec::new();
        let mut let_vars: HashMap<String, (Range, Range)> = HashMap::new();
        let mut reassigned: HashSet<String> = HashSet::new();

        file.walk(|node, _depth| {
            if node.kind() == "lexical_declaration" {
                let text = file.node_text(node);
                if text.starts_with("let ") {
                    let decl_range = Range::from_node(node);
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        if child.kind() == "variable_declarator" {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                let name = file.node_text(&name_node).to_string();
                                let_vars.insert(
                                    name,
                                    (Range::from_node(&name_node), decl_range.clone()),
                                );
                            }
                        }
                    }
                }
            }
            if node.kind() == "assignment_expression" {
                if let Some(left) = node.child_by_field_name("left") {
                    if left.kind() == "identifier" {
                        reassigned.insert(file.node_text(&left).to_string());
                    }
                }
            }
            if node.kind() == "update_expression" {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        reassigned.insert(file.node_text(&child).to_string());
                    }
                }
            }
            true
        });

        for (name, (name_range, decl_range)) in let_vars {
            if !reassigned.contains(&name) {
                let fix_range = Range::new(
                    decl_range.start,
                    crate::diagnostic::Position::new(
                        decl_range.start.line,
                        decl_range.start.character + 3,
                    ),
                );
                diagnostics.push(
                    Diagnostic::new(
                        name_range,
                        DiagnosticSeverity::Hint,
                        "TS104",
                        DiagnosticCategory::Style,
                        format!("'{}' is never reassigned, use const instead", name),
                    )
                    .with_fix(
                        "Replace 'let' with 'const'",
                        vec![TextEdit {
                            range: fix_range,
                            new_text: "const".to_string(),
                        }],
                    ),
                );
            }
        }
        diagnostics
    }

    /// TS006: no-floating-promises — expression statement with call but no await
    fn check_floating_promises(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "expression_statement" {
                if let Some(child) = node.child(0) {
                    if child.kind() == "call_expression" {
                        let text = file.node_text(&child);
                        if text.contains("async") || text.ends_with("Async()") {
                            diagnostics.push(Diagnostic::warning(
                                Range::from_node(node),
                                "TS006",
                                DiagnosticCategory::Logic,
                                "Floating promise — add 'await' or handle the returned promise",
                            ));
                        }
                    }
                }
            }
            true
        });
        diagnostics
    }

    /// TS007: strict-boolean-expressions — if condition is a bare identifier
    fn check_strict_boolean(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "if_statement" {
                if let Some(cond) = node.child_by_field_name("condition") {
                    let inner = if cond.kind() == "parenthesized_expression" {
                        cond.child(1)
                    } else {
                        Some(cond)
                    };
                    if let Some(expr) = inner {
                        if expr.kind() == "identifier" {
                            diagnostics.push(Diagnostic::new(
                                Range::from_node(&expr),
                                DiagnosticSeverity::Information,
                                "TS007",
                                DiagnosticCategory::Type,
                                "Non-boolean used in condition — use an explicit comparison",
                            ));
                        }
                    }
                }
            }
            true
        });
        diagnostics
    }

    /// TS008: no-unnecessary-type-assertion — `x as T` where x and T look identical
    fn check_unnecessary_type_assertion(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "as_expression" {
                if let (Some(expr), Some(ty)) = (node.child(0), node.child(2)) {
                    let expr_text = file.node_text(&expr).trim().to_string();
                    let ty_text = file.node_text(&ty).trim().to_string();
                    if expr_text == ty_text {
                        diagnostics.push(Diagnostic::warning(
                            Range::from_node(node),
                            "TS008",
                            DiagnosticCategory::Type,
                            "Unnecessary type assertion — expression already has this type",
                        ));
                    }
                }
            }
            true
        });
        diagnostics
    }

    /// TS009: no-empty-interface — interface with no members
    fn check_empty_interface(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "interface_declaration" {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "interface_body" || child.kind() == "object_type" {
                        let mut inner = child.walk();
                        let has_members = child.children(&mut inner).any(|c| {
                            c.kind() != "{" && c.kind() != "}" && !c.kind().contains("comment")
                        });
                        if !has_members {
                            diagnostics.push(Diagnostic::warning(
                                Range::from_node(node),
                                "TS009",
                                DiagnosticCategory::Style,
                                "Empty interface — use a type alias or add members",
                            ));
                        }
                    }
                }
            }
            true
        });
        diagnostics
    }

    /// TS010: no-duplicate-enum-values
    fn check_duplicate_enum_values(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "enum_declaration" {
                let mut seen = std::collections::HashMap::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "enum_body" {
                        let mut inner = child.walk();
                        for member in child.children(&mut inner) {
                            if let Some(val) = member.child_by_field_name("value") {
                                let val_text = file.node_text(&val).trim().to_string();
                                if let Some(prev_line) = seen.get(&val_text) {
                                    diagnostics.push(Diagnostic::warning(
                                        Range::from_node(&val),
                                        "TS010",
                                        DiagnosticCategory::Logic,
                                        format!(
                                            "Duplicate enum value '{}' (first at line {})",
                                            val_text, prev_line
                                        ),
                                    ));
                                } else {
                                    seen.insert(val_text, val.start_position().row + 1);
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

    /// TS011: prefer-optional-chain — detect `foo && foo.bar`
    fn check_prefer_optional_chain(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "binary_expression" {
                if let Some(op) = node.child_by_field_name("operator") {
                    if file.node_text(&op) == "&&" {
                        if let (Some(left), Some(right)) = (
                            node.child_by_field_name("left"),
                            node.child_by_field_name("right"),
                        ) {
                            if left.kind() == "identifier" && right.kind() == "member_expression" {
                                if let Some(obj) = right.child_by_field_name("object") {
                                    let left_text = file.node_text(&left);
                                    let obj_text = file.node_text(&obj);
                                    if left_text == obj_text {
                                        diagnostics.push(Diagnostic::new(
                                            Range::from_node(node),
                                            DiagnosticSeverity::Hint,
                                            "TS011",
                                            DiagnosticCategory::Style,
                                            "Prefer optional chaining (?.) over '&&' guard",
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

    /// TS012: no-namespace — detect namespace/module declarations
    fn check_no_namespace(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "ambient_declaration"
                || node.kind() == "module"
                || node.kind() == "internal_module"
            {
                let text = file.node_text(node);
                if text.starts_with("namespace") || text.starts_with("module") {
                    diagnostics.push(Diagnostic::warning(
                        Range::from_node(node),
                        "TS012",
                        DiagnosticCategory::Style,
                        "Avoid namespace/module declarations — use ES modules instead",
                    ));
                }
            }
            true
        });
        diagnostics
    }

    /// TS013: explicit-function-return-type
    fn check_explicit_return_type(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "function_declaration" {
                let mut ret_cursor = node.walk();
                let has_return_type = node
                    .children(&mut ret_cursor)
                    .any(|c| c.kind() == "type_annotation");
                if !has_return_type {
                    diagnostics.push(Diagnostic::new(
                        Range::from_node(node),
                        DiagnosticSeverity::Information,
                        "TS013",
                        DiagnosticCategory::Type,
                        "Function missing explicit return type annotation",
                    ));
                }
            }
            true
        });
        diagnostics
    }

    /// TS014: no-var-requires — detect require() calls
    fn check_no_var_requires(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "call_expression" {
                if let Some(func) = node.child_by_field_name("function") {
                    if func.kind() == "identifier" && file.node_text(&func) == "require" {
                        diagnostics.push(Diagnostic::warning(
                            Range::from_node(node),
                            "TS014",
                            DiagnosticCategory::Style,
                            "Use ES 'import' instead of 'require()'",
                        ));
                    }
                }
            }
            true
        });
        diagnostics
    }

    /// TS015: consistent-type-imports — import of uppercase names could be type-only
    fn check_consistent_type_imports(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "import_statement" {
                let text = file.node_text(node);
                if text.contains("import type") || text.contains("import {type") {
                    return true;
                }
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "import_clause" {
                        let mut inner = child.walk();
                        for spec in child.children(&mut inner) {
                            if spec.kind() == "named_imports" {
                                let mut spec_cur = spec.walk();
                                let all_upper = spec
                                    .children(&mut spec_cur)
                                    .filter(|c| c.kind() == "import_specifier")
                                    .all(|c| {
                                        let n = file.node_text(&c).trim().to_string();
                                        n.chars().next().map_or(false, |ch| ch.is_uppercase())
                                    });
                                let has_any = spec
                                    .children(&mut spec.walk())
                                    .any(|c| c.kind() == "import_specifier");
                                if has_any && all_upper {
                                    diagnostics.push(Diagnostic::new(
                                        Range::from_node(node),
                                        DiagnosticSeverity::Hint,
                                        "TS015",
                                        DiagnosticCategory::Style,
                                        "All imported names are types — use 'import type' instead",
                                    ));
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
}

impl Default for TypeScriptChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Checker for TypeScriptChecker {
    fn language(&self) -> Language {
        Language::TypeScript
    }

    fn check(&self, file: &ParsedFile, _config: &LintConfig) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Check for syntax errors
        if file.has_errors {
            file.walk(|node, _depth| {
                if node.is_error() || node.is_missing() {
                    diagnostics.push(Diagnostic::error(
                        Range::from_node(node),
                        "TS000",
                        DiagnosticCategory::Syntax,
                        "Syntax error",
                    ));
                }
                true
            });
        }

        diagnostics.extend(self.check_any_type(file));
        diagnostics.extend(self.check_non_null_assertion(file));
        diagnostics.extend(self.check_type_assertion(file));
        diagnostics.extend(self.check_console_log(file));
        diagnostics.extend(self.check_prefer_const(file));
        diagnostics.extend(self.check_floating_promises(file));
        diagnostics.extend(self.check_strict_boolean(file));
        diagnostics.extend(self.check_unnecessary_type_assertion(file));
        diagnostics.extend(self.check_empty_interface(file));
        diagnostics.extend(self.check_duplicate_enum_values(file));
        diagnostics.extend(self.check_prefer_optional_chain(file));
        diagnostics.extend(self.check_no_namespace(file));
        diagnostics.extend(self.check_explicit_return_type(file));
        diagnostics.extend(self.check_no_var_requires(file));
        diagnostics.extend(self.check_consistent_type_imports(file));

        diagnostics
    }

    fn available_rules(&self) -> Vec<&'static str> {
        vec![
            "TS000", // Syntax error
            "TS001", // any type
            "TS002", // Type assertion
            "TS006", // no-floating-promises
            "TS007", // strict-boolean-expressions
            "TS008", // no-unnecessary-type-assertion
            "TS009", // no-empty-interface
            "TS010", // no-duplicate-enum-values
            "TS011", // prefer-optional-chain
            "TS012", // no-namespace
            "TS013", // explicit-function-return-type
            "TS014", // no-var-requires
            "TS015", // consistent-type-imports
            "TS102", // Non-null assertion
            "TS103", // console.log
            "TS104", // Prefer const
        ]
    }
}
