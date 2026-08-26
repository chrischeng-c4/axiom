// CODEGEN-BEGIN
//! Coarse perf gate.
//!
//! Asserts the in-memory engine meets the v1 budget envelope on a
//! single core. These are floor thresholds, not the full regression
//! suite (Criterion benches under `benches/` drive that). They are
//! deliberately loose enough to survive shared-runner jitter while
//! still catching order-of-magnitude regressions.
//!
//! ## Contracts inherited from the retired EC shells
//!
//! This sentence was the whole of the `// Contract:` comment in an AW-EC shell under
//! `apps/lumen/e2e/`, which ran `cargo test -p lumen --test perf_gate` in a subprocess
//! and asserted the child's exit status. The coarse timing contract now runs only
//! through the explicit release-profile candidate command. The shell added a second,
//! nested run and nothing else. It was deleted on 2026-08-20 with the EC machinery it
//! belonged to, and the sentence is the only thing it held that nothing else did.
//! The line below is prefixed with the EC id the shell was filed under.
//!
//! - `lumen-claim-competitor-performance-envelope` — Absolute latency and throughput
//!   floors stay within the ratcheted perf gate envelope.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use lumen::storage::Engine;
use lumen::types::{
    Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    MatchOp, MatchQuery, QueryNode, SearchRequest, TermQuery,
};

const WARMUPS: usize = 5;
const SAMPLES: usize = 21;

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
                version: None,
            });
            items.push(IndexItem {
                external_id: id,
                field: "email".into(),
                value: FieldValue::String(format!("u{}@x.com", rng() % 1_000)),
                version: None,
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

fn measured_samples<F>(mut operation: F) -> Vec<Duration>
where
    F: FnMut() -> Duration,
{
    for _ in 0..WARMUPS {
        operation();
    }
    let mut samples = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        samples.push(operation());
    }
    samples
}

fn assert_median_under(
    label: &str,
    samples: &mut [Duration],
    bound: Duration,
) -> (Duration, Duration) {
    assert_eq!(samples.len(), SAMPLES);
    samples.sort_unstable();
    let median = samples[10];
    let p95 = samples[19];
    eprintln!(
        "{label}: samples={samples:?}, median={median:?}, p95_element_19={p95:?}, bound<{bound:?}"
    );
    assert!(
        median < bound,
        "{label} median {median:?} is not below strict bound {bound:?}"
    );
    (median, p95)
}

#[test]
#[ignore = "coarse performance gate runs in the release candidate workflow"]
fn index_throughput_floor() {
    let mut samples = measured_samples(|| {
        // Floor: 5 000 single-field writes per second on one thread.
        let e = Arc::new(Engine::new());
        e.create_collection("u", schema()).unwrap();
        let items: Vec<_> = (0..5_000)
            .map(|i| IndexItem {
                external_id: format!("u{i}"),
                field: "email".into(),
                value: FieldValue::String(format!("u{i}@x.com")),
                version: None,
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
        start.elapsed()
    });
    // Budget: 5 000 keyword writes in well under 1 s on a dev box.
    assert_median_under("index 5k", &mut samples, Duration::from_millis(1_000));
}

#[test]
#[ignore = "coarse performance gate runs in the release candidate workflow"]
fn match_query_latency_floor() {
    let e = fixture_engine(10_000);
    let mut samples = measured_samples(|| {
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
                    offset: 0,
                    cursor: None,
                    routing_key: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap();
        let elapsed = start.elapsed();
        assert!(resp.total > 0, "expected non-empty match results");
        elapsed
    });
    // 10 k docs, single-token match with BM25 scoring. Budget: < 50 ms.
    assert_median_under("match", &mut samples, Duration::from_millis(50));
}

#[test]
#[ignore = "coarse performance gate runs in the release candidate workflow"]
fn term_query_latency_floor() {
    let e = fixture_engine(10_000);
    let mut samples = measured_samples(|| {
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
                    offset: 0,
                    cursor: None,
                    routing_key: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap();
        start.elapsed()
    });
    assert_median_under("term", &mut samples, Duration::from_millis(20));
}

#[test]
fn median_statistic_and_ignored_inventory() {
    assert_eq!(WARMUPS, 5);
    assert_eq!(SAMPLES, 21);
    let mut operation_count = 0;
    let measured = measured_samples(|| {
        operation_count += 1;
        Duration::ZERO
    });
    assert_eq!(operation_count, 5 + 21);
    assert_eq!(measured.len(), 21);

    let source = include_str!("perf_gate.rs");
    let mut attributes = Vec::new();
    let mut inventory = Vec::new();
    for raw in source.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if line.starts_with("#[") {
            attributes.push(line);
        } else if let Some(rest) = line.strip_prefix("fn ") {
            if attributes.contains(&"#[test]") {
                inventory.push((
                    rest.split('(').next().unwrap(),
                    attributes.iter().any(|attr| attr.starts_with("#[ignore")),
                ));
            }
            attributes.clear();
        } else {
            attributes.clear();
        }
    }
    assert_eq!(
        inventory,
        vec![
            ("index_throughput_floor", true),
            ("match_query_latency_floor", true),
            ("term_query_latency_floor", true),
            ("median_statistic_and_ignored_inventory", false),
        ]
    );

    let mut one_outlier = [Duration::from_millis(10); SAMPLES];
    one_outlier[19] = Duration::from_millis(19);
    one_outlier[20] = Duration::from_millis(200);
    let (_, p95) = assert_median_under("one outlier", &mut one_outlier, Duration::from_millis(50));
    assert_eq!(p95, Duration::from_millis(19));

    let mut majority_above_bound = [Duration::from_millis(60); SAMPLES];
    majority_above_bound[0] = Duration::from_millis(10);
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        assert_median_under(
            "majority above bound",
            &mut majority_above_bound,
            Duration::from_millis(50),
        );
    }))
    .is_err());

    for bound in [20, 50, 1_000] {
        let mut equal = [Duration::from_millis(bound); SAMPLES];
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assert_median_under("equal bound", &mut equal, Duration::from_millis(bound));
        }))
        .is_err());
    }
}
// CODEGEN-END
