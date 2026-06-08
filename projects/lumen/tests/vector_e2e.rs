//! End-to-end HTTP integration test for the vector / kNN surface.
//!
//! Drives the real axum router via `axum-test::TestServer`. Mirrors
//! the README §1b contract for `FieldType::Vector` + the `knn`
//! query node, and verifies that backup + restore preserves the
//! kNN top-K.

use std::sync::Arc;

use axum_test::TestServer;
use serde_json::{json, Value};

fn server() -> TestServer {
    let engine = Arc::new(lumen::storage::Engine::new());
    let app = lumen::api::router(lumen::api::AppState::open(engine));
    TestServer::new(app).expect("test server")
}

/// Tiny seeded LCG so the fixture is deterministic regardless of host
/// `rand` version. We use it for vectors only; bench paths upstream
/// rely on the same shape.
struct Lcg(u64);
impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(6_364_136_223_846_793_005) ^ 0x9E37_79B9_7F4A_7C15)
    }
    fn next_f32(&mut self) -> f32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1442695040888963407);
        let bits = (self.0 >> 32) as u32;
        // Uniform in (-1.0, 1.0).
        ((bits as f64) / (u32::MAX as f64) * 2.0 - 1.0) as f32
    }
    fn vec(&mut self, dim: usize) -> Vec<f32> {
        (0..dim).map(|_| self.next_f32()).collect()
    }
}

fn vec_json(v: &[f32]) -> Value {
    Value::Array(
        v.iter()
            .map(|x| Value::Number(serde_json::Number::from_f64(*x as f64).unwrap()))
            .collect(),
    )
}

