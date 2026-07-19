// HANDWRITE-BEGIN gap="sift-logging-api-tests" tracker="1664" reason="Verify typed query, stable tail resume, min cursor lag, and project read authorization."
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

fn log(id: &str, project: &str, message: &str) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        project,
        "test",
        id,
        SignalKind::Log,
        serde_json::json!({"jsonPayload": {"message": message}}),
    );
    event.severity = Some("ERROR".into());
    event
        .resource
        .insert("gcp.resource.type".into(), "k8s_container".into());
    event
}

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn write(app: &axum::Router, event: EventEnvelope) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&event).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn query_pagination_tail_resume_and_projection_lag_are_stable() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    let app = router(state);
    write(&app, log("log-1", "project-a", "first failure")).await;
    write(&app, log("log-2", "project-a", "second failure")).await;

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/logs:query")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"project":"project-a","min_cursor":2,"limit":1}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    let first = body_json(first).await;
    assert_eq!(first["records"][0]["event_id"], "log-1");
    assert_eq!(first["next_cursor"], 1);
    assert_eq!(first["projection_cursor"], 2);
    assert_eq!(first["has_more"], true);

    let tail = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/logs:tail?project=project-a&after_cursor=1&min_cursor=2&limit=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tail.status(), StatusCode::OK);
    let tail = body_json(tail).await;
    assert_eq!(tail["records"][0]["event_id"], "log-2");
    assert_eq!(tail["next_cursor"], 2);

    let lag = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/logs:query")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"project":"project-a","min_cursor":99,"limit":10}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(lag.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(lag.headers()["retry-after"], "1");
    let lag = body_json(lag).await;
    assert_eq!(lag["error"], "projection_lag");
    assert_eq!(lag["projection"], "logging-store");
    assert_eq!(lag["required_cursor"], 99);
    assert_eq!(lag["current_cursor"], 2);
}

#[tokio::test]
async fn logging_reads_require_project_scoped_read_access() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    state
        .journal()
        .append(log("a-1", "project-a", "allowed"))
        .unwrap();
    state
        .journal()
        .append(log("b-1", "project-b", "denied"))
        .unwrap();
    let verifier = Arc::new(SiftVerifier::new(SiftAuthConfig {
        required: true,
        tokens: HashMap::from([(
            "reader-token".to_string(),
            TokenClaims {
                subject: "logging-reader".to_string(),
                roles: HashMap::from([("project-a".to_string(), Role::Read)]),
            },
        )]),
    }));
    let app = protected_router(state, verifier);

    for (project, expected) in [
        ("project-a", StatusCode::OK),
        ("project-b", StatusCode::FORBIDDEN),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/logs:query")
                    .header("authorization", "Bearer reader-token")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"project":"{project}"}}"#)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), expected);
    }
}

// HANDWRITE-END
