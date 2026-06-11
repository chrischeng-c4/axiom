//! Mermaid diagram lint checker (line-based, no tree-sitter)

use std::collections::{HashMap, HashSet};

use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range};
use crate::syntax::{Language, ParsedFile};

const VALID_DIAGRAM_TYPES: &[&str] = &[
    "graph",
    "flowchart",
    "sequenceDiagram",
    "classDiagram",
    "stateDiagram",
    "stateDiagram-v2",
    "erDiagram",
    "gantt",
    "pie",
    "mindmap",
    "timeline",
    "gitgraph",
    "journey",
    "quadrantChart",
    "requirementDiagram",
    "C4Context",
];

/// Mermaid checker — line-based, no tree-sitter
pub struct MermaidChecker;

impl MermaidChecker {
    pub fn new() -> Self {
        Self
    }

    /// Extract the first non-empty, non-comment token from a line.
    fn first_token(line: &str) -> Option<&str> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("%%") {
            return None;
        }
        trimmed.split_whitespace().next()
    }

    /// Return the diagram type from the first meaningful line, or None.
    fn diagram_type<'a>(lines: &[&'a str]) -> Option<&'a str> {
        for line in lines {
            if let Some(tok) = Self::first_token(line) {
                return Some(tok);
            }
        }
        None
    }

    /// MM001: Unknown diagram type
    fn check_unknown_diagram(&self, lines: &[&str]) -> Vec<Diagnostic> {
        let Some(dtype) = Self::diagram_type(lines) else {
            return Vec::new();
        };
        // Allow directives like `%%{init: ...}%%`
        if dtype.starts_with("%%") {
            return Vec::new();
        }
        // Strip optional direction suffix: `graph LR` → dtype = "graph"
        let base = dtype.split_whitespace().next().unwrap_or(dtype);
        // Case-sensitive match — Mermaid diagram types are case-sensitive
        if !VALID_DIAGRAM_TYPES.contains(&base) {
            // Find line index
            let line_num = lines
                .iter()
                .position(|l| l.trim().starts_with(base))
                .unwrap_or(0) as u32;
            return vec![Diagnostic::error(
                Range::new(Position::new(line_num, 0), Position::new(line_num, base.len() as u32)),
                "MM001",
                DiagnosticCategory::Syntax,
                format!("Unknown Mermaid diagram type '{}'. Expected one of: graph, flowchart, sequenceDiagram, …", base),
            )];
        }
        Vec::new()
    }

    /// MM004: Empty diagram — only diagram type declaration, no content
    fn check_empty_diagram(&self, lines: &[&str]) -> Vec<Diagnostic> {
        let mut content_lines = 0usize;
        let mut found_type = false;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("%%") {
                continue;
            }
            if !found_type {
                found_type = true;
                // Check it is a valid diagram type declaration
                let base = trimmed.split_whitespace().next().unwrap_or("");
                if VALID_DIAGRAM_TYPES.contains(&base) {
                    continue;
                }
            }
            content_lines += 1;
        }

        if found_type && content_lines == 0 {
            vec![Diagnostic::warning(
                Range::new(Position::new(0, 0), Position::new(0, 1)),
                "MM004",
                DiagnosticCategory::Logic,
                "Diagram has no content — add nodes or connections",
            )]
        } else {
            Vec::new()
        }
    }

    /// Return true if the diagram is a flowchart/graph type.
    fn is_flowchart(dtype: &str) -> bool {
        matches!(dtype, "graph" | "flowchart")
    }

    /// Extract a node ID from a token like `A`, `A[text]`, `A(text)`, `A{text}`, `A((text))`.
    fn extract_node_id(token: &str) -> Option<&str> {
        // Strip bracket/paren/brace content
        if let Some(pos) = token.find(|c| matches!(c, '[' | '(' | '{' | '>')) {
            let id = &token[..pos];
            if !id.is_empty()
                && id
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            {
                return Some(id);
            }
        }
        // Plain identifier (no brackets)
        if !token.is_empty()
            && token
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Some(token);
        }
        None
    }

    /// Parse flowchart lines for node definitions and edge references.
    /// Returns (defined: Map<id, line>, referenced: Vec<(id, line)>)
    fn parse_flowchart_nodes(lines: &[&str]) -> (HashMap<String, u32>, Vec<(String, u32)>) {
        let mut defined: HashMap<String, u32> = HashMap::new();
        let mut referenced: Vec<(String, u32)> = Vec::new();
        let arrow_patterns = [
            "-->", "---", "-.-", "==>", "-.->", "--o", "--x", "<-->", "o--o",
        ];

        for (line_idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("%%") {
                continue;
            }
            let line_num = line_idx as u32;

            // Skip diagram type line and directives
            let first = trimmed.split_whitespace().next().unwrap_or("");
            if VALID_DIAGRAM_TYPES.contains(&first)
                || trimmed.starts_with("subgraph")
                || trimmed == "end"
            {
                continue;
            }

            // Check if this line contains an arrow — edge line
            let has_arrow = arrow_patterns.iter().any(|a| trimmed.contains(a));

            if has_arrow {
                // Split on arrow patterns to get endpoints; handles `A --> B` and `A -->|label| B`
                // Simple approach: split on whitespace and look for identifiers before/after arrows
                let tokens: Vec<&str> = trimmed.split_whitespace().collect();
                let mut i = 0;
                while i < tokens.len() {
                    let tok = tokens[i];
                    // Skip arrow tokens
                    if arrow_patterns.iter().any(|a| {
                        tok.starts_with(a)
                            || tok.ends_with(a)
                            || tok.contains("->")
                            || tok.contains("--")
                    }) {
                        i += 1;
                        continue;
                    }
                    // Skip edge labels like `|text|`
                    if tok.starts_with('|') {
                        i += 1;
                        continue;
                    }
                    // Strip trailing label: `A` from `A -->|label| B`
                    if let Some(id) = Self::extract_node_id(tok) {
                        referenced.push((id.to_string(), line_num));
                        // If has brackets, it's also a definition
                        if tok.contains('[') || tok.contains('(') || tok.contains('{') {
                            defined.entry(id.to_string()).or_insert(line_num);
                        }
                    }
                    i += 1;
                }
            } else {
                // Node definition line: `A[label]` or `A(label)` etc.
                let tokens: Vec<&str> = trimmed.split_whitespace().collect();
                if let Some(&first_tok) = tokens.first() {
                    if let Some(id) = Self::extract_node_id(first_tok) {
                        defined.entry(id.to_string()).or_insert(line_num);
                    }
                }
            }
        }

        (defined, referenced)
    }

    /// MM002: Undefined node reference
    fn check_undefined_nodes(&self, lines: &[&str], dtype: &str) -> Vec<Diagnostic> {
        if !Self::is_flowchart(dtype) {
            return Vec::new();
        }
        let (defined, referenced) = Self::parse_flowchart_nodes(lines);
        let mut diagnostics = Vec::new();

        // Track which undefined nodes we've already reported to avoid duplicates
        let mut reported: HashSet<String> = HashSet::new();

        for (node_id, line_num) in &referenced {
            if !defined.contains_key(node_id) && !reported.contains(node_id) {
                reported.insert(node_id.clone());
                let col = lines
                    .get(*line_num as usize)
                    .and_then(|l| l.find(node_id.as_str()))
                    .unwrap_or(0) as u32;
                diagnostics.push(Diagnostic::warning(
                    Range::new(
                        Position::new(*line_num, col),
                        Position::new(*line_num, col + node_id.len() as u32),
                    ),
                    "MM002",
                    DiagnosticCategory::Names,
                    format!("Node '{}' is referenced but never defined", node_id),
                ));
            }
        }

        diagnostics
    }

    /// MM003: Duplicate node ID
    fn check_duplicate_nodes(&self, lines: &[&str], dtype: &str) -> Vec<Diagnostic> {
        if !Self::is_flowchart(dtype) {
            return Vec::new();
        }

        let mut seen: HashMap<String, u32> = HashMap::new();
        let mut diagnostics = Vec::new();

        for (line_idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("%%") {
                continue;
            }
            let first = trimmed.split_whitespace().next().unwrap_or("");
            if VALID_DIAGRAM_TYPES.contains(&first)
                || trimmed.starts_with("subgraph")
                || trimmed == "end"
            {
                continue;
            }
            // Look for node definitions (lines without arrows)
            let has_arrow = trimmed.contains("-->")
                || trimmed.contains("---")
                || trimmed.contains("-.-")
                || trimmed.contains("==>");
            if !has_arrow {
                let tokens: Vec<&str> = trimmed.split_whitespace().collect();
                if let Some(&first_tok) = tokens.first() {
                    // Must have bracket to count as explicit definition
                    if first_tok.contains('[') || first_tok.contains('(') || first_tok.contains('{')
                    {
                        if let Some(id) = Self::extract_node_id(first_tok) {
                            let line_num = line_idx as u32;
                            if let Some(&prev_line) = seen.get(id) {
                                let col = line.find(id).unwrap_or(0) as u32;
                                diagnostics.push(Diagnostic::warning(
                                    Range::new(
                                        Position::new(line_num, col),
                                        Position::new(line_num, col + id.len() as u32),
                                    ),
                                    "MM003",
                                    DiagnosticCategory::Names,
                                    format!(
                                        "Node '{}' is defined multiple times (first at line {})",
                                        id,
                                        prev_line + 1
                                    ),
                                ));
                            } else {
                                seen.insert(id.to_string(), line_num);
                            }
                        }
                    }
                }
            }
        }

        diagnostics
    }

    /// MM005: Basic syntax errors — mismatched brackets, arrows without endpoints
    fn check_syntax_errors(&self, lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (line_idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("%%") {
                continue;
            }
            let line_num = line_idx as u32;

            // Check mismatched brackets
            let open_square = trimmed.matches('[').count();
            let close_square = trimmed.matches(']').count();
            let open_paren = trimmed.matches('(').count();
            let close_paren = trimmed.matches(')').count();
            let open_curly = trimmed.matches('{').count();
            let close_curly = trimmed.matches('}').count();

            if open_square != close_square {
                diagnostics.push(Diagnostic::new(
                    Range::new(
                        Position::new(line_num, 0),
                        Position::new(line_num, trimmed.len() as u32),
                    ),
                    DiagnosticSeverity::Error,
                    "MM005",
                    DiagnosticCategory::Syntax,
                    "Mismatched square brackets '[' ']'".to_string(),
                ));
            }
            if open_paren != close_paren {
                diagnostics.push(Diagnostic::new(
                    Range::new(
                        Position::new(line_num, 0),
                        Position::new(line_num, trimmed.len() as u32),
                    ),
                    DiagnosticSeverity::Error,
                    "MM005",
                    DiagnosticCategory::Syntax,
                    "Mismatched parentheses '(' ')'".to_string(),
                ));
            }
            if open_curly != close_curly {
                diagnostics.push(Diagnostic::new(
                    Range::new(
                        Position::new(line_num, 0),
                        Position::new(line_num, trimmed.len() as u32),
                    ),
                    DiagnosticSeverity::Error,
                    "MM005",
                    DiagnosticCategory::Syntax,
                    "Mismatched curly braces '{' '}'".to_string(),
                ));
            }

            // Arrow without endpoints: line starts with an arrow token
            let arrow_only = trimmed.starts_with("-->")
                || trimmed.starts_with("---")
                || trimmed.starts_with("==>");
            if arrow_only {
                diagnostics.push(Diagnostic::error(
                    Range::new(
                        Position::new(line_num, 0),
                        Position::new(line_num, trimmed.len() as u32),
                    ),
                    "MM005",
                    DiagnosticCategory::Syntax,
                    "Arrow without a source node endpoint".to_string(),
                ));
            }
        }

        diagnostics
    }
}

