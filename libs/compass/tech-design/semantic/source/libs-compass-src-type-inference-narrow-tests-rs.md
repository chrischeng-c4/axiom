---
id: libs-compass-src-type-inference-narrow-tests-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/type_inference/narrow_tests.rs`.
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

# Standardized libs/compass/src/type_inference/narrow_tests.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/type_inference/narrow_tests.rs` captured during libs codegen standardization.

No public Rust symbols detected by the source-unit capture pass.


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Tests for type narrowing

use super::*;

#[test]
fn test_is_none_narrowing() {
    let mut narrower = TypeNarrower::new();
    narrower.push_scope();

    let original_types: HashMap<String, Type> = [("x".to_string(), Type::optional(Type::Str))]
        .into_iter()
        .collect();

    let condition = NarrowingCondition::IsNone {
        var_name: "x".to_string(),
    };

    narrower.apply_condition(&condition, &original_types);

    assert_eq!(narrower.get_narrowed("x"), Some(&Type::None));
}

#[test]
fn test_is_not_none_narrowing() {
    let mut narrower = TypeNarrower::new();
    narrower.push_scope();

    let original_types: HashMap<String, Type> = [("x".to_string(), Type::optional(Type::Str))]
        .into_iter()
        .collect();

    let condition = NarrowingCondition::IsNotNone {
        var_name: "x".to_string(),
    };

    narrower.apply_condition(&condition, &original_types);

    assert_eq!(narrower.get_narrowed("x"), Some(&Type::Str));
}

#[test]
fn test_isinstance_narrowing() {
    let mut narrower = TypeNarrower::new();
    narrower.push_scope();

    let original_types: HashMap<String, Type> =
        [("x".to_string(), Type::union(vec![Type::Int, Type::Str]))]
            .into_iter()
            .collect();

    let condition = NarrowingCondition::IsInstance {
        var_name: "x".to_string(),
        types: vec![Type::Int],
    };

    narrower.apply_condition(&condition, &original_types);

    assert_eq!(narrower.get_narrowed("x"), Some(&Type::Int));
}

#[test]
fn test_scope_stack() {
    let mut narrower = TypeNarrower::new();

    narrower.push_scope();
    narrower.narrow_var("x", Type::Int);
    assert_eq!(narrower.get_narrowed("x"), Some(&Type::Int));

    narrower.push_scope();
    narrower.narrow_var("x", Type::Str);
    assert_eq!(narrower.get_narrowed("x"), Some(&Type::Str));

    narrower.pop_scope();
    assert_eq!(narrower.get_narrowed("x"), Some(&Type::Int));

    narrower.pop_scope();
    assert_eq!(narrower.get_narrowed("x"), None);
}

#[test]
fn test_callable_narrowing() {
    let mut narrower = TypeNarrower::new();
    narrower.push_scope();

    let original_types: HashMap<String, Type> =
        [("f".to_string(), Type::Any)].into_iter().collect();

    let condition = NarrowingCondition::IsCallable {
        var_name: "f".to_string(),
    };

    narrower.apply_condition(&condition, &original_types);

    let narrowed = narrower.get_narrowed("f").unwrap();
    assert!(matches!(narrowed, Type::Callable { .. }));
}

#[test]
fn test_hasattr_narrowing() {
    let mut narrower = TypeNarrower::new();
    narrower.push_scope();

    let original_types: HashMap<String, Type> =
        [("obj".to_string(), Type::Any)].into_iter().collect();

    let condition = NarrowingCondition::HasAttr {
        var_name: "obj".to_string(),
        attr_name: "foo".to_string(),
    };

    narrower.apply_condition(&condition, &original_types);

    // HasAttr creates a marker type
    let narrowed = narrower.get_narrowed("obj").unwrap();
    assert!(matches!(narrowed, Type::Instance { name, .. } if name.contains("HasAttr")));
}

#[test]
fn test_callable_negation() {
    // Test that not callable(x) produces NotCallable
    let cond = NarrowingCondition::IsCallable {
        var_name: "f".to_string(),
    };
    let negated = negate_condition(&cond);
    assert!(matches!(&negated, NarrowingCondition::NotCallable { var_name } if var_name == "f"));

    // And back
    let double_neg = negate_condition(&negated);
    assert!(matches!(&double_neg, NarrowingCondition::IsCallable { var_name } if var_name == "f"));
}

