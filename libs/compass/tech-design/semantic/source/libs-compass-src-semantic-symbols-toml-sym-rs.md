---
id: libs-compass-src-semantic-symbols-toml-sym-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/semantic/symbols/toml_sym.rs`.
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

# Standardized libs/compass/src/semantic/symbols/toml_sym.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/semantic/symbols/toml_sym.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `visit_toml_lines` | libs/compass/src/semantic/symbols/toml_sym.rs | function | pub | 13 | pub(crate) fn visit_toml_lines(&mut self, source: &str) { |
| `build_toml` | libs/compass/src/semantic/symbols/toml_sym.rs | function | pub | 76 | pub fn build_toml(mut self, file: &crate::syntax::ParsedFile) -> super::SymbolTable { |
| `build_toml_from_source` | libs/compass/src/semantic/symbols/toml_sym.rs | function | pub | 83 | pub fn build_toml_from_source(mut self, source: &str) -> super::SymbolTable { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! TOML symbol extraction (line-based)
//!
//! Extracts symbols from TOML files:
//! - Sections `[name]` -> Module
//! - Key-value pairs `key = value` -> Variable
//! - Array tables `[[name]]` -> Module

use super::{SymbolKind, SymbolTableBuilder};
use crate::diagnostic::{Position, Range};

impl SymbolTableBuilder {
    /// Visit TOML source and extract symbols.
    pub(crate) fn visit_toml_lines(&mut self, source: &str) {
        for (line_idx, line) in source.lines().enumerate() {
            let line_num = line_idx as u32;
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Array table: [[name]]
            if trimmed.starts_with("[[") && trimmed.ends_with("]]") {
                let name = &trimmed[2..trimmed.len() - 2].trim();
                if !name.is_empty() {
                    let col = line.find('[').unwrap_or(0) as u32;
                    self.table.add_symbol(
                        name.to_string(),
                        SymbolKind::Module,
                        mk_range(line_num, col, trimmed.len()),
                        None,
                        Some("array table".to_string()),
                        self.current_scope,
                    );
                }
                continue;
            }

            // Table header: [name]
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let name = &trimmed[1..trimmed.len() - 1].trim();
                if !name.is_empty() {
                    let col = line.find('[').unwrap_or(0) as u32;
                    self.table.add_symbol(
                        name.to_string(),
                        SymbolKind::Module,
                        mk_range(line_num, col, trimmed.len()),
                        None,
                        Some("table".to_string()),
                        self.current_scope,
                    );
                }
                continue;
            }

            // Key-value pair: key = value
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                if !key.is_empty() {
                    let col = line.find(key).unwrap_or(0) as u32;
                    self.table.add_symbol(
                        key.to_string(),
                        SymbolKind::Variable,
                        mk_range(line_num, col, key.len()),
                        None,
                        Some("key-value".to_string()),
                        self.current_scope,
                    );
                }
            }
        }
    }

    /// Build symbol table for a TOML file (line-based).
    pub fn build_toml(mut self, file: &crate::syntax::ParsedFile) -> super::SymbolTable {
        self.visit_toml_lines(&file.source);
        self.table
    }

    /// Build symbol table for TOML from raw source (test helper).
    #[cfg(test)]
    pub fn build_toml_from_source(mut self, source: &str) -> super::SymbolTable {
        self.visit_toml_lines(source);
        self.table
    }
}

fn mk_range(line: u32, col: u32, len: usize) -> Range {
    Range::new(
        Position::new(line, col),
        Position::new(line, col + len as u32),
    )
}

#[cfg(test)]
mod tests {
    use super::super::{SymbolKind, SymbolTableBuilder};

    fn build(source: &str) -> super::super::SymbolTable {
        SymbolTableBuilder::new().build_toml_from_source(source)
    }

    #[test]
    fn test_table_headers() {
        let src = "[package]\nname = \"test\"\n[dependencies]\nfoo = \"1.0\"\n";
        let table = build(src);
        let modules: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Module)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            modules.contains(&"package"),
            "missing [package], got {:?}",
            modules
        );
        assert!(
            modules.contains(&"dependencies"),
            "missing [dependencies], got {:?}",
            modules
        );
    }

    #[test]
    fn test_key_value_pairs() {
        let src = "[package]\nname = \"test\"\nversion = \"0.1.0\"\n";
        let table = build(src);
        let vars: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Variable)
            .map(|s| s.name.as_str())
            .collect();
        assert!(vars.contains(&"name"), "missing 'name', got {:?}", vars);
        assert!(
            vars.contains(&"version"),
            "missing 'version', got {:?}",
            vars
        );
    }

    #[test]
    fn test_array_table() {
        let src = "[[bin]]\nname = \"app\"\npath = \"src/main.rs\"\n[[bin]]\nname = \"tool\"\n";
        let table = build(src);
        let modules: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Module)
            .map(|s| s.name.as_str())
            .collect();
        assert_eq!(
            modules.iter().filter(|&&m| m == "bin").count(),
            2,
            "expected 2 [[bin]] modules, got {:?}",
            modules
        );
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/semantic/symbols/toml_sym.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/semantic/symbols/toml_sym.rs` captured during libs codegen standardization.
```
