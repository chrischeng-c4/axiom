---
id: libs-compass-src-type-inference-infer-tests-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/type_inference/infer_tests.rs`.
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

# Standardized libs/compass/src/type_inference/infer_tests.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/type_inference/infer_tests.rs` captured during libs codegen standardization.

No public Rust symbols detected by the source-unit capture pass.


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Tests for type inference

use std::collections::HashMap;

use super::*;
use crate::syntax::MultiParser;

fn infer_type(code: &str) -> Type {
    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(code, crate::syntax::Language::Python).unwrap();
    let mut inferencer = TypeInferencer::new(code);

    // Find first expression
    let root = parsed.tree.root_node();
    if let Some(stmt) = root.child(0) {
        if stmt.kind() == "expression_statement" {
            if let Some(expr) = stmt.child(0) {
                return inferencer.infer_expr(&expr);
            }
        }
    }
    Type::Unknown
}

#[test]
fn test_infer_literals() {
    assert_eq!(infer_type("42"), Type::Int);
    assert_eq!(infer_type("3.14"), Type::Float);
    assert_eq!(infer_type("\"hello\""), Type::Str);
    assert_eq!(infer_type("True"), Type::Bool);
    assert_eq!(infer_type("None"), Type::None);
}

#[test]
fn test_infer_binary_ops() {
    assert_eq!(infer_type("1 + 2"), Type::Int);
    assert_eq!(infer_type("1.0 + 2"), Type::Float);
    assert_eq!(infer_type("\"a\" + \"b\""), Type::Str);
    assert_eq!(infer_type("10 / 3"), Type::Float);
    assert_eq!(infer_type("10 // 3"), Type::Int);
}

#[test]
fn test_infer_containers() {
    assert_eq!(infer_type("[1, 2, 3]"), Type::list(Type::Int));
    assert_eq!(infer_type("{\"a\": 1}"), Type::dict(Type::Str, Type::Int));
    assert_eq!(
        infer_type("(1, \"a\")"),
        Type::Tuple(vec![Type::Int, Type::Str])
    );
}

#[test]
fn test_class_analysis() {
    let code = r#"
class Person:
    name: str
    age: int = 0

    def __init__(self, name: str, age: int) -> None:
        self.name = name
        self.age = age

    def greet(self) -> str:
        return "Hello, " + self.name
"#;
    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(code, crate::syntax::Language::Python).unwrap();
    let mut inferencer = TypeInferencer::new(code);

    // Find class definition
    let root = parsed.tree.root_node();
    if let Some(class_node) = root.child(0) {
        if class_node.kind() == "class_definition" {
            let class_info = inferencer.analyze_class(&class_node);

            assert_eq!(class_info.name, "Person");

            // Check class variables
            assert!(class_info.class_vars.contains_key("name"));
            assert!(class_info.class_vars.contains_key("age"));

            // Check methods
            assert!(class_info.methods.contains_key("__init__"));
            assert!(class_info.methods.contains_key("greet"));

            // Check __init__ sets instance attributes
            assert!(class_info.attributes.contains_key("name"));
            assert!(class_info.attributes.contains_key("age"));
        }
    }
}

#[test]
fn test_class_attribute_inference() {
    let code = r#"
class Point:
    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y

p = Point(1, 2)
p.x
"#;
    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(code, crate::syntax::Language::Python).unwrap();
    let mut inferencer = TypeInferencer::new(code);

    // Walk through the code to analyze class and assignments
    let root = parsed.tree.root_node();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "class_definition" {
            inferencer.analyze_class(&child);
        }
    }

    // Check that Point class was registered
    let class_info = inferencer.get_class("Point");
    assert!(class_info.is_some());
    let class_info = class_info.unwrap();
    assert!(class_info.attributes.contains_key("x"));
    assert!(class_info.attributes.contains_key("y"));
}

#[test]
fn test_typing_import_integration() {
    let code = "from typing import List, Optional";
    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(code, crate::syntax::Language::Python).unwrap();
    let mut inferencer = TypeInferencer::new(code);

    let root = parsed.tree.root_node();
    if let Some(import_node) = root.child(0) {
        inferencer.analyze_import(&import_node);
    }

    // Verify List and Optional are now in env
    assert!(inferencer.env().lookup("List").is_some());
    assert!(inferencer.env().lookup("Optional").is_some());
}