// === Phase B Tests ===

#[test]
fn test_typeguard_narrowing() {
    let mut narrower = TypeNarrower::new();
    narrower.push_scope();

    let original_types: HashMap<String, Type> =
        [("x".to_string(), Type::Any)].into_iter().collect();

    // TypeGuard[int] narrows to int in positive branch
    let condition = NarrowingCondition::TypeGuard {
        var_name: "x".to_string(),
        narrowed_type: Type::Int,
    };

    narrower.apply_condition(&condition, &original_types);
    assert_eq!(narrower.get_narrowed("x"), Some(&Type::Int));
}

#[test]
fn test_typeguard_negation_returns_unknown() {
    // TypeGuard doesn't narrow in negative branch (PEP 647)
    let cond = NarrowingCondition::TypeGuard {
        var_name: "x".to_string(),
        narrowed_type: Type::Int,
    };
    let negated = negate_condition(&cond);
    assert!(matches!(negated, NarrowingCondition::Unknown));
}

#[test]
fn test_typeis_narrowing() {
    let mut narrower = TypeNarrower::new();
    narrower.push_scope();

    let original_types: HashMap<String, Type> =
        [("x".to_string(), Type::Any)].into_iter().collect();

    // TypeIs[str] narrows to str
    let condition = NarrowingCondition::TypeIs {
        var_name: "x".to_string(),
        narrowed_type: Type::Str,
    };

    narrower.apply_condition(&condition, &original_types);
    assert_eq!(narrower.get_narrowed("x"), Some(&Type::Str));
}

#[test]
fn test_typeis_negation_wraps_in_not() {
    // TypeIs supports narrowing in both branches (PEP 742)
    let cond = NarrowingCondition::TypeIs {
        var_name: "x".to_string(),
        narrowed_type: Type::Int,
    };
    let negated = negate_condition(&cond);
    // Negation creates a Not wrapper for TypeIs
    assert!(matches!(&negated, NarrowingCondition::Not(inner)
        if matches!(&**inner, NarrowingCondition::TypeIs { var_name, narrowed_type }
            if var_name == "x" && *narrowed_type == Type::Int)));
}

#[test]
fn test_typecheck_narrowing() {
    let mut narrower = TypeNarrower::new();
    narrower.push_scope();

    let original_types: HashMap<String, Type> =
        [("x".to_string(), Type::Any)].into_iter().collect();

    // type(x) is int narrows to int
    let condition = NarrowingCondition::TypeCheck {
        var_name: "x".to_string(),
        target_type: Type::Int,
    };

    narrower.apply_condition(&condition, &original_types);
    assert_eq!(narrower.get_narrowed("x"), Some(&Type::Int));
}

#[test]
fn test_typecheck_negation() {
    let cond = NarrowingCondition::TypeCheck {
        var_name: "x".to_string(),
        target_type: Type::Int,
    };
    let negated = negate_condition(&cond);
    assert!(
        matches!(&negated, NarrowingCondition::NotTypeCheck { var_name, target_type }
        if var_name == "x" && *target_type == Type::Int)
    );

    // And back
    let double_neg = negate_condition(&negated);
    assert!(
        matches!(&double_neg, NarrowingCondition::TypeCheck { var_name, target_type }
        if var_name == "x" && *target_type == Type::Int)
    );
}

#[test]
fn test_isinstance_tuple_narrowing() {
    let mut narrower = TypeNarrower::new();
    narrower.push_scope();

    let original_types: HashMap<String, Type> =
        [("x".to_string(), Type::Any)].into_iter().collect();

    // isinstance(x, (int, str)) narrows to int | str
    let condition = NarrowingCondition::IsInstance {
        var_name: "x".to_string(),
        types: vec![Type::Int, Type::Str],
    };

    narrower.apply_condition(&condition, &original_types);
    let narrowed = narrower.get_narrowed("x").unwrap();

    // Should be a union of int and str
    match narrowed {
        Type::Union(types) => {
            assert_eq!(types.len(), 2);
            assert!(types.contains(&Type::Int));
            assert!(types.contains(&Type::Str));
        }
        _ => panic!("Expected Union type, got {:?}", narrowed),
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/type_inference/narrow_tests.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/type_inference/narrow_tests.rs` captured during libs codegen standardization.
```
