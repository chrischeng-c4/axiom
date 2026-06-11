//! Extended CSS lint rules (CSS006 - CSS010)
//!
//! Split from css.rs to keep files under 500 lines.

use crate::syntax::ParsedFile;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Range};
use std::collections::{HashMap, HashSet};

use super::css::CssChecker;

/// Known vendor prefixes and their standard property counterparts
const VENDOR_PREFIXES: &[&str] = &["-webkit-", "-moz-", "-ms-", "-o-"];

/// Shorthand property groups: (shorthand, individual properties)
const SHORTHAND_GROUPS: &[(&str, &[&str])] = &[
    ("margin", &["margin-top", "margin-right", "margin-bottom", "margin-left"]),
    ("padding", &["padding-top", "padding-right", "padding-bottom", "padding-left"]),
    ("border", &["border-width", "border-style", "border-color"]),
    ("border-radius", &[
        "border-top-left-radius", "border-top-right-radius",
        "border-bottom-right-radius", "border-bottom-left-radius",
    ]),
];

impl CssChecker {
    /// CSS006: Vendor prefix without standard property
    ///
    /// Using a vendor-prefixed property without the standard fallback
    /// means the rule won't work once the prefix is dropped.
    pub(crate) fn check_vendor_prefix(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        // Collect all properties per rule_set
        let mut rule_sets: Vec<(Range, Vec<(String, Range)>)> = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "rule_set" || node.kind() == "keyframe_block" {
                let props = collect_properties(node, file);
                if !props.is_empty() {
                    rule_sets.push((Range::from_node(node), props));
                }
            }
            true
        });

        for (_rule_range, props) in &rule_sets {
            let prop_names: HashSet<&str> = props.iter().map(|(n, _)| n.as_str()).collect();
            for (name, range) in props {
                for prefix in VENDOR_PREFIXES {
                    if let Some(standard) = name.strip_prefix(prefix) {
                        if !standard.is_empty() && !prop_names.contains(standard) {
                            diagnostics.push(Diagnostic::warning(
                                *range,
                                "CSS006",
                                DiagnosticCategory::Style,
                                format!(
                                    "Vendor-prefixed '{}' without standard '{}' fallback",
                                    name, standard
                                ),
                            ));
                        }
                        break;
                    }
                }
            }
        }

        diagnostics
    }

    /// CSS007: Shorthand property optimization opportunity
    ///
    /// When all individual properties of a shorthand group are present,
    /// they can be combined into the shorthand form.
    pub(crate) fn check_shorthand_optimization(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "rule_set" {
                let props = collect_properties(node, file);
                let prop_names: HashSet<&str> = props.iter().map(|(n, _)| n.as_str()).collect();

                for (shorthand, individuals) in SHORTHAND_GROUPS {
                    let present: Vec<&&str> = individuals
                        .iter()
                        .filter(|p| prop_names.contains(**p))
                        .collect();
                    if present.len() == individuals.len() {
                        // All individual properties present
                        diagnostics.push(Diagnostic::new(
                            Range::from_node(node),
                            DiagnosticSeverity::Information,
                            "CSS007",
                            DiagnosticCategory::Style,
                            format!(
                                "Consider using shorthand '{}' instead of individual properties",
                                shorthand
                            ),
                        ));
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// CSS008: Inconsistent color format in same file
    ///
    /// Mixing hex (#fff), rgb(), hsl() etc. in the same file reduces
    /// consistency. Pick one format and use it throughout.
    pub(crate) fn check_color_format(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut formats: HashMap<&str, usize> = HashMap::new();
        let mut first_ranges: HashMap<&str, Range> = HashMap::new();

        file.walk(|node, _depth| {
            if node.kind() == "color_value" {
                let text = file.node_text(node);
                if text.starts_with('#') {
                    *formats.entry("hex").or_insert(0) += 1;
                    first_ranges.entry("hex").or_insert_with(|| Range::from_node(node));
                }
            }
            if node.kind() == "call_expression" {
                let text = file.node_text(node);
                let lower = text.to_ascii_lowercase();
                if lower.starts_with("rgb") {
                    *formats.entry("rgb").or_insert(0) += 1;
                    first_ranges.entry("rgb").or_insert_with(|| Range::from_node(node));
                } else if lower.starts_with("hsl") {
                    *formats.entry("hsl").or_insert(0) += 1;
                    first_ranges.entry("hsl").or_insert_with(|| Range::from_node(node));
                }
            }
            true
        });

        if formats.len() > 1 {
            let format_list: Vec<&str> = formats.keys().copied().collect();
            // Report on the file's first node
            let range = first_ranges
                .values()
                .min_by_key(|r| (r.start.line, r.start.character))
                .copied()
                .unwrap_or_default();
            diagnostics.push(Diagnostic::new(
                range,
                DiagnosticSeverity::Information,
                "CSS008",
                DiagnosticCategory::Style,
                format!(
                    "Inconsistent color formats in file: {} — pick one format",
                    format_list.join(", ")
                ),
            ));
        }

        diagnostics
    }

    /// CSS009: Zero value with unnecessary unit
    ///
    /// `0px`, `0em`, `0rem` etc. are equivalent to `0`. The unit is
    /// unnecessary and adds noise.
    pub(crate) fn check_zero_with_unit(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "integer_value" || node.kind() == "float_value" {
                let text = file.node_text(node);
                // Check if it's a zero value with a unit
                if is_zero_with_unit(text) {
                    diagnostics.push(Diagnostic::new(
                        Range::from_node(node),
                        DiagnosticSeverity::Information,
                        "CSS009",
                        DiagnosticCategory::Style,
                        format!("Unnecessary unit on zero value '{}' — use '0' instead", text),
                    ));
                }
            }
            true
        });

        diagnostics
    }

    /// CSS010: Missing `font-display` in `@font-face`
    ///
    /// Without `font-display`, browsers use their default behavior which
    /// may cause invisible text during font loading (FOIT).
    pub(crate) fn check_font_display(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            // tree-sitter-css: @font-face is an at_rule or font_face_statement
            if matches!(node.kind(), "at_rule" | "font_face_statement") {
                let text = file.node_text(node);
                if text.trim_start().starts_with("@font-face") {
                    let has_font_display = has_property_in_block(node, "font-display", file);
                    if !has_font_display {
                        diagnostics.push(Diagnostic::warning(
                            Range::from_node(node),
                            "CSS010",
                            DiagnosticCategory::Style,
                            "Missing 'font-display' in @font-face — add it to control font loading behavior",
                        ));
                    }
                }
            }
            true
        });

        diagnostics
    }
}

