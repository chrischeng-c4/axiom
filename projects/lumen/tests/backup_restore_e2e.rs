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

#[cfg(feature = "backup")]
fn http_server() -> TestServer {
    let engine = Arc::new(lumen::storage::Engine::new());
    let app = lumen::api::router(lumen::api::AppState::open(engine));
    TestServer::builder()
        .http_transport()
        .build(app)
        .expect("http test server")
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

/// #1095: the CLI helper path can export SnapshotV1 bytes over HTTP and import
/// them into a fresh server.
/// @spec projects/lumen/tech-design/interfaces/cli/lumen-cli-add-dump-load-export-import-snapshot-verbs.md#unit-test
#[cfg(feature = "backup")]
#[tokio::test]
async fn http_snapshot_helpers_export_then_import() {
    let src = http_server();
    src.put("/collections/u")
        .json(&json!({ "fields": { "email": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    src.post("/collections/u/index")
        .json(&json!({
            "items": [
                { "external_id": "u1", "field": "email", "value": "a@x.com" },
                { "external_id": "u2", "field": "email", "value": "b@x.com" }
            ]
        }))
        .await
        .assert_status_ok();

    let base = src.server_url("/").expect("server url").to_string();
    let payload = lumen::backup::fetch_snapshot_bytes(&base, None)
        .await
        .expect("export snapshot bytes");
    let snap: Value = serde_json::from_slice(&payload).expect("snapshot json");
    assert_eq!(snap["version"], 1);

    let file = tempfile::NamedTempFile::new().expect("snapshot file");
    std::fs::write(file.path(), &payload).expect("write snapshot");
    let imported = std::fs::read(file.path()).expect("read snapshot");

    let dst = http_server();
    let dst_base = dst.server_url("/").expect("server url").to_string();
    lumen::backup::restore_snapshot_bytes(&dst_base, None, &imported)
        .await
        .expect("import snapshot bytes");

    let r = dst
        .post("/collections/u/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "a@x.com" } },
            "limit": 10
        }))
        .await;
    let body: Value = r.json();
    assert_eq!(body["total"], 1);
    assert_eq!(body["hits"][0]["external_id"], "u1");
}
// CODEGEN-END
