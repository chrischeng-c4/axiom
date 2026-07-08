---
id: libs-compass-src-type-inference-annotation-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/type_inference/annotation.rs`.
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

# Standardized libs/compass/src/type_inference/annotation.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/type_inference/annotation.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `parse_type_annotation` | libs/compass/src/type_inference/annotation.rs | function | pub | 10 | pub fn parse_type_annotation(source: &str, node: &Node) -> Type { |
| `parse_simple_type` | libs/compass/src/type_inference/annotation.rs | function | pub | 35 | pub fn parse_simple_type(name: &str) -> Type { |
| `parse_generic_type` | libs/compass/src/type_inference/annotation.rs | function | pub | 58 | pub fn parse_generic_type(source: &str, node: &Node) -> Type { |
| `parse_type_args` | libs/compass/src/type_inference/annotation.rs | function | pub | 166 | pub fn parse_type_args(source: &str, node: &Node) -> Vec<Type> { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Type annotation parsing
//!
//! This module handles parsing Python type annotations from AST nodes.

use tree_sitter::Node;

use super::ty::{Param, ParamKind, Type};

/// Parse a type annotation from an AST node
pub fn parse_type_annotation(source: &str, node: &Node) -> Type {
    let text = node_text(source, node);

    match node.kind() {
        "identifier" | "type" => parse_simple_type(text),
        "subscript" => parse_generic_type(source, node),
        "binary_operator" => {
            // Union type: X | Y
            let left = node.child_by_field_name("left");
            let right = node.child_by_field_name("right");
            match (left, right) {
                (Some(l), Some(r)) => {
                    let left_ty = parse_type_annotation(source, &l);
                    let right_ty = parse_type_annotation(source, &r);
                    Type::union(vec![left_ty, right_ty])
                }
                _ => Type::Unknown,
            }
        }
        "none" => Type::None,
        _ => parse_simple_type(text),
    }
}

/// Parse a simple type name
pub fn parse_simple_type(name: &str) -> Type {
    match name {
        "int" => Type::Int,
        "float" => Type::Float,
        "str" => Type::Str,
        "bool" => Type::Bool,
        "bytes" => Type::Bytes,
        "None" => Type::None,
        "Any" => Type::Any,
        "object" => Type::Any,
        // PEP 673: Self type
        "Self" => Type::SelfType { class_name: None },
        // PEP 675: LiteralString
        "LiteralString" => Type::LiteralString,
        _ => Type::Instance {
            name: name.to_string(),
            module: None,
            type_args: vec![],
        },
    }
}

/// Parse a generic type like list[int], dict[str, int]
pub fn parse_generic_type(source: &str, node: &Node) -> Type {
    let base = node
        .child_by_field_name("value")
        .map(|n| node_text(source, &n))
        .unwrap_or("");

    let args = parse_type_args(source, node);

    match base {
        "list" | "List" => Type::list(args.first().cloned().unwrap_or(Type::Unknown)),
        "dict" | "Dict" => {
            let key = args.first().cloned().unwrap_or(Type::Unknown);
            let val = args.get(1).cloned().unwrap_or(Type::Unknown);
            Type::dict(key, val)
        }
        "set" | "Set" => Type::Set(Box::new(args.first().cloned().unwrap_or(Type::Unknown))),
        "tuple" | "Tuple" => Type::Tuple(args),
        "Optional" => {
            let inner = args.first().cloned().unwrap_or(Type::Unknown);
            Type::optional(inner)
        }
        "Union" => Type::union(args),
        "Callable" => {
            // Callable[[arg_types], return_type]
            if args.len() >= 2 {
                let ret = args.last().cloned().unwrap_or(Type::Unknown);
                let params: Vec<_> = args[..args.len() - 1]
                    .iter()
                    .enumerate()
                    .map(|(i, ty)| Param {
                        name: format!("_{}", i),
                        ty: ty.clone(),
                        has_default: false,
                        kind: ParamKind::Positional,
                    })
                    .collect();
                Type::Callable {
                    params,
                    ret: Box::new(ret),
                }
            } else {
                Type::Unknown
            }
        }
        "type" | "Type" => {
            let inner = args.first().cloned().unwrap_or(Type::Unknown);
            if let Type::Instance { name, module, .. } = inner {
                Type::ClassType { name, module }
            } else {
                Type::Unknown
            }
        }
        // PEP 591: Final
        "Final" => {
            let inner = args.first().cloned().unwrap_or(Type::Unknown);
            Type::Final(Box::new(inner))
        }
        // PEP 593: Annotated
        "Annotated" => {
            if args.is_empty() {
                Type::Unknown
            } else {
                let inner = args[0].clone();
                // Metadata is simplified - store as strings
                let metadata: Vec<String> = args[1..].iter().map(|t| t.to_string()).collect();
                Type::Annotated {
                    inner: Box::new(inner),
                    metadata,
                }
            }
        }
        // PEP 612: Concatenate
        "Concatenate" => {
            if args.is_empty() {
                Type::Unknown
            } else {
                let param_spec = args.last().cloned().unwrap_or(Type::Unknown);
                let params = args[..args.len().saturating_sub(1)].to_vec();
                Type::Concatenate {
                    params,
                    param_spec: Box::new(param_spec),
                }
            }
        }
        // PEP 646: Unpack
        "Unpack" => {
            let inner = args.first().cloned().unwrap_or(Type::Unknown);
            Type::Unpack(Box::new(inner))
        }
        // PEP 647: TypeGuard
        "TypeGuard" => {
            let inner = args.first().cloned().unwrap_or(Type::Unknown);
            Type::TypeGuard(Box::new(inner))
        }
        // PEP 742: TypeIs
        "TypeIs" => {
            let inner = args.first().cloned().unwrap_or(Type::Unknown);
            Type::TypeIs(Box::new(inner))
        }
        _ => Type::Instance {
            name: base.to_string(),
            module: None,
            type_args: args,
        },
    }
}

/// Parse type arguments from a subscript node
pub fn parse_type_args(source: &str, node: &Node) -> Vec<Type> {
    let mut args = Vec::new();

    if let Some(subscript) = node.child_by_field_name("subscript") {
        match subscript.kind() {
            "tuple" | "expression_list" => {
                let mut cursor = subscript.walk();
                for child in subscript.children(&mut cursor) {
                    if child.kind() != "," && child.kind() != "(" && child.kind() != ")" {
                        args.push(parse_type_annotation(source, &child));
                    }
                }
            }
            _ => {
                args.push(parse_type_annotation(source, &subscript));
            }
        }
    }

    args
}

/// Get text of a node
fn node_text<'a>(source: &'a str, node: &Node) -> &'a str {
    node.utf8_text(source.as_bytes()).unwrap_or("")
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/type_inference/annotation.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/type_inference/annotation.rs` captured during libs codegen standardization.
```
