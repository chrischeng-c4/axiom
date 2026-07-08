---
id: libs-compass-src-semantic-symbols-typescript-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/semantic/symbols/typescript.rs`.
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

# Standardized libs/compass/src/semantic/symbols/typescript.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/semantic/symbols/typescript.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `visit_typescript_node` | libs/compass/src/semantic/symbols/typescript.rs | function | pub | 21 | pub(crate) fn visit_typescript_node( |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! TypeScript/JavaScript symbol extraction visitor methods
//!
//! Extracts: functions, classes, interfaces, type aliases, enums,
//! variables, and imports from TS/JS ASTs.

use super::{SymbolKind, SymbolTableBuilder, TypeInfo};
use crate::diagnostic::Range;
use crate::syntax::ParsedFile;

/// Extract name and location from a node's "name" field
fn name_loc(node: &tree_sitter::Node<'_>, file: &ParsedFile) -> (String, Range) {
    let name_node = node.child_by_field_name("name");
    let name = name_node
        .map(|n| file.node_text(&n).to_string())
        .unwrap_or_default();
    let loc = name_node.map(|n| Range::from_node(&n)).unwrap_or_default();
    (name, loc)
}

impl SymbolTableBuilder {
    pub(crate) fn visit_typescript_node(
        &mut self,
        node: &tree_sitter::Node<'_>,
        file: &ParsedFile,
    ) {
        if node.is_error() || node.is_missing() {
            return;
        }

        match node.kind() {
            "function_declaration" => {
                self.visit_ts_function(node, file);
                return;
            }
            "class_declaration" => {
                self.visit_ts_class(node, file);
                return;
            }
            "interface_declaration" => {
                self.visit_ts_interface(node, file);
                return;
            }
            "type_alias_declaration" => {
                self.visit_ts_type_alias(node, file);
                return;
            }
            "enum_declaration" => {
                self.visit_ts_enum(node, file);
                return;
            }
            "lexical_declaration" | "variable_declaration" => {
                self.visit_ts_variable_decl(node, file);
                return;
            }
            "import_statement" => {
                self.visit_ts_import(node, file);
                return;
            }
            "export_statement" => {
                // Recurse into export to find the actual declaration
                self.visit_ts_children(node, file);
                return;
            }
            _ => {}
        }
        self.visit_ts_children(node, file);
    }

    fn visit_ts_children(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let mut c = node.walk();
        for child in node.children(&mut c) {
            if !child.is_error() && !child.is_missing() {
                self.visit_typescript_node(&child, file);
            }
        }
    }

    fn visit_ts_function(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let (name, loc) = name_loc(node, file);
        let ret = node
            .child_by_field_name("return_type")
            .map(|n| TypeInfo::Named(file.node_text(&n).to_string()));
        self.table.add_symbol(
            name,
            SymbolKind::Function,
            loc,
            ret,
            None,
            self.current_scope,
        );
        self.push_scope();
        if let Some(p) = node.child_by_field_name("parameters") {
            self.visit_ts_params(&p, file);
        }
        if let Some(b) = node.child_by_field_name("body") {
            self.visit_typescript_node(&b, file);
        }
        self.pop_scope();
    }

    fn visit_ts_class(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let (name, loc) = name_loc(node, file);
        self.table
            .add_symbol(name, SymbolKind::Class, loc, None, None, self.current_scope);
        self.push_scope();
        if let Some(b) = node.child_by_field_name("body") {
            self.visit_typescript_node(&b, file);
        }
        self.pop_scope();
    }

    fn visit_ts_interface(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let (name, loc) = name_loc(node, file);
        self.table.add_symbol(
            name,
            SymbolKind::Interface,
            loc,
            None,
            None,
            self.current_scope,
        );
    }

    fn visit_ts_type_alias(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let (name, loc) = name_loc(node, file);
        self.table.add_symbol(
            name,
            SymbolKind::TypeAlias,
            loc,
            None,
            None,
            self.current_scope,
        );
    }

    fn visit_ts_enum(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let (name, loc) = name_loc(node, file);
        self.table
            .add_symbol(name, SymbolKind::Enum, loc, None, None, self.current_scope);
        self.push_scope();
        if let Some(body) = node.child_by_field_name("body") {
            let mut c = body.walk();
            for child in body.children(&mut c) {
                if child.kind() == "enum_member" || child.kind() == "property_identifier" {
                    let raw = file.node_text(&child).to_string();
                    let mname = raw.split('=').next().unwrap_or("").trim().to_string();
                    if !mname.is_empty() {
                        self.table.add_symbol(
                            mname,
                            SymbolKind::EnumMember,
                            Range::from_node(&child),
                            None,
                            None,
                            self.current_scope,
                        );
                    }
                }
            }
        }
        self.pop_scope();
    }

    fn visit_ts_variable_decl(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let mut c = node.walk();
        for child in node.children(&mut c) {
            if child.kind() != "variable_declarator" {
                continue;
            }
            let (name, loc) = name_loc(&child, file);
            let ti = child
                .child_by_field_name("type")
                .map(|n| TypeInfo::Named(file.node_text(&n).to_string()));
            if !name.is_empty() {
                self.table.add_symbol(
                    name,
                    SymbolKind::Variable,
                    loc,
                    ti,
                    None,
                    self.current_scope,
                );
            }
            if let Some(v) = child.child_by_field_name("value") {
                self.visit_typescript_node(&v, file);
            }
        }
    }

    fn visit_ts_import(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let mut c = node.walk();
        for child in node.children(&mut c) {
            if child.kind() == "import_clause" {
                self.visit_ts_import_clause(&child, file);
            }
        }
    }

    fn visit_ts_import_clause(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let mut c = node.walk();
        for child in node.children(&mut c) {
            if child.kind() == "identifier" {
                let name = file.node_text(&child).to_string();
                self.table.add_symbol(
                    name,
                    SymbolKind::Import,
                    Range::from_node(&child),
                    None,
                    None,
                    self.current_scope,
                );
            } else if child.kind() == "named_imports" {
                let mut ic = child.walk();
                for spec in child.children(&mut ic) {
                    if spec.kind() != "import_specifier" {
                        continue;
                    }
                    let alias = spec.child_by_field_name("alias");
                    let nn = alias.or_else(|| spec.child_by_field_name("name"));
                    if let Some(n) = nn {
                        let name = file.node_text(&n).to_string();
                        self.table.add_symbol(
                            name,
                            SymbolKind::Import,
                            Range::from_node(&n),
                            None,
                            None,
                            self.current_scope,
                        );
                    }
                }
            }
        }
    }

    fn visit_ts_params(&mut self, params: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let mut c = params.walk();
        for child in params.children(&mut c) {
            match child.kind() {
                "required_parameter" | "optional_parameter" => {
                    let pn = child.child_by_field_name("pattern");
                    let name = pn
                        .map(|n| file.node_text(&n).to_string())
                        .unwrap_or_default();
                    let loc = pn.map(|n| Range::from_node(&n)).unwrap_or_default();
                    let ti = child
                        .child_by_field_name("type")
                        .map(|n| TypeInfo::Named(file.node_text(&n).to_string()));
                    if !name.is_empty() {
                        self.table.add_symbol(
                            name,
                            SymbolKind::Parameter,
                            loc,
                            ti,
                            None,
                            self.current_scope,
                        );
                    }
                }
                "identifier" => {
                    let name = file.node_text(&child).to_string();
                    if !name.is_empty() {
                        self.table.add_symbol(
                            name,
                            SymbolKind::Parameter,
                            Range::from_node(&child),
                            None,
                            None,
                            self.current_scope,
                        );
                    }
                }
                _ => {}
            }
        }
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/semantic/symbols/typescript.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/semantic/symbols/typescript.rs` captured during libs codegen standardization.
```
