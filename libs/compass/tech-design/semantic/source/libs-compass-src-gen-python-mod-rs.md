---
id: libs-compass-src-gen-python-mod-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/gen/python/mod.rs`.
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

# Standardized libs/compass/src/gen/python/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/gen/python/mod.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `meteor` | libs/compass/src/gen/python/mod.rs | module | pub | 11 | pub mod meteor; |
| `nebula` | libs/compass/src/gen/python/mod.rs | module | pub | 12 | pub mod nebula; |
| `photon` | libs/compass/src/gen/python/mod.rs | module | pub | 13 | pub mod photon; |
| `quasar` | libs/compass/src/gen/python/mod.rs | module | pub | 14 | pub mod quasar; |
| `rust_scanner` | libs/compass/src/gen/python/mod.rs | module | pub | 15 | pub mod rust_scanner; |
| `shield` | libs/compass/src/gen/python/mod.rs | module | pub | 16 | pub mod shield; |
| `test_extractor` | libs/compass/src/gen/python/mod.rs | module | pub | 17 | pub mod test_extractor; |
| `titan` | libs/compass/src/gen/python/mod.rs | module | pub | 18 | pub mod titan; |
| `SwarmGenerator` | libs/compass/src/gen/python/mod.rs | re-export | pub | 20 | pub use meteor::SwarmGenerator; |
| `NebulaGenerator` | libs/compass/src/gen/python/mod.rs | re-export | pub | 21 | pub use nebula::NebulaGenerator; |
| `PhotonGenerator` | libs/compass/src/gen/python/mod.rs | re-export | pub | 22 | pub use photon::PhotonGenerator; |
| `QuasarGenerator` | libs/compass/src/gen/python/mod.rs | re-export | pub | 23 | pub use quasar::QuasarGenerator; |
| `ShieldGenerator` | libs/compass/src/gen/python/mod.rs | re-export | pub | 28 | pub use shield::ShieldGenerator; |
| `RustTest` | libs/compass/src/gen/python/mod.rs | re-export | pub | 29 | pub use test_extractor::{RustTest, TestExtractor, TestExtractorConfig}; |
| `TestExtractor` | libs/compass/src/gen/python/mod.rs | re-export | pub | 29 | pub use test_extractor::{RustTest, TestExtractor, TestExtractorConfig}; |
| `TestExtractorConfig` | libs/compass/src/gen/python/mod.rs | re-export | pub | 29 | pub use test_extractor::{RustTest, TestExtractor, TestExtractorConfig}; |
| `TitanGenerator` | libs/compass/src/gen/python/mod.rs | re-export | pub | 30 | pub use titan::TitanGenerator; |
| `type_to_python` | libs/compass/src/gen/python/mod.rs | function | pub | 36 | pub fn type_to_python(ty: &Type) -> String { |
| `format_to_python_type` | libs/compass/src/gen/python/mod.rs | function | pub | 99 | pub fn format_to_python_type(format: &StringFormat) -> &'static str { |
| `get_type_imports` | libs/compass/src/gen/python/mod.rs | function | pub | 112 | pub fn get_type_imports(ty: &Type) -> Vec<&'static str> { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Python code generators for cclab ecosystem
//!
//! Generators for:
//! - cclab.shield (data validation models)
//! - cclab.titan (PostgreSQL ORM)
//! - cclab.nebula (MongoDB documents)
//! - cclab.photon (HTTP client)
//! - cclab.quasar (API routes)
//! - cclab.meteor (event handlers)
//!
pub mod meteor;
pub mod nebula;
pub mod photon;
pub mod quasar;
pub mod rust_scanner;
pub mod shield;
pub mod test_extractor;
pub mod titan;

pub use meteor::SwarmGenerator;
pub use nebula::NebulaGenerator;
pub use photon::PhotonGenerator;
pub use quasar::QuasarGenerator;
pub use rust_scanner::{
    RustEnum, RustEnumVariant, RustExports, RustField, RustFunction, RustMethod, RustParam,
    RustScanner, RustStruct, StructKind,
};
pub use shield::ShieldGenerator;
pub use test_extractor::{RustTest, TestExtractor, TestExtractorConfig};
pub use titan::TitanGenerator;

use crate::spec::ir::StringFormat;
use crate::type_inference::Type;

/// Convert Type IR to Python type annotation string
pub fn type_to_python(ty: &Type) -> String {
    match ty {
        Type::Never => "Never".to_string(),
        Type::None => "None".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Int => "int".to_string(),
        Type::Float => "float".to_string(),
        Type::Str => "str".to_string(),
        Type::Bytes => "bytes".to_string(),
        Type::List(inner) => format!("list[{}]", type_to_python(inner)),
        Type::Dict(key, value) => {
            format!("dict[{}, {}]", type_to_python(key), type_to_python(value))
        }
        Type::Set(inner) => format!("set[{}]", type_to_python(inner)),
        Type::Tuple(items) => {
            let items_str: Vec<String> = items.iter().map(type_to_python).collect();
            format!("tuple[{}]", items_str.join(", "))
        }
        Type::Optional(inner) => format!("Optional[{}]", type_to_python(inner)),
        Type::Union(types) => {
            if types.len() == 2 && types.contains(&Type::None) {
                let non_none = types.iter().find(|t| **t != Type::None).unwrap();
                format!("Optional[{}]", type_to_python(non_none))
            } else {
                let types_str: Vec<String> = types.iter().map(type_to_python).collect();
                format!("Union[{}]", types_str.join(", "))
            }
        }
        Type::Instance {
            name, type_args, ..
        } => {
            if type_args.is_empty() {
                name.clone()
            } else {
                let args_str: Vec<String> = type_args.iter().map(type_to_python).collect();
                format!("{}[{}]", name, args_str.join(", "))
            }
        }
        Type::Callable { params, ret } => {
            let params_str: Vec<String> = params.iter().map(|p| type_to_python(&p.ty)).collect();
            format!(
                "Callable[[{}], {}]",
                params_str.join(", "),
                type_to_python(ret)
            )
        }
        Type::Literal(lit) => {
            use crate::type_inference::LiteralValue;
            match lit {
                LiteralValue::Int(i) => format!("Literal[{}]", i),
                LiteralValue::Float(f) => format!("Literal[{}]", f),
                LiteralValue::Str(s) => format!("Literal[\"{}\"]", s),
                LiteralValue::Bool(b) => format!("Literal[{}]", if *b { "True" } else { "False" }),
                LiteralValue::None => "Literal[None]".to_string(),
            }
        }
        Type::Any => "Any".to_string(),
        Type::Unknown => "Any".to_string(),
        _ => "Any".to_string(),
    }
}

/// Convert StringFormat to Python type hint
pub fn format_to_python_type(format: &StringFormat) -> &'static str {
    match format {
        StringFormat::Email => "EmailStr",
        StringFormat::Uri | StringFormat::Url => "HttpUrl",
        StringFormat::Uuid => "UUID",
        StringFormat::DateTime => "datetime",
        StringFormat::Date => "date",
        StringFormat::Time => "time",
        _ => "str",
    }
}

/// Get imports needed for a type
pub fn get_type_imports(ty: &Type) -> Vec<&'static str> {
    let mut imports = Vec::new();

    match ty {
        Type::Optional(_) => imports.push("Optional"),
        Type::Union(_) => imports.push("Union"),
        Type::Callable { .. } => imports.push("Callable"),
        Type::Literal(_) => imports.push("Literal"),
        Type::Any | Type::Unknown => imports.push("Any"),
        _ => {}
    }

    // Recurse into container types
    match ty {
        Type::List(inner) | Type::Set(inner) | Type::Optional(inner) => {
            imports.extend(get_type_imports(inner));
        }
        Type::Dict(k, v) => {
            imports.extend(get_type_imports(k));
            imports.extend(get_type_imports(v));
        }
        Type::Union(types) | Type::Tuple(types) => {
            for t in types {
                imports.extend(get_type_imports(t));
            }
        }
        _ => {}
    }

    imports
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/gen/python/mod.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/gen/python/mod.rs` captured during libs codegen standardization.
```
