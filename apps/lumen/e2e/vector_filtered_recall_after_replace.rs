//! Filtered kNN must stay correct after a document is re-indexed.
//!
//! Re-indexing an `external_id` that already exists changes the live corpus
//! not at all. These rows hold the corpus, the query, and the allow-set fixed
//! and vary only how many times the identical documents are written, so the
//! only thing that moves between rows is the index's write history.

use std::sync::Arc;

use axum_test::TestServer;
use serde_json::{json, Value};

const CORPUS: usize = 60;
const ALLOWED_FROM: usize = 30;
const K: usize = 5;

fn server() -> TestServer {
    let engine = Arc::new(lumen::storage::Engine::new());
    let app = lumen::api::router(lumen::api::AppState::open(engine));
    TestServer::new(app).expect("test server")
}

fn vec_json(v: &[f32]) -> Value {
    Value::Array(
        v.iter()
            .map(|x| Value::Number(serde_json::Number::from_f64(*x as f64).unwrap()))
            .collect(),
    )
}

/// The documents sit on a 1-D ray at integer offsets, so the exact answer to a
/// query at the origin restricted to `bucket=in` is `v30 v31 v32 v33 v34`,
/// derivable without running the index at all.
fn expected_ids() -> Vec<String> {
    (ALLOWED_FROM..ALLOWED_FROM + K)
        .map(|i| format!("v{i:02}"))
        .collect()
}

async fn create_collection(s: &TestServer, name: &str, backend: &str) {
    s.put(&format!("/collections/{name}"))
        .json(&json!({
            "fields": {
                "embedding": {
                    "type": "vector",
                    "dim": 3,
                    "metric": "l2",
                    "backend": backend
                },
                "bucket": { "type": "keyword" }
            }
        }))
        .await
        .assert_status_ok();
}

