// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! `/stats` is engine *metadata*, not data analytics.
//!
//! These tests pin the shape so callers can rely on it and so a future
//! refactor doesn't accidentally turn /stats into a terms-aggregation
//! endpoint.

use std::sync::Arc;

use axum_test::TestServer;
use serde_json::{json, Value};

use lumen::api::{router, AppState};
use lumen::storage::Engine;

fn server() -> TestServer {
    let engine = Arc::new(Engine::new());
    let app = router(AppState::open(engine));
    TestServer::new(app).expect("test server")
}

#[tokio::test]
async fn stats_returns_per_field_metadata_and_documents_indexed() {
    let s = server();
    s.put("/collections/users")
        .json(&json!({
            "fields": {
                "bio":   { "type": "text" },
                "email": { "type": "keyword" },
                "age":   { "type": "number" }
            }
        }))
        .await
        .assert_status_ok();
    s.post("/collections/users/index")
        .json(&json!({ "items": [
            { "external_id": "u1", "field": "bio",   "value": "rust engineer in taipei" },
            { "external_id": "u1", "field": "email", "value": "a@x.com" },
            { "external_id": "u1", "field": "age",   "value": 30 },
            { "external_id": "u2", "field": "bio",   "value": "designer" },
            { "external_id": "u2", "field": "email", "value": "b@y.com" },
            { "external_id": "u2", "field": "age",   "value": 25 }
        ]}))
        .await
        .assert_status_ok();

    let stats: Value = s.get("/collections/users/stats").await.json();
    assert_eq!(stats["documents_indexed"], 2);
    assert!(stats["fields"]["bio"]["unique_terms"].as_u64().unwrap() > 0);
    assert_eq!(stats["fields"]["bio"]["type"], "text");
    assert_eq!(stats["fields"]["email"]["type"], "keyword");
    assert_eq!(stats["fields"]["age"]["type"], "number");

    // text fields expose avg_doc_len for BM25 transparency.
    let avg = stats["fields"]["bio"]["avg_doc_len"].as_f64().unwrap();
    assert!(avg > 0.0, "text avg_doc_len should be > 0, got {avg}");

    // non-text fields don't carry avg_doc_len.
    assert!(stats["fields"]["email"]["avg_doc_len"].is_null());
    assert!(stats["fields"]["age"]["avg_doc_len"].is_null());

    // Engine-level rollups.
    assert!(stats["storage"]["total_bytes"].as_u64().unwrap() > 0);
    assert_eq!(stats["cache"]["posting_hit_ratio"], 1.0);

    // last_indexed_at is an RFC 3339 string after a successful write.
    let ts = stats["last_indexed_at"]
        .as_str()
        .expect("last_indexed_at present");
    assert!(ts.ends_with('Z'), "expected UTC RFC3339, got {ts}");
}

#[tokio::test]
async fn stats_documents_indexed_matches_distinct_external_ids() {
    let s = server();
    s.put("/collections/users")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    let mut items = vec![];
    for i in 0..123 {
        items.push(json!({
            "external_id": format!("u{i}"),
            "field": "e",
            "value": format!("u{i}@x.com")
        }));
    }
    s.post("/collections/users/index")
        .json(&json!({ "items": items }))
        .await
        .assert_status_ok();
    let stats: Value = s.get("/collections/users/stats").await.json();
    assert_eq!(
        stats["documents_indexed"], 123,
        "caller-visible doc count must equal what was written"
    );
}

#[tokio::test]
async fn stats_last_indexed_at_absent_before_first_write() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    let stats: Value = s.get("/collections/u/stats").await.json();
    // No write yet → field absent (serde skip_serializing_if).
    assert!(
        stats.get("last_indexed_at").is_none() || stats["last_indexed_at"].is_null(),
        "last_indexed_at must be absent before any successful write"
    );
}

#[tokio::test]
async fn stats_per_field_bytes_attribute_capacity_to_the_right_field() {
    let s = server();
    s.put("/collections/u")
        .json(&json!({
            "fields": {
                "small": { "type": "keyword" },
                "big":   { "type": "text" }
            }
        }))
        .await
        .assert_status_ok();
    let mut items = vec![];
    for i in 0..50 {
        items.push(json!({
            "external_id": format!("u{i}"),
            "field": "small",
            "value": format!("v{i}")
        }));
        items.push(json!({
            "external_id": format!("u{i}"),
            "field": "big",
            "value": "the quick brown fox jumps over the lazy dog ".repeat(10)
        }));
    }
    s.post("/collections/u/index")
        .json(&json!({ "items": items }))
        .await
        .assert_status_ok();
    let stats: Value = s.get("/collections/u/stats").await.json();
    let small_bytes = stats["fields"]["small"]["bytes"].as_u64().unwrap();
    let big_bytes = stats["fields"]["big"]["bytes"].as_u64().unwrap();
    assert!(
        big_bytes > small_bytes,
        "text field should account for more bytes than a 2-char keyword field; got big={big_bytes} small={small_bytes}"
    );
    assert_eq!(
        stats["storage"]["total_bytes"].as_u64().unwrap(),
        small_bytes + big_bytes,
        "storage.total_bytes must equal sum of per-field bytes (engine metadata, not analytics)"
    );
}
// CODEGEN-END
