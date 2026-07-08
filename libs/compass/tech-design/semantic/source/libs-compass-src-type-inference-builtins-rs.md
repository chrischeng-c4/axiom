---
id: libs-compass-src-type-inference-builtins-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/type_inference/builtins.rs`.
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

# Standardized libs/compass/src/type_inference/builtins.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/type_inference/builtins.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `add_builtins` | libs/compass/src/type_inference/builtins.rs | function | pub | 9 | pub fn add_builtins(env: &mut TypeEnv) { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Builtin type definitions for Python
//!
//! This module provides type bindings for Python builtin functions.

use super::ty::{Param, ParamKind, Type};
use super::type_env::TypeEnv;

/// Add Python builtin function types to the environment
pub fn add_builtins(env: &mut TypeEnv) {
    // Builtin functions
    env.bind(
        "len".to_string(),
        Type::callable(vec![Type::Any], Type::Int),
    );
    env.bind(
        "str".to_string(),
        Type::callable(vec![Type::Any], Type::Str),
    );
    env.bind(
        "int".to_string(),
        Type::callable(vec![Type::Any], Type::Int),
    );
    env.bind(
        "float".to_string(),
        Type::callable(vec![Type::Any], Type::Float),
    );
    env.bind(
        "bool".to_string(),
        Type::callable(vec![Type::Any], Type::Bool),
    );
    env.bind(
        "list".to_string(),
        Type::callable(vec![], Type::list(Type::Unknown)),
    );
    env.bind(
        "dict".to_string(),
        Type::callable(vec![], Type::dict(Type::Unknown, Type::Unknown)),
    );
    env.bind(
        "set".to_string(),
        Type::callable(vec![], Type::Set(Box::new(Type::Unknown))),
    );
    env.bind(
        "print".to_string(),
        Type::Callable {
            params: vec![Param {
                name: "values".to_string(),
                ty: Type::Any,
                has_default: false,
                kind: ParamKind::VarPositional,
            }],
            ret: Box::new(Type::None),
        },
    );
    env.bind(
        "range".to_string(),
        Type::callable(vec![Type::Int], Type::list(Type::Int)),
    );
    env.bind(
        "enumerate".to_string(),
        Type::callable(
            vec![Type::list(Type::Unknown)],
            Type::list(Type::Tuple(vec![Type::Int, Type::Unknown])),
        ),
    );
    env.bind(
        "zip".to_string(),
        Type::callable(
            vec![Type::list(Type::Unknown), Type::list(Type::Unknown)],
            Type::list(Type::Tuple(vec![Type::Unknown, Type::Unknown])),
        ),
    );
    env.bind(
        "isinstance".to_string(),
        Type::callable(vec![Type::Any, Type::Any], Type::Bool),
    );
    env.bind(
        "hasattr".to_string(),
        Type::callable(vec![Type::Any, Type::Str], Type::Bool),
    );
    env.bind(
        "getattr".to_string(),
        Type::callable(vec![Type::Any, Type::Str], Type::Any),
    );
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/type_inference/builtins.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/type_inference/builtins.rs` captured during libs codegen standardization.
```
