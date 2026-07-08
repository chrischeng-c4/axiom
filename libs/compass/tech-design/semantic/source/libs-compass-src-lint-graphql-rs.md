---
id: libs-compass-src-lint-graphql-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/lint/graphql.rs`.
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

# Standardized libs/compass/src/lint/graphql.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/lint/graphql.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `GraphqlChecker` | libs/compass/src/lint/graphql.rs | struct | pub | 14 | pub struct GraphqlChecker; |
| `new` | libs/compass/src/lint/graphql.rs | function | pub | 17 | pub fn new() -> Self { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! GraphQL lint checker — AST-based via tree-sitter-graphql (R3)
//!
//! Rules: GQ001-GQ007
//!
//! When a real tree-sitter tree is available the checker uses AST node matching;
//! it falls back to the proven line-based implementation for dummy/error trees.

use super::Checker;
use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range};
use crate::syntax::{Language, ParsedFile};
use std::collections::{HashMap, HashSet};

pub struct GraphqlChecker;

impl GraphqlChecker {
    pub fn new() -> Self {
        Self
    }

    // =========================================================================
    // AST-based checks (tree-sitter-graphql node kinds)
    // =========================================================================

    /// GQ001 via AST: report tree-sitter syntax errors directly.
    fn ast_check_syntax(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for err in file.collect_errors() {
            let (row, col) = err.start_position;
            let pos = Position::new((row - 1) as u32, (col - 1) as u32);
            diags.push(Diagnostic::new(
                Range::new(pos, pos),
                DiagnosticSeverity::Error,
                "GQ001",
                DiagnosticCategory::Syntax,
                "GraphQL syntax error",
            ));
        }
        diags
    }

    /// GQ002 via AST: collect defined type names and flag undefined references.
    fn ast_check_undefined_types(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let source = file.source.as_bytes();

        // Built-in scalar types
        let mut defined: HashSet<String> = ["String", "Int", "Float", "Boolean", "ID"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut refs: Vec<(String, u32)> = Vec::new();

        let type_def_kinds: &[&str] = &[
            "object_type_definition",
            "interface_type_definition",
            "union_type_definition",
            "enum_type_definition",
            "input_object_type_definition",
            "scalar_type_definition",
        ];

        file.walk(|node, _depth| {
            let kind = node.kind();

            // Collect type definitions
            if type_def_kinds.contains(&kind) {
                if let Some(name_node) = node.child_by_field_name("name") {
                    if let Ok(name) = name_node.utf8_text(source) {
                        defined.insert(name.to_string());
                    }
                }
            }

            // Collect named type references (fields, arguments, etc.)
            if kind == "named_type" {
                if let Ok(name) = node.utf8_text(source) {
                    let line = node.start_position().row as u32;
                    refs.push((name.to_string(), line));
                }
            }

            true
        });

        for (tn, line) in &refs {
            if !defined.contains(tn.as_str()) {
                diags.push(Diagnostic::new(
                    lr(*line),
                    DiagnosticSeverity::Warning,
                    "GQ002",
                    DiagnosticCategory::Names,
                    format!("Reference to undefined type '{}'", tn),
                ));
            }
        }
        diags
    }

    /// GQ003 via AST: find @deprecated directives on field definitions.
    fn ast_check_deprecated(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let source = file.source.as_bytes();

        file.walk(|node, _depth| {
            if node.kind() == "directive" {
                if let Some(name_node) = node.child_by_field_name("name") {
                    if name_node.utf8_text(source).ok() == Some("deprecated") {
                        let line = node.start_position().row as u32;
                        diags.push(Diagnostic::new(
                            lr(line),
                            DiagnosticSeverity::Warning,
                            "GQ003",
                            DiagnosticCategory::Logic,
                            "Field marked @deprecated — consider removing or updating usage",
                        ));
                    }
                }
            }
            true
        });
        diags
    }

    /// GQ004 via AST: measure selection set nesting depth.
    fn ast_check_deep_nesting(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut warned: HashSet<u32> = HashSet::new();

        file.walk(|node, depth| {
            if node.kind() == "selection_set" && depth > 5 {
                let line = node.start_position().row as u32;
                if warned.insert(line) {
                    diags.push(Diagnostic::new(
                        lr(line),
                        DiagnosticSeverity::Warning,
                        "GQ004",
                        DiagnosticCategory::Style,
                        format!("Nesting depth {} exceeds maximum of 5", depth),
                    ));
                }
            }
            true
        });
        diags
    }

    /// GQ005 via AST: type/interface/input definitions without a description.
    fn ast_check_missing_descriptions(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let source = file.source.as_bytes();

        let def_kinds: &[&str] = &[
            "object_type_definition",
            "interface_type_definition",
            "input_object_type_definition",
        ];

        file.walk(|node, _depth| {
            if def_kinds.contains(&node.kind()) {
                // Check for a description string_value child
                let has_desc = (0..node.child_count())
                    .filter_map(|i| node.child(i))
                    .any(|c| c.kind() == "string_value" || c.kind() == "block_string_value");
                if !has_desc {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = name_node.utf8_text(source).unwrap_or("");
                        if !matches!(name, "Query" | "Mutation" | "Subscription") {
                            let line = node.start_position().row as u32;
                            diags.push(Diagnostic::new(
                                lr(line),
                                DiagnosticSeverity::Hint,
                                "GQ005",
                                DiagnosticCategory::Style,
                                format!("Type '{}' has no description", name),
                            ));
                        }
                    }
                }
            }
            true
        });
        diags
    }

    /// GQ006 via AST: fragment defined but never spread.
    fn ast_check_unused_fragments(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let source = file.source.as_bytes();
        let mut defs: HashMap<String, u32> = HashMap::new();
        let mut spreads: HashSet<String> = HashSet::new();

        file.walk(|node, _depth| {
            match node.kind() {
                "fragment_definition" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        if let Ok(name) = name_node.utf8_text(source) {
                            defs.insert(name.to_string(), node.start_position().row as u32);
                        }
                    }
                }
                "fragment_spread" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        if let Ok(name) = name_node.utf8_text(source) {
                            spreads.insert(name.to_string());
                        }
                    }
                }
                _ => {}
            }
            true
        });

        for (name, line) in &defs {
            if !spreads.contains(name.as_str()) {
                diags.push(Diagnostic::new(
                    lr(*line),
                    DiagnosticSeverity::Warning,
                    "GQ006",
                    DiagnosticCategory::Logic,
                    format!("Fragment '{}' is defined but never used", name),
                ));
            }
        }
        diags
    }

    /// GQ007 via AST: duplicate fields within the same selection set.
    fn ast_check_duplicate_fields(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let source = file.source.as_bytes();

        file.walk(|node, _depth| {
            if node.kind() == "selection_set" {
                let mut seen: HashMap<String, u32> = HashMap::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "field" {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            if let Ok(fname) = name_node.utf8_text(source) {
                                let line = child.start_position().row as u32;
                                if let Some(&prev) = seen.get(fname) {
                                    diags.push(Diagnostic::new(
                                        lr(line),
                                        DiagnosticSeverity::Warning,
                                        "GQ007",
                                        DiagnosticCategory::Logic,
                                        format!(
                                            "Duplicate field '{}' (first at line {})",
                                            fname,
                                            prev + 1
                                        ),
                                    ));
                                } else {
                                    seen.insert(fname.to_string(), line);
                                }
                            }
                        }
                    }
                }
            }
            true
        });
        diags
    }

    // =========================================================================
    // Line-based fallback (used for tests / dummy trees)
    // =========================================================================

    fn lb_check_syntax(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut depth: i32 = 0;
        let mut open_line: Option<u32> = None;
        for (i, line) in source.lines().enumerate() {
            let t = line.trim();
            if t.starts_with('#') || t.is_empty() {
                continue;
            }
            for ch in t.chars() {
                if ch == '{' {
                    if depth == 0 {
                        open_line = Some(i as u32);
                    }
                    depth += 1;
                } else if ch == '}' {
                    depth -= 1;
                    if depth < 0 {
                        diags.push(Diagnostic::new(
                            lr(i as u32),
                            DiagnosticSeverity::Error,
                            "GQ001",
                            DiagnosticCategory::Syntax,
                            "Unmatched closing brace",
                        ));
                        depth = 0;
                    }
                }
            }
        }
        if depth > 0 {
            diags.push(Diagnostic::new(
                lr(open_line.unwrap_or(0)),
                DiagnosticSeverity::Error,
                "GQ001",
                DiagnosticCategory::Syntax,
                format!("Unmatched opening brace ({} unclosed)", depth),
            ));
        }
        diags
    }

    fn lb_check_undefined_types(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut defined: HashSet<String> = ["String", "Int", "Float", "Boolean", "ID"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut refs: Vec<(String, u32)> = Vec::new();
        for (i, line) in source.lines().enumerate() {
            let t = line.trim();
            if t.starts_with('#') || t.is_empty() {
                continue;
            }
            for kw in &[
                "type ",
                "input ",
                "interface ",
                "enum ",
                "union ",
                "scalar ",
            ] {
                if t.starts_with(kw) {
                    let name = t[kw.len()..].split_whitespace().next().unwrap_or("");
                    if !name.is_empty() {
                        defined.insert(name.to_string());
                    }
                }
            }
            if t.contains(':') {
                if let Some(cp) = t.find(':') {
                    let tp = t[cp + 1..]
                        .trim()
                        .trim_start_matches('[')
                        .trim_end_matches('!')
                        .trim_end_matches(']')
                        .trim_end_matches('!')
                        .trim();
                    if !tp.is_empty()
                        && tp.chars().next().map_or(false, |c| c.is_uppercase())
                        && !tp.contains('(')
                    {
                        refs.push((tp.to_string(), i as u32));
                    }
                }
            }
        }
        for (tn, line) in &refs {
            if !defined.contains(tn.as_str()) {
                diags.push(Diagnostic::new(
                    lr(*line),
                    DiagnosticSeverity::Warning,
                    "GQ002",
                    DiagnosticCategory::Names,
                    format!("Reference to undefined type '{}'", tn),
                ));
            }
        }
        diags
    }

    fn lb_check_deprecated(&self, source: &str) -> Vec<Diagnostic> {
        source
            .lines()
            .enumerate()
            .filter(|(_, l)| l.contains("@deprecated"))
            .map(|(i, _)| {
                Diagnostic::new(
                    lr(i as u32),
                    DiagnosticSeverity::Warning,
                    "GQ003",
                    DiagnosticCategory::Logic,
                    "Field marked @deprecated — consider removing or updating usage",
                )
            })
            .collect()
    }

    fn lb_check_deep_nesting(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut depth: i32 = 0;
        let mut warned: HashSet<u32> = HashSet::new();
        for (i, line) in source.lines().enumerate() {
            let t = line.trim();
            if t.starts_with('#') || t.is_empty() {
                continue;
            }
            for ch in t.chars() {
                if ch == '{' {
                    depth += 1;
                    if depth > 5 && !warned.contains(&(i as u32)) {
                        warned.insert(i as u32);
                        diags.push(Diagnostic::new(
                            lr(i as u32),
                            DiagnosticSeverity::Warning,
                            "GQ004",
                            DiagnosticCategory::Style,
                            format!("Nesting depth {} exceeds maximum of 5", depth),
                        ));
                    }
                } else if ch == '}' {
                    depth -= 1;
                    if depth < 0 {
                        depth = 0;
                    }
                }
            }
        }
        diags
    }

    fn lb_check_missing_descriptions(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let t = line.trim();
            for kw in &["type ", "input ", "interface "] {
                if t.starts_with(kw) {
                    let name = t[kw.len()..].split_whitespace().next().unwrap_or("");
                    if matches!(name, "Query" | "Mutation" | "Subscription") {
                        continue;
                    }
                    let has_desc = i > 0 && {
                        let prev = lines[i - 1].trim();
                        prev.starts_with('#') || prev.starts_with("\"\"\"") || prev.starts_with('"')
                    };
                    if !has_desc && !name.is_empty() {
                        diags.push(Diagnostic::new(
                            lr(i as u32),
                            DiagnosticSeverity::Hint,
                            "GQ005",
                            DiagnosticCategory::Style,
                            format!("Type '{}' has no description", name),
                        ));
                    }
                }
            }
        }
        diags
    }

    fn lb_check_unused_fragments(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut defs: HashMap<String, u32> = HashMap::new();
        let mut spreads: HashSet<String> = HashSet::new();
        for (i, line) in source.lines().enumerate() {
            let t = line.trim();
            if t.starts_with('#') || t.is_empty() {
                continue;
            }
            if t.starts_with("fragment ") {
                let name = t
                    .strip_prefix("fragment ")
                    .unwrap_or("")
                    .split_whitespace()
                    .next()
                    .unwrap_or("");
                if !name.is_empty() {
                    defs.insert(name.to_string(), i as u32);
                }
            }
            let mut s = t;
            while let Some(pos) = s.find("...") {
                let after = &s[pos + 3..];
                let sn = after
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .next()
                    .unwrap_or("");
                if !sn.is_empty() && sn != "on" {
                    spreads.insert(sn.to_string());
                }
                if pos + 3 >= s.len() {
                    break;
                }
                s = &s[pos + 3..];
            }
        }
        for (name, line) in &defs {
            if !spreads.contains(name) {
                diags.push(Diagnostic::new(
                    lr(*line),
                    DiagnosticSeverity::Warning,
                    "GQ006",
                    DiagnosticCategory::Logic,
                    format!("Fragment '{}' is defined but never used", name),
                ));
            }
        }
        diags
    }

    fn lb_check_duplicate_fields(&self, source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut depth_fields: Vec<HashMap<String, u32>> = vec![HashMap::new()];
        let mut depth: usize = 0;
        let skip = [
            "...",
            "query",
            "mutation",
            "subscription",
            "fragment",
            "type",
            "input",
            "interface",
            "enum",
        ];
        for (i, line) in source.lines().enumerate() {
            let t = line.trim();
            if t.starts_with('#') || t.is_empty() {
                continue;
            }
            for ch in t.chars() {
                if ch == '{' {
                    depth += 1;
                    if depth >= depth_fields.len() {
                        depth_fields.push(HashMap::new());
                    } else {
                        depth_fields[depth].clear();
                    }
                } else if ch == '}' {
                    if depth > 0 {
                        depth_fields[depth].clear();
                        depth -= 1;
                    }
                }
            }
            if !t.contains('{')
                && !t.contains('}')
                && depth > 0
                && !skip.iter().any(|s| t.starts_with(s))
            {
                let field = t
                    .split(|c: char| c == '(' || c == ':' || c == '@' || c.is_whitespace())
                    .next()
                    .unwrap_or("");
                if !field.is_empty()
                    && field.chars().next().map_or(false, |c| c.is_alphabetic())
                    && depth < depth_fields.len()
                {
                    if let Some(&prev) = depth_fields[depth].get(field) {
                        diags.push(Diagnostic::new(
                            lr(i as u32),
                            DiagnosticSeverity::Warning,
                            "GQ007",
                            DiagnosticCategory::Logic,
                            format!("Duplicate field '{}' (first at line {})", field, prev + 1),
                        ));
                    } else {
                        depth_fields[depth].insert(field.to_string(), i as u32);
                    }
                }
            }
        }
        diags
    }
}

impl Default for GraphqlChecker {
    fn default() -> Self {
        Self::new()
    }
}

fn lr(line: u32) -> Range {
    Range::new(Position::new(line, 0), Position::new(line, u32::MAX))
}

impl Checker for GraphqlChecker {
    fn language(&self) -> Language {
        Language::GraphQL
    }

    fn check(&self, file: &ParsedFile, _config: &LintConfig) -> Vec<Diagnostic> {
        // Use AST-based checks when the file has a real tree-sitter tree
        // (language == GraphQL means parser.parse() succeeded with the real grammar).
        // Fall back to line-based for dummy trees (line_based() or has_errors).
        if file.language == Language::GraphQL && !file.has_errors && !file.is_line_based {
            let mut d = Vec::new();
            d.extend(self.ast_check_syntax(file));
            d.extend(self.ast_check_undefined_types(file));
            d.extend(self.ast_check_deprecated(file));
            d.extend(self.ast_check_deep_nesting(file));
            d.extend(self.ast_check_missing_descriptions(file));
            d.extend(self.ast_check_unused_fragments(file));
            d.extend(self.ast_check_duplicate_fields(file));
            d
        } else {
            let mut d = Vec::new();
            d.extend(self.lb_check_syntax(&file.source));
            d.extend(self.lb_check_undefined_types(&file.source));
            d.extend(self.lb_check_deprecated(&file.source));
            d.extend(self.lb_check_deep_nesting(&file.source));
            d.extend(self.lb_check_missing_descriptions(&file.source));
            d.extend(self.lb_check_unused_fragments(&file.source));
            d.extend(self.lb_check_duplicate_fields(&file.source));
            d
        }
    }

    fn available_rules(&self) -> Vec<&'static str> {
        vec![
            "GQ001", "GQ002", "GQ003", "GQ004", "GQ005", "GQ006", "GQ007",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::LintConfig;

    fn make(s: &str) -> ParsedFile {
        ParsedFile::line_based(s.to_string(), Language::GraphQL)
    }
    fn codes(d: &[Diagnostic]) -> Vec<&str> {
        d.iter().map(|d| d.code.as_str()).collect()
    }

    #[test]
    fn test_unmatched_braces() {
        let d = GraphqlChecker::new().check(
            &make("type User {\n  id: ID!\n  name: String\n"),
            &LintConfig::default(),
        );
        assert!(
            codes(&d).contains(&"GQ001"),
            "expected GQ001, got {:?}",
            codes(&d)
        );
    }

    #[test]
    fn test_deprecated_field() {
        let d = GraphqlChecker::new().check(
            &make("type User {\n  id: ID!\n  old: String @deprecated(reason: \"Use name\")\n}\n"),
            &LintConfig::default(),
        );
        assert!(
            codes(&d).contains(&"GQ003"),
            "expected GQ003, got {:?}",
            codes(&d)
        );
    }

    #[test]
    fn test_deep_nesting() {
        let src = "query {\n  a {\n    b {\n      c {\n        d {\n          e {\n            f\n          }\n        }\n      }\n    }\n  }\n}\n";
        let d = GraphqlChecker::new().check(&make(src), &LintConfig::default());
        assert!(
            codes(&d).contains(&"GQ004"),
            "expected GQ004, got {:?}",
            codes(&d)
        );
    }

    #[test]
    fn test_unused_fragment() {
        let src =
            "fragment UserFields on User {\n  id\n  name\n}\nquery {\n  users {\n    id\n  }\n}\n";
        let d = GraphqlChecker::new().check(&make(src), &LintConfig::default());
        assert!(
            codes(&d).contains(&"GQ006"),
            "expected GQ006, got {:?}",
            codes(&d)
        );
    }

    #[test]
    fn test_duplicate_field() {
        let d = GraphqlChecker::new().check(
            &make("query {\n  users {\n    id\n    name\n    id\n  }\n}\n"),
            &LintConfig::default(),
        );
        assert!(
            codes(&d).contains(&"GQ007"),
            "expected GQ007, got {:?}",
            codes(&d)
        );
    }

    #[test]
    fn test_missing_description() {
        let d = GraphqlChecker::new()
            .check(&make("type User {\n  id: ID!\n}\n"), &LintConfig::default());
        assert!(
            codes(&d).contains(&"GQ005"),
            "expected GQ005, got {:?}",
            codes(&d)
        );
    }

    #[test]
    fn test_fragment_used() {
        let src = "fragment UserFields on User {\n  id\n  name\n}\nquery {\n  users {\n    ...UserFields\n  }\n}\n";
        let d = GraphqlChecker::new().check(&make(src), &LintConfig::default());
        assert!(
            !codes(&d).contains(&"GQ006"),
            "should not flag used fragment, got {:?}",
            codes(&d)
        );
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/lint/graphql.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/lint/graphql.rs` captured during libs codegen standardization.
```
