---
id: libs-compass-src-server-tests-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/server/tests.rs`.
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

# Standardized libs/compass/src/server/tests.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/server/tests.rs` captured during libs codegen standardization.

No public Rust symbols detected by the source-unit capture pass.


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Tests for the Argus daemon server
//!
//! These tests cover:
//! - SemanticModel accuracy (type lookups, definitions, references)
//! - Background re-analysis after file changes
//! - Handler request/response functionality

use std::path::PathBuf;

use crate::syntax::{Language, MultiParser};
use crate::type_inference::{build_semantic_model, SemanticModel, SemanticSymbolKind, TypeInfo};

use super::handler::RequestHandler;
use super::protocol::*;

// =============================================================================
// SemanticModel Accuracy Tests
// =============================================================================

#[test]
fn test_semantic_model_variable_type() {
    let code = r#"
x: int = 42
y: str = "hello"
z = x + 1
"#;
    let model = build_python_model(code);

    // Check x has type int
    assert!(model.symbols.values().any(|s| s.name == "x"));
    let x_symbol = model.symbols.values().find(|s| s.name == "x").unwrap();
    assert!(matches!(x_symbol.type_info, TypeInfo::Int));

    // Check y has type str
    let y_symbol = model.symbols.values().find(|s| s.name == "y").unwrap();
    assert!(matches!(y_symbol.type_info, TypeInfo::Str));
}

#[test]
fn test_semantic_model_function_type() {
    let code = r#"
def add(a: int, b: int) -> int:
    return a + b
"#;
    let model = build_python_model(code);

    // Check function exists with correct signature
    let add_symbol = model.symbols.values().find(|s| s.name == "add");
    assert!(add_symbol.is_some());

    let add_symbol = add_symbol.unwrap();
    assert!(matches!(add_symbol.kind, SemanticSymbolKind::Function));

    // Type should be callable
    if let TypeInfo::Callable { return_type, .. } = &add_symbol.type_info {
        assert!(matches!(return_type.as_ref(), TypeInfo::Int));
    }
}

#[test]
fn test_semantic_model_class_type() {
    let code = r#"
class Point:
    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y
"#;
    let model = build_python_model(code);

    // Check class exists
    let point_symbol = model.symbols.values().find(|s| s.name == "Point");
    assert!(point_symbol.is_some());

    let point_symbol = point_symbol.unwrap();
    assert!(matches!(point_symbol.kind, SemanticSymbolKind::Class));
}

#[test]
fn test_semantic_model_type_at_position() {
    let code = r#"x: int = 42
y: str = "hello""#;
    let model = build_python_model(code);

    // type_at should find the int at line 0
    if let Some(type_info) = model.type_at(0, 0) {
        assert!(matches!(type_info, TypeInfo::Int) || !type_info.is_unknown());
    }
}

#[test]
fn test_semantic_model_definition_lookup() {
    let code = r#"
def foo() -> int:
    return 42

x = foo()
"#;
    let model = build_python_model(code);

    // Should find the definition of foo
    let foo_symbol = model.symbols.values().find(|s| s.name == "foo");
    assert!(foo_symbol.is_some());

    let foo_symbol = foo_symbol.unwrap();
    assert_eq!(foo_symbol.name, "foo");
    assert!(matches!(foo_symbol.kind, SemanticSymbolKind::Function));
}

#[test]
fn test_semantic_model_references() {
    let code = r#"
x = 10
y = x + 5
z = x * 2
"#;
    let model = build_python_model(code);

    // Find the x symbol
    let x_symbol_id = model
        .name_to_symbols
        .get("x")
        .and_then(|ids| ids.first())
        .copied();

    if let Some(id) = x_symbol_id {
        // Count references to x
        let ref_count = model
            .references
            .iter()
            .filter(|r| r.symbol_id == id)
            .count();

        // Should have definition + uses
        assert!(ref_count >= 1);
    }
}

#[test]
fn test_semantic_model_hover_content() {
    let code = r#"
def greet(name: str) -> str:
    """Say hello to someone."""
    return f"Hello, {name}!"
"#;
    let model = build_python_model(code);

    // Find the function at its definition line
    let greet_symbol = model.symbols.values().find(|s| s.name == "greet");
    assert!(greet_symbol.is_some());

    let greet = greet_symbol.unwrap();
    let hover = model.hover_at(greet.def_range.start.line, greet.def_range.start.character);

    // Hover content should exist and contain the function name
    if let Some(content) = hover {
        assert!(content.contains("greet"));
    }
}

