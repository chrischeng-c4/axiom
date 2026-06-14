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
