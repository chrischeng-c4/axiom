// HANDWRITE-BEGIN gap="sift-metric-api-tests" tracker="1667" reason="Verify authorized typed query, pagination, and projection lag."
use std::{collections::HashMap, sync::Arc};

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use service_auth::{Role, TokenClaims};
use sift::{
    auth::{SiftAuthConfig, SiftVerifier},
    protected_router, EventEnvelope, MetricPoint, MetricTemporality, ServiceState, SignalKind,
};
use tower::ServiceExt;

fn metric(id: &str, project: &str, name: &str) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        project,
        "prod",
        id,
        SignalKind::Metric,
        serde_json::json!({}),
    );
    event.resource.insert("service.name".into(), "api".into());
    event.metric = Some(MetricPoint {
        name: name.into(),
        value: 1.0,
        unit: Some("1".into()),
        temporality: MetricTemporality::Delta,
        exemplars: Vec::new(),
    });
    event
}

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn metric_query_enforces_project_read_pagination_and_projection_lag() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    state
        .journal()
        .append(metric("metric-1", "project-a", "alpha"))
        .unwrap();
    state
        .journal()
        .append(metric("metric-2", "project-a", "beta"))
        .unwrap();
    state
        .journal()
        .append(metric("metric-3", "project-b", "hidden"))
        .unwrap();
    let verifier = Arc::new(SiftVerifier::new(SiftAuthConfig {
        required: true,
        tokens: HashMap::from([(
            "reader-token".to_string(),
            TokenClaims {
                subject: "metric-reader".to_string(),
                roles: HashMap::from([("project-a".to_string(), Role::Read)]),
            },
        )]),
    }));
    let app = protected_router(state, verifier);

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/metrics:query")
                .header("authorization", "Bearer reader-token")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"project":"project-a","min_cursor":3,"limit":1}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    let first = body_json(first).await;
    assert_eq!(first["series"].as_array().unwrap().len(), 1);
    assert_eq!(first["projection_cursor"], 3);
    assert_eq!(first["has_more"], true);
    let next = first["next_series_id"].as_str().unwrap();

    let second = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/metrics:query")
                .header("authorization", "Bearer reader-token")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"project":"project-a","after_series_id":"{next}","limit":1}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::OK);
    assert_eq!(
        body_json(second).await["series"].as_array().unwrap().len(),
        1
    );

    let denied = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/metrics:query")
                .header("authorization", "Bearer reader-token")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"project":"project-b"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);

    let lag = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/metrics:query")
                .header("authorization", "Bearer reader-token")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"project":"project-a","min_cursor":99}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(lag.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(lag.headers()["retry-after"], "1");
    let lag = body_json(lag).await;
    assert_eq!(lag["error"], "projection_lag");
    assert_eq!(lag["projection"], "metric-store");
    assert_eq!(lag["current_cursor"], 3);
}
// HANDWRITE-END
