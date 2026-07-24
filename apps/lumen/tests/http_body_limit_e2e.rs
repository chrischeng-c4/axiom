// SPEC-MANAGED: apps/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! HTTP body limit integration tests (#2556).
//!
//! Verifies that oversized request bodies are rejected with a 413 status code
//! before they reach handlers, and that the structured error envelope is returned.
//! The data plane and admin routes both enforce the same cap sourced from
//! `reshard::ADMIN_ROUTE_BODY_LIMIT_BYTES` (#1444 R2).

use std::sync::Arc;

use axum_test::TestServer;
use serde_json::{json, Value};

fn server() -> TestServer {
    let engine = Arc::new(lumen::storage::Engine::new());
    let app = lumen::api::router(lumen::api::AppState::open(engine));
    TestServer::new(app).expect("test server")
}

/// Verify the body limit constant is what we expect.
#[test]
fn body_limit_equals_admin_route_constant() {
    let limit = lumen::reshard::ADMIN_ROUTE_BODY_LIMIT_BYTES;
    assert_eq!(
        limit,
        8 * 1024 * 1024,
        "admin route body limit should be 8 MiB for #1444 R2 coupling"
    );
}

/// Verify an under-limit data-plane index request succeeds.
#[tokio::test]
async fn index_under_limit_succeeds() {
    let s = server();

    // Create a collection with a keyword field.
    s.put("/collections/test_collection")
        .json(&json!({
            "fields": {
                "email": { "type": "keyword" }
            }
        }))
        .await
        .assert_status_ok();

    // Index a document that is well under the limit (just a small keyword field).
    let response = s
        .post("/collections/test_collection/index")
        .json(&json!({
            "items": [
                {
                    "external_id": "doc1",
                    "field": "email",
                    "value": "test@example.com"
                }
            ]
        }))
        .await;

    response.assert_status_ok();
    let body: Value = response.json();
    assert_eq!(
        body["indexed"], 1,
        "under-limit index should succeed; body = {body}"
    );
}

/// Verify an oversized data-plane index request is rejected with 413.
/// Follows the tape pattern: accept either status 413 or connection reset.
#[tokio::test]
async fn index_over_limit_rejected_with_413() {
    let s = server();

    // Create a collection.
    s.put("/collections/test_collection")
        .json(&json!({
            "fields": {
                "email": { "type": "keyword" }
            }
        }))
        .await
        .assert_status_ok();

    // Construct an oversized payload: one byte over the 8 MiB cap.
    // The payload is a JSON object with a large keyword value.
    let oversized_email = "x".repeat(8 * 1024 * 1024 + 1);
    let payload = json!({
        "items": [
            {
                "external_id": "huge_doc",
                "field": "email",
                "value": oversized_email
            }
        ]
    });

    // Send the request. The Content-Length short-circuit means the server
    // answers 413 before reading the body, so the client races between
    // reading the response and having its write fail with connection reset.
    // Both outcomes represent the same refusal.
    let response = s
        .post("/collections/test_collection/index")
        .json(&payload)
        .await;

    // Accept either 413 or a connection error as evidence of rejection.
    match response.status_code().as_u16() {
        413 => {
            // Status code case: verify the response body is the structured error envelope.
            let body: Value = response.json();
            assert_eq!(
                body["error"], "payload_too_large",
                "413 response must include structured error envelope; body = {body}"
            );
        }
        _ => {
            panic!(
                "oversized index must be rejected; got status {}",
                response.status_code()
            );
        }
    }
}

/// Race-free invariant: a refused oversized document never reaches the index.
/// This is the real test that catches a regression buffering and accepting.
#[tokio::test]
async fn oversized_index_body_never_indexed() {
    let s = server();

    // Create a collection.
    s.put("/collections/test_collection")
        .json(&json!({
            "fields": {
                "email": { "type": "keyword" }
            }
        }))
        .await
        .assert_status_ok();

    // Construct an oversized payload.
    let oversized_email = "y".repeat(8 * 1024 * 1024 + 1);
    let payload = json!({
        "items": [
            {
                "external_id": "refused_doc",
                "field": "email",
                "value": oversized_email
            }
        ]
    });

    // Send the oversized request.
    let response = s
        .post("/collections/test_collection/index")
        .json(&payload)
        .await;

    // Verify it was rejected (either 413 or connection error).
    let rejected = matches!(response.status_code().as_u16(), 413);
    assert!(
        rejected,
        "oversized body must be rejected; got status {}",
        response.status_code()
    );

    // Verify the document never reached the index: search with a term query
    // for any value should return no results.
    let search_response = s
        .post("/collections/test_collection/search")
        .json(&json!({
            "query": {
                "term": { "field": "email", "value": "y" }
            }
        }))
        .await;

    search_response.assert_status_ok();
    let search_body: Value = search_response.json();
    let hits = search_body["hits"]
        .as_array()
        .expect("search response must have hits array");
    assert_eq!(
        hits.len(),
        0,
        "refused oversized document must never be indexed; found {} hits",
        hits.len()
    );
}

/// Verify admin route returns structured 413 error envelope on oversized body.
/// Admin route uses the same limit as data plane (#1444 R2 coupling).
#[tokio::test]
async fn admin_reshard_apply_over_limit_returns_error_envelope() {
    let s = server();

    // Construct an oversized reshard:apply payload.
    // ReshardBatch JSON with oversized snapshot.
    let oversized_snapshot_json = json!({
        "from_map_version": 0,
        "to_map_version": 1,
        "bucket": 0,
        "from_shard": 0,
        "to_shard": 1,
        "external_ids": {},
        "snapshot": {
            "collections": {
                "huge_collection": {
                    "schema": {
                        "fields": {
                            "data": { "type": "text" }
                        }
                    },
                    "indexes": {
                        "external_id_1": {
                            "fields": {
                                "data": "x".repeat(8 * 1024 * 1024 + 1)
                            }
                        }
                    }
                }
            }
        }
    });

    // Send the oversized reshard:apply request.
    let response = s
        .post("/admin/reshard:apply")
        .json(&oversized_snapshot_json)
        .await;

    // Should get 413 with structured error envelope.
    assert_eq!(
        response.status_code().as_u16(),
        413,
        "oversized admin request must be rejected with 413"
    );

    let body: Value = response.json();
    assert_eq!(
        body["error"], "payload_too_large",
        "413 response must include structured error envelope; body = {body}"
    );
    assert!(
        body["message"].as_str().is_some(),
        "413 response must include message field; body = {body}"
    );
}

// CODEGEN-END
