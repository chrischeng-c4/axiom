// CODEGEN-BEGIN
//! Online field deletion.
//!
//! ## Contracts inherited from the retired EC shells
//!
//! These 2 sentences were the whole of the `// Contract:` comment in 2 AW-EC shells
//! under `apps/lumen/e2e/`, each of which ran `cargo test -p lumen --test
//! drop_field_e2e` in a subprocess and asserted the child's exit status. `cargo test -p
//! lumen` already runs this target directly, so the shells added a second, nested run
//! and nothing else. They were deleted on 2026-08-20 with the EC machinery they
//! belonged to, and the sentence is the only thing they held that nothing else did.
//! Each line below is prefixed with the EC id its shell was filed under.
//!
//! - `lumen-claim-competitor-feature-schema-metadata` — Schema lifecycle,
//!   reindex/replay, and stats/metadata behavior pass the production conformance tests.
//! - `lumen-claim-schema-ddl-drop-field` — Collection DDL, online drop-field drain, and
//!   drain readiness semantics pass.

use std::sync::Arc;

use axum_test::TestServer;
use serde_json::{json, Value};

fn server() -> TestServer {
    let engine = Arc::new(lumen::storage::Engine::new());
    let app = lumen::api::router(lumen::api::AppState::open(engine));
    TestServer::new(app).expect("test server")
}

#[tokio::test]
async fn drop_field_removes_postings_and_bumps_version() {
    let s = server();
    let create = s
        .put("/collections/u")
        .json(&json!({
            "fields": {
                "email": { "type": "keyword" },
                "bio":   { "type": "text" }
            }
        }))
        .await;
    create.assert_status_ok();
    let body: Value = create.json();
    let v0 = body["version"].as_u64().unwrap();

    s.post("/collections/u/index")
        .json(&json!({ "items": [
            { "external_id": "u1", "field": "email", "value": "a@x.com" },
            { "external_id": "u1", "field": "bio",   "value": "rust engineer" }
        ]}))
        .await
        .assert_status_ok();

    let resp = s.delete("/collections/u/fields/email").await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["field_name"], "email");
    assert_eq!(body["version"].as_u64().unwrap(), v0 + 1);

    // Searching the dropped field is now 422 (unknown field).
    let r = s
        .post("/collections/u/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "a@x.com" } },
            "limit": 10
        }))
        .await;
    r.assert_status(axum::http::StatusCode::UNPROCESSABLE_ENTITY);

    // The remaining field still works.
    let r = s
        .post("/collections/u/search")
        .json(&json!({
            "query": { "match": { "field": "bio", "text": "rust" } },
            "limit": 10
        }))
        .await;
    let body: Value = r.json();
    assert_eq!(body["total"], 1);
}

#[tokio::test]
async fn drop_nonexistent_field_returns_422() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    let r = s.delete("/collections/u/fields/nope").await;
    r.assert_status(axum::http::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn drop_field_on_missing_collection_404() {
    let s = server();
    let r = s.delete("/collections/missing/fields/x").await;
    r.assert_status(axum::http::StatusCode::NOT_FOUND);
}
// CODEGEN-END
