---
id: libs-compass-src-lint-html-rules-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/lint/html_rules.rs`.
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

# Standardized libs/compass/src/lint/html_rules.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/lint/html_rules.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `check_meta_charset` | libs/compass/src/lint/html_rules.rs | function | pub | 16 | pub(crate) fn check_meta_charset(&self, file: &ParsedFile) -> Vec<Diagnostic> { |
| `check_form_action` | libs/compass/src/lint/html_rules.rs | function | pub | 44 | pub(crate) fn check_form_action(&self, file: &ParsedFile) -> Vec<Diagnostic> { |
| `check_duplicate_ids` | libs/compass/src/lint/html_rules.rs | function | pub | 71 | pub(crate) fn check_duplicate_ids(&self, file: &ParsedFile) -> Vec<Diagnostic> { |
| `check_inline_style` | libs/compass/src/lint/html_rules.rs | function | pub | 107 | pub(crate) fn check_inline_style(&self, file: &ParsedFile) -> Vec<Diagnostic> { |
| `check_script_async_defer` | libs/compass/src/lint/html_rules.rs | function | pub | 136 | pub(crate) fn check_script_async_defer(&self, file: &ParsedFile) -> Vec<Diagnostic> { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Extended HTML lint rules (HTML006 - HTML011)
//!
//! Split from html.rs to keep files under 500 lines.

use crate::syntax::ParsedFile;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, Range};
use std::collections::HashMap;

use super::html::HtmlChecker;

impl HtmlChecker {
    /// HTML006: Missing `<meta charset>` in `<head>`
    ///
    /// Every HTML document should declare its character encoding via
    /// `<meta charset="...">` in the head for correct rendering.
    pub(crate) fn check_meta_charset(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "element" {
                let tag_name = Self::get_tag_name(node, file);
                if tag_name.eq_ignore_ascii_case("head") {
                    let has_charset = Self::has_meta_charset(node, file);
                    if !has_charset {
                        diagnostics.push(Diagnostic::warning(
                            Range::from_node(node),
                            "HTML006",
                            DiagnosticCategory::Style,
                            "Missing <meta charset=\"...\"> in <head>",
                        ));
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// HTML007: `<form>` element without `action` attribute
    ///
    /// A form without an explicit action will submit to the current URL,
    /// which is usually unintentional.
    pub(crate) fn check_form_action(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "element" {
                let tag_name = Self::get_tag_name(node, file);
                if tag_name.eq_ignore_ascii_case("form") {
                    if !Self::has_attribute(node, "action", file) {
                        diagnostics.push(Diagnostic::warning(
                            Range::from_node(node),
                            "HTML007",
                            DiagnosticCategory::Logic,
                            "Missing 'action' attribute on <form> — form will submit to current URL",
                        ));
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// HTML008: Duplicate element IDs
    ///
    /// Element IDs must be unique within a document. Duplicates cause
    /// unpredictable behavior with CSS selectors and JavaScript.
    pub(crate) fn check_duplicate_ids(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut seen: HashMap<String, Range> = HashMap::new();

        file.walk(|node, _depth| {
            if node.kind() == "element" || node.kind() == "self_closing_tag" {
                if let Some(id_value) = Self::get_attribute_value(node, "id", file) {
                    let id = strip_quotes(id_value).trim().to_string();
                    if !id.is_empty() {
                        if let Some(prev_range) = seen.get(&id) {
                            diagnostics.push(Diagnostic::warning(
                                Range::from_node(node),
                                "HTML008",
                                DiagnosticCategory::Logic,
                                format!(
                                    "Duplicate element ID '{}' (first at line {})",
                                    id,
                                    prev_range.start.line + 1
                                ),
                            ));
                        } else {
                            seen.insert(id, Range::from_node(node));
                        }
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// HTML009: Inline `style` attribute usage
    ///
    /// Inline styles are harder to maintain and override. Prefer external
    /// or embedded stylesheets with CSS classes.
    pub(crate) fn check_inline_style(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "element" || node.kind() == "self_closing_tag" {
                if Self::has_attribute(node, "style", file) {
                    diagnostics.push(Diagnostic::warning(
                        Range::from_node(node),
                        "HTML009",
                        DiagnosticCategory::Style,
                        "Avoid inline 'style' attribute — use CSS classes instead",
                    ));
                }
            }
            true
        });

        diagnostics
    }

    /// HTML010: Missing `<title>` in `<head>` (alias of HTML005)
    ///
    /// Note: This is logically the same as HTML005 but included here for
    /// rule numbering completeness. The actual check is in html.rs as HTML005.

    /// HTML011: `<script>` without `async` or `defer` attribute
    ///
    /// Scripts without async/defer block page rendering. Add one of these
    /// attributes for external scripts to improve page load performance.
    pub(crate) fn check_script_async_defer(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "element" {
                let tag_name = Self::get_tag_name(node, file);
                if tag_name.eq_ignore_ascii_case("script") {
                    // Only flag external scripts (those with a src attribute)
                    if Self::has_attribute(node, "src", file) {
                        let has_async = Self::has_attribute(node, "async", file);
                        let has_defer = Self::has_attribute(node, "defer", file);
                        let has_type_module = Self::is_module_script(node, file);
                        if !has_async && !has_defer && !has_type_module {
                            diagnostics.push(Diagnostic::warning(
                                Range::from_node(node),
                                "HTML011",
                                DiagnosticCategory::Style,
                                "Render-blocking <script> — add 'async' or 'defer' attribute",
                            ));
                        }
                    }
                }
            }
            true
        });

        diagnostics
    }

    // ===== Extended Helper Methods =====

    /// Check if head has a meta charset element
    fn has_meta_charset(node: &tree_sitter::Node<'_>, file: &ParsedFile) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "element" || child.kind() == "self_closing_tag" {
                let tag = Self::get_tag_name(&child, file);
                if tag.eq_ignore_ascii_case("meta") {
                    if Self::has_attribute(&child, "charset", file) {
                        return true;
                    }
                    // Also check http-equiv="Content-Type"
                    if let Some(val) = Self::get_attribute_value(&child, "http-equiv", file) {
                        if strip_quotes(val)
                            .eq_ignore_ascii_case("content-type")
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Check if a script element has type="module"
    fn is_module_script(node: &tree_sitter::Node<'_>, file: &ParsedFile) -> bool {
        if let Some(val) = Self::get_attribute_value(node, "type", file) {
            return strip_quotes(val).eq_ignore_ascii_case("module");
        }
        false
    }
}

/// Strip surrounding quotes from an attribute value
fn strip_quotes(s: &str) -> &str {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        &s[1..s.len() - 1]
    } else {
        s
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/lint/html_rules.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/lint/html_rules.rs` captured during libs codegen standardization.
```
