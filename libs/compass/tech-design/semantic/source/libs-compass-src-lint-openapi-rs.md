---
id: libs-compass-src-lint-openapi-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/lint/openapi.rs`.
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

# Standardized libs/compass/src/lint/openapi.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/lint/openapi.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `OpenApiChecker` | libs/compass/src/lint/openapi.rs | struct | pub | 8 | pub struct OpenApiChecker; |
| `new` | libs/compass/src/lint/openapi.rs | function | pub | 11 | pub fn new() -> Self { |
| `is_openapi` | libs/compass/src/lint/openapi.rs | function | pub | 16 | pub fn is_openapi(source: &str) -> bool { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! OpenAPI 3.x lint checker (source-line analysis on YAML)

use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range};
use crate::syntax::{Language, ParsedFile};

/// OpenAPI 3.x checker — line-based YAML analysis
pub struct OpenApiChecker;

impl OpenApiChecker {
    pub fn new() -> Self {
        Self
    }

    /// Detect whether the source is an OpenAPI 3.x document.
    pub fn is_openapi(source: &str) -> bool {
        source.lines().take(10).any(|line| {
            let t = line.trim();
            t.starts_with("openapi:") && t.contains("3.")
        })
    }

    /// OA001: Missing required top-level fields and info sub-fields
    fn check_required_fields(lines: &[&str]) -> Vec<Diagnostic> {
        let mut has_info = false;
        let mut has_paths = false;
        let mut has_title = false;
        let mut has_version = false;
        let mut in_info = false;

        for line in lines.iter() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let indent = line.len() - line.trim_start().len();
            if indent == 0 {
                if in_info {
                    in_info = false;
                }
                if trimmed.starts_with("info:") {
                    has_info = true;
                    in_info = true;
                } else if trimmed.starts_with("paths:") {
                    has_paths = true;
                }
            } else if in_info {
                if trimmed.starts_with("title:") {
                    has_title = true;
                } else if trimmed.starts_with("version:") {
                    has_version = true;
                }
            }
        }

        let mut diags = Vec::new();
        let r = Range::new(Position::new(0, 0), Position::new(0, 1));
        if !has_info {
            diags.push(Diagnostic::error(
                r,
                "OA001",
                DiagnosticCategory::Logic,
                "Missing required field 'info' in OpenAPI document",
            ));
        }
        if !has_paths {
            diags.push(Diagnostic::error(
                r,
                "OA001",
                DiagnosticCategory::Logic,
                "Missing required field 'paths' in OpenAPI document",
            ));
        }
        if has_info && !has_title {
            diags.push(Diagnostic::error(
                r,
                "OA001",
                DiagnosticCategory::Logic,
                "Missing required field 'info.title' in OpenAPI document",
            ));
        }
        if has_info && !has_version {
            diags.push(Diagnostic::error(
                r,
                "OA001",
                DiagnosticCategory::Logic,
                "Missing required field 'info.version' in OpenAPI document",
            ));
        }
        diags
    }

    /// OA002: Invalid $ref format — must start with `#/`, be a relative path, or a URL
    fn check_ref_format(lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("$ref:") {
                let value = rest.trim().trim_matches('"').trim_matches('\'');
                if value.is_empty() {
                    diagnostics.push(Diagnostic::error(
                        line_range(i, line),
                        "OA002",
                        DiagnosticCategory::Logic,
                        "Empty $ref value — must be '#/...' or a relative path",
                    ));
                } else if !value.starts_with('#')
                    && !value.starts_with('.')
                    && !value.starts_with('/')
                    && !value.contains("://")
                    && !value.ends_with(".yaml")
                    && !value.ends_with(".yml")
                    && !value.ends_with(".json")
                {
                    diagnostics.push(Diagnostic::warning(
                        line_range(i, line),
                        "OA002",
                        DiagnosticCategory::Logic,
                        format!(
                            "Suspicious $ref '{}' — use '#/components/...' for internal refs",
                            value
                        ),
                    ));
                }
            }
        }
        diagnostics
    }

    /// OA003: Path item with no HTTP methods
    fn check_empty_path_items(lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let http_methods = [
            "get:", "post:", "put:", "delete:", "patch:", "options:", "head:",
        ];
        let mut in_paths = false;
        let mut current_path: Option<(usize, String)> = None;
        let mut path_has_method = false;

        let flush = |diags: &mut Vec<Diagnostic>,
                     path: Option<(usize, String)>,
                     has_method: bool,
                     lines: &[&str]| {
            if let Some((pl, ps)) = path {
                if !has_method {
                    diags.push(Diagnostic::warning(
                        line_range(pl, lines[pl]),
                        "OA003",
                        DiagnosticCategory::Logic,
                        format!("Path '{}' has no HTTP methods defined", ps),
                    ));
                }
            }
        };

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let indent = line.len() - line.trim_start().len();

            if indent == 0 {
                if trimmed.starts_with("paths:") {
                    flush(
                        &mut diagnostics,
                        current_path.take(),
                        path_has_method,
                        lines,
                    );
                    in_paths = true;
                } else if in_paths {
                    flush(
                        &mut diagnostics,
                        current_path.take(),
                        path_has_method,
                        lines,
                    );
                    in_paths = false;
                }
                continue;
            }

            if !in_paths {
                continue;
            }

            if indent == 2 {
                flush(
                    &mut diagnostics,
                    current_path.take(),
                    path_has_method,
                    lines,
                );
                let key = trimmed.trim_end_matches(':');
                if key.starts_with('/') {
                    current_path = Some((i, key.to_string()));
                    path_has_method = false;
                } else {
                    current_path = None;
                }
            } else if indent >= 4 && current_path.is_some() {
                if http_methods.iter().any(|&m| trimmed.starts_with(m)) {
                    path_has_method = true;
                }
            }
        }

        flush(
            &mut diagnostics,
            current_path.take(),
            path_has_method,
            lines,
        );
        diagnostics
    }

    /// OA004: HTTP method block missing operationId
    fn check_missing_operation_id(lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let http_methods = [
            "get:", "post:", "put:", "delete:", "patch:", "options:", "head:",
        ];
        let mut in_paths = false;
        let mut current_method: Option<(usize, String)> = None;
        let mut has_op_id = false;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let indent = line.len() - line.trim_start().len();

            if indent == 0 {
                if in_paths {
                    if let Some((ml, mn)) = current_method.take() {
                        if !has_op_id {
                            diagnostics.push(op_id_diag(ml, mn, lines));
                        }
                    }
                }
                in_paths = trimmed.starts_with("paths:");
                continue;
            }

            if !in_paths {
                continue;
            }

            if indent == 4 && http_methods.iter().any(|&m| trimmed.starts_with(m)) {
                if let Some((ml, mn)) = current_method.take() {
                    if !has_op_id {
                        diagnostics.push(op_id_diag(ml, mn, lines));
                    }
                }
                current_method = Some((i, trimmed.trim_end_matches(':').to_string()));
                has_op_id = false;
            } else if indent == 2 {
                // New path — flush method
                if let Some((ml, mn)) = current_method.take() {
                    if !has_op_id {
                        diagnostics.push(op_id_diag(ml, mn, lines));
                    }
                }
            } else if current_method.is_some() && trimmed.starts_with("operationId:") {
                has_op_id = true;
            }
        }

        if let Some((ml, mn)) = current_method.take() {
            if !has_op_id {
                diagnostics.push(op_id_diag(ml, mn, lines));
            }
        }

        diagnostics
    }

    /// OA005: Description heading level skip
    fn check_description_markdown(lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("description:") {
                let value = rest.trim().trim_matches('"').trim_matches('\'');
                if value.contains("# ") && value.contains("### ") && !value.contains("## ") {
                    diagnostics.push(Diagnostic::new(
                        line_range(i, line), DiagnosticSeverity::Hint, "OA005",
                        DiagnosticCategory::Style,
                        "Description skips heading level (h1 to h3) — use sequential heading levels",
                    ));
                }
            }
        }
        diagnostics
    }
}

