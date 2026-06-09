//! TOML lint checker (line-based)
//!
//! Rules:
//! - TM001: Syntax error (toml parse failure)
//! - TM002: Duplicate table header
//! - TM003: Empty table (header with no key-value pairs)
//! - TM004: Deprecated Cargo.toml keys
//! - TM005: Long string values (> 200 chars, suggest multi-line)

use std::collections::HashMap;

use super::Checker;
use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range};
use crate::syntax::{Language, ParsedFile};

/// TOML checker (line-based analysis)
pub struct TomlChecker;

impl TomlChecker {
    pub fn new() -> Self {
        Self
    }

    /// TM001: Attempt to parse as TOML; report first syntax error.
    fn check_syntax(&self, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        if let Err(err) = source.parse::<toml::Value>() {
            let msg = err.message();
            let span = err.span();
            let (line, col) = if let Some(range) = span {
                offset_to_line_col(source, range.start)
            } else {
                (0, 0)
            };
            diagnostics.push(Diagnostic::new(
                line_range(line as u32, col as u32),
                DiagnosticSeverity::Error,
                "TM001",
                DiagnosticCategory::Syntax,
                format!("TOML syntax error: {}", msg),
            ));
        }
        diagnostics
    }

    /// TM002: Duplicate table headers.
    fn check_duplicate_tables(&self, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut seen: HashMap<String, u32> = HashMap::new();

        for (line_idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            // Match [table] but not [[array-table]]
            if trimmed.starts_with('[') && !trimmed.starts_with("[[") {
                let header = extract_table_header(trimmed);
                if let Some(header) = header {
                    let normalized = header.to_lowercase();
                    if let Some(&prev_line) = seen.get(&normalized) {
                        diagnostics.push(Diagnostic::new(
                            full_line_range(line_idx as u32),
                            DiagnosticSeverity::Warning,
                            "TM002",
                            DiagnosticCategory::Logic,
                            format!(
                                "Duplicate table header '{}' (first seen at line {})",
                                header,
                                prev_line + 1
                            ),
                        ));
                    } else {
                        seen.insert(normalized, line_idx as u32);
                    }
                }
            }
        }
        diagnostics
    }

    /// TM003: Empty table — table header with no key-value pairs before next header.
    fn check_empty_tables(&self, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut last_header: Option<(String, u32)> = None;
        let mut has_content = false;

        for (line_idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with('[') {
                // Check previous header
                if let Some((ref name, header_line)) = last_header {
                    if !has_content {
                        diagnostics.push(Diagnostic::new(
                            full_line_range(header_line),
                            DiagnosticSeverity::Warning,
                            "TM003",
                            DiagnosticCategory::Style,
                            format!("Empty table '{}' — no key-value pairs", name),
                        ));
                    }
                }
                let header = if trimmed.starts_with("[[") {
                    extract_array_table_header(trimmed)
                } else {
                    extract_table_header(trimmed)
                };
                last_header = header.map(|h| (h.to_string(), line_idx as u32));
                has_content = false;
                continue;
            }

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Any non-header, non-empty, non-comment line counts as content
            if last_header.is_some() && trimmed.contains('=') {
                has_content = true;
            }
        }

        // Check last header
        if let Some((ref name, header_line)) = last_header {
            if !has_content {
                diagnostics.push(Diagnostic::new(
                    full_line_range(header_line),
                    DiagnosticSeverity::Warning,
                    "TM003",
                    DiagnosticCategory::Style,
                    format!("Empty table '{}' — no key-value pairs", name),
                ));
            }
        }

        diagnostics
    }

