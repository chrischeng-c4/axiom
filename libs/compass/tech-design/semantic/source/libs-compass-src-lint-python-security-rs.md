---
id: libs-compass-src-lint-python-security-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/lint/python_security.rs`.
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

# Standardized libs/compass/src/lint/python_security.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/lint/python_security.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `check_eval_usage` | libs/compass/src/lint/python_security.rs | function | pub | 7 | pub(super) fn check_eval_usage(file: &ParsedFile) -> Vec<Diagnostic> { |
| `check_exec_usage` | libs/compass/src/lint/python_security.rs | function | pub | 17 | pub(super) fn check_exec_usage(file: &ParsedFile) -> Vec<Diagnostic> { |
| `check_pickle_usage` | libs/compass/src/lint/python_security.rs | function | pub | 27 | pub(super) fn check_pickle_usage(file: &ParsedFile) -> Vec<Diagnostic> { |
| `check_subprocess_shell` | libs/compass/src/lint/python_security.rs | function | pub | 58 | pub(super) fn check_subprocess_shell(file: &ParsedFile) -> Vec<Diagnostic> { |
| `check_hardcoded_secrets` | libs/compass/src/lint/python_security.rs | function | pub | 109 | pub(super) fn check_hardcoded_secrets(lines: &[&str]) -> Vec<Diagnostic> { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Python security lint rules (PY301-PY305)

use crate::diagnostic::{Diagnostic, DiagnosticCategory, Position, Range};
use crate::syntax::ParsedFile;

/// PY301: `eval()` usage
pub(super) fn check_eval_usage(file: &ParsedFile) -> Vec<Diagnostic> {
    check_function_call(
        file,
        "eval",
        "PY301",
        "Use of eval() is a security risk — consider ast.literal_eval() or safer alternatives",
    )
}

/// PY302: `exec()` usage
pub(super) fn check_exec_usage(file: &ParsedFile) -> Vec<Diagnostic> {
    check_function_call(
        file,
        "exec",
        "PY302",
        "Use of exec() is a security risk — avoid executing dynamic code",
    )
}

/// PY303: `pickle.loads()` / `pickle.load()` usage
pub(super) fn check_pickle_usage(file: &ParsedFile) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    file.walk(|node, _depth| {
        if node.kind() == "call" {
            if let Some(func) = node.child_by_field_name("function") {
                let func_text = file.node_text(&func);
                if func_text == "pickle.loads"
                    || func_text == "pickle.load"
                    || func_text == "cPickle.loads"
                    || func_text == "cPickle.load"
                {
                    diagnostics.push(Diagnostic::warning(
                        Range::from_node(node),
                        "PY303",
                        DiagnosticCategory::Security,
                        format!(
                            "{}() can execute arbitrary code during deserialization — use a safer format",
                            func_text,
                        ),
                    ));
                }
            }
        }
        true
    });

    diagnostics
}

/// PY304: `subprocess` with `shell=True`
pub(super) fn check_subprocess_shell(file: &ParsedFile) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    file.walk(|node, _depth| {
        if node.kind() == "call" {
            if let Some(func) = node.child_by_field_name("function") {
                let func_text = file.node_text(&func);
                let is_subprocess = func_text.starts_with("subprocess.")
                    || func_text == "Popen"
                    || func_text == "call"
                    || func_text == "check_output"
                    || func_text == "check_call";

                if !is_subprocess {
                    return true;
                }

                // Check for shell=True in keyword arguments
                if let Some(args) = node.child_by_field_name("arguments") {
                    let mut cursor = args.walk();
                    for child in args.children(&mut cursor) {
                        if child.kind() == "keyword_argument" {
                            if let Some(name) = child.child_by_field_name("name") {
                                if file.node_text(&name) == "shell" {
                                    if let Some(value) = child.child_by_field_name("value") {
                                        if file.node_text(&value) == "True" {
                                            diagnostics.push(Diagnostic::warning(
                                                Range::from_node(node),
                                                "PY304",
                                                DiagnosticCategory::Security,
                                                format!(
                                                    "{}() with shell=True is a security risk — use a list of arguments instead",
                                                    func_text,
                                                ),
                                            ));
                                        }
                                    }
                                }
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

/// PY305: Hardcoded secrets (strings matching patterns like `password=`, `secret=`, `api_key=`)
pub(super) fn check_hardcoded_secrets(lines: &[&str]) -> Vec<Diagnostic> {
    const SECRET_PATTERNS: &[&str] = &[
        "password=",
        "password =",
        "secret=",
        "secret =",
        "api_key=",
        "api_key =",
        "apikey=",
        "apikey =",
        "token=",
        "token =",
        "private_key=",
        "private_key =",
        "access_key=",
        "access_key =",
        "secret_key=",
        "secret_key =",
    ];

    let mut diagnostics = Vec::new();

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let lower = trimmed.to_lowercase();
        for pattern in SECRET_PATTERNS {
            if lower.contains(pattern) {
                // Check that this looks like an assignment with a literal value
                // (not just referencing os.environ or a variable)
                let after_pat = &lower[lower.find(pattern).unwrap() + pattern.len()..];
                let after_trimmed = after_pat.trim();
                // Skip if the value references an env var or is empty
                if after_trimmed.starts_with("os.environ")
                    || after_trimmed.starts_with("os.getenv")
                    || after_trimmed.starts_with("environ")
                    || after_trimmed.starts_with("none")
                    || after_trimmed.starts_with("\"\"")
                    || after_trimmed.starts_with("''")
                    || after_trimmed.is_empty()
                {
                    continue;
                }
                // Only flag if it looks like there's a string literal value
                if after_trimmed.starts_with('"')
                    || after_trimmed.starts_with('\'')
                    || after_trimmed.starts_with("b\"")
                    || after_trimmed.starts_with("b'")
                {
                    diagnostics.push(Diagnostic::warning(
                        Range::new(
                            Position::new(line_num as u32, 0),
                            Position::new(line_num as u32, line.len() as u32),
                        ),
                        "PY305",
                        DiagnosticCategory::Security,
                        format!(
                            "Possible hardcoded secret matching '{}' — use environment variables or a secrets manager",
                            pattern.trim(),
                        ),
                    ));
                    break; // One diagnostic per line
                }
            }
        }
    }

    diagnostics
}

/// Helper: check for a simple function call by name
fn check_function_call(
    file: &ParsedFile,
    func_name: &str,
    code: &str,
    message: &str,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    file.walk(|node, _depth| {
        if node.kind() == "call" {
            if let Some(func) = node.child_by_field_name("function") {
                let func_text = file.node_text(&func);
                if func_text == func_name {
                    diagnostics.push(Diagnostic::warning(
                        Range::from_node(node),
                        code,
                        DiagnosticCategory::Security,
                        message,
                    ));
                }
            }
        }
        true
    });

    diagnostics
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/lint/python_security.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/lint/python_security.rs` captured during libs codegen standardization.
```
