// HANDWRITE-BEGIN gap="sift-error-report-api-tests" tracker="1666" reason="Verify durable authorized lifecycle, reopen, mute expiry, and audit/change evidence."
use std::{collections::HashMap, sync::Arc};

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use service_auth::{Role, TokenClaims};
use sift::{
    auth::{SiftAuthConfig, SiftVerifier},
    protected_router, EventEnvelope, EventQuery, ServiceState, SignalKind,
};
use tower::ServiceExt;

fn exception(id: &str, number: u64) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        "project-a",
        "prod",
        id,
        SignalKind::Exception,
        serde_json::json!({
            "exception.type": "DatabaseError",
            "exception.message": format!("order {number} failed"),
            "exception.stacktrace": format!("at checkout::load (src/checkout.rs:{number})"),
        }),
    );
    event.trace_id = Some(format!("trace-{number}"));
    event
        .resource
        .insert("service.name".into(), "checkout".into());
    event
}

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn verifier() -> Arc<SiftVerifier> {
    Arc::new(SiftVerifier::new(SiftAuthConfig {
        required: true,
        tokens: HashMap::from([
            (
                "reader-token".to_string(),
                TokenClaims {
                    subject: "reader".to_string(),
                    roles: HashMap::from([("project-a".to_string(), Role::Read)]),
                },
            ),
            (
                "writer-token".to_string(),
                TokenClaims {
                    subject: "on-call".to_string(),
                    roles: HashMap::from([("project-a".to_string(), Role::Write)]),
                },
            ),
        ]),
    }))
}

#[tokio::test]
async fn lifecycle_requires_write_and_emits_durable_audit_change_evidence() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    state.journal().append(exception("error-1", 100)).unwrap();
    let app = protected_router(state.clone(), verifier());

    let query = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/errors:query")
                .header("authorization", "Bearer reader-token")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"project":"project-a","min_cursor":1}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(query.status(), StatusCode::OK);
    let fingerprint = body_json(query).await["groups"][0]["fingerprint"]
        .as_str()
        .unwrap()
        .to_string();

    let denied = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/errors/{fingerprint}/state?project=project-a"))
                .header("authorization", "Bearer reader-token")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"state":"acknowledged"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);

    let acknowledged = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/errors/{fingerprint}/state?project=project-a"))
                .header("authorization", "Bearer writer-token")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"state":"acknowledged","reason":"investigating"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(acknowledged.status(), StatusCode::OK);
    let acknowledged = body_json(acknowledged).await;
    assert_eq!(acknowledged["state"], "acknowledged");
    assert_eq!(acknowledged["actor"], "on-call");
    assert!(acknowledged["commit_index"].as_u64().unwrap() > 0);

    let audit = state
        .journal()
        .query(EventQuery {
            signal: Some(SignalKind::AuditEvent),
            after: 0,
            limit: 10,
        })
        .unwrap();
    let change = state
        .journal()
        .query(EventQuery {
            signal: Some(SignalKind::ChangeEvent),
            after: 0,
            limit: 10,
        })
        .unwrap();
    assert_eq!(audit.len(), 1);
    assert_eq!(change.len(), 1);
    assert_eq!(audit[0].event.payload["to"], "acknowledged");
    assert_eq!(audit[0].event.payload["actor"], "on-call");

    drop(app);
    drop(state);
    let reopened = Arc::new(ServiceState::open(temp.path()).unwrap());
    let persisted = protected_router(reopened, verifier())
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/v1/errors/{fingerprint}?project=project-a&min_cursor=3"
                ))
                .header("authorization", "Bearer reader-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(persisted.status(), StatusCode::OK);
    assert_eq!(body_json(persisted).await["state"], "acknowledged");
}

#[tokio::test]
async fn resolved_group_reopens_on_a_new_occurrence_and_mute_is_bounded() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    state.journal().append(exception("error-1", 100)).unwrap();
    let app = protected_router(state.clone(), verifier());
    let query = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/errors:query")
                .header("authorization", "Bearer reader-token")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"project":"project-a"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let fingerprint = body_json(query).await["groups"][0]["fingerprint"]
        .as_str()
        .unwrap()
        .to_string();

    let expired_mute = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/errors/{fingerprint}/state?project=project-a"))
                .header("authorization", "Bearer writer-token")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"state":"muted","muted_until":"2020-01-01T00:00:00Z"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(expired_mute.status(), StatusCode::BAD_REQUEST);

    let resolved = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/errors/{fingerprint}/state?project=project-a"))
                .header("authorization", "Bearer writer-token")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"state":"resolved"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resolved.status(), StatusCode::OK);

    state.journal().append(exception("error-2", 200)).unwrap();
    let detail = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/v1/errors/{fingerprint}?project=project-a&min_cursor=4"
                ))
                .header("authorization", "Bearer reader-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail.status(), StatusCode::OK);
    let detail = body_json(detail).await;
    assert_eq!(detail["state"], "open");
    assert_eq!(detail["reopened"], true);
    assert_eq!(detail["occurrence_count"], 2);
}
// HANDWRITE-END