async fn index_corpus(s: &TestServer, name: &str) {
    let mut items = Vec::new();
    for i in 0..CORPUS {
        let bucket = if i < ALLOWED_FROM { "out" } else { "in" };
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
    s.post(&format!("/collections/{name}/index"))
        .json(&json!({ "items": items }))
        .await
        .assert_status_ok();
}

/// Writes every document again with the byte-identical vector it already
/// holds. The live corpus is `CORPUS` documents before and after.
async fn replace_corpus(s: &TestServer, name: &str, rounds: usize) {
    for _ in 0..rounds {
        let mut items = Vec::new();
        for i in 0..CORPUS {
            items.push(json!({
                "external_id": format!("v{i:02}"),
                "field": "embedding",
                "value": vec_json(&[i as f32, 0.0, 0.0]),
            }));
        }
        s.post(&format!("/collections/{name}/index"))
            .json(&json!({ "items": items }))
            .await
            .assert_status_ok();
    }
}

async fn filtered_ids(s: &TestServer, name: &str) -> Vec<String> {
    let resp = s
        .post(&format!("/collections/{name}/search"))
        .json(&json!({
            "query": { "and": [
                { "knn": { "field": "embedding", "vector": vec_json(&[0.0, 0.0, 0.0]), "k": K } },
                { "term": { "field": "bucket", "value": "in" } }
            ]},
            "limit": K
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    body["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["external_id"].as_str().unwrap().to_string())
        .collect()
}

async fn unfiltered_ids(s: &TestServer, name: &str) -> Vec<String> {
    let resp = s
        .post(&format!("/collections/{name}/search"))
        .json(&json!({
            "query": { "knn": { "field": "embedding", "vector": vec_json(&[0.0, 0.0, 0.0]), "k": K } },
            "limit": K
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    body["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["external_id"].as_str().unwrap().to_string())
        .collect()
}

async fn ids_after(rounds: usize) -> Vec<String> {
    let s = server();
    create_collection(&s, "items", "hnsw-cpu").await;
    index_corpus(&s, "items").await;
    replace_corpus(&s, "items", rounds).await;
    let ids = filtered_ids(&s, "items").await;
    eprintln!(
        "replace_rounds={rounds} live_corpus={CORPUS} hits={} ids={ids:?}",
        ids.len()
    );
    ids
}

#[tokio::test]
async fn filtered_knn_returns_the_exact_allowed_top_k_with_no_replace() {
    assert_eq!(ids_after(0).await, expected_ids());
}

#[tokio::test]
async fn filtered_knn_returns_the_exact_allowed_top_k_after_one_replace() {
    assert_eq!(
        ids_after(1).await,
        expected_ids(),
        "one re-index of the identical corpus must not change the answer"
    );
}

#[tokio::test]
async fn filtered_knn_returns_the_exact_allowed_top_k_after_twenty_replaces() {
    assert_eq!(
        ids_after(20).await,
        expected_ids(),
        "recall must not decay with write history"
    );
}

#[tokio::test]
async fn filtered_knn_after_replace_agrees_with_the_exact_backend() {
    let approx = server();
    create_collection(&approx, "items", "hnsw-cpu").await;
    index_corpus(&approx, "items").await;
    replace_corpus(&approx, "items", 3).await;

    let exact = server();
    create_collection(&exact, "items", "flat-cpu").await;
    index_corpus(&exact, "items").await;
    replace_corpus(&exact, "items", 3).await;

    let ground_truth = filtered_ids(&exact, "items").await;
    assert_eq!(
        ground_truth,
        expected_ids(),
        "the exact backend defines ground truth and must itself survive replace"
    );
    assert_eq!(
        filtered_ids(&approx, "items").await,
        ground_truth,
        "the approximate backend must agree with brute force on this corpus"
    );
}

#[tokio::test]
async fn unfiltered_knn_returns_the_exact_top_k_after_one_replace() {
    let s = server();
    create_collection(&s, "items", "hnsw-cpu").await;
    index_corpus(&s, "items").await;
    replace_corpus(&s, "items", 1).await;
    let ids = unfiltered_ids(&s, "items").await;
    eprintln!("unfiltered replace_rounds=1 hits={} ids={ids:?}", ids.len());
    let want: Vec<String> = (0..K).map(|i| format!("v{i:02}")).collect();
    assert_eq!(ids, want);
}

#[test]
fn permissive_knn_after_replace_returns_stable_complete_valid_answers_across_hnsw_topologies() {
    use std::collections::HashSet;

    use lumen::types::{VectorBackend, VectorMetric, VectorSpec};
    use lumen::vector_index::{HnswCpuIndex, VectorIndex};

    // hnsw_rs topology has no controllable seed. This verifies the same
    // topology-independent public predicate for every fresh topology, not
    // general exact HNSW recall.
    const INDEPENDENT_GRAPHS: usize = 64;
    const REPEATED_QUERIES_PER_GRAPH: usize = 2;

    for graph_index in 0..INDEPENDENT_GRAPHS {
        let spec = VectorSpec {
            dim: 3,
            metric: VectorMetric::L2,
            backend: VectorBackend::HnswCpu,
            quantize: None,
        };
        let index = HnswCpuIndex::new(spec);
        for _ in 0..2 {
            for i in 0..CORPUS {
                let eid = format!("v{i:02}");
                index
                    .add(&eid, &[i as f32, 0.0, 0.0])
                    .expect("fixture insert");
            }
        }
        assert_eq!(index.len(), CORPUS);

        let mut first_hits = None;
        for query_index in 0..REPEATED_QUERIES_PER_GRAPH {
            let before = index.exact_scan_fallbacks();
            let hits = index
                .search_knn_filtered(&[0.0, 0.0, 0.0], K, &|_| true)
                .expect("permissive graph search");
            let after = index.exact_scan_fallbacks();
            let context = || {
                format!(
                    "graph={graph_index} query={query_index} hits={hits:?} before={before} after={after}"
                )
            };
            assert_eq!(hits.len(), K, "{}", context());

            let ids: Vec<&str> = hits.iter().map(|(eid, _)| eid.as_str()).collect();
            let unique: HashSet<&str> = ids.iter().copied().collect();
            assert_eq!(unique.len(), K, "{}", context());

            for (eid, score) in &hits {
                let rank = eid
                    .strip_prefix('v')
                    .and_then(|value| value.parse::<usize>().ok())
                    .filter(|&value| value < CORPUS)
                    .unwrap_or_else(|| panic!("{}", context()));
                assert_eq!(*eid, format!("v{rank:02}"), "{}", context());
                assert_eq!(*score, -(rank as f32), "{}", context());
            }
            assert!(
                hits.windows(2).all(|pair| pair[0].1 >= pair[1].1),
                "{}",
                context()
            );

            if let Some(first) = &first_hits {
                assert_eq!(hits, *first, "{}", context());
            } else {
                first_hits = Some(hits.clone());
            }
        }
    }
}

#[test]
fn selective_knn_with_fewer_than_k_allowed_after_replace_falls_back_to_exact_scan() {
    use lumen::types::{VectorBackend, VectorMetric, VectorSpec};
    use lumen::vector_index::{HnswCpuIndex, VectorIndex};

    let spec = VectorSpec {
        dim: 3,
        metric: VectorMetric::L2,
        backend: VectorBackend::HnswCpu,
        quantize: None,
    };
    let index = HnswCpuIndex::new(spec);

    for i in 0..CORPUS {
        let eid = format!("v{i:02}");
        index
            .add(&eid, &[i as f32, 0.0, 0.0])
            .expect("initial insert");
    }
    for i in 0..CORPUS {
        let eid = format!("v{i:02}");
        index
            .add(&eid, &[i as f32, 0.0, 0.0])
            .expect("replace insert");
    }

    let allowed = ["v02", "v04", "v07"];
    let before = index.exact_scan_fallbacks();
    let hits = index
        .search_knn_filtered(&[0.0, 0.0, 0.0], K, &|eid| allowed.contains(&eid))
        .expect("search knn");
    let after = index.exact_scan_fallbacks();

    let ids: Vec<String> = hits.into_iter().map(|(eid, _)| eid).collect();
    let expected = vec!["v02".to_string(), "v04".to_string(), "v07".to_string()];
    assert_eq!(ids, expected);
    assert_eq!(
        after,
        before + 1,
        "allow-set smaller than k cannot satisfy out.len() == k and must fall back to exact scan"
    );
}

#[test]
fn unresolved_nearest_orphans_force_exact_fallback() {
    use lumen::types::{VectorBackend, VectorMetric, VectorSpec};
    use lumen::vector_index::{HnswCpuIndex, VectorIndex};

    const CORPUS: usize = 60;
    const K: usize = 5;
    const EF_SEARCH: usize = 65;

    let spec = VectorSpec {
        dim: 3,
        metric: VectorMetric::L2,
        backend: VectorBackend::HnswCpu,
        quantize: None,
    };
    let orphaned = HnswCpuIndex::new(spec.clone());
    orphaned.set_ef_search(EF_SEARCH);
    for i in 0..CORPUS {
        let eid = format!("v{i:02}");
        orphaned
            .add(&eid, &[i as f32, 0.0, 0.0])
            .expect("initial insert");
    }
    for i in 0..K {
        let eid = format!("v{i:02}");
        orphaned
            .add(&eid, &[(1000 + i) as f32, 0.0, 0.0])
            .expect("orphaning replacement insert");
    }
    assert_eq!(orphaned.len(), CORPUS);

    let allowed = |eid: &str| {
        eid.strip_prefix('v')
            .and_then(|n| n.parse::<usize>().ok())
            .is_some_and(|i| (K..K * 3).contains(&i))
    };
    let before = orphaned.exact_scan_fallbacks();
    let hits = orphaned
        .search_knn_filtered(&[0.0, 0.0, 0.0], K, &allowed)
        .expect("orphaned search");
    let after = orphaned.exact_scan_fallbacks();
    let ids: Vec<String> = hits.into_iter().map(|(eid, _)| eid).collect();
    let expected: Vec<String> = (K..K * 2).map(|i| format!("v{i:02}")).collect();
    assert_eq!(ids, expected);
    assert_eq!(after, before + 1);

    let clean = HnswCpuIndex::new(spec);
    clean.set_ef_search(EF_SEARCH);
    for i in 0..CORPUS {
        let eid = format!("v{i:02}");
        let coordinate = if i < K { 1000 + i } else { i };
        clean
            .add(&eid, &[coordinate as f32, 0.0, 0.0])
            .expect("clean insert");
    }
    assert_eq!(clean.len(), CORPUS);
    let before = clean.exact_scan_fallbacks();
    let hits = clean
        .search_knn_filtered(&[0.0, 0.0, 0.0], K, &allowed)
        .expect("clean search");
    let after = clean.exact_scan_fallbacks();
    let ids: Vec<String> = hits.into_iter().map(|(eid, _)| eid).collect();
    assert_eq!(ids, expected);
    assert_eq!(after, before);
}