fn line_range(line_num: usize, line: &str) -> Range {
    Range::new(
        Position::new(line_num as u32, 0),
        Position::new(line_num as u32, line.len() as u32),
    )
}

fn op_id_diag(line: usize, method: String, lines: &[&str]) -> Diagnostic {
    let col = lines.get(line).map(|l| l.len()).unwrap_or(0);
    Diagnostic::new(
        Range::new(
            Position::new(line as u32, 0),
            Position::new(line as u32, col as u32),
        ),
        DiagnosticSeverity::Warning,
        "OA004",
        DiagnosticCategory::Style,
        format!("HTTP method '{}' is missing 'operationId'", method),
    )
}

impl Default for OpenApiChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Checker for OpenApiChecker {
    fn language(&self) -> Language {
        Language::Yaml
    }

    fn check(&self, file: &ParsedFile, _config: &LintConfig) -> Vec<Diagnostic> {
        if !Self::is_openapi(&file.source) {
            return Vec::new();
        }

        let lines: Vec<&str> = file.source.lines().collect();
        let mut diagnostics = Vec::new();

        diagnostics.extend(Self::check_required_fields(&lines));
        diagnostics.extend(Self::check_ref_format(&lines));
        diagnostics.extend(Self::check_empty_path_items(&lines));
        diagnostics.extend(Self::check_missing_operation_id(&lines));
        diagnostics.extend(Self::check_description_markdown(&lines));

        diagnostics
    }

