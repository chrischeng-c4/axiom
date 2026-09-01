// HANDWRITE-BEGIN gap="sift-auth-route-contract-tests" tracker="1604" reason="Verify required bearer auth protects data-plane routes while probes remain reachable."
use std::{collections::HashMap, sync::Arc};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use service_auth::{Role, TokenClaims};
use sift::{
    auth::{SiftAuthConfig, SiftVerifier},
    openapi, protected_router, ServiceState,
};
use tower::ServiceExt;

fn required_app(data_dir: &std::path::Path) -> axum::Router {
    let verifier = Arc::new(SiftVerifier::new(SiftAuthConfig {
        required: true,
        tokens: HashMap::from([(
            "writer-token".to_string(),
            TokenClaims {
                subject: "sift-test".to_string(),
                roles: HashMap::from([("*".to_string(), Role::Write)]),
            },
        )]),
    }));
    let state = Arc::new(ServiceState::open(data_dir).expect("open journal"));
    service_http::standard_probe_routes(state.clone(), Some(state.clone()), openapi)
        .merge(protected_router(state, verifier))
}

fn valid_logs() -> serde_json::Value {
    serde_json::json!({
        "resourceLogs": [{
            "resource": {"attributes": [
                {"key": "service.name", "value": {"stringValue": "sift-test"}}
            ]},
            "scopeLogs": [{"logRecords": [{"body": {"stringValue": "ok"}}]}]
        }]
    })
}

#[tokio::test]
async fn required_auth_protects_data_plane_but_not_operational_probes() {
    let data_dir = tempfile::tempdir().expect("temporary journal");
    let app = required_app(data_dir.path());

    let health = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(health.status(), StatusCode::OK);

    let missing = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/logs")
                .header("content-type", "application/json")
                .header("x-sift-project", "project-a")
                .body(Body::from(serde_json::to_vec(&valid_logs()).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);

    let accepted = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/logs")
                .header("content-type", "application/json")
                .header("x-sift-project", "project-a")
                .header("authorization", "Bearer writer-token")
                .body(Body::from(serde_json::to_vec(&valid_logs()).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(accepted.status(), StatusCode::OK);
}

// HANDWRITE-END
