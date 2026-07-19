// HANDWRITE-BEGIN gap="sift-audit-change-api-tests" tracker="1668" reason="Verify retention/hold precedence, scoped authorization, controlled export, and mutation evidence."
use std::{collections::HashMap, sync::Arc};

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use service_auth::{Role, TokenClaims};
use sift::{
    auth::{SiftAuthConfig, SiftVerifier},
    projection::export_content_sha256,
    protected_router, EventEnvelope, EventQuery, ServiceState, SignalKind,
};
use tower::ServiceExt;

fn old_change() -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        "project-a",
        "prod",
        "old-deploy",
        SignalKind::ChangeEvent,
        serde_json::json!({
            "actor": "deployer",
            "action": "deployment.apply",
            "target": "deployment/checkout",
            "version": "v1"
        }),
    );
    event.occurred_at = "2024-01-15T00:00:00Z".into();
    event.observed_at.clone_from(&event.occurred_at);
    event
        .resource
        .insert("service.name".into(), "checkout".into());
    event
}

fn verifier() -> Arc<SiftVerifier> {
    Arc::new(SiftVerifier::new(SiftAuthConfig {
        required: true,
        tokens: HashMap::from([
            (
                "reader-token".to_string(),
                TokenClaims {
                    subject: "auditor".to_string(),
                    roles: HashMap::from([("project-a".to_string(), Role::Read)]),
                },
            ),
            (
                "admin-token".to_string(),
                TokenClaims {
                    subject: "compliance-admin".to_string(),
                    roles: HashMap::from([("project-a".to_string(), Role::Admin)]),
                },
            ),
        ]),
    }))
}

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), 4 * 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn audit_query(app: &axum::Router, token: &str, action: &str) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/audit:query")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"project":"project-a","action":"{action}"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn legal_hold_overrides_retention_until_admin_releases_it() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    state.journal().append(old_change()).unwrap();
    let app = protected_router(state, verifier());

    let expired = audit_query(&app, "reader-token", "deployment.apply").await;
    assert_eq!(expired.status(), StatusCode::OK);
    assert!(body_json(expired).await["records"]
        .as_array()
        .unwrap()
        .is_empty());

    let denied = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/audit/holds/case-1?project=project-a")
                .header("authorization", "Bearer reader-token")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"start_time":"2024-01-01T00:00:00Z","end_time":"2024-02-01T00:00:00Z","reason":"case"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);

    let held = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/audit/holds/case-1?project=project-a")
                .header("authorization", "Bearer admin-token")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"start_time":"2024-01-01T00:00:00Z","end_time":"2024-02-01T00:00:00Z","reason":"case-123"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(held.status(), StatusCode::OK);
    let held = body_json(held).await;
    assert_eq!(held["active"], true);
    assert_eq!(held["actor"], "compliance-admin");

    let visible = audit_query(&app, "reader-token", "deployment.apply").await;
    let visible = body_json(visible).await;
    assert_eq!(visible["records"].as_array().unwrap().len(), 1);
    assert_eq!(visible["records"][0]["retained_by_hold"], true);

    let released = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/audit/holds/case-1?project=project-a")
                .header("authorization", "Bearer admin-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(released.status(), StatusCode::OK);
    assert_eq!(body_json(released).await["active"], false);
    let expired_again = audit_query(&app, "reader-token", "deployment.apply").await;
    assert!(body_json(expired_again).await["records"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn controlled_export_is_admin_scoped_hashed_durable_and_audited() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    let mut current = old_change();
    current.event_id = "current-deploy".into();
    current.occurred_at = chrono::Utc::now().to_rfc3339();
    current.observed_at.clone_from(&current.occurred_at);
    state.journal().append(current).unwrap();
    let app = protected_router(state.clone(), verifier());
    let body = r#"{"id":"export-1","query":{"project":"project-a","limit":100}}"#;

    let denied = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/audit:export")
                .header("authorization", "Bearer reader-token")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);

    let exported = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/audit:export")
                .header("authorization", "Bearer admin-token")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(exported.status(), StatusCode::OK);
    let exported = body_json(exported).await;
    assert_eq!(exported["manifest"]["id"], "export-1");
    assert_eq!(exported["manifest"]["actor"], "compliance-admin");
    assert_eq!(exported["manifest"]["record_count"], 1);
    let records: Vec<sift::projection::AuditChangeRecordV1> =
        serde_json::from_value(exported["records"].clone()).unwrap();
    assert_eq!(
        exported["manifest"]["content_sha256"],
        export_content_sha256(&records).unwrap()
    );
    assert!(exported["manifest"]["commit_index"].as_u64().unwrap() > 0);

    let audit_evidence = state
        .journal()
        .query(EventQuery {
            signal: Some(SignalKind::AuditEvent),
            after: 0,
            limit: 20,
        })
        .unwrap();
    assert!(audit_evidence
        .iter()
        .any(|event| event.event.payload["action"] == "audit.export"));

    let duplicate = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/audit:export")
                .header("authorization", "Bearer admin-token")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(duplicate.status(), StatusCode::BAD_REQUEST);
}
// HANDWRITE-END
