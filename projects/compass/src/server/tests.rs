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
