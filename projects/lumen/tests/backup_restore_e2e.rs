// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! Backup → restore round-trip.

use std::sync::Arc;

use axum_test::TestServer;
use serde_json::{json, Value};

fn server() -> TestServer {
    let engine = Arc::new(lumen::storage::Engine::new());
    let app = lumen::api::router(lumen::api::AppState::open(engine));
    TestServer::new(app).expect("test server")
}

#[tokio::test]
async fn snapshot_then_restore_into_fresh_engine() {
    let src = server();
    src.put("/collections/u")
        .json(&json!({
            "fields": {
                "bio":   { "type": "text" },
                "email": { "type": "keyword" },
                "tags":  { "type": "set" },
                "age":   { "type": "number" }
            }
        }))
        .await
        .assert_status_ok();

    src.post("/collections/u/index")
        .json(&json!({
            "items": [
                { "external_id": "u1", "field": "bio",   "value": "rust engineer in taipei" },
                { "external_id": "u1", "field": "email", "value": "a@x.com" },
                { "external_id": "u1", "field": "tags",  "value": ["rust","db"] },
                { "external_id": "u1", "field": "age",   "value": 30 },
                { "external_id": "u2", "field": "email", "value": "a@x.com" },
                { "external_id": "u2", "field": "age",   "value": 25 }
            ]
        }))
        .await
        .assert_status_ok();

    let dump = src.get("/admin/backup").await;
    dump.assert_status_ok();
    let snap: Value = dump.json();
    assert_eq!(snap["version"], 1);
    assert!(snap["collections"]["u"].is_object());

    // Boot a fresh engine and restore.
    let dst = server();
    dst.post("/admin/restore")
        .json(&snap)
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);

    // Queries against the restored engine return the same results.
    let r = dst
        .post("/collections/u/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "a@x.com" } },
            "limit": 10
        }))
        .await;
    let body: Value = r.json();
    assert_eq!(body["total"], 2);

    let r = dst
        .post("/collections/u/duplicates")
        .json(&json!({ "field": "email" }))
        .await;
    let body: Value = r.json();
    assert_eq!(body["groups"].as_array().unwrap().len(), 1);

    let r = dst
        .post("/collections/u/search")
        .json(&json!({
            "query": { "range": { "field": "age", "gte": 26 } },
            "limit": 10
        }))
        .await;
    let body: Value = r.json();
    assert_eq!(body["total"], 1);
    assert_eq!(body["hits"][0]["external_id"], "u1");

    let r = dst
        .post("/collections/u/search")
        .json(&json!({
            "query": { "match": { "field": "bio", "text": "rust" } },
            "limit": 10
        }))
        .await;
    let body: Value = r.json();
    assert_eq!(body["total"], 1);
    assert_eq!(body["hits"][0]["external_id"], "u1");

    let r = dst
        .post("/collections/u/search")
        .json(&json!({
            "query": { "term": { "field": "tags", "value": "rust" } },
            "limit": 10
        }))
        .await;
    let body: Value = r.json();
    assert_eq!(body["total"], 1);
}

#[tokio::test]
async fn restore_rejects_wrong_version() {
    let s = server();
    let resp = s
        .post("/admin/restore")
        .json(&json!({ "version": 999, "collections": {} }))
        .await;
    resp.assert_status_bad_request();
}
// CODEGEN-END