    /// TM004: Deprecated Cargo.toml keys.
    fn check_deprecated_keys(&self, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let deprecated = [
            (
                "authors-email",
                "use [package.authors] with email in angle brackets",
            ),
            ("crate_type", "use [lib] crate-type instead"),
        ];

        for (line_idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }

            for (key, suggestion) in &deprecated {
                if let Some(eq_pos) = trimmed.find('=') {
                    let lhs = trimmed[..eq_pos].trim();
                    if lhs == *key {
                        diagnostics.push(Diagnostic::new(
                            full_line_range(line_idx as u32),
                            DiagnosticSeverity::Warning,
                            "TM004",
                            DiagnosticCategory::Style,
                            format!("Deprecated key '{}' — {}", key, suggestion),
                        ));
                    }
                }
            }
        }
        diagnostics
    }

    /// TM005: Long string values (> 200 chars on single line).
    fn check_long_strings(&self, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (line_idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }

            if let Some(eq_pos) = trimmed.find('=') {
                let value = trimmed[eq_pos + 1..].trim();
                // Check if it's a quoted string value > 200 chars
                if (value.starts_with('"') || value.starts_with('\'')) && value.len() > 200 {
                    diagnostics.push(Diagnostic::new(
                        full_line_range(line_idx as u32),
                        DiagnosticSeverity::Hint,
                        "TM005",
                        DiagnosticCategory::Style,
                        format!(
                            "String value is {} chars — consider using a multi-line string",
                            value.len()
                        ),
                    ));
                }
            }
        }
        diagnostics
    }
    // =========================================================================
    // AST-based checks (tree-sitter-toml node kinds) — R3
    // =========================================================================

    /// TM001 via AST: report tree-sitter parse errors.
    fn ast_check_syntax(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        file.collect_errors()
            .into_iter()
            .map(|err| {
                let (row, col) = err.start_position;
                let pos = Position::new(
                    (row.saturating_sub(1)) as u32,
                    (col.saturating_sub(1)) as u32,
                );
                Diagnostic::new(
                    Range::new(pos, pos),
                    DiagnosticSeverity::Error,
                    "TM001",
                    DiagnosticCategory::Syntax,
                    "TOML syntax error",
                )
            })
            .collect()
    }

    /// TM002 via AST: detect duplicate table headers.
    fn ast_check_duplicate_tables(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let source = file.source.as_bytes();
        let mut seen: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

        file.walk(|node, _depth| {
            // tree-sitter-toml: table nodes have kind "table" or "table_array_element"
            if node.kind() == "table" {
                if let Some(key_node) = node.child_by_field_name("key").or_else(|| {
                    (0..node.child_count())
                        .filter_map(|i| node.child(i))
                        .find(|n| n.kind() == "key" || n.kind() == "dotted_key")
                }) {
                    if let Ok(key) = key_node.utf8_text(source) {
                        let line = node.start_position().row as u32;
                        if let Some(&prev) = seen.get(key) {
                            diags.push(Diagnostic::new(
                                full_line_range(line),
                                DiagnosticSeverity::Error,
                                "TM002",
                                DiagnosticCategory::Logic,
                                format!("Duplicate table '[{}]' (first at line {})", key, prev + 1),
                            ));
                        } else {
                            seen.insert(key.to_string(), line);
                        }
                    }
                }
            }
            true
        });
        diags
    }

    /// TM003 via AST: empty table (table header with no key-value pairs).
    fn ast_check_empty_tables(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let source = file.source.as_bytes();

        file.walk(|node, _depth| {
            if node.kind() == "table" {
                let has_pairs = (0..node.child_count())
                    .filter_map(|i| node.child(i))
                    .any(|c| matches!(c.kind(), "pair" | "keyval"));
                if !has_pairs {
                    let name = node
                        .child_by_field_name("key")
                        .and_then(|n| n.utf8_text(source).ok())
                        .unwrap_or("?");
                    let line = node.start_position().row as u32;
                    diags.push(Diagnostic::new(
                        full_line_range(line),
                        DiagnosticSeverity::Warning,
                        "TM003",
                        DiagnosticCategory::Style,
                        format!("Table '[{}]' is empty", name),
                    ));
                }
            }
            true
        });
        diags
    }
}

impl Default for TomlChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a single-line Range at a given 0-indexed line.
fn full_line_range(line: u32) -> Range {
    Range::new(Position::new(line, 0), Position::new(line, u32::MAX))
}

/// Build a Range at a specific column.
fn line_range(line: u32, col: u32) -> Range {
    Range::new(Position::new(line, col), Position::new(line, u32::MAX))
}

/// Convert byte offset to (line, col).
fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 0;
    let mut col = 0;
    for (i, ch) in source.chars().enumerate() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// Extract table name from `[name]`.
fn extract_table_header(trimmed: &str) -> Option<&str> {
    let inner = trimmed.strip_prefix('[')?.strip_suffix(']')?;
    let inner = inner.trim();
    if inner.is_empty() {
        None
    } else {
        Some(inner)
    }
}