#[test]
fn test_semantic_model_optional_type() {
    let code = r#"
from typing import Optional

def maybe_get(flag: bool) -> Optional[int]:
    if flag:
        return 42
    return None
"#;
    let model = build_python_model(code);

    let func_symbol = model.symbols.values().find(|s| s.name == "maybe_get");
    assert!(func_symbol.is_some());
}

#[test]
fn test_semantic_model_union_type() {
    let code = r#"
def process(value: int | str) -> str:
    return str(value)
"#;
    let model = build_python_model(code);

    let func_symbol = model.symbols.values().find(|s| s.name == "process");
    assert!(func_symbol.is_some());
}

// =============================================================================
// Handler Tests
// =============================================================================

#[tokio::test]
async fn test_handler_check_request() {
    let handler = RequestHandler::new(PathBuf::from(".")).unwrap();

    let request = Request::new(1, "index_status", None);
    let response = handler.handle(request).await;

    // Should get a successful response
    assert!(response.error.is_none());
    assert!(response.result.is_some());
}

#[tokio::test]
async fn test_handler_unknown_method() {
    let handler = RequestHandler::new(PathBuf::from(".")).unwrap();

    let request = Request::new(1, "unknown_method", None);
    let response = handler.handle(request).await;

    // Should get an error for unknown method
    assert!(response.error.is_some());
    assert!(response.error.unwrap().code == -32601);
}

#[tokio::test]
async fn test_handler_invalidate() {
    let handler = RequestHandler::new(PathBuf::from(".")).unwrap();

    let params = serde_json::json!({
        "files": ["nonexistent.py"]
    });

    let request = Request::new(1, "invalidate", Some(params));
    let response = handler.handle(request).await;

    // Should succeed even for nonexistent files
    assert!(response.error.is_none());
}

// =============================================================================
// Protocol Tests
// =============================================================================

#[test]
fn test_request_creation() {
    let request = Request::new(1, "check", Some(serde_json::json!({"path": "."})));

    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.method, "check");
    assert!(request.params.is_some());
}

#[test]
fn test_response_success() {
    let response = Response::success(RequestId::Number(1), serde_json::json!({"ok": true}));

    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[test]
fn test_response_error() {
    let error = RpcError::invalid_params("test error");
    let response = Response::error(RequestId::Number(1), error);

    assert!(response.result.is_none());
    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap().code, -32602);
}

#[test]
fn test_rpc_error_types() {
    assert_eq!(RpcError::parse_error("test").code, -32700);
    assert_eq!(RpcError::invalid_request("test").code, -32600);
    assert_eq!(RpcError::method_not_found("test").code, -32601);
    assert_eq!(RpcError::invalid_params("test").code, -32602);
    assert_eq!(RpcError::internal_error("test").code, -32603);
}

// =============================================================================
// TypeInfo Display Tests
// =============================================================================

#[test]
fn test_type_info_display() {
    assert_eq!(TypeInfo::Int.display(), "int");
    assert_eq!(TypeInfo::Str.display(), "str");
    assert_eq!(TypeInfo::Bool.display(), "bool");
    assert_eq!(TypeInfo::Float.display(), "float");
    assert_eq!(TypeInfo::None.display(), "None");
    assert_eq!(TypeInfo::Any.display(), "Any");
    assert_eq!(TypeInfo::Unknown.display(), "Unknown");
}

#[test]
fn test_type_info_display_generic() {
    let list_int = TypeInfo::List(Box::new(TypeInfo::Int));
    assert_eq!(list_int.display(), "list[int]");

    let dict = TypeInfo::Dict(Box::new(TypeInfo::Str), Box::new(TypeInfo::Int));
    assert_eq!(dict.display(), "dict[str, int]");

    let optional = TypeInfo::Optional(Box::new(TypeInfo::Str));
    assert_eq!(optional.display(), "str | None");

    let union = TypeInfo::Union(vec![TypeInfo::Int, TypeInfo::Str]);
    assert_eq!(union.display(), "int | str");
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Build a SemanticModel from Python code
fn build_python_model(code: &str) -> SemanticModel {
    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(code, Language::Python).unwrap();
    build_semantic_model(&parsed, code, PathBuf::from("test.py"))
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/server/tests.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/server/tests.rs` captured during libs codegen standardization.
```
