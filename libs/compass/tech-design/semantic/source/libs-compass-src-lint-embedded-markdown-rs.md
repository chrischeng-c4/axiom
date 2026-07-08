---
id: libs-compass-src-lint-embedded-markdown-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/lint/embedded_markdown.rs`.
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

# Standardized libs/compass/src/lint/embedded_markdown.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/lint/embedded_markdown.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DescriptionField` | libs/compass/src/lint/embedded_markdown.rs | struct | pub | 9 | pub struct DescriptionField { |
| `extract_description_fields` | libs/compass/src/lint/embedded_markdown.rs | function | pub | 23 | pub fn extract_description_fields(source: &str) -> Vec<DescriptionField> { |
| `lint_embedded_markdown` | libs/compass/src/lint/embedded_markdown.rs | function | pub | 125 | pub fn lint_embedded_markdown(content: &str, line_offset: usize) -> Vec<Diagnostic> { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Helper for extracting and linting Markdown content embedded in API specs
//! (OpenAPI/AsyncAPI YAML or JSON description fields).

use super::markdown::line_range;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity};

/// A description field extracted from a YAML/JSON spec.
#[derive(Debug, Clone)]
pub struct DescriptionField {
    /// The raw markdown content.
    pub content: String,
    /// The 0-indexed line number in the original source where this content begins.
    pub line_offset: usize,
}

/// Extract markdown content from YAML/JSON description/summary fields.
///
/// Handles two YAML forms:
/// - Inline:  `description: "some text"`
/// - Block:   `description: |`  followed by indented lines
///
/// Returns one `DescriptionField` per occurrence found.
pub fn extract_description_fields(source: &str) -> Vec<DescriptionField> {
    let mut results = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        // Look for "description:" or "summary:" keys
        let field_key = if trimmed.starts_with("description:") {
            Some("description:")
        } else if trimmed.starts_with("summary:") {
            Some("summary:")
        } else {
            None
        };

        if let Some(key) = field_key {
            let value_part = trimmed[key.len()..].trim();

            if value_part == "|" || value_part == "|-" || value_part == "|+" {
                // Block scalar — collect indented lines that follow
                let base_indent = leading_spaces(lines[i]);
                let content_start = i + 1;
                let mut j = content_start;

                while j < lines.len() {
                    let next = lines[j];
                    let next_indent = leading_spaces(next);
                    // Stop when we hit a line at or before the key's indent level
                    // (unless the line is blank)
                    if !next.trim().is_empty() && next_indent <= base_indent {
                        break;
                    }
                    j += 1;
                }

                if j > content_start {
                    // Strip the common leading indent from each content line
                    let min_indent = lines[content_start..j]
                        .iter()
                        .filter(|l| !l.trim().is_empty())
                        .map(|l| leading_spaces(l))
                        .min()
                        .unwrap_or(0);

                    let content: String = lines[content_start..j]
                        .iter()
                        .map(|l| {
                            if l.len() > min_indent {
                                &l[min_indent..]
                            } else {
                                l.trim()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n");

                    results.push(DescriptionField {
                        content,
                        line_offset: content_start,
                    });
                }

                i = j;
                continue;
            } else if value_part.starts_with('"') || value_part.starts_with('\'') {
                // Quoted inline string — strip surrounding quotes
                let inner = value_part
                    .trim_start_matches('"')
                    .trim_end_matches('"')
                    .trim_start_matches('\'')
                    .trim_end_matches('\'');
                if !inner.is_empty() {
                    results.push(DescriptionField {
                        content: inner.to_string(),
                        line_offset: i,
                    });
                }
            } else if !value_part.is_empty() {
                // Bare inline value
                results.push(DescriptionField {
                    content: value_part.to_string(),
                    line_offset: i,
                });
            }
        }

        i += 1;
    }

    results
}

/// Lint embedded markdown content using a subset of Markdown rules.
///
/// Only applies:
/// - Heading level skip (MD001)
/// - Line length > 120 chars (MD004)
///
/// Line numbers in the returned diagnostics are adjusted by `line_offset`
/// so they map back to positions in the original source file.
pub fn lint_embedded_markdown(content: &str, line_offset: usize) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut last_heading_level: Option<usize> = None;

    for (line_idx, line) in content.lines().enumerate() {
        let abs_line = (line_offset + line_idx) as u32;
        let trimmed = line.trim();

        // MD001: heading level skip
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|c| *c == '#').count();
            if level <= 6 {
                if let Some(prev) = last_heading_level {
                    if level > prev + 1 {
                        diagnostics.push(Diagnostic::new(
                            line_range(abs_line),
                            DiagnosticSeverity::Warning,
                            "MD001",
                            DiagnosticCategory::Style,
                            format!(
                                "Embedded heading level skipped: h{} after h{} — avoid skipping heading levels",
                                level, prev
                            ),
                        ));
                    }
                }
                last_heading_level = Some(level);
                continue;
            }
        }

        // MD004: line length > 120
        if line.len() > 120 {
            let line_trimmed = line.trim();
            if !line_trimmed.starts_with("http://") && !line_trimmed.starts_with("https://") {
                diagnostics.push(Diagnostic::new(
                    line_range(abs_line),
                    DiagnosticSeverity::Warning,
                    "MD004",
                    DiagnosticCategory::Style,
                    format!(
                        "Embedded description line length {} exceeds 120 characters",
                        line.len()
                    ),
                ));
            }
        }
    }

    diagnostics
}

/// Returns the number of leading space characters (not tabs, for simplicity).
fn leading_spaces(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ').count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_inline_description() {
        let yaml = "paths:\n  /foo:\n    description: \"Some **markdown** text\"\n";
        let fields = extract_description_fields(yaml);
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].content, "Some **markdown** text");
        assert_eq!(fields[0].line_offset, 2);
    }

    #[test]
    fn test_extract_block_description() {
        let yaml = "paths:\n  /foo:\n    description: |\n      # Heading\n      Some body text.\n  /bar:\n    get: {}\n";
        let fields = extract_description_fields(yaml);
        assert_eq!(fields.len(), 1, "should extract one block description");
        assert!(
            fields[0].content.contains("# Heading"),
            "content should contain the heading"
        );
        assert_eq!(fields[0].line_offset, 3, "content starts at line 3");
    }

    #[test]
    fn test_extract_summary_field() {
        let yaml = "info:\n  summary: \"Short summary text\"\n";
        let fields = extract_description_fields(yaml);
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].content, "Short summary text");
    }

    #[test]
    fn test_lint_heading_skip() {
        let content = "# Title\n### Skip level\n";
        let diags = lint_embedded_markdown(content, 10);
        let codes: Vec<_> = diags.iter().map(|d| d.code.as_str()).collect();
        assert!(
            codes.contains(&"MD001"),
            "expected MD001 for heading skip, got {:?}",
            codes
        );
        // Line numbers should be offset
        let d = diags.iter().find(|d| d.code == "MD001").unwrap();
        assert_eq!(
            d.range.start.line, 11,
            "line should be offset by 10 + 1 = 11"
        );
    }

    #[test]
    fn test_lint_line_length() {
        let long_line = "x".repeat(130);
        let content = format!("# Title\n{}\n", long_line);
        let diags = lint_embedded_markdown(&content, 5);
        let codes: Vec<_> = diags.iter().map(|d| d.code.as_str()).collect();
        assert!(
            codes.contains(&"MD004"),
            "expected MD004 for long line, got {:?}",
            codes
        );
        let d = diags.iter().find(|d| d.code == "MD004").unwrap();
        assert_eq!(d.range.start.line, 6, "line should be offset by 5 + 1 = 6");
    }

    #[test]
    fn test_lint_clean_content_no_diagnostics() {
        let content = "# Title\n\n## Section\n\nNormal paragraph.\n";
        let diags = lint_embedded_markdown(content, 0);
        assert!(
            diags.is_empty(),
            "expected no diagnostics for clean content, got {:?}",
            diags
        );
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/lint/embedded_markdown.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/lint/embedded_markdown.rs` captured during libs codegen standardization.
```
