//! OpenRPC 1.x lint checker (line-based JSON analysis)

use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range};
use crate::syntax::{Language, ParsedFile};
use std::collections::HashMap;

/// OpenRPC 1.x checker — line-based JSON analysis
pub struct OpenRpcChecker;

impl OpenRpcChecker {
    pub fn new() -> Self {
        Self
    }

    /// Detect whether the source is an OpenRPC document.
    /// Checks the first 20 lines for the `"openrpc"` key.
    pub fn is_openrpc(source: &str) -> bool {
        source.lines().take(20).any(|line| {
            let trimmed = line.trim();
            trimmed.contains("\"openrpc\"") || trimmed.contains("'openrpc'")
        })
    }

    /// Standalone check — does not require a ParsedFile.
    pub fn check_json(source: &str) -> Vec<Diagnostic> {
        let lines: Vec<&str> = source.lines().collect();
        let mut diagnostics = Vec::new();

        diagnostics.extend(Self::check_required_fields(source));
        diagnostics.extend(Self::check_methods_structure(&lines));
        diagnostics.extend(Self::check_ref_format(&lines));
        diagnostics.extend(Self::check_description_markdown(&lines));

        diagnostics
    }

    /// OR001: Missing required top-level fields: openrpc, info, methods; info.title, info.version
    fn check_required_fields(source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let r = Range::new(Position::new(0, 0), Position::new(0, 1));

        if !source.contains("\"openrpc\"") {
            diags.push(Diagnostic::error(
                r,
                "OR001",
                DiagnosticCategory::Logic,
                "Missing required field 'openrpc' in OpenRPC document",
            ));
        }
        if !source.contains("\"info\"") {
            diags.push(Diagnostic::error(
                r,
                "OR001",
                DiagnosticCategory::Logic,
                "Missing required field 'info' in OpenRPC document",
            ));
        }
        if !source.contains("\"methods\"") {
            diags.push(Diagnostic::error(
                r,
                "OR001",
                DiagnosticCategory::Logic,
                "Missing required field 'methods' in OpenRPC document",
            ));
        }
        // info sub-fields — heuristic: just check source-wide presence
        if source.contains("\"info\"") && !source.contains("\"title\"") {
            diags.push(Diagnostic::error(
                r,
                "OR001",
                DiagnosticCategory::Logic,
                "Missing required field 'info.title' in OpenRPC document",
            ));
        }
        if source.contains("\"info\"") && !source.contains("\"version\"") {
            diags.push(Diagnostic::error(
                r,
                "OR001",
                DiagnosticCategory::Logic,
                "Missing required field 'info.version' in OpenRPC document",
            ));
        }

        diags
    }

