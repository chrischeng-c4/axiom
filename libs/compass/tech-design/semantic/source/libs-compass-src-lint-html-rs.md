---
id: libs-compass-src-lint-html-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/lint/html.rs`.
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

# Standardized libs/compass/src/lint/html.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/lint/html.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `HtmlChecker` | libs/compass/src/lint/html.rs | struct | pub | 11 | pub struct HtmlChecker; |
| `new` | libs/compass/src/lint/html.rs | function | pub | 14 | pub fn new() -> Self { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! HTML code checker

use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, Range};
use crate::syntax::{Language, ParsedFile};

/// Deprecated HTML tags that should not be used in modern HTML
const DEPRECATED_TAGS: &[&str] = &["center", "font", "marquee"];

/// HTML checker
pub struct HtmlChecker;

impl HtmlChecker {
    pub fn new() -> Self {
        Self
    }

    /// HTML001: Missing `alt` attribute on `<img>` elements
    ///
    /// The `alt` attribute provides alternative text for screen readers
    /// and when images fail to load. Required for accessibility (WCAG).
    fn check_img_alt(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "element" || node.kind() == "self_closing_tag" {
                let tag_name = Self::get_tag_name(node, file);
                if tag_name.eq_ignore_ascii_case("img") {
                    if !Self::has_attribute(node, "alt", file) {
                        diagnostics.push(Diagnostic::warning(
                            Range::from_node(node),
                            "HTML001",
                            DiagnosticCategory::Style,
                            "Missing 'alt' attribute on <img> — required for accessibility",
                        ));
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// HTML002: Missing `lang` attribute on `<html>` element
    ///
    /// The `lang` attribute helps screen readers select the correct
    /// pronunciation and assists search engines with content language.
    fn check_html_lang(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "element" {
                let tag_name = Self::get_tag_name(node, file);
                if tag_name.eq_ignore_ascii_case("html") {
                    if !Self::has_attribute(node, "lang", file) {
                        diagnostics.push(Diagnostic::warning(
                            Range::from_node(node),
                            "HTML002",
                            DiagnosticCategory::Style,
                            "Missing 'lang' attribute on <html> — required for accessibility",
                        ));
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// HTML003: Empty `href` attribute on `<a>` elements
    ///
    /// An empty `href=""` creates a link that navigates to the current
    /// page, which is usually unintentional. Use `href="#"` or a real URL.
    fn check_empty_href(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "element" {
                let tag_name = Self::get_tag_name(node, file);
                if tag_name.eq_ignore_ascii_case("a") {
                    if let Some(href_value) = Self::get_attribute_value(node, "href", file) {
                        let trimmed = href_value.trim();
                        if trimmed.is_empty() || trimmed == "\"\"" || trimmed == "''" {
                            diagnostics.push(Diagnostic::warning(
                                Range::from_node(node),
                                "HTML003",
                                DiagnosticCategory::Logic,
                                "Empty 'href' attribute on <a> — use a valid URL or '#'",
                            ));
                        }
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// HTML004: Deprecated HTML tags
    ///
    /// Tags like `<center>`, `<font>`, and `<marquee>` are deprecated
    /// in HTML5. Use CSS for styling instead.
    fn check_deprecated_tags(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "element" || node.kind() == "self_closing_tag" {
                let tag_name = Self::get_tag_name(node, file);
                let tag_lower = tag_name.to_ascii_lowercase();
                if DEPRECATED_TAGS.contains(&tag_lower.as_str()) {
                    diagnostics.push(Diagnostic::warning(
                        Range::from_node(node),
                        "HTML004",
                        DiagnosticCategory::Style,
                        format!(
                            "Deprecated HTML tag <{}> — use CSS for styling instead",
                            tag_lower
                        ),
                    ));
                }
            }
            true
        });

        diagnostics
    }

    /// HTML005: Missing `<title>` element in `<head>`
    ///
    /// Every HTML document should have a `<title>` element inside `<head>`
    /// for proper SEO and browser tab labeling.
    fn check_missing_title(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "element" {
                let tag_name = Self::get_tag_name(node, file);
                if tag_name.eq_ignore_ascii_case("head") {
                    // Search for a <title> child element
                    let has_title = Self::has_child_tag(node, "title", file);
                    if !has_title {
                        diagnostics.push(Diagnostic::warning(
                            Range::from_node(node),
                            "HTML005",
                            DiagnosticCategory::Style,
                            "Missing <title> element in <head>",
                        ));
                    }
                }
            }
            true
        });

        diagnostics
    }

    // ===== Helper Methods =====

    /// Extract the tag name from an element or self_closing_tag node.
    fn get_tag_name<'a>(node: &tree_sitter::Node<'a>, file: &'a ParsedFile) -> &'a str {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if matches!(child.kind(), "start_tag" | "self_closing_tag") {
                let mut ic = child.walk();
                for inner in child.children(&mut ic) {
                    if inner.kind() == "tag_name" {
                        return file.node_text(&inner);
                    }
                }
            }
            if child.kind() == "tag_name" {
                return file.node_text(&child);
            }
        }
        ""
    }

    /// Check whether an element has a given attribute name.
    fn has_attribute(node: &tree_sitter::Node<'_>, attr_name: &str, file: &ParsedFile) -> bool {
        Self::get_attribute_value(node, attr_name, file).is_some()
            || Self::find_attr_name(node, attr_name, file)
    }

    /// Check attribute_name children for a match (fallback for missing field).
    fn find_attr_name(node: &tree_sitter::Node<'_>, attr_name: &str, file: &ParsedFile) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if matches!(child.kind(), "start_tag" | "self_closing_tag") {
                let mut ic = child.walk();
                for inner in child.children(&mut ic) {
                    if inner.kind() == "attribute" {
                        let mut ac = inner.walk();
                        for attr_child in inner.children(&mut ac) {
                            if attr_child.kind() == "attribute_name"
                                && file.node_text(&attr_child).eq_ignore_ascii_case(attr_name)
                            {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Get the value of a named attribute, if present.
    fn get_attribute_value<'a>(
        node: &tree_sitter::Node<'a>,
        attr_name: &str,
        file: &'a ParsedFile,
    ) -> Option<&'a str> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if matches!(child.kind(), "start_tag" | "self_closing_tag") {
                let mut ic = child.walk();
                for inner in child.children(&mut ic) {
                    if inner.kind() != "attribute" {
                        continue;
                    }
                    let mut found = false;
                    let mut ac = inner.walk();
                    for attr_child in inner.children(&mut ac) {
                        if attr_child.kind() == "attribute_name"
                            && file.node_text(&attr_child).eq_ignore_ascii_case(attr_name)
                        {
                            found = true;
                        }
                        if found
                            && matches!(
                                attr_child.kind(),
                                "quoted_attribute_value" | "attribute_value"
                            )
                        {
                            return Some(file.node_text(&attr_child));
                        }
                    }
                }
            }
        }
        None
    }

    /// Check whether an element has a child element with the given tag name.
    fn has_child_tag(node: &tree_sitter::Node<'_>, tag: &str, file: &ParsedFile) -> bool {
        let mut cursor = node.walk();
        let result = node.children(&mut cursor).any(|c| {
            c.kind() == "element" && Self::get_tag_name(&c, file).eq_ignore_ascii_case(tag)
        });
        result
    }
}

impl Default for HtmlChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Checker for HtmlChecker {
    fn language(&self) -> Language {
        Language::Html
    }

    fn check(&self, file: &ParsedFile, _config: &LintConfig) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Check for syntax errors from tree-sitter
        if file.has_errors {
            file.walk(|node, _depth| {
                if node.is_error() || node.is_missing() {
                    diagnostics.push(Diagnostic::error(
                        Range::from_node(node),
                        "HTML000",
                        DiagnosticCategory::Syntax,
                        "Syntax error",
                    ));
                }
                true
            });
        }

        // Run all checks
        diagnostics.extend(self.check_img_alt(file));
        diagnostics.extend(self.check_html_lang(file));
        diagnostics.extend(self.check_empty_href(file));
        diagnostics.extend(self.check_deprecated_tags(file));
        diagnostics.extend(self.check_missing_title(file));

        diagnostics
    }

    fn available_rules(&self) -> Vec<&'static str> {
        vec![
            "HTML000", // Syntax error
            "HTML001", // Missing alt on img
            "HTML002", // Missing lang on html
            "HTML003", // Empty href on a
            "HTML004", // Deprecated tags
            "HTML005", // Missing title in head
        ]
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/lint/html.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/lint/html.rs` captured during libs codegen standardization.
```
