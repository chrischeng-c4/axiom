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

use lumen::storage::{Engine, MAX_INDEX_ITEMS};
use lumen::types::{
    Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    MatchOp, MatchQuery, QueryNode, RangeBound, RangeQuery, SearchRequest, SearchResponse,
    SortMissing, SortOrder, SortSpec, TermQuery,
};

const WARMUPS: usize = 5;
const SAMPLES: usize = 21;
const TRUNCATE_COLLECTION: &str = "truncate-perf";
const TRUNCATE_FIELD: &str = "sig";
const TRUNCATE_SMALL_DOCUMENTS: usize = 10;
const TRUNCATE_LARGE_DOCUMENTS: usize = 100_000;
const TRUNCATE_SCALE_ALLOWANCE: u32 = 32;
const TRUNCATE_JITTER_FLOOR: Duration = Duration::from_millis(50);
const TRUNCATE_ABSOLUTE_CEILING: Duration = Duration::from_millis(250);
const READ_COLLECTION: &str = "number-read-perf";
const READ_FIELD: &str = "rank";
const READ_DOCUMENTS: usize = 100_000;
const READ_PAGE_SIZE: u32 = 100;
const READ_RANGE_START: usize = 20_000;
const READ_SORT_LOWER_BOUND: usize = 10_000;
const READ_CURSOR_LOWER_BOUND: usize = 20_000;
const READ_RANGE_MEDIAN_BOUND: Duration = Duration::from_millis(250);
const READ_SORT_MEDIAN_BOUND: Duration = Duration::from_millis(500);
const READ_CURSOR_MEDIAN_BOUND: Duration = Duration::from_millis(500);

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

