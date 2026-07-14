// HANDWRITE-BEGIN gap="sift-trace-api-tests" tracker="1665" reason="Verify authorized trace retrieval, not found, and projection lag."
use std::{collections::HashMap, sync::Arc};

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use service_auth::{Role, TokenClaims};
use sift::{
    auth::{SiftAuthConfig, SiftVerifier},
    protected_router, router, EventEnvelope, ServiceState, SignalKind,
};
use tower::ServiceExt;

fn span(project: &str, trace_id: &str, span_id: &str) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        project,
        "test",
        format!("{trace_id}-{span_id}"),
        SignalKind::Span,
        serde_json::json!({
            "name": span_id,
            "start_time_unix_nano": 10,
            "end_time_unix_nano": 20,
            "status": {"code": "ok"}
        }),
    );
    event.trace_id = Some(trace_id.into());
    event.span_id = Some(span_id.into());
    event
}

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn trace_retrieval_enforces_project_read_and_not_found() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    state
        .journal()
        .append(span("project-a", "trace-a", "root"))
        .unwrap();
    let verifier = Arc::new(SiftVerifier::new(SiftAuthConfig {
        required: true,
        tokens: HashMap::from([(
            "reader-token".to_string(),
            TokenClaims {
                subject: "trace-reader".to_string(),
                roles: HashMap::from([("project-a".to_string(), Role::Read)]),
            },
        )]),
    }));
    let app = protected_router(state, verifier);

    let allowed = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/traces/trace-a?project=project-a&min_cursor=1")
                .header("authorization", "Bearer reader-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(allowed.status(), StatusCode::OK);
    let allowed = body_json(allowed).await;
    assert_eq!(allowed["trace_id"], "trace-a");
    assert_eq!(allowed["spans"][0]["span_id"], "root");
    assert_eq!(allowed["projection_cursor"], 1);

    let denied = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/traces/trace-a?project=project-b")
                .header("authorization", "Bearer reader-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);

    let missing = app
        .oneshot(
            Request::builder()
                .uri("/v1/traces/missing?project=project-a")
                .header("authorization", "Bearer reader-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn trace_min_cursor_uses_shared_projection_lag_contract() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    state
        .journal()
        .append(span("project-a", "trace-a", "root"))
        .unwrap();
    let response = router(state)
        .oneshot(
            Request::builder()
                .uri("/v1/traces/trace-a?project=project-a&min_cursor=99")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(response.headers()["retry-after"], "1");
    let body = body_json(response).await;
    assert_eq!(body["error"], "projection_lag");
    assert_eq!(body["projection"], "trace-store");
    assert_eq!(body["required_cursor"], 99);
    assert_eq!(body["current_cursor"], 1);
}

// HANDWRITE-END
