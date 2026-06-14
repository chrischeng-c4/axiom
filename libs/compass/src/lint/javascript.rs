//! JavaScript code checker

use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Range};
use crate::syntax::{Language, ParsedFile};

/// JavaScript checker
pub struct JavaScriptChecker;

impl JavaScriptChecker {
    pub fn new() -> Self {
        Self
    }

    /// JS001: Detect `var` declarations and suggest `let` or `const`
    ///
    /// The `var` keyword has function scope and hoisting behavior that
    /// can lead to subtle bugs. Modern JavaScript should use `let` or `const`.
    fn check_var_usage(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "variable_declaration" {
                let text = file.node_text(node);
                if text.starts_with("var ") {
                    diagnostics.push(Diagnostic::warning(
                        Range::from_node(node),
                        "JS001",
                        DiagnosticCategory::Style,
                        "Avoid 'var' — use 'let' or 'const' for block scoping",
                    ));
                }
            }
            true
        });

        diagnostics
    }

    /// JS002: Detect `console.log` statements left in code
    ///
    /// Console statements should be removed before production deployment.
    fn check_console_log(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "call_expression" {
                if let Some(func) = node.child_by_field_name("function") {
                    if func.kind() == "member_expression" {
                        let text = file.node_text(&func);
                        if text == "console.log" {
                            diagnostics.push(Diagnostic::warning(
                                Range::from_node(node),
                                "JS002",
                                DiagnosticCategory::Style,
                                "Remove 'console.log' before production",
                            ));
                        }
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// JS003: Detect loose equality (`==` / `!=`) and suggest strict equality
    ///
    /// Loose equality performs type coercion which can lead to unexpected
    /// results. Use `===` and `!==` for predictable comparisons.
    fn check_loose_equality(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "binary_expression" {
                if let Some(op) = node.child_by_field_name("operator") {
                    let op_text = file.node_text(&op);
                    if op_text == "==" {
                        diagnostics.push(Diagnostic::warning(
                            Range::from_node(node),
                            "JS003",
                            DiagnosticCategory::Logic,
                            "Use '===' instead of '==' to avoid type coercion",
                        ));
                    } else if op_text == "!=" {
                        diagnostics.push(Diagnostic::warning(
                            Range::from_node(node),
                            "JS003",
                            DiagnosticCategory::Logic,
                            "Use '!==' instead of '!=' to avoid type coercion",
                        ));
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// JS004: Detect `eval()` usage (security risk)
    ///
    /// `eval()` executes arbitrary strings as code, creating security
    /// vulnerabilities and making code harder to analyze.
    fn check_eval_usage(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "call_expression" {
                if let Some(func) = node.child_by_field_name("function") {
                    if func.kind() == "identifier" {
                        let name = file.node_text(&func);
                        if name == "eval" {
                            diagnostics.push(Diagnostic::new(
                                Range::from_node(node),
                                DiagnosticSeverity::Error,
                                "JS004",
                                DiagnosticCategory::Security,
                                "Avoid 'eval()' — it executes arbitrary code and is a security risk",
                            ));
                        }
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// JS005: Detect `debugger` statements left in code
    fn check_debugger_statement(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "debugger_statement" {
                diagnostics.push(Diagnostic::warning(
                    Range::from_node(node),
                    "JS005",
                    DiagnosticCategory::Logic,
                    "Remove 'debugger' statement before production",
                ));
            }
            true
        });
        diagnostics
    }

    /// JS006: no-implied-eval — setTimeout/setInterval with string arg
    fn check_implied_eval(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "call_expression" {
                if let Some(func) = node.child_by_field_name("function") {
                    let name = file.node_text(&func);
                    if name == "setTimeout" || name == "setInterval" {
                        if let Some(args) = node.child_by_field_name("arguments") {
                            if let Some(first_arg) = args.child(1) {
                                if first_arg.kind() == "string"
                                    || first_arg.kind() == "template_string"
                                {
                                    diagnostics.push(Diagnostic::error(
                                        Range::from_node(node),
                                        "JS006",
                                        DiagnosticCategory::Security,
                                        format!(
                                            "Implied eval — pass a function to '{}', not a string",
                                            name
                                        ),
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

    /// JS007: no-proto — detect __proto__ property access
    fn check_no_proto(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "member_expression" {
                if let Some(prop) = node.child_by_field_name("property") {
                    if file.node_text(&prop) == "__proto__" {
                        diagnostics.push(Diagnostic::warning(
                            Range::from_node(node),
                            "JS007",
                            DiagnosticCategory::Style,
                            "Use Object.getPrototypeOf() instead of '__proto__'",
                        ));
                    }
                }
            }
            true
        });
        diagnostics
    }

    /// JS008: no-with — detect with statements
    fn check_no_with(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "with_statement" {
                diagnostics.push(Diagnostic::error(
                    Range::from_node(node),
                    "JS008",
                    DiagnosticCategory::Logic,
                    "'with' statement is forbidden — it makes code unpredictable",
                ));
            }
            true
        });
        diagnostics
    }

    /// JS009: no-alert — detect alert/confirm/prompt calls
    fn check_no_alert(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "call_expression" {
                if let Some(func) = node.child_by_field_name("function") {
                    if func.kind() == "identifier" {
                        let name = file.node_text(&func);
                        if name == "alert" || name == "confirm" || name == "prompt" {
                            diagnostics.push(Diagnostic::warning(
                                Range::from_node(node),
                                "JS009",
                                DiagnosticCategory::Style,
                                format!(
                                    "Unexpected '{}()' — use a custom UI component instead",
                                    name
                                ),
                            ));
                        }
                    }
                }
            }
            true
        });
        diagnostics
    }

    /// JS010: no-caller — detect arguments.caller/arguments.callee
    fn check_no_caller(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "member_expression" {
                if let Some(obj) = node.child_by_field_name("object") {
                    if file.node_text(&obj) == "arguments" {
                        if let Some(prop) = node.child_by_field_name("property") {
                            let prop_text = file.node_text(&prop);
                            if prop_text == "caller" || prop_text == "callee" {
                                diagnostics.push(Diagnostic::error(
                                    Range::from_node(node),
                                    "JS010",
                                    DiagnosticCategory::Logic,
                                    format!(
                                        "'arguments.{}' is deprecated and forbidden in strict mode",
                                        prop_text
                                    ),
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

    /// JS011: no-extend-native — detect prototype extension of built-in objects
    fn check_no_extend_native(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "assignment_expression" {
                if let Some(left) = node.child_by_field_name("left") {
                    let text = file.node_text(&left);
                    let builtins = ["Object", "Array", "String", "Number", "Boolean", "Function"];
                    for b in &builtins {
                        if text.starts_with(&format!("{}.prototype.", b)) {
                            diagnostics.push(Diagnostic::warning(
                                Range::from_node(node),
                                "JS011",
                                DiagnosticCategory::Logic,
                                format!("Do not extend native '{}' prototype", b),
                            ));
                            break;
                        }
                    }
                }
            }
            true
        });
        diagnostics
    }

    /// JS012: no-new-wrappers — detect new String/Number/Boolean
    fn check_no_new_wrappers(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "new_expression" {
                if let Some(ctor) = node.child_by_field_name("constructor") {
                    let name = file.node_text(&ctor);
                    if name == "String" || name == "Number" || name == "Boolean" {
                        diagnostics.push(Diagnostic::warning(
                            Range::from_node(node),
                            "JS012",
                            DiagnosticCategory::Style,
                            format!(
                                "Do not use '{}' as a constructor — use a literal instead",
                                name
                            ),
                        ));
                    }
                }
            }
            true
        });
        diagnostics
    }

    /// JS013: no-throw-literal — detect throwing non-Error values
    fn check_no_throw_literal(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "throw_statement" {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    let k = child.kind();
                    if k == "string"
                        || k == "number"
                        || k == "null"
                        || k == "undefined"
                        || k == "true"
                        || k == "false"
                        || k == "template_string"
                    {
                        diagnostics.push(Diagnostic::warning(
                            Range::from_node(node),
                            "JS013",
                            DiagnosticCategory::Logic,
                            "Throw an Error object instead of a literal",
                        ));
                    }
                }
            }
            true
        });
        diagnostics
    }

    /// JS014: no-return-assign — detect assignment inside return
    fn check_no_return_assign(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "return_statement" {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "assignment_expression" {
                        diagnostics.push(Diagnostic::warning(
                            Range::from_node(node),
                            "JS014",
                            DiagnosticCategory::Logic,
                            "Unexpected assignment in return statement",
                        ));
                    }
                }
            }
            true
        });
        diagnostics
    }

    /// JS015: no-self-compare — detect x === x or x == x
    fn check_no_self_compare(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "binary_expression" {
                if let Some(op) = node.child_by_field_name("operator") {
                    let op_text = file.node_text(&op);
                    if op_text == "==" || op_text == "===" || op_text == "!=" || op_text == "!==" {
                        if let (Some(left), Some(right)) = (
                            node.child_by_field_name("left"),
                            node.child_by_field_name("right"),
                        ) {
                            if file.node_text(&left) == file.node_text(&right) {
                                diagnostics.push(Diagnostic::warning(
                                    Range::from_node(node),
                                    "JS015",
                                    DiagnosticCategory::Logic,
                                    "Comparing a value to itself is always redundant",
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
}

impl Default for JavaScriptChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Checker for JavaScriptChecker {
    fn language(&self) -> Language {
        Language::JavaScript
    }

    fn check(&self, file: &ParsedFile, _config: &LintConfig) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Check for syntax errors from tree-sitter
        if file.has_errors {
            file.walk(|node, _depth| {
                if node.is_error() || node.is_missing() {
                    diagnostics.push(Diagnostic::error(
                        Range::from_node(node),
                        "JS000",
                        DiagnosticCategory::Syntax,
                        "Syntax error",
                    ));
                }
                true
            });
        }

        // Run all checks
        diagnostics.extend(self.check_var_usage(file));
        diagnostics.extend(self.check_console_log(file));
        diagnostics.extend(self.check_loose_equality(file));
        diagnostics.extend(self.check_eval_usage(file));
        diagnostics.extend(self.check_debugger_statement(file));
        diagnostics.extend(self.check_implied_eval(file));
        diagnostics.extend(self.check_no_proto(file));
        diagnostics.extend(self.check_no_with(file));
        diagnostics.extend(self.check_no_alert(file));
        diagnostics.extend(self.check_no_caller(file));
        diagnostics.extend(self.check_no_extend_native(file));
        diagnostics.extend(self.check_no_new_wrappers(file));
        diagnostics.extend(self.check_no_throw_literal(file));
        diagnostics.extend(self.check_no_return_assign(file));
        diagnostics.extend(self.check_no_self_compare(file));

        diagnostics
    }

    fn available_rules(&self) -> Vec<&'static str> {
        vec![
            "JS000", // Syntax error
            "JS001", // var usage
            "JS002", // console.log
            "JS003", // Loose equality (== / !=)
            "JS004", // eval() usage
            "JS005", // debugger statement
            "JS006", // no-implied-eval
            "JS007", // no-proto
            "JS008", // no-with
            "JS009", // no-alert
            "JS010", // no-caller
            "JS011", // no-extend-native
            "JS012", // no-new-wrappers
            "JS013", // no-throw-literal
            "JS014", // no-return-assign
            "JS015", // no-self-compare
        ]
    }
}
