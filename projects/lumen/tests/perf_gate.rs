// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! Coarse perf gate.
//!
//! Asserts the in-memory engine meets the v1 budget envelope on a
//! single core. These are floor thresholds, not the full regression
//! suite (Criterion benches under `benches/` drive that). They are
//! deliberately loose enough to survive shared-runner jitter while
//! still catching order-of-magnitude regressions.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use lumen::storage::Engine;
use lumen::types::{
    Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    MatchOp, MatchQuery, QueryNode, SearchRequest, TermQuery,
};

fn schema() -> CreateCollectionRequest {
    let mut fields = BTreeMap::new();
    fields.insert(
        "bio".into(),
        FieldSpec {
            field_type: FieldType::Text,
            analyzer: Some(Analyzer::WhitespaceLower),
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        },
    );
    fields.insert(
        "email".into(),
        FieldSpec {
            field_type: FieldType::Keyword,
            analyzer: None,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        },
    );
    CreateCollectionRequest { fields }
}

fn fixture_engine(n: usize) -> Arc<Engine> {
    let e = Arc::new(Engine::new());
    e.create_collection("u", schema()).unwrap();
    // Tiny seeded LCG so the corpus is deterministic and reproducible.
    let mut seed: u64 = 0xC0DE_FACE;
    let mut rng = || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        seed
    };
    let mut items = Vec::with_capacity(n.min(10_000));
    let mut indexed = 0;
    while indexed < n {
        items.clear();
        // Two writes per doc, so cap at 5 000 docs per batch to stay
        // under the engine's 10 000-item bulk limit.
        let take = (n - indexed).min(5_000);
        for _ in 0..take {
            let id = format!("u{}", rng() % 1_000_000);
            let words = [
                "alpha", "beta", "gamma", "delta", "engineer", "rust", "ml", "designer",
            ];
            let bio = (0..6)
                .map(|_| words[(rng() % words.len() as u64) as usize])
                .collect::<Vec<_>>()
                .join(" ");
            items.push(IndexItem {
                external_id: id.clone(),
                field: "bio".into(),
                value: FieldValue::String(bio),
            });
            items.push(IndexItem {
                external_id: id,
                field: "email".into(),
                value: FieldValue::String(format!("u{}@x.com", rng() % 1_000)),
            });
        }
        e.index(
            "u",
            IndexRequest {
                items: items.clone(),
                request_id: None,
            },
        )
        .unwrap();
        indexed += take;
    }
    e
}

#[test]
fn index_throughput_floor() {
    // Floor: 5 000 single-field writes per second on one thread.
    let e = Arc::new(Engine::new());
    e.create_collection("u", schema()).unwrap();
    let items: Vec<_> = (0..5_000)
        .map(|i| IndexItem {
            external_id: format!("u{i}"),
            field: "email".into(),
            value: FieldValue::String(format!("u{i}@x.com")),
        })
        .collect();
    let start = Instant::now();
    e.index(
        "u",
        IndexRequest {
            items,
            request_id: None,
        },
    )
    .unwrap();
    let elapsed = start.elapsed();
    // Budget: 5 000 keyword writes in well under 1 s on a dev box.
    assert!(
        elapsed.as_millis() < 1_000,
        "index 5k took {elapsed:?} — perf regression?"
    );
}

#[test]
fn match_query_latency_floor() {
    let e = fixture_engine(10_000);
    let start = Instant::now();
    let resp = e
        .search(
            "u",
            SearchRequest {
                query: QueryNode::Match(MatchQuery {
                    // "alpha" is guaranteed in the corpus generator's word list.
                    field: "bio".into(),
                    text: "alpha".into(),
                    op: MatchOp::Or,
                }),
                limit: 20,
                cursor: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .unwrap();
    let elapsed = start.elapsed();
    assert!(resp.total > 0, "expected non-empty match results");
    // 10 k docs, single-token match with BM25 scoring. Budget: < 50 ms.
    assert!(
        elapsed.as_millis() < 50,
        "match took {elapsed:?} — perf regression?"
    );
}

#[test]
fn term_query_latency_floor() {
    let e = fixture_engine(10_000);
    let start = Instant::now();
    let _ = e
        .search(
            "u",
            SearchRequest {
                query: QueryNode::Term(TermQuery {
                    field: "email".into(),
                    value: FieldValue::String("u0@x.com".into()),
                }),
                limit: 10,
                cursor: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .unwrap();
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 20,
        "term took {elapsed:?} — perf regression?"
    );
}
// CODEGEN-END
