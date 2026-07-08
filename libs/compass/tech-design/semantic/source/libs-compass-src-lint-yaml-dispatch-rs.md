---
id: libs-compass-src-lint-yaml-dispatch-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/lint/yaml_dispatch.rs`.
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

# Standardized libs/compass/src/lint/yaml_dispatch.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/lint/yaml_dispatch.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `YamlDispatcher` | libs/compass/src/lint/yaml_dispatch.rs | struct | pub | 19 | pub struct YamlDispatcher { |
| `new` | libs/compass/src/lint/yaml_dispatch.rs | function | pub | 28 | pub fn new() -> Self { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! YAML/JSON dispatcher — routes to the appropriate sub-checker based on content

use super::asyncapi::AsyncApiChecker;
use super::openapi::OpenApiChecker;
use super::openrpc::OpenRpcChecker;
use super::{Checker, GitlabCiChecker, KubernetesChecker};
use crate::checker::LintConfig;
use crate::diagnostic::Diagnostic;
use crate::syntax::{Language, ParsedFile};

/// Composite checker that dispatches YAML/JSON files to the appropriate sub-checker.
///
/// Routing priority:
/// 1. OpenAPI 3.x  — `openapi: 3.` in first 10 lines
/// 2. AsyncAPI     — `asyncapi:` in first 10 lines
/// 3. OpenRPC      — `"openrpc"` and `"methods"` in source
/// 4. Kubernetes   — `apiVersion:` and `kind:` at line start
/// 5. GitLab CI    — fallback
pub struct YamlDispatcher {
    k8s: KubernetesChecker,
    gitlab: GitlabCiChecker,
    openapi: OpenApiChecker,
    asyncapi: AsyncApiChecker,
    openrpc: OpenRpcChecker,
}

impl YamlDispatcher {
    pub fn new() -> Self {
        Self {
            k8s: KubernetesChecker::new(),
            gitlab: GitlabCiChecker::new(),
            openapi: OpenApiChecker::new(),
            asyncapi: AsyncApiChecker::new(),
            openrpc: OpenRpcChecker::new(),
        }
    }

    /// Determine whether the source looks like a Kubernetes manifest.
    ///
    /// A file is treated as K8s when it has at least one line starting with
    /// `apiVersion:` AND at least one line starting with `kind:`.
    fn is_kubernetes(source: &str) -> bool {
        let mut has_api_version = false;
        let mut has_kind = false;

        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("apiVersion:") {
                has_api_version = true;
            }
            if trimmed.starts_with("kind:") {
                has_kind = true;
            }
            if has_api_version && has_kind {
                return true;
            }
        }

        false
    }
}

impl Default for YamlDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Checker for YamlDispatcher {
    fn language(&self) -> Language {
        Language::Yaml
    }

    fn check(&self, file: &ParsedFile, config: &LintConfig) -> Vec<Diagnostic> {
        if OpenApiChecker::is_openapi(&file.source) {
            self.openapi.check(file, config)
        } else if AsyncApiChecker::is_asyncapi(&file.source) {
            self.asyncapi.check(file, config)
        } else if OpenRpcChecker::is_openrpc(&file.source) {
            self.openrpc.check(file, config)
        } else if Self::is_kubernetes(&file.source) {
            self.k8s.check(file, config)
        } else {
            self.gitlab.check(file, config)
        }
    }

    fn available_rules(&self) -> Vec<&'static str> {
        let mut rules = self.k8s.available_rules();
        rules.extend(self.gitlab.available_rules());
        rules.extend(self.openapi.available_rules());
        rules.extend(self.asyncapi.available_rules());
        rules.extend(self.openrpc.available_rules());
        rules
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/lint/yaml_dispatch.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/lint/yaml_dispatch.rs` captured during libs codegen standardization.
```
