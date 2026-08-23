//! CJK matching still works when the `jieba` feature is compiled out.
//!
//! Runs over the HTTP API with the default feature set: a `text` field is
//! declared with the jieba analyzer, a document containing 北京大學 is indexed,
//! and a `match` query for 北京 must return it. The superseded fallback matched
//! only the exact whole string, so this case fails against that behaviour
//! rather than describing it.
//!
//! The feature being off is the whole point. A run with `jieba` enabled
//! exercises the real tokenizer and says nothing about the fallback path that
//! ships in the default build.
// HANDWRITE-BEGIN gap="missing-generator:e2e-test:3297e901" tracker="#1975" reason="New end-to-end test (default feature set, jieba OFF): create a collection with a `text` field declared `analyzer: jieba`, index a document whose value is 北京大學, run a `match` query for 北京 over the HTTP API, and assert the document is returned (AC5, fails before this change since the old fallback only matches the exact whole string)."
// E2E test for CJK-bigram fallback in Jieba analyzer when feature is OFF
// This test verifies that Chinese text can be matched via bigrams when jieba feature is disabled.

use axum_test::TestServer;
use serde_json::{json, Value};
use std::sync::Arc;

fn server() -> TestServer {
    let engine = Arc::new(lumen::storage::Engine::new());
    let app = lumen::api::router(lumen::api::AppState::open(engine));
    TestServer::new(app).expect("test server")
}

#[tokio::test]
async fn jieba_bigram_fallback_cjk_match_e2e() {
    let s = server();

    // Create a collection with a text field using jieba analyzer
    s.put("/collections/docs")
        .json(&json!({
            "fields": {
                "title": { "type": "text", "analyzer": "jieba" }
            }
        }))
        .await
        .assert_status_ok();

    // Index a document with Chinese text
    s.post("/collections/docs/index")
        .json(&json!({
            "items": [
                { "external_id": "doc1", "field": "title", "value": "北京大學" }
            ]
        }))
        .await
        .assert_status_ok();

    // Test 1: Match query for "北京" (first bigram) should return the document
    // This is the key test - before the fix, matching "北京" would fail because
    // the whole-string fallback only indexed "北京大學" as a single token
    let resp = s
        .post("/collections/docs/search")
        .json(&json!({
            "query": { "match": { "field": "title", "text": "北京" } },
            "limit": 10
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(
        body["total"], 1,
        "Partial CJK match should work with bigram tokenization: {body}"
    );
    assert_eq!(body["hits"][0]["external_id"], "doc1");

    // Test 2: Match query for the full string should still work (bigram coverage)
    let resp = s
        .post("/collections/docs/search")
        .json(&json!({
            "query": { "match": { "field": "title", "text": "北京大學" } },
            "limit": 10
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(
        body["total"], 1,
        "Full-string match should still work via bigram coverage: {body}"
    );
    assert_eq!(body["hits"][0]["external_id"], "doc1");

    // Test 3: Match query for a middle bigram like "京大" should also work
    let resp = s
        .post("/collections/docs/search")
        .json(&json!({
            "query": { "match": { "field": "title", "text": "京大" } },
            "limit": 10
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["total"], 1, "Middle bigram match should work: {body}");
    assert_eq!(body["hits"][0]["external_id"], "doc1");
}
// HANDWRITE-END