#[test]
fn test_collections_import_integration() {
    let code = "from collections import deque, Counter";
    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(code, crate::syntax::Language::Python).unwrap();
    let mut inferencer = TypeInferencer::new(code);

    let root = parsed.tree.root_node();
    if let Some(import_node) = root.child(0) {
        inferencer.analyze_import(&import_node);
    }

    assert!(inferencer.env().lookup("deque").is_some());
    assert!(inferencer.env().lookup("Counter").is_some());
}

#[test]
fn test_import_with_alias() {
    let code = "from typing import List as L, Dict as D";
    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(code, crate::syntax::Language::Python).unwrap();
    let mut inferencer = TypeInferencer::new(code);

    let root = parsed.tree.root_node();
    if let Some(import_node) = root.child(0) {
        inferencer.analyze_import(&import_node);
    }

    // Should be available under aliases
    assert!(inferencer.env().lookup("L").is_some());
    assert!(inferencer.env().lookup("D").is_some());
    // Original names should not be bound
    assert!(inferencer.env().lookup("List").is_none());
    assert!(inferencer.env().lookup("Dict").is_none());
}

#[test]
fn test_inheritance_attribute_lookup() {
    let code = r#"
class Animal:
    species: str = "unknown"

    def speak(self) -> str:
        return "sound"

class Dog(Animal):
    def bark(self) -> str:
        return "woof"
"#;
    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(code, crate::syntax::Language::Python).unwrap();
    let mut inferencer = TypeInferencer::new(code);

    // Analyze all classes
    let root = parsed.tree.root_node();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "class_definition" {
            inferencer.analyze_class(&child);
        }
    }

    // Dog should have its own method
    let bark = inferencer.get_attribute_recursive("Dog", "bark");
    assert!(bark.is_some());

    // Dog should inherit speak from Animal
    let speak = inferencer.get_attribute_recursive("Dog", "speak");
    assert!(speak.is_some());

    // Dog should inherit class var from Animal
    let species = inferencer.get_attribute_recursive("Dog", "species");
    assert!(species.is_some());

    // Animal should not have bark
    let animal_bark = inferencer.get_attribute_recursive("Animal", "bark");
    assert!(animal_bark.is_none());
}

#[test]
fn test_is_subclass() {
    let code = r#"
class Animal:
    pass

class Dog(Animal):
    pass

class Labrador(Dog):
    pass
"#;
    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(code, crate::syntax::Language::Python).unwrap();
    let mut inferencer = TypeInferencer::new(code);

    // Analyze all classes
    let root = parsed.tree.root_node();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "class_definition" {
            inferencer.analyze_class(&child);
        }
    }

    // Self is a subclass of self
    assert!(inferencer.is_subclass("Dog", "Dog"));

    // Dog is a subclass of Animal
    assert!(inferencer.is_subclass("Dog", "Animal"));

    // Labrador is a subclass of Dog and Animal (transitive)
    assert!(inferencer.is_subclass("Labrador", "Dog"));
    assert!(inferencer.is_subclass("Labrador", "Animal"));

    // Animal is NOT a subclass of Dog
    assert!(!inferencer.is_subclass("Animal", "Dog"));
}

#[test]
fn test_generic_call_inference() {
    use crate::type_inference::ty::{Param, ParamKind};

    // Test that calling a generic function infers type arguments
    // We'll manually create a generic function and test the inference

    // Create a generic identity function: def identity(x: T) -> T
    let t = Type::type_var(0, "T");
    let identity_fn = Type::Callable {
        params: vec![Param {
            name: "x".to_string(),
            ty: t.clone(),
            has_default: false,
            kind: ParamKind::Positional,
        }],
        ret: Box::new(t),
    };

    // Simulate unifying with Int argument
    let mut subs = HashMap::new();
    let param_ty = &identity_fn;
    if let Type::Callable { params, ret } = param_ty {
        // Unify parameter T with Int
        params[0].ty.unify(&Type::Int, &mut subs);

        // Apply substitution to return type
        let inferred_ret = ret.substitute(&subs);
        assert_eq!(inferred_ret, Type::Int);
    }
}

#[test]
fn test_generic_list_inference() {
    use crate::type_inference::ty::TypeVarId;

    // Test inferring element type from list[T] -> list[str]
    let t = Type::type_var(0, "T");
    let list_t = Type::list(t);

    let mut subs = HashMap::new();
    list_t.unify(&Type::list(Type::Str), &mut subs);

    // T should be inferred as Str
    assert_eq!(subs.get(&TypeVarId(0)), Some(&Type::Str));
}