    fn available_rules(&self) -> Vec<&'static str> {
        vec!["OA001", "OA002", "OA003", "OA004", "OA005"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint::Checker;

    fn check(source: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::line_based(source.to_string(), Language::Yaml);
        OpenApiChecker::new().check(&file, &LintConfig::default())
    }

    #[test]
    fn test_is_openapi_detects_version() {
        assert!(OpenApiChecker::is_openapi(
            "openapi: 3.0.0\ninfo:\n  title: Test\n"
        ));
        assert!(OpenApiChecker::is_openapi("openapi: \"3.1.0\"\n"));
        assert!(!OpenApiChecker::is_openapi("asyncapi: 2.0.0\n"));
        assert!(!OpenApiChecker::is_openapi("apiVersion: v1\nkind: Pod\n"));
    }

    #[test]
    fn test_missing_required_fields() {
        let source = "openapi: 3.0.0\ninfo:\n  title: My API\n";
        let diags = check(source);
        let codes: Vec<&str> = diags.iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&"OA001"), "expected OA001, got {:?}", codes);
    }

    #[test]
    fn test_valid_openapi_no_false_positives() {
        let source = "\
openapi: 3.0.0
info:
  title: My API
  version: 1.0.0
paths:
  /users:
    get:
      operationId: listUsers
      responses:
        '200':
          description: OK
";
        let diags = check(source);
        let oa001: Vec<_> = diags.iter().filter(|d| d.code == "OA001").collect();
        let oa003: Vec<_> = diags.iter().filter(|d| d.code == "OA003").collect();
        let oa004: Vec<_> = diags.iter().filter(|d| d.code == "OA004").collect();
        assert!(oa001.is_empty(), "unexpected OA001: {:?}", oa001);
        assert!(oa003.is_empty(), "unexpected OA003: {:?}", oa003);
        assert!(oa004.is_empty(), "unexpected OA004: {:?}", oa004);
    }

    #[test]
    fn test_invalid_ref_format() {
        let source = "\
openapi: 3.0.0
info:
  title: T
  version: 1.0.0
paths:
  /x:
    get:
      operationId: getX
      requestBody:
        $ref: bad_ref_no_hash
";
        let diags = check(source);
        assert!(
            diags.iter().any(|d| d.code == "OA002"),
            "expected OA002, got {:?}",
            diags
        );
    }

    #[test]
    fn test_empty_path_item() {
        let source = "\
openapi: 3.0.0
info:
  title: T
  version: 1.0.0
paths:
  /empty:
    x-custom: value
";
        let diags = check(source);
        assert!(
            diags.iter().any(|d| d.code == "OA003"),
            "expected OA003, got {:?}",
            diags
        );
    }

    #[test]
    fn test_missing_operation_id() {
        let source = "\
openapi: 3.0.0
info:
  title: T
  version: 1.0.0
paths:
  /items:
    get:
      responses:
        '200':
          description: OK
";
        let diags = check(source);
        assert!(
            diags.iter().any(|d| d.code == "OA004"),
            "expected OA004, got {:?}",
            diags
        );
    }

    #[test]
    fn test_not_openapi_returns_empty() {
        let source = "kind: Pod\napiVersion: v1\n";
        let diags = check(source);
        assert!(diags.is_empty(), "expected no diags for non-openapi YAML");
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/lint/openapi.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/lint/openapi.rs` captured during libs codegen standardization.
```
