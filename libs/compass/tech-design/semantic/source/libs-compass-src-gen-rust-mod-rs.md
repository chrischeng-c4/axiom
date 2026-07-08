---
id: libs-compass-src-gen-rust-mod-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/gen/rust/mod.rs`.
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

# Standardized libs/compass/src/gen/rust/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/gen/rust/mod.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `axum` | libs/compass/src/gen/rust/mod.rs | module | pub | 9 | pub mod axum; |
| `reqwest` | libs/compass/src/gen/rust/mod.rs | module | pub | 10 | pub mod reqwest; |
| `serde` | libs/compass/src/gen/rust/mod.rs | module | pub | 11 | pub mod serde; |
| `sqlx` | libs/compass/src/gen/rust/mod.rs | module | pub | 12 | pub mod sqlx; |
| `AxumGenerator` | libs/compass/src/gen/rust/mod.rs | re-export | pub | 14 | pub use self::axum::AxumGenerator; |
| `ReqwestGenerator` | libs/compass/src/gen/rust/mod.rs | re-export | pub | 15 | pub use self::reqwest::ReqwestGenerator; |
| `SerdeGenerator` | libs/compass/src/gen/rust/mod.rs | re-export | pub | 16 | pub use self::serde::SerdeGenerator; |
| `SqlxGenerator` | libs/compass/src/gen/rust/mod.rs | re-export | pub | 17 | pub use self::sqlx::SqlxGenerator; |
| `type_to_rust` | libs/compass/src/gen/rust/mod.rs | function | pub | 23 | pub fn type_to_rust(ty: &Type) -> String { |
| `format_to_rust_type` | libs/compass/src/gen/rust/mod.rs | function | pub | 79 | pub fn format_to_rust_type(format: &StringFormat) -> &'static str { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Rust code generators
//!
//! Generators for:
//! - serde (structs with serialization)
//! - sqlx (database models)
//! - axum (route handlers)
//! - reqwest (HTTP client)

pub mod axum;
pub mod reqwest;
pub mod serde;
pub mod sqlx;

pub use self::axum::AxumGenerator;
pub use self::reqwest::ReqwestGenerator;
pub use self::serde::SerdeGenerator;
pub use self::sqlx::SqlxGenerator;

use crate::spec::ir::StringFormat;
use crate::type_inference::Type;

/// Convert Type IR to Rust type string
pub fn type_to_rust(ty: &Type) -> String {
    match ty {
        Type::Never => "!".to_string(),
        Type::None => "()".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Int => "i64".to_string(),
        Type::Float => "f64".to_string(),
        Type::Str => "String".to_string(),
        Type::Bytes => "Vec<u8>".to_string(),
        Type::List(inner) => format!("Vec<{}>", type_to_rust(inner)),
        Type::Dict(key, value) => {
            format!("HashMap<{}, {}>", type_to_rust(key), type_to_rust(value))
        }
        Type::Set(inner) => format!("HashSet<{}>", type_to_rust(inner)),
        Type::Tuple(items) => {
            let items_str: Vec<String> = items.iter().map(type_to_rust).collect();
            format!("({})", items_str.join(", "))
        }
        Type::Optional(inner) => format!("Option<{}>", type_to_rust(inner)),
        Type::Union(types) => {
            // Rust doesn't have built-in union types, use enum or first type
            if types.len() == 2 && types.contains(&Type::None) {
                let non_none = types.iter().find(|t| **t != Type::None).unwrap();
                format!("Option<{}>", type_to_rust(non_none))
            } else {
                // For complex unions, we'd need to generate an enum
                "serde_json::Value".to_string()
            }
        }
        Type::Instance {
            name, type_args, ..
        } => {
            if type_args.is_empty() {
                name.clone()
            } else {
                let args_str: Vec<String> = type_args.iter().map(type_to_rust).collect();
                format!("{}<{}>", name, args_str.join(", "))
            }
        }
        Type::Literal(lit) => {
            // Rust doesn't have literal types like TypeScript
            use crate::type_inference::LiteralValue;
            match lit {
                LiteralValue::Int(_) => "i64".to_string(),
                LiteralValue::Float(_) => "f64".to_string(),
                LiteralValue::Str(_) => "String".to_string(),
                LiteralValue::Bool(_) => "bool".to_string(),
                LiteralValue::None => "()".to_string(),
            }
        }
        Type::Any | Type::Unknown => "serde_json::Value".to_string(),
        _ => "serde_json::Value".to_string(),
    }
}

/// Convert StringFormat to Rust type
pub fn format_to_rust_type(format: &StringFormat) -> &'static str {
    match format {
        StringFormat::Uuid => "uuid::Uuid",
        StringFormat::DateTime => "chrono::DateTime<chrono::Utc>",
        StringFormat::Date => "chrono::NaiveDate",
        StringFormat::Time => "chrono::NaiveTime",
        StringFormat::Duration => "std::time::Duration",
        StringFormat::Uri | StringFormat::Url => "url::Url",
        _ => "String",
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/gen/rust/mod.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/gen/rust/mod.rs` captured during libs codegen standardization.
```