#[tokio::test]
async fn knn_round_trip_finds_neighbours_and_orders_them() {
    let s = server();

    s.put("/collections/items")
        .json(&json!({
            "fields": {
                "embedding": {
                    "type": "vector",
                    "dim": 64,
                    "metric": "cosine"
                }
            }
        }))
        .await
        .assert_status_ok();

    let mut rng = Lcg::new(0xC0DE_C0DE);
    let mut items = Vec::with_capacity(200);
    let mut all_vecs: Vec<(String, Vec<f32>)> = Vec::with_capacity(200);
    for i in 0..200 {
        let v = rng.vec(64);
        items.push(json!({
            "external_id": format!("e{i}"),
            "field": "embedding",
            "value": vec_json(&v),
        }));
        all_vecs.push((format!("e{i}"), v));
    }
    s.post("/collections/items/index")
        .json(&json!({ "items": items }))
        .await
        .assert_status_ok();

    let query = rng.vec(64);
    let resp = s
        .post("/collections/items/search")
        .json(&json!({
            "query": {
                "knn": {
                    "field": "embedding",
                    "vector": vec_json(&query),
                    "k": 5
                }
            },
            "limit": 5
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    let hits = body["hits"].as_array().unwrap();
    assert_eq!(hits.len(), 5, "expected 5 hits, got {body}");
    let scores: Vec<f64> = hits.iter().map(|h| h["score"].as_f64().unwrap()).collect();
    for w in scores.windows(2) {
        assert!(
            w[0] >= w[1],
            "scores not monotone non-increasing: {scores:?}"
        );
    }
}

#[tokio::test]
async fn filtered_knn_returns_nearest_within_filter_no_recall_collapse() {
    // Regression for the pgvector failure mode: `knn AND <filter>` must return
    // the nearest k *within* the filtered set, not a post-filter over the
    // global top-k (which collapses recall when the filter excludes the
    // nearest neighbours).
    let s = server();
    s.put("/collections/items")
        .json(&json!({
            "fields": {
                "embedding": { "type": "vector", "dim": 3, "metric": "l2" },
                "bucket": { "type": "keyword" }
            }
        }))
        .await
        .assert_status_ok();

    // 60 docs on a 1-D ray: v{i} sits at [i,0,0], so the nearest to [0,0,0] is
    // the smallest i. The 30 NEAREST (i < 30) are bucket=out (excluded by the
    // filter); the matching bucket=in docs start only at i = 30.
    let mut items = Vec::new();
    for i in 0..60usize {
        let bucket = if i < 30 { "out" } else { "in" };
        items.push(json!({
            "external_id": format!("v{i:02}"),
            "field": "embedding",
            "value": vec_json(&[i as f32, 0.0, 0.0]),
        }));
        items.push(json!({
            "external_id": format!("v{i:02}"),
            "field": "bucket",
            "value": bucket,
        }));
    }
    s.post("/collections/items/index")
        .json(&json!({ "items": items }))
        .await
        .assert_status_ok();

    let resp = s
        .post("/collections/items/search")
        .json(&json!({
            "query": { "and": [
                { "knn": { "field": "embedding", "vector": vec_json(&[0.0, 0.0, 0.0]), "k": 5 } },
                { "term": { "field": "bucket", "value": "in" } }
            ]},
            "limit": 5
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    let hits = body["hits"].as_array().unwrap();
    // No collapse: a post-filter over the global top-5 (all bucket=out) would
    // return 0; filter-correct kNN returns the 5 nearest bucket=in docs.
    assert_eq!(hits.len(), 5, "filtered kNN collapsed the result: {body}");
    let ids: Vec<String> = hits
        .iter()
        .map(|h| h["external_id"].as_str().unwrap().to_string())
        .collect();
    for id in &ids {
        let i: usize = id.trim_start_matches('v').parse().unwrap();
        assert!(
            i >= 30,
            "bucket=out doc {id} leaked through the filter: {ids:?}"
        );
    }
    // Nearest in-bucket neighbour ranks first.
    assert_eq!(
        ids[0], "v30",
        "nearest in-bucket neighbour should lead: {ids:?}"
    );
}

#[tokio::test]
async fn knn_backup_then_restore_preserves_topk_order() {
    let src = server();
    src.put("/collections/items")
        .json(&json!({
            "fields": {
                "embedding": {
                    "type": "vector",
                    "dim": 32,
                    "metric": "l2"
                }
            }
        }))
        .await
        .assert_status_ok();

    let mut rng = Lcg::new(0xFACE_BEEF);
    let mut items = Vec::with_capacity(60);
    for i in 0..60 {
        let v = rng.vec(32);
        items.push(json!({
            "external_id": format!("v{i}"),
            "field": "embedding",
            "value": vec_json(&v),
        }));
    }
    src.post("/collections/items/index")
        .json(&json!({ "items": items }))
        .await
        .assert_status_ok();

    let query = rng.vec(32);
    let r0 = src
        .post("/collections/items/search")
        .json(&json!({
            "query": { "knn": { "field": "embedding", "vector": vec_json(&query), "k": 5 } },
            "limit": 5
        }))
        .await;
    r0.assert_status_ok();
    let body0: Value = r0.json();
    let ids_before: Vec<String> = body0["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["external_id"].as_str().unwrap().to_string())
        .collect();

    // Snapshot the source and restore into a fresh engine.
    let dump = src.get("/admin/backup").await;
    dump.assert_status_ok();
    let snap: Value = dump.json();

    let dst = server();
    dst.post("/admin/restore")
        .json(&snap)
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);

    let r1 = dst
        .post("/collections/items/search")
        .json(&json!({
            "query": { "knn": { "field": "embedding", "vector": vec_json(&query), "k": 5 } },
            "limit": 5
        }))
        .await;
    r1.assert_status_ok();
    let body1: Value = r1.json();
    let ids_after: Vec<String> = body1["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["external_id"].as_str().unwrap().to_string())
        .collect();

    assert_eq!(
        ids_before, ids_after,
        "top-K ids changed after restore: {ids_before:?} -> {ids_after:?}"
    );
}

#[tokio::test]
async fn vector_with_scalar_quantization_works_end_to_end() {
    let s = server();
    s.put("/collections/items")
        .json(&json!({
            "fields": {
                "embedding": {
                    "type": "vector",
                    "dim": 16,
                    "metric": "l2",
                    "quantize": "sq"
                }
            }
        }))
        .await
        .assert_status_ok();

    let mut rng = Lcg::new(0xA11C_E_BEEF);
    let mut items = Vec::new();
    let mut vecs: Vec<(String, Vec<f32>)> = Vec::new();
    for i in 0..40 {
        let v = rng.vec(16);
        items.push(json!({
            "external_id": format!("q{i}"),
            "field": "embedding",
            "value": vec_json(&v),
        }));
        vecs.push((format!("q{i}"), v));
    }
    s.post("/collections/items/index")
        .json(&json!({ "items": items }))
        .await
        .assert_status_ok();

    // Querying with one of the inserted vectors should still find it
    // (or something very close) at rank 1 — SQ loses precision but
    // not so much that we ever rank the wrong item as the absolute
    // top across this small corpus.
    let (target_eid, q) = vecs[7].clone();
    let resp = s
        .post("/collections/items/search")
        .json(&json!({
            "query": { "knn": { "field": "embedding", "vector": vec_json(&q), "k": 3 } },
            "limit": 3
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    let hits = body["hits"].as_array().unwrap();
    assert!(hits.len() >= 1);
    assert_eq!(hits[0]["external_id"], target_eid);
}

#[tokio::test]
async fn vector_field_rejects_dim_mismatch() {
    let s = server();
    s.put("/collections/items")
        .json(&json!({
            "fields": { "embedding": { "type": "vector", "dim": 8, "metric": "cosine" } }
        }))
        .await
        .assert_status_ok();

    let resp = s
        .post("/collections/items/index")
        .json(&json!({
            "items": [
                { "external_id": "x1", "field": "embedding", "value": [0.1, 0.2] }
            ]
        }))
        .await;
    // dim 2 vs declared dim 8 — must fail with 4xx, not crash.
    assert!(
        resp.status_code().is_client_error(),
        "expected client error, got {}",
        resp.status_code()
    );
}