/// Collect all property names and their ranges within a rule set block
fn collect_properties(
    node: &tree_sitter::Node<'_>,
    file: &ParsedFile,
) -> Vec<(String, Range)> {
    let mut props = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "block" {
            let mut ic = child.walk();
            for inner in child.children(&mut ic) {
                if inner.kind() == "declaration" {
                    let mut dc = inner.walk();
                    for dchild in inner.children(&mut dc) {
                        if dchild.kind() == "property_name" {
                            let name = file.node_text(&dchild).to_string();
                            props.push((name, Range::from_node(&dchild)));
                            break;
                        }
                    }
                }
            }
        }
    }
    props
}

/// Check if a value is a zero with a unit suffix (e.g., "0px", "0em")
fn is_zero_with_unit(text: &str) -> bool {
    let text = text.trim();
    if text == "0" || text.is_empty() {
        return false;
    }
    // Extract numeric part
    let numeric: String = text.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
    if numeric.is_empty() {
        return false;
    }
    let has_unit = text.len() > numeric.len();
    if !has_unit {
        return false;
    }
    // Check if numeric value is zero
    matches!(numeric.as_str(), "0" | "0.0" | "0.00" | ".0" | "00")
}

/// Check if a block node contains a specific property name
fn has_property_in_block(
    node: &tree_sitter::Node<'_>,
    prop_name: &str,
    file: &ParsedFile,
) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "block" {
            let mut ic = child.walk();
            for inner in child.children(&mut ic) {
                if inner.kind() == "declaration" {
                    let mut dc = inner.walk();
                    for dchild in inner.children(&mut dc) {
                        if dchild.kind() == "property_name" {
                            if file.node_text(&dchild).eq_ignore_ascii_case(prop_name) {
                                return true;
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
    false
}