    /// OR002-OR004: Validate method objects in the methods array.
    /// OR002: method missing 'params'
    /// OR003: method missing 'result'
    /// OR004: duplicate method names
    fn check_methods_structure(lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut in_methods_array = false;
        let mut brace_depth: i32 = 0;
        // method object tracking: (start_line, has_params, has_result, name)
        let mut method_start: Option<usize> = None;
        let mut method_has_params = false;
        let mut method_has_result = false;
        let mut method_name: Option<String> = None;
        // For OR004: name -> first occurrence line
        let mut seen_names: HashMap<String, usize> = HashMap::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.contains("\"methods\"") && trimmed.contains('[') {
                in_methods_array = true;
            }

            if !in_methods_array {
                continue;
            }

            for ch in trimmed.chars() {
                match ch {
                    '{' => {
                        brace_depth += 1;
                        if brace_depth == 1 {
                            method_start = Some(i);
                            method_has_params = false;
                            method_has_result = false;
                            method_name = None;
                            // Pre-scan the entire line now so single-line objects
                            // like { "name": "x", "params": [], "result": {} }
                            // have their fields recorded before the closing '}' fires.
                            if trimmed.contains("\"params\"") {
                                method_has_params = true;
                            }
                            if trimmed.contains("\"result\"") {
                                method_has_result = true;
                            }
                            if trimmed.contains("\"name\"") {
                                if let Some(name) = extract_json_string_value(trimmed, "name") {
                                    method_name = Some(name);
                                }
                            }
                        }
                    }
                    '}' => {
                        if brace_depth == 1 {
                            if let Some(sl) = method_start.take() {
                                let col = lines.get(sl).map(|l| l.len()).unwrap_or(0);
                                let r = Range::new(
                                    Position::new(sl as u32, 0),
                                    Position::new(sl as u32, col as u32),
                                );
                                if !method_has_params {
                                    diagnostics.push(Diagnostic::new(
                                        r,
                                        DiagnosticSeverity::Warning,
                                        "OR002",
                                        DiagnosticCategory::Logic,
                                        format!(
                                            "Method '{}' is missing 'params' field",
                                            method_name.as_deref().unwrap_or("<unnamed>")
                                        ),
                                    ));
                                }
                                if !method_has_result {
                                    diagnostics.push(Diagnostic::new(
                                        r,
                                        DiagnosticSeverity::Warning,
                                        "OR003",
                                        DiagnosticCategory::Logic,
                                        format!(
                                            "Method '{}' is missing 'result' field",
                                            method_name.as_deref().unwrap_or("<unnamed>")
                                        ),
                                    ));
                                }
                                // OR004: duplicate name check
                                if let Some(ref name) = method_name {
                                    if let Some(first_line) = seen_names.get(name) {
                                        let col2 = lines.get(sl).map(|l| l.len()).unwrap_or(0);
                                        diagnostics.push(Diagnostic::error(
                                            Range::new(
                                                Position::new(sl as u32, 0),
                                                Position::new(sl as u32, col2 as u32),
                                            ),
                                            "OR004",
                                            DiagnosticCategory::Logic,
                                            format!(
                                                "Duplicate method name '{}' — first defined at line {}",
                                                name, first_line + 1
                                            ),
                                        ));
                                    } else {
                                        seen_names.insert(name.clone(), sl);
                                    }
                                }
                            }
                        }
                        brace_depth -= 1;
                        if brace_depth < 0 {
                            brace_depth = 0;
                            in_methods_array = false;
                        }
                    }
                    ']' if brace_depth == 0 => {
                        in_methods_array = false;
                    }
                    _ => {}
                }
            }

            // Extract field presence for multi-line method objects (depth==1, not first line).
            // The first line is already scanned eagerly in the '{' branch above.
            if brace_depth == 1 && method_start.is_some_and(|sl| sl != i) {
                if trimmed.contains("\"params\"") {
                    method_has_params = true;
                }
                if trimmed.contains("\"result\"") {
                    method_has_result = true;
                }
                if trimmed.contains("\"name\"") && method_name.is_none() {
                    if let Some(name) = extract_json_string_value(trimmed, "name") {
                        method_name = Some(name);
                    }
                }
            }
        }

