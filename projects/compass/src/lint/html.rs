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
