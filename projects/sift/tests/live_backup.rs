// HANDWRITE-BEGIN gap="missing-generator:unit-test:sift-live-backup" tracker="#2367" reason="Prove the protected live snapshot route exports exact DurableJournal bytes and the scheduled-runner helper uploads those bytes without reopening local storage."
use std::{collections::HashMap, sync::Arc};

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use service_auth::{Role, TokenClaims};
use sift::{
    auth::{SiftAuthConfig, SiftVerifier},
    backup::{backup_live_journal, fetch_live_snapshot},
    protected_router, EventEnvelope, ServiceState, SignalKind,
};
use tower::ServiceExt;

fn event(id: &str) -> EventEnvelope {
    let mut event = EventEnvelope::new(
        id,
        SignalKind::Log,
        serde_json::json!({"message":"live backup"}),
    );
    event
        .resource
        .insert("service.name".to_string(), "sift-backup-test".to_string());
    event
}

fn protected_state() -> (tempfile::TempDir, Arc<ServiceState>, axum::Router) {
    let data_dir = tempfile::tempdir().expect("temporary journal");
    let state = Arc::new(ServiceState::open(data_dir.path()).expect("open journal"));
    state
        .journal()
        .append(event("backup-event"))
        .expect("append durable event");
    let verifier = Arc::new(SiftVerifier::new(SiftAuthConfig {
        required: true,
        tokens: HashMap::from([
            (
                "writer-token".to_string(),
                TokenClaims {
                    subject: "writer".to_string(),
                    roles: HashMap::from([("*".to_string(), Role::Write)]),
                },
            ),
            (
                "admin-token".to_string(),
                TokenClaims {
                    subject: "backup-runner".to_string(),
                    roles: HashMap::from([("*".to_string(), Role::Admin)]),
                },
            ),
        ]),
    }));
    let app = protected_router(state.clone(), verifier);
    (data_dir, state, app)
}

#[tokio::test]
async fn protected_endpoint_returns_exact_snapshot_bytes_only_to_admin() {
    let (_data_dir, state, app) = protected_state();
    let expected = state
        .journal()
        .snapshot_bytes()
        .expect("serialize expected snapshot");

    for (token, expected_status) in [
        (None, StatusCode::UNAUTHORIZED),
        (Some("writer-token"), StatusCode::FORBIDDEN),
    ] {
        let mut request = Request::builder().uri("/admin/backup");
        if let Some(token) = token {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        let response = app
            .clone()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), expected_status);
    }

    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/backup")
                .header("authorization", "Bearer admin-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "application/json");
    let actual = to_bytes(response.into_body(), 4 * 1024 * 1024)
        .await
        .expect("read snapshot response");
    assert_eq!(actual.as_ref(), expected.as_slice());
}

#[tokio::test]
async fn live_helper_sends_bearer_and_uploads_exact_response_bytes() {
    let data_dir = tempfile::tempdir().expect("temporary journal");
    let state = Arc::new(ServiceState::open(data_dir.path()).expect("open journal"));
    state
        .journal()
        .append(event("http-backup-event"))
        .expect("append durable event");
    let expected = state
        .journal()
        .snapshot_bytes()
        .expect("serialize expected snapshot");
    let verifier = Arc::new(SiftVerifier::new(SiftAuthConfig {
        required: true,
        tokens: HashMap::from([(
            "admin-token".to_string(),
            TokenClaims {
                subject: "backup-runner".to_string(),
                roles: HashMap::from([("*".to_string(), Role::Admin)]),
            },
        )]),
    }));
    let app = protected_router(state, verifier);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test server");
    let address = listener.local_addr().expect("test server address");
    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve backup route");
    });
    let base_url = format!("http://{address}");

    let unauthorized = fetch_live_snapshot(&base_url, None)
        .await
        .expect_err("required auth must fail closed");
    assert!(unauthorized.to_string().contains("401"));

    let backup_dir = tempfile::tempdir().expect("backup destination");
    let result = backup_live_journal(
        &base_url,
        Some("admin-token"),
        &format!("file://{}", backup_dir.path().display()),
        None,
    )
    .await
    .expect("upload live snapshot");
    assert_eq!(result.object.bytes, expected.len());
    let uploaded =
        std::fs::read(backup_dir.path().join(result.object.key)).expect("read uploaded snapshot");
    assert_eq!(uploaded, expected);
    server.abort();
}
// HANDWRITE-END
