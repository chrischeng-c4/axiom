// <HANDWRITE gap="standardize:claim-code" tracker="projects-lumen-tests-hybrid-rrf-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
//! End-to-end test for hybrid retrieval via Reciprocal Rank Fusion
//! (`QueryNode::Rrf`). Asserts the defining RRF property: a document
//! that ranks in *both* legs outranks one that is #1 in only a single
//! leg — because fused score sums `1/(k + rank)` across legs.

use std::sync::Arc;

use axum_test::TestServer;
use serde_json::{json, Value};

fn server() -> TestServer {
    let engine = Arc::new(lumen::storage::Engine::new());
    let app = lumen::api::router(lumen::api::AppState::open(engine));
    TestServer::new(app).expect("test server")
}

#[tokio::test]
async fn rrf_ranks_doc_strong_in_both_legs_above_a_single_leg_winner() {
    let s = server();
    s.put("/collections/docs")
        .json(&json!({
            "fields": {
                "f1": { "type": "text", "analyzer": "whitespace_lower" },
                "f2": { "type": "text", "analyzer": "whitespace_lower" }
            }
        }))
        .await
        .assert_status_ok();

    // Two legs: leg1 = match f1 "alpha", leg2 = match f2 "beta".
    //   * "one":  f1 has TF=3 for alpha (rank #1 in leg1), f2 has no "beta"
    //             (absent from leg2). Same f1 length as "both" so BM25 length
    //             normalisation can't flip the TF ordering.
    //   * "both": f1 has TF=1 for alpha (rank #2 in leg1), f2 matches "beta"
    //             (rank #1 in leg2).
    // RRF(k=60): both = 1/62 + 1/61 ≈ 0.0325 ; one = 1/61 ≈ 0.0164.
    // So "both" must outrank "one" even though "one" wins leg1 outright.
    let items = json!({ "items": [
        { "external_id": "one",  "field": "f1", "value": "alpha alpha alpha" },
        { "external_id": "one",  "field": "f2", "value": "gamma" },
        { "external_id": "both", "field": "f1", "value": "alpha beta gamma" },
        { "external_id": "both", "field": "f2", "value": "beta" }
    ]});
    s.post("/collections/docs/index")
        .json(&items)
        .await
        .assert_status_ok();

    let resp = s
        .post("/collections/docs/search")
        .json(&json!({
            "query": { "rrf": { "k": 60, "queries": [
                { "match": { "field": "f1", "text": "alpha" } },
                { "match": { "field": "f2", "text": "beta" } }
            ] } },
            "limit": 10
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    let ids: Vec<String> = body["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["external_id"].as_str().unwrap().to_string())
        .collect();

    assert!(
        ids.contains(&"both".to_string()),
        "fusion must include both legs' hits: {ids:?}"
    );
    assert!(
        ids.contains(&"one".to_string()),
        "leg1-only winner must still appear: {ids:?}"
    );
    assert_eq!(
        ids[0], "both",
        "a doc ranked in both legs must outrank a single-leg #1 under RRF: {ids:?}"
    );
}

#[tokio::test]
async fn rrf_default_k_is_applied_when_omitted() {
    // `k` omitted → defaults to 60; the query must still parse and fuse.
    let s = server();
    s.put("/collections/d2")
        .json(&json!({ "fields": { "t": { "type": "text", "analyzer": "whitespace_lower" } } }))
        .await
        .assert_status_ok();
    s.post("/collections/d2/index")
        .json(&json!({ "items": [
            { "external_id": "a", "field": "t", "value": "hello world" },
            { "external_id": "b", "field": "t", "value": "hello there" }
        ] }))
        .await
        .assert_status_ok();

    let resp = s
        .post("/collections/d2/search")
        .json(&json!({
            "query": { "rrf": { "queries": [
                { "match": { "field": "t", "text": "hello" } }
            ] } },
            "limit": 10
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(
        body["hits"].as_array().unwrap().len(),
        2,
        "rrf with default k should fuse the single leg's two hits: {body}"
    );
}

// </HANDWRITE>
