---
id: libs-compass-src-lint-rust-checker-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/lint/rust_checker.rs`.
capability_refs:
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: multi-language-parser-and-checker-dispatch-contract
  gap: multi-language-parser-and-checker-dispatch-contract
  coverage: full
  rationale: "Multi-language parser and checker dispatch contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: agent-diagnostic-output-contract
  gap: agent-diagnostic-output-contract
  coverage: full
  rationale: "Agent diagnostic output contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: symbol-outline-and-propagated-type-query-contract
  gap: symbol-outline-and-propagated-type-query-contract
  coverage: full
  rationale: "Symbol outline and propagated type query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: semantic-search-and-graph-query-contract
  gap: semantic-search-and-graph-query-contract
  coverage: full
  rationale: "Semantic search and graph query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: structured-refactoring-contract
  gap: structured-refactoring-contract
  coverage: full
  rationale: "Structured refactoring contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: spec-parser-and-state-machine-validation-contract
  gap: spec-parser-and-state-machine-validation-contract
  coverage: full
  rationale: "Spec parser and state-machine validation contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: python-and-rust-generator-registry-contract
  gap: python-and-rust-generator-registry-contract
  coverage: full
  rationale: "Python and Rust generator registry contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: argus-daemon-protocol-and-request-handling-contract
  gap: argus-daemon-protocol-and-request-handling-contract
  coverage: full
  rationale: "Argus daemon protocol and request handling contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: watch-bridge-and-incremental-dirty-file-contract
  gap: watch-bridge-and-incremental-dirty-file-contract
  coverage: full
  rationale: "Watch bridge and incremental dirty-file contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
fill_sections: [overview, source, changes]
---

# Standardized libs/compass/src/lint/rust_checker.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/lint/rust_checker.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `RustChecker` | libs/compass/src/lint/rust_checker.rs | struct | pub | 8 | pub struct RustChecker; |
| `new` | libs/compass/src/lint/rust_checker.rs | function | pub | 11 | pub fn new() -> Self { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Rust code checker

use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Range};
use crate::syntax::{Language, ParsedFile};

/// Rust checker
pub struct RustChecker;

impl RustChecker {
    pub fn new() -> Self {
        Self
    }

    /// Check for unsafe blocks
    fn check_unsafe_blocks(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "unsafe_block" {
                diagnostics.push(Diagnostic::new(
                    Range::from_node(node),
                    DiagnosticSeverity::Information,
                    "RS201",
                    DiagnosticCategory::Security,
                    "Unsafe block - ensure memory safety is manually verified",
                ));
            }
            true
        });

        diagnostics
    }

    /// Check for .clone() calls that might be unnecessary
    fn check_clone_usage(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "call_expression" {
                if let Some(func) = node.child_by_field_name("function") {
                    if func.kind() == "field_expression" {
                        if let Some(field) = func.child_by_field_name("field") {
                            if file.node_text(&field) == "clone" {
                                diagnostics.push(Diagnostic::new(
                                    Range::from_node(node),
                                    DiagnosticSeverity::Hint,
                                    "RS001",
                                    DiagnosticCategory::Style,
                                    "Consider if .clone() is necessary - borrowing may be more efficient",
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

    /// Check for unwrap() calls
    fn check_unwrap(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "call_expression" {
                if let Some(func) = node.child_by_field_name("function") {
                    if func.kind() == "field_expression" {
                        if let Some(field) = func.child_by_field_name("field") {
                            let field_name = file.node_text(&field);
                            if field_name == "unwrap" || field_name == "expect" {
                                diagnostics.push(Diagnostic::warning(
                                    Range::from_node(node),
                                    "RS101",
                                    DiagnosticCategory::Logic,
                                    format!(
                                        ".{}() can panic - consider using ? or match for error handling",
                                        field_name
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

    /// RS102: todo!/unimplemented! macros
    fn check_todo_macros(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "macro_invocation" {
                if let Some(macro_node) = node.child_by_field_name("macro") {
                    let macro_name = file.node_text(&macro_node);
                    if macro_name == "todo" || macro_name == "unimplemented" {
                        diagnostics.push(Diagnostic::warning(
                            Range::from_node(node),
                            "RS102",
                            DiagnosticCategory::Logic,
                            format!("{}! macro will panic at runtime", macro_name),
                        ));
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// RS103: dbg! macro left in code
    fn check_dbg_macro(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "macro_invocation" {
                if let Some(macro_node) = node.child_by_field_name("macro") {
                    if file.node_text(&macro_node) == "dbg" {
                        diagnostics.push(Diagnostic::new(
                            Range::from_node(node),
                            DiagnosticSeverity::Hint,
                            "RS103",
                            DiagnosticCategory::Style,
                            "dbg! macro should be removed in production",
                        ));
                    }
                }
            }
            true
        });
        diagnostics
    }

    /// RS006: needless-return — return at end of function body
    fn check_needless_return(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "function_item" || node.kind() == "closure_expression" {
                if let Some(body) = node.child_by_field_name("body") {
                    if body.kind() == "block" {
                        let child_count = body.child_count();
                        if child_count >= 2 {
                            // Last non-`}` child
                            let last = body.child(child_count - 2);
                            if let Some(last_node) = last {
                                if last_node.kind() == "return_expression" {
                                    diagnostics.push(Diagnostic::new(
                                        Range::from_node(&last_node),
                                        DiagnosticSeverity::Hint,
                                        "RS006",
                                        DiagnosticCategory::Style,
                                        "Needless 'return' — remove it and use an expression tail",
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

    /// RS007: large-enum-variant — variant with many fields
    fn check_large_enum_variant(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "enum_item" {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "enum_variant_list" {
                        let mut inner = child.walk();
                        for variant in child.children(&mut inner) {
                            if variant.kind() == "enum_variant" {
                                let mut vc = variant.walk();
                                for part in variant.children(&mut vc) {
                                    if part.kind() == "field_declaration_list" {
                                        let count = {
                                            let mut pc = part.walk();
                                            part.children(&mut pc).filter(|c| c.kind() == "field_declaration").count()
                                        };
                                        if count > 4 {
                                            diagnostics.push(Diagnostic::warning(
                                                Range::from_node(&variant), "RS007", DiagnosticCategory::Style,
                                                format!("Large enum variant with {} fields — consider boxing", count),
                                            ));
                                        }
                                    }
                                    if part.kind() == "ordered_field_declaration_list" {
                                        let count = {
                                            let mut pc = part.walk();
                                            part.children(&mut pc).filter(|c| c.kind() != "(" && c.kind() != ")" && c.kind() != ",").count()
                                        };
                                        if count > 3 {
                                            diagnostics.push(Diagnostic::warning(
                                                Range::from_node(&variant), "RS007", DiagnosticCategory::Style,
                                                format!("Large enum variant with {} tuple fields — consider a struct", count),
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

    /// RS008: manual-map — match with Some/None arms that could be .map()
    fn check_manual_map(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "match_expression" {
                if let Some(body) = node.child_by_field_name("body") {
                    let mut cursor = body.walk();
                    let arms: Vec<_> = body
                        .children(&mut cursor)
                        .filter(|c| c.kind() == "match_arm")
                        .collect();
                    if arms.len() == 2 {
                        let text0 = file.node_text(&arms[0]);
                        let text1 = file.node_text(&arms[1]);
                        let has_some = text0.contains("Some") || text1.contains("Some");
                        let has_none = text0.contains("None") || text1.contains("None");
                        if has_some && has_none {
                            diagnostics.push(Diagnostic::new(
                                Range::from_node(node),
                                DiagnosticSeverity::Hint,
                                "RS008",
                                DiagnosticCategory::Style,
                                "Manual match on Option — consider using .map()",
                            ));
                        }
                    }
                }
            }
            true
        });
        diagnostics
    }

    /// RS009: single-match — match with wildcard that could be if-let
    fn check_single_match(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "match_expression" {
                if let Some(body) = node.child_by_field_name("body") {
                    let mut cursor = body.walk();
                    let arms: Vec<_> = body
                        .children(&mut cursor)
                        .filter(|c| c.kind() == "match_arm")
                        .collect();
                    if arms.len() == 2 {
                        let arm1_text = file.node_text(&arms[1]).trim().to_string();
                        if arm1_text.starts_with("_ =>") {
                            let rhs = arm1_text.trim_start_matches("_ =>").trim();
                            if rhs == "()" || rhs == "{}," || rhs == "{}" || rhs == "()," {
                                diagnostics.push(Diagnostic::new(
                                    Range::from_node(node),
                                    DiagnosticSeverity::Hint,
                                    "RS009",
                                    DiagnosticCategory::Style,
                                    "Single-arm match — consider using 'if let' instead",
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

    /// RS010: wildcard-imports — use foo::*
    fn check_wildcard_imports(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "use_wildcard" {
                diagnostics.push(Diagnostic::warning(
                    Range::from_node(node),
                    "RS010",
                    DiagnosticCategory::Style,
                    "Wildcard import — prefer explicit imports for clarity",
                ));
            }
            true
        });
        diagnostics
    }

    /// RS011: missing-docs-public — pub item without doc comment
    fn check_missing_docs_public(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            let kind = node.kind();
            if kind == "function_item" || kind == "struct_item" || kind == "enum_item" {
                let text = file.node_text(node);
                if text.starts_with("pub ") || text.starts_with("pub(") {
                    let has_doc = node.prev_sibling().map_or(false, |s| {
                        let st = file.node_text(&s).trim().to_string();
                        st.starts_with("///") || st.starts_with("#[doc")
                    });
                    if !has_doc {
                        diagnostics.push(Diagnostic::new(
                            Range::from_node(node),
                            DiagnosticSeverity::Information,
                            "RS011",
                            DiagnosticCategory::Style,
                            "Public item missing documentation comment",
                        ));
                    }
                }
            }
            true
        });
        diagnostics
    }

    /// RS012: redundant-closure — .map(|x| foo(x)) could be .map(foo)
    fn check_redundant_closure(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "closure_expression" {
                if let Some(params) = node.child_by_field_name("parameters") {
                    let param_text = file.node_text(&params).trim().to_string();
                    let param_name = param_text
                        .trim_start_matches('|')
                        .trim_end_matches('|')
                        .trim();
                    if !param_name.is_empty() && !param_name.contains(',') {
                        if let Some(body) = node.child_by_field_name("body") {
                            if body.kind() == "call_expression" {
                                if let Some(args) = body.child_by_field_name("arguments") {
                                    let args_text = file.node_text(&args).trim().to_string();
                                    let expected = format!("({})", param_name);
                                    if args_text == expected {
                                        diagnostics.push(Diagnostic::new(
                                            Range::from_node(node),
                                            DiagnosticSeverity::Hint,
                                            "RS012",
                                            DiagnosticCategory::Style,
                                            "Redundant closure — pass the function directly",
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

    /// RS013: print-macro — print!/println! left in code
    fn check_print_macro(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "macro_invocation" {
                if let Some(macro_node) = node.child_by_field_name("macro") {
                    let name = file.node_text(&macro_node);
                    if name == "print" || name == "println" {
                        diagnostics.push(Diagnostic::new(
                            Range::from_node(node),
                            DiagnosticSeverity::Hint,
                            "RS013",
                            DiagnosticCategory::Style,
                            format!("{}! macro — use a logger or eprintln! instead", name),
                        ));
                    }
                }
            }
            true
        });
        diagnostics
    }

    /// RS014: string-to-string — .to_string() on a string literal
    fn check_string_to_string(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "call_expression" {
                if let Some(func) = node.child_by_field_name("function") {
                    if func.kind() == "field_expression" {
                        if let Some(field) = func.child_by_field_name("field") {
                            if file.node_text(&field) == "to_string" {
                                if let Some(val) = func.child_by_field_name("value") {
                                    if val.kind() == "string_literal" {
                                        diagnostics.push(Diagnostic::new(
                                            Range::from_node(node),
                                            DiagnosticSeverity::Hint,
                                            "RS014",
                                            DiagnosticCategory::Style,
                                            "Use .to_owned() or String::from() on string literals",
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

    /// RS015: unused-must-use — discarding return value of method call
    fn check_unused_must_use(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "expression_statement" {
                if let Some(child) = node.child(0) {
                    if child.kind() == "call_expression" {
                        if let Some(func) = child.child_by_field_name("function") {
                            if func.kind() == "field_expression" {
                                diagnostics.push(Diagnostic::new(
                                    Range::from_node(node), DiagnosticSeverity::Information,
                                    "RS015", DiagnosticCategory::Logic,
                                    "Method return value is unused — consider binding or handling it",
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

impl Default for RustChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Checker for RustChecker {
    fn language(&self) -> Language {
        Language::Rust
    }

    fn check(&self, file: &ParsedFile, _config: &LintConfig) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Check for syntax errors
        if file.has_errors {
            file.walk(|node, _depth| {
                if node.is_error() || node.is_missing() {
                    diagnostics.push(Diagnostic::error(
                        Range::from_node(node),
                        "RS000",
                        DiagnosticCategory::Syntax,
                        "Syntax error",
                    ));
                }
                true
            });
        }

        diagnostics.extend(self.check_unsafe_blocks(file));
        diagnostics.extend(self.check_clone_usage(file));
        diagnostics.extend(self.check_unwrap(file));
        diagnostics.extend(self.check_todo_macros(file));
        diagnostics.extend(self.check_dbg_macro(file));
        diagnostics.extend(self.check_needless_return(file));
        diagnostics.extend(self.check_large_enum_variant(file));
        diagnostics.extend(self.check_manual_map(file));
        diagnostics.extend(self.check_single_match(file));
        diagnostics.extend(self.check_wildcard_imports(file));
        diagnostics.extend(self.check_missing_docs_public(file));
        diagnostics.extend(self.check_redundant_closure(file));
        diagnostics.extend(self.check_print_macro(file));
        diagnostics.extend(self.check_string_to_string(file));
        diagnostics.extend(self.check_unused_must_use(file));

        diagnostics
    }

    fn available_rules(&self) -> Vec<&'static str> {
        vec![
            "RS000", // Syntax error
            "RS001", // Unnecessary clone
            "RS006", // needless-return
            "RS007", // large-enum-variant
            "RS008", // manual-map
            "RS009", // single-match
            "RS010", // wildcard-imports
            "RS011", // missing-docs-public
            "RS012", // redundant-closure
            "RS013", // print-macro
            "RS014", // string-to-string
            "RS015", // unused-must-use
            "RS101", // unwrap/expect usage
            "RS102", // todo!/unimplemented!
            "RS103", // dbg! macro
            "RS201", // Unsafe block
        ]
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/lint/rust_checker.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/lint/rust_checker.rs` captured during libs codegen standardization.
```
