//! CSS code checker

use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Range};
use crate::syntax::{Language, ParsedFile};
use std::collections::HashMap;

/// CSS checker
pub struct CssChecker;

impl CssChecker {
    pub fn new() -> Self {
        Self
    }

    /// CSS001: Detect duplicate selectors in the same stylesheet
    fn check_duplicate_selectors(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut seen: HashMap<String, Range> = HashMap::new();
        file.walk(|node, _depth| {
            if node.kind() == "rule_set" {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "selectors" {
                        let sel = file.node_text(&child).trim().to_string();
                        if let Some(prev) = seen.get(&sel) {
                            diagnostics.push(Diagnostic::warning(
                                Range::from_node(&child),
                                "CSS001",
                                DiagnosticCategory::Style,
                                format!(
                                    "Duplicate selector '{}' (first at line {})",
                                    sel,
                                    prev.start.line + 1
                                ),
                            ));
                        } else {
                            seen.insert(sel, Range::from_node(&child));
                        }
                        break;
                    }
                }
            }
            true
        });
        diagnostics
    }

    /// CSS002: Detect `!important` usage
    fn check_important_usage(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "important" {
                diagnostics.push(Diagnostic::new(
                    Range::from_node(node),
                    DiagnosticSeverity::Information,
                    "CSS002",
                    DiagnosticCategory::Style,
                    "Avoid '!important' — it breaks the natural cascade and is hard to override",
                ));
                return true;
            }
            if node.kind() == "declaration" {
                let text = file.node_text(node);
                if text.contains("!important") {
                    diagnostics.push(Diagnostic::new(
                        Range::from_node(node),
                        DiagnosticSeverity::Information,
                        "CSS002",
                        DiagnosticCategory::Style,
                        "Avoid '!important' — it breaks the natural cascade",
                    ));
                }
            }
            true
        });
        diagnostics
    }

    /// CSS003: Detect `@import` statements
    fn check_import_usage(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "import_statement" {
                diagnostics.push(Diagnostic::warning(
                    Range::from_node(node),
                    "CSS003",
                    DiagnosticCategory::Style,
                    "Avoid '@import' — use <link> tag or a bundler for better performance",
                ));
            }
            true
        });
        diagnostics
    }

    /// CSS004: Detect empty rule sets
    fn check_empty_rules(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "rule_set" {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "block" {
                        let has_decl = {
                            let mut ic = child.walk();
                            let result = child.children(&mut ic).any(|c| {
                                c.kind() == "declaration"
                                    || c.kind() == "rule_set"
                                    || c.kind() == "at_rule"
                            });
                            result
                        };
                        if !has_decl {
                            diagnostics.push(Diagnostic::warning(
                                Range::from_node(node),
                                "CSS004",
                                DiagnosticCategory::Style,
                                "Empty rule set — remove it or add declarations",
                            ));
                        }
                    }
                }
            }
            true
        });
        diagnostics
    }

    /// CSS005: Detect universal selector `*` usage
    fn check_universal_selector(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "universal_selector" {
                diagnostics.push(Diagnostic::new(
                    Range::from_node(node),
                    DiagnosticSeverity::Information,
                    "CSS005",
                    DiagnosticCategory::Style,
                    "Universal selector '*' may impact performance — use specific selectors",
                ));
            }
            true
        });
        diagnostics
    }

    /// CSS006: no-id-selectors — detect #id selectors
    fn check_no_id_selectors(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "id_selector" {
                diagnostics.push(Diagnostic::new(
                    Range::from_node(node),
                    DiagnosticSeverity::Information,
                    "CSS006",
                    DiagnosticCategory::Style,
                    "Avoid ID selectors — they have high specificity and are hard to override",
                ));
            }
            true
        });
        diagnostics
    }

    /// CSS007: shorthand-property-overrides — longhand after shorthand in same rule
    fn check_shorthand_overrides(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let shorthands = [
            (
                "margin",
                &["margin-top", "margin-right", "margin-bottom", "margin-left"][..],
            ),
            (
                "padding",
                &[
                    "padding-top",
                    "padding-right",
                    "padding-bottom",
                    "padding-left",
                ],
            ),
            (
                "border",
                &[
                    "border-top",
                    "border-right",
                    "border-bottom",
                    "border-left",
                    "border-width",
                    "border-style",
                    "border-color",
                ],
            ),
            (
                "background",
                &[
                    "background-color",
                    "background-image",
                    "background-position",
                ],
            ),
        ];
        file.walk(|node, _depth| {
            if node.kind() == "rule_set" || node.kind() == "block" {
                let mut seen_short: Vec<&str> = Vec::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "declaration" {
                        if let Some(prop) = child.child_by_field_name("name") {
                            let name = file.node_text(&prop).trim().to_string();
                            // Check if this is a shorthand
                            for &(short, _) in &shorthands {
                                if name == short {
                                    seen_short.push(short);
                                }
                            }
                            // Check if this longhand follows its shorthand
                            for &(short, longs) in &shorthands {
                                if seen_short.contains(&short) && longs.contains(&name.as_str()) {
                                    diagnostics.push(Diagnostic::warning(
                                        Range::from_node(&child),
                                        "CSS007",
                                        DiagnosticCategory::Logic,
                                        format!(
                                            "'{}' overrides part of shorthand '{}'",
                                            name, short
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

    /// CSS008: z-index-max — z-index > 9999
    fn check_z_index_max(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "declaration" {
                if let Some(prop) = node.child_by_field_name("name") {
                    if file.node_text(&prop).trim() == "z-index" {
                        if let Some(val) = node.child_by_field_name("value") {
                            let val_text = file.node_text(&val).trim().to_string();
                            if let Ok(n) = val_text.parse::<i64>() {
                                if n > 9999 {
                                    diagnostics.push(Diagnostic::warning(
                                        Range::from_node(node),
                                        "CSS008",
                                        DiagnosticCategory::Style,
                                        format!(
                                            "z-index value {} exceeds 9999 — use a lower value",
                                            n
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

    /// CSS009: color-no-invalid-hex — invalid hex color length
    fn check_invalid_hex_color(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "color_value" {
                let text = file.node_text(node).trim().to_string();
                if text.starts_with('#') {
                    let hex_part = &text[1..];
                    let len = hex_part.len();
                    if len != 3 && len != 4 && len != 6 && len != 8 {
                        diagnostics.push(Diagnostic::error(
                            Range::from_node(node),
                            "CSS009",
                            DiagnosticCategory::Syntax,
                            format!(
                                "Invalid hex color '{}' — must be 3, 4, 6, or 8 hex digits",
                                text
                            ),
                        ));
                    }
                }
            }
            true
        });
        diagnostics
    }

    /// CSS010: font-family-no-missing-generic — font-family without generic fallback
    fn check_font_family_generic(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let generics = [
            "serif",
            "sans-serif",
            "monospace",
            "cursive",
            "fantasy",
            "system-ui",
            "ui-serif",
            "ui-sans-serif",
            "ui-monospace",
            "ui-rounded",
        ];
        file.walk(|node, _depth| {
            if node.kind() == "declaration" {
                if let Some(prop) = node.child_by_field_name("name") {
                    if file.node_text(&prop).trim() == "font-family" {
                        if let Some(val) = node.child_by_field_name("value") {
                            let val_text = file.node_text(&val).to_lowercase();
                            let has_generic = generics.iter().any(|g| val_text.contains(g));
                            if !has_generic {
                                diagnostics.push(Diagnostic::warning(
                                    Range::from_node(node), "CSS010", DiagnosticCategory::Style,
                                    "font-family missing a generic family keyword (e.g., sans-serif)",
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

    /// CSS011: no-duplicate-properties — duplicate property names in same rule
    fn check_duplicate_properties(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "rule_set" || node.kind() == "block" {
                let mut seen: HashMap<String, usize> = HashMap::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "declaration" {
                        if let Some(prop) = child.child_by_field_name("name") {
                            let name = file.node_text(&prop).trim().to_string();
                            if let Some(&prev_line) = seen.get(&name) {
                                diagnostics.push(Diagnostic::warning(
                                    Range::from_node(&child),
                                    "CSS011",
                                    DiagnosticCategory::Style,
                                    format!(
                                        "Duplicate property '{}' (first at line {})",
                                        name, prev_line
                                    ),
                                ));
                            } else {
                                seen.insert(name, prop.start_position().row + 1);
                            }
                        }
                    }
                }
            }
            true
        });
        diagnostics
    }

    /// CSS012: declaration-block-no-shorthand-override — shorthand after longhand
    fn check_shorthand_after_longhand(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let shorthands = [
            (
                "margin",
                &["margin-top", "margin-right", "margin-bottom", "margin-left"][..],
            ),
            (
                "padding",
                &[
                    "padding-top",
                    "padding-right",
                    "padding-bottom",
                    "padding-left",
                ],
            ),
            (
                "border",
                &["border-top", "border-right", "border-bottom", "border-left"],
            ),
            (
                "background",
                &[
                    "background-color",
                    "background-image",
                    "background-position",
                ],
            ),
        ];
        file.walk(|node, _depth| {
            if node.kind() == "rule_set" || node.kind() == "block" {
                let mut seen_long: Vec<String> = Vec::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "declaration" {
                        if let Some(prop) = child.child_by_field_name("name") {
                            let name = file.node_text(&prop).trim().to_string();
                            for &(short, longs) in &shorthands {
                                if longs.contains(&name.as_str()) {
                                    seen_long.push(name.clone());
                                }
                                if name == short && seen_long.iter().any(|l| longs.contains(&l.as_str())) {
                                    diagnostics.push(Diagnostic::warning(
                                        Range::from_node(&child), "CSS012", DiagnosticCategory::Logic,
                                        format!("Shorthand '{}' overrides preceding longhand properties", short),
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

    /// CSS013: no-descending-specificity — #id rule after .class rule
    fn check_descending_specificity(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut seen_class = false;
        file.walk(|node, _depth| {
            if node.kind() == "class_selector" {
                seen_class = true;
            }
            if node.kind() == "id_selector" && seen_class {
                diagnostics.push(Diagnostic::new(
                    Range::from_node(node),
                    DiagnosticSeverity::Information,
                    "CSS013",
                    DiagnosticCategory::Style,
                    "ID selector appears after class selector — potential specificity issue",
                ));
            }
            true
        });
        diagnostics
    }

    /// CSS014: unit-no-unknown — detect non-standard CSS units
    fn check_unknown_units(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let known = [
            "px", "em", "rem", "%", "vh", "vw", "vmin", "vmax", "ch", "ex", "cm", "mm", "in", "pt",
            "pc", "s", "ms", "deg", "rad", "grad", "turn", "fr", "dpi", "dpcm", "dppx", "lh",
            "rlh", "dvh", "dvw", "svh", "svw", "lvh", "lvw", "cqw", "cqh",
        ];
        file.walk(|node, _depth| {
            if node.kind() == "declaration" {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "plain_value" {
                        let text = file.node_text(&child).trim().to_string();
                        // Check if it looks like number+unit
                        let unit_start = text.find(|c: char| c.is_alphabetic());
                        if let Some(idx) = unit_start {
                            if text[..idx].parse::<f64>().is_ok() {
                                let unit = &text[idx..];
                                if !known.contains(&unit) {
                                    diagnostics.push(Diagnostic::warning(
                                        Range::from_node(&child),
                                        "CSS014",
                                        DiagnosticCategory::Syntax,
                                        format!("Unknown CSS unit '{}'", unit),
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

    /// CSS015: block-no-empty — detect empty blocks in at-rules (e.g. @media)
    fn check_empty_at_rule_blocks(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        file.walk(|node, _depth| {
            if node.kind() == "at_rule"
                || node.kind() == "media_statement"
                || node.kind() == "supports_statement"
                || node.kind() == "keyframes_statement"
            {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "block" {
                        let mut inner = child.walk();
                        let has_content = child.children(&mut inner).any(|c| {
                            c.kind() != "{" && c.kind() != "}" && !c.kind().contains("comment")
                        });
                        if !has_content {
                            diagnostics.push(Diagnostic::warning(
                                Range::from_node(node),
                                "CSS015",
                                DiagnosticCategory::Style,
                                "Empty block — remove the at-rule or add declarations",
                            ));
                        }
                    }
                }
            }
            true
        });
        diagnostics
    }
}

impl Default for CssChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Checker for CssChecker {
    fn language(&self) -> Language {
        Language::Css
    }

    fn check(&self, file: &ParsedFile, _config: &LintConfig) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Check for syntax errors from tree-sitter
        if file.has_errors {
            file.walk(|node, _depth| {
                if node.is_error() || node.is_missing() {
                    diagnostics.push(Diagnostic::error(
                        Range::from_node(node),
                        "CSS000",
                        DiagnosticCategory::Syntax,
                        "Syntax error",
                    ));
                }
                true
            });
        }

        // Run all checks
        diagnostics.extend(self.check_duplicate_selectors(file));
        diagnostics.extend(self.check_important_usage(file));
        diagnostics.extend(self.check_import_usage(file));
        diagnostics.extend(self.check_empty_rules(file));
        diagnostics.extend(self.check_universal_selector(file));
        diagnostics.extend(self.check_no_id_selectors(file));
        diagnostics.extend(self.check_shorthand_overrides(file));
        diagnostics.extend(self.check_z_index_max(file));
        diagnostics.extend(self.check_invalid_hex_color(file));
        diagnostics.extend(self.check_font_family_generic(file));
        diagnostics.extend(self.check_duplicate_properties(file));
        diagnostics.extend(self.check_shorthand_after_longhand(file));
        diagnostics.extend(self.check_descending_specificity(file));
        diagnostics.extend(self.check_unknown_units(file));
        diagnostics.extend(self.check_empty_at_rule_blocks(file));

        diagnostics
    }

    fn available_rules(&self) -> Vec<&'static str> {
        vec![
            "CSS000", // Syntax error
            "CSS001", // Duplicate selectors
            "CSS002", // !important usage
            "CSS003", // @import usage
            "CSS004", // Empty rule sets
            "CSS005", // Universal selector *
            "CSS006", // no-id-selectors
            "CSS007", // shorthand-property-overrides
            "CSS008", // z-index-max
            "CSS009", // color-no-invalid-hex
            "CSS010", // font-family-no-missing-generic
            "CSS011", // no-duplicate-properties
            "CSS012", // declaration-block-no-shorthand-override
            "CSS013", // no-descending-specificity
            "CSS014", // unit-no-unknown
            "CSS015", // block-no-empty
        ]
    }
}
