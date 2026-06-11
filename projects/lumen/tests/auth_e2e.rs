// <HANDWRITE gap="standardize:claim-code" tracker="projects-lumen-tests-auth-e2e-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
//! Auth + RBAC end-to-end tests.

use std::collections::HashMap;
use std::sync::Arc;

use axum_test::TestServer;
use serde_json::json;

use lumen::api::{router, AppState};
use lumen::auth::{AuthConfig, Role, TokenClaims};
use lumen::storage::Engine;

fn auth_server(required: bool, tokens: Vec<(&str, TokenClaims)>) -> TestServer {
    let engine = Arc::new(Engine::new());
    let cfg = AuthConfig {
        required,
        tokens: tokens
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    };
    let app = router(AppState::new(engine, Arc::new(cfg)));
    TestServer::new(app).expect("test server")
}

fn claim(subject: &str, roles: &[(&str, Role)]) -> TokenClaims {
    TokenClaims {
        subject: subject.into(),
        roles: roles.iter().map(|(c, r)| (c.to_string(), *r)).collect(),
    }
}

#[tokio::test]
async fn required_auth_rejects_missing_bearer() {
    let s = auth_server(true, vec![]);
    let resp = s.get("/collections").await;
    resp.assert_status_unauthorized();
}

#[tokio::test]
async fn required_auth_rejects_invalid_bearer() {
    let s = auth_server(true, vec![]);
    let resp = s
        .get("/collections")
        .add_header("authorization", "Bearer nope")
        .await;
    resp.assert_status_unauthorized();
}

#[tokio::test]
async fn admin_can_create_collection() {
    let s = auth_server(
        true,
        vec![("tok-admin", claim("ops", &[("*", Role::Admin)]))],
    );
    let resp = s
        .put("/collections/u")
        .add_header("authorization", "Bearer tok-admin")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await;
    resp.assert_status_ok();
}

#[tokio::test]
async fn read_only_cannot_create() {
    let s = auth_server(true, vec![("tok-r", claim("viewer", &[("u", Role::Read)]))]);
    let resp = s
        .put("/collections/u")
        .add_header("authorization", "Bearer tok-r")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await;
    resp.assert_status_forbidden();
}

#[tokio::test]
async fn write_can_index_but_not_drop() {
    let tokens = HashMap::from([
        ("tok-w".to_string(), claim("worker", &[("u", Role::Write)])),
        ("tok-a".to_string(), claim("ops", &[("*", Role::Admin)])),
    ]);
    let s = TestServer::new(router(AppState::new(
        Arc::new(Engine::new()),
        Arc::new(AuthConfig {
            required: true,
            tokens,
        }),
    )))
    .unwrap();
    // Admin creates the collection.
    s.put("/collections/u")
        .add_header("authorization", "Bearer tok-a")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    // Writer indexes.
    s.post("/collections/u/index")
        .add_header("authorization", "Bearer tok-w")
        .json(&json!({
            "items": [{ "external_id": "u1", "field": "e", "value": "a@x.com" }]
        }))
        .await
        .assert_status_ok();
    // Writer cannot drop the collection.
    s.delete("/collections/u")
        .add_header("authorization", "Bearer tok-w")
        .await
        .assert_status_forbidden();
}

#[tokio::test]
async fn list_collections_filters_by_role() {
    let s = auth_server(
        true,
        vec![
            ("tok-a", claim("ops", &[("*", Role::Admin)])),
            ("tok-r1", claim("alice", &[("public", Role::Read)])),
        ],
    );
    // Admin creates two collections.
    for id in ["public", "private"] {
        s.put(&format!("/collections/{id}"))
            .add_header("authorization", "Bearer tok-a")
            .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
            .await
            .assert_status_ok();
    }
    // Restricted user sees only `public`.
    let resp = s
        .get("/collections")
        .add_header("authorization", "Bearer tok-r1")
        .await;
    let body: serde_json::Value = resp.json();
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0], "public");
}

#[tokio::test]
async fn unauthenticated_request_to_metrics_still_works() {
    // Scrape paths bypass auth even when required.
    let s = auth_server(true, vec![]);
    s.get("/metrics").await.assert_status_ok();
    s.get("/healthz").await.assert_status_ok();
    s.get("/readyz").await.assert_status_ok();
}

// </HANDWRITE>