fn truncate_schema() -> CreateCollectionRequest {
    let mut fields = BTreeMap::new();
    fields.insert(
        TRUNCATE_FIELD.into(),
        FieldSpec {
            field_type: FieldType::Hash,
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

fn truncate_fixture_engine(documents: usize) -> Engine {
    let engine = Engine::new();
    engine
        .create_collection(TRUNCATE_COLLECTION, truncate_schema())
        .unwrap();

    let mut items = Vec::with_capacity(MAX_INDEX_ITEMS);
    for start in (0..documents).step_by(MAX_INDEX_ITEMS) {
        let end = (start + MAX_INDEX_ITEMS).min(documents);
        items.clear();
        for document in start..end {
            items.push(IndexItem {
                external_id: format!("truncate-document-{document:010}"),
                field: TRUNCATE_FIELD.into(),
                value: FieldValue::String(format!("{document:016x}")),
                version: None,
            });
        }
        engine
            .index(
                TRUNCATE_COLLECTION,
                IndexRequest {
                    items: std::mem::take(&mut items),
                    request_id: None,
                },
            )
            .unwrap();
        items = Vec::with_capacity(MAX_INDEX_ITEMS);
    }
    engine
}

fn measure_truncate(documents: usize) -> Duration {
    let engine = truncate_fixture_engine(documents);
    assert_eq!(
        engine.stats(TRUNCATE_COLLECTION).unwrap().documents_indexed,
        documents as u64,
        "fixture must contain exactly {documents} documents before truncate"
    );

    let started = Instant::now();
    engine.truncate_docs(TRUNCATE_COLLECTION).unwrap();
    let elapsed = started.elapsed();

    assert_eq!(
        engine.stats(TRUNCATE_COLLECTION).unwrap().documents_indexed,
        0,
        "truncate must make the fresh collection visible"
    );
    elapsed
}

fn truncate_cost_bound(small: Duration) -> Duration {
    small
        .saturating_mul(TRUNCATE_SCALE_ALLOWANCE)
        .max(TRUNCATE_JITTER_FLOOR)
        .min(TRUNCATE_ABSOLUTE_CEILING)
}

fn number_read_schema() -> CreateCollectionRequest {
    let mut fields = BTreeMap::new();
    fields.insert(
        READ_FIELD.into(),
        FieldSpec {
            field_type: FieldType::Number,
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

fn read_document_id(document: usize) -> String {
    format!("number-read-document-{document:010}")
}

fn number_read_fixture_engine() -> Engine {
    let engine = Engine::new();
    engine
        .create_collection(READ_COLLECTION, number_read_schema())
        .unwrap();

    let mut items = Vec::with_capacity(MAX_INDEX_ITEMS);
    for start in (0..READ_DOCUMENTS).step_by(MAX_INDEX_ITEMS) {
        let end = (start + MAX_INDEX_ITEMS).min(READ_DOCUMENTS);
        items.clear();
        for document in start..end {
            items.push(IndexItem {
                external_id: read_document_id(document),
                field: READ_FIELD.into(),
                value: FieldValue::Number(document as f64),
                version: None,
            });
        }
        engine
            .index(
                READ_COLLECTION,
                IndexRequest {
                    items: std::mem::take(&mut items),
                    request_id: None,
                },
            )
            .unwrap();
        items = Vec::with_capacity(MAX_INDEX_ITEMS);
    }
    engine
}

fn number_range_request(start: usize) -> SearchRequest {
    SearchRequest {
        query: QueryNode::Range(RangeQuery {
            field: READ_FIELD.into(),
            gt: None,
            gte: Some(RangeBound::Number(start as f64)),
            lt: Some(RangeBound::Number((start + READ_PAGE_SIZE as usize) as f64)),
            lte: None,
        }),
        limit: READ_PAGE_SIZE,
        offset: 0,
        cursor: None,
        routing_key: None,
        sort: None,
        track_total: true,
        collapse: None,
    }
}

fn sorted_number_request(cursor: Option<String>, lower_bound: usize) -> SearchRequest {
    SearchRequest {
        query: QueryNode::Range(RangeQuery {
            field: READ_FIELD.into(),
            gt: None,
            gte: Some(RangeBound::Number(lower_bound as f64)),
            lt: None,
            lte: None,
        }),
        limit: READ_PAGE_SIZE,
        offset: 0,
        cursor,
        routing_key: None,
        sort: Some(vec![SortSpec {
            field: READ_FIELD.into(),
            order: SortOrder::Asc,
            missing: SortMissing::Exclude,
        }]),
        track_total: true,
        collapse: None,
    }
}

fn assert_range_filter(response: &SearchResponse, start: usize) {
    let end = start + READ_PAGE_SIZE as usize;
    assert_eq!(response.total, (end - start) as u64);
    assert_eq!(response.hits.len(), READ_PAGE_SIZE as usize);
    let actual: BTreeMap<_, _> = response
        .hits
        .iter()
        .map(|hit| (hit.external_id.clone(), ()))
        .collect();
    let expected: BTreeMap<_, _> = (start..end)
        .map(|document| (read_document_id(document), ()))
        .collect();
    assert_eq!(actual, expected, "range returned the wrong document IDs");
}

fn assert_sorted_number_page(response: &SearchResponse, page_start: usize) {
    assert_eq!(response.total, (READ_DOCUMENTS - page_start) as u64);
    assert_eq!(response.hits.len(), READ_PAGE_SIZE as usize);
    let actual: Vec<_> = response
        .hits
        .iter()
        .map(|hit| hit.external_id.clone())
        .collect();
    let expected: Vec<_> = (page_start..page_start + READ_PAGE_SIZE as usize)
        .map(read_document_id)
        .collect();
    assert_eq!(
        actual, expected,
        "numeric sort order or page boundary changed"
    );
    assert!(
        response.cursor.is_some(),
        "more sorted pages must have a cursor"
    );
}

fn assert_pages_do_not_overlap(first: &SearchResponse, second: &SearchResponse) {
    for first_hit in &first.hits {
        assert!(
            second
                .hits
                .iter()
                .all(|second_hit| second_hit.external_id != first_hit.external_id),
            "cursor page repeated {}",
            first_hit.external_id
        );
    }
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
#[ignore = "coarse performance gate runs in the release candidate workflow"]
fn truncate_docs_cost_is_constant_from_10_to_100k_documents() {
    let ten_documents = measure_truncate(TRUNCATE_SMALL_DOCUMENTS);
    let one_hundred_thousand_documents = measure_truncate(TRUNCATE_LARGE_DOCUMENTS);
    let bound = truncate_cost_bound(ten_documents);
    eprintln!(
        "truncate_docs: documents={TRUNCATE_SMALL_DOCUMENTS}, elapsed={ten_documents:?}; \
         documents={TRUNCATE_LARGE_DOCUMENTS}, elapsed={one_hundred_thousand_documents:?}; \
         bound={bound:?}"
    );

    // The large fixture has 10,000 times the documents. The 50 ms floor accepts
    // normal hosted-runner scheduling jitter.  The 32x allowance and 250 ms cap
    // still reject a per-document truncate path instead of a state replacement.
    assert!(
        one_hundred_thousand_documents <= bound,
        "truncate of {TRUNCATE_LARGE_DOCUMENTS} documents took \
         {one_hundred_thousand_documents:?}; bound from {TRUNCATE_SMALL_DOCUMENTS} \
         documents is {bound:?}"
    );
}

#[test]
#[ignore = "coarse performance gate runs in the release candidate workflow"]
fn number_read_costs_on_100k_documents() {
    let engine = number_read_fixture_engine();
    assert_eq!(
        engine.stats(READ_COLLECTION).unwrap().documents_indexed,
        READ_DOCUMENTS as u64,
        "fixture must contain exactly {READ_DOCUMENTS} documents"
    );
    assert!(
        READ_RANGE_START + (WARMUPS + SAMPLES) * READ_PAGE_SIZE as usize <= READ_DOCUMENTS
            && READ_SORT_LOWER_BOUND + WARMUPS + SAMPLES + READ_PAGE_SIZE as usize
                <= READ_DOCUMENTS
            && READ_CURSOR_LOWER_BOUND + WARMUPS + SAMPLES + 2 * READ_PAGE_SIZE as usize
                <= READ_DOCUMENTS,
        "every unique warmup and sample page must stay within the fixture"
    );

    let mut range_start = READ_RANGE_START;
    let mut range_samples = measured_samples(|| {
        let start = range_start;
        range_start += READ_PAGE_SIZE as usize;
        let started = Instant::now();
        let response = engine
            .search(READ_COLLECTION, number_range_request(start))
            .unwrap();
        let elapsed = started.elapsed();
        assert_range_filter(&response, start);
        elapsed
    });
    assert_median_under(
        "number range filter 100k",
        &mut range_samples,
        READ_RANGE_MEDIAN_BOUND,
    );

    let mut sort_lower_bound = READ_SORT_LOWER_BOUND;
    let mut sort_samples = measured_samples(|| {
        let lower_bound = sort_lower_bound;
        sort_lower_bound += 1;
        let started = Instant::now();
        let response = engine
            .search(READ_COLLECTION, sorted_number_request(None, lower_bound))
            .unwrap();
        let elapsed = started.elapsed();
        assert_sorted_number_page(&response, lower_bound);
        elapsed
    });
    assert_median_under(
        "number explicit sort 100k",
        &mut sort_samples,
        READ_SORT_MEDIAN_BOUND,
    );

    let mut cursor_lower_bound = READ_CURSOR_LOWER_BOUND;
    let mut cursor_samples = measured_samples(|| {
        let lower_bound = cursor_lower_bound;
        cursor_lower_bound += 1;
        let first_page = engine
            .search(READ_COLLECTION, sorted_number_request(None, lower_bound))
            .unwrap();
        assert_sorted_number_page(&first_page, lower_bound);
        let started = Instant::now();
        let response = engine
            .search(
                READ_COLLECTION,
                sorted_number_request(
                    Some(
                        first_page
                            .cursor
                            .clone()
                            .expect("first sorted page must produce a cursor"),
                    ),
                    lower_bound,
                ),
            )
            .unwrap();
        let elapsed = started.elapsed();
        assert_sorted_number_page(&response, lower_bound + READ_PAGE_SIZE as usize);
        assert_pages_do_not_overlap(&first_page, &response);
        elapsed
    });
    assert_median_under(
        "number sorted cursor page 100k",
        &mut cursor_samples,
        READ_CURSOR_MEDIAN_BOUND,
    );
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
            (
                "truncate_docs_cost_is_constant_from_10_to_100k_documents",
                true
            ),
            ("number_read_costs_on_100k_documents", true),
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