/// Extract table name from `[[name]]`.
fn extract_array_table_header(trimmed: &str) -> Option<&str> {
    let inner = trimmed.strip_prefix("[[")?.strip_suffix("]]")?;
    let inner = inner.trim();
    if inner.is_empty() {
        None
    } else {
        Some(inner)
    }
}

impl Checker for TomlChecker {
    fn language(&self) -> Language {
        Language::Toml
    }

    fn check(&self, file: &ParsedFile, _config: &LintConfig) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // R3: AST-based syntax error detection when a real tree is present.
        // The TOML library's own parser handles TM001 (syntax errors) reliably
        // regardless of mode; additional semantic checks use AST node walking.
        if file.language == Language::Toml && !file.has_errors && !file.is_line_based {
            // TM001 via AST: tree-sitter parse errors (supplemental to toml crate)
            diagnostics.extend(self.ast_check_syntax(file));
            // Semantic checks using AST node matching
            diagnostics.extend(self.ast_check_duplicate_tables(file));
            diagnostics.extend(self.ast_check_empty_tables(file));
            // Cargo-specific and long-string checks remain line-based
            diagnostics.extend(self.check_deprecated_keys(&file.source));
            diagnostics.extend(self.check_long_strings(&file.source));
        } else {
            // Line-based fallback (uses toml crate for TM001)
            diagnostics.extend(self.check_syntax(&file.source));
            diagnostics.extend(self.check_duplicate_tables(&file.source));
            diagnostics.extend(self.check_empty_tables(&file.source));
            diagnostics.extend(self.check_deprecated_keys(&file.source));
            diagnostics.extend(self.check_long_strings(&file.source));
        }

        diagnostics
    }

    fn available_rules(&self) -> Vec<&'static str> {
        vec![
            "TM001", // Syntax error
            "TM002", // Duplicate table header
            "TM003", // Empty table
            "TM004", // Deprecated Cargo.toml keys
            "TM005", // Long string values
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::LintConfig;

    fn make_file(source: &str) -> ParsedFile {
        ParsedFile::line_based(source.to_string(), Language::Toml)
    }

    fn codes(diagnostics: &[Diagnostic]) -> Vec<&str> {
        diagnostics.iter().map(|d| d.code.as_str()).collect()
    }

    #[test]
    fn test_syntax_error() {
        let source = "[package\nname = \"test\"\n";
        let file = make_file(source);
        let checker = TomlChecker::new();
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            codes(&diags).contains(&"TM001"),
            "expected TM001 for syntax error, got {:?}",
            codes(&diags)
        );
    }

    #[test]
    fn test_duplicate_table() {
        let source = "[package]\nname = \"a\"\n[package]\nname = \"b\"\n";
        let file = make_file(source);
        let checker = TomlChecker::new();
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            codes(&diags).contains(&"TM002"),
            "expected TM002 for duplicate table, got {:?}",
            codes(&diags)
        );
    }

    #[test]
    fn test_empty_table() {
        let source = "[package]\nname = \"a\"\n[empty]\n[other]\nval = 1\n";
        let file = make_file(source);
        let checker = TomlChecker::new();
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            codes(&diags).contains(&"TM003"),
            "expected TM003 for empty table, got {:?}",
            codes(&diags)
        );
    }

    #[test]
    fn test_deprecated_key() {
        let source = "[package]\nauthors-email = \"a@b.com\"\n";
        let file = make_file(source);
        let checker = TomlChecker::new();
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            codes(&diags).contains(&"TM004"),
            "expected TM004 for deprecated key, got {:?}",
            codes(&diags)
        );
    }

    #[test]
    fn test_long_string() {
        let long_val = "x".repeat(210);
        let source = format!("[package]\ndescription = \"{}\"\n", long_val);
        let file = make_file(&source);
        let checker = TomlChecker::new();
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            codes(&diags).contains(&"TM005"),
            "expected TM005 for long string, got {:?}",
            codes(&diags)
        );
    }

    #[test]
    fn test_clean_toml() {
        let source = "[package]\nname = \"test\"\nversion = \"0.1.0\"\n";
        let file = make_file(source);
        let checker = TomlChecker::new();
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            diags.is_empty(),
            "unexpected diagnostics on clean file: {:?}",
            codes(&diags)
        );
    }
}