        diagnostics
    }

    /// OR005: $ref format check
    fn check_ref_format(lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("\"$ref\"") {
                let rest = rest.trim().trim_start_matches(':').trim();
                let value = rest
                    .trim_matches('"')
                    .trim_matches('\'')
                    .trim_end_matches(',');
                if !value.is_empty()
                    && !value.starts_with('#')
                    && !value.starts_with('.')
                    && !value.starts_with('/')
                    && !value.contains("://")
                {
                    diagnostics.push(Diagnostic::warning(
                        line_range(i, line),
                        "OR005",
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

    /// OR005 (secondary): Description heading level skip
    fn check_description_markdown(lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("\"description\"") {
                if let Some(value) = extract_json_string_value(trimmed, "description") {
                    if value.contains("# ") && value.contains("### ") && !value.contains("## ") {
                        diagnostics.push(Diagnostic::new(
                            line_range(i, line), DiagnosticSeverity::Hint, "OR005",
                            DiagnosticCategory::Style,
                            "Description skips heading level (h1 to h3) — use sequential heading levels",
                        ));
                    }
                }
            }
        }
        diagnostics
    }
}

/// Extract the string value for a JSON key on the same line, e.g. `"name": "foo"` -> `"foo"`
fn extract_json_string_value(line: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let pos = line.find(&pattern)?;
    let after = &line[pos + pattern.len()..];
    let colon_pos = after.find(':')?;
    let value_part = after[colon_pos + 1..].trim();
    if value_part.starts_with('"') {
        let inner = &value_part[1..];
        let end = inner.find('"')?;
        Some(inner[..end].to_string())
    } else {
        None
    }
}

fn line_range(line_num: usize, line: &str) -> Range {
    Range::new(
        Position::new(line_num as u32, 0),
        Position::new(line_num as u32, line.len() as u32),
    )
}

impl Default for OpenRpcChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Checker for OpenRpcChecker {
    fn language(&self) -> Language {
        Language::Yaml
    }

    fn check(&self, file: &ParsedFile, _config: &LintConfig) -> Vec<Diagnostic> {
        if !Self::is_openrpc(&file.source) {
            return Vec::new();
        }
        Self::check_json(&file.source)
    }

    fn available_rules(&self) -> Vec<&'static str> {
        vec!["OR001", "OR002", "OR003", "OR004", "OR005"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint::Checker;

    fn check(source: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::line_based(source.to_string(), Language::Yaml);
        OpenRpcChecker::new().check(&file, &LintConfig::default())
    }

    #[test]
    fn test_is_openrpc_detects() {
        assert!(OpenRpcChecker::is_openrpc(
            "{\"openrpc\": \"1.2.6\", \"methods\": []}"
        ));
        assert!(!OpenRpcChecker::is_openrpc("{\"openapi\": \"3.0.0\"}"));
        assert!(!OpenRpcChecker::is_openrpc("asyncapi: 2.0.0\n"));
    }

    #[test]
    fn test_missing_required_fields() {
        let source = "{\n  \"openrpc\": \"1.2.6\"\n}\n";
        let diags = check(source);
        let codes: Vec<&str> = diags.iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&"OR001"), "expected OR001, got {:?}", codes);
    }

    #[test]
    fn test_valid_openrpc_no_false_positives() {
        let source = r#"{
  "openrpc": "1.2.6",
  "info": {
    "title": "My RPC API",
    "version": "1.0.0"
  },
  "methods": [
    {
      "name": "getUser",
      "params": [],
      "result": { "name": "user", "schema": {} }
    }
  ]
}
"#;
        let diags = check(source);
        let or001: Vec<_> = diags.iter().filter(|d| d.code == "OR001").collect();
        let or002: Vec<_> = diags.iter().filter(|d| d.code == "OR002").collect();
        let or003: Vec<_> = diags.iter().filter(|d| d.code == "OR003").collect();
        let or004: Vec<_> = diags.iter().filter(|d| d.code == "OR004").collect();
        assert!(or001.is_empty(), "unexpected OR001: {:?}", or001);
        assert!(or002.is_empty(), "unexpected OR002: {:?}", or002);
        assert!(or003.is_empty(), "unexpected OR003: {:?}", or003);
        assert!(or004.is_empty(), "unexpected OR004: {:?}", or004);
    }

    #[test]
    fn test_method_missing_params_and_result() {
        let source = r#"{
  "openrpc": "1.2.6",
  "info": { "title": "T", "version": "1.0.0" },
  "methods": [
    {
      "name": "doSomething"
    }
  ]
}
"#;
        let diags = check(source);
        assert!(
            diags.iter().any(|d| d.code == "OR002"),
            "expected OR002, got {:?}",
            diags
        );
        assert!(
            diags.iter().any(|d| d.code == "OR003"),
            "expected OR003, got {:?}",
            diags
        );
    }

    #[test]
    fn test_duplicate_method_names() {
        let source = r#"{
  "openrpc": "1.2.6",
  "info": { "title": "T", "version": "1.0.0" },
  "methods": [
    { "name": "getUser", "params": [], "result": {} },
    { "name": "getUser", "params": [], "result": {} }
  ]
}
"#;
        let diags = check(source);
        assert!(
            diags.iter().any(|d| d.code == "OR004"),
            "expected OR004, got {:?}",
            diags
        );
    }

    #[test]
    fn test_not_openrpc_returns_empty() {
        let source = "{\n  \"openapi\": \"3.0.0\"\n}\n";
        let diags = check(source);
        assert!(diags.is_empty(), "expected no diags for non-openrpc JSON");
    }
}
