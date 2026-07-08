---
id: libs-compass-src-lint-mod-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/lint/mod.rs`.
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

# Standardized libs/compass/src/lint/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/lint/mod.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `autofix` | libs/compass/src/lint/mod.rs | module | pub | 4 | pub mod autofix; |
| `custom` | libs/compass/src/lint/mod.rs | module | pub | 6 | pub mod custom; |
| `embedded_markdown` | libs/compass/src/lint/mod.rs | module | pub | 8 | pub mod embedded_markdown; |
| `markdown` | libs/compass/src/lint/mod.rs | module | pub | 17 | pub mod markdown; |
| `AsyncApiChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 38 | pub use asyncapi::AsyncApiChecker; |
| `CssChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 39 | pub use css::CssChecker; |
| `CustomLintEngine` | libs/compass/src/lint/mod.rs | re-export | pub | 40 | pub use custom::CustomLintEngine; |
| `DockerfileChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 41 | pub use dockerfile::DockerfileChecker; |
| `GitlabCiChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 42 | pub use gitlab_ci::GitlabCiChecker; |
| `GoChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 43 | pub use go::GoChecker; |
| `GraphqlChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 44 | pub use graphql::GraphqlChecker; |
| `HtmlChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 45 | pub use html::HtmlChecker; |
| `JavaScriptChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 46 | pub use javascript::JavaScriptChecker; |
| `KubernetesChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 47 | pub use kubernetes::KubernetesChecker; |
| `MarkdownChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 48 | pub use markdown::MarkdownChecker; |
| `MdxChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 49 | pub use mdx::MdxChecker; |
| `MermaidChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 50 | pub use mermaid::MermaidChecker; |
| `OpenApiChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 51 | pub use openapi::OpenApiChecker; |
| `OpenRpcChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 52 | pub use openrpc::OpenRpcChecker; |
| `ProtoChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 53 | pub use proto::ProtoChecker; |
| `PythonChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 54 | pub use python::PythonChecker; |
| `RustChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 55 | pub use rust_checker::RustChecker; |
| `detect_sql_injection` | libs/compass/src/lint/mod.rs | re-export | pub | 56 | pub use sql::{detect_sql_injection, SqlChecker}; |
| `SqlChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 56 | pub use sql::{detect_sql_injection, SqlChecker}; |
| `TerraformChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 57 | pub use terraform::TerraformChecker; |
| `TomlChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 58 | pub use toml_checker::TomlChecker; |
| `TypeScriptChecker` | libs/compass/src/lint/mod.rs | re-export | pub | 59 | pub use typescript::TypeScriptChecker; |
| `YamlDispatcher` | libs/compass/src/lint/mod.rs | re-export | pub | 60 | pub use yaml_dispatch::YamlDispatcher; |
| `Checker` | libs/compass/src/lint/mod.rs | trait | pub | 63 | pub trait Checker: Send + Sync { |
| `CheckerRegistry` | libs/compass/src/lint/mod.rs | struct | pub | 70 | pub struct CheckerRegistry { |
| `new` | libs/compass/src/lint/mod.rs | function | pub | 75 | pub fn new() -> Self { |
| `get` | libs/compass/src/lint/mod.rs | function | pub | 99 | pub fn get(&self, language: Language) -> Option<&dyn Checker> { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Language-specific checkers

mod asyncapi;
pub mod autofix;
mod css;
pub mod custom;
mod dockerfile;
pub mod embedded_markdown;
mod gitlab_ci;
mod gitlab_ci_rules;
mod go;
mod graphql;
mod html;
mod javascript;
mod kubernetes;
mod kubernetes_rules;
pub mod markdown;
mod mdx;
mod mermaid;
mod openapi;
mod openrpc;
mod proto;
mod python;
mod python_security;
mod rust_checker;
mod sql;
mod terraform;
mod terraform_rules;
mod toml_checker;
mod typescript;
mod yaml_dispatch;

use crate::checker::LintConfig;
use crate::diagnostic::Diagnostic;
use crate::syntax::{Language, ParsedFile};
use std::collections::HashMap;

pub use asyncapi::AsyncApiChecker;
pub use css::CssChecker;
pub use custom::CustomLintEngine;
pub use dockerfile::DockerfileChecker;
pub use gitlab_ci::GitlabCiChecker;
pub use go::GoChecker;
pub use graphql::GraphqlChecker;
pub use html::HtmlChecker;
pub use javascript::JavaScriptChecker;
pub use kubernetes::KubernetesChecker;
pub use markdown::MarkdownChecker;
pub use mdx::MdxChecker;
pub use mermaid::MermaidChecker;
pub use openapi::OpenApiChecker;
pub use openrpc::OpenRpcChecker;
pub use proto::ProtoChecker;
pub use python::PythonChecker;
pub use rust_checker::RustChecker;
pub use sql::{detect_sql_injection, SqlChecker};
pub use terraform::TerraformChecker;
pub use toml_checker::TomlChecker;
pub use typescript::TypeScriptChecker;
pub use yaml_dispatch::YamlDispatcher;

/// Trait for language-specific checkers
pub trait Checker: Send + Sync {
    fn language(&self) -> Language;
    fn check(&self, file: &ParsedFile, config: &LintConfig) -> Vec<Diagnostic>;
    fn available_rules(&self) -> Vec<&'static str>;
}

/// Registry of all checkers
pub struct CheckerRegistry {
    checkers: HashMap<Language, Box<dyn Checker>>,
}

impl CheckerRegistry {
    pub fn new() -> Self {
        let mut checkers: HashMap<Language, Box<dyn Checker>> = HashMap::new();

        checkers.insert(Language::Python, Box::new(PythonChecker::new()));
        checkers.insert(Language::TypeScript, Box::new(TypeScriptChecker::new()));
        checkers.insert(Language::Rust, Box::new(RustChecker::new()));
        checkers.insert(Language::JavaScript, Box::new(JavaScriptChecker::new()));
        checkers.insert(Language::Go, Box::new(GoChecker::new()));
        checkers.insert(Language::Html, Box::new(HtmlChecker::new()));
        checkers.insert(Language::Css, Box::new(CssChecker::new()));
        checkers.insert(Language::Dockerfile, Box::new(DockerfileChecker));
        checkers.insert(Language::Hcl, Box::new(TerraformChecker));
        checkers.insert(Language::Yaml, Box::new(YamlDispatcher::new()));
        checkers.insert(Language::Markdown, Box::new(MarkdownChecker::new()));
        checkers.insert(Language::Mdx, Box::new(MdxChecker::new()));
        checkers.insert(Language::Mermaid, Box::new(MermaidChecker::new()));
        checkers.insert(Language::Toml, Box::new(TomlChecker::new()));
        checkers.insert(Language::Sql, Box::new(SqlChecker::new()));
        checkers.insert(Language::Proto, Box::new(ProtoChecker::new()));
        checkers.insert(Language::GraphQL, Box::new(GraphqlChecker::new()));

        Self { checkers }
    }

    pub fn get(&self, language: Language) -> Option<&dyn Checker> {
        self.checkers.get(&language).map(|c| c.as_ref())
    }
}

impl Default for CheckerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/lint/mod.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/lint/mod.rs` captured during libs codegen standardization.
```