impl Default for MermaidChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Checker for MermaidChecker {
    fn language(&self) -> Language {
        Language::Mermaid
    }

    fn check(&self, file: &ParsedFile, _config: &LintConfig) -> Vec<Diagnostic> {
        let lines: Vec<&str> = file.source.lines().collect();
        let mut diagnostics = Vec::new();

        diagnostics.extend(self.check_unknown_diagram(&lines));
        diagnostics.extend(self.check_empty_diagram(&lines));
        diagnostics.extend(self.check_syntax_errors(&lines));

        // Node-level checks only make sense once we know the diagram type
        if let Some(dtype) = Self::diagram_type(&lines) {
            diagnostics.extend(self.check_undefined_nodes(&lines, dtype));
            diagnostics.extend(self.check_duplicate_nodes(&lines, dtype));
        }

        diagnostics
    }

    fn available_rules(&self) -> Vec<&'static str> {
        vec![
            "MM001", // Unknown diagram type
            "MM002", // Undefined node reference
            "MM003", // Duplicate node ID
            "MM004", // Empty diagram
            "MM005", // Syntax error (mismatched brackets, arrows without endpoints)
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::{super::Checker, MermaidChecker};
    use crate::checker::LintConfig;
    use crate::syntax::ParsedFile;

    fn make_file(source: &str) -> ParsedFile {
        ParsedFile::line_based(source.to_string(), crate::syntax::Language::Mermaid)
    }

    fn check(source: &str) -> Vec<String> {
        let checker = MermaidChecker::new();
        let file = make_file(source);
        let config = LintConfig::default();
        checker
            .check(&file, &config)
            .iter()
            .map(|d| d.code.clone())
            .collect()
    }

    #[test]
    fn test_valid_flowchart_no_errors() {
        let src = "flowchart LR\n    A[Start] --> B[End]\n";
        let codes = check(src);
        assert!(
            codes.is_empty(),
            "expected no diagnostics, got: {:?}",
            codes
        );
    }

    #[test]
    fn test_mm001_unknown_diagram_type() {
        let src = "invalidType\n    A --> B\n";
        let codes = check(src);
        assert!(
            codes.contains(&"MM001".to_string()),
            "expected MM001, got: {:?}",
            codes
        );
    }

    #[test]
    fn test_mm004_empty_diagram() {
        let src = "flowchart LR\n";
        let codes = check(src);
        assert!(
            codes.contains(&"MM004".to_string()),
            "expected MM004, got: {:?}",
            codes
        );
    }

    #[test]
    fn test_mm003_duplicate_node() {
        let src = "graph TD\n    A[First]\n    A[Second]\n    A --> B[End]\n";
        let codes = check(src);
        assert!(
            codes.contains(&"MM003".to_string()),
            "expected MM003, got: {:?}",
            codes
        );
    }

    #[test]
    fn test_mm005_mismatched_brackets() {
        let src = "flowchart LR\n    A[Start --> B[End]\n";
        let codes = check(src);
        assert!(
            codes.contains(&"MM005".to_string()),
            "expected MM005, got: {:?}",
            codes
        );
    }

    #[test]
    fn test_sequence_diagram_no_node_checks() {
        // sequenceDiagram doesn't apply flowchart node rules
        let src = "sequenceDiagram\n    Alice->>Bob: Hello\n    Bob-->>Alice: Hi\n";
        let codes = check(src);
        assert!(
            !codes.contains(&"MM002".to_string()),
            "MM002 should not fire for sequenceDiagram"
        );
        assert!(
            !codes.contains(&"MM003".to_string()),
            "MM003 should not fire for sequenceDiagram"
        );
    }

    #[test]
    fn test_available_rules() {
        let checker = MermaidChecker::new();
        let rules = checker.available_rules();
        assert!(rules.contains(&"MM001"));
        assert!(rules.contains(&"MM004"));
        assert!(rules.contains(&"MM005"));
    }
}
