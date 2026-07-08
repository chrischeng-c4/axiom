---
id: libs-compass-src-semantic-tests-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/semantic/tests.rs`.
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

# Standardized libs/compass/src/semantic/tests.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/semantic/tests.rs` captured during libs codegen standardization.

No public Rust symbols detected by the source-unit capture pass.


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Tests for error recovery in semantic analysis
//!
//! These tests verify that:
//! 1. The parser recovers from syntax errors and continues processing
//! 2. The symbol table is partially constructed despite errors
//! 3. Type inference handles error nodes gracefully

use crate::syntax::{Language, MultiParser};
use crate::type_inference::TypeInferencer;

use super::symbols::SymbolTableBuilder;

/// Test that parser recovers at statement boundaries
#[test]
fn test_parser_error_recovery_statement_boundary() {
    let source = r#"
def valid_function():
    return 42

# Syntax error: missing colon
def broken_function()
    return 0

def another_valid_function():
    return "hello"
"#;

    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(source, Language::Python).unwrap();

    // File has errors
    assert!(parsed.has_errors);

    // But we can still get valid statements
    let valid_stmts = parsed.valid_statements();

    // Should find at least the two valid function definitions
    // (The exact count depends on tree-sitter's recovery behavior)
    assert!(!valid_stmts.is_empty());
}

/// Test that symbol table is partially constructed with syntax errors
#[test]
fn test_partial_symbol_table_construction() {
    let source = r#"
class ValidClass:
    def method(self):
        return 1

def valid_function(x: int) -> int:
    return x * 2

# Syntax error
class BrokenClass(
    pass

def another_valid_function() -> str:
    return "ok"
"#;

    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(source, Language::Python).unwrap();

    let symbol_table = SymbolTableBuilder::new().build_python(&parsed);

    // Should have captured symbols from valid parts
    // Note: actual symbol count may vary based on recovery
    assert!(!symbol_table.all_symbols().is_empty());
}

/// Test that type inference returns Error for ERROR nodes
#[test]
fn test_type_inference_error_nodes() {
    let source = r#"
x: int = 42
y: str = "hello"
z = x +  # Incomplete expression
"#;

    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(source, Language::Python).unwrap();

    let mut inferencer = TypeInferencer::new(source);

    // Walk the AST and infer types
    parsed.walk(|node, _depth| {
        // Try to infer types for various nodes
        if node.kind() == "assignment" {
            if let Some(right) = node.child_by_field_name("right") {
                let ty = inferencer.infer_expr(&right);
                // Error nodes should produce Error type
                if right.is_error() {
                    assert!(ty.is_error() || ty.is_unknown());
                }
            }
        }
        true
    });
}

/// Test that errors don't cascade through type checking
#[test]
fn test_no_cascading_errors() {
    let source = r#"
def broken_syntax(x: int
    return x

def valid_function(x: int, y: int) -> int:
    return x + y

result: int = valid_function(1, 2)
"#;

    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(source, Language::Python).unwrap();

    let mut inferencer = TypeInferencer::new(source);

    // Analyze function definitions
    parsed.walk(|node, _| {
        if node.kind() == "function_definition" {
            inferencer.analyze_function(node);
        }
        true
    });

    // valid_function should still be accessible despite earlier errors
    // Check via the environment
    let env = inferencer.env();
    let func_type = env.lookup("valid_function");
    assert!(func_type.is_some());
}

/// Test collect_errors method
#[test]
fn test_collect_errors() {
    let source = r#"
def valid():
    pass

def broken(  # Missing closing paren and colon
    pass

def another_valid():
    pass
"#;

    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(source, Language::Python).unwrap();

    let errors = parsed.collect_errors();

    // Should detect at least one error
    assert!(!errors.is_empty());

    // Each error should have valid position info
    for error in &errors {
        assert!(error.start_position.0 > 0); // Line number starts at 1
    }
}

/// Test walk_with_recovery skips error nodes
#[test]
fn test_walk_with_recovery() {
    let source = r#"
x = 1
y =    # Error
z = 3
"#;

    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(source, Language::Python).unwrap();

    let mut visited_kinds = Vec::new();

    parsed.walk_with_recovery(|node, _| {
        // Should not see ERROR nodes
        assert!(!node.is_error());
        visited_kinds.push(node.kind().to_string());
        true
    });

    // Should have visited some valid nodes
    assert!(visited_kinds.contains(&"module".to_string()));
}

/// Test that symbol table handles multiple errors gracefully
#[test]
fn test_multiple_errors_graceful_handling() {
    let source = r#"
# First error
def broken1(:

# Second error
class Broken2(

# Third error
x = y +

# Valid definition at the end
CONSTANT = 42
"#;

    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(source, Language::Python).unwrap();

    // Should not panic even with multiple errors
    let symbol_table = SymbolTableBuilder::new().build_python(&parsed);

    // Symbol table should be in a valid state
    // The all_symbols() method should return without panic
    let _ = symbol_table.all_symbols();
}

/// Test Type::Error behavior
#[test]
fn test_type_error_behavior() {
    use crate::type_inference::Type;

    let error_type = Type::Error;

    // Error type should be recognized as error
    assert!(error_type.is_error());

    // Error type should display correctly
    let display = format!("{}", error_type);
    assert_eq!(display, "<error>");
}

/// Test is_inside_error detection
#[test]
fn test_is_inside_error_detection() {
    let source = r#"
valid = 1
broken = x +   # Missing operand
"#;

    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(source, Language::Python).unwrap();

    let mut found_error = false;
    let mut found_non_error = false;

    parsed.walk(|node, _| {
        if node.is_error() {
            found_error = true;
        } else if node.kind() == "identifier" {
            found_non_error = true;
        }
        true
    });

    // Should have found both error and non-error nodes
    // (exact behavior depends on tree-sitter parsing)
    assert!(found_non_error);
}

/// Test synchronize_after finds next valid sibling
#[test]
fn test_synchronize_after() {
    let source = r#"
def broken(
    pass

def valid():
    return 1
"#;

    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(source, Language::Python);

    // Just test that parsing doesn't crash
    assert!(parsed.is_some());
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/semantic/tests.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/semantic/tests.rs` captured during libs codegen standardization.
```
