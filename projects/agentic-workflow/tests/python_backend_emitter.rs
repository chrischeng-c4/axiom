// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests.md#tests
// CODEGEN-BEGIN
// Byte-equivalence regression for the Python backend emitter (T7 of
// python-backend-emitter.md). Hand-written until codegen can emit
// regenerability tests from spec test plans.
use agentic_workflow::generate::gen::python::{
    emit_pydantic_model, emit_router, HttpMethod, PydanticField, PydanticModelIr, RouteRecord,
    RouterIr,
};

const SPEC_ID: &str = "fixture-platform-backend-orders";

fn orders_router() -> RouterIr {
    RouterIr {
        name: String::from("orders_router"),
        prefix: String::from("/orders"),
        tag: String::from("orders"),
        routes: vec![
            RouteRecord {
                method: HttpMethod::Get,
                path: String::from("/"),
                handler_symbol: String::from("list_orders"),
                request_model: None,
                response_model: String::from("OrderResponse"),
            },
            RouteRecord {
                method: HttpMethod::Post,
                path: String::from("/"),
                handler_symbol: String::from("create_order"),
                request_model: Some(String::from("OrderCreate")),
                response_model: String::from("OrderResponse"),
            },
            RouteRecord {
                method: HttpMethod::Get,
                path: String::from("/{order_id}"),
                handler_symbol: String::from("get_order"),
                request_model: None,
                response_model: String::from("OrderResponse"),
            },
        ],
    }
}

fn order_create() -> PydanticModelIr {
    PydanticModelIr {
        name: String::from("OrderCreate"),
        base: String::from("BaseModel"),
        docstring: None,
        fields: vec![
            PydanticField {
                name: String::from("customer_id"),
                py_type: String::from("int"),
                default: None,
            },
            PydanticField {
                name: String::from("items"),
                py_type: String::from("list[str]"),
                default: None,
            },
            PydanticField {
                name: String::from("notes"),
                py_type: String::from("str | None"),
                default: Some(String::from("None")),
            },
        ],
    }
}

fn order_response() -> PydanticModelIr {
    PydanticModelIr {
        name: String::from("OrderResponse"),
        base: String::from("BaseModel"),
        docstring: None,
        fields: vec![
            PydanticField {
                name: String::from("id"),
                py_type: String::from("int"),
                default: None,
            },
            PydanticField {
                name: String::from("customer_id"),
                py_type: String::from("int"),
                default: None,
            },
            PydanticField {
                name: String::from("status"),
                py_type: String::from("str"),
                default: None,
            },
        ],
    }
}

#[test]
fn router_emit_is_byte_equivalent() {
    let ir = orders_router();
    let first = emit_router(SPEC_ID, &ir);
    let second = emit_router(SPEC_ID, &ir);
    assert_eq!(first, second, "router emission is not byte-equivalent");
    assert_eq!(first.path, "orders_router/router.py");
    assert!(first.content.contains("from fastapi import APIRouter"));
    assert!(first.content.contains("from .models import OrderCreate"));
    assert!(first.content.contains("from .models import OrderResponse"));
    assert!(first
        .content
        .contains("router = APIRouter(prefix=\"/orders\", tags=[\"orders\"])"));
    assert!(first
        .content
        .contains("@router.post(\"/\", response_model=OrderResponse)"));
    assert!(first
        .content
        .contains("async def create_order(payload: OrderCreate) -> OrderResponse:"));
    assert!(first
        .content
        .contains("async def list_orders() -> OrderResponse:"));
    assert!(first.content.ends_with('\n'));
}

#[test]
fn pydantic_model_emit_is_byte_equivalent() {
    for ir in [order_create(), order_response()] {
        let first = emit_pydantic_model(SPEC_ID, &ir);
        let second = emit_pydantic_model(SPEC_ID, &ir);
        assert_eq!(
            first, second,
            "pydantic model emission is not byte-equivalent"
        );
        assert!(first.path.starts_with("models/"));
        assert!(first.content.starts_with("from pydantic import BaseModel"));
        assert!(first
            .content
            .contains(&format!("class {}(BaseModel):", ir.name)));
        assert!(first.content.ends_with('\n'));
    }
}

#[test]
fn pydantic_model_emits_defaults_and_optional_fields() {
    let emitted = emit_pydantic_model(SPEC_ID, &order_create());
    assert_eq!(emitted.path, "models/order_create.py");
    assert!(emitted.content.contains("customer_id: int\n"));
    assert!(emitted.content.contains("items: list[str]\n"));
    assert!(emitted.content.contains("notes: str | None = None\n"));
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests.md#schema
// CODEGEN-BEGIN
// SPEC-REF: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests.md#schema
// TODO: Existing source behavior is covered by this feature/domain semantic TD.

// CODEGEN-END
