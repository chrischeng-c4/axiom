---
id: libs-compass-src-format-mod-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/format/mod.rs`.
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

# Standardized libs/compass/src/format/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/format/mod.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `detect` | libs/compass/src/format/mod.rs | module | pub | 5 | pub mod detect; |
| `FormatResult` | libs/compass/src/format/mod.rs | struct | pub | 13 | pub struct FormatResult { |
| `FormatterConfig` | libs/compass/src/format/mod.rs | struct | pub | 26 | pub struct FormatterConfig { |
| `FormatterRegistry` | libs/compass/src/format/mod.rs | struct | pub | 38 | pub struct FormatterRegistry { |
| `new` | libs/compass/src/format/mod.rs | function | pub | 48 | pub fn new() -> Self { |
| `format` | libs/compass/src/format/mod.rs | function | pub | 157 | pub fn format(&self, source: &str, language: &str) -> Option<FormatResult> { |
| `format_check` | libs/compass/src/format/mod.rs | function | pub | 201 | pub fn format_check(&self, source: &str, language: &str) -> Option<bool> { |
| `is_available` | libs/compass/src/format/mod.rs | function | pub | 207 | pub fn is_available(&self, language: &str) -> bool { |
| `status` | libs/compass/src/format/mod.rs | function | pub | 216 | pub fn status(&self) -> Vec<(String, String, bool)> { |
| `language_for_extension` | libs/compass/src/format/mod.rs | function | pub | 227 | pub fn language_for_extension(ext: &str) -> Option<&'static str> { |
| `format_file` | libs/compass/src/format/mod.rs | function | pub | 243 | pub fn format_file(&self, path: &Path) -> Option<FormatResult> { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Formatter integration — unified interface for external formatters
//!
//! Wraps rustfmt, prettier, gofmt, black, terraform fmt, etc.

pub mod detect;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Result of formatting a file
#[derive(Debug, Clone)]
pub struct FormatResult {
    /// Original source (before formatting)
    pub original: String,
    /// Formatted source (after formatting)
    pub formatted: String,
    /// Whether the file was changed
    pub changed: bool,
    /// Formatter used
    pub formatter: String,
}

/// Configuration for a single formatter
#[derive(Debug, Clone)]
pub struct FormatterConfig {
    /// Name of the formatter binary
    pub binary_name: String,
    /// Arguments for formatting (reading from stdin, writing to stdout)
    pub format_args: Vec<String>,
    /// Arguments for check mode (exit code indicates if formatting needed)
    pub check_args: Vec<String>,
    /// Whether this formatter reads from stdin
    pub stdin_mode: bool,
}

/// Registry of formatters per language
pub struct FormatterRegistry {
    /// Language -> formatter config
    formatters: HashMap<String, FormatterConfig>,
    /// Cached binary availability
    available: HashMap<String, PathBuf>,
}

impl FormatterRegistry {
    /// Create a new registry with default formatter configs.
    /// Probes for binary availability using `detect::find_binary`.
    pub fn new() -> Self {
        let mut registry = Self {
            formatters: HashMap::new(),
            available: HashMap::new(),
        };

        // Register default formatters
        registry.register(
            "rust",
            FormatterConfig {
                binary_name: "rustfmt".into(),
                format_args: vec!["--edition".into(), "2021".into()],
                check_args: vec!["--check".into(), "--edition".into(), "2021".into()],
                stdin_mode: true,
            },
        );
        registry.register(
            "python",
            FormatterConfig {
                binary_name: "black".into(),
                format_args: vec!["-".into(), "-q".into()],
                check_args: vec!["--check".into(), "-q".into()],
                stdin_mode: true,
            },
        );
        registry.register(
            "go",
            FormatterConfig {
                binary_name: "gofmt".into(),
                format_args: vec![],
                check_args: vec!["-l".into()],
                stdin_mode: true,
            },
        );
        registry.register(
            "javascript",
            FormatterConfig {
                binary_name: "prettier".into(),
                format_args: vec!["--parser".into(), "babel".into()],
                check_args: vec!["--check".into(), "--parser".into(), "babel".into()],
                stdin_mode: true,
            },
        );
        registry.register(
            "typescript",
            FormatterConfig {
                binary_name: "prettier".into(),
                format_args: vec!["--parser".into(), "typescript".into()],
                check_args: vec!["--check".into(), "--parser".into(), "typescript".into()],
                stdin_mode: true,
            },
        );
        registry.register(
            "css",
            FormatterConfig {
                binary_name: "prettier".into(),
                format_args: vec!["--parser".into(), "css".into()],
                check_args: vec!["--check".into(), "--parser".into(), "css".into()],
                stdin_mode: true,
            },
        );
        registry.register(
            "html",
            FormatterConfig {
                binary_name: "prettier".into(),
                format_args: vec!["--parser".into(), "html".into()],
                check_args: vec!["--check".into(), "--parser".into(), "html".into()],
                stdin_mode: true,
            },
        );
        registry.register(
            "hcl",
            FormatterConfig {
                binary_name: "terraform".into(),
                format_args: vec!["fmt".into(), "-".into()],
                check_args: vec!["fmt".into(), "-check".into()],
                stdin_mode: true,
            },
        );
        registry.register(
            "sql",
            FormatterConfig {
                binary_name: "pg_format".into(),
                format_args: vec![],
                check_args: vec![], // pg_format doesn't have check mode
                stdin_mode: true,
            },
        );

        // Detect available binaries
        registry.detect_all();

        registry
    }

    fn register(&mut self, language: &str, config: FormatterConfig) {
        self.formatters.insert(language.to_string(), config);
    }

    fn detect_all(&mut self) {
        for (_, config) in &self.formatters {
            if let Some(path) = detect::find_binary(&config.binary_name) {
                self.available.insert(config.binary_name.clone(), path);
            }
        }
    }

    /// Format source code for a given language.
    /// Returns None if no formatter is available for the language.
    pub fn format(&self, source: &str, language: &str) -> Option<FormatResult> {
        let config = self.formatters.get(language)?;
        let binary_path = self.available.get(&config.binary_name)?;

        let mut cmd = Command::new(binary_path);
        for arg in &config.format_args {
            cmd.arg(arg);
        }

        if config.stdin_mode {
            use std::io::Write;
            cmd.stdin(std::process::Stdio::piped());
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::piped());

            let mut child = cmd.spawn().ok()?;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(source.as_bytes()).ok()?;
            }
            let output = child.wait_with_output().ok()?;
            if output.status.success() {
                let formatted = String::from_utf8_lossy(&output.stdout).to_string();
                let changed = formatted != source;
                Some(FormatResult {
                    original: source.to_string(),
                    formatted,
                    changed,
                    formatter: config.binary_name.clone(),
                })
            } else {
                // Formatter failed, return unchanged
                tracing::warn!(
                    "Formatter {} failed: {}",
                    config.binary_name,
                    String::from_utf8_lossy(&output.stderr)
                );
                None
            }
        } else {
            None
        }
    }

    /// Check if formatting would change the file (without modifying it)
    pub fn format_check(&self, source: &str, language: &str) -> Option<bool> {
        let result = self.format(source, language)?;
        Some(result.changed)
    }

    /// Check if a formatter is available for the language
    pub fn is_available(&self, language: &str) -> bool {
        if let Some(config) = self.formatters.get(language) {
            self.available.contains_key(&config.binary_name)
        } else {
            false
        }
    }

    /// List all registered languages and their availability
    pub fn status(&self) -> Vec<(String, String, bool)> {
        self.formatters
            .iter()
            .map(|(lang, config)| {
                let available = self.available.contains_key(&config.binary_name);
                (lang.clone(), config.binary_name.clone(), available)
            })
            .collect()
    }

    /// Map a file extension to a language name
    pub fn language_for_extension(ext: &str) -> Option<&'static str> {
        match ext {
            "rs" => Some("rust"),
            "py" | "pyi" => Some("python"),
            "go" => Some("go"),
            "js" | "jsx" => Some("javascript"),
            "ts" | "tsx" => Some("typescript"),
            "css" => Some("css"),
            "html" | "htm" => Some("html"),
            "tf" | "tfvars" => Some("hcl"),
            "sql" => Some("sql"),
            _ => None,
        }
    }

    /// Format a file by path (auto-detect language from extension)
    pub fn format_file(&self, path: &Path) -> Option<FormatResult> {
        let ext = path.extension()?.to_str()?;
        let language = Self::language_for_extension(ext)?;
        let source = std::fs::read_to_string(path).ok()?;
        self.format(&source, language)
    }
}

impl Default for FormatterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_for_extension() {
        assert_eq!(
            FormatterRegistry::language_for_extension("rs"),
            Some("rust")
        );
        assert_eq!(
            FormatterRegistry::language_for_extension("py"),
            Some("python")
        );
        assert_eq!(
            FormatterRegistry::language_for_extension("pyi"),
            Some("python")
        );
        assert_eq!(FormatterRegistry::language_for_extension("go"), Some("go"));
        assert_eq!(
            FormatterRegistry::language_for_extension("js"),
            Some("javascript")
        );
        assert_eq!(
            FormatterRegistry::language_for_extension("jsx"),
            Some("javascript")
        );
        assert_eq!(
            FormatterRegistry::language_for_extension("ts"),
            Some("typescript")
        );
        assert_eq!(
            FormatterRegistry::language_for_extension("tsx"),
            Some("typescript")
        );
        assert_eq!(
            FormatterRegistry::language_for_extension("css"),
            Some("css")
        );
        assert_eq!(
            FormatterRegistry::language_for_extension("html"),
            Some("html")
        );
        assert_eq!(
            FormatterRegistry::language_for_extension("htm"),
            Some("html")
        );
        assert_eq!(FormatterRegistry::language_for_extension("tf"), Some("hcl"));
        assert_eq!(
            FormatterRegistry::language_for_extension("tfvars"),
            Some("hcl")
        );
        assert_eq!(
            FormatterRegistry::language_for_extension("sql"),
            Some("sql")
        );
        assert_eq!(FormatterRegistry::language_for_extension("unknown"), None);
    }

    #[test]
    fn test_status_returns_all_registered_formatters() {
        let registry = FormatterRegistry::new();
        let status = registry.status();
        let languages: Vec<&str> = status.iter().map(|(lang, _, _)| lang.as_str()).collect();

        assert!(languages.contains(&"rust"));
        assert!(languages.contains(&"python"));
        assert!(languages.contains(&"go"));
        assert!(languages.contains(&"javascript"));
        assert!(languages.contains(&"typescript"));
        assert!(languages.contains(&"css"));
        assert!(languages.contains(&"html"));
        assert!(languages.contains(&"hcl"));
        assert!(languages.contains(&"sql"));
        assert_eq!(status.len(), 9);
    }

    #[test]
    fn test_is_available_unknown_language() {
        let registry = FormatterRegistry::new();
        assert!(!registry.is_available("brainfuck"));
        assert!(!registry.is_available(""));
        assert!(!registry.is_available("cobol"));
    }

    #[test]
    fn test_format_returns_none_for_unavailable_formatter() {
        let registry = FormatterRegistry {
            formatters: HashMap::new(),
            available: HashMap::new(),
        };
        let result = registry.format("fn main() {}", "rust");
        assert!(result.is_none());
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/format/mod.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/format/mod.rs` captured during libs codegen standardization.
```
