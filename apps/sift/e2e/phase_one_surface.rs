use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sift::{openapi, router, ServiceState};
use tower::ServiceExt;

#[tokio::test]
async fn only_phase_one_signal_and_product_routes_are_public() {
    let temp = tempfile::tempdir().unwrap();
    let app = router(Arc::new(ServiceState::open(temp.path()).unwrap()));
    for (method, path) in [
        ("POST", "/v1/events"),
        ("POST", "/v1/events:write"),
        ("POST", "/v1/profiles"),
        ("POST", "/v1/errors:query"),
        ("POST", "/v1/audit:query"),
        ("GET", "/v1/replay"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{method} {path}");
    }
}

#[test]
fn openapi_lists_the_new_product_boundary_only() {
    let document = serde_json::to_value(openapi()).unwrap();
    let paths = document["paths"].as_object().unwrap();
    for required in [
        "/v1/logs",
        "/v1/metrics",
        "/v1/traces",
        "/api/v1/query",
        "/api/v1/logs/tail",
        "/api/v1/traces/{trace_id}",
        "/api/v1/correlate",
        "/api/v1/services",
        "/api/v1/queries/{query_id}",
        "/prometheus/api/v1/write",
        "/prometheus/api/v1/query",
        "/prometheus/api/v1/query_range",
    ] {
        assert!(paths.contains_key(required), "OpenAPI missing {required}");
    }
    for removed in [
        "/v1/events",
        "/v1/events:write",
        "/v1/profiles",
        "/v1/errors:query",
        "/v1/audit:query",
        "/v1/replay",
    ] {
        assert!(!paths.contains_key(removed), "OpenAPI retained {removed}");
    }
}