// ============= Phase G Tests =============

#[test]
fn test_dataclass_analysis() {
    let code = r#"
from dataclasses import dataclass

@dataclass
class Point:
    x: int
    y: int
    label: str = "default"
"#;
    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(code, crate::syntax::Language::Python).unwrap();
    let mut inferencer = TypeInferencer::new(code);

    // Analyze all nodes (handling decorated_definition wrapper)
    let root = parsed.tree.root_node();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "decorated_definition" {
            // The decorated class is wrapped in decorated_definition
            let mut inner_cursor = child.walk();
            for inner in child.children(&mut inner_cursor) {
                if inner.kind() == "class_definition" {
                    inferencer.analyze_class(&inner);
                }
            }
        } else if child.kind() == "class_definition" {
            inferencer.analyze_class(&child);
        }
    }

    // Check that Point class was registered
    let class_info = inferencer.get_class("Point");
    assert!(class_info.is_some(), "Point class should be registered");
    let class_info = class_info.unwrap();

    // Check that __init__ was generated
    assert!(
        class_info.methods.contains_key("__init__"),
        "Dataclass should have __init__ method"
    );

    // Check that fields are registered as attributes
    assert!(
        class_info.attributes.contains_key("x"),
        "x attribute should exist"
    );
    assert!(
        class_info.attributes.contains_key("y"),
        "y attribute should exist"
    );
    assert!(
        class_info.attributes.contains_key("label"),
        "label attribute should exist"
    );
}

#[test]
fn test_namedtuple_analysis() {
    let code = r#"
from typing import NamedTuple

class Point(NamedTuple):
    x: int
    y: int
"#;
    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(code, crate::syntax::Language::Python).unwrap();
    let mut inferencer = TypeInferencer::new(code);

    // Analyze all nodes
    let root = parsed.tree.root_node();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "class_definition" {
            inferencer.analyze_class(&child);
        }
    }

    // Check that Point class was registered
    let class_info = inferencer.get_class("Point");
    assert!(class_info.is_some(), "Point class should be registered");
    let class_info = class_info.unwrap();

    // Check that fields are registered as attributes
    assert!(
        class_info.attributes.contains_key("x"),
        "x attribute should exist on NamedTuple"
    );
    assert!(
        class_info.attributes.contains_key("y"),
        "y attribute should exist on NamedTuple"
    );
}

#[test]
fn test_property_analysis() {
    let code = r#"
class Circle:
    def __init__(self, radius: float) -> None:
        self._radius = radius

    @property
    def radius(self) -> float:
        return self._radius

    @property
    def area(self) -> float:
        return 3.14159 * self._radius ** 2

    def normal_method(self) -> str:
        return "not a property"
"#;
    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(code, crate::syntax::Language::Python).unwrap();
    let mut inferencer = TypeInferencer::new(code);

    // Analyze all nodes
    let root = parsed.tree.root_node();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "class_definition" {
            inferencer.analyze_class(&child);
        }
    }

    // Check that Circle class was registered
    let class_info = inferencer.get_class("Circle");
    assert!(class_info.is_some(), "Circle class should be registered");
    let class_info = class_info.unwrap();

    // Properties should be registered as attributes, not methods
    assert!(
        class_info.attributes.contains_key("radius"),
        "radius should be an attribute (property)"
    );
    assert!(
        class_info.attributes.contains_key("area"),
        "area should be an attribute (property)"
    );

    // Normal method should still be a method
    assert!(
        class_info.methods.contains_key("normal_method"),
        "normal_method should be a method"
    );
    assert!(
        !class_info.attributes.contains_key("normal_method"),
        "normal_method should NOT be an attribute"
    );

    // Check that properties have correct types (from return annotation)
    let radius_type = class_info.attributes.get("radius");
    assert!(radius_type.is_some());
    assert_eq!(*radius_type.unwrap(), Type::Float);

    let area_type = class_info.attributes.get("area");
    assert!(area_type.is_some());
    assert_eq!(*area_type.unwrap(), Type::Float);
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/type_inference/infer_tests.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/type_inference/infer_tests.rs` captured during libs codegen standardization.
```
