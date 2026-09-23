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
            ("docker_run_forwards_recovery_profile_only_when_set", false),
            (
                "docker_run_forwards_diagnostic_environment_only_for_diagnostic_mode",
                false,
            ),
            ("recovery_observation_is_bounded_and_diagnostic_only", false),
            ("diagnostic_restart_extends_only_its_outer_watchdog", false),
            (
                "restart_command_timeout_kills_and_reaps_a_stuck_child",
                false,
            ),
            ("approved_30_minute_durable_workload", true),
            (
                "fingerprint_only_readback_cannot_pass_an_ignored_field_predicate",
                false,
            ),
            (
                "vector_readback_rejects_the_target_for_both_candidate_vectors",
                false,
            ),
            (
                "selected_qualifying_cell_is_explicit_and_never_a_diagnostic_false_green",
                false,
            ),
            (
                "qualifying_receipt_path_must_be_absolute_with_an_existing_directory_parent",
                false,
            ),
            (
                "approved_index_operation_requires_the_frozen_fourteen_field_schema",
                false,
            ),
            (
                "interval_trace_histograms_require_complete_finite_monotonic_rows",
                false,
            ),
            (
                "interval_trace_records_baseline_and_fixed_cadence_deltas",
                false,
            ),
            (
                "interval_trace_counts_every_429_after_detailed_journal_cap",
                false,
            ),
            (
                "interval_trace_is_bounded_valid_json_without_raw_data",
                false,
            ),
            (
                "interval_trace_assigns_bucket_zero_failures_once",
                false,
            ),
            (
                "interval_trace_preserves_error_totals_when_detail_is_capped",
                false,
            ),
            (
                "interval_trace_separates_timeouts_from_other_transport_after_detailed_journal_cap",
                false,
            ),
            (
                "runtime_sample_keeps_pending_occupancy_and_durable_progress_without_metric_comments",
                false,
            ),
            ("runtime_metrics_reject_a_missing_required_row", false),
            ("runtime_metrics_reject_a_duplicate_required_row", false),
            ("runtime_metrics_reject_an_invalid_required_row", false),
            ("runtime_metrics_reject_an_unavailable_vmhwm_probe", false,),
            (
                "duration_counters_reject_backward_and_nonfinite_values",
                false,
            ),
            (
                "warmup_retry_policy_accepts_only_the_one_second_backpressure_hint",
                false,
            ),
            ("seed_backpressure_retry_honors_absolute_setup_deadline", false),
            (
                "hnsw_cache_seal_requires_a_strict_hnsw_receipt_and_sends_no_body",
                false,
            ),
            ("seed_checkpoint_requires_one_persisted_drained_publication", false),
            ("seed_checkpoint_honors_setup_and_body_deadlines", false),
            ("request_deadline_is_the_approved_five_seconds", false),
            (
                "post_input_workload_drain_deadline_returns_promptly",
                false,
            ),
            (
                "hnsw_cache_seal_uses_remaining_post_input_deadline_for_delayed_response",
                false,
            ),
            (
                "hnsw_cache_seal_never_response_fails_at_remaining_post_input_deadline",
                false,
            ),
            (
                "input_window_deadline_returns_promptly_with_the_timed_out_stage",
                false,
            ),
            (
                "input_window_timeout_aborts_sampler_before_drive_finalization",
                false,
            ),
            (
                "request_pump_capacity_wait_honors_input_window_deadline",
                false,
            ),
            (
                "post_restart_backend_residency_rejects_flat_and_hnsw_coercion",
                false,
            ),
            (
                "post_restart_backend_residency_rejects_missing_or_unknown_stats",
                false,
            ),
            (
                "post_restart_backend_residency_covers_every_fixed_collection_and_vector_field",
                false,
            ),
            (
                "bounded_failure_streams_keep_success_stderr_and_http_bodies_small",
                false,
            ),
            (
                "docker_log_tail_keeps_final_stdout_and_stderr_within_artifact_cap",
                false,
            ),
            (
                "failure_evidence_keeps_request_chain_and_collects_before_cleanup",
                false,
            ),
            (
                "restart_total_deadline_covers_restart_port_lookup_and_readiness",
                false,
            ),
            (
                "restart_diagnostics_emit_complete_record_after_not_ready_polls",
                false,
            ),
            (
                "recovery_observation_can_find_late_ready_but_restart_still_fails",
                false,
            ),
            (
                "restart_diagnostics_keep_partial_records_for_terminal_phase_failures",
                false,
            ),
            (
                "restart_failure_trace_writes_timing_and_cold_readback_unavailable",
                false,
            ),
            (
                "journal_records_a_real_reqwest_connect_error_with_full_chain",
                false,
            ),
            (
                "request_error_journal_lands_in_the_failure_evidence_bundle",
                false,
            ),
            (
                "request_error_journal_caps_entries_and_counts_the_overflow",
                false,
            ),
            (
                "non_success_status_record_carries_status_and_a_truncated_body",
                false,
            ),
            (
                "qualifying_matrix_keeps_every_mutation_endpoint_and_backend",
                false,
            ),
            (
                "restart_phase_diagnostics_success_reports_monotonic_phase_elapsed_and_first_ready",
                false,
            ),
            (
                "restart_phase_diagnostics_keeps_one_total_deadline_and_counts_readiness_attempts",
                false,
            ),
            (
                "restart_phase_diagnostics_retains_bounded_terminal_records_for_every_phase",
                false,
            ),
            (
                "restart_readiness_diagnostics_keep_bounded_http_transport_process_and_deadline_evidence",
                false,
            ),
            (
                "restart_readiness_diagnostic_record_names_each_failure_kind_and_caps_text",
                false,
            ),
            (
                "readyz_readiness_trace_late_binds_and_distinguishes_repeated_nonready",
                false,
            ),
            (
                "readyz_readiness_trace_caps_rows_and_counts_omissions",
                false,
            ),
            (
                "readyz_readiness_trace_does_not_retain_raw_errors_or_bodies",
                false,
            ),
            (
                "readyz_readiness_trace_success_does_not_create_trace_or_change_receipt",
                false,
            ),
            (
                "readyz_readiness_trace_renders_ordered_safe_listener_bound_terminal_failure",
                false,
            ),
            (
                "readyz_readiness_trace_rejects_non_monotonic_poll_timing",
                false,
            ),
            (
                "readyz_readiness_trace_caps_rows_and_reports_all_omissions",
                false,
            ),
            (
                "readyz_readiness_trace_rendering_excludes_body_error_and_url_data",
                false,
            ),
            (
                "readyz_readiness_trace_is_written_before_cleanup_on_failure",
                false,
            ),
            (
                "readyz_readiness_trace_pre_poll_restart_failure_creates_no_trace_artifact",
                false,
            ),
            (
                "workload_slots_are_absolute_for_independent_queries_and_paced_mutations",
                false,
            ),
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

// DURABLE-WORKLOAD-BEGIN
#[path = "support/perf_cell_receipt.rs"]
mod perf_cell_receipt;
#[path = "support/perf_workload_ledger.rs"]
mod perf_workload_ledger;

mod durable_workload {
    //! Approved #4246 durable workload in the existing `perf_gate` target.
    //!
    //! # Facets
    //!
    //! - Behavior: `apps/lumen/e2e/perf_gate.rs:2868` drives all mutation and
    //!   query paths before it writes one receipt. `perf_cell_receipt.rs:1529`
    //!   accepts only a complete exact sixteen-cell aggregate. This target runs
    //!   under `cargo test -p lumen --test perf_gate -- --ignored` for release.
    //! - Security: `perf_gate.rs:2923` rejects an ambiguous selected mode.
    //!   `perf_cell_receipt.rs:1545`, `:1573`, and `:1595` reject malformed,
    //!   duplicate, unexpected, mismatched, diagnostic, or failed receipts.
    //!   `perf_cell_receipt.rs:1694` refuses an existing output file.
    //! - Performance: `perf_cell_receipt.rs:1620`, `:1658`, and `:1677` bind
    //!   the approved 30-minute, 100 offered docops/s, 95% in-window completion,
    //!   10 QPS, latency, drain, RSS, checkpoint, and merge observations.
    //!   `perf_workload_ledger.rs:1068` records actual request/query drain.

    use std::collections::{BTreeMap, BTreeSet, VecDeque};
    use std::env;
    use std::error::Error as StdError;
    use std::fmt::{self, Display};
    use std::fs;
    use std::future::Future;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use std::sync::Arc;
    #[cfg(test)]
    use std::sync::Mutex as StdMutex;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    use super::perf_cell_receipt::{
        self, percent_milli, rate_milli, Binding as ReceiptBinding, Cell as ReceiptCell,
        Limits as ReceiptLimits, Measurement as ReceiptMeasurement, Outcome as ReceiptOutcome,
        Receipt,
    };
    use super::perf_workload_ledger::{
        Backend, DocumentOperation, Endpoint, GateFailure, Outcome, QueryClass, Request,
        RequestItem, WorkloadCase, WorkloadLedger, WorkloadReport,
    };
    use async_trait::async_trait;
    use futures::StreamExt;
    use serde_json::{json, Map, Value};
    #[cfg(test)]
    use tokio::io::AsyncWriteExt;
    use tokio::io::{AsyncRead, AsyncReadExt};
    use tokio::sync::Mutex;
    use tokio::task::{AbortHandle, JoinSet};

    const HOT_COLLECTION: &str = "perf-hot";
    const IDLE_COLLECTIONS: usize = 181;
    const HOT_DOCUMENTS: usize = 500_000;
    const MUTATION_DOCUMENTS_PER_CLASS: usize = (INPUT_SECONDS as usize / 3) * DOCOPS_PER_SECOND;
    // This base document is seeded once and lies outside the delete and replace
    // turns. Its fixed vector makes the bounded two-candidate kNN readback
    // independent of global HNSW recall.
    const VECTOR_READBACK_REFERENCE_NUMBER: usize = 400_000;
    const READBACK_MISMATCH_TOKEN: &str = "readback-mismatch";
    const IDLE_DOCUMENTS_PER_COLLECTION: usize = 100;
    const FIELD_COUNT: usize = 14;
    const WORKLOAD_SEED: u64 = perf_cell_receipt::WORKLOAD_SEED;
    const NGRAM_TEXT_FIELDS: u64 = perf_cell_receipt::NGRAM_TEXT_FIELDS;
    const VECTOR_DIMENSIONS: u64 = perf_cell_receipt::VECTOR_DIMENSIONS;
    const INPUT_SECONDS: u64 = 30 * 60;
    const DOCOPS_PER_SECOND: usize = 100;
    const QUERY_QPS: usize = 10;
    const DOCKER_CPUS: &str = "2.5";
    const DOCKER_MEMORY_BYTES: u64 = 16 * 1024 * 1024 * 1024;
    // An explicit byte count avoids Docker's unit-parser ambiguity. The
    // subsequent inspect assertion requires this exact 16 GiB value.
    const DOCKER_MEMORY: &str = "17179869184";
    const SNAPSHOT_SECONDS: &str = "15";
    const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
    const STARTUP_TIMEOUT: Duration = Duration::from_secs(30);
    const RECOVERY_OBSERVATION_ENV: &str = "LUMEN_PERF_RECOVERY_OBSERVATION_SECS";
    const RESTART_DIAGNOSTIC_PREFIX: &str = "PERF_RESTART_DIAGNOSTIC";
    const READYZ_READINESS_TRACE_FILE: &str = "readyz-readiness-trace.txt";
    const READYZ_READINESS_TRACE_MAX_ROWS: usize = 512;
    const RESTART_READINESS_BODY_MAX_BYTES: usize = 4 * 1024;
    const RESTART_DIAGNOSTIC_TEXT_MAX_BYTES: usize = 4 * 1024;
    const SETUP_TIMEOUT: Duration = Duration::from_secs(INPUT_SECONDS);
    const DRAIN_TIMEOUT: Duration = Duration::from_millis(perf_cell_receipt::DRAIN_LIMIT_MS);
    // No separate post-input budget is declared. Derive this guard from the
    // existing drain, startup, and request bounds so a success-path operation
    // cannot leave the durable cell running without changing those limits.
    const POST_INPUT_TIMEOUT: Duration = Duration::from_secs(
        DRAIN_TIMEOUT.as_secs() + STARTUP_TIMEOUT.as_secs() + REQUEST_TIMEOUT.as_secs(),
    );
    // Failure collection runs only after a case has already failed. Keep the
    // retained log bounded even when the failed container produced a large log.
    const EVIDENCE_LOG_TAIL_LINES: &str = "2000";
    const EVIDENCE_ARTIFACT_MAX_BYTES: usize = 1_024 * 1_024;
    // Keep each process channel below half the artifact cap so a successful
    // command retains both stdout and stderr without a later whole-artifact
    // truncation hiding stderr.
    const EVIDENCE_COMMAND_CHANNEL_MAX_BYTES: usize = (EVIDENCE_ARTIFACT_MAX_BYTES - 1_024) / 2;
    const EVIDENCE_HTTP_BODY_MAX_BYTES: usize = EVIDENCE_ARTIFACT_MAX_BYTES - 1_024;
    const REQUEST_CONCURRENCY: usize = 256;
    const QUERY_CONCURRENCY: usize = 64;
    const MUTATION_SLOT: Duration = Duration::from_millis(10);
    const QUERY_SLOT: Duration = Duration::from_millis(100);
    const INTERVAL_TRACE_CADENCE: Duration = Duration::from_secs(5);
    const INTERVAL_TRACE_MAX_SAMPLES: usize = 362;
    const INTERVAL_TRACE_MAX_BYTES: usize = 512 * 1024;
    const INTERVAL_TRACE_BUCKET_LABELS: [&str; 12] = [
        "0.001", "0.005", "0.01", "0.025", "0.05", "0.1", "0.25", "0.5", "1", "2.5", "5", "10",
    ];
    const STATE_WRITE_LOCK_HISTOGRAM: &str = "lumen_engine_state_write_lock_wait_seconds";
    const HNSW_ADD_HISTOGRAM: &str = "lumen_hnsw_add_seconds";
    const CHECKPOINT_COUNTER: &str = "lumen_segment_checkpoint_completed_total";
    const CHECKPOINT_ATTEMPT_STARTED_COUNTER: &str = "lumen_segment_checkpoint_started_total";
    const CHECKPOINT_ATTEMPT_IN_FLIGHT: &str = "lumen_segment_checkpoint_in_flight";
    const CHECKPOINT_ATTEMPT_FAILED_COUNTER: &str = "lumen_segment_checkpoint_failed_total";
    const MERGE_COUNTER: &str = "lumen_segment_merge_completed_total";
    const CHECKPOINT_DURATION_COUNT: &str = "lumen_segment_checkpoint_duration_seconds_count";
    const CHECKPOINT_DURATION_SUM: &str = "lumen_segment_checkpoint_duration_seconds_sum";
    const CAPTURE_LOCK_DURATION_COUNT: &str = "lumen_segment_capture_lock_seconds_count";
    const CAPTURE_LOCK_DURATION_SUM: &str = "lumen_segment_capture_lock_seconds_sum";
    const CHECKPOINT_BYTES_COUNTER: &str = "lumen_segment_checkpoint_bytes_total";
    const MERGE_READ_BYTES_COUNTER: &str = "lumen_segment_merge_read_bytes_total";
    const MERGE_WRITE_BYTES_COUNTER: &str = "lumen_segment_merge_write_bytes_total";
    const BACKPRESSURE_COUNTER: &str = "lumen_segment_backpressure_total";
    const PENDING_DELTA_BYTES: &str = "lumen_segment_pending_delta_bytes";
    const PENDING_DELTA_LAYERS: &str = "lumen_segment_pending_delta_layers";
    const SEGMENT_DISK_BYTES: &str = "lumen_segment_disk_bytes";
    const PROCESS_RSS_HIGH_WATER_BYTES: &str = "lumen_process_rss_high_water_bytes";

    const FIELD_NAMES: [&str; FIELD_COUNT] = [
        "tag",
        "category",
        "status",
        "price",
        "rank",
        "labels",
        "fingerprint",
        "title_ngram",
        "body_ngram",
        "summary_ngram",
        "title_text",
        "body_text",
        "embedding",
        "region",
    ];
    // Every collection in the fixed workload has exactly this one vector field.
    // The post-restart proof below checks its public resident-byte observation
    // instead of trusting the requested schema backend or receipt label.
    const WORKLOAD_VECTOR_FIELD: &str = "embedding";

    #[derive(Debug, Clone)]
    struct RequestFailure {
        display: String,
        source_chain: Vec<String>,
        is_timeout: bool,
        is_connect: bool,
        is_request: bool,
        is_body: bool,
        is_decode: bool,
        // Set by readback call sites (`semantic_search`) so the evidence
        // bundle can name which oracle observation failed. Transport
        // failures recorded straight into the request-error journal have
        // no separate context string; their endpoint/request id already
        // identify them.
        context: Option<String>,
    }

    impl RequestFailure {
        fn from_reqwest(error: reqwest::Error) -> Self {
            let display = error.to_string();
            let mut source_chain = vec![display.clone()];
            let mut source = error.source();
            while let Some(next) = source {
                source_chain.push(next.to_string());
                source = next.source();
            }
            Self {
                display,
                source_chain,
                is_timeout: error.is_timeout(),
                is_connect: error.is_connect(),
                is_request: error.is_request(),
                is_body: error.is_body(),
                is_decode: error.is_decode(),
                context: None,
            }
        }

        /// Synthesizes a chain-carrying failure for a harness-observed
        /// condition with no underlying [`reqwest::Error`] — a non-2xx
        /// workload response or the outer per-request deadline elapsing.
        /// `source_chain` still carries the single display line so every
        /// evidence consumer renders it exactly like a transport error.
        fn synthetic(display: String, is_timeout: bool) -> Self {
            Self {
                source_chain: vec![display.clone()],
                display,
                is_timeout,
                is_connect: false,
                is_request: false,
                is_body: false,
                is_decode: false,
                context: None,
            }
        }

        fn with_context(mut self, context: impl Into<String>) -> Self {
            self.context = Some(context.into());
            self
        }
    }

    #[derive(Debug)]
    enum HarnessError {
        MissingEnvironment(&'static str),
        InvalidSelection(String),
        UnpinnedImage(String),
        Command {
            program: &'static str,
            args: Vec<String>,
            detail: String,
        },
        DockerLimits(String),
        Startup(String),
        Http(String),
        RequestFailure(RequestFailure),
        MissingRuntimeMetric(&'static str),
        DuplicateRuntimeMetric(&'static str),
        MetricParse {
            name: &'static str,
            value: String,
        },
        Workload(Vec<GateFailure>),
        Task(String),
        DataInvariant(String),
        SetupTimeout {
            stage: &'static str,
        },
        InputWindowTimeout {
            stage: &'static str,
            timeout: Duration,
        },
        PostInputTimeout {
            stage: &'static str,
            timeout: Duration,
        },
    }

    impl HarnessError {
        fn request_failure(error: reqwest::Error) -> Self {
            Self::RequestFailure(RequestFailure::from_reqwest(error))
        }
    }

    impl Display for HarnessError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::MissingEnvironment(name) => {
                    write!(formatter, "required environment is missing: {name}")
                }
                Self::InvalidSelection(detail) => {
                    write!(formatter, "invalid performance matrix selection: {detail}")
                }
                Self::UnpinnedImage(image) => write!(
                    formatter,
                    "LUMEN_PERF_IMAGE must be repo@sha256:<64hex> or local sha256:<64hex>: {image}"
                ),
                Self::Command {
                    program,
                    args,
                    detail,
                } => write!(formatter, "{program} {} failed: {detail}", args.join(" ")),
                Self::DockerLimits(detail) => write!(
                    formatter,
                    "Docker did not apply the approved resource limits: {detail}"
                ),
                Self::Startup(detail) => {
                    write!(formatter, "Lumen container did not become ready: {detail}")
                }
                Self::Http(detail) => write!(formatter, "HTTP workload request failed: {detail}"),
                Self::RequestFailure(detail) => {
                    write!(
                        formatter,
                        "HTTP workload request failed: {}",
                        detail.display
                    )
                }
                Self::MissingRuntimeMetric(name) => {
                    write!(formatter, "required runtime metric is absent: {name}")
                }
                Self::DuplicateRuntimeMetric(name) => {
                    write!(
                        formatter,
                        "required runtime metric appears more than once: {name}"
                    )
                }
                Self::MetricParse { name, value } => {
                    write!(formatter, "cannot parse metric {name}: {value}")
                }
                Self::Workload(failures) => {
                    write!(formatter, "approved workload gate failed: {failures:?}")
                }
                Self::Task(detail) => write!(formatter, "workload task failed: {detail}"),
                Self::DataInvariant(detail) => {
                    write!(formatter, "workload fixture invariant failed: {detail}")
                }
                Self::SetupTimeout { stage } => {
                    write!(
                        formatter,
                        "setup stage {stage} exceeded its absolute deadline"
                    )
                }
                Self::InputWindowTimeout { stage, timeout } => write!(
                    formatter,
                    "input-window stage {stage} exceeded the {timeout:?} deadline"
                ),
                Self::PostInputTimeout { stage, timeout } => write!(
                    formatter,
                    "post-input stage {stage} exceeded the {timeout:?} deadline"
                ),
            }
        }
    }

    impl StdError for HarnessError {}

    type Result<T> = std::result::Result<T, HarnessError>;

    #[derive(Debug, Clone, Copy)]
    enum VectorBackend {
        FlatCpu,
        HnswCpu,
    }

    impl VectorBackend {
        fn ledger_backend(self) -> Backend {
            match self {
                Self::FlatCpu => Backend::FlatCpu,
                Self::HnswCpu => Backend::HnswCpu,
            }
        }

        fn wire_name(self) -> &'static str {
            match self {
                Self::FlatCpu => "flat-cpu",
                Self::HnswCpu => "hnsw-cpu",
            }
        }

        fn requires_hnsw_cache_seal(self) -> bool {
            matches!(self, Self::HnswCpu)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct HnswCacheSealReceipt {
        cache_fields: u64,
        durability: &'static str,
        mutation_epoch: u64,
        mutation_apply_revision: u64,
    }

    #[derive(Debug, Clone, Copy)]
    struct CaseConfig {
        primary_endpoint: Endpoint,
        primary_batch_size: usize,
        vector_backend: VectorBackend,
    }

    impl CaseConfig {
        fn from_optional_selection(selection: &SelectionInput) -> Result<Option<Self>> {
            let endpoint_value = match selection.endpoint.as_deref() {
                Some(value) => value,
                None => return Ok(None),
            };
            let endpoint = match endpoint_value {
                "index" => Endpoint::Index,
                "replace" => Endpoint::Replace,
                "unindex" => Endpoint::Unindex,
                other => {
                    return Err(HarnessError::InvalidSelection(format!(
                        "LUMEN_PERF_ENDPOINT={other}; expected index, replace, or unindex"
                    )))
                }
            };
            let batch = selection
                .batch
                .as_deref()
                .ok_or(HarnessError::MissingEnvironment("LUMEN_PERF_BATCH"))?
                .parse::<usize>()
                .map_err(|error| {
                    HarnessError::InvalidSelection(format!("LUMEN_PERF_BATCH: {error}"))
                })?;
            let valid = match endpoint {
                Endpoint::Index | Endpoint::Unindex => matches!(batch, 1 | 100 | 1_000),
                Endpoint::Replace => matches!(batch, 1 | 32),
            };
            if !valid {
                return Err(HarnessError::InvalidSelection(format!(
                    "endpoint {endpoint:?} does not admit batch {batch}"
                )));
            }
            let vector_backend = match selection
                .backend
                .as_deref()
                .ok_or(HarnessError::MissingEnvironment("LUMEN_PERF_BACKEND"))?
            {
                "flat-cpu" => VectorBackend::FlatCpu,
                "hnsw-cpu" => VectorBackend::HnswCpu,
                other => {
                    return Err(HarnessError::InvalidSelection(format!(
                        "LUMEN_PERF_BACKEND={other}; expected flat-cpu or hnsw-cpu"
                    )))
                }
            };
            Ok(Some(Self {
                primary_endpoint: endpoint,
                primary_batch_size: batch,
                vector_backend,
            }))
        }

        fn ledger_case(self) -> WorkloadCase {
            WorkloadCase::new(
                self.primary_endpoint,
                self.primary_batch_size,
                self.vector_backend.ledger_backend(),
            )
        }

        fn batch_size_for(self, endpoint: Endpoint) -> usize {
            self.ledger_case().batch_size_for(endpoint)
        }

        fn receipt_cell(self) -> ReceiptCell {
            ReceiptCell::new(
                endpoint_name(self.primary_endpoint),
                self.primary_batch_size as u64,
                self.vector_backend.wire_name(),
            )
        }
    }

    #[derive(Debug, Clone, Default)]
    struct SelectionInput {
        endpoint: Option<String>,
        batch: Option<String>,
        backend: Option<String>,
        diagnostic: Option<String>,
        qualifying: Option<String>,
        recovery_observation_secs: Option<String>,
    }

    impl SelectionInput {
        fn from_environment() -> Result<Self> {
            Ok(Self {
                endpoint: optional_env("LUMEN_PERF_ENDPOINT")?,
                batch: optional_env("LUMEN_PERF_BATCH")?,
                backend: optional_env("LUMEN_PERF_BACKEND")?,
                diagnostic: optional_env("LUMEN_PERF_DIAGNOSTIC")?,
                qualifying: optional_env("LUMEN_PERF_QUALIFYING_CELL")?,
                recovery_observation_secs: optional_env(RECOVERY_OBSERVATION_ENV)?,
            })
        }
    }

    #[derive(Debug, Clone, Copy)]
    enum SelectedMode {
        FullMatrix,
        Diagnostic(CaseConfig),
        Qualifying(CaseConfig),
    }

    impl SelectedMode {
        fn from_selection(selection: &SelectionInput) -> Result<Self> {
            let config = CaseConfig::from_optional_selection(selection)?;
            let diagnostic = exact_flag(selection.diagnostic.as_deref(), "LUMEN_PERF_DIAGNOSTIC")?;
            let qualifying = exact_flag(
                selection.qualifying.as_deref(),
                "LUMEN_PERF_QUALIFYING_CELL",
            )?;
            diagnostic_recovery_observation(
                selection.recovery_observation_secs.as_deref(),
                diagnostic,
            )?;
            if diagnostic && qualifying {
                return Err(HarnessError::InvalidSelection(
                    "LUMEN_PERF_DIAGNOSTIC and LUMEN_PERF_QUALIFYING_CELL are mutually exclusive"
                        .to_owned(),
                ));
            }
            match (config, diagnostic, qualifying) {
                (None, false, false) => Ok(Self::FullMatrix),
                (None, _, _) => Err(HarnessError::InvalidSelection(
                    "a diagnostic or qualifying mode needs endpoint, batch, and backend"
                        .to_owned(),
                )),
                (Some(config), true, false) => Ok(Self::Diagnostic(config)),
                (Some(config), false, true) => Ok(Self::Qualifying(config)),
                (Some(_), true, true) => Err(HarnessError::InvalidSelection(
                    "LUMEN_PERF_DIAGNOSTIC and LUMEN_PERF_QUALIFYING_CELL are mutually exclusive"
                        .to_owned(),
                )),
                (Some(_), false, false) => Err(HarnessError::InvalidSelection(
                    "a selected endpoint/batch/backend cell needs exactly one of LUMEN_PERF_DIAGNOSTIC=1 or LUMEN_PERF_QUALIFYING_CELL=1"
                        .to_owned(),
                )),
            }
        }
    }

    #[derive(Debug, Clone)]
    struct QualifyingContext {
        repository: String,
        run_id: String,
        run_attempt: String,
        commit: String,
        image_reference: String,
        receipt_path: PathBuf,
    }

    impl QualifyingContext {
        fn from_environment() -> Result<Self> {
            let image_reference = required_env("LUMEN_PERF_IMAGE")?;
            if !is_repository_digest(&image_reference) {
                return Err(HarnessError::InvalidSelection(
                    "a qualifying receipt requires LUMEN_PERF_IMAGE=repo@sha256:<64hex>; local image IDs are diagnostic only"
                        .to_owned(),
                ));
            }
            let receipt_path = required_env("LUMEN_PERF_RECEIPT_PATH")?;
            if receipt_path.is_empty() {
                return Err(HarnessError::InvalidSelection(
                    "LUMEN_PERF_RECEIPT_PATH must name a new receipt file".to_owned(),
                ));
            }
            let receipt_path = PathBuf::from(receipt_path);
            if !receipt_path.is_absolute() {
                return Err(HarnessError::InvalidSelection(
                    "LUMEN_PERF_RECEIPT_PATH must be an absolute path".to_owned(),
                ));
            }
            let parent = receipt_path.parent().ok_or_else(|| {
                HarnessError::InvalidSelection(
                    "LUMEN_PERF_RECEIPT_PATH must have a parent directory".to_owned(),
                )
            })?;
            let metadata = fs::metadata(parent).map_err(|error| {
                HarnessError::InvalidSelection(format!(
                    "LUMEN_PERF_RECEIPT_PATH parent {} is not an existing directory: {error}",
                    parent.display()
                ))
            })?;
            if !metadata.is_dir() {
                return Err(HarnessError::InvalidSelection(format!(
                    "LUMEN_PERF_RECEIPT_PATH parent {} is not a directory",
                    parent.display()
                )));
            }
            Ok(Self {
                repository: required_env("LUMEN_PERF_REPOSITORY")?,
                run_id: required_env("LUMEN_PERF_RUN_ID")?,
                run_attempt: required_env("LUMEN_PERF_RUN_ATTEMPT")?,
                commit: required_env("LUMEN_PERF_COMMIT")?,
                image_reference,
                receipt_path,
            })
        }

        fn binding(&self, actual_image_id: String) -> ReceiptBinding {
            ReceiptBinding {
                repository: self.repository.clone(),
                run_id: self.run_id.clone(),
                run_attempt: self.run_attempt.clone(),
                commit: self.commit.clone(),
                image_reference: self.image_reference.clone(),
                actual_image_id,
            }
        }
    }

    fn optional_env(name: &'static str) -> Result<Option<String>> {
        match env::var(name) {
            Ok(value) => Ok(Some(value)),
            Err(env::VarError::NotPresent) => Ok(None),
            Err(error) => Err(HarnessError::InvalidSelection(format!("{name}: {error}"))),
        }
    }

    fn exact_flag(value: Option<&str>, name: &'static str) -> Result<bool> {
        match value {
            None => Ok(false),
            Some("1") => Ok(true),
            Some(other) => Err(HarnessError::InvalidSelection(format!(
                "{name}={other}; expected exactly 1"
            ))),
        }
    }

    fn diagnostic_recovery_observation(
        raw: Option<&str>,
        diagnostic: bool,
    ) -> Result<Option<Duration>> {
        let Some(raw) = raw else {
            return Ok(None);
        };
        if !diagnostic {
            return Err(HarnessError::InvalidSelection(format!(
                "{RECOVERY_OBSERVATION_ENV} needs LUMEN_PERF_DIAGNOSTIC=1"
            )));
        }
        let seconds = raw
            .parse::<u64>()
            .ok()
            .filter(|seconds| (31..=120).contains(seconds))
            .ok_or_else(|| {
                HarnessError::InvalidSelection(format!(
                    "{RECOVERY_OBSERVATION_ENV} must be an integer from 31 through 120"
                ))
            })?;
        Ok(Some(Duration::from_secs(seconds)))
    }

    fn endpoint_name(endpoint: Endpoint) -> &'static str {
        match endpoint {
            Endpoint::Index => "index",
            Endpoint::Replace => "replace",
            Endpoint::Unindex => "unindex",
        }
    }

    fn qualifying_matrix() -> Vec<CaseConfig> {
        let mut cases = Vec::new();
        let cells: &[(Endpoint, &[usize])] = &[
            (Endpoint::Index, &[1, 100, 1_000]),
            (Endpoint::Replace, &[1, 32]),
            (Endpoint::Unindex, &[1, 100, 1_000]),
        ];
        for vector_backend in [VectorBackend::FlatCpu, VectorBackend::HnswCpu] {
            for &(primary_endpoint, batch_sizes) in cells {
                for primary_batch_size in batch_sizes {
                    cases.push(CaseConfig {
                        primary_endpoint,
                        primary_batch_size: *primary_batch_size,
                        vector_backend,
                    });
                }
            }
        }
        assert_eq!(
            cases.len(),
            16,
            "the approved matrix has eight cells per backend"
        );
        cases
    }

    fn required_env(name: &'static str) -> Result<String> {
        env::var(name).map_err(|_| HarnessError::MissingEnvironment(name))
    }

    #[derive(Debug)]
    struct DockerLumen {
        container: String,
        volume: String,
        base: String,
        client: reqwest::Client,
        image_reference: String,
        image_id: String,
        cleanup_armed: bool,
        recovery_observation: Option<Duration>,
        // Shared across every `send_*` task the request pump spawns for
        // this server so a failed durable cell's evidence bundle can name
        // why, not only that, each counted `Outcome::Failed`/`TimedOut`.
        request_error_journal: Arc<Mutex<RequestErrorJournal>>,
        // Failure-only parsed capacity samples. This is deliberately separate
        // from the qualifying receipt and is never written on success.
        interval_trace: Arc<Mutex<IntervalTrace>>,
        // This record is populated by the typed restart and cold-readback
        // paths. It is written only with a failed run's evidence bundle.
        restart_failure_trace: Arc<Mutex<Option<RestartFailureTrace>>>,
        readyz_readiness_trace: Arc<std::sync::Mutex<Option<ReadyzReadinessTrace>>>,
    }

    #[derive(Clone, Debug)]
    enum ReadyzReadinessTraceRow {
        HttpStatus { elapsed: Duration, status: u16 },
        ConnectError { elapsed: Duration },
        RequestTimeout { elapsed: Duration },
        TransportError { elapsed: Duration },
        Cancelled { elapsed: Duration },
    }

    #[derive(Clone, Debug)]
    struct ReadyzReadinessTrace {
        listener: String,
        rows: Vec<ReadyzReadinessTraceRow>,
        omitted_rows: u64,
        terminal: Option<&'static str>,
        last_elapsed: Option<Duration>,
    }

    impl Default for ReadyzReadinessTrace {
        fn default() -> Self {
            Self::new("post-restart-readyz")
        }
    }

    impl ReadyzReadinessTrace {
        fn new(listener: impl Into<String>) -> Self {
            Self {
                listener: listener.into(),
                rows: Vec::new(),
                omitted_rows: 0,
                terminal: None,
                last_elapsed: None,
            }
        }
        fn push(
            &mut self,
            elapsed: Duration,
            row: ReadyzReadinessTraceRow,
        ) -> std::result::Result<(), String> {
            if self.last_elapsed.is_some_and(|last| elapsed < last) {
                return Err("readiness poll elapsed time regressed".to_owned());
            }
            self.last_elapsed = Some(elapsed);
            if self.rows.len() < READYZ_READINESS_TRACE_MAX_ROWS {
                self.rows.push(row);
            } else {
                self.omitted_rows += 1;
            }
            Ok(())
        }
        fn record_nonready(
            &mut self,
            elapsed: Duration,
            status: u16,
        ) -> std::result::Result<(), String> {
            self.push(
                elapsed,
                ReadyzReadinessTraceRow::HttpStatus { elapsed, status },
            )
        }
        fn record_timeout(&mut self, elapsed: Duration) -> std::result::Result<(), String> {
            self.push(elapsed, ReadyzReadinessTraceRow::RequestTimeout { elapsed })
        }
        fn record_transport_error(&mut self, elapsed: Duration) -> std::result::Result<(), String> {
            self.push(elapsed, ReadyzReadinessTraceRow::TransportError { elapsed })
        }
        #[cfg(test)]
        fn record_transport(&mut self, elapsed: Duration) -> std::result::Result<(), String> {
            self.record_transport_error(elapsed)
        }
        #[cfg(test)]
        fn record_deadline(&mut self, elapsed: Duration) -> std::result::Result<(), String> {
            self.terminal = Some("request_timeout");
            self.record_timeout(elapsed)
        }
        fn record_cancelled(&mut self, elapsed: Duration) -> std::result::Result<(), String> {
            self.terminal = Some("cancelled");
            self.push(elapsed, ReadyzReadinessTraceRow::Cancelled { elapsed })
        }
        fn render_failure(&self) -> Option<String> {
            let terminal = self.terminal?;
            let mut out = format!(
                "schema_version=1\nphase=readyz\nlistener={}\nrestart_command=docker restart\n",
                self.listener
            );
            for (index, row) in self.rows.iter().enumerate() {
                match row {
                    ReadyzReadinessTraceRow::HttpStatus { elapsed, status } => {
                        out.push_str(&format!(
                            "poll={} elapsed_ms={} category=http_status:{}\n",
                            index + 1,
                            elapsed.as_millis(),
                            status
                        ))
                    }
                    ReadyzReadinessTraceRow::ConnectError { elapsed } => out.push_str(&format!(
                        "poll={} elapsed_ms={} category=connect_error\n",
                        index + 1,
                        elapsed.as_millis()
                    )),
                    ReadyzReadinessTraceRow::RequestTimeout { elapsed } => out.push_str(&format!(
                        "poll={} elapsed_ms={} category=request_timeout\n",
                        index + 1,
                        elapsed.as_millis()
                    )),
                    ReadyzReadinessTraceRow::TransportError { elapsed } => out.push_str(&format!(
                        "poll={} elapsed_ms={} category=transport_error\n",
                        index + 1,
                        elapsed.as_millis()
                    )),
                    ReadyzReadinessTraceRow::Cancelled { elapsed } => out.push_str(&format!(
                        "poll={} elapsed_ms={} category=cancelled\n",
                        index + 1,
                        elapsed.as_millis()
                    )),
                }
            }
            out.push_str(&format!(
                "polls_omitted={}\nterminal={}",
                self.omitted_rows, terminal
            ));
            Some(out)
        }
    }

    /// Private, best-effort timing for the restart path.  An absent value
    /// means that the phase never started or never reached readiness.
    #[derive(Clone, Copy, Debug, Default)]
    struct RestartPhaseDiagnostics {
        total_elapsed: Duration,
        restart_elapsed: Option<Duration>,
        port_lookup_elapsed: Option<Duration>,
        readyz_wait_elapsed: Option<Duration>,
        readiness_attempts: usize,
        first_ready_elapsed: Option<Duration>,
    }

    #[derive(Clone, Debug, Default)]
    struct RestartReadinessDiagnostics {
        readiness_failure: Option<RestartReadinessFailure>,
        docker_process_state: Option<DockerProcessState>,
    }

    #[derive(Clone, Debug)]
    enum RestartReadinessFailure {
        HttpStatus { status: String, body: String },
        Transport { error: String },
        Deadline,
    }

    #[derive(Clone, Debug)]
    struct DockerProcessState {
        detail: String,
    }

    #[derive(Clone, Copy, Debug)]
    enum RestartFailureTracePhase {
        DockerRestart,
        PublishedPort,
        Readyz,
        ColdReadback,
    }

    #[derive(Clone, Copy, Debug)]
    enum RestartFailureTraceOutcome {
        Success,
        Error,
        Timeout,
        RestartFailed,
    }

    #[derive(Clone, Copy, Debug)]
    enum RestartFailureTraceColdReadback {
        Pending,
        Succeeded,
        Failed,
        Unavailable,
    }

    /// Fixed-shape failure evidence for the restart and cold-readback path.
    /// It deliberately stores no command output, endpoint, status, or body.
    #[derive(Clone, Copy, Debug)]
    struct RestartFailureTrace {
        phase: RestartFailureTracePhase,
        outcome: RestartFailureTraceOutcome,
        total_elapsed_ms: u64,
        restart_elapsed_ms: Option<u64>,
        port_lookup_elapsed_ms: Option<u64>,
        readyz_wait_elapsed_ms: Option<u64>,
        readiness_attempts: u64,
        first_ready_elapsed_ms: Option<u64>,
        cold_readback: RestartFailureTraceColdReadback,
        cold_readback_outcome: Option<RestartFailureTraceOutcome>,
        cold_readback_elapsed_ms: Option<u64>,
    }

    impl RestartFailureTracePhase {
        fn wire_name(self) -> &'static str {
            match self {
                Self::DockerRestart => "docker_restart",
                Self::PublishedPort => "published_port",
                Self::Readyz => "readyz",
                Self::ColdReadback => "cold_readback",
            }
        }
    }

    impl RestartFailureTraceOutcome {
        fn wire_name(self) -> &'static str {
            match self {
                Self::Success => "success",
                Self::Error => "error",
                Self::Timeout => "timeout",
                Self::RestartFailed => "restart_failed",
            }
        }
    }

    impl RestartFailureTraceColdReadback {
        fn wire_name(self) -> &'static str {
            match self {
                Self::Pending => "pending",
                Self::Succeeded => "succeeded",
                Self::Failed => "failed",
                Self::Unavailable => "unavailable",
            }
        }
    }

    impl RestartFailureTrace {
        fn from_restart(
            phase: RestartFailureTracePhase,
            outcome: RestartFailureTraceOutcome,
            diagnostics: RestartPhaseDiagnostics,
            cold_readback: RestartFailureTraceColdReadback,
            cold_readback_outcome: Option<RestartFailureTraceOutcome>,
        ) -> Self {
            let elapsed_ms = |value: Option<Duration>| value.map(|value| value.as_millis() as u64);
            Self {
                phase,
                outcome,
                total_elapsed_ms: diagnostics.total_elapsed.as_millis() as u64,
                restart_elapsed_ms: elapsed_ms(diagnostics.restart_elapsed),
                port_lookup_elapsed_ms: elapsed_ms(diagnostics.port_lookup_elapsed),
                readyz_wait_elapsed_ms: elapsed_ms(diagnostics.readyz_wait_elapsed),
                readiness_attempts: diagnostics.readiness_attempts as u64,
                first_ready_elapsed_ms: elapsed_ms(diagnostics.first_ready_elapsed),
                cold_readback,
                cold_readback_outcome,
                cold_readback_elapsed_ms: None,
            }
        }

        fn with_cold_readback(
            mut self,
            cold_readback: RestartFailureTraceColdReadback,
            outcome: RestartFailureTraceOutcome,
            elapsed: Duration,
        ) -> Self {
            self.phase = RestartFailureTracePhase::ColdReadback;
            self.outcome = outcome;
            self.cold_readback = cold_readback;
            self.cold_readback_outcome = Some(outcome);
            self.cold_readback_elapsed_ms = Some(elapsed.as_millis() as u64);
            self
        }

        fn render(self) -> Result<String> {
            serde_json::to_string(&json!({
                "schema_version": 1,
                "kind": "lumen-perf-restart-cold-readback",
                "phase": self.phase.wire_name(),
                "outcome": self.outcome.wire_name(),
                "total_elapsed_ms": self.total_elapsed_ms,
                "restart_elapsed_ms": self.restart_elapsed_ms,
                "port_lookup_elapsed_ms": self.port_lookup_elapsed_ms,
                "readyz_wait_elapsed_ms": self.readyz_wait_elapsed_ms,
                "readiness_attempts": self.readiness_attempts,
                "first_ready_elapsed_ms": self.first_ready_elapsed_ms,
                "cold_readback": self.cold_readback.wire_name(),
                "cold_readback_outcome": self.cold_readback_outcome.map(RestartFailureTraceOutcome::wire_name),
                "cold_readback_elapsed_ms": self.cold_readback_elapsed_ms,
            }))
            .map_err(|error| HarnessError::DataInvariant(format!("cannot serialize restart failure trace: {error}")))
        }
    }

    fn bounded_restart_diagnostic_text(bytes: &[u8], total_bytes: u64, truncated: bool) -> String {
        format!(
            "bytes={total_bytes} truncated={truncated} text={:?}",
            String::from_utf8_lossy(bytes)
        )
    }

    fn bounded_restart_diagnostic_error(error: impl ToString) -> String {
        let text = error.to_string();
        let bytes = text.as_bytes();
        let retained = &bytes[..bytes.len().min(RESTART_DIAGNOSTIC_TEXT_MAX_BYTES)];
        bounded_restart_diagnostic_text(retained, bytes.len() as u64, retained.len() != bytes.len())
    }

    fn render_restart_phase_diagnostics(
        diagnostics: &RestartPhaseDiagnostics,
        readiness: &RestartReadinessDiagnostics,
        phase: &str,
        outcome: &str,
    ) -> String {
        let elapsed = |value: Option<Duration>| match value {
            Some(value) => value.as_millis().to_string(),
            None => "null".to_owned(),
        };
        let (readyz_failure, readyz_status, readyz_body, readyz_transport_error) =
            match readiness.readiness_failure.as_ref() {
                None => (
                    "none".to_owned(),
                    "null".to_owned(),
                    "null".to_owned(),
                    "null".to_owned(),
                ),
                Some(RestartReadinessFailure::HttpStatus { status, body }) => (
                    "http_status".to_owned(),
                    status.clone(),
                    body.clone(),
                    "null".to_owned(),
                ),
                Some(RestartReadinessFailure::Transport { error }) => (
                    "transport".to_owned(),
                    "null".to_owned(),
                    "null".to_owned(),
                    error.clone(),
                ),
                Some(RestartReadinessFailure::Deadline) => (
                    "deadline".to_owned(),
                    "null".to_owned(),
                    "null".to_owned(),
                    "null".to_owned(),
                ),
            };
        let docker_process_state = readiness
            .docker_process_state
            .as_ref()
            .map(|state| state.detail.as_str())
            .unwrap_or("null");
        format!(
            "{RESTART_DIAGNOSTIC_PREFIX} phase={phase} outcome={outcome} total_elapsed_ms={} restart_elapsed_ms={} port_lookup_elapsed_ms={} readyz_wait_elapsed_ms={} readiness_attempts={} first_ready_elapsed_ms={} readyz_failure={readyz_failure} readyz_status={readyz_status:?} readyz_body={readyz_body:?} readyz_transport_error={readyz_transport_error:?} docker_process_state={docker_process_state:?}",
            diagnostics.total_elapsed.as_millis(),
            elapsed(diagnostics.restart_elapsed),
            elapsed(diagnostics.port_lookup_elapsed),
            elapsed(diagnostics.readyz_wait_elapsed),
            diagnostics.readiness_attempts,
            elapsed(diagnostics.first_ready_elapsed),
        )
    }

    fn emit_restart_phase_diagnostics(
        diagnostics: &RestartPhaseDiagnostics,
        phase: &str,
        outcome: &str,
    ) -> String {
        emit_restart_phase_diagnostics_with_readiness(
            diagnostics,
            &RestartReadinessDiagnostics::default(),
            phase,
            outcome,
        )
    }

    fn emit_restart_phase_diagnostics_with_readiness(
        diagnostics: &RestartPhaseDiagnostics,
        readiness: &RestartReadinessDiagnostics,
        phase: &str,
        outcome: &str,
    ) -> String {
        let record = render_restart_phase_diagnostics(diagnostics, readiness, phase, outcome);
        eprintln!("{record}");
        record
    }

    trait CleanupCommandRunner {
        fn run_cleanup(&mut self, args: Vec<String>);
    }

    struct DockerCleanupCommandRunner;

    impl CleanupCommandRunner for DockerCleanupCommandRunner {
        fn run_cleanup(&mut self, args: Vec<String>) {
            if let Err(error) = run_command_with_timeout_blocking("docker", args, REQUEST_TIMEOUT) {
                eprintln!("PERF_FAILURE_CLEANUP_ERROR {error}");
            }
        }
    }

    fn cleanup_docker_resources<R: CleanupCommandRunner>(
        runner: &mut R,
        container: &str,
        volume: &str,
    ) {
        runner.run_cleanup(vec!["rm".to_owned(), "-f".to_owned(), container.to_owned()]);
        runner.run_cleanup(vec![
            "volume".to_owned(),
            "rm".to_owned(),
            "-f".to_owned(),
            volume.to_owned(),
        ]);
    }

    #[async_trait]
    trait EvidenceCommandRunner {
        async fn run(&mut self, args: Vec<String>) -> std::result::Result<String, String>;
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum EvidenceRetention {
        // Inspect and HTTP evidence starts with the request context.
        Prefix,
        // Docker service logs need the final records nearest the failed request.
        Tail,
    }

    #[derive(Debug)]
    struct BoundedEvidence {
        retained: Vec<u8>,
        total_bytes: u64,
        limit: usize,
        truncated: bool,
        retention: EvidenceRetention,
    }

    impl BoundedEvidence {
        fn new(limit: usize, retention: EvidenceRetention) -> Self {
            Self {
                retained: Vec::with_capacity(limit),
                total_bytes: 0,
                limit,
                truncated: false,
                retention,
            }
        }

        fn push(&mut self, bytes: &[u8]) {
            self.total_bytes = self.total_bytes.saturating_add(bytes.len() as u64);
            match self.retention {
                EvidenceRetention::Prefix => {
                    let remaining = self.limit.saturating_sub(self.retained.len());
                    let kept = remaining.min(bytes.len());
                    self.retained.extend_from_slice(&bytes[..kept]);
                    self.truncated |= kept < bytes.len();
                }
                EvidenceRetention::Tail => {
                    if bytes.len() >= self.limit {
                        self.retained.clear();
                        self.retained
                            .extend_from_slice(&bytes[bytes.len().saturating_sub(self.limit)..]);
                    } else {
                        let dropped = self
                            .retained
                            .len()
                            .saturating_add(bytes.len())
                            .saturating_sub(self.limit);
                        if dropped > 0 {
                            self.retained.drain(..dropped);
                        }
                        self.retained.extend_from_slice(bytes);
                    }
                    self.truncated |= self.total_bytes > self.retained.len() as u64;
                }
            }
        }

        fn render(&self, channel: &str) -> String {
            let channel_label = match self.retention {
                EvidenceRetention::Prefix => channel.to_owned(),
                EvidenceRetention::Tail => format!("{channel} tail"),
            };
            let mut rendered = format!(
                "[{channel_label}]\n{}",
                String::from_utf8_lossy(&self.retained)
            );
            if self.truncated {
                let retained_description = match self.retention {
                    EvidenceRetention::Prefix => "retaining",
                    EvidenceRetention::Tail => "retaining final",
                };
                rendered.push_str(&format!(
                    "\n[{channel_label} truncated after {retained_description} {} of at least {} bytes]\n",
                    self.retained.len(), self.total_bytes
                ));
            } else if !rendered.ends_with('\n') {
                rendered.push('\n');
            }
            rendered
        }
    }

    async fn read_bounded_reader<R: AsyncRead + Unpin>(
        mut reader: R,
        limit: usize,
        retention: EvidenceRetention,
    ) -> std::result::Result<BoundedEvidence, String> {
        let mut evidence = BoundedEvidence::new(limit, retention);
        let mut buffer = [0_u8; 8_192];
        loop {
            let count = reader
                .read(&mut buffer)
                .await
                .map_err(|error| format!("cannot drain evidence stream: {error}"))?;
            if count == 0 {
                return Ok(evidence);
            }
            evidence.push(&buffer[..count]);
        }
    }

    async fn read_bounded_chunks<Stream, Chunk, Error, Describe>(
        stream: Stream,
        limit: usize,
        describe_error: Describe,
    ) -> std::result::Result<BoundedEvidence, String>
    where
        Stream: futures::Stream<Item = std::result::Result<Chunk, Error>>,
        Chunk: AsRef<[u8]>,
        Describe: Fn(Error) -> String,
    {
        let mut evidence = BoundedEvidence::new(limit, EvidenceRetention::Prefix);
        futures::pin_mut!(stream);
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|error| describe_error(error))?;
            evidence.push(chunk.as_ref());
        }
        Ok(evidence)
    }

    fn render_command_streams(stdout: &BoundedEvidence, stderr: &BoundedEvidence) -> String {
        format!("{}{}", stdout.render("stdout"), stderr.render("stderr"))
    }

    fn command_result(
        rendered: &str,
        succeeded: bool,
        status: &str,
        stdout: BoundedEvidence,
        stderr: BoundedEvidence,
    ) -> std::result::Result<String, String> {
        let streams = render_command_streams(&stdout, &stderr);
        if succeeded {
            Ok(streams)
        } else {
            Err(format!(
                "docker {rendered} exited with {status}:\n{streams}"
            ))
        }
    }

    async fn collect_docker_output(
        mut child: tokio::process::Child,
        retention: EvidenceRetention,
    ) -> std::result::Result<(std::process::ExitStatus, BoundedEvidence, BoundedEvidence), String>
    {
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "docker evidence command has no stdout pipe".to_owned())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "docker evidence command has no stderr pipe".to_owned())?;
        let (status, stdout, stderr) = tokio::join!(
            child.wait(),
            read_bounded_reader(stdout, EVIDENCE_COMMAND_CHANNEL_MAX_BYTES, retention),
            read_bounded_reader(stderr, EVIDENCE_COMMAND_CHANNEL_MAX_BYTES, retention),
        );
        Ok((
            status.map_err(|error| format!("cannot wait for docker evidence command: {error}"))?,
            stdout?,
            stderr?,
        ))
    }

    fn docker_evidence_retention(args: &[String]) -> EvidenceRetention {
        match args.first().map(String::as_str) {
            Some("logs") => EvidenceRetention::Tail,
            _ => EvidenceRetention::Prefix,
        }
    }

    struct DockerEvidenceCommandRunner;

    #[async_trait]
    impl EvidenceCommandRunner for DockerEvidenceCommandRunner {
        async fn run(&mut self, args: Vec<String>) -> std::result::Result<String, String> {
            let rendered = args.join(" ");
            let retention = docker_evidence_retention(&args);
            let mut command = tokio::process::Command::new("docker");
            command.args(&args);
            command.stdout(Stdio::piped());
            command.stderr(Stdio::piped());
            command.kill_on_drop(true);
            let (status, stdout, stderr) = tokio::time::timeout(REQUEST_TIMEOUT, async {
                let child = command
                    .spawn()
                    .map_err(|error| format!("cannot start docker {rendered}: {error}"))?;
                collect_docker_output(child, retention).await
            })
            .await
            .map_err(|_| {
                format!(
                    "docker {rendered} exceeded the {}-second evidence collection deadline",
                    REQUEST_TIMEOUT.as_secs()
                )
            })??;
            command_result(
                &rendered,
                status.success(),
                &status.to_string(),
                stdout,
                stderr,
            )
        }
    }

    #[derive(Debug)]
    struct FailureEvidenceDirectory {
        path: PathBuf,
    }

    impl FailureEvidenceDirectory {
        fn create(
            container: &str,
            volume: &str,
            error: &HarnessError,
        ) -> std::result::Result<Self, String> {
            let evidence = Self::create_in(&env::temp_dir(), container, volume, error)?;
            eprintln!(
                "PERF_FAILURE_EVIDENCE path={} container={} volume={}",
                evidence.path.display(),
                container,
                volume
            );
            Ok(evidence)
        }

        fn create_in(
            root: &Path,
            container: &str,
            volume: &str,
            error: &HarnessError,
        ) -> std::result::Result<Self, String> {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            for attempt in 0..100 {
                let path = root.join(format!(
                    "lumen-perf-failure-{}-{}-{}",
                    std::process::id(),
                    nonce,
                    attempt
                ));
                match fs::create_dir(&path) {
                    Ok(()) => {
                        let evidence = Self { path };
                        evidence.write_text("failure.txt", &failure_report(error))?;
                        evidence.write_text("container.txt", container)?;
                        evidence.write_text("volume.txt", volume)?;
                        return Ok(evidence);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(error) => {
                        return Err(format!(
                            "cannot create failure evidence directory {}: {error}",
                            path.display()
                        ));
                    }
                }
            }
            Err("cannot allocate a unique failure evidence directory".to_owned())
        }

        fn write_text(&self, name: &str, content: &str) -> std::result::Result<(), String> {
            fs::write(self.path.join(name), bounded_evidence_text(content)).map_err(|error| {
                format!(
                    "cannot write failure evidence {}: {error}",
                    self.path.join(name).display()
                )
            })
        }

        fn record_result(&self, name: &str, result: std::result::Result<String, String>) {
            let content = match result {
                Ok(content) => content,
                Err(error) => format!("ERROR: {error}\n"),
            };
            if let Err(error) = self.write_text(name, &content) {
                eprintln!("PERF_FAILURE_EVIDENCE_WRITE_ERROR {error}");
            }
        }

        async fn record_container<R: EvidenceCommandRunner + Send>(
            &self,
            runner: &mut R,
            container: &str,
        ) {
            self.record_result(
                "docker-logs.txt",
                runner
                    .run(vec![
                        "logs".to_owned(),
                        "--tail".to_owned(),
                        EVIDENCE_LOG_TAIL_LINES.to_owned(),
                        container.to_owned(),
                    ])
                    .await,
            );
            self.record_result(
                "docker-inspect.json",
                runner
                    .run(vec!["inspect".to_owned(), container.to_owned()])
                    .await,
            );
        }
    }

    fn bounded_evidence_text(content: &str) -> String {
        if content.len() <= EVIDENCE_ARTIFACT_MAX_BYTES {
            return content.to_owned();
        }
        format!(
            "{}\n[truncated after {} bytes]\n",
            String::from_utf8_lossy(&content.as_bytes()[..EVIDENCE_ARTIFACT_MAX_BYTES]),
            EVIDENCE_ARTIFACT_MAX_BYTES
        )
    }

    fn request_failure_report(detail: &RequestFailure) -> String {
        let mut report = format!(
            "request_display={}\nrequest_timeout={}\nrequest_connect={}\nrequest_request={}\nrequest_body={}\nrequest_decode={}\nrequest_context={}\nrequest_source_chain:\n",
            detail.display,
            detail.is_timeout,
            detail.is_connect,
            detail.is_request,
            detail.is_body,
            detail.is_decode,
            detail.context.as_deref().unwrap_or("none")
        );
        for (index, source) in detail.source_chain.iter().enumerate() {
            report.push_str(&format!("{index}: {source}\n"));
        }
        report
    }

    fn failure_report(error: &HarnessError) -> String {
        let mut report = format!("original_error_display={error}\n");
        match error {
            HarnessError::RequestFailure(detail) => report.push_str(&request_failure_report(detail)),
            _ => report.push_str(
                "request_timeout=unavailable\nrequest_connect=unavailable\nrequest_request=unavailable\nrequest_body=unavailable\nrequest_decode=unavailable\nrequest_context=unavailable\n",
            ),
        }
        report
    }

    fn request_error_for_evidence(error: reqwest::Error) -> String {
        request_failure_report(&RequestFailure::from_reqwest(error))
    }

    /// One workload request that ended `Outcome::Failed` or
    /// `Outcome::TimedOut`, retained with its full classified error so a
    /// failed durable cell's evidence bundle can name why every counted
    /// `RequestErrors`/`TimedOut` outcome happened instead of only the last
    /// error the harness happened to observe.
    #[derive(Debug, Clone)]
    struct RequestErrorRecord {
        elapsed_since_clock_start: Duration,
        endpoint: String,
        identifier: String,
        error: RequestFailure,
        status: Option<u16>,
        body: Option<String>,
    }

    fn render_request_error_record(index: usize, record: &RequestErrorRecord) -> String {
        let mut report = format!(
            "record={index} elapsed_ms={} endpoint={} identifier={} status={} timeout={} connect={} request={} body_err={} decode={}\n",
            record.elapsed_since_clock_start.as_millis(),
            record.endpoint,
            record.identifier,
            record
                .status
                .map(|status| status.to_string())
                .unwrap_or_else(|| "none".to_owned()),
            record.error.is_timeout,
            record.error.is_connect,
            record.error.is_request,
            record.error.is_body,
            record.error.is_decode,
        );
        report.push_str(&format!("  display={}\n", record.error.display));
        for (chain_index, source) in record.error.source_chain.iter().enumerate() {
            report.push_str(&format!("  chain[{chain_index}]={source}\n"));
        }
        if let Some(body) = &record.body {
            report.push_str(&format!("  response_body={body}\n"));
        }
        report
    }

    /// Bounded, shared record of every classified workload request failure
    /// observed during one durable cell. Capped independently of the ledger
    /// so a pathological run cannot grow the evidence bundle without limit;
    /// entries past the cap are only counted, in `overflow`.
    const REQUEST_ERROR_JOURNAL_CAP: usize = 256;
    // The response body captured alongside a non-2xx workload failure is
    // untrusted server output; cap it well below the whole-artifact bound.
    const REQUEST_ERROR_BODY_MAX_BYTES: usize = 512;

    #[derive(Debug, Default)]
    struct RequestErrorJournal {
        records: Vec<RequestErrorRecord>,
        overflow: u64,
        interval_failures: BTreeMap<u64, IntervalFailureCounts>,
    }

    impl RequestErrorJournal {
        fn push(&mut self, record: RequestErrorRecord) {
            self.interval_failures
                .entry(
                    record.elapsed_since_clock_start.as_millis() as u64
                        / INTERVAL_TRACE_CADENCE.as_millis() as u64,
                )
                .or_default()
                .record(&record);
            if self.records.len() < REQUEST_ERROR_JOURNAL_CAP {
                self.records.push(record);
            } else {
                self.overflow += 1;
            }
        }
    }

    fn render_request_error_journal(journal: &RequestErrorJournal) -> String {
        if journal.records.is_empty() && journal.overflow == 0 {
            return "no workload request failures were observed\n".to_owned();
        }
        let mut report = String::new();
        for (index, record) in journal.records.iter().enumerate() {
            report.push_str(&render_request_error_record(index, record));
        }
        if journal.overflow > 0 {
            report.push_str(&format!("overflow={}\n", journal.overflow));
        }
        report
    }

    /// Compact failure-only capacity evidence. This never enters a receipt or
    /// changes a gate. It stores parsed counters only, never a metrics body or
    /// a request identifier.
    #[derive(Debug, Default)]
    struct IntervalTrace {
        samples: Vec<IntervalTraceSample>,
        previous: Option<IntervalTraceSnapshot>,
    }

    #[derive(Debug)]
    struct IntervalTraceSample {
        elapsed_ms: u64,
        phase: &'static str,
        scrape_state: &'static str,
        snapshot: Option<IntervalTraceSnapshot>,
        delta: Option<IntervalTraceDelta>,
    }

    #[derive(Debug, Clone)]
    struct IntervalTraceSnapshot {
        reserved: u64,
        active: u64,
        frozen: u64,
        total: u64,
        checkpoints: u64,
        checkpoint_attempt_started: u64,
        checkpoint_attempt_in_flight: u64,
        checkpoint_attempt_failed: u64,
        merges: u64,
        state_write_lock: IntervalHistogram,
        hnsw_add: IntervalHistogram,
    }

    #[derive(Debug)]
    struct IntervalTraceDelta {
        checkpoints: u64,
        checkpoint_attempt_started: u64,
        checkpoint_attempt_failed: u64,
        merges: u64,
        state_write_lock: IntervalHistogram,
        hnsw_add: IntervalHistogram,
    }

    #[derive(Debug, Clone)]
    struct IntervalHistogram {
        count: u64,
        sum_us: u64,
        /// Prometheus renders cumulative finite buckets. Keep the same fixed
        /// labels in the trace so a consumer need not see raw metric text.
        buckets: [u64; INTERVAL_TRACE_BUCKET_LABELS.len()],
    }

    #[derive(Debug, Default)]
    struct IntervalFailureCounts {
        http_429: u64,
        other_http: u64,
        timeout: u64,
        other_transport: u64,
    }

    impl IntervalFailureCounts {
        fn record(&mut self, record: &RequestErrorRecord) {
            match record.status {
                Some(429) => self.http_429 += 1,
                Some(_) => self.other_http += 1,
                None if record.error.is_timeout => self.timeout += 1,
                None => self.other_transport += 1,
            }
        }

        fn json(&self) -> Value {
            json!({
                "http_429": self.http_429,
                "other_http": self.other_http,
                "timeout": self.timeout,
                "other_transport": self.other_transport,
            })
        }
    }

    impl IntervalTrace {
        fn record(&mut self, elapsed: Duration, phase: &'static str, metrics: &str) {
            if self.samples.len() >= INTERVAL_TRACE_MAX_SAMPLES {
                return;
            }
            let elapsed_ms = elapsed.as_millis() as u64;
            match IntervalTraceSnapshot::parse(metrics) {
                Ok(snapshot) => {
                    let delta = self.previous.as_ref().and_then(|before| {
                        IntervalTraceDelta::from_snapshots(&snapshot, before).ok()
                    });
                    // A backwards cumulative observation is diagnostic data,
                    // not a reason to move an existing workload gate.
                    let scrape_state = if self.previous.is_some() && delta.is_none() {
                        "backwards"
                    } else {
                        "ok"
                    };
                    if scrape_state == "ok" {
                        self.previous = Some(snapshot.clone());
                    }
                    self.samples.push(IntervalTraceSample {
                        elapsed_ms,
                        phase,
                        scrape_state,
                        snapshot: Some(snapshot),
                        delta,
                    });
                }
                Err(_) => self.samples.push(IntervalTraceSample {
                    elapsed_ms,
                    phase,
                    scrape_state: "parse_error",
                    snapshot: None,
                    delta: None,
                }),
            }
        }

        fn render(&self, journal: &RequestErrorJournal) -> Result<String> {
            self.render_bounded(journal, INTERVAL_TRACE_MAX_BYTES)
        }

        #[cfg(test)]
        fn render_with_byte_cap(
            &self,
            journal: &RequestErrorJournal,
            max_bytes: usize,
        ) -> Result<String> {
            self.render_bounded(journal, max_bytes)
        }

        fn render_bounded(
            &self,
            journal: &RequestErrorJournal,
            max_bytes: usize,
        ) -> Result<String> {
            let mut samples = self
                .samples
                .iter()
                .map(interval_trace_sample_json)
                .collect::<Vec<_>>();
            let mut error_sample_indices = BTreeMap::new();
            for (bucket, failures) in &journal.interval_failures {
                // The post-seed baseline and the first input scrape can both
                // be in bucket zero. A bucket's failures belong to exactly one
                // sample: prefer the input sample, then any sole sample.
                let existing = samples
                    .iter()
                    .position(|sample| {
                        sample["phase"] == "input"
                            && sample["elapsed_ms"].as_u64().unwrap_or_default()
                                / INTERVAL_TRACE_CADENCE.as_millis() as u64
                                == *bucket
                    })
                    .or_else(|| {
                        samples.iter().position(|sample| {
                            sample["elapsed_ms"].as_u64().unwrap_or_default()
                                / INTERVAL_TRACE_CADENCE.as_millis() as u64
                                == *bucket
                        })
                    });
                let index = match existing {
                    Some(index) => index,
                    None => {
                        samples.push(json!({
                            "elapsed_ms": bucket * INTERVAL_TRACE_CADENCE.as_millis() as u64,
                            "phase": "input",
                            "scrape_state": "not_sampled",
                            "pending": Value::Null,
                            "checkpoint": Value::Null,
                            "checkpoint_attempt": Value::Null,
                            "merge": Value::Null,
                            "state_write_lock": Value::Null,
                            "state_write_lock_delta": Value::Null,
                            "hnsw_add": Value::Null,
                            "hnsw_add_delta": Value::Null,
                            "request_failures": IntervalFailureCounts::default().json(),
                        }));
                        samples.len() - 1
                    }
                };
                samples[index]["request_failures"] = failures.json();
                error_sample_indices.insert(*bucket, index);
            }

            let mut omitted_request_failure_intervals = Vec::new();
            while samples.len() > INTERVAL_TRACE_MAX_SAMPLES {
                let ordinary = samples.iter().rposition(|sample| {
                    let bucket = sample["elapsed_ms"].as_u64().unwrap_or_default()
                        / INTERVAL_TRACE_CADENCE.as_millis() as u64;
                    sample["phase"] != "pre_input" && !error_sample_indices.contains_key(&bucket)
                });
                let index = ordinary.unwrap_or(samples.len() - 1);
                let sample = samples.remove(index);
                let bucket = sample["elapsed_ms"].as_u64().unwrap_or_default()
                    / INTERVAL_TRACE_CADENCE.as_millis() as u64;
                if request_failure_value_is_nonzero(&sample["request_failures"]) {
                    // A bounded document can drop metric detail, but never the
                    // only aggregate for a counted failure interval.
                    omitted_request_failure_intervals.push(json!({
                        "interval": bucket,
                        "request_failures": sample["request_failures"],
                    }));
                }
            }
            loop {
                let document = json!({
                    "schema_version": 3,
                    "kind": "lumen-perf-interval-trace",
                    "cadence_ms": INTERVAL_TRACE_CADENCE.as_millis() as u64,
                    "histogram_bucket_labels": INTERVAL_TRACE_BUCKET_LABELS,
                    "samples": samples,
                    "omitted_request_failure_intervals": omitted_request_failure_intervals,
                });
                let encoded = serde_json::to_string(&document).map_err(|error| {
                    HarnessError::DataInvariant(format!("cannot serialize interval trace: {error}"))
                })?;
                if encoded.len() <= max_bytes || samples.is_empty() {
                    return Ok(encoded);
                }
                let sample = samples.pop().expect("non-empty checked above");
                let bucket = sample["elapsed_ms"].as_u64().unwrap_or_default()
                    / INTERVAL_TRACE_CADENCE.as_millis() as u64;
                if request_failure_value_is_nonzero(&sample["request_failures"]) {
                    omitted_request_failure_intervals.push(json!({
                        "interval": bucket,
                        "request_failures": sample["request_failures"],
                    }));
                }
            }
        }
    }

    impl IntervalTraceSnapshot {
        fn parse(metrics: &str) -> Result<Self> {
            Ok(Self {
                reserved: prometheus_counter(metrics, "lumen_pending_change_reserved_bytes")?,
                active: prometheus_counter(metrics, "lumen_pending_change_active_bytes")?,
                frozen: prometheus_counter(metrics, "lumen_pending_change_frozen_bytes")?,
                total: prometheus_counter(metrics, "lumen_pending_change_total_bytes")?,
                checkpoints: prometheus_counter(metrics, CHECKPOINT_COUNTER)?,
                checkpoint_attempt_started: prometheus_counter(metrics, CHECKPOINT_ATTEMPT_STARTED_COUNTER)?,
                checkpoint_attempt_in_flight: prometheus_counter(metrics, CHECKPOINT_ATTEMPT_IN_FLIGHT)?,
                checkpoint_attempt_failed: prometheus_counter(metrics, CHECKPOINT_ATTEMPT_FAILED_COUNTER)?,
                merges: prometheus_counter(metrics, MERGE_COUNTER)?,
                state_write_lock: IntervalHistogram::parse(metrics, STATE_WRITE_LOCK_HISTOGRAM)?,
                hnsw_add: IntervalHistogram::parse(metrics, HNSW_ADD_HISTOGRAM)?,
            })
        }
    }

    impl IntervalTraceDelta {
        fn from_snapshots(
            after: &IntervalTraceSnapshot,
            before: &IntervalTraceSnapshot,
        ) -> Result<Self> {
            Ok(Self {
                checkpoints: counter_delta(
                    CHECKPOINT_COUNTER,
                    after.checkpoints,
                    before.checkpoints,
                )?,
                checkpoint_attempt_started: counter_delta(
                    CHECKPOINT_ATTEMPT_STARTED_COUNTER,
                    after.checkpoint_attempt_started,
                    before.checkpoint_attempt_started,
                )?,
                checkpoint_attempt_failed: counter_delta(
                    CHECKPOINT_ATTEMPT_FAILED_COUNTER,
                    after.checkpoint_attempt_failed,
                    before.checkpoint_attempt_failed,
                )?,
                merges: counter_delta(MERGE_COUNTER, after.merges, before.merges)?,
                state_write_lock: after.state_write_lock.delta(&before.state_write_lock)?,
                hnsw_add: after.hnsw_add.delta(&before.hnsw_add)?,
            })
        }
    }

    impl IntervalHistogram {
        fn parse(metrics: &str, name: &'static str) -> Result<Self> {
            let mut buckets = [0; INTERVAL_TRACE_BUCKET_LABELS.len()];
            for (index, label) in INTERVAL_TRACE_BUCKET_LABELS.iter().enumerate() {
                buckets[index] = prometheus_histogram_value(metrics, name, label)?;
                if index > 0 && buckets[index] < buckets[index - 1] {
                    return Err(HarnessError::DataInvariant(format!(
                        "histogram buckets moved backwards: {name} le={label}"
                    )));
                }
            }
            let count = prometheus_histogram_scalar(metrics, name, "count")?
                .parse::<u64>()
                .map_err(|_| HarnessError::MetricParse {
                    name,
                    value: name.to_owned(),
                })?;
            let raw_sum = prometheus_histogram_scalar(metrics, name, "sum")?;
            let sum_seconds = raw_sum
                .parse::<f64>()
                .map_err(|_| HarnessError::MetricParse {
                    name,
                    value: raw_sum.to_owned(),
                })?;
            let sum_us = sum_seconds * 1_000_000.0;
            if !sum_us.is_finite() || sum_us < 0.0 || sum_us > u64::MAX as f64 {
                return Err(HarnessError::MetricParse {
                    name,
                    value: raw_sum.to_owned(),
                });
            }
            let infinity = prometheus_histogram_value(metrics, name, "+Inf")?;
            if infinity != count || buckets.last().is_some_and(|last| *last > count) {
                return Err(HarnessError::DataInvariant(format!(
                    "histogram count does not match +Inf bucket: {name}"
                )));
            }
            Ok(Self {
                count,
                sum_us: sum_us.round() as u64,
                buckets,
            })
        }

        fn delta(&self, before: &Self) -> Result<Self> {
            let mut buckets = [0; INTERVAL_TRACE_BUCKET_LABELS.len()];
            for (index, bucket) in buckets.iter_mut().enumerate() {
                *bucket = counter_delta(
                    "interval histogram bucket",
                    self.buckets[index],
                    before.buckets[index],
                )?;
            }
            Ok(Self {
                count: counter_delta("interval histogram count", self.count, before.count)?,
                sum_us: counter_delta("interval histogram sum_us", self.sum_us, before.sum_us)?,
                buckets,
            })
        }

        fn json(&self, buckets_are_deltas: bool) -> Value {
            let bucket_values = INTERVAL_TRACE_BUCKET_LABELS
                .iter()
                .zip(self.buckets)
                .map(|(label, value)| ((*label).to_owned(), Value::from(value)))
                .collect::<Map<_, _>>();
            let mut object = Map::new();
            object.insert("count".to_owned(), Value::from(self.count));
            object.insert("sum_us".to_owned(), Value::from(self.sum_us));
            object.insert(
                if buckets_are_deltas {
                    "bucket_deltas".to_owned()
                } else {
                    "bucket_totals".to_owned()
                },
                Value::Object(bucket_values),
            );
            Value::Object(object)
        }
    }

    fn prometheus_histogram_value(metrics: &str, name: &'static str, label: &str) -> Result<u64> {
        let metric = format!("{name}_bucket{{le=\"{label}\"}}");
        let mut values = metrics.lines().filter_map(|line| {
            let (candidate, value) =
                line.split_once(|character: char| character.is_whitespace())?;
            (candidate == metric).then_some(value.trim())
        });
        let value = values
            .next()
            .ok_or(HarnessError::MissingRuntimeMetric(name))?;
        if values.next().is_some() {
            return Err(HarnessError::DuplicateRuntimeMetric(name));
        }
        value.parse::<u64>().map_err(|_| HarnessError::MetricParse {
            name,
            value: value.to_owned(),
        })
    }

    fn prometheus_histogram_scalar<'a>(
        metrics: &'a str,
        name: &'static str,
        suffix: &str,
    ) -> Result<&'a str> {
        let metric = format!("{name}_{suffix}");
        let mut values = metrics.lines().filter_map(|line| {
            let (candidate, value) =
                line.split_once(|character: char| character.is_whitespace())?;
            (candidate == metric).then_some(value.trim())
        });
        let value = values
            .next()
            .ok_or(HarnessError::MissingRuntimeMetric(name))?;
        if values.next().is_some() {
            return Err(HarnessError::DuplicateRuntimeMetric(name));
        }
        if value.is_empty() {
            return Err(HarnessError::MetricParse {
                name,
                value: value.to_owned(),
            });
        }
        Ok(value)
    }

    fn interval_trace_sample_json(sample: &IntervalTraceSample) -> Value {
        let snapshot = sample.snapshot.as_ref();
        let delta = sample.delta.as_ref();
        json!({
            "elapsed_ms": sample.elapsed_ms,
            "phase": sample.phase,
            "scrape_state": sample.scrape_state,
            "pending": snapshot.map(|value| json!({"reserved_bytes": value.reserved, "active_bytes": value.active, "frozen_bytes": value.frozen, "total_bytes": value.total})),
            "checkpoint": snapshot.map(|value| json!({"total": value.checkpoints, "delta": delta.map(|value| value.checkpoints).unwrap_or(0)})),
            "checkpoint_attempt": snapshot.map(|value| json!({
                "started_total": value.checkpoint_attempt_started,
                "started_delta": delta.map(|value| value.checkpoint_attempt_started).unwrap_or(0),
                "in_flight": value.checkpoint_attempt_in_flight,
                "failed_total": value.checkpoint_attempt_failed,
                "failed_delta": delta.map(|value| value.checkpoint_attempt_failed).unwrap_or(0),
            })),
            "merge": snapshot.map(|value| json!({"total": value.merges, "delta": delta.map(|value| value.merges).unwrap_or(0)})),
            "state_write_lock": snapshot.map(|value| value.state_write_lock.json(false)),
            "state_write_lock_delta": delta.map(|value| value.state_write_lock.json(true)),
            "hnsw_add": snapshot.map(|value| value.hnsw_add.json(false)),
            "hnsw_add_delta": delta.map(|value| value.hnsw_add.json(true)),
            "request_failures": IntervalFailureCounts::default().json(),
        })
    }

    fn request_failure_value_is_nonzero(value: &Value) -> bool {
        ["http_429", "other_http", "timeout", "other_transport"]
            .iter()
            .any(|name| value[*name].as_u64().unwrap_or_default() > 0)
    }

    fn truncate_evidence_body(body: &str) -> String {
        if body.len() <= REQUEST_ERROR_BODY_MAX_BYTES {
            return body.to_owned();
        }
        format!(
            "{}...[truncated after {} bytes]",
            String::from_utf8_lossy(&body.as_bytes()[..REQUEST_ERROR_BODY_MAX_BYTES]),
            REQUEST_ERROR_BODY_MAX_BYTES
        )
    }

    /// The classified failure `post_json` (and the `send_unindex` inline
    /// transport call) returns instead of the bare [`Outcome`] the ledger
    /// still records. `outcome` is exactly the value the caller would have
    /// computed before this change, so ledger counts never move; only the
    /// evidence carried alongside them is new.
    #[derive(Debug, Clone)]
    struct PostJsonFailure {
        error: RequestFailure,
        outcome: Outcome,
        status: Option<u16>,
        body: Option<String>,
    }

    /// Appends one classified failure to the shared journal and returns the
    /// [`Outcome`] the ledger must still record — the one call site every
    /// workload request failure funnels through on its way into evidence.
    async fn record_request_error(
        journal: &Arc<Mutex<RequestErrorJournal>>,
        elapsed_since_clock_start: Duration,
        endpoint: impl Into<String>,
        identifier: impl Into<String>,
        failure: PostJsonFailure,
    ) -> Outcome {
        let outcome = failure.outcome;
        journal.lock().await.push(RequestErrorRecord {
            elapsed_since_clock_start,
            endpoint: endpoint.into(),
            identifier: identifier.into(),
            error: failure.error,
            status: failure.status,
            body: failure.body,
        });
        outcome
    }

    #[async_trait]
    trait FailureProbe {
        async fn metrics(&mut self) -> std::result::Result<String, String>;
        async fn hot_stats(&mut self) -> std::result::Result<String, String>;
    }

    struct DockerFailureProbe<'a> {
        server: &'a DockerLumen,
    }

    #[async_trait]
    impl<'a> FailureProbe for DockerFailureProbe<'a> {
        async fn metrics(&mut self) -> std::result::Result<String, String> {
            self.server.evidence_get("/metrics").await
        }

        async fn hot_stats(&mut self) -> std::result::Result<String, String> {
            self.server
                .evidence_get(&format!("/collections/{HOT_COLLECTION}/stats"))
                .await
        }
    }

    struct UnavailableFailureProbe;

    #[async_trait]
    impl FailureProbe for UnavailableFailureProbe {
        async fn metrics(&mut self) -> std::result::Result<String, String> {
            Err("HTTP client was not constructed during startup".to_owned())
        }

        async fn hot_stats(&mut self) -> std::result::Result<String, String> {
            Err("HTTP client was not constructed during startup".to_owned())
        }
    }

    async fn finish_failed_docker_run<
        Evidence: EvidenceCommandRunner + Send,
        Probe: FailureProbe + Send,
        Cleanup: CleanupCommandRunner,
    >(
        evidence: &FailureEvidenceDirectory,
        evidence_runner: &mut Evidence,
        probe: &mut Probe,
        cleanup_runner: &mut Cleanup,
        container: &str,
        volume: &str,
        request_error_report: &str,
        interval_trace: Option<&str>,
    ) {
        finish_failed_docker_run_with_restart_trace(
            evidence,
            evidence_runner,
            probe,
            cleanup_runner,
            container,
            volume,
            request_error_report,
            interval_trace,
            None,
        )
        .await;
    }

    async fn finish_failed_docker_run_with_restart_trace<
        Evidence: EvidenceCommandRunner + Send,
        Probe: FailureProbe + Send,
        Cleanup: CleanupCommandRunner,
    >(
        evidence: &FailureEvidenceDirectory,
        evidence_runner: &mut Evidence,
        probe: &mut Probe,
        cleanup_runner: &mut Cleanup,
        container: &str,
        volume: &str,
        request_error_report: &str,
        interval_trace: Option<&str>,
        restart_failure_trace: Option<&str>,
    ) {
        evidence.record_container(evidence_runner, container).await;
        evidence.record_result("metrics.txt", probe.metrics().await);
        evidence.record_result("hot-stats.json", probe.hot_stats().await);
        eprintln!("PERF_REQUEST_ERRORS {request_error_report}");
        if let Err(error) = evidence.write_text("request-errors.txt", request_error_report) {
            eprintln!("PERF_FAILURE_EVIDENCE_WRITE_ERROR {error}");
        }
        if let Some(interval_trace) = interval_trace {
            if let Err(error) = evidence.write_text("interval-trace.json", interval_trace) {
                eprintln!("PERF_FAILURE_EVIDENCE_WRITE_ERROR {error}");
            }
        }
        if let Some(restart_failure_trace) = restart_failure_trace {
            if let Err(error) =
                evidence.write_text("restart-failure-trace.json", restart_failure_trace)
            {
                eprintln!("PERF_FAILURE_EVIDENCE_WRITE_ERROR {error}");
            }
        }
        cleanup_docker_resources(cleanup_runner, container, volume);
    }

    /// Owns Docker cleanup while startup has not yet produced a [`DockerLumen`].
    /// This closes failures after `docker run` but before port discovery or
    /// readiness completes.
    struct DockerCleanup {
        container: String,
        volume: String,
        armed: bool,
    }

    impl DockerCleanup {
        fn new(container: String, volume: String) -> Self {
            Self {
                container,
                volume,
                armed: true,
            }
        }

        fn disarm(&mut self) {
            self.armed = false;
        }

        async fn finish_failure(&mut self, error: &HarnessError) {
            let evidence =
                match FailureEvidenceDirectory::create(&self.container, &self.volume, error) {
                    Ok(evidence) => evidence,
                    Err(capture_error) => {
                        eprintln!("PERF_FAILURE_EVIDENCE_ERROR {capture_error}");
                        return;
                    }
                };
            let mut runner = DockerEvidenceCommandRunner;
            let mut probe = UnavailableFailureProbe;
            let mut cleanup_runner = DockerCleanupCommandRunner;
            finish_failed_docker_run(
                &evidence,
                &mut runner,
                &mut probe,
                &mut cleanup_runner,
                &self.container,
                &self.volume,
                "no request-error journal: failure occurred before the workload client existed\n",
                None,
            )
            .await;
            self.armed = false;
        }
    }

    impl Drop for DockerCleanup {
        fn drop(&mut self) {
            if self.armed {
                let mut runner = DockerCleanupCommandRunner;
                cleanup_docker_resources(&mut runner, &self.container, &self.volume);
            }
        }
    }

    fn docker_run_args(
        container: &str,
        volume: &str,
        image: &str,
        recovery_profile: Option<&str>,
        diagnostic: bool,
    ) -> Vec<String> {
        let resource_args = [
            "--cpus",
            DOCKER_CPUS,
            "--memory",
            DOCKER_MEMORY,
            "--memory-swap",
            DOCKER_MEMORY,
        ];
        let mut args = vec![
            "run".to_owned(),
            "-d".to_owned(),
            "--rm".to_owned(),
            "--name".to_owned(),
            container.to_owned(),
            "--mount".to_owned(),
            format!("type=volume,src={volume},dst=/var/lib/lumen/data"),
            "-e".to_owned(),
            "LUMEN_AUTH=off".to_owned(),
            "-e".to_owned(),
            "LUMEN_WAL=embedded".to_owned(),
            "-e".to_owned(),
            "LUMEN_PERSISTENCE=segment".to_owned(),
            "-e".to_owned(),
            "LUMEN_DATA_DIR=/var/lib/lumen/data".to_owned(),
            "-e".to_owned(),
            format!("LUMEN_SNAPSHOT_SECS={SNAPSHOT_SECONDS}"),
        ];
        let insert_at = 5;
        args.splice(
            insert_at..insert_at,
            resource_args.iter().map(|arg| (*arg).to_owned()),
        );
        if let Some(recovery_profile) = recovery_profile {
            args.extend([
                "-e".to_owned(),
                format!("LUMEN_RECOVERY_PROFILE={recovery_profile}"),
            ]);
        }
        if diagnostic {
            args.extend([
                "-e".to_owned(),
                "LUMEN_PERF_DIAGNOSTIC=1".to_owned(),
                "-e".to_owned(),
                "LUMEN_LOG_FORMAT=json".to_owned(),
                "-e".to_owned(),
                "RUST_LOG=info,http.access=warn".to_owned(),
            ]);
        }
        args.extend([
            "-p".to_owned(),
            "127.0.0.1::7373".to_owned(),
            image.to_owned(),
        ]);
        args
    }

    fn docker_owned(args: &[String]) -> Result<String> {
        let borrowed = args.iter().map(String::as_str).collect::<Vec<_>>();
        docker(&borrowed)
    }

    #[test]
    fn docker_run_forwards_recovery_profile_only_when_set() {
        let without_profile = docker_run_args("container", "volume", "image", None, false);
        assert!(
            !without_profile
                .iter()
                .any(|arg| arg.starts_with("LUMEN_RECOVERY_PROFILE=")),
            "an unset recovery profile must not alter Docker's environment"
        );

        let with_profile = docker_run_args("container", "volume", "image", Some("1"), false);
        assert!(
            with_profile
                .windows(2)
                .any(|pair| pair == ["-e", "LUMEN_RECOVERY_PROFILE=1"]),
            "an explicitly set recovery profile must reach docker run"
        );
    }

    #[test]
    fn docker_run_forwards_diagnostic_environment_only_for_diagnostic_mode() {
        let diagnostic = docker_run_args("container", "volume", "image", None, true);
        for environment in [
            "LUMEN_PERF_DIAGNOSTIC=1",
            "LUMEN_LOG_FORMAT=json",
            "RUST_LOG=info,http.access=warn",
        ] {
            assert!(
                diagnostic
                    .windows(2)
                    .any(|pair| pair == ["-e", environment]),
                "diagnostic mode must forward {environment}"
            );
        }

        let qualifying = docker_run_args("container", "volume", "image", None, false);
        for environment in [
            "LUMEN_PERF_DIAGNOSTIC=1",
            "LUMEN_LOG_FORMAT=json",
            "RUST_LOG=info,http.access=warn",
        ] {
            assert!(
                !qualifying
                    .windows(2)
                    .any(|pair| pair == ["-e", environment]),
                "qualifying mode must not forward {environment}"
            );
        }
    }

    #[test]
    fn recovery_observation_is_bounded_and_diagnostic_only() {
        assert_eq!(
            diagnostic_recovery_observation(Some("120"), true).unwrap(),
            Some(Duration::from_secs(120))
        );
        assert_eq!(
            diagnostic_recovery_observation(Some("31"), true).unwrap(),
            Some(Duration::from_secs(31))
        );
        for value in ["30", "121", "0", "invalid"] {
            assert!(
                diagnostic_recovery_observation(Some(value), true).is_err(),
                "{value} must not extend the diagnostic observation window"
            );
        }
        assert!(diagnostic_recovery_observation(Some("120"), false).is_err());
    }

    #[test]
    fn diagnostic_restart_extends_only_its_outer_watchdog() {
        let shared_deadline = tokio::time::Instant::now() + POST_INPUT_TIMEOUT;
        let (qualifying_deadline, qualifying_timeout) =
            restart_post_input_window(shared_deadline, None).unwrap();
        assert_eq!(qualifying_deadline, shared_deadline);
        assert_eq!(qualifying_timeout, POST_INPUT_TIMEOUT);

        let (diagnostic_deadline, diagnostic_timeout) =
            restart_post_input_window(shared_deadline, Some(Duration::from_secs(120))).unwrap();
        let extension = Duration::from_secs(120) - STARTUP_TIMEOUT;
        assert_eq!(diagnostic_deadline, shared_deadline + extension);
        assert_eq!(diagnostic_timeout, POST_INPUT_TIMEOUT + extension);
    }

    impl DockerLumen {
        async fn start() -> Result<Self> {
            let image = required_env("LUMEN_PERF_IMAGE")?;
            if !is_immutable_image_reference(&image) {
                return Err(HarnessError::UnpinnedImage(image));
            }
            let diagnostic = env::var("LUMEN_PERF_DIAGNOSTIC").ok().as_deref() == Some("1");
            let recovery_observation = diagnostic_recovery_observation(
                optional_env(RECOVERY_OBSERVATION_ENV)?.as_deref(),
                diagnostic,
            )?;
            let nonce = format!(
                "{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|error| HarnessError::Startup(error.to_string()))?
                    .as_nanos()
            );
            let container = format!("lumen-perf-{nonce}");
            let volume = format!("lumen-perf-data-{nonce}");
            docker(&["volume", "create", &volume])?;
            let mut cleanup = DockerCleanup::new(container.clone(), volume.clone());
            let mut server = match (|| -> Result<Self> {
                let run_args = docker_run_args(
                    &container,
                    &volume,
                    &image,
                    env::var("LUMEN_RECOVERY_PROFILE").ok().as_deref(),
                    diagnostic,
                );
                docker_owned(&run_args)?;
                let image_id = verify_image_identity(&image)?;
                let port_output = docker(&["port", &container, "7373/tcp"])?;
                let base = published_loopback_base(&port_output)?;
                Ok(Self {
                    container: container.clone(),
                    volume: volume.clone(),
                    base,
                    client: reqwest::Client::builder()
                        .timeout(REQUEST_TIMEOUT)
                        .build()
                        .map_err(HarnessError::request_failure)?,
                    image_reference: image.clone(),
                    image_id,
                    cleanup_armed: true,
                    recovery_observation,
                    request_error_journal: Arc::new(Mutex::new(RequestErrorJournal::default())),
                    interval_trace: Arc::new(Mutex::new(IntervalTrace::default())),
                    restart_failure_trace: Arc::new(Mutex::new(None)),
                    readyz_readiness_trace: Arc::new(std::sync::Mutex::new(None)),
                })
            })() {
                Ok(server) => server,
                Err(error) => {
                    cleanup.finish_failure(&error).await;
                    return Err(error);
                }
            };
            cleanup.disarm();
            if let Err(error) = server.assert_docker_limits() {
                server.finish_failure(&error).await;
                return Err(error);
            }
            if let Err(error) = server.wait_ready().await {
                server.finish_failure(&error).await;
                return Err(error);
            }
            eprintln!(
                "approved workload image reference={} actual_image_id={}",
                server.image_reference, server.image_id
            );
            Ok(server)
        }

        fn assert_docker_limits(&self) -> Result<()> {
            let output = docker(&[
                "inspect",
                "--format",
                "{{.HostConfig.NanoCpus}} {{.HostConfig.Memory}}",
                &self.container,
            ])?;
            let values = output.split_whitespace().collect::<Vec<_>>();
            let expected_cpu = "2500000000";
            let expected_memory = DOCKER_MEMORY_BYTES.to_string();
            if values.as_slice() != [expected_cpu, expected_memory.as_str()] {
                return Err(HarnessError::DockerLimits(format!(
                    "expected nano_cpus={} memory_bytes={}; got {output:?}",
                    expected_cpu, expected_memory
                )));
            }
            Ok(())
        }

        async fn wait_ready(&self) -> Result<()> {
            self.wait_ready_until(Instant::now() + STARTUP_TIMEOUT)
                .await
        }

        async fn wait_ready_until(&self, deadline: Instant) -> Result<()> {
            loop {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    return Err(HarnessError::Startup(format!(
                        "GET /readyz did not become successful within {} seconds; retained docker-logs.txt has the bounded container tail",
                        STARTUP_TIMEOUT.as_secs()
                    )));
                }
                if let Ok(Ok(response)) = tokio::time::timeout(
                    remaining,
                    self.client.get(format!("{}/readyz", self.base)).send(),
                )
                .await
                {
                    if response.status().is_success() {
                        return Ok(());
                    }
                }
                if Instant::now() >= deadline {
                    return Err(HarnessError::Startup(format!(
                        "GET /readyz did not become successful within {} seconds; retained docker-logs.txt has the bounded container tail",
                        STARTUP_TIMEOUT.as_secs()
                    )));
                }
                tokio::time::sleep(
                    Duration::from_millis(100)
                        .min(deadline.saturating_duration_since(Instant::now())),
                )
                .await;
            }
        }

        async fn metrics(&self) -> Result<String> {
            let (status, text) = fetch_interval_metrics(
                &self.client,
                format!("{}/metrics", self.base),
                REQUEST_TIMEOUT,
            )
            .await?;
            if !status.is_success() {
                return Err(HarnessError::Http(format!(
                    "GET /metrics returned {status}: {text}"
                )));
            }
            Ok(text)
        }

        async fn evidence_get(&self, path: &str) -> std::result::Result<String, String> {
            tokio::time::timeout(REQUEST_TIMEOUT, async {
                let response = self
                    .client
                    .get(format!("{}{}", self.base, path))
                    .send()
                    .await
                    .map_err(request_error_for_evidence)?;
                let status = response.status();
                let body = read_bounded_chunks(
                    response.bytes_stream(),
                    EVIDENCE_HTTP_BODY_MAX_BYTES,
                    request_error_for_evidence,
                )
                .await?;
                Ok(format!("status={status}\n{}", body.render("body")))
            })
            .await
            .map_err(|_| {
                format!(
                    "GET {path} exceeded the {}-second evidence collection deadline",
                    REQUEST_TIMEOUT.as_secs()
                )
            })?
        }

        async fn finish_failure(&mut self, error: &HarnessError) {
            let mut evidence_runner = DockerEvidenceCommandRunner;
            let mut cleanup_runner = DockerCleanupCommandRunner;
            self.finish_failure_with(
                error,
                &env::temp_dir(),
                &mut evidence_runner,
                &mut cleanup_runner,
            )
            .await;
        }

        async fn finish_failure_with<E, C>(
            &mut self,
            error: &HarnessError,
            evidence_root: &Path,
            evidence_runner: &mut E,
            cleanup_runner: &mut C,
        ) -> Option<PathBuf>
        where
            E: EvidenceCommandRunner + Send,
            C: CleanupCommandRunner,
        {
            let evidence = match FailureEvidenceDirectory::create_in(
                evidence_root,
                &self.container,
                &self.volume,
                error,
            ) {
                Ok(evidence) => evidence,
                Err(capture_error) => {
                    eprintln!("PERF_FAILURE_EVIDENCE_ERROR {capture_error}");
                    return None;
                }
            };
            eprintln!(
                "PERF_FAILURE_EVIDENCE path={} container={} volume={}",
                evidence.path.display(),
                self.container,
                self.volume
            );
            let request_error_report =
                render_request_error_journal(&*self.request_error_journal.lock().await);
            let interval_trace = {
                let trace = self.interval_trace.lock().await;
                let journal = self.request_error_journal.lock().await;
                trace.render(&journal).map_err(|error| error.to_string())
            };
            if let Err(error) = &interval_trace {
                eprintln!("PERF_FAILURE_EVIDENCE_WRITE_ERROR {error}");
            }
            let restart_failure_trace = self
                .restart_failure_trace
                .lock()
                .await
                .as_ref()
                .copied()
                .map(RestartFailureTrace::render)
                .transpose();
            if let Err(error) = &restart_failure_trace {
                eprintln!("PERF_FAILURE_EVIDENCE_WRITE_ERROR {error}");
            }
            let readyz_readiness_trace = {
                let mut guard = self
                    .readyz_readiness_trace
                    .lock()
                    .expect("readyz trace mutex");
                if matches!(error, HarnessError::PostInputTimeout { .. }) {
                    if let Some(trace) = guard.as_mut().filter(|trace| trace.terminal.is_none()) {
                        let elapsed =
                            trace.last_elapsed.unwrap_or_default() + Duration::from_millis(1);
                        let _ = trace.record_cancelled(elapsed);
                    }
                }
                guard
                    .as_ref()
                    .and_then(ReadyzReadinessTrace::render_failure)
            };
            if let Some(trace) = readyz_readiness_trace.as_deref() {
                if let Err(error) = evidence.write_text(READYZ_READINESS_TRACE_FILE, trace) {
                    eprintln!("PERF_FAILURE_EVIDENCE_WRITE_ERROR {error}");
                }
            }
            let mut probe = DockerFailureProbe { server: self };
            finish_failed_docker_run_with_restart_trace(
                &evidence,
                evidence_runner,
                &mut probe,
                cleanup_runner,
                &self.container,
                &self.volume,
                &request_error_report,
                interval_trace.as_deref().ok(),
                restart_failure_trace
                    .as_ref()
                    .ok()
                    .and_then(|trace| trace.as_deref()),
            )
            .await;
            drop(probe);
            self.cleanup_armed = false;
            Some(evidence.path)
        }

        fn record_readyz_readiness_failure(&self, elapsed: Duration, row: ReadyzReadinessTraceRow) {
            let mut guard = self
                .readyz_readiness_trace
                .lock()
                .expect("readyz trace mutex");
            let trace = guard.get_or_insert_with(ReadyzReadinessTrace::default);
            if let Err(error) = trace.push(elapsed, row) {
                eprintln!("PERF_READYZ_TRACE_ERROR {error}");
            }
        }

        fn record_readyz_readiness_deadline(&self, elapsed: Duration) {
            let mut guard = self
                .readyz_readiness_trace
                .lock()
                .expect("readyz trace mutex");
            let trace = guard.get_or_insert_with(ReadyzReadinessTrace::default);
            trace.terminal = Some("request_timeout");
            if let Err(error) = trace.record_timeout(elapsed) {
                eprintln!("PERF_READYZ_TRACE_ERROR {error}");
            }
        }

        fn clear_readyz_readiness_trace(&self) {
            *self
                .readyz_readiness_trace
                .lock()
                .expect("readyz trace mutex") = None;
        }

        async fn restart_and_wait_ready(&mut self) -> Result<Duration> {
            self.restart_and_wait_ready_with(&mut DockerRestartCommandRunner)
                .await
        }

        /// Runs the restart inside the one post-input deadline.  Keeping this
        /// as one seam lets the case finalizer observe the real timeout result
        /// after the restart future is dropped.
        async fn post_input_restart_step(
            &mut self,
            deadline: tokio::time::Instant,
        ) -> Result<Duration> {
            let (deadline, timeout) =
                restart_post_input_window(deadline, self.recovery_observation)?;
            post_input_step(deadline, timeout, "restart", self.restart_and_wait_ready()).await
        }

        async fn post_input_restart_step_with<R: RestartCommandRunner + Send>(
            &mut self,
            deadline: tokio::time::Instant,
            timeout: Duration,
            runner: &mut R,
        ) -> Result<Duration> {
            post_input_step(
                deadline,
                timeout,
                "restart",
                self.restart_and_wait_ready_with(runner),
            )
            .await
        }

        async fn restart_and_wait_ready_with<R: RestartCommandRunner + Send>(
            &mut self,
            runner: &mut R,
        ) -> Result<Duration> {
            self.restart_and_wait_ready_with_diagnostic_observer(
                runner,
                Instant::now() + STARTUP_TIMEOUT,
                |_, _| {},
            )
            .await
        }

        async fn record_restart_failure_trace(
            &self,
            phase: RestartFailureTracePhase,
            outcome: RestartFailureTraceOutcome,
            diagnostics: RestartPhaseDiagnostics,
        ) {
            *self.restart_failure_trace.lock().await = Some(RestartFailureTrace::from_restart(
                phase,
                outcome,
                diagnostics,
                RestartFailureTraceColdReadback::Unavailable,
                Some(RestartFailureTraceOutcome::RestartFailed),
            ));
        }

        async fn record_restart_success_trace(&self, diagnostics: RestartPhaseDiagnostics) {
            *self.restart_failure_trace.lock().await = Some(RestartFailureTrace::from_restart(
                RestartFailureTracePhase::Readyz,
                RestartFailureTraceOutcome::Success,
                diagnostics,
                RestartFailureTraceColdReadback::Pending,
                None,
            ));
        }

        async fn record_cold_readback_trace(&self, succeeded: bool, elapsed: Duration) {
            let mut trace = self.restart_failure_trace.lock().await;
            let Some(existing) = *trace else {
                return;
            };
            *trace = Some(existing.with_cold_readback(
                if succeeded {
                    RestartFailureTraceColdReadback::Succeeded
                } else {
                    RestartFailureTraceColdReadback::Failed
                },
                if succeeded {
                    RestartFailureTraceOutcome::Success
                } else {
                    RestartFailureTraceOutcome::Error
                },
                elapsed,
            ));
        }

        async fn observe_recovery_after_readyz_timeout(&self, started: Instant) {
            let Some(limit) = self.recovery_observation else {
                return;
            };
            let deadline = started + limit;
            let mut attempts = 0_u64;
            while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
                if remaining.is_zero() {
                    break;
                }
                attempts += 1;
                if let Ok(Ok(response)) = tokio::time::timeout(
                    remaining.min(REQUEST_TIMEOUT),
                    self.client.get(format!("{}/readyz", self.base)).send(),
                )
                .await
                {
                    if response.status().is_success() {
                        eprintln!(
                            "PERF_RECOVERY_OBSERVATION outcome=ready observed_elapsed_ms={} attempts={} official_timeout_ms={} observation_limit_ms={}",
                            started.elapsed().as_millis(), attempts, STARTUP_TIMEOUT.as_millis(), limit.as_millis()
                        );
                        return;
                    }
                }
                tokio::time::sleep(
                    Duration::from_millis(100)
                        .min(deadline.saturating_duration_since(Instant::now())),
                )
                .await;
            }
            eprintln!(
                "PERF_RECOVERY_OBSERVATION outcome=not_ready observed_elapsed_ms={} attempts={} official_timeout_ms={} observation_limit_ms={}",
                started.elapsed().as_millis(), attempts, STARTUP_TIMEOUT.as_millis(), limit.as_millis()
            );
        }

        async fn refresh_restart_endpoint<R: RestartCommandRunner>(
            &mut self,
            runner: &mut R,
            deadline: Instant,
        ) -> Result<()> {
            // Docker can assign another random host port on restart. Resolve
            // the same container again before any readiness or cold readback.
            // Lookup and readiness share the existing startup deadline.
            let remaining = deadline.saturating_duration_since(Instant::now());
            tokio::time::timeout(remaining, async {
                let output = runner.published_port(&self.container).await?;
                if Instant::now() >= deadline {
                    return Err(HarnessError::Startup(
                        "published-port lookup exhausted the startup budget".to_owned(),
                    ));
                }
                let base = published_loopback_base(&output)?;
                eprintln!(
                    "PERF_RESTART_ENDPOINT container={} previous={} current={}",
                    self.container, self.base, base
                );
                self.base = base;
                self.wait_ready_until(deadline).await
            })
            .await
            .map_err(|_| {
                HarnessError::Startup(format!(
                    "published-port refresh and readiness exceeded the remaining {remaining:?} startup budget"
                ))
            })?
        }

        async fn restart_and_wait_ready_with_diagnostic_observer<R, Observe>(
            &mut self,
            runner: &mut R,
            deadline: Instant,
            mut observe: Observe,
        ) -> Result<Duration>
        where
            R: RestartCommandRunner + Send,
            Observe: FnMut(&str, RestartPhaseDiagnostics),
        {
            let started = Instant::now();
            let mut diagnostics = RestartPhaseDiagnostics::default();
            let mut readiness = RestartReadinessDiagnostics::default();
            let mut emit = |diagnostics: &mut RestartPhaseDiagnostics,
                            readiness: &RestartReadinessDiagnostics,
                            phase: &'static str,
                            outcome: &'static str| {
                diagnostics.total_elapsed = started.elapsed();
                let record = if readiness.readiness_failure.is_some() {
                    emit_restart_phase_diagnostics_with_readiness(
                        diagnostics,
                        readiness,
                        phase,
                        outcome,
                    )
                } else {
                    let record = emit_restart_phase_diagnostics(diagnostics, phase, outcome);
                    record
                };
                observe(&record, *diagnostics);
            };

            let restart_started = Instant::now();
            let remaining = deadline.saturating_duration_since(Instant::now());
            match tokio::time::timeout(remaining, runner.restart(&self.container)).await {
                Err(_) => {
                    diagnostics.restart_elapsed = Some(restart_started.elapsed());
                    emit(&mut diagnostics, &readiness, "docker-restart", "timeout");
                    self.record_restart_failure_trace(
                        RestartFailureTracePhase::DockerRestart,
                        RestartFailureTraceOutcome::Timeout,
                        diagnostics,
                    )
                    .await;
                    return Err(HarnessError::Startup(
                        "restart command exhausted the total restart deadline".to_owned(),
                    ));
                }
                Ok(Err(error)) => {
                    diagnostics.restart_elapsed = Some(restart_started.elapsed());
                    emit(&mut diagnostics, &readiness, "docker-restart", "error");
                    self.record_restart_failure_trace(
                        RestartFailureTracePhase::DockerRestart,
                        RestartFailureTraceOutcome::Error,
                        diagnostics,
                    )
                    .await;
                    return Err(error);
                }
                Ok(Ok(())) => {
                    diagnostics.restart_elapsed = Some(restart_started.elapsed());
                }
            }

            let port_lookup_started = Instant::now();
            let remaining = deadline.saturating_duration_since(Instant::now());
            let output =
                match tokio::time::timeout(remaining, runner.published_port(&self.container)).await
                {
                    Err(_) => {
                        diagnostics.port_lookup_elapsed = Some(port_lookup_started.elapsed());
                        emit(&mut diagnostics, &readiness, "published-port", "timeout");
                        self.record_restart_failure_trace(
                            RestartFailureTracePhase::PublishedPort,
                            RestartFailureTraceOutcome::Timeout,
                            diagnostics,
                        )
                        .await;
                        return Err(HarnessError::Startup(
                            "published-port refresh and readiness exhausted the startup budget"
                                .to_owned(),
                        ));
                    }
                    Ok(Err(error)) => {
                        diagnostics.port_lookup_elapsed = Some(port_lookup_started.elapsed());
                        emit(&mut diagnostics, &readiness, "published-port", "error");
                        self.record_restart_failure_trace(
                            RestartFailureTracePhase::PublishedPort,
                            RestartFailureTraceOutcome::Error,
                            diagnostics,
                        )
                        .await;
                        return Err(error);
                    }
                    Ok(Ok(output)) => {
                        diagnostics.port_lookup_elapsed = Some(port_lookup_started.elapsed());
                        output
                    }
                };
            let base = match published_loopback_base(&output) {
                Ok(base) => base,
                Err(error) => {
                    emit(&mut diagnostics, &readiness, "published-port", "error");
                    self.record_restart_failure_trace(
                        RestartFailureTracePhase::PublishedPort,
                        RestartFailureTraceOutcome::Error,
                        diagnostics,
                    )
                    .await;
                    return Err(error);
                }
            };
            eprintln!(
                "PERF_RESTART_ENDPOINT container={} previous={} current={}",
                self.container, self.base, base
            );
            self.base = base;

            let readyz_wait_started = Instant::now();
            loop {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    self.record_readyz_readiness_deadline(started.elapsed());
                    diagnostics.readyz_wait_elapsed = Some(readyz_wait_started.elapsed());
                    readiness.readiness_failure = Some(RestartReadinessFailure::Deadline);
                    readiness.docker_process_state = Some(DockerProcessState {
                        detail: "deadline_before_process_inspect".to_owned(),
                    });
                    emit(&mut diagnostics, &readiness, "readyz", "timeout");
                    self.record_restart_failure_trace(
                        RestartFailureTracePhase::Readyz,
                        RestartFailureTraceOutcome::Timeout,
                        diagnostics,
                    )
                    .await;
                    self.observe_recovery_after_readyz_timeout(started).await;
                    return Err(HarnessError::Startup(format!(
                        "GET /readyz did not become successful within {} seconds; retained docker-logs.txt has the bounded container tail",
                        STARTUP_TIMEOUT.as_secs()
                    )));
                }
                diagnostics.readiness_attempts += 1;
                match tokio::time::timeout(
                    remaining,
                    self.client.get(format!("{}/readyz", self.base)).send(),
                )
                .await
                {
                    Ok(Ok(response)) if response.status().is_success() => {
                        self.clear_readyz_readiness_trace();
                        diagnostics.readyz_wait_elapsed = Some(readyz_wait_started.elapsed());
                        #[rustfmt::skip]
                        diagnostics.first_ready_elapsed.get_or_insert_with(|| started.elapsed());
                        emit(&mut diagnostics, &readiness, "complete", "success");
                        self.record_restart_success_trace(diagnostics).await;
                        return Ok(started.elapsed());
                    }
                    Ok(Ok(response)) => {
                        let elapsed = started.elapsed();
                        self.record_readyz_readiness_failure(
                            elapsed,
                            ReadyzReadinessTraceRow::HttpStatus {
                                elapsed,
                                status: response.status().as_u16(),
                            },
                        );
                        let status = response.status().to_string();
                        let body_deadline = deadline.saturating_duration_since(Instant::now());
                        readiness.readiness_failure = Some(
                            match tokio::time::timeout(
                                body_deadline,
                                read_bounded_chunks(
                                    response.bytes_stream(),
                                    RESTART_READINESS_BODY_MAX_BYTES,
                                    bounded_restart_diagnostic_error,
                                ),
                            )
                            .await
                            {
                                Ok(Ok(body)) => RestartReadinessFailure::HttpStatus {
                                    status,
                                    body: bounded_restart_diagnostic_text(
                                        &body.retained,
                                        body.total_bytes,
                                        body.truncated,
                                    ),
                                },
                                Ok(Err(error)) => RestartReadinessFailure::Transport { error },
                                Err(_) => RestartReadinessFailure::Deadline,
                            },
                        );
                    }
                    Ok(Err(error)) => {
                        let elapsed = started.elapsed();
                        self.record_readyz_readiness_failure(
                            elapsed,
                            if error.is_connect() {
                                ReadyzReadinessTraceRow::ConnectError { elapsed }
                            } else {
                                ReadyzReadinessTraceRow::TransportError { elapsed }
                            },
                        );
                        readiness.readiness_failure = Some(RestartReadinessFailure::Transport {
                            error: bounded_restart_diagnostic_error(error),
                        });
                    }
                    Err(_) => {
                        let elapsed = started.elapsed();
                        self.record_readyz_readiness_failure(
                            elapsed,
                            ReadyzReadinessTraceRow::RequestTimeout { elapsed },
                        );
                        readiness.readiness_failure = Some(RestartReadinessFailure::Deadline);
                    }
                }
                if readiness.docker_process_state.is_none() {
                    let remaining = deadline.saturating_duration_since(Instant::now());
                    readiness.docker_process_state = Some(
                        match tokio::time::timeout(remaining, runner.process_state(&self.container))
                            .await
                        {
                            Ok(state) => state,
                            Err(_) => DockerProcessState {
                                detail: "process_inspect_exhausted_readiness_deadline".to_owned(),
                            },
                        },
                    );
                }
                if Instant::now() >= deadline {
                    self.record_readyz_readiness_deadline(started.elapsed());
                    diagnostics.readyz_wait_elapsed = Some(readyz_wait_started.elapsed());
                    emit(&mut diagnostics, &readiness, "readyz", "timeout");
                    self.record_restart_failure_trace(
                        RestartFailureTracePhase::Readyz,
                        RestartFailureTraceOutcome::Timeout,
                        diagnostics,
                    )
                    .await;
                    self.observe_recovery_after_readyz_timeout(started).await;
                    return Err(HarnessError::Startup(format!(
                        "GET /readyz did not become successful within {} seconds; retained docker-logs.txt has the bounded container tail",
                        STARTUP_TIMEOUT.as_secs()
                    )));
                }
                tokio::time::sleep(
                    Duration::from_millis(100)
                        .min(deadline.saturating_duration_since(Instant::now())),
                )
                .await;
            }
        }
    }

    fn published_loopback_base(output: &str) -> Result<String> {
        let port = output
            .trim()
            .strip_prefix("127.0.0.1:")
            .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
            .and_then(|value| value.parse::<u16>().ok())
            .filter(|port| *port != 0)
            .ok_or_else(|| {
                HarnessError::Startup(format!(
                    "cannot parse one published loopback port: {output:?}"
                ))
            })?;
        Ok(format!("http://127.0.0.1:{port}"))
    }

    #[async_trait]
    trait RestartCommandRunner {
        async fn restart(&mut self, container: &str) -> Result<()>;
        async fn published_port(&mut self, container: &str) -> Result<String>;

        async fn process_state(&mut self, _container: &str) -> DockerProcessState {
            DockerProcessState {
                detail: "not_collected".to_owned(),
            }
        }
    }

    struct DockerRestartCommandRunner;

    #[async_trait]
    impl RestartCommandRunner for DockerRestartCommandRunner {
        async fn restart(&mut self, container: &str) -> Result<()> {
            run_command_with_timeout("docker", &["restart", container], STARTUP_TIMEOUT)
                .await
                .map(|_| ())
        }

        async fn published_port(&mut self, container: &str) -> Result<String> {
            let args = vec![
                "port".to_owned(),
                container.to_owned(),
                "7373/tcp".to_owned(),
            ];
            let failure = |detail: String| HarnessError::Command {
                program: "docker",
                args: args.clone(),
                detail,
            };
            let mut command = tokio::process::Command::new("docker");
            command
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);
            let (status, stdout, stderr) = tokio::time::timeout(REQUEST_TIMEOUT, async {
                let child = command.spawn().map_err(|error| error.to_string())?;
                collect_docker_output(child, EvidenceRetention::Prefix).await
            })
            .await
            .map_err(|_| failure("published-port lookup exceeded request deadline".to_owned()))?
            .map_err(&failure)?;
            if !status.success() || stdout.truncated {
                return Err(failure(format!(
                    "published-port lookup status={status}: {}",
                    render_command_streams(&stdout, &stderr)
                )));
            }
            String::from_utf8(stdout.retained).map_err(|error| failure(error.to_string()))
        }

        async fn process_state(&mut self, container: &str) -> DockerProcessState {
            let args = vec![
                "inspect".to_owned(),
                "--format".to_owned(),
                "{{.State.Running}}\\t{{.State.ExitCode}}\\t{{.State.OOMKilled}}\\t{{.State.Error}}"
                    .to_owned(),
                container.to_owned(),
            ];
            let mut command = tokio::process::Command::new("docker");
            command
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);
            let result = tokio::time::timeout(REQUEST_TIMEOUT, async {
                let child = command.spawn().map_err(|error| error.to_string())?;
                collect_docker_output(child, EvidenceRetention::Prefix).await
            })
            .await;
            let (status, stdout, _stderr) = match result {
                Ok(Ok(values)) => values,
                Ok(Err(error)) => {
                    return DockerProcessState {
                        detail: format!(
                            "inspect_error={}",
                            bounded_restart_diagnostic_error(error)
                        ),
                    };
                }
                Err(_) => {
                    return DockerProcessState {
                        detail: "inspect_timeout".to_owned(),
                    };
                }
            };
            if !status.success() || stdout.truncated {
                return DockerProcessState {
                    detail: format!(
                        "inspect_command_status={status} output={}",
                        bounded_restart_diagnostic_text(
                            &stdout.retained,
                            stdout.total_bytes,
                            stdout.truncated,
                        )
                    ),
                };
            }
            let output = String::from_utf8_lossy(&stdout.retained);
            let mut fields = output.trim_end().splitn(4, '\t');
            let (Some(running), Some(exit_code), Some(oom_killed), Some(error)) =
                (fields.next(), fields.next(), fields.next(), fields.next())
            else {
                return DockerProcessState {
                    detail: format!(
                        "inspect_unparseable={}",
                        bounded_restart_diagnostic_text(
                            &stdout.retained,
                            stdout.total_bytes,
                            stdout.truncated,
                        )
                    ),
                };
            };
            DockerProcessState {
                detail: format!(
                    "running={running} exit_code={exit_code} oom_killed={oom_killed} error={}",
                    bounded_restart_diagnostic_error(error),
                ),
            }
        }
    }

    impl Drop for DockerLumen {
        fn drop(&mut self) {
            if self.cleanup_armed {
                let mut runner = DockerCleanupCommandRunner;
                cleanup_docker_resources(&mut runner, &self.container, &self.volume);
            }
        }
    }

    fn docker(args: &[&str]) -> Result<String> {
        let output = Command::new("docker")
            .args(args)
            .output()
            .map_err(|error| HarnessError::Command {
                program: "docker",
                args: args.iter().map(|arg| (*arg).to_owned()).collect(),
                detail: error.to_string(),
            })?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
        } else {
            Err(HarnessError::Command {
                program: "docker",
                args: args.iter().map(|arg| (*arg).to_owned()).collect(),
                detail: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            })
        }
    }

    async fn run_command_with_timeout(
        program: &'static str,
        args: &[&str],
        timeout: Duration,
    ) -> Result<String> {
        let args = args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>();
        tokio::task::spawn_blocking(move || {
            run_command_with_timeout_blocking(program, args, timeout)
        })
        .await
        .map_err(|error| HarnessError::Task(format!("{program} timeout worker failed: {error}")))?
    }

    fn run_command_with_timeout_blocking(
        program: &'static str,
        args: Vec<String>,
        timeout: Duration,
    ) -> Result<String> {
        let rendered_args = args.clone();
        let mut child = Command::new(program)
            .args(&args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| HarnessError::Command {
                program,
                args: rendered_args.clone(),
                detail: error.to_string(),
            })?;

        let deadline = Instant::now() + timeout;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() >= deadline => {
                    let kill = child.kill();
                    let reap = child.wait();
                    return Err(HarnessError::Command {
                        program,
                        args: rendered_args,
                        detail: format!(
                            "timed out after {timeout:?}; kill={}; reap={}",
                            format_kill_result(kill),
                            format_reap_result(reap),
                        ),
                    });
                }
                Ok(None) => {
                    let remaining = deadline.saturating_duration_since(Instant::now());
                    std::thread::sleep(remaining.min(Duration::from_millis(10)));
                }
                Err(error) => {
                    let kill = child.kill();
                    let reap = child.wait();
                    return Err(HarnessError::Command {
                        program,
                        args: rendered_args,
                        detail: format!(
                            "cannot poll child: {error}; kill={}; reap={}",
                            format_kill_result(kill),
                            format_reap_result(reap),
                        ),
                    });
                }
            }
        };
        if status.success() {
            Ok(String::new())
        } else {
            Err(HarnessError::Command {
                program,
                args: rendered_args,
                detail: status.to_string(),
            })
        }
    }

    fn format_kill_result(result: std::io::Result<()>) -> String {
        match result {
            Ok(()) => "ok".to_owned(),
            Err(error) => format!("error: {error}"),
        }
    }

    fn format_reap_result(result: std::io::Result<std::process::ExitStatus>) -> String {
        match result {
            Ok(status) => format!("ok ({status})"),
            Err(error) => format!("error: {error}"),
        }
    }

    #[test]
    fn restart_command_timeout_kills_and_reaps_a_stuck_child() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build restart-timeout regression runtime");
        runtime.block_on(async {
            let started = Instant::now();
            let error = run_command_with_timeout(
                "sh",
                &["-c", "exec sleep 30"],
                Duration::from_millis(100),
            )
            .await
            .expect_err("a stuck restart command must hit its explicit timeout");
            assert!(
                started.elapsed() < Duration::from_secs(2),
                "the timeout regression must return promptly, elapsed={:?}",
                started.elapsed()
            );
            let rendered = error.to_string();
            assert!(
                rendered.contains("timed out after 100ms"),
                "timeout failure must name the exact bound: {rendered}"
            );
            assert!(
                rendered.contains("kill=") && rendered.contains("reap="),
                "timeout failure must report child kill and reap outcomes: {rendered}"
            );
        });
    }

    fn is_sha256_identifier(value: &str) -> bool {
        let Some(hex) = value.strip_prefix("sha256:") else {
            return false;
        };
        hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
    }

    fn is_immutable_image_reference(value: &str) -> bool {
        is_sha256_identifier(value)
            || value
                .rsplit_once('@')
                .is_some_and(|(_, digest)| is_sha256_identifier(digest))
    }

    fn is_repository_digest(value: &str) -> bool {
        value.rsplit_once('@').is_some_and(|(repository, digest)| {
            !repository.is_empty() && is_sha256_identifier(digest)
        })
    }

    fn verify_image_identity(reference: &str) -> Result<String> {
        let image_id = docker(&["image", "inspect", "--format", "{{.Id}}", reference])?;
        if !is_sha256_identifier(&image_id) {
            return Err(HarnessError::DataInvariant(format!(
                "Docker did not return an immutable image ID for {reference}: {image_id:?}"
            )));
        }
        if is_sha256_identifier(reference) {
            if image_id != reference {
                return Err(HarnessError::DataInvariant(format!(
                "local immutable image ID changed: requested {reference}, Docker ran {image_id}"
            )));
            }
            return Ok(image_id);
        }

        let repo_digests = docker(&[
            "image",
            "inspect",
            "--format",
            "{{json .RepoDigests}}",
            reference,
        ])?;
        let repo_digests = serde_json::from_str::<Vec<String>>(&repo_digests).map_err(|error| {
            HarnessError::DataInvariant(format!(
                "Docker returned invalid RepoDigests for {reference}: {error}"
            ))
        })?;
        if !repo_digests.iter().any(|digest| digest == reference) {
            return Err(HarnessError::DataInvariant(format!(
            "Docker image ID {image_id} has no RepoDigest matching requested {reference}; got {repo_digests:?}"
        )));
        }
        Ok(image_id)
    }

    #[derive(Debug, Clone, Copy)]
    struct RuntimeCounters {
        checkpoints: u64,
        merges: u64,
        checkpoint_duration_count: u64,
        checkpoint_duration_seconds: f64,
        capture_lock_duration_count: u64,
        capture_lock_duration_seconds: f64,
        checkpoint_bytes: u64,
        merge_read_bytes: u64,
        merge_write_bytes: u64,
        backpressure_events: u64,
        pending_delta_bytes: u64,
        pending_delta_layers: u64,
        segment_disk_bytes: u64,
        process_rss_high_water_bytes: u64,
    }

    impl RuntimeCounters {
        fn parse(metrics: &str) -> Result<Self> {
            if prometheus_counter(metrics, "lumen_process_rss_high_water_available")? != 1 {
                return Err(HarnessError::DataInvariant(
                    "process VmHWM measurement is unavailable".to_owned(),
                ));
            }
            Ok(Self {
                checkpoints: prometheus_counter(metrics, CHECKPOINT_COUNTER)?,
                merges: prometheus_counter(metrics, MERGE_COUNTER)?,
                checkpoint_duration_count: prometheus_counter(metrics, CHECKPOINT_DURATION_COUNT)?,
                checkpoint_duration_seconds: prometheus_float(metrics, CHECKPOINT_DURATION_SUM)?,
                capture_lock_duration_count: prometheus_counter(
                    metrics,
                    CAPTURE_LOCK_DURATION_COUNT,
                )?,
                capture_lock_duration_seconds: prometheus_float(
                    metrics,
                    CAPTURE_LOCK_DURATION_SUM,
                )?,
                checkpoint_bytes: prometheus_counter(metrics, CHECKPOINT_BYTES_COUNTER)?,
                merge_read_bytes: prometheus_counter(metrics, MERGE_READ_BYTES_COUNTER)?,
                merge_write_bytes: prometheus_counter(metrics, MERGE_WRITE_BYTES_COUNTER)?,
                backpressure_events: prometheus_counter(metrics, BACKPRESSURE_COUNTER)?,
                pending_delta_bytes: prometheus_counter(metrics, PENDING_DELTA_BYTES)?,
                pending_delta_layers: prometheus_counter(metrics, PENDING_DELTA_LAYERS)?,
                segment_disk_bytes: prometheus_counter(metrics, SEGMENT_DISK_BYTES)?,
                process_rss_high_water_bytes: prometheus_counter(
                    metrics,
                    PROCESS_RSS_HIGH_WATER_BYTES,
                )?,
            })
        }

        fn delta(self, before: Self) -> Result<Self> {
            Ok(Self {
                checkpoints: counter_delta(
                    CHECKPOINT_COUNTER,
                    self.checkpoints,
                    before.checkpoints,
                )?,
                merges: counter_delta(MERGE_COUNTER, self.merges, before.merges)?,
                checkpoint_duration_count: counter_delta(
                    CHECKPOINT_DURATION_COUNT,
                    self.checkpoint_duration_count,
                    before.checkpoint_duration_count,
                )?,
                checkpoint_duration_seconds: float_delta(
                    CHECKPOINT_DURATION_SUM,
                    self.checkpoint_duration_seconds,
                    before.checkpoint_duration_seconds,
                )?,
                capture_lock_duration_count: counter_delta(
                    CAPTURE_LOCK_DURATION_COUNT,
                    self.capture_lock_duration_count,
                    before.capture_lock_duration_count,
                )?,
                capture_lock_duration_seconds: float_delta(
                    CAPTURE_LOCK_DURATION_SUM,
                    self.capture_lock_duration_seconds,
                    before.capture_lock_duration_seconds,
                )?,
                checkpoint_bytes: counter_delta(
                    CHECKPOINT_BYTES_COUNTER,
                    self.checkpoint_bytes,
                    before.checkpoint_bytes,
                )?,
                merge_read_bytes: counter_delta(
                    MERGE_READ_BYTES_COUNTER,
                    self.merge_read_bytes,
                    before.merge_read_bytes,
                )?,
                merge_write_bytes: counter_delta(
                    MERGE_WRITE_BYTES_COUNTER,
                    self.merge_write_bytes,
                    before.merge_write_bytes,
                )?,
                backpressure_events: counter_delta(
                    BACKPRESSURE_COUNTER,
                    self.backpressure_events,
                    before.backpressure_events,
                )?,
                // Gauges are end-of-interval observations, not warmup deltas.
                pending_delta_bytes: self.pending_delta_bytes,
                pending_delta_layers: self.pending_delta_layers,
                segment_disk_bytes: self.segment_disk_bytes,
                // The source metric reports Lumen's own VmHWM. It is not a
                // once-per-second `docker top` sample nor a maximum across an
                // arbitrary process list.
                process_rss_high_water_bytes: self.process_rss_high_water_bytes,
            })
        }

        fn assert_complete_interval_evidence(self) -> Result<()> {
            if self.checkpoints == 0 || self.merges == 0 {
                return Err(HarnessError::DataInvariant(format!(
                "measured interval needs checkpoint and merge completion, got checkpoints={} merges={}",
                self.checkpoints, self.merges
            )));
            }
            if self.checkpoint_duration_count < self.checkpoints
                || self.capture_lock_duration_count < self.checkpoints
            {
                return Err(HarnessError::DataInvariant(format!(
                "checkpoint duration/capture-lock observations do not cover completed checkpoints: checkpoints={} duration_count={} capture_lock_count={}",
                self.checkpoints, self.checkpoint_duration_count, self.capture_lock_duration_count
            )));
            }
            if self.checkpoint_bytes == 0
                || self.merge_read_bytes == 0
                || self.merge_write_bytes == 0
            {
                return Err(HarnessError::DataInvariant(format!(
                "completed interval work lacks checkpoint or merge IO bytes: checkpoint={} merge_read={} merge_write={}",
                self.checkpoint_bytes, self.merge_read_bytes, self.merge_write_bytes
            )));
            }
            if self.segment_disk_bytes == 0 || self.process_rss_high_water_bytes == 0 {
                return Err(HarnessError::DataInvariant(format!(
                    "disk/RSS observability is incomplete: disk_bytes={} rss_high_water_bytes={}",
                    self.segment_disk_bytes, self.process_rss_high_water_bytes
                )));
            }
            Ok(())
        }
    }

    fn prometheus_counter(metrics: &str, name: &'static str) -> Result<u64> {
        let value = prometheus_value(metrics, name)?;
        value.parse::<u64>().map_err(|_| HarnessError::MetricParse {
            name,
            value: value.to_owned(),
        })
    }

    fn prometheus_float(metrics: &str, name: &'static str) -> Result<f64> {
        let raw = prometheus_value(metrics, name)?;
        let value = raw.parse::<f64>().map_err(|_| HarnessError::MetricParse {
            name,
            value: raw.to_owned(),
        })?;
        if !value.is_finite() {
            return Err(HarnessError::MetricParse {
                name,
                value: raw.to_owned(),
            });
        }
        Ok(value)
    }

    fn prometheus_value<'a>(metrics: &'a str, name: &'static str) -> Result<&'a str> {
        let mut values = metrics.lines().filter_map(|line| {
            let (metric, value) = line.split_once(|character: char| character.is_whitespace())?;
            (metric == name).then_some(value.trim())
        });
        let value = values
            .next()
            .ok_or(HarnessError::MissingRuntimeMetric(name))?;
        if values.next().is_some() {
            return Err(HarnessError::DuplicateRuntimeMetric(name));
        }
        if value.is_empty() {
            return Err(HarnessError::MetricParse {
                name,
                value: value.to_owned(),
            });
        }
        Ok(value)
    }

    fn counter_delta(name: &'static str, after: u64, before: u64) -> Result<u64> {
        after.checked_sub(before).ok_or_else(|| {
        HarnessError::DataInvariant(format!(
            "runtime counter moved backwards during measured interval: {name} before={before} after={after}"
        ))
    })
    }

    fn float_delta(name: &'static str, after: f64, before: f64) -> Result<f64> {
        if !after.is_finite() || !before.is_finite() || after < before {
            return Err(HarnessError::DataInvariant(format!(
            "runtime cumulative duration moved backwards or was not finite: {name} before={before} after={after}"
        )));
        }
        Ok(after - before)
    }

    #[derive(Debug, Clone)]
    struct Clock {
        started: Instant,
    }

    impl Clock {
        fn new() -> Self {
            Self {
                started: Instant::now(),
            }
        }

        fn elapsed(&self) -> Duration {
            self.started.elapsed()
        }

        fn deadline(&self, offset: Duration) -> tokio::time::Instant {
            tokio::time::Instant::from_std(self.started + offset)
        }
    }

    #[derive(Clone)]
    struct IndexedField {
        operation: u64,
        field: String,
        body: Value,
    }

    #[derive(Clone)]
    struct Replacement {
        operation: u64,
        body: Value,
    }

    #[derive(Clone)]
    struct Removal {
        operation: u64,
        external_id: String,
    }

    struct Batcher<T> {
        capacity: usize,
        pending: VecDeque<T>,
    }

    impl<T> Batcher<T> {
        fn new(capacity: usize) -> Self {
            Self {
                capacity,
                pending: VecDeque::new(),
            }
        }

        fn push(&mut self, item: T) {
            self.pending.push_back(item);
        }

        fn take_full(&mut self) -> Option<Vec<T>> {
            (self.pending.len() >= self.capacity)
                .then(|| self.pending.drain(..self.capacity).collect::<Vec<_>>())
        }

        fn is_empty(&self) -> bool {
            self.pending.is_empty()
        }
    }

    struct RequestPump {
        tasks: JoinSet<()>,
        max_in_flight: usize,
    }

    impl RequestPump {
        fn new(max_in_flight: usize) -> Self {
            Self {
                tasks: JoinSet::new(),
                max_in_flight,
            }
        }

        async fn push<F>(
            &mut self,
            future: F,
            input_deadline: tokio::time::Instant,
            input_timeout: Duration,
        ) -> Result<()>
        where
            F: Future<Output = ()> + Send + 'static,
        {
            check_input_deadline(input_deadline, input_timeout, "input_workload")?;
            while self.tasks.len() >= self.max_in_flight {
                tokio::time::timeout_at(input_deadline, self.tasks.join_next())
                    .await
                    .map_err(|_| HarnessError::InputWindowTimeout {
                        stage: "input_workload",
                        timeout: input_timeout,
                    })?
                    .ok_or_else(|| HarnessError::Task("request pump lost a task".to_owned()))?
                    .map_err(|error| HarnessError::Task(error.to_string()))?;
                check_input_deadline(input_deadline, input_timeout, "input_workload")?;
            }
            check_input_deadline(input_deadline, input_timeout, "input_workload")?;
            self.tasks.spawn(future);
            Ok(())
        }

        async fn drain(mut self, label: &'static str) -> Result<()> {
            match tokio::time::timeout(DRAIN_TIMEOUT, async {
                while let Some(result) = self.tasks.join_next().await {
                    result.map_err(|error| {
                        HarnessError::Task(format!("{label} task failed: {error}"))
                    })?;
                }
                Ok::<_, HarnessError>(())
            })
            .await
            {
                Ok(result) => result,
                Err(_) => {
                    eprintln!("PERF_STAGE_TIMEOUT {label} workload_drain");
                    Err(HarnessError::PostInputTimeout {
                        stage: "workload_drain",
                        timeout: DRAIN_TIMEOUT,
                    })
                }
            }
        }
    }

    async fn post_json(
        request: reqwest::RequestBuilder,
    ) -> std::result::Result<Value, PostJsonFailure> {
        match tokio::time::timeout(REQUEST_TIMEOUT, async {
            let response = request.send().await.map_err(|error| PostJsonFailure {
                error: RequestFailure::from_reqwest(error),
                outcome: Outcome::Failed,
                status: None,
                body: None,
            })?;
            let status = response.status();
            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                return Err(PostJsonFailure {
                    error: RequestFailure::synthetic(
                        format!("workload request returned non-success status {status}"),
                        false,
                    ),
                    outcome: Outcome::Failed,
                    status: Some(status.as_u16()),
                    body: Some(truncate_evidence_body(&body)),
                });
            }
            response
                .json::<Value>()
                .await
                .map_err(|error| PostJsonFailure {
                    error: RequestFailure::from_reqwest(error),
                    outcome: Outcome::Failed,
                    status: Some(status.as_u16()),
                    body: None,
                })
        })
        .await
        {
            Ok(result) => result,
            Err(_) => Err(PostJsonFailure {
                error: RequestFailure::synthetic(
                    format!(
                        "workload request exceeded the {}-second deadline",
                        REQUEST_TIMEOUT.as_secs()
                    ),
                    true,
                ),
                outcome: Outcome::TimedOut,
                status: None,
                body: None,
            }),
        }
    }

    fn is_warmup_retry_after_one(status: reqwest::StatusCode, retry_after: Option<&str>) -> bool {
        status == reqwest::StatusCode::TOO_MANY_REQUESTS && retry_after == Some("1")
    }

    async fn send_index(
        client: reqwest::Client,
        base: String,
        ledger: Arc<Mutex<WorkloadLedger>>,
        clock: Clock,
        journal: Arc<Mutex<RequestErrorJournal>>,
        request_id: u64,
        scheduled_at: Duration,
        entries: Vec<IndexedField>,
    ) {
        let items = entries
            .iter()
            .map(|entry| RequestItem::new(entry.operation, entry.field.clone()))
            .collect::<Vec<_>>();
        let body = json!({
            "items": entries.iter().map(|entry| entry.body.clone()).collect::<Vec<_>>()
        });
        let request = client
            .post(format!("{base}/collections/{HOT_COLLECTION}/index"))
            .json(&body);
        // Acquire the evidence lock before the timestamp. A stalled ledger
        // must not make queued work look as if its body had already started.
        // `post_json(request)` calls `send` immediately after this scope.
        let mut ledger_guard = ledger.lock().await;
        let body_started = clock.elapsed();
        ledger_guard.submit_request(Request::new(
            request_id,
            Endpoint::Index,
            scheduled_at,
            body_started,
            items,
        ));
        drop(ledger_guard);
        let outcome = match post_json(request).await {
            Ok(response) if response["indexed"].as_u64() == Some(entries.len() as u64) => {
                Outcome::Succeeded
            }
            Ok(_) => Outcome::Failed,
            Err(failure) => {
                record_request_error(
                    &journal,
                    clock.elapsed(),
                    format!("POST /collections/{HOT_COLLECTION}/index"),
                    format!("request_id={request_id}"),
                    failure,
                )
                .await
            }
        };
        let mut ledger = ledger.lock().await;
        for entry in &entries {
            ledger.record_item_result(request_id, entry.operation, &entry.field, outcome);
        }
        ledger.finish_request(request_id, clock.elapsed(), outcome);
    }

    async fn send_replace(
        client: reqwest::Client,
        base: String,
        ledger: Arc<Mutex<WorkloadLedger>>,
        clock: Clock,
        journal: Arc<Mutex<RequestErrorJournal>>,
        request_id: u64,
        scheduled_at: Duration,
        entries: Vec<Replacement>,
    ) {
        let items = entries
            .iter()
            .map(|entry| RequestItem::new(entry.operation, "$document"))
            .collect::<Vec<_>>();
        let body = json!({
            "docs": entries.iter().map(|entry| entry.body.clone()).collect::<Vec<_>>()
        });
        let request = client
            .put(format!("{base}/collections/{HOT_COLLECTION}/docs:replace"))
            .json(&body);
        let mut ledger_guard = ledger.lock().await;
        let body_started = clock.elapsed();
        ledger_guard.submit_request(Request::new(
            request_id,
            Endpoint::Replace,
            scheduled_at,
            body_started,
            items,
        ));
        drop(ledger_guard);
        let outcome = match post_json(request).await {
            Ok(response)
                if response["results"].as_array().is_some_and(|results| {
                    results.len() == entries.len()
                        && results.iter().all(|result| result["status"] == "ok")
                }) =>
            {
                Outcome::Succeeded
            }
            Ok(_) => Outcome::Failed,
            Err(failure) => {
                record_request_error(
                    &journal,
                    clock.elapsed(),
                    format!("PUT /collections/{HOT_COLLECTION}/docs:replace"),
                    format!("request_id={request_id}"),
                    failure,
                )
                .await
            }
        };
        let mut ledger = ledger.lock().await;
        for entry in &entries {
            ledger.record_item_result(request_id, entry.operation, "$document", outcome);
        }
        ledger.finish_request(request_id, clock.elapsed(), outcome);
    }

    async fn send_unindex(
        client: reqwest::Client,
        base: String,
        ledger: Arc<Mutex<WorkloadLedger>>,
        clock: Clock,
        journal: Arc<Mutex<RequestErrorJournal>>,
        request_id: u64,
        scheduled_at: Duration,
        entries: Vec<Removal>,
    ) {
        let items = entries
            .iter()
            .map(|entry| RequestItem::new(entry.operation, "$external_id"))
            .collect::<Vec<_>>();
        let body = json!({
            "external_ids": entries
                .iter()
                .map(|entry| entry.external_id.clone())
                .collect::<Vec<_>>()
        });
        let request = client
            .post(format!("{base}/collections/{HOT_COLLECTION}/docs:unindex"))
            .json(&body);
        let mut ledger_guard = ledger.lock().await;
        let body_started = clock.elapsed();
        ledger_guard.submit_request(Request::new(
            request_id,
            Endpoint::Unindex,
            scheduled_at,
            body_started,
            items,
        ));
        drop(ledger_guard);
        let unindex_endpoint = format!("POST /collections/{HOT_COLLECTION}/docs:unindex");
        let unindex_identifier = format!("request_id={request_id}");
        let outcome = match tokio::time::timeout(REQUEST_TIMEOUT, request.send()).await {
            Ok(Ok(response)) if response.status().as_u16() == 204 => Outcome::Succeeded,
            Ok(Ok(response)) => {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                record_request_error(
                    &journal,
                    clock.elapsed(),
                    unindex_endpoint.clone(),
                    unindex_identifier.clone(),
                    PostJsonFailure {
                        error: RequestFailure::synthetic(
                            format!("workload request returned non-success status {status}"),
                            false,
                        ),
                        outcome: Outcome::Failed,
                        status: Some(status.as_u16()),
                        body: Some(truncate_evidence_body(&body)),
                    },
                )
                .await
            }
            Ok(Err(error)) => {
                record_request_error(
                    &journal,
                    clock.elapsed(),
                    unindex_endpoint.clone(),
                    unindex_identifier.clone(),
                    PostJsonFailure {
                        error: RequestFailure::from_reqwest(error),
                        outcome: Outcome::Failed,
                        status: None,
                        body: None,
                    },
                )
                .await
            }
            Err(_) => {
                record_request_error(
                    &journal,
                    clock.elapsed(),
                    unindex_endpoint,
                    unindex_identifier,
                    PostJsonFailure {
                        error: RequestFailure::synthetic(
                            format!(
                                "workload request exceeded the {}-second deadline",
                                REQUEST_TIMEOUT.as_secs()
                            ),
                            true,
                        ),
                        outcome: Outcome::TimedOut,
                        status: None,
                        body: None,
                    },
                )
                .await
            }
        };
        let mut ledger = ledger.lock().await;
        for entry in &entries {
            ledger.record_item_result(request_id, entry.operation, "$external_id", outcome);
        }
        ledger.finish_request(request_id, clock.elapsed(), outcome);
    }

    async fn send_query(
        client: reqwest::Client,
        base: String,
        ledger: Arc<Mutex<WorkloadLedger>>,
        clock: Clock,
        journal: Arc<Mutex<RequestErrorJournal>>,
        scheduled_at: Duration,
        class: QueryClass,
        collection: String,
        body: Value,
    ) {
        // Semantic readbacks do not call this function, so these are only
        // the fixed ten-QPS workload requests. Build the body before the
        // timing mark; `post_json` calls `send` immediately after it.
        let request = client
            .post(format!("{base}/collections/{collection}/search"))
            .json(&body);
        let body_started = clock.elapsed();
        let outcome = match post_json(request).await {
            Ok(response)
                if response["hits"]
                    .as_array()
                    .is_some_and(|hits| !hits.is_empty()) =>
            {
                Outcome::Succeeded
            }
            Ok(_) => Outcome::Failed,
            Err(failure) => {
                record_request_error(
                    &journal,
                    clock.elapsed(),
                    format!("POST /collections/{collection}/search"),
                    format!("query_class={class:?}"),
                    failure,
                )
                .await
            }
        };
        ledger.lock().await.record_classified_query(
            class,
            scheduled_at,
            body_started,
            Some(clock.elapsed()),
            outcome,
        );
    }

    async fn seed(
        server: &DockerLumen,
        backend: VectorBackend,
        setup_deadline: tokio::time::Instant,
    ) -> Result<()> {
        check_setup_deadline(setup_deadline)?;
        create_collection(server, HOT_COLLECTION, backend, setup_deadline).await?;
        seed_collection(
            server,
            HOT_COLLECTION,
            HOT_DOCUMENTS,
            0,
            "hot",
            setup_deadline,
        )
        .await?;
        assert_document_count_setup(server, HOT_COLLECTION, HOT_DOCUMENTS, setup_deadline).await?;
        for collection_number in 0..IDLE_COLLECTIONS {
            check_setup_deadline(setup_deadline)?;
            let collection = idle_collection(collection_number);
            create_collection(server, &collection, backend, setup_deadline).await?;
            seed_collection(
                server,
                &collection,
                IDLE_DOCUMENTS_PER_COLLECTION,
                collection_number * IDLE_DOCUMENTS_PER_COLLECTION,
                "idle",
                setup_deadline,
            )
            .await?;
            assert_document_count_setup(
                server,
                &collection,
                IDLE_DOCUMENTS_PER_COLLECTION,
                setup_deadline,
            )
            .await?;
        }
        Ok(())
    }

    async fn checkpoint_seed(
        client: &reqwest::Client,
        base: &str,
        setup_deadline: tokio::time::Instant,
    ) -> Result<String> {
        // Only fixture preparation gets this one awaited checkpoint. The
        // measured workload still has its original five-second request bound,
        // offers the full load, and cannot retry errors. Reuse the existing
        // 60-second drain bound without extending the total setup deadline.
        let deadline = setup_deadline.min(tokio::time::Instant::now() + DRAIN_TIMEOUT);
        let timeout = || HarnessError::SetupTimeout {
            stage: "seed_checkpoint",
        };
        if tokio::time::Instant::now() >= deadline {
            return Err(timeout());
        }
        tokio::time::timeout_at(deadline, async {
            let response = client
                .post(format!("{base}/admin/checkpoint"))
                .timeout(DRAIN_TIMEOUT)
                .json(&json!({}))
                .send()
                .await
                .map_err(HarnessError::request_failure)?;
            let status = response.status();
            if !status.is_success() {
                return Err(HarnessError::Http(format!(
                    "seed checkpoint returned {status}"
                )));
            }
            let response = response
                .json::<Value>()
                .await
                .map_err(HarnessError::request_failure)?;
            if response.get("persisted").and_then(Value::as_bool) != Some(true) {
                return Err(HarnessError::DataInvariant(
                    "seed checkpoint did not confirm persisted=true".to_owned(),
                ));
            }
            let (status, metrics) =
                fetch_interval_metrics(client, format!("{base}/metrics"), REQUEST_TIMEOUT).await?;
            if !status.is_success() {
                return Err(HarnessError::Http(format!(
                    "seed checkpoint metrics returned {status}"
                )));
            }
            // Persisted deltas can remain on disk. Only pending change charges
            // must be zero, so fixture work cannot enter the measured window.
            for name in [
                "lumen_pending_change_active_bytes",
                "lumen_pending_change_frozen_bytes",
                "lumen_pending_change_reserved_bytes",
            ] {
                let bytes = prometheus_counter(&metrics, name)?;
                if bytes != 0 {
                    return Err(HarnessError::DataInvariant(format!(
                        "seed checkpoint left pending work: {name}={bytes}"
                    )));
                }
            }
            eprintln!("PERF_SEED_CHECKPOINT persisted=true active_bytes=0 frozen_bytes=0 reserved_bytes=0");
            Ok(metrics)
        })
        .await
        .map_err(|_| timeout())?
    }

    fn parse_hnsw_cache_seal_receipt(value: Value) -> Result<HnswCacheSealReceipt> {
        let object = value.as_object().ok_or_else(|| {
            HarnessError::DataInvariant("restart cache seal receipt must be a JSON object".to_owned())
        })?;
        let keys = ["sealed", "cache_fields", "durability", "mutation_stamp"];
        if object.len() != keys.len() || keys.iter().any(|key| !object.contains_key(*key)) {
            return Err(HarnessError::DataInvariant(
                "restart cache seal receipt has an unexpected JSON shape".to_owned(),
            ));
        }
        if object.get("sealed").and_then(Value::as_bool) != Some(true) {
            return Err(HarnessError::DataInvariant(
                "restart cache seal receipt does not confirm sealed=true".to_owned(),
            ));
        }
        let cache_fields = object
            .get("cache_fields")
            .and_then(Value::as_u64)
            .filter(|fields| *fields >= 1)
            .ok_or_else(|| {
                HarnessError::DataInvariant(
                    "restart cache seal receipt must report cache_fields >= 1".to_owned(),
                )
            })?;
        let durability = match object.get("durability").and_then(Value::as_str) {
            Some("aof_synced") => "aof_synced",
            Some("checkpoint_committed") => "checkpoint_committed",
            _ => {
                return Err(HarnessError::DataInvariant(
                    "restart cache seal receipt has an invalid durability boundary".to_owned(),
                ))
            }
        };
        let stamp = object
            .get("mutation_stamp")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                HarnessError::DataInvariant(
                    "restart cache seal receipt lacks mutation_stamp".to_owned(),
                )
            })?;
        let stamp_keys = ["epoch", "apply_revision"];
        if stamp.len() != stamp_keys.len()
            || stamp_keys.iter().any(|key| !stamp.contains_key(*key))
        {
            return Err(HarnessError::DataInvariant(
                "restart cache seal receipt has an unexpected mutation_stamp shape".to_owned(),
            ));
        }
        let mutation_epoch = stamp
            .get("epoch")
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                HarnessError::DataInvariant(
                    "restart cache seal receipt mutation_stamp.epoch must be an integer".to_owned(),
                )
            })?;
        let mutation_apply_revision = stamp
            .get("apply_revision")
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                HarnessError::DataInvariant(
                    "restart cache seal receipt mutation_stamp.apply_revision must be an integer"
                        .to_owned(),
                )
            })?;
        Ok(HnswCacheSealReceipt {
            cache_fields,
            durability,
            mutation_epoch,
            mutation_apply_revision,
        })
    }

    fn cache_seal_receipt_line(receipt: &HnswCacheSealReceipt) -> String {
        json!({
            "schema": "lumen.perf-cache-seal-receipt.v1",
            "sealed": true,
            "cache_fields": receipt.cache_fields,
            "durability": receipt.durability,
            "mutation_stamp": {
                "epoch": receipt.mutation_epoch,
                "apply_revision": receipt.mutation_apply_revision,
            },
        })
        .to_string()
    }

    async fn seal_hnsw_cache(
        client: &reqwest::Client,
        base: &str,
    ) -> Result<HnswCacheSealReceipt> {
        let response = client
            .post(format!("{base}/admin/restart:seal-hnsw-cache"))
            .send()
            .await
            .map_err(HarnessError::request_failure)?;
        let status = response.status();
        if status != reqwest::StatusCode::OK {
            return Err(HarnessError::Http(format!(
                "restart cache seal returned {status}"
            )));
        }
        let value = response
            .json::<Value>()
            .await
            .map_err(HarnessError::request_failure)?;
        parse_hnsw_cache_seal_receipt(value)
    }

    async fn create_collection(
        server: &DockerLumen,
        collection: &str,
        backend: VectorBackend,
        setup_deadline: tokio::time::Instant,
    ) -> Result<()> {
        let response = setup_step(setup_deadline, async {
            server
                .client
                .put(format!("{}/collections/{collection}", server.base))
                .json(&schema(backend))
                .send()
                .await
                .map_err(|error| HarnessError::request_failure(error))
        })
        .await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(HarnessError::Http(format!(
                "create collection {collection} returned {}",
                response.status()
            )))
        }
    }

    async fn seed_collection(
        server: &DockerLumen,
        collection: &str,
        documents: usize,
        start: usize,
        tag: &str,
        setup_deadline: tokio::time::Instant,
    ) -> Result<()> {
        let mut items = Vec::with_capacity(1_000);
        for number in start..start + documents {
            check_setup_deadline(setup_deadline)?;
            let external_id = if collection == HOT_COLLECTION {
                format!("hot-base-{number:06}")
            } else {
                format!("{collection}-base-{number:06}")
            };
            for (field, value) in document_fields(number, tag).into_iter() {
                items.push(json!({
                    "external_id": external_id,
                    "field": field,
                    "value": value,
                }));
                if items.len() == 1_000 {
                    seed_index_batch(server, collection, &items, setup_deadline).await?;
                    items.clear();
                }
            }
        }
        if !items.is_empty() {
            seed_index_batch(server, collection, &items, setup_deadline).await?;
        }
        Ok(())
    }

    async fn seed_index_batch(
        server: &DockerLumen,
        collection: &str,
        items: &[Value],
        setup_deadline: tokio::time::Instant,
    ) -> Result<()> {
        let body = json!({ "items": items });
        loop {
            check_setup_deadline(setup_deadline)?;
            // Only setup may honor an explicit one-second backpressure hint.
            // Measured mutations use `post_json` or `send_unindex` and record
            // every refusal or timeout in the ledger without a retry.
            let response = setup_step(setup_deadline, async {
                tokio::time::timeout(REQUEST_TIMEOUT, async {
                    let response = server
                        .client
                        .post(format!("{}/collections/{collection}/index", server.base))
                        .json(&body)
                        .send()
                        .await
                        .map_err(|error| HarnessError::request_failure(error))?;
                    let status = response.status();
                    let retry_after_one = is_warmup_retry_after_one(
                        status,
                        response
                            .headers()
                            .get(reqwest::header::RETRY_AFTER)
                            .and_then(|value| value.to_str().ok()),
                    );
                    if retry_after_one {
                        return Ok::<_, HarnessError>(None);
                    }
                    let body = response
                        .json::<Value>()
                        .await
                        .map_err(|error| HarnessError::request_failure(error))?;
                    Ok(Some((status, body)))
                })
                .await
                .map_err(|_| {
                    HarnessError::Http(format!(
                        "seed index {collection} exceeded the five-second request deadline"
                    ))
                })?
            })
            .await?;
            let Some((status, response_body)) = response else {
                tokio::time::timeout_at(setup_deadline, tokio::time::sleep(Duration::from_secs(1)))
                    .await
                    .map_err(|_| setup_timeout())?;
                continue;
            };
            if status.is_success() && response_body["indexed"].as_u64() == Some(items.len() as u64)
            {
                return Ok(());
            }
            return Err(HarnessError::Http(format!(
                "seed index {collection} returned {status}: {response_body}"
            )));
        }
    }

    async fn assert_document_count_setup(
        server: &DockerLumen,
        collection: &str,
        expected: usize,
        setup_deadline: tokio::time::Instant,
    ) -> Result<()> {
        let (status, body) = setup_step(setup_deadline, async {
            let response = server
                .client
                .get(format!("{}/collections/{collection}/stats", server.base))
                .send()
                .await
                .map_err(|error| HarnessError::request_failure(error))?;
            let status = response.status();
            let body = response
                .json::<Value>()
                .await
                .map_err(|error| HarnessError::request_failure(error))?;
            Ok::<_, HarnessError>((status, body))
        })
        .await?;
        if status.is_success() && body["documents_indexed"].as_u64() == Some(expected as u64) {
            Ok(())
        } else {
            Err(HarnessError::DataInvariant(format!(
                "collection {collection} expected {expected} documents, got {body}"
            )))
        }
    }

    async fn assert_document_count(
        server: &DockerLumen,
        collection: &str,
        expected: usize,
    ) -> Result<()> {
        let (status, body) = tokio::time::timeout(REQUEST_TIMEOUT, async {
            let response = server
                .client
                .get(format!("{}/collections/{collection}/stats", server.base))
                .send()
                .await
                .map_err(|error| HarnessError::request_failure(error))?;
            let status = response.status();
            let body = response
                .json::<Value>()
                .await
                .map_err(|error| HarnessError::request_failure(error))?;
            Ok::<_, HarnessError>((status, body))
        })
        .await
        .map_err(|_| {
            HarnessError::Http(format!(
                "document count {collection} exceeded the five-second request deadline"
            ))
        })??;
        if status.is_success() && body["documents_indexed"].as_u64() == Some(expected as u64) {
            Ok(())
        } else {
            Err(HarnessError::DataInvariant(format!(
                "collection {collection} expected {expected} documents, got {body}"
            )))
        }
    }

    fn workload_collections() -> impl Iterator<Item = String> {
        std::iter::once(HOT_COLLECTION.to_owned()).chain((0..IDLE_COLLECTIONS).map(idle_collection))
    }

    fn post_restart_vector_bytes(collection: &str, stats: &Value) -> Result<u64> {
        let fields = stats["fields"].as_object().ok_or_else(|| {
            HarnessError::DataInvariant(format!(
                "post-restart backend attestation for {collection} lacks a fields object: {stats}"
            ))
        })?;
        let vector = fields
            .get(WORKLOAD_VECTOR_FIELD)
            .and_then(Value::as_object)
            .ok_or_else(|| {
                HarnessError::DataInvariant(format!(
                    "post-restart backend attestation for {collection} lacks vector field {WORKLOAD_VECTOR_FIELD}: {stats}"
                ))
            })?;
        match vector.get("type").and_then(Value::as_str) {
            Some("vector") => {}
            Some(other) => {
                return Err(HarnessError::DataInvariant(format!(
                    "post-restart backend attestation for {collection} reports {WORKLOAD_VECTOR_FIELD} as {other:?}, not vector"
                )));
            }
            None => {
                return Err(HarnessError::DataInvariant(format!(
                    "post-restart backend attestation for {collection} lacks {WORKLOAD_VECTOR_FIELD}.type"
                )));
            }
        }
        vector
            .get("bytes")
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                HarnessError::DataInvariant(format!(
                    "post-restart backend attestation for {collection} lacks an unsigned {WORKLOAD_VECTOR_FIELD}.bytes observation"
                ))
            })
    }

    fn assert_post_restart_vector_residency(
        backend: VectorBackend,
        collection: &str,
        bytes: u64,
    ) -> Result<()> {
        match backend {
            VectorBackend::FlatCpu if bytes == 0 => Ok(()),
            VectorBackend::FlatCpu => Err(HarnessError::DataInvariant(format!(
                "post-restart backend attestation for {collection} expected flat-cpu {WORKLOAD_VECTOR_FIELD}.bytes=0 after segment reopen, got {bytes}"
            ))),
            VectorBackend::HnswCpu if bytes > 0 => Ok(()),
            VectorBackend::HnswCpu => Err(HarnessError::DataInvariant(format!(
                "post-restart backend attestation for {collection} expected hnsw-cpu {WORKLOAD_VECTOR_FIELD}.bytes>0 after segment reopen, got {bytes}"
            ))),
        }
    }

    async fn assert_post_restart_vector_backends(
        server: &DockerLumen,
        backend: VectorBackend,
    ) -> Result<()> {
        for collection in workload_collections() {
            let response = server
                .client
                .get(format!("{}/collections/{collection}/stats", server.base))
                .send()
                .await
                .map_err(|error| HarnessError::request_failure(error))?;
            let status = response.status();
            let stats = response
                .json::<Value>()
                .await
                .map_err(|error| HarnessError::request_failure(error))?;
            if !status.is_success() {
                return Err(HarnessError::DataInvariant(format!(
                    "post-restart backend attestation could not read stats for {collection}: {status} {stats}"
                )));
            }
            let bytes = post_restart_vector_bytes(&collection, &stats)?;
            assert_post_restart_vector_residency(backend, &collection, bytes)?;
        }
        Ok(())
    }

    async fn assert_recovered_text_query(server: &DockerLumen) -> Result<()> {
        let response = server
            .client
            .post(format!(
                "{}/collections/{HOT_COLLECTION}/search",
                server.base
            ))
            .json(&json!({
                "query": {
                    "match": {
                        "field": "title_ngram",
                        "text": "durable search",
                        "op": "and",
                    }
                },
                "limit": 10,
            }))
            .send()
            .await
            .map_err(|error| HarnessError::request_failure(error))?;
        let status = response.status();
        let body = response
            .json::<Value>()
            .await
            .map_err(|error| HarnessError::request_failure(error))?;
        if status.is_success() && body["hits"].as_array().is_some_and(|hits| !hits.is_empty()) {
            Ok(())
        } else {
            Err(HarnessError::DataInvariant(format!(
                "cold restart did not recover a nonempty Text/BM25 result: {status} {body}"
            )))
        }
    }

    #[derive(Debug, Clone)]
    struct SemanticDocument {
        external_id: String,
        number: usize,
        tag: &'static str,
    }

    #[derive(Debug, Clone)]
    struct MutationReadback {
        indexed: SemanticDocument,
        replaced: SemanticDocument,
        deleted: SemanticDocument,
    }

    fn mutation_readback() -> MutationReadback {
        // These are the first deterministic operations on the three existing
        // scheduler turns. They remain well away from each other: index adds
        // a new ID, replace changes an untouched base ID, and unindex removes
        // a separate base ID.
        MutationReadback {
            indexed: SemanticDocument {
                external_id: "hot-added-000000".to_owned(),
                number: HOT_DOCUMENTS,
                tag: "hot",
            },
            replaced: SemanticDocument {
                external_id: "hot-base-060000".to_owned(),
                number: 60_000,
                tag: "hot-updated",
            },
            deleted: SemanticDocument {
                external_id: "hot-base-000000".to_owned(),
                number: 0,
                tag: "hot",
            },
        }
    }

    fn vector_readback_reference() -> SemanticDocument {
        SemanticDocument {
            external_id: format!("hot-base-{VECTOR_READBACK_REFERENCE_NUMBER:06}"),
            number: VECTOR_READBACK_REFERENCE_NUMBER,
            tag: "hot",
        }
    }

    fn assert_vector_reference_is_untouched(reference: &SemanticDocument) -> Result<()> {
        let first_untouched_base =
            MUTATION_DOCUMENTS_PER_CLASS.checked_mul(2).ok_or_else(|| {
                HarnessError::DataInvariant(
                    "mutation schedule overflowed while locating vector readback reference"
                        .to_owned(),
                )
            })?;
        let expected_id = format!("hot-base-{:06}", reference.number);
        // `offer_deletions` owns [0, M), `offer_replacements` owns [M, 2M),
        // and `offer_additions` creates only hot-added IDs. This reference is
        // therefore a seeded hot-base document whose fields are never changed
        // by the full 30-minute schedule.
        if reference.tag != "hot"
            || reference.external_id != expected_id
            || !(first_untouched_base..HOT_DOCUMENTS).contains(&reference.number)
        {
            return Err(HarnessError::DataInvariant(format!(
                "vector readback reference must be an untouched seeded hot-base document; \
                 id={} number={} untouched=[{first_untouched_base},{HOT_DOCUMENTS})",
                reference.external_id, reference.number
            )));
        }
        Ok(())
    }

    fn fingerprint_query(fingerprint: &str) -> Value {
        json!({ "hamming": {
            "field": "fingerprint",
            "hash": fingerprint,
            "max_distance": 0,
        }})
    }

    fn anchored_semantic_query(fingerprint: &str, field_clause: Value) -> Value {
        json!({ "and": [fingerprint_query(fingerprint), field_clause] })
    }

    fn readback_mismatch_value(field: &str, number: usize) -> String {
        format!("{READBACK_MISMATCH_TOKEN}-window-{number}-{field}")
    }

    fn mismatched_fingerprint(fingerprint: &str) -> Result<String> {
        let mut bytes = fingerprint.as_bytes().to_vec();
        let Some(first) = bytes.first_mut() else {
            return Err(HarnessError::DataInvariant(
                "semantic fixture fingerprint is empty".to_owned(),
            ));
        };
        if !first.is_ascii_hexdigit() {
            return Err(HarnessError::DataInvariant(format!(
                "semantic fixture fingerprint is not hexadecimal: {fingerprint}"
            )));
        }
        *first = if *first == b'0' { b'1' } else { b'0' };
        String::from_utf8(bytes).map_err(|error| {
            HarnessError::DataInvariant(format!(
                "semantic fixture fingerprint lost UTF-8 after mismatch mutation: {error}"
            ))
        })
    }

    fn semantic_field_clause(field: &str, value: &Value) -> Result<Value> {
        match field {
            "tag" | "category" | "status" | "region" => {
                let value = value.as_str().ok_or_else(|| {
                    HarnessError::DataInvariant(format!(
                        "semantic fixture field {field} is not a string: {value}"
                    ))
                })?;
                Ok(json!({ "term": { "field": field, "value": value } }))
            }
            "price" | "rank" => {
                let value = value.as_f64().ok_or_else(|| {
                    HarnessError::DataInvariant(format!(
                        "semantic fixture field {field} is not a number: {value}"
                    ))
                })?;
                Ok(json!({ "range": {
                    "field": field,
                    "gte": value,
                    "lte": value,
                }}))
            }
            "labels" => {
                let labels = value.as_array().ok_or_else(|| {
                    HarnessError::DataInvariant(format!(
                        "semantic fixture field labels is not an array: {value}"
                    ))
                })?;
                if labels.len() != 2 {
                    return Err(HarnessError::DataInvariant(format!(
                        "semantic fixture labels must have two values: {labels:?}"
                    )));
                }
                let mut clauses = Vec::with_capacity(labels.len());
                for label in labels {
                    let label = label.as_str().ok_or_else(|| {
                        HarnessError::DataInvariant(format!(
                            "semantic fixture label is not a string: {label}"
                        ))
                    })?;
                    clauses.push(json!({ "term": { "field": "labels", "value": label } }));
                }
                Ok(json!({ "and": clauses }))
            }
            "fingerprint" => {
                let fingerprint = value.as_str().ok_or_else(|| {
                    HarnessError::DataInvariant(format!(
                        "semantic fixture fingerprint is not a string: {value}"
                    ))
                })?;
                Ok(fingerprint_query(fingerprint))
            }
            "title_ngram" | "body_ngram" | "summary_ngram" => {
                let value = value.as_str().ok_or_else(|| {
                    HarnessError::DataInvariant(format!(
                        "semantic fixture field {field} is not text: {value}"
                    ))
                })?;
                // Use every term in the actual long fixture value. A short
                // prefix would let a partial or stale n-gram field look right.
                Ok(json!({ "match": {
                    "field": field,
                    "text": value,
                    "op": "and",
                }}))
            }
            "title_text" | "body_text" => {
                let value = value.as_str().ok_or_else(|| {
                    HarnessError::DataInvariant(format!(
                        "semantic fixture field {field} is not text: {value}"
                    ))
                })?;
                Ok(json!({ "match": {
                    "field": field,
                    "text": value,
                    "op": "and",
                }}))
            }
            "embedding" => Err(HarnessError::DataInvariant(
                "embedding readback requires the bounded two-document candidate pair".to_owned(),
            )),
            unexpected => {
                return Err(HarnessError::DataInvariant(format!(
                    "semantic readback has no query for frozen field {unexpected}"
                )));
            }
        }
    }

    fn semantic_wrong_field_clause(field: &str, value: &Value, number: usize) -> Result<Value> {
        match field {
            "tag" | "category" | "status" | "region" => {
                let _ = value.as_str().ok_or_else(|| {
                    HarnessError::DataInvariant(format!(
                        "semantic fixture field {field} is not a string: {value}"
                    ))
                })?;
                Ok(json!({ "term": {
                    "field": field,
                    "value": readback_mismatch_value(field, number),
                }}))
            }
            "price" | "rank" => {
                let value = value.as_f64().ok_or_else(|| {
                    HarnessError::DataInvariant(format!(
                        "semantic fixture field {field} is not a number: {value}"
                    ))
                })?;
                let mismatch = value + 1.0;
                Ok(json!({ "range": {
                    "field": field,
                    "gte": mismatch,
                    "lte": mismatch,
                }}))
            }
            "labels" => {
                let labels = value.as_array().ok_or_else(|| {
                    HarnessError::DataInvariant(format!(
                        "semantic fixture field labels is not an array: {value}"
                    ))
                })?;
                if labels.len() != 2 || labels.iter().any(|label| !label.is_string()) {
                    return Err(HarnessError::DataInvariant(format!(
                        "semantic fixture labels must have two string values: {labels:?}"
                    )));
                }
                Ok(json!({ "term": {
                    "field": "labels",
                    "value": readback_mismatch_value(field, number),
                }}))
            }
            "fingerprint" => {
                let fingerprint = value.as_str().ok_or_else(|| {
                    HarnessError::DataInvariant(format!(
                        "semantic fixture fingerprint is not a string: {value}"
                    ))
                })?;
                Ok(fingerprint_query(&mismatched_fingerprint(fingerprint)?))
            }
            "title_ngram" | "body_ngram" | "summary_ngram" | "title_text" | "body_text" => {
                let value = value.as_str().ok_or_else(|| {
                    HarnessError::DataInvariant(format!(
                        "semantic fixture field {field} is not text: {value}"
                    ))
                })?;
                // The known absent token and window make this a disjoint
                // full-value probe, rather than a different document number
                // whose n-grams could overlap the target's prefix.
                Ok(json!({ "match": {
                    "field": field,
                    "text": format!(
                        "{value} {READBACK_MISMATCH_TOKEN} window {number} field {field}"
                    ),
                    "op": "and",
                }}))
            }
            "embedding" => Err(HarnessError::DataInvariant(
                "embedding readback requires the bounded two-document candidate pair".to_owned(),
            )),
            unexpected => Err(HarnessError::DataInvariant(format!(
                "semantic readback has no mismatch query for frozen field {unexpected}"
            ))),
        }
    }

    fn semantic_field_query(field: &str, value: &Value, fingerprint: &str) -> Result<Value> {
        Ok(anchored_semantic_query(
            fingerprint,
            semantic_field_clause(field, value)?,
        ))
    }

    fn semantic_wrong_field_query(
        field: &str,
        value: &Value,
        fingerprint: &str,
        number: usize,
    ) -> Result<Value> {
        Ok(anchored_semantic_query(
            fingerprint,
            semantic_wrong_field_clause(field, value, number)?,
        ))
    }

    fn bounded_vector_query(
        target_fingerprint: &str,
        reference_fingerprint: &str,
        vector: &Value,
    ) -> Result<Value> {
        let vector = vector.as_array().ok_or_else(|| {
            HarnessError::DataInvariant(format!(
                "semantic fixture embedding is not an array: {vector}"
            ))
        })?;
        if vector.len() != VECTOR_DIMENSIONS as usize {
            return Err(HarnessError::DataInvariant(format!(
                "semantic fixture embedding has wrong dimension: {}",
                vector.len()
            )));
        }
        if target_fingerprint == reference_fingerprint {
            return Err(HarnessError::DataInvariant(
                "bounded vector readback candidate fingerprints must differ".to_owned(),
            ));
        }
        Ok(json!({ "and": [
            { "or": [
                fingerprint_query(target_fingerprint),
                fingerprint_query(reference_fingerprint),
            ]},
            { "knn": {
                "field": "embedding",
                "vector": vector,
                "k": 1,
            }},
        ]}))
    }

    fn embedding_numerator(number: usize, dimension: usize) -> u64 {
        (seeded_value(number, dimension + 3) % 997) + 1
    }

    fn assert_vector_pair_is_distinct_and_non_collinear(
        target: &SemanticDocument,
        reference: &SemanticDocument,
    ) -> Result<()> {
        let target_components = (0..VECTOR_DIMENSIONS as usize)
            .map(|dimension| embedding_numerator(target.number, dimension))
            .collect::<Vec<_>>();
        let reference_components = (0..VECTOR_DIMENSIONS as usize)
            .map(|dimension| embedding_numerator(reference.number, dimension))
            .collect::<Vec<_>>();
        if target_components == reference_components {
            return Err(HarnessError::DataInvariant(format!(
                "bounded vector readback target and reference have the same fixture vector: {} and {}",
                target.external_id, reference.external_id
            )));
        }
        let target_first = target_components[0];
        let reference_first = reference_components[0];
        let non_collinear = (1..VECTOR_DIMENSIONS as usize).any(|dimension| {
            target_first * reference_components[dimension]
                != target_components[dimension] * reference_first
        });
        if !non_collinear {
            return Err(HarnessError::DataInvariant(format!(
                "bounded vector readback target and reference are collinear: {} and {}",
                target.external_id, reference.external_id
            )));
        }
        Ok(())
    }

    async fn semantic_search(server: &DockerLumen, query: Value, context: &str) -> Result<Value> {
        let response = tokio::time::timeout(
            REQUEST_TIMEOUT,
            server
                .client
                .post(format!(
                    "{}/collections/{HOT_COLLECTION}/search",
                    server.base
                ))
                .json(&json!({ "query": query, "limit": 1 }))
                .send(),
        )
        .await
        .map_err(|_| {
            HarnessError::RequestFailure(
                RequestFailure::synthetic(
                    format!(
                        "semantic readback exceeded the {}-second deadline",
                        REQUEST_TIMEOUT.as_secs()
                    ),
                    true,
                )
                .with_context(context),
            )
        })?
        .map_err(|error| {
            HarnessError::RequestFailure(RequestFailure::from_reqwest(error).with_context(context))
        })?;
        let status = response.status();
        let body = response.json::<Value>().await.map_err(|error| {
            HarnessError::RequestFailure(RequestFailure::from_reqwest(error).with_context(context))
        })?;
        if status.is_success() {
            Ok(body)
        } else {
            Err(HarnessError::DataInvariant(format!(
                "{context}: semantic readback returned {status}: {body}"
            )))
        }
    }

    fn response_contains_id(response: &Value, external_id: &str) -> bool {
        response["hits"].as_array().is_some_and(|hits| {
            hits.iter()
                .any(|hit| hit["external_id"].as_str() == Some(external_id))
        })
    }

    async fn assert_bounded_embedding_readback(
        server: &DockerLumen,
        target: &SemanticDocument,
        target_fields: &Map<String, Value>,
        reference: &SemanticDocument,
        phase: &str,
    ) -> Result<()> {
        assert_vector_reference_is_untouched(reference)?;
        if target.external_id == reference.external_id {
            return Err(HarnessError::DataInvariant(
                "bounded vector readback target and reference IDs must differ".to_owned(),
            ));
        }
        assert_vector_pair_is_distinct_and_non_collinear(target, reference)?;

        let reference_fields = document_fields(reference.number, reference.tag);
        let target_fingerprint = target_fields["fingerprint"].as_str().ok_or_else(|| {
            HarnessError::DataInvariant("semantic target lacks string fingerprint".to_owned())
        })?;
        let reference_fingerprint = reference_fields["fingerprint"].as_str().ok_or_else(|| {
            HarnessError::DataInvariant("semantic reference lacks string fingerprint".to_owned())
        })?;
        let target_vector = target_fields["embedding"].clone();
        let reference_vector = reference_fields["embedding"].clone();

        // The OR filter bounds each kNN probe to precisely these two live
        // documents. With k=1, each document's exact fixture vector must rank
        // its own ID first, without assuming global HNSW self-recall.
        for (label, vector, expected_id) in [
            ("target", target_vector, target.external_id.as_str()),
            (
                "reference",
                reference_vector,
                reference.external_id.as_str(),
            ),
        ] {
            let query = bounded_vector_query(target_fingerprint, reference_fingerprint, &vector)?;
            let context =
                format!("{phase}: bounded embedding {label} probe must select {expected_id}");
            let response = semantic_search(server, query, &context).await?;
            if !response_contains_id(&response, expected_id) {
                return Err(HarnessError::DataInvariant(format!(
                    "{context}: wrong bounded vector candidate: {response}"
                )));
            }
        }
        Ok(())
    }

    async fn assert_document_fields_readback(
        server: &DockerLumen,
        document: &SemanticDocument,
        phase: &str,
    ) -> Result<()> {
        let fields = document_fields(document.number, document.tag);
        let fingerprint = fields["fingerprint"].as_str().ok_or_else(|| {
            HarnessError::DataInvariant("semantic fixture lacks string fingerprint".to_owned())
        })?;
        for field in FIELD_NAMES {
            if field == "embedding" {
                continue;
            }
            let value = fields.get(field).ok_or_else(|| {
                HarnessError::DataInvariant(format!("semantic fixture lacks frozen field {field}"))
            })?;
            let query = semantic_field_query(field, value, fingerprint)?;
            let context = format!("{phase}: expected {field} for {}", document.external_id);
            let response = semantic_search(server, query, &context).await?;
            if !response_contains_id(&response, &document.external_id) {
                return Err(HarnessError::DataInvariant(format!(
                    "{context}: expected target was not searchable: {response}"
                )));
            }

            // The fingerprint holds this probe to the known target. A search
            // implementation that ignores this field predicate would still
            // return that target, so the disjoint value must exclude it.
            let mismatch = semantic_wrong_field_query(field, value, fingerprint, document.number)?;
            let mismatch_context = format!(
                "{phase}: wrong {field} must exclude {}",
                document.external_id
            );
            let mismatch_response = semantic_search(server, mismatch, &mismatch_context).await?;
            if response_contains_id(&mismatch_response, &document.external_id) {
                return Err(HarnessError::DataInvariant(format!(
                    "{mismatch_context}: ignored field predicate retained target: {mismatch_response}"
                )));
            }
        }
        assert_bounded_embedding_readback(
            server,
            document,
            &fields,
            &vector_readback_reference(),
            phase,
        )
        .await
    }

    async fn assert_deleted_document_readback(
        server: &DockerLumen,
        document: &SemanticDocument,
        phase: &str,
    ) -> Result<()> {
        let fields = document_fields(document.number, document.tag);
        let fingerprint = fields["fingerprint"].as_str().ok_or_else(|| {
            HarnessError::DataInvariant("semantic fixture lacks string fingerprint".to_owned())
        })?;
        let context = format!("{phase}: deleted {}", document.external_id);
        let response = semantic_search(server, fingerprint_query(fingerprint), &context).await?;
        if response_contains_id(&response, &document.external_id) {
            return Err(HarnessError::DataInvariant(format!(
                "{context}: deleted target remained searchable: {response}"
            )));
        }
        Ok(())
    }

    async fn assert_mutation_readback(server: &DockerLumen, phase: &str) -> Result<()> {
        // These bounded queries run after the measured input and after its
        // metric snapshot. They prove acknowledged target/content effects but
        // never contribute to the ten-QPS workload ledger.
        let oracle = mutation_readback();
        assert_document_fields_readback(server, &oracle.indexed, phase).await?;
        assert_document_fields_readback(server, &oracle.replaced, phase).await?;
        assert_deleted_document_readback(server, &oracle.deleted, phase).await
    }

    async fn post_input_step<F, T>(
        deadline: tokio::time::Instant,
        timeout: Duration,
        stage: &'static str,
        operation: F,
    ) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        tokio::time::timeout_at(deadline, operation)
            .await
            .map_err(|_| HarnessError::PostInputTimeout { stage, timeout })?
    }

    fn restart_post_input_window(
        deadline: tokio::time::Instant,
        recovery_observation: Option<Duration>,
    ) -> Result<(tokio::time::Instant, Duration)> {
        let extension = recovery_observation
            .unwrap_or_default()
            .saturating_sub(STARTUP_TIMEOUT);
        let deadline = deadline.checked_add(extension).ok_or_else(|| {
            HarnessError::DataInvariant("diagnostic restart deadline overflowed".to_owned())
        })?;
        let timeout = POST_INPUT_TIMEOUT.checked_add(extension).ok_or_else(|| {
            HarnessError::DataInvariant("diagnostic restart timeout overflowed".to_owned())
        })?;
        Ok((deadline, timeout))
    }

    fn setup_timeout() -> HarnessError {
        HarnessError::SetupTimeout { stage: "seed" }
    }

    fn check_setup_deadline(deadline: tokio::time::Instant) -> Result<()> {
        if tokio::time::Instant::now() >= deadline {
            Err(setup_timeout())
        } else {
            Ok(())
        }
    }

    async fn setup_step<F, T>(deadline: tokio::time::Instant, operation: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        check_setup_deadline(deadline)?;
        tokio::time::timeout_at(deadline, operation)
            .await
            .map_err(|_| setup_timeout())?
    }

    async fn input_window_step<F, T>(
        deadline: tokio::time::Instant,
        timeout: Duration,
        stage: &'static str,
        operation: F,
    ) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        check_input_deadline(deadline, timeout, stage)?;
        tokio::time::timeout_at(deadline, operation)
            .await
            .map_err(|_| HarnessError::InputWindowTimeout { stage, timeout })?
    }

    fn check_input_deadline(
        deadline: tokio::time::Instant,
        timeout: Duration,
        stage: &'static str,
    ) -> Result<()> {
        if tokio::time::Instant::now() >= deadline {
            Err(HarnessError::InputWindowTimeout { stage, timeout })
        } else {
            Ok(())
        }
    }

    async fn input_window_step_with_sampler<F, T>(
        sampler_abort: &AbortHandle,
        deadline: tokio::time::Instant,
        timeout: Duration,
        stage: &'static str,
        operation: F,
    ) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        let result = input_window_step(deadline, timeout, stage, operation).await;
        if result.is_err() {
            sampler_abort.abort();
        }
        result
    }

    fn interval_metrics_timeout(stage: &'static str, timeout: Duration) -> HarnessError {
        HarnessError::RequestFailure(RequestFailure::synthetic(
            format!("interval metrics {stage} exceeded the {timeout:?} deadline"),
            true,
        ))
    }

    async fn fetch_interval_metrics(
        client: &reqwest::Client,
        url: String,
        timeout: Duration,
    ) -> Result<(reqwest::StatusCode, String)> {
        let response = tokio::time::timeout(timeout, client.get(url).send())
            .await
            .map_err(|_| interval_metrics_timeout("request", timeout))?
            .map_err(HarnessError::request_failure)?;
        let status = response.status();
        let body = tokio::time::timeout(timeout, response.text())
            .await
            .map_err(|_| interval_metrics_timeout("body", timeout))?
            .map_err(HarnessError::request_failure)?;
        Ok((status, body))
    }

    fn runtime_sample(elapsed: Duration, counters: RuntimeCounters, metrics: &str) -> String {
        let pending = metrics
            .lines()
            .filter(|line| line.starts_with("lumen_pending_change_"))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "PERF_RUNTIME_SAMPLE elapsed_ms={} counters={counters:?} pending=[{pending}]",
            elapsed.as_millis()
        )
    }

    async fn drive_workload(
        server: &DockerLumen,
        config: CaseConfig,
    ) -> Result<(WorkloadReport, RuntimeCounters, Duration)> {
        let ledger = Arc::new(Mutex::new(WorkloadLedger::new(config.ledger_case())));
        ledger
            .lock()
            .await
            .set_input_window(Duration::ZERO, Duration::from_secs(INPUT_SECONDS));
        let clock = Clock::new();
        let input_window = Duration::from_secs(INPUT_SECONDS);
        let input_deadline = clock.deadline(input_window);
        // Only samples completed inside the input window may prove that
        // checkpoint and compaction happened under the offered workload.
        let sampler_client = server.client.clone();
        let sampler_base = server.base.clone();
        let sampler_clock = clock.clone();
        let sampler_trace = server.interval_trace.clone();
        let sampler = tokio::spawn(async move {
            let mut first = None;
            let mut last = None;
            let mut next_log = Duration::ZERO;
            let mut next_trace = Duration::ZERO;
            let input_end = input_window;
            while sampler_clock.elapsed() < input_end {
                let (status, metrics) = fetch_interval_metrics(
                    &sampler_client,
                    format!("{sampler_base}/metrics"),
                    REQUEST_TIMEOUT,
                )
                .await?;
                if !status.is_success() {
                    return Err(HarnessError::Http(format!(
                        "interval metrics returned {status}"
                    )));
                }
                let counters = RuntimeCounters::parse(&metrics)?;
                if sampler_clock.elapsed() >= input_end {
                    break;
                }
                // Reuse the required scrape. These bounded diagnostic lines
                // survive a failed ledger gate and do not affect its decision.
                let elapsed = sampler_clock.elapsed();
                if elapsed >= next_log {
                    eprintln!("{}", runtime_sample(elapsed, counters, &metrics));
                    next_log = elapsed + Duration::from_secs(10);
                }
                if elapsed >= next_trace {
                    sampler_trace
                        .lock()
                        .await
                        .record(elapsed, "input", &metrics);
                    next_trace = elapsed + INTERVAL_TRACE_CADENCE;
                }
                first.get_or_insert(counters);
                last = Some(counters);
                let next = (sampler_clock.elapsed() + Duration::from_secs(1)).min(input_end);
                tokio::time::sleep_until(sampler_clock.deadline(next)).await;
            }
            let first = first.ok_or_else(|| {
                HarnessError::DataInvariant(
                    "no runtime metric sample completed within measured input".to_owned(),
                )
            })?;
            let last = last.expect("first and last sample are set together");
            last.delta(first)
        });
        let sampler_abort = sampler.abort_handle();

        let mut mutations = RequestPump::new(REQUEST_CONCURRENCY);
        let mut queries = RequestPump::new(QUERY_CONCURRENCY);
        let mut index_batch = Batcher::new(config.batch_size_for(Endpoint::Index));
        let mut replace_batch = Batcher::new(config.batch_size_for(Endpoint::Replace));
        let mut unindex_batch = Batcher::new(config.batch_size_for(Endpoint::Unindex));
        let mut operation_id = 1u64;
        let mut request_id = 1u64;

        for second in 0..INPUT_SECONDS {
            let input_step = input_window_step_with_sampler(
                &sampler_abort,
                input_deadline,
                input_window,
                "input_workload",
                async {
                    tokio::time::sleep_until(clock.deadline(Duration::from_secs(second))).await;
                    // Offer the independently scheduled queries before mutation
                    // preparation. A slow mutation pump must not postpone their
                    // absolute 100 ms slots.
                    offer_queries(
                        &clock,
                        &ledger,
                        &mut queries,
                        server,
                        second,
                        input_deadline,
                        input_window,
                    )
                    .await?;
                    match second % 3 {
                        0 => {
                            offer_additions(
                                &ledger,
                                &clock,
                                second,
                                &mut operation_id,
                                &mut index_batch,
                                &mut mutations,
                                &mut request_id,
                                server,
                                input_deadline,
                                input_window,
                            )
                            .await?;
                        }
                        1 => {
                            offer_replacements(
                                &ledger,
                                &clock,
                                second,
                                &mut operation_id,
                                &mut replace_batch,
                                &mut mutations,
                                &mut request_id,
                                server,
                                input_deadline,
                                input_window,
                            )
                            .await?;
                        }
                        _ => {
                            offer_deletions(
                                &ledger,
                                &clock,
                                second,
                                &mut operation_id,
                                &mut unindex_batch,
                                &mut mutations,
                                &mut request_id,
                                server,
                                input_deadline,
                                input_window,
                            )
                            .await?;
                        }
                    }
                    Ok(())
                },
            )
            .await;
            input_step?;
        }

        // The receipt records this elapsed duration. The ledger keeps the
        // fixed logical window for per-second classification, while this
        // clock observation proves the driver reached its final input edge.
        tokio::time::sleep_until(clock.deadline(input_window)).await;
        let observed_input_duration = clock.elapsed();
        if observed_input_duration < input_window {
            return Err(HarnessError::DataInvariant(format!(
                "input driver stopped before the approved duration: observed={observed_input_duration:?} required={INPUT_SECONDS}s"
            )));
        }

        if !(index_batch.is_empty() && replace_batch.is_empty() && unindex_batch.is_empty()) {
            return Err(HarnessError::DataInvariant(
                "approved 60,000-operation thirds must fill every selected batch exactly"
                    .to_owned(),
            ));
        }
        let post_input_deadline = tokio::time::Instant::now()
            .checked_add(POST_INPUT_TIMEOUT)
            .ok_or_else(|| {
                HarnessError::DataInvariant(
                    "post-input deadline overflowed the Tokio instant range".to_owned(),
                )
            })?;
        eprintln!("PERF_STAGE_BEGIN workload_drain");
        let workload_drain = post_input_step(
            post_input_deadline,
            POST_INPUT_TIMEOUT,
            "workload_drain",
            async move {
                eprintln!("PERF_STAGE_BEGIN sampler_drain");
                let mut deltas = match tokio::time::timeout(DRAIN_TIMEOUT, sampler).await {
                    Ok(result) => result.map_err(|error| HarnessError::Task(error.to_string()))??,
                    Err(_) => {
                        return Err(HarnessError::PostInputTimeout {
                            stage: "workload_drain",
                            timeout: DRAIN_TIMEOUT,
                        });
                    }
                };
                eprintln!("PERF_STAGE_END sampler_drain");
                eprintln!("PERF_STAGE_BEGIN mutation_drain");
                mutations.drain("mutation requests").await?;
                eprintln!("PERF_STAGE_END mutation_drain");
                eprintln!("PERF_STAGE_BEGIN query_drain");
                queries.drain("query requests").await?;
                eprintln!("PERF_STAGE_END query_drain");

                eprintln!("PERF_STAGE_BEGIN post_drain_metrics");
                let metrics_after = server.metrics().await?;
                eprintln!("PERF_STAGE_END post_drain_metrics");
                let counters_after = RuntimeCounters::parse(&metrics_after)?;
                // VmHWM remains a peak through the drain, while completion/IO evidence
                // above remains strictly within measured input.
                deltas.process_rss_high_water_bytes = counters_after.process_rss_high_water_bytes;
                eprintln!("PERF_RUNTIME_DELTA {deltas:?}");
                deltas.assert_complete_interval_evidence()?;
                let mut ledger = ledger.lock().await;
                for _ in 0..deltas.checkpoints {
                    ledger.record_checkpoint_completion();
                }
                for _ in 0..deltas.merges {
                    ledger.record_merge_completion();
                }
                ledger.observe_peak_rss_bytes(deltas.process_rss_high_water_bytes);
                let report = ledger.validate().map_err(HarnessError::Workload)?;
                eprintln!(
        "interval durable evidence: checkpoint_duration_s={} capture_lock_s={} checkpoint_bytes={} merge_read_bytes={} merge_write_bytes={} pending_delta_bytes={} pending_delta_layers={} backpressure_events={} disk_bytes={} ledger_docops={}",
        deltas.checkpoint_duration_seconds,
        deltas.capture_lock_duration_seconds,
        deltas.checkpoint_bytes,
        deltas.merge_read_bytes,
        deltas.merge_write_bytes,
        deltas.pending_delta_bytes,
        deltas.pending_delta_layers,
        deltas.backpressure_events,
        deltas.segment_disk_bytes,
        report.docops_completed_in_input,
    );
                Ok((report, deltas))
            },
        )
        .await;
        if matches!(
            &workload_drain,
            Err(HarnessError::PostInputTimeout {
                stage: "workload_drain",
                ..
            })
        ) {
            sampler_abort.abort();
        }
        eprintln!("PERF_STAGE_END workload_drain");
        let (report, deltas) = workload_drain?;
        Ok((report, deltas, observed_input_duration))
    }

    fn absolute_slot(second: u64, offset: usize, cadence: Duration) -> Duration {
        Duration::from_secs(second)
            .checked_add(
                cadence
                    .checked_mul(offset.try_into().expect("slot offset fits u32"))
                    .expect("slot offset cannot overflow duration"),
            )
            .expect("workload slot cannot overflow duration")
    }

    fn mutation_slot(second: u64, offset: usize) -> Duration {
        absolute_slot(second, offset, MUTATION_SLOT)
    }

    fn query_slot(second: u64, offset: usize) -> Duration {
        absolute_slot(second, offset, QUERY_SLOT)
    }

    async fn offer_additions(
        ledger: &Arc<Mutex<WorkloadLedger>>,
        clock: &Clock,
        second: u64,
        operation_id: &mut u64,
        batch: &mut Batcher<IndexedField>,
        pump: &mut RequestPump,
        request_id: &mut u64,
        server: &DockerLumen,
        input_deadline: tokio::time::Instant,
        input_timeout: Duration,
    ) -> Result<()> {
        let add_base = second / 3 * DOCOPS_PER_SECOND as u64;
        for offset in 0..DOCOPS_PER_SECOND {
            let scheduled_at = mutation_slot(second, offset);
            tokio::time::sleep_until(clock.deadline(scheduled_at)).await;
            check_input_deadline(input_deadline, input_timeout, "input_workload")?;
            let operation = *operation_id;
            *operation_id += 1;
            let external_id = format!("hot-added-{:06}", add_base + offset as u64);
            let fields = document_fields(HOT_DOCUMENTS + add_base as usize + offset, "hot");
            let operation_record =
                approved_index_operation(operation, external_id.clone(), &fields)?;
            ledger
                .lock()
                .await
                .begin_operation(clock.elapsed(), operation_record);
            for (field, value) in fields {
                check_input_deadline(input_deadline, input_timeout, "input_workload")?;
                batch.push(IndexedField {
                    operation,
                    field: field.clone(),
                    body: json!({
                        "external_id": external_id,
                        "field": field,
                        "value": value,
                    }),
                });
            }
            dispatch_full_index_batches(
                batch,
                pump,
                request_id,
                ledger,
                clock,
                server,
                scheduled_at,
                input_deadline,
                input_timeout,
            )
            .await?;
        }
        Ok(())
    }

    async fn offer_replacements(
        ledger: &Arc<Mutex<WorkloadLedger>>,
        clock: &Clock,
        second: u64,
        operation_id: &mut u64,
        batch: &mut Batcher<Replacement>,
        pump: &mut RequestPump,
        request_id: &mut u64,
        server: &DockerLumen,
        input_deadline: tokio::time::Instant,
        input_timeout: Duration,
    ) -> Result<()> {
        let update_base = second / 3 * DOCOPS_PER_SECOND as u64;
        for offset in 0..DOCOPS_PER_SECOND {
            let scheduled_at = mutation_slot(second, offset);
            tokio::time::sleep_until(clock.deadline(scheduled_at)).await;
            check_input_deadline(input_deadline, input_timeout, "input_workload")?;
            let operation = *operation_id;
            *operation_id += 1;
            let number = 60_000 + update_base as usize + offset;
            let external_id = format!("hot-base-{number:06}");
            ledger.lock().await.begin_operation(
                clock.elapsed(),
                DocumentOperation::replace(operation, external_id.clone()),
            );
            batch.push(Replacement {
                operation,
                body: json!({
                    "external_id": external_id,
                    "fields": document_fields(number, "hot-updated"),
                }),
            });
            dispatch_full_replace_batches(
                batch,
                pump,
                request_id,
                ledger,
                clock,
                server,
                scheduled_at,
                input_deadline,
                input_timeout,
            )
            .await?;
        }
        Ok(())
    }

    async fn offer_deletions(
        ledger: &Arc<Mutex<WorkloadLedger>>,
        clock: &Clock,
        second: u64,
        operation_id: &mut u64,
        batch: &mut Batcher<Removal>,
        pump: &mut RequestPump,
        request_id: &mut u64,
        server: &DockerLumen,
        input_deadline: tokio::time::Instant,
        input_timeout: Duration,
    ) -> Result<()> {
        let delete_base = second / 3 * DOCOPS_PER_SECOND as u64;
        for offset in 0..DOCOPS_PER_SECOND {
            let scheduled_at = mutation_slot(second, offset);
            tokio::time::sleep_until(clock.deadline(scheduled_at)).await;
            check_input_deadline(input_deadline, input_timeout, "input_workload")?;
            let operation = *operation_id;
            *operation_id += 1;
            let external_id = format!("hot-base-{:06}", delete_base + offset as u64);
            ledger.lock().await.begin_operation(
                clock.elapsed(),
                DocumentOperation::unindex(operation, external_id.clone()),
            );
            batch.push(Removal {
                operation,
                external_id,
            });
            dispatch_full_unindex_batches(
                batch,
                pump,
                request_id,
                ledger,
                clock,
                server,
                scheduled_at,
                input_deadline,
                input_timeout,
            )
            .await?;
        }
        Ok(())
    }

    async fn dispatch_full_index_batches(
        batch: &mut Batcher<IndexedField>,
        pump: &mut RequestPump,
        request_id: &mut u64,
        ledger: &Arc<Mutex<WorkloadLedger>>,
        clock: &Clock,
        server: &DockerLumen,
        scheduled_at: Duration,
        input_deadline: tokio::time::Instant,
        input_timeout: Duration,
    ) -> Result<()> {
        while let Some(entries) = batch.take_full() {
            check_input_deadline(input_deadline, input_timeout, "input_workload")?;
            let id = *request_id;
            *request_id += 1;
            pump.push(
                send_index(
                    server.client.clone(),
                    server.base.clone(),
                    ledger.clone(),
                    clock.clone(),
                    server.request_error_journal.clone(),
                    id,
                    scheduled_at,
                    entries,
                ),
                input_deadline,
                input_timeout,
            )
            .await?;
        }
        Ok(())
    }

    async fn dispatch_full_replace_batches(
        batch: &mut Batcher<Replacement>,
        pump: &mut RequestPump,
        request_id: &mut u64,
        ledger: &Arc<Mutex<WorkloadLedger>>,
        clock: &Clock,
        server: &DockerLumen,
        scheduled_at: Duration,
        input_deadline: tokio::time::Instant,
        input_timeout: Duration,
    ) -> Result<()> {
        while let Some(entries) = batch.take_full() {
            check_input_deadline(input_deadline, input_timeout, "input_workload")?;
            let id = *request_id;
            *request_id += 1;
            pump.push(
                send_replace(
                    server.client.clone(),
                    server.base.clone(),
                    ledger.clone(),
                    clock.clone(),
                    server.request_error_journal.clone(),
                    id,
                    scheduled_at,
                    entries,
                ),
                input_deadline,
                input_timeout,
            )
            .await?;
        }
        Ok(())
    }

    async fn dispatch_full_unindex_batches(
        batch: &mut Batcher<Removal>,
        pump: &mut RequestPump,
        request_id: &mut u64,
        ledger: &Arc<Mutex<WorkloadLedger>>,
        clock: &Clock,
        server: &DockerLumen,
        scheduled_at: Duration,
        input_deadline: tokio::time::Instant,
        input_timeout: Duration,
    ) -> Result<()> {
        while let Some(entries) = batch.take_full() {
            check_input_deadline(input_deadline, input_timeout, "input_workload")?;
            let id = *request_id;
            *request_id += 1;
            pump.push(
                send_unindex(
                    server.client.clone(),
                    server.base.clone(),
                    ledger.clone(),
                    clock.clone(),
                    server.request_error_journal.clone(),
                    id,
                    scheduled_at,
                    entries,
                ),
                input_deadline,
                input_timeout,
            )
            .await?;
        }
        Ok(())
    }

    async fn offer_queries(
        clock: &Clock,
        ledger: &Arc<Mutex<WorkloadLedger>>,
        pump: &mut RequestPump,
        server: &DockerLumen,
        second: u64,
        input_deadline: tokio::time::Instant,
        input_timeout: Duration,
    ) -> Result<()> {
        for offset in 0..QUERY_QPS {
            check_input_deadline(input_deadline, input_timeout, "input_workload")?;
            let scheduled_at = query_slot(second, offset);
            let (class, collection, body) = query_for(second, offset);
            let client = server.client.clone();
            let base = server.base.clone();
            let ledger = ledger.clone();
            let clock = clock.clone();
            let journal = server.request_error_journal.clone();
            pump.push(
                async move {
                    tokio::time::sleep_until(clock.deadline(scheduled_at)).await;
                    send_query(
                        client,
                        base,
                        ledger,
                        clock,
                        journal,
                        scheduled_at,
                        class,
                        collection,
                        body,
                    )
                    .await;
                },
                input_deadline,
                input_timeout,
            )
            .await?;
        }
        Ok(())
    }

    fn schema(backend: VectorBackend) -> Value {
        assert_eq!(
            NGRAM_TEXT_FIELDS, 3,
            "the receipt must bind the three n-gram Text fields in the frozen schema"
        );
        json!({
            "fields": {
                "tag": { "type": "keyword" },
                "category": { "type": "keyword" },
                "status": { "type": "keyword" },
                "price": { "type": "number" },
                "rank": { "type": "number" },
                "labels": { "type": "set" },
                "fingerprint": { "type": "hash" },
                "title_ngram": { "type": "text", "analyzer": "ngram" },
                "body_ngram": { "type": "text", "analyzer": "ngram" },
                "summary_ngram": { "type": "text", "analyzer": "ngram" },
                "title_text": { "type": "text", "analyzer": "whitespace_lower" },
                "body_text": { "type": "text", "analyzer": "whitespace_lower" },
                "embedding": {
                    "type": "vector",
                    "dim": VECTOR_DIMENSIONS,
                    "metric": "cosine",
                    "backend": backend.wire_name(),
                },
                "region": { "type": "keyword" },
            }
        })
    }

    fn document_fields(number: usize, tag: &str) -> Map<String, Value> {
        let mut fields = Map::new();
        fields.insert("tag".to_owned(), json!(tag));
        fields.insert(
            "category".to_owned(),
            json!(format!("category-{}", number % 17)),
        );
        fields.insert(
            "status".to_owned(),
            json!(if number % 2 == 0 { "active" } else { "pending" }),
        );
        fields.insert("price".to_owned(), json!(number as f64 + 0.25));
        fields.insert("rank".to_owned(), json!((number % 10_000) as f64));
        fields.insert(
            "labels".to_owned(),
            json!([
                format!("label-{}", number % 13),
                format!("group-{}", number % 7)
            ]),
        );
        fields.insert("fingerprint".to_owned(), json!(format!("{number:016x}")));
        fields.insert("title_ngram".to_owned(), json!(ngram_text(number, 0)));
        fields.insert("body_ngram".to_owned(), json!(ngram_text(number, 1)));
        fields.insert("summary_ngram".to_owned(), json!(ngram_text(number, 2)));
        fields.insert(
            "title_text".to_owned(),
            json!(format!("title {tag} {number}")),
        );
        fields.insert(
            "body_text".to_owned(),
            json!(format!("body {tag} {}", number % 101)),
        );
        fields.insert("embedding".to_owned(), json!(embedding(number)));
        fields.insert("region".to_owned(), json!(format!("region-{}", number % 5)));
        assert_eq!(
            fields.len(),
            FIELD_COUNT,
            "fixture must index all fourteen fields"
        );
        fields
    }

    /// Builds the ledger record from the approved schema, never from a partial
    /// request body. A malformed fixture must fail before it can under-count an
    /// Index document as complete.
    fn approved_index_operation(
        operation: u64,
        external_id: String,
        fields: &Map<String, Value>,
    ) -> Result<DocumentOperation> {
        let expected = FIELD_NAMES
            .iter()
            .map(|field| (*field).to_owned())
            .collect::<BTreeSet<_>>();
        let actual = fields.keys().cloned().collect::<BTreeSet<_>>();
        if actual != expected {
            let missing = expected.difference(&actual).cloned().collect::<Vec<_>>();
            let extra = actual.difference(&expected).cloned().collect::<Vec<_>>();
            return Err(HarnessError::DataInvariant(format!(
            "approved Index operation must contain exactly {FIELD_COUNT} frozen fields; missing={missing:?} extra={extra:?}"
        )));
        }
        Ok(DocumentOperation::index(
            operation,
            external_id,
            FIELD_NAMES.iter().map(|field| (*field).to_owned()),
        ))
    }

    fn ngram_text(number: usize, slot: usize) -> String {
        let target = 250 + (seeded_value(number, slot) % 36) as usize;
        let mut text = format!("ngram document {number} slot {slot} ");
        while text.len() < target {
            text.push_str("durable search token ");
        }
        text.truncate(target);
        text
    }

    fn embedding(number: usize) -> Vec<f32> {
        (0..VECTOR_DIMENSIONS as usize)
            .map(|dimension| embedding_numerator(number, dimension) as f32 / 997.0)
            .collect()
    }

    fn seeded_value(number: usize, slot: usize) -> u64 {
        WORKLOAD_SEED
            .wrapping_add((number as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15))
            .rotate_left(((slot * 13) % 64) as u32)
            .wrapping_mul(0xd6e8_feb8_6659_fd93)
    }

    fn idle_collection(number: usize) -> String {
        format!("perf-idle-{number:03}")
    }

    fn query_for(second: u64, offset: usize) -> (QueryClass, String, Value) {
        match offset % 5 {
            0 | 1 => (
                QueryClass::Hot,
                HOT_COLLECTION.to_owned(),
                json!({
                    "query": { "term": { "field": "tag", "value": "hot" } },
                    "limit": 10,
                }),
            ),
            2 => (
                QueryClass::Hot,
                HOT_COLLECTION.to_owned(),
                json!({
                    "query": {
                        "match": {
                            "field": "title_ngram",
                            "text": "durable search",
                            "op": "and",
                        }
                    },
                    "limit": 10,
                }),
            ),
            3 => (
                QueryClass::Hot,
                HOT_COLLECTION.to_owned(),
                json!({
                    "query": {
                        "knn": {
                            "field": "embedding",
                            "vector": embedding((second as usize * 10 + offset) % HOT_DOCUMENTS),
                            "k": 10,
                        }
                    },
                    "limit": 10,
                }),
            ),
            _ => (
                QueryClass::Idle,
                idle_collection((second as usize + offset) % IDLE_COLLECTIONS),
                json!({
                    "query": { "term": { "field": "tag", "value": "idle" } },
                    "limit": 10,
                }),
            ),
        }
    }

    fn emit_rate_report(report: &WorkloadReport, observed_input_duration: Duration) -> Result<()> {
        let input_seconds = report
            .input_duration
            .ok_or_else(|| {
                HarnessError::DataInvariant(
                    "cannot report rates without the fixed input membership window".to_owned(),
                )
            })?
            .as_secs_f64();
        if input_seconds <= 0.0 {
            return Err(HarnessError::DataInvariant(
                "cannot report rates without a positive observed input interval".to_owned(),
            ));
        }
        eprintln!(
            "approved workload rates membership_seconds={input_seconds} observed_input_seconds={} input_requests_started_per_s={} input_requests_completed_per_s={} input_items_started_per_s={} input_items_completed_per_s={} input_docops_started_per_s={} input_docops_completed_per_s={} input_queries_started_per_s={} input_queries_completed_per_s={} input_index_requests={} input_replace_requests={} input_unindex_requests={} input_hot_queries={} input_idle_queries={}",
            observed_input_duration.as_secs_f64(),
            report.requests_started_in_input as f64 / input_seconds,
            report.requests_completed_in_input as f64 / input_seconds,
            report.items_started_in_input as f64 / input_seconds,
            report.items_completed_in_input as f64 / input_seconds,
            report.docops_offered_in_input as f64 / input_seconds,
            report.docops_completed_in_input as f64 / input_seconds,
            report.queries_started_in_input as f64 / input_seconds,
            report.queries_completed_in_input as f64 / input_seconds,
            report.index_requests_started_in_input,
            report.replace_requests_started_in_input,
            report.unindex_requests_started_in_input,
            report.hot_queries_started_in_input,
            report.idle_queries_started_in_input,
        );
        Ok(())
    }

    #[derive(Debug)]
    struct CompletedCase {
        report: WorkloadReport,
        counter_delta: RuntimeCounters,
        observed_input_duration: Duration,
        restart_duration: Duration,
        live_mutation_readback: bool,
        cold_mutation_readback: bool,
        image_reference: String,
        actual_image_id: String,
    }

    fn duration_millis(name: &'static str, duration: Duration) -> Result<u64> {
        u64::try_from(duration.as_millis()).map_err(|_| {
            HarnessError::DataInvariant(format!(
                "{name} duration does not fit receipt milliseconds"
            ))
        })
    }

    fn required_duration_millis(name: &'static str, duration: Option<Duration>) -> Result<u64> {
        duration
            .ok_or_else(|| {
                HarnessError::DataInvariant(format!(
                    "{name} has no completed latency or drain observation"
                ))
            })
            .and_then(|duration| duration_millis(name, duration))
    }

    fn seconds_to_nanos(name: &'static str, seconds: f64) -> Result<u64> {
        let nanos = seconds * 1_000_000_000.0;
        if !nanos.is_finite() || nanos < 0.0 || nanos > u64::MAX as f64 {
            return Err(HarnessError::DataInvariant(format!(
                "{name} cannot be represented as receipt nanoseconds: {seconds}"
            )));
        }
        Ok(nanos.round() as u64)
    }

    fn receipt_for(
        context: &QualifyingContext,
        config: CaseConfig,
        completed: &CompletedCase,
    ) -> Result<Receipt> {
        if completed.image_reference != context.image_reference {
            return Err(HarnessError::DataInvariant(format!(
                "qualifying image changed from {} to {}",
                context.image_reference, completed.image_reference
            )));
        }
        let report = &completed.report;
        let input_duration_ms =
            required_duration_millis("input membership", report.input_duration)?;
        let observed_input_elapsed_ms =
            duration_millis("observed input", completed.observed_input_duration)?;
        let measurement = ReceiptMeasurement {
            input_duration_ms,
            observed_input_elapsed_ms,
            requests_offered: report.requests_offered,
            requests_finished: report.requests_finished,
            requests_completed: report.requests_completed,
            requests_started_in_input: report.requests_started_in_input,
            requests_finished_in_input: report.requests_finished_in_input,
            requests_completed_in_input: report.requests_completed_in_input,
            requests_started_per_second_milli: rate_milli(
                report.requests_started_in_input,
                input_duration_ms,
            )
            .map_err(|error| HarnessError::DataInvariant(error.to_string()))?,
            requests_completed_per_second_milli: rate_milli(
                report.requests_completed_in_input,
                input_duration_ms,
            )
            .map_err(|error| HarnessError::DataInvariant(error.to_string()))?,
            request_errors: report.request_errors,
            client_cancellations: report.client_cancellations,
            items_offered: report.items_offered,
            items_completed: report.items_completed,
            items_failed: report.items_failed,
            items_started_in_input: report.items_started_in_input,
            items_completed_in_input: report.items_completed_in_input,
            items_started_per_second_milli: rate_milli(
                report.items_started_in_input,
                input_duration_ms,
            )
            .map_err(|error| HarnessError::DataInvariant(error.to_string()))?,
            items_completed_per_second_milli: rate_milli(
                report.items_completed_in_input,
                input_duration_ms,
            )
            .map_err(|error| HarnessError::DataInvariant(error.to_string()))?,
            docops_offered: report.docops_offered,
            docops_completed: report.docops_completed,
            docops_offered_in_input: report.docops_offered_in_input,
            docops_completed_in_input: report.docops_completed_in_input,
            index_requests_started_in_input: report.index_requests_started_in_input,
            replace_requests_started_in_input: report.replace_requests_started_in_input,
            unindex_requests_started_in_input: report.unindex_requests_started_in_input,
            docops_offered_per_second_milli: rate_milli(
                report.docops_offered_in_input,
                input_duration_ms,
            )
            .map_err(|error| HarnessError::DataInvariant(error.to_string()))?,
            docops_completed_per_second_milli: rate_milli(
                report.docops_completed_in_input,
                input_duration_ms,
            )
            .map_err(|error| HarnessError::DataInvariant(error.to_string()))?,
            docops_completion_percent_milli: percent_milli(
                report.docops_completed_in_input,
                report.docops_offered_in_input,
            )
            .map_err(|error| HarnessError::DataInvariant(error.to_string()))?,
            request_latency_p99_ms: required_duration_millis(
                "request p99",
                report.request_latency_p99,
            )?,
            request_latency_max_ms: required_duration_millis(
                "request maximum",
                report.request_latency_max,
            )?,
            queries_offered: report.queries_offered,
            queries_completed: report.queries_completed,
            queries_started_in_input: report.queries_started_in_input,
            queries_completed_in_input: report.queries_completed_in_input,
            hot_queries_started_in_input: report.hot_queries_started_in_input,
            idle_queries_started_in_input: report.idle_queries_started_in_input,
            queries_started_per_second_milli: rate_milli(
                report.queries_started_in_input,
                input_duration_ms,
            )
            .map_err(|error| HarnessError::DataInvariant(error.to_string()))?,
            queries_completed_per_second_milli: rate_milli(
                report.queries_completed_in_input,
                input_duration_ms,
            )
            .map_err(|error| HarnessError::DataInvariant(error.to_string()))?,
            query_errors_or_timeouts: report.query_errors_or_timeouts,
            query_latency_p99_ms: required_duration_millis("query p99", report.query_latency_p99)?,
            query_latency_max_ms: required_duration_millis(
                "query maximum",
                report.query_latency_max,
            )?,
            request_drain_ms: required_duration_millis("request drain", report.request_drain)?,
            query_drain_ms: required_duration_millis("query drain", report.query_drain)?,
            checkpoint_delta: completed.counter_delta.checkpoints,
            merge_delta: completed.counter_delta.merges,
            checkpoint_bytes: completed.counter_delta.checkpoint_bytes,
            merge_read_bytes: completed.counter_delta.merge_read_bytes,
            merge_write_bytes: completed.counter_delta.merge_write_bytes,
            // The runtime exports a cumulative hold duration and count. It
            // does not export a true maximum, so the receipt makes no max claim.
            capture_hold_ns_total: seconds_to_nanos(
                "capture hold total",
                completed.counter_delta.capture_lock_duration_seconds,
            )?,
            pending_delta_bytes: completed.counter_delta.pending_delta_bytes,
            pending_delta_layers: completed.counter_delta.pending_delta_layers,
            backpressure_events: completed.counter_delta.backpressure_events,
            segment_disk_bytes: completed.counter_delta.segment_disk_bytes,
            peak_rss_bytes: completed.counter_delta.process_rss_high_water_bytes,
            restart_duration_ms: duration_millis("restart", completed.restart_duration)?,
            restart_recovered: true,
            live_mutation_readback: completed.live_mutation_readback,
            cold_mutation_readback: completed.cold_mutation_readback,
        };
        let receipt = Receipt {
            schema_version: perf_cell_receipt::SCHEMA_VERSION,
            kind: perf_cell_receipt::RECEIPT_KIND.to_owned(),
            binding: context.binding(completed.actual_image_id.clone()),
            cell: config.receipt_cell(),
            limits: ReceiptLimits::approved(),
            measurement,
            outcome: ReceiptOutcome {
                qualifying: true,
                diagnostic: false,
                succeeded: true,
            },
        };
        receipt
            .validate()
            .map_err(|error| HarnessError::DataInvariant(error.to_string()))?;
        Ok(receipt)
    }

    async fn run_case(config: CaseConfig) -> Result<CompletedCase> {
        let mut server = DockerLumen::start().await?;
        let result = run_case_with_server(&mut server, config).await;
        finish_case_result(&mut server, result).await
    }

    async fn finish_case_result<T>(server: &mut DockerLumen, result: Result<T>) -> Result<T> {
        if let Err(error) = &result {
            server.finish_failure(error).await;
        }
        result
    }

    async fn finish_case_result_with<T, E, C>(
        server: &mut DockerLumen,
        result: Result<T>,
        evidence_root: &Path,
        evidence_runner: &mut E,
        cleanup_runner: &mut C,
    ) -> Result<T>
    where
        E: EvidenceCommandRunner + Send,
        C: CleanupCommandRunner,
    {
        if let Err(error) = &result {
            server
                .finish_failure_with(error, evidence_root, evidence_runner, cleanup_runner)
                .await;
        }
        result
    }

    async fn run_case_with_server(
        server: &mut DockerLumen,
        config: CaseConfig,
    ) -> Result<CompletedCase> {
        // The metric preflight is intentionally before the costly seed. Current
        // production metrics do not yet expose this complete set, so integration
        // fails closed before the costly seed until all runtime seams land.
        RuntimeCounters::parse(&server.metrics().await?)?;

        let setup_deadline = tokio::time::Instant::now()
            .checked_add(SETUP_TIMEOUT)
            .ok_or_else(|| {
                HarnessError::DataInvariant(
                    "setup deadline overflowed the Tokio instant range".to_owned(),
                )
            })?;
        eprintln!("PERF_STAGE_BEGIN seed");
        match seed(&server, config.vector_backend, setup_deadline).await {
            Ok(()) => eprintln!("PERF_STAGE_END seed"),
            Err(error @ HarnessError::SetupTimeout { .. }) => {
                eprintln!("PERF_STAGE_TIMEOUT seed");
                return Err(error);
            }
            Err(error) => return Err(error),
        }
        eprintln!("PERF_STAGE_BEGIN seed_checkpoint");
        let seed_checkpoint_metrics =
            match checkpoint_seed(&server.client, &server.base, setup_deadline).await {
                Ok(metrics) => {
                    eprintln!("PERF_STAGE_END seed_checkpoint");
                    metrics
                }
                Err(error @ HarnessError::SetupTimeout { .. }) => {
                    eprintln!("PERF_STAGE_TIMEOUT seed_checkpoint");
                    return Err(error);
                }
                Err(error) => return Err(error),
            };
        // This is the same mandatory post-seed checkpoint scrape. It is not
        // another request and it stays outside the measured input window.
        server.interval_trace.lock().await.record(
            Duration::ZERO,
            "pre_input",
            &seed_checkpoint_metrics,
        );
        // The clock and metric baseline are created only after seed work is
        // durable. This preparation checkpoint cannot satisfy interval gates.
        let (report, counter_delta, observed_input_duration) =
            drive_workload(&server, config).await?;
        let post_input_deadline = tokio::time::Instant::now()
            .checked_add(POST_INPUT_TIMEOUT)
            .ok_or_else(|| {
                HarnessError::DataInvariant(
                    "post-input deadline overflowed the Tokio instant range".to_owned(),
                )
            })?;
        eprintln!("PERF_STAGE_BEGIN live_readback");
        post_input_step(
            post_input_deadline,
            POST_INPUT_TIMEOUT,
            "live_readback",
            assert_mutation_readback(&server, "live after measured drain"),
        )
        .await?;
        eprintln!("PERF_STAGE_END live_readback");
        let live_mutation_readback = true;
        post_input_step(
            post_input_deadline,
            POST_INPUT_TIMEOUT,
            "checkpoint_merge_assertions",
            async {
                assert!(
                    counter_delta.checkpoints > 0,
                    "periodic SegmentRdbStore checkpoints must complete during measured input"
                );
                assert!(
                    counter_delta.merges > 0,
                    "a real segment merge must complete during measured input"
                );
                Ok(())
            },
        )
        .await?;
        if config.vector_backend.requires_hnsw_cache_seal() {
            eprintln!("PERF_STAGE_BEGIN hnsw_cache_seal");
            // This administrative request is governed by the shared absolute
            // post-input deadline below.  Do not reuse the workload client:
            // its five-second default can expire before that remaining budget.
            let cache_seal_client = reqwest::Client::new();
            let receipt = post_input_step(
                post_input_deadline,
                POST_INPUT_TIMEOUT,
                "hnsw_cache_seal",
                seal_hnsw_cache(&cache_seal_client, &server.base),
            )
            .await?;
            eprintln!("PERF_CACHE_SEAL_RECEIPT {}", cache_seal_receipt_line(&receipt));
            eprintln!("PERF_STAGE_END hnsw_cache_seal");
        }
        eprintln!("PERF_STAGE_BEGIN restart");
        let restart_elapsed = server.post_input_restart_step(post_input_deadline).await?;
        eprintln!("PERF_STAGE_END restart");
        // The restart opens the segment payload after the measured checkpoint
        // and merge work. This is the first post-restart data-plane observation:
        // it runs before any cold semantic or kNN readback can change a lazy
        // vector footprint, and no mutation follows the measured drain.
        eprintln!("PERF_STAGE_BEGIN vector_attestation");
        post_input_step(
            post_input_deadline,
            POST_INPUT_TIMEOUT,
            "vector_attestation",
            assert_post_restart_vector_backends(&server, config.vector_backend),
        )
        .await?;
        eprintln!("PERF_STAGE_END vector_attestation");
        eprintln!("PERF_STAGE_BEGIN count_query_readback");
        post_input_step(
            post_input_deadline,
            POST_INPUT_TIMEOUT,
            "count_query_readback",
            async {
                assert_document_count(&server, HOT_COLLECTION, HOT_DOCUMENTS)
                    .await
                    .map_err(|error| {
                        HarnessError::DataInvariant(format!(
                            "restart must recover the durable hot collection: {error}"
                        ))
                    })?;
                assert_recovered_text_query(&server)
                    .await
                    .map_err(|error| {
                        HarnessError::DataInvariant(format!(
                            "restart must recover Text/BM25 query data: {error}"
                        ))
                    })?;
                Ok(())
            },
        )
        .await?;
        eprintln!("PERF_STAGE_END count_query_readback");
        eprintln!("PERF_STAGE_BEGIN cold_readback");
        let cold_readback_started = Instant::now();
        let cold_readback = post_input_step(
            post_input_deadline,
            POST_INPUT_TIMEOUT,
            "cold_readback",
            assert_mutation_readback(&server, "cold after restart"),
        )
        .await;
        server
            .record_cold_readback_trace(cold_readback.is_ok(), cold_readback_started.elapsed())
            .await;
        cold_readback?;
        eprintln!("PERF_STAGE_END cold_readback");
        let cold_mutation_readback = true;
        post_input_step(
            post_input_deadline,
            POST_INPUT_TIMEOUT,
            "reporting",
            async {
                eprintln!(
                    "approved durable workload endpoint={:?} batch={} backend={} snapshot_secs={} restart_seconds={}",
                    config.primary_endpoint,
                    config.primary_batch_size,
                    config.vector_backend.wire_name(),
                    SNAPSHOT_SECONDS,
                    restart_elapsed.as_secs_f64(),
                );
                emit_rate_report(&report, observed_input_duration)?;
                eprintln!("approved durable workload report: {report:#?}");
                Ok(())
            },
        )
        .await?;
        Ok(CompletedCase {
            report,
            counter_delta,
            observed_input_duration,
            restart_duration: restart_elapsed,
            live_mutation_readback,
            cold_mutation_readback,
            image_reference: server.image_reference.clone(),
            actual_image_id: server.image_id.clone(),
        })
    }

    #[test]
    #[ignore = "30-minute Docker release workload; default execution runs all sixteen matrix cells serially"]
    fn approved_30_minute_durable_workload() {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(8)
            .enable_all()
            .build()
            .expect("build performance workload runtime")
            .block_on(async {
                let selection = SelectionInput::from_environment()
                    .expect("read performance workload mode environment");
                match SelectedMode::from_selection(&selection)
                    .expect("validate performance workload mode")
                {
                    SelectedMode::Diagnostic(config) => {
                        eprintln!("NON-QUALIFYING diagnostic performance cell selected");
                        run_case(config)
                            .await
                            .expect("drive selected diagnostic performance cell");
                    }
                    SelectedMode::Qualifying(config) => {
                        let context = QualifyingContext::from_environment()
                            .expect("read qualifying receipt environment");
                        let completed = run_case(config)
                            .await
                            .expect("drive selected qualifying performance cell");
                        let receipt = receipt_for(&context, config, &completed)
                            .expect("build a complete qualifying performance receipt");
                        perf_cell_receipt::write_new(&context.receipt_path, &receipt)
                            .expect("write one new qualifying performance receipt");
                        eprintln!(
                            "QUALIFYING durable performance receipt cell={} path={}",
                            receipt.cell.id,
                            context.receipt_path.display()
                        );
                    }
                    SelectedMode::FullMatrix => {
                        for config in qualifying_matrix() {
                            run_case(config)
                                .await
                                .expect("drive one qualifying performance matrix cell");
                        }
                    }
                }
            });
    }

    fn selected_cell(endpoint: &str, batch: &str, backend: &str) -> SelectionInput {
        SelectionInput {
            endpoint: Some(endpoint.to_owned()),
            batch: Some(batch.to_owned()),
            backend: Some(backend.to_owned()),
            ..SelectionInput::default()
        }
    }

    #[cfg(test)]
    #[derive(Clone, Copy)]
    enum FakeReadbackBehavior {
        FingerprintOnly,
        ScalarPredicatesThenWrongVectorOrder,
    }

    #[cfg(test)]
    #[derive(Clone)]
    struct FakeReadbackState {
        behavior: FakeReadbackBehavior,
        target: SemanticDocument,
        fields: Map<String, Value>,
    }

    #[cfg(test)]
    fn json_contains_key(value: &Value, wanted: &str) -> bool {
        match value {
            Value::Array(values) => values.iter().any(|value| json_contains_key(value, wanted)),
            Value::Object(values) => {
                values.contains_key(wanted)
                    || values
                        .values()
                        .any(|value| json_contains_key(value, wanted))
            }
            _ => false,
        }
    }

    #[cfg(test)]
    fn fake_response_is_wrong_scalar_predicate(query: &Value, fields: &Map<String, Value>) -> bool {
        let Some(object) = query.as_object() else {
            return false;
        };
        if let Some(hamming) = object.get("hamming") {
            let expected = fields.get("fingerprint").and_then(Value::as_str);
            let actual = hamming.get("hash").and_then(Value::as_str);
            if actual != expected {
                return true;
            }
        }
        if let Some(term) = object.get("term") {
            if term
                .get("value")
                .and_then(Value::as_str)
                .is_some_and(|value| value.contains(READBACK_MISMATCH_TOKEN))
            {
                return true;
            }
        }
        if let Some(matching) = object.get("match") {
            if matching
                .get("text")
                .and_then(Value::as_str)
                .is_some_and(|text| text.contains(READBACK_MISMATCH_TOKEN))
            {
                return true;
            }
        }
        if let Some(range) = object.get("range") {
            let field = range.get("field").and_then(Value::as_str);
            let actual = range.get("gte").and_then(Value::as_f64);
            let expected = field
                .and_then(|field| fields.get(field))
                .and_then(Value::as_f64);
            if let (Some(actual), Some(expected)) = (actual, expected) {
                if actual != expected {
                    return true;
                }
            }
        }
        object
            .values()
            .any(|value| fake_response_is_wrong_scalar_predicate(value, fields))
    }

    #[cfg(test)]
    async fn fake_readback_handler(
        axum::extract::State(state): axum::extract::State<FakeReadbackState>,
        axum::Json(request): axum::Json<Value>,
    ) -> axum::Json<Value> {
        let query = &request["query"];
        let emits_target = match state.behavior {
            FakeReadbackBehavior::FingerprintOnly => true,
            FakeReadbackBehavior::ScalarPredicatesThenWrongVectorOrder => {
                if json_contains_key(query, "knn") {
                    // This deliberately gives the target for both exact vectors.
                    // The production pair must reject that wrong candidate order.
                    true
                } else {
                    !fake_response_is_wrong_scalar_predicate(query, &state.fields)
                }
            }
        };
        let hits = if emits_target {
            vec![json!({ "external_id": state.target.external_id })]
        } else {
            Vec::new()
        };
        axum::Json(json!({ "hits": hits }))
    }

    #[cfg(test)]
    async fn fake_readback_lumen(
        behavior: FakeReadbackBehavior,
        target: SemanticDocument,
    ) -> (&'static DockerLumen, tokio::sync::oneshot::Sender<()>) {
        let fields = document_fields(target.number, target.tag);
        let app = axum::Router::new()
            .route(
                "/collections/perf-hot/search",
                axum::routing::post(fake_readback_handler),
            )
            .with_state(FakeReadbackState {
                behavior,
                target: target.clone(),
                fields,
            });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind fake semantic readback listener");
        let address = listener
            .local_addr()
            .expect("read fake semantic readback listener address");
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await
                .expect("serve fake semantic readback responder");
        });
        let server = Box::leak(Box::new(DockerLumen {
            // This test-only value must never drop, because production Drop
            // owns Docker cleanup. Leaking this tiny client fixture keeps the
            // ordinary oracle entirely local and does not invoke Docker.
            container: "fake-readback-never-dropped".to_owned(),
            volume: "fake-readback-never-dropped".to_owned(),
            base: format!("http://{address}"),
            client: reqwest::Client::builder()
                .build()
                .expect("build fake semantic readback client"),
            image_reference: "fake-readback".to_owned(),
            image_id: "fake-readback".to_owned(),
            cleanup_armed: false,
            recovery_observation: None,
            request_error_journal: Arc::new(Mutex::new(RequestErrorJournal::default())),
            interval_trace: Arc::new(Mutex::new(IntervalTrace::default())),
            restart_failure_trace: Arc::new(Mutex::new(None)),
            readyz_readiness_trace: Arc::new(std::sync::Mutex::new(None)),
        }));
        (server, shutdown_tx)
    }

    #[cfg(test)]
    fn run_readback_oracle(behavior: FakeReadbackBehavior) -> Result<()> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build local readback oracle runtime")
            .block_on(async move {
                let target = mutation_readback().indexed;
                let (server, shutdown) = fake_readback_lumen(behavior, target.clone()).await;
                let result =
                    assert_document_fields_readback(server, &target, "fake readback").await;
                let _ = shutdown.send(());
                result
            })
    }

    #[test]
    fn fingerprint_only_readback_cannot_pass_an_ignored_field_predicate() {
        assert!(
            run_readback_oracle(FakeReadbackBehavior::FingerprintOnly).is_err(),
            "a responder that returns the fingerprint target for every field predicate must fail readback"
        );
    }

    #[test]
    fn vector_readback_rejects_the_target_for_both_candidate_vectors() {
        assert!(
            run_readback_oracle(FakeReadbackBehavior::ScalarPredicatesThenWrongVectorOrder)
                .is_err(),
            "the exact reference vector must select the known reference, not the target"
        );
    }

    #[test]
    fn selected_qualifying_cell_is_explicit_and_never_a_diagnostic_false_green() {
        assert!(matches!(
            SelectedMode::from_selection(&SelectionInput::default()),
            Ok(SelectedMode::FullMatrix)
        ));

        let mut diagnostic = selected_cell("index", "100", "flat-cpu");
        diagnostic.diagnostic = Some("1".to_owned());
        assert!(matches!(
            SelectedMode::from_selection(&diagnostic),
            Ok(SelectedMode::Diagnostic(_))
        ));

        let mut qualifying = selected_cell("replace", "32", "hnsw-cpu");
        qualifying.qualifying = Some("1".to_owned());
        assert!(matches!(
            SelectedMode::from_selection(&qualifying),
            Ok(SelectedMode::Qualifying(_))
        ));
        qualifying.recovery_observation_secs = Some("120".to_owned());
        assert!(
            SelectedMode::from_selection(&qualifying).is_err(),
            "qualifying cells must reject diagnostic recovery observation"
        );

        let no_mode = selected_cell("unindex", "1", "flat-cpu");
        assert!(SelectedMode::from_selection(&no_mode).is_err());

        let mut both = selected_cell("index", "1", "flat-cpu");
        both.diagnostic = Some("1".to_owned());
        both.qualifying = Some("1".to_owned());
        assert!(SelectedMode::from_selection(&both).is_err());

        let missing_cell = SelectionInput {
            qualifying: Some("1".to_owned()),
            ..SelectionInput::default()
        };
        assert!(SelectedMode::from_selection(&missing_cell).is_err());
    }

    #[test]
    fn qualifying_receipt_path_must_be_absolute_with_an_existing_directory_parent() {
        let root = tempfile::tempdir().expect("create qualifying receipt path fixture");
        let existing_parent = root.path().join("receipts");
        fs::create_dir(&existing_parent).expect("create receipt parent");
        let ordinary_file = root.path().join("ordinary-file");
        fs::write(&ordinary_file, "not a directory").expect("create ordinary file");

        let variables = [
            (
                "LUMEN_PERF_IMAGE",
                "ghcr.io/example/lumen@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ),
            ("LUMEN_PERF_REPOSITORY", "example/lumen"),
            ("LUMEN_PERF_RUN_ID", "123"),
            ("LUMEN_PERF_RUN_ATTEMPT", "1"),
            ("LUMEN_PERF_COMMIT", "abcdef0"),
        ];
        for (name, value) in variables {
            env::set_var(name, value);
        }
        for path in [
            PathBuf::from("receipts/index-1-flat-cpu.json"),
            root.path().join("missing").join("index-1-flat-cpu.json"),
            ordinary_file.join("index-1-flat-cpu.json"),
        ] {
            env::set_var("LUMEN_PERF_RECEIPT_PATH", &path);
            assert!(
                QualifyingContext::from_environment().is_err(),
                "qualifying context must reject unsafe receipt path {}",
                path.display()
            );
        }
        env::set_var(
            "LUMEN_PERF_RECEIPT_PATH",
            existing_parent.join("index-1-flat-cpu.json"),
        );
        assert!(
            QualifyingContext::from_environment().is_ok(),
            "an absolute path below an existing receipt directory must remain valid"
        );
        for (name, _) in variables {
            env::remove_var(name);
        }
        env::remove_var("LUMEN_PERF_RECEIPT_PATH");
    }

    #[test]
    fn approved_index_operation_requires_the_frozen_fourteen_field_schema() {
        let fields = document_fields(7, "hot");
        let operation = approved_index_operation(1, "hot-base-000007".to_owned(), &fields)
            .expect("the frozen production fixture contains all fourteen fields");
        assert_eq!(operation.required_items.len(), FIELD_COUNT);
        assert_eq!(
            operation
                .required_items
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>(),
            FIELD_NAMES
                .iter()
                .map(|field| (*field).to_owned())
                .collect::<BTreeSet<_>>(),
            "the ledger must require the frozen schema, not only the submitted fields"
        );

        let mut thirteen_fields = fields.clone();
        thirteen_fields.remove("region");
        assert!(
            matches!(
                approved_index_operation(2, "missing-region".to_owned(), &thirteen_fields),
                Err(HarnessError::DataInvariant(_))
            ),
            "a thirteen-field Index operation must not enter the workload ledger"
        );

        let mut one_field = Map::new();
        one_field.insert("tag".to_owned(), json!("hot"));
        assert!(
            matches!(
                approved_index_operation(3, "one-field".to_owned(), &one_field),
                Err(HarnessError::DataInvariant(_))
            ),
            "a one-field Index operation must not enter the workload ledger"
        );

        let declared = schema(VectorBackend::FlatCpu)["fields"]
            .as_object()
            .expect("approved schema has fields")
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        let expected = FIELD_NAMES
            .iter()
            .map(|field| (*field).to_owned())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            declared, expected,
            "the schema must declare exactly the frozen fourteen fields"
        );
    }

    #[cfg(test)]
    fn complete_runtime_metrics(vmhwm_available: u64) -> String {
        [
            format!("lumen_process_rss_high_water_available {vmhwm_available}"),
            format!("{CHECKPOINT_COUNTER} 10"),
            format!("{CHECKPOINT_ATTEMPT_STARTED_COUNTER} 10"),
            format!("{CHECKPOINT_ATTEMPT_IN_FLIGHT} 0"),
            format!("{CHECKPOINT_ATTEMPT_FAILED_COUNTER} 0"),
            format!("{MERGE_COUNTER} 5"),
            format!("{CHECKPOINT_DURATION_COUNT} 10"),
            format!("{CHECKPOINT_DURATION_SUM} 1.5"),
            format!("{CAPTURE_LOCK_DURATION_COUNT} 10"),
            format!("{CAPTURE_LOCK_DURATION_SUM} 0.5"),
            format!("{CHECKPOINT_BYTES_COUNTER} 100"),
            format!("{MERGE_READ_BYTES_COUNTER} 100"),
            format!("{MERGE_WRITE_BYTES_COUNTER} 100"),
            format!("{BACKPRESSURE_COUNTER} 0"),
            format!("{PENDING_DELTA_BYTES} 10"),
            format!("{PENDING_DELTA_LAYERS} 1"),
            format!("{SEGMENT_DISK_BYTES} 100"),
            format!("{PROCESS_RSS_HIGH_WATER_BYTES} 1024"),
        ]
        .join("\n")
    }

    #[cfg(test)]
    fn complete_interval_trace_metrics(
        checkpoints: u64,
        merges: u64,
        state_count: u64,
        state_sum_seconds: &str,
        hnsw_count: u64,
        hnsw_sum_seconds: &str,
    ) -> String {
        let mut metrics = complete_runtime_metrics(1)
            .replacen(
                &format!("{CHECKPOINT_COUNTER} 10"),
                &format!("{CHECKPOINT_COUNTER} {checkpoints}"),
                1,
            )
            .replacen(
                &format!("{MERGE_COUNTER} 5"),
                &format!("{MERGE_COUNTER} {merges}"),
                1,
            );
        metrics.push_str("\nlumen_pending_change_reserved_bytes 7\nlumen_pending_change_active_bytes 11\nlumen_pending_change_frozen_bytes 13\nlumen_pending_change_total_bytes 31");
        for (name, count, sum) in [
            (STATE_WRITE_LOCK_HISTOGRAM, state_count, state_sum_seconds),
            (HNSW_ADD_HISTOGRAM, hnsw_count, hnsw_sum_seconds),
        ] {
            for label in INTERVAL_TRACE_BUCKET_LABELS {
                metrics.push_str(&format!("\n{name}_bucket{{le=\"{label}\"}} {count}"));
            }
            metrics.push_str(&format!("\n{name}_bucket{{le=\"+Inf\"}} {count}"));
            metrics.push_str(&format!("\n{name}_sum {sum}\n{name}_count {count}"));
        }
        metrics
    }

    #[cfg(test)]
    fn interval_trace_failure_totals(document: &Value) -> [u64; 4] {
        let mut totals = [0; 4];
        for key in ["samples", "omitted_request_failure_intervals"] {
            for item in document[key].as_array().into_iter().flatten() {
                let failures = &item["request_failures"];
                for (index, name) in ["http_429", "other_http", "timeout", "other_transport"]
                    .iter()
                    .enumerate()
                {
                    totals[index] += failures[*name].as_u64().unwrap_or_default();
                }
            }
        }
        totals
    }

    #[test]
    fn interval_trace_histograms_require_complete_finite_monotonic_rows() {
        let valid = complete_interval_trace_metrics(10, 5, 1, "0.001", 1, "0.002");
        assert!(IntervalTraceSnapshot::parse(&valid).is_ok());
        for invalid in [
            valid.replacen(
                &format!("{STATE_WRITE_LOCK_HISTOGRAM}_bucket{{le=\"0.001\"}} 1\n"),
                "",
                1,
            ),
            format!("{valid}\n{HNSW_ADD_HISTOGRAM}_bucket{{le=\"0.001\"}} 1"),
            valid.replacen(
                &format!("{HNSW_ADD_HISTOGRAM}_sum 0.002"),
                &format!("{HNSW_ADD_HISTOGRAM}_sum NaN"),
                1,
            ),
            valid.replacen(
                &format!("{STATE_WRITE_LOCK_HISTOGRAM}_bucket{{le=\"0.005\"}} 1"),
                &format!("{STATE_WRITE_LOCK_HISTOGRAM}_bucket{{le=\"0.005\"}} 0"),
                1,
            ),
        ] {
            assert!(IntervalTraceSnapshot::parse(&invalid).is_err());
        }
    }

    #[test]
    fn interval_trace_records_baseline_and_fixed_cadence_deltas() {
        let mut trace = IntervalTrace::default();
        let second = complete_interval_trace_metrics(12, 7, 3, "0.003", 4, "0.008")
            .replacen(
                &format!("{CHECKPOINT_ATTEMPT_STARTED_COUNTER} 10"),
                &format!("{CHECKPOINT_ATTEMPT_STARTED_COUNTER} 12"),
                1,
            )
            .replacen(
                &format!("{CHECKPOINT_ATTEMPT_IN_FLIGHT} 0"),
                &format!("{CHECKPOINT_ATTEMPT_IN_FLIGHT} 1"),
                1,
            )
            .replacen(
                &format!("{CHECKPOINT_ATTEMPT_FAILED_COUNTER} 0"),
                &format!("{CHECKPOINT_ATTEMPT_FAILED_COUNTER} 1"),
                1,
            );
        let third = complete_interval_trace_metrics(14, 8, 5, "0.005", 6, "0.012")
            .replacen(
                &format!("{CHECKPOINT_ATTEMPT_STARTED_COUNTER} 10"),
                &format!("{CHECKPOINT_ATTEMPT_STARTED_COUNTER} 14"),
                1,
            )
            .replacen(
                &format!("{CHECKPOINT_ATTEMPT_FAILED_COUNTER} 0"),
                &format!("{CHECKPOINT_ATTEMPT_FAILED_COUNTER} 2"),
                1,
            );
        trace.record(
            Duration::ZERO,
            "pre_input",
            &complete_interval_trace_metrics(10, 5, 1, "0.001", 1, "0.002"),
        );
        trace.record(
            INTERVAL_TRACE_CADENCE,
            "input",
            &second,
        );
        trace.record(
            INTERVAL_TRACE_CADENCE * 2,
            "input",
            &third,
        );
        let rendered = trace.render(&RequestErrorJournal::default()).unwrap();
        let document: Value = serde_json::from_str(&rendered).unwrap();
        assert_eq!(document["schema_version"], 3);
        assert_eq!(document["cadence_ms"], 5_000);
        assert_eq!(document["samples"].as_array().unwrap().len(), 3);
        assert_eq!(document["samples"][1]["checkpoint"]["delta"], 2);
        assert_eq!(document["samples"][1]["checkpoint_attempt"]["started_delta"], 2);
        assert_eq!(document["samples"][1]["checkpoint_attempt"]["in_flight"], 1);
        assert_eq!(document["samples"][1]["checkpoint_attempt"]["failed_delta"], 1);
        assert_eq!(document["samples"][2]["checkpoint_attempt"]["failed_total"], 2);
        assert_eq!(
            document["samples"][1]["state_write_lock_delta"]["sum_us"],
            2_000
        );
        assert_eq!(document["samples"][2]["hnsw_add_delta"]["count"], 2);
    }

    #[test]
    fn interval_trace_counts_every_429_after_detailed_journal_cap() {
        let mut journal = RequestErrorJournal::default();
        for _ in 0..(REQUEST_ERROR_JOURNAL_CAP + 45) {
            journal.push(RequestErrorRecord {
                elapsed_since_clock_start: INTERVAL_TRACE_CADENCE,
                endpoint: "secret-url-must-not-render".to_owned(),
                identifier: "secret-id-must-not-render".to_owned(),
                error: RequestFailure::synthetic("hidden response text".to_owned(), false),
                status: Some(429),
                body: Some("hidden body".to_owned()),
            });
        }
        assert_eq!(journal.records.len(), REQUEST_ERROR_JOURNAL_CAP);
        assert_eq!(journal.overflow, 45);
        let mut trace = IntervalTrace::default();
        trace.record(
            INTERVAL_TRACE_CADENCE,
            "input",
            &complete_interval_trace_metrics(10, 5, 1, "0.001", 1, "0.002"),
        );
        let rendered = trace.render(&journal).unwrap();
        let document: Value = serde_json::from_str(&rendered).unwrap();
        assert_eq!(document["samples"][0]["request_failures"]["http_429"], 301);
        assert!(!rendered.contains("secret-id-must-not-render"));
        assert!(!rendered.contains("hidden response text"));
    }

    #[test]
    fn interval_trace_is_bounded_valid_json_without_raw_data() {
        let mut trace = IntervalTrace::default();
        let metrics = complete_interval_trace_metrics(10, 5, 1, "0.001", 1, "0.002");
        for sample in 0..(INTERVAL_TRACE_MAX_SAMPLES + 10) {
            trace.record(
                Duration::from_millis(sample as u64 * INTERVAL_TRACE_CADENCE.as_millis() as u64),
                "input",
                &metrics,
            );
        }
        let rendered = trace.render(&RequestErrorJournal::default()).unwrap();
        let document: Value = serde_json::from_str(&rendered).unwrap();
        assert!(rendered.len() <= INTERVAL_TRACE_MAX_BYTES);
        assert!(document["samples"].as_array().unwrap().len() <= INTERVAL_TRACE_MAX_SAMPLES);
        assert!(!rendered.contains("# HELP"));
        assert!(!rendered.contains("http://"));
    }

    #[test]
    fn interval_trace_assigns_bucket_zero_failures_once() {
        let mut journal = RequestErrorJournal::default();
        for status in [Some(429), Some(500), None] {
            journal.push(RequestErrorRecord {
                elapsed_since_clock_start: Duration::from_millis(1),
                endpoint: "ignored-endpoint".to_owned(),
                identifier: "ignored-identifier".to_owned(),
                error: RequestFailure::synthetic("ignored error".to_owned(), false),
                status,
                body: None,
            });
        }
        let mut trace = IntervalTrace::default();
        let metrics = complete_interval_trace_metrics(10, 5, 1, "0.001", 1, "0.002");
        trace.record(Duration::ZERO, "pre_input", &metrics);
        trace.record(Duration::from_millis(1), "input", &metrics);

        let document: Value = serde_json::from_str(&trace.render(&journal).unwrap()).unwrap();
        assert_eq!(interval_trace_failure_totals(&document), [1, 1, 0, 1]);
        assert_eq!(document["samples"][0]["request_failures"]["http_429"], 0);
        assert_eq!(document["samples"][1]["request_failures"]["http_429"], 1);
    }

    #[test]
    fn interval_trace_preserves_error_totals_when_detail_is_capped() {
        let intervals = INTERVAL_TRACE_MAX_SAMPLES + 24;
        let mut error_only_journal = RequestErrorJournal::default();
        for bucket in 0..intervals {
            for (status, timeout) in [
                (Some(429), false),
                (Some(500), false),
                (Some(500), false),
                (None, true),
                (None, true),
                (None, true),
                (None, false),
                (None, false),
                (None, false),
                (None, false),
            ] {
                error_only_journal.push(RequestErrorRecord {
                    elapsed_since_clock_start: Duration::from_millis(
                        bucket as u64 * INTERVAL_TRACE_CADENCE.as_millis() as u64,
                    ),
                    endpoint: "ignored-endpoint".to_owned(),
                    identifier: "ignored-identifier".to_owned(),
                    error: RequestFailure::synthetic("ignored error".to_owned(), timeout),
                    status,
                    body: None,
                });
            }
        }
        let sample_capped: Value = serde_json::from_str(
            &IntervalTrace::default()
                .render(&error_only_journal)
                .unwrap(),
        )
        .unwrap();
        assert!(sample_capped["samples"].as_array().unwrap().len() <= INTERVAL_TRACE_MAX_SAMPLES);
        assert!(
            sample_capped["omitted_request_failure_intervals"]
                .as_array()
                .unwrap()
                .len()
                > 0
        );
        assert_eq!(
            interval_trace_failure_totals(&sample_capped),
            [
                intervals as u64,
                intervals as u64 * 2,
                intervals as u64 * 3,
                intervals as u64 * 4,
            ]
        );

        let mut trace = IntervalTrace::default();
        let mut byte_capped_journal = RequestErrorJournal::default();
        let metrics = complete_interval_trace_metrics(10, 5, 1, "0.001", 1, "0.002");
        for bucket in 0..12_u64 {
            trace.record(
                Duration::from_millis(bucket * INTERVAL_TRACE_CADENCE.as_millis() as u64),
                "input",
                &metrics,
            );
            for (status, timeout) in [
                (Some(429), false),
                (Some(429), false),
                (Some(429), false),
                (Some(429), false),
                (Some(500), false),
                (Some(500), false),
                (Some(500), false),
                (Some(500), false),
                (Some(500), false),
                (None, true),
                (None, true),
                (None, true),
                (None, true),
                (None, true),
                (None, true),
                (None, false),
                (None, false),
                (None, false),
                (None, false),
                (None, false),
                (None, false),
                (None, false),
            ] {
                byte_capped_journal.push(RequestErrorRecord {
                    elapsed_since_clock_start: Duration::from_millis(
                        bucket * INTERVAL_TRACE_CADENCE.as_millis() as u64,
                    ),
                    endpoint: "ignored-endpoint".to_owned(),
                    identifier: "ignored-identifier".to_owned(),
                    error: RequestFailure::synthetic("ignored error".to_owned(), timeout),
                    status,
                    body: None,
                });
            }
        }
        let byte_capped_rendered = trace
            .render_with_byte_cap(&byte_capped_journal, 4 * 1024)
            .unwrap();
        let byte_capped: Value = serde_json::from_str(&byte_capped_rendered).unwrap();
        assert!(byte_capped_rendered.len() <= 4 * 1024);
        assert!(byte_capped["samples"].as_array().unwrap().len() < 12);
        assert!(
            byte_capped["omitted_request_failure_intervals"]
                .as_array()
                .unwrap()
                .len()
                > 0
        );
        // Detail samples can be omitted, but every interval keeps one and
        // only one aggregate request-failure total in the document.
        assert_eq!(
            interval_trace_failure_totals(&byte_capped),
            [48, 60, 72, 84]
        );
    }

    /// Overflow records must retain their distinct timeout and transport
    /// totals, not merge them into one transport-or-timeout bucket.
    #[test]
    fn interval_trace_separates_timeouts_from_other_transport_after_detailed_journal_cap() {
        let mut journal = RequestErrorJournal::default();
        for index in 0..REQUEST_ERROR_JOURNAL_CAP {
            journal.push(RequestErrorRecord {
                elapsed_since_clock_start: INTERVAL_TRACE_CADENCE,
                endpoint: "ignored-endpoint".to_owned(),
                identifier: format!("ignored-{index}"),
                error: RequestFailure::synthetic("ignored status".to_owned(), false),
                status: Some(429),
                body: None,
            });
        }
        for (timeout, detail) in [
            (true, "request deadline elapsed"),
            (false, "connection reset by peer"),
        ] {
            journal.push(RequestErrorRecord {
                elapsed_since_clock_start: INTERVAL_TRACE_CADENCE,
                endpoint: "ignored-endpoint".to_owned(),
                identifier: "ignored-after-cap".to_owned(),
                error: RequestFailure::synthetic(detail.to_owned(), timeout),
                status: None,
                body: None,
            });
        }
        assert_eq!(journal.records.len(), REQUEST_ERROR_JOURNAL_CAP);
        assert_eq!(journal.overflow, 2);

        let mut trace = IntervalTrace::default();
        trace.record(
            INTERVAL_TRACE_CADENCE,
            "input",
            &complete_interval_trace_metrics(10, 5, 1, "0.001", 1, "0.002"),
        );
        let document: Value = serde_json::from_str(&trace.render(&journal).unwrap()).unwrap();
        assert_eq!(document["schema_version"], 3);
        assert_eq!(interval_trace_failure_totals(&document), [256, 0, 1, 1]);
        assert_eq!(document["samples"][0]["request_failures"]["timeout"], 1);
        assert_eq!(
            document["samples"][0]["request_failures"]["other_transport"],
            1
        );
    }

    #[test]
    fn runtime_sample_keeps_pending_occupancy_and_durable_progress_without_metric_comments() {
        let metrics = format!(
            "{}\n# HELP lumen_pending_change_total_bytes pending changes\n\
             lumen_pending_change_reserved_bytes 7\n\
             lumen_pending_change_active_bytes 11\n\
             lumen_pending_change_frozen_bytes 13\n\
             lumen_pending_change_total_bytes 31\n\
             unrelated_metric 999",
            complete_runtime_metrics(1)
        );
        let counters = RuntimeCounters::parse(&metrics).unwrap();
        let sample = runtime_sample(Duration::from_millis(10_250), counters, &metrics);
        assert!(sample.starts_with("PERF_RUNTIME_SAMPLE elapsed_ms=10250 "));
        assert!(sample.contains("checkpoints: 10, merges: 5"));
        for (state, bytes) in [
            ("reserved", 7),
            ("active", 11),
            ("frozen", 13),
            ("total", 31),
        ] {
            assert!(sample.contains(&format!("lumen_pending_change_{state}_bytes {bytes}")));
        }
        assert!(!sample.contains("# HELP"));
        assert!(!sample.contains("unrelated_metric"));
        assert!(!sample.contains('\n'));
    }

    #[test]
    fn runtime_metrics_reject_a_missing_required_row() {
        let metrics = complete_runtime_metrics(1)
            .lines()
            .filter(|line| !line.starts_with(MERGE_COUNTER))
            .collect::<Vec<_>>()
            .join("\n");
        match RuntimeCounters::parse(&metrics) {
            Err(HarnessError::MissingRuntimeMetric(name)) if name == MERGE_COUNTER => {}
            other => panic!("expected missing merge metric, got {other:?}"),
        }
    }

    #[test]
    fn runtime_metrics_reject_a_duplicate_required_row() {
        let metrics = format!("{}\n{CHECKPOINT_COUNTER} 11", complete_runtime_metrics(1));
        match RuntimeCounters::parse(&metrics) {
            Err(HarnessError::DuplicateRuntimeMetric(name)) if name == CHECKPOINT_COUNTER => {}
            other => panic!("expected duplicate checkpoint metric, got {other:?}"),
        }
    }

    #[test]
    fn runtime_metrics_reject_an_invalid_required_row() {
        let expected = format!("{CHECKPOINT_COUNTER} 10");
        let invalid = complete_runtime_metrics(1).replacen(
            &expected,
            &format!("{CHECKPOINT_COUNTER} not-a-counter"),
            1,
        );
        match RuntimeCounters::parse(&invalid) {
            Err(HarnessError::MetricParse { name, .. }) if name == CHECKPOINT_COUNTER => {}
            other => panic!("expected invalid checkpoint metric, got {other:?}"),
        }
    }

    #[test]
    fn runtime_metrics_reject_an_unavailable_vmhwm_probe() {
        match RuntimeCounters::parse(&complete_runtime_metrics(0)) {
            Err(HarnessError::DataInvariant(detail))
                if detail == "process VmHWM measurement is unavailable" => {}
            other => panic!("expected unavailable VmHWM metric, got {other:?}"),
        }
    }

    #[test]
    fn duration_counters_reject_backward_and_nonfinite_values() {
        for (after, before) in [
            (1.0, 2.0),
            (f64::NAN, 0.0),
            (f64::INFINITY, 0.0),
            (1.0, f64::NEG_INFINITY),
        ] {
            assert!(
                matches!(
                    float_delta(CHECKPOINT_DURATION_SUM, after, before),
                    Err(HarnessError::DataInvariant(_))
                ),
                "duration values must be finite and monotonic: before={before} after={after}"
            );
        }
        let expected = format!("{CHECKPOINT_DURATION_SUM} 1.5");
        let invalid = complete_runtime_metrics(1).replacen(
            &expected,
            &format!("{CHECKPOINT_DURATION_SUM} NaN"),
            1,
        );
        match RuntimeCounters::parse(&invalid) {
            Err(HarnessError::MetricParse { name, .. }) if name == CHECKPOINT_DURATION_SUM => {}
            other => panic!("expected nonfinite checkpoint duration, got {other:?}"),
        }
    }

    #[test]
    fn warmup_retry_policy_accepts_only_the_one_second_backpressure_hint() {
        assert!(is_warmup_retry_after_one(
            reqwest::StatusCode::TOO_MANY_REQUESTS,
            Some("1")
        ));
        assert!(!is_warmup_retry_after_one(
            reqwest::StatusCode::TOO_MANY_REQUESTS,
            Some("2")
        ));
        assert!(!is_warmup_retry_after_one(
            reqwest::StatusCode::TOO_MANY_REQUESTS,
            None
        ));
        assert!(!is_warmup_retry_after_one(
            reqwest::StatusCode::SERVICE_UNAVAILABLE,
            Some("1")
        ));
    }

    #[test]
    fn seed_backpressure_retry_honors_absolute_setup_deadline() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build setup deadline unit-test runtime");
        runtime.block_on(async {
            let (base, shutdown) = fake_always_backpressure_server().await;
            let server = DockerLumen {
                container: "fake-setup-never-dropped".to_owned(),
                volume: "fake-setup-never-dropped".to_owned(),
                base,
                client: reqwest::Client::builder()
                    .build()
                    .expect("build fake setup client"),
                image_reference: "fake-setup".to_owned(),
                image_id: "fake-setup".to_owned(),
                cleanup_armed: false,
                recovery_observation: None,
                request_error_journal: Arc::new(Mutex::new(RequestErrorJournal::default())),
                interval_trace: Arc::new(Mutex::new(IntervalTrace::default())),
                restart_failure_trace: Arc::new(Mutex::new(None)),
                readyz_readiness_trace: Arc::new(std::sync::Mutex::new(None)),
            };
            let timeout = Duration::from_millis(25);
            let deadline = tokio::time::Instant::now() + timeout;
            let started = Instant::now();
            let result = seed_index_batch(
                &server,
                HOT_COLLECTION,
                &[json!({
                    "external_id": "setup-timeout",
                    "field": "tag",
                    "value": "setup-timeout"
                })],
                deadline,
            )
            .await;
            let _ = shutdown.send(());

            assert!(
                started.elapsed() < Duration::from_secs(1),
                "setup backpressure must stop promptly, elapsed={:?}",
                started.elapsed()
            );
            match result {
                Err(HarnessError::SetupTimeout { stage: "seed" }) => {}
                other => panic!("expected a typed seed setup timeout, got {other:?}"),
            }
        });
    }

    #[cfg(test)]
    async fn fake_seed_checkpoint_server(
        status: reqwest::StatusCode,
        body: &'static str,
        metrics: String,
        delay_headers: bool,
        delay_body: bool,
    ) -> (
        String,
        Arc<StdMutex<Vec<&'static str>>>,
        tokio::task::JoinHandle<()>,
    ) {
        let calls = Arc::new(StdMutex::new(Vec::new()));
        let checkpoint_calls = calls.clone();
        let metric_calls = calls.clone();
        let app = axum::Router::new()
            .route(
                "/admin/checkpoint",
                axum::routing::post(move || {
                    let calls = checkpoint_calls.clone();
                    async move {
                        calls.lock().unwrap().push("checkpoint");
                        if delay_headers {
                            tokio::time::sleep(Duration::from_secs(2)).await;
                        }
                        let body =
                            axum::body::Body::from_stream(futures::stream::once(async move {
                                if delay_body {
                                    tokio::time::sleep(Duration::from_secs(2)).await;
                                }
                                Ok::<_, std::convert::Infallible>(body)
                            }));
                        let mut response = axum::response::Response::new(body);
                        *response.status_mut() = status;
                        response
                    }
                }),
            )
            .route(
                "/metrics",
                axum::routing::get(move || {
                    let calls = metric_calls.clone();
                    let metrics = metrics.clone();
                    async move {
                        calls.lock().unwrap().push("metrics");
                        metrics
                    }
                }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        (format!("http://{address}"), calls, task)
    }

    #[cfg(test)]
    async fn fake_hnsw_cache_seal_server(
        status: reqwest::StatusCode,
        response_body: &'static str,
    ) -> (
        String,
        Arc<StdMutex<Vec<usize>>>,
        tokio::task::JoinHandle<()>,
    ) {
        let bodies = Arc::new(StdMutex::new(Vec::new()));
        let received_bodies = bodies.clone();
        let app = axum::Router::new().route(
            "/admin/restart:seal-hnsw-cache",
            axum::routing::post(move |request_body: axum::body::Bytes| {
                let received_bodies = received_bodies.clone();
                async move {
                    received_bodies.lock().unwrap().push(request_body.len());
                    let mut response =
                        axum::response::Response::new(axum::body::Body::from(response_body));
                    *response.status_mut() = status;
                    response
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        (format!("http://{address}"), bodies, task)
    }

    #[cfg(test)]
    async fn fake_delayed_hnsw_cache_seal_server(
        delay: Duration,
    ) -> (String, tokio::task::JoinHandle<()>) {
        let app = axum::Router::new().route(
            "/admin/restart:seal-hnsw-cache",
            axum::routing::post(move || async move {
                tokio::time::sleep(delay).await;
                axum::Json(json!({
                    "sealed": true,
                    "cache_fields": 1,
                    "durability": "checkpoint_committed",
                    "mutation_stamp": {"epoch": 9, "apply_revision": 43},
                }))
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        (format!("http://{address}"), task)
    }

    #[cfg(test)]
    async fn fake_never_responding_hnsw_cache_seal_server() ->
        (String, tokio::task::JoinHandle<()>) {
        let app = axum::Router::new().route(
            "/admin/restart:seal-hnsw-cache",
            axum::routing::post(|| async {
                std::future::pending::<axum::response::Response>().await
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        (format!("http://{address}"), task)
    }

    #[test]
    fn hnsw_cache_seal_requires_a_strict_hnsw_receipt_and_sends_no_body() {
        assert!(!VectorBackend::FlatCpu.requires_hnsw_cache_seal());
        assert!(VectorBackend::HnswCpu.requires_hnsw_cache_seal());

        let valid = json!({
            "sealed": true,
            "cache_fields": 1,
            "durability": "aof_synced",
            "mutation_stamp": {"epoch": 7, "apply_revision": 42},
        });
        let parsed = parse_hnsw_cache_seal_receipt(valid.clone()).unwrap();
        let line: Value = serde_json::from_str(&cache_seal_receipt_line(&parsed)).unwrap();
        assert_eq!(line["schema"], "lumen.perf-cache-seal-receipt.v1");
        assert_eq!(line["sealed"], true);
        assert_eq!(line["cache_fields"], 1);
        assert_eq!(line["durability"], "aof_synced");
        assert_eq!(line["mutation_stamp"], json!({"epoch": 7, "apply_revision": 42}));
        for invalid in [
            json!({"sealed": false, "cache_fields": 1, "durability": "aof_synced", "mutation_stamp": {"epoch": 7, "apply_revision": 42}}),
            json!({"sealed": true, "cache_fields": 0, "durability": "aof_synced", "mutation_stamp": {"epoch": 7, "apply_revision": 42}}),
            json!({"sealed": true, "cache_fields": 1, "durability": "volatile", "mutation_stamp": {"epoch": 7, "apply_revision": 42}}),
            json!({"sealed": true, "cache_fields": 1, "durability": "aof_synced", "mutation_stamp": {"epoch": 7}}),
            json!({"sealed": true, "cache_fields": 1, "durability": "aof_synced", "mutation_stamp": {"epoch": 7, "apply_revision": 42}, "extra": true}),
        ] {
            assert!(
                matches!(
                    parse_hnsw_cache_seal_receipt(invalid),
                    Err(HarnessError::DataInvariant(_))
                ),
                "malformed seal receipt must fail closed"
            );
        }

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let (base, bodies, task) = fake_hnsw_cache_seal_server(
                reqwest::StatusCode::OK,
                r#"{"sealed":true,"cache_fields":1,"durability":"checkpoint_committed","mutation_stamp":{"epoch":9,"apply_revision":43}}"#,
            )
            .await;
            let result = seal_hnsw_cache(&reqwest::Client::new(), &base).await;
            task.abort();
            assert_eq!(
                result.unwrap(),
                HnswCacheSealReceipt {
                    cache_fields: 1,
                    durability: "checkpoint_committed",
                    mutation_epoch: 9,
                    mutation_apply_revision: 43,
                }
            );
            assert_eq!(
                *bodies.lock().unwrap(),
                vec![0],
                "the seal call must use an empty request body"
            );
        });
    }

    #[test]
    fn seed_checkpoint_requires_one_persisted_drained_publication() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let client = reqwest::Client::new();
            let drained = "lumen_pending_change_active_bytes 0\nlumen_pending_change_frozen_bytes 0\nlumen_pending_change_reserved_bytes 0\n";
            let mut cases = vec![
                (200, r#"{"persisted":true}"#, drained.to_owned(), true, 2),
                (429, r#"{"persisted":true}"#, drained.to_owned(), false, 1),
                (503, r#"{"persisted":true}"#, drained.to_owned(), false, 1),
                (200, r#"{"persisted":false}"#, drained.to_owned(), false, 1),
                (200, r#"{"persisted":"true"}"#, drained.to_owned(), false, 1),
                (200, "not-json", drained.to_owned(), false, 1),
                (200, r#"{"persisted":true}"#, String::new(), false, 2),
            ];
            for metric in ["active", "frozen", "reserved"] {
                cases.push((
                    200,
                    r#"{"persisted":true}"#,
                    drained.replace(&format!("{metric}_bytes 0"), &format!("{metric}_bytes 1")),
                    false,
                    2,
                ));
            }
            for (status, body, metrics, succeeds, expected_calls) in cases {
                let (base, calls, task) = fake_seed_checkpoint_server(
                    reqwest::StatusCode::from_u16(status).unwrap(),
                    body,
                    metrics.clone(),
                    false,
                    false,
                )
                .await;
                let result = checkpoint_seed(
                    &client,
                    &base,
                    tokio::time::Instant::now() + Duration::from_secs(1),
                )
                .await;
                task.abort();
                assert_eq!(
                    result.is_ok(), succeeds,
                    "status={status} body={body} metrics={metrics}: {result:?}"
                );
                let calls = calls.lock().unwrap();
                assert_eq!(
                    calls.len(), expected_calls,
                    "checkpoint cannot be skipped or retried"
                );
                assert_eq!(calls[0], "checkpoint");
                if expected_calls == 2 {
                    assert_eq!(calls[1], "metrics", "drain is checked only after publication");
                }
            }
            assert_eq!(DRAIN_TIMEOUT, Duration::from_secs(60));
            assert_eq!(INPUT_SECONDS, 1800);
            assert_eq!(DOCOPS_PER_SECOND, 100);
            assert_eq!(QUERY_QPS, 10);
        });
    }

    #[test]
    fn seed_checkpoint_honors_setup_and_body_deadlines() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let client = reqwest::Client::new();
            for (delay_headers, delay_body, expired) in [
                (true, false, false),
                (false, true, false),
                (false, false, true),
            ] {
                let (base, calls, task) = fake_seed_checkpoint_server(
                    reqwest::StatusCode::OK,
                    r#"{"persisted":true}"#,
                    String::new(),
                    delay_headers,
                    delay_body,
                )
                .await;
                let now = tokio::time::Instant::now();
                let deadline = now
                    + if expired {
                        Duration::ZERO
                    } else {
                        Duration::from_millis(25)
                    };
                let started = Instant::now();
                let result = checkpoint_seed(&client, &base, deadline).await;
                task.abort();
                assert!(
                    started.elapsed() < Duration::from_secs(1),
                    "setup must stop promptly"
                );
                assert!(
                    matches!(
                        result,
                        Err(HarnessError::SetupTimeout {
                            stage: "seed_checkpoint"
                        })
                    ),
                    "{result:?}"
                );
                assert_eq!(
                    calls.lock().unwrap().len(),
                    usize::from(!expired),
                    "a timed-out checkpoint is never retried"
                );
            }
        });
    }

    #[test]
    fn request_deadline_is_the_approved_five_seconds() {
        assert_eq!(REQUEST_TIMEOUT, Duration::from_secs(5));
    }

    #[test]
    fn post_input_workload_drain_deadline_returns_promptly() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build post-input deadline unit-test runtime");
        runtime.block_on(async {
            let started = Instant::now();
            let timeout = Duration::from_millis(25);
            let deadline = tokio::time::Instant::now() + timeout;
            let error = post_input_step(deadline, timeout, "workload_drain", async {
                tokio::time::sleep(Duration::from_secs(30)).await;
                Ok(())
            })
            .await
            .expect_err("a pending post-input operation must hit the shared deadline");

            assert!(
                started.elapsed() < Duration::from_secs(1),
                "the pending post-input operation must return promptly, elapsed={:?}",
                started.elapsed()
            );
            match error {
                HarnessError::PostInputTimeout {
                    stage: "workload_drain",
                    timeout: observed,
                } => assert_eq!(observed, timeout),
                other => panic!("expected a typed workload-drain timeout, got {other:?}"),
            }
        });
    }

    #[test]
    fn hnsw_cache_seal_uses_remaining_post_input_deadline_for_delayed_response() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build post-input deadline unit-test runtime");
        runtime.block_on(async {
            let (base, task) = fake_delayed_hnsw_cache_seal_server(Duration::from_secs(5) + Duration::from_millis(250)).await;
            let timeout = Duration::from_secs(6);
            let result = post_input_step(
                tokio::time::Instant::now() + timeout,
                timeout,
                "hnsw_cache_seal",
                seal_hnsw_cache(&reqwest::Client::new(), &base),
            )
            .await;
            task.abort();
            assert!(result.is_ok(), "seal must succeed before the outer deadline: {result:?}");
        });
    }

    #[test]
    fn hnsw_cache_seal_never_response_fails_at_remaining_post_input_deadline() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build post-input deadline unit-test runtime");
        runtime.block_on(async {
            let (base, task) = fake_never_responding_hnsw_cache_seal_server().await;
            let timeout = Duration::from_millis(25);
            let result = post_input_step(
                tokio::time::Instant::now() + timeout,
                timeout,
                "hnsw_cache_seal",
                seal_hnsw_cache(&reqwest::Client::new(), &base),
            )
            .await;
            task.abort();
            assert!(
                matches!(result, Err(HarnessError::PostInputTimeout { stage: "hnsw_cache_seal", timeout: observed }) if observed == timeout),
                "seal must fail at the outer deadline: {result:?}"
            );
        });
    }

    #[test]
    fn input_window_deadline_returns_promptly_with_the_timed_out_stage() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build input-window deadline unit-test runtime");
        runtime.block_on(async {
            let started = Instant::now();
            let timeout = Duration::from_millis(25);
            let deadline = tokio::time::Instant::now() + timeout;
            let error = input_window_step(deadline, timeout, "input_workload", async {
                tokio::time::sleep(Duration::from_secs(30)).await;
                Ok(())
            })
            .await
            .expect_err("a pending input operation must hit the absolute window deadline");

            assert!(
                started.elapsed() < Duration::from_secs(1),
                "the pending input operation must return promptly, elapsed={:?}",
                started.elapsed()
            );
            match error {
                HarnessError::InputWindowTimeout {
                    stage: "input_workload",
                    timeout: observed,
                } => assert_eq!(observed, timeout),
                other => panic!("expected a typed input-window timeout, got {other:?}"),
            }
        });
    }

    #[test]
    fn input_window_timeout_aborts_sampler_before_drive_finalization() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build input-window cancellation unit-test runtime");
        runtime.block_on(async {
            let sampler = tokio::spawn(async {
                loop {
                    tokio::task::yield_now().await;
                }
            });
            let sampler_abort = sampler.abort_handle();
            let started = Instant::now();
            let timeout = Duration::from_millis(25);
            let deadline = tokio::time::Instant::now() + timeout;
            let error = input_window_step_with_sampler(
                &sampler_abort,
                deadline,
                timeout,
                "input_workload",
                async {
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    Ok(())
                },
            )
            .await
            .expect_err("expired input work must return without entering finalization");

            assert!(
                started.elapsed() < Duration::from_secs(1),
                "the expired input path must return promptly, elapsed={:?}",
                started.elapsed()
            );
            assert!(
                matches!(
                    &error,
                    HarnessError::InputWindowTimeout {
                        stage: "input_workload",
                        timeout: observed,
                    } if *observed == timeout
                ),
                "expired input must return its typed stage error: {error:?}"
            );
            tokio::task::yield_now().await;
            assert!(
                sampler.is_finished(),
                "the sampler must be cancelled before drive_workload returns"
            );
            let sampler_error = sampler.await.expect_err("sampler must end by cancellation");
            assert!(
                sampler_error.is_cancelled(),
                "sampler cancellation must be explicit: {sampler_error}"
            );
        });
    }

    #[test]
    fn request_pump_capacity_wait_honors_input_window_deadline() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build request-pump deadline unit-test runtime");
        runtime.block_on(async {
            let mut pump = RequestPump::new(1);
            let timeout = Duration::from_millis(25);
            let deadline = tokio::time::Instant::now() + timeout;
            pump.push(
                async {
                    tokio::time::sleep(Duration::from_secs(30)).await;
                },
                deadline,
                timeout,
            )
            .await
            .expect("the first request fits within the pump capacity");

            let started = Instant::now();
            let error = pump
                .push(async {}, deadline, timeout)
                .await
                .expect_err("a full request pump must honor the input deadline");
            assert!(
                started.elapsed() < Duration::from_secs(1),
                "a stalled in-flight request must not block past the input deadline, elapsed={:?}",
                started.elapsed()
            );
            assert!(
                matches!(
                    error,
                    HarnessError::InputWindowTimeout {
                        stage: "input_workload",
                        timeout: observed,
                    } if observed == timeout
                ),
                "capacity wait must return the typed input-window timeout: {error:?}"
            );
        });
    }

    #[test]
    fn post_restart_backend_residency_rejects_flat_and_hnsw_coercion() {
        assert!(
            assert_post_restart_vector_residency(VectorBackend::FlatCpu, HOT_COLLECTION, 0).is_ok()
        );
        assert!(
            assert_post_restart_vector_residency(VectorBackend::HnswCpu, HOT_COLLECTION, 1).is_ok()
        );
        assert!(matches!(
            assert_post_restart_vector_residency(VectorBackend::FlatCpu, HOT_COLLECTION, 1),
            Err(HarnessError::DataInvariant(_))
        ));
        assert!(matches!(
            assert_post_restart_vector_residency(VectorBackend::HnswCpu, HOT_COLLECTION, 0),
            Err(HarnessError::DataInvariant(_))
        ));
    }

    #[test]
    fn post_restart_backend_residency_rejects_missing_or_unknown_stats() {
        let valid = json!({
            "fields": {
                "embedding": { "type": "vector", "bytes": 0 }
            }
        });
        assert_eq!(
            post_restart_vector_bytes(HOT_COLLECTION, &valid).unwrap(),
            0
        );

        for stats in [
            json!({}),
            json!({ "fields": {} }),
            json!({ "fields": { "embedding": { "type": "keyword", "bytes": 0 } } }),
            json!({ "fields": { "embedding": { "type": "vector" } } }),
            json!({ "fields": { "embedding": { "type": "vector", "bytes": "0" } } }),
            json!({ "fields": { "embedding": { "type": "vector", "bytes": -1 } } }),
        ] {
            assert!(matches!(
                post_restart_vector_bytes(HOT_COLLECTION, &stats),
                Err(HarnessError::DataInvariant(_))
            ));
        }
    }

    #[test]
    fn post_restart_backend_residency_covers_every_fixed_collection_and_vector_field() {
        let collections = workload_collections().collect::<Vec<_>>();
        assert_eq!(collections.len(), IDLE_COLLECTIONS + 1);
        assert_eq!(
            collections.first().map(String::as_str),
            Some(HOT_COLLECTION)
        );
        assert_eq!(
            collections.last().map(String::as_str),
            Some("perf-idle-180")
        );
        assert_eq!(
            collections.iter().collect::<BTreeSet<_>>().len(),
            collections.len(),
            "every workload collection needs one distinct post-restart observation"
        );

        let schema = schema(VectorBackend::FlatCpu);
        let fields = schema["fields"]
            .as_object()
            .expect("fixed workload schema has fields");
        let vector_fields = fields
            .iter()
            .filter_map(|(name, spec)| {
                (spec["type"].as_str() == Some("vector")).then_some(name.as_str())
            })
            .collect::<Vec<_>>();
        assert_eq!(
            vector_fields,
            vec![WORKLOAD_VECTOR_FIELD],
            "a new workload vector field needs an explicit post-restart backend attestation"
        );
    }

    #[cfg(test)]
    struct FakeEvidenceRunner {
        results: VecDeque<std::result::Result<String, String>>,
        events: Arc<StdMutex<Vec<String>>>,
    }

    #[cfg(test)]
    #[async_trait]
    impl EvidenceCommandRunner for FakeEvidenceRunner {
        async fn run(&mut self, args: Vec<String>) -> std::result::Result<String, String> {
            self.events
                .lock()
                .expect("record fake evidence command")
                .push(format!("docker {}", args.join(" ")));
            self.results
                .pop_front()
                .expect("fake evidence runner has one result per collection command")
        }
    }

    #[cfg(test)]
    struct FakeFailureProbe {
        metrics: Option<std::result::Result<String, String>>,
        hot_stats: Option<std::result::Result<String, String>>,
        events: Arc<StdMutex<Vec<String>>>,
    }

    #[cfg(test)]
    #[async_trait]
    impl FailureProbe for FakeFailureProbe {
        async fn metrics(&mut self) -> std::result::Result<String, String> {
            self.events
                .lock()
                .expect("record fake metrics probe")
                .push("GET /metrics".to_owned());
            self.metrics
                .take()
                .expect("fake metrics probe has one result")
        }

        async fn hot_stats(&mut self) -> std::result::Result<String, String> {
            self.events
                .lock()
                .expect("record fake hot-stats probe")
                .push("GET /collections/perf-hot/stats".to_owned());
            self.hot_stats
                .take()
                .expect("fake hot-stats probe has one result")
        }
    }

    #[cfg(test)]
    struct FakeCleanupRunner {
        events: Arc<StdMutex<Vec<String>>>,
    }

    #[cfg(test)]
    impl CleanupCommandRunner for FakeCleanupRunner {
        fn run_cleanup(&mut self, args: Vec<String>) {
            self.events
                .lock()
                .expect("record fake cleanup command")
                .push(format!("docker {}", args.join(" ")));
        }
    }

    #[cfg(test)]
    struct TraceCheckingCleanupRunner {
        evidence_root: PathBuf,
        events: Arc<StdMutex<Vec<String>>>,
        checked_trace: bool,
    }

    #[cfg(test)]
    impl CleanupCommandRunner for TraceCheckingCleanupRunner {
        fn run_cleanup(&mut self, args: Vec<String>) {
            if !self.checked_trace && args.first().is_some_and(|arg| arg == "rm") {
                let evidence = fs::read_dir(&self.evidence_root)
                    .expect("read real failure evidence root before Docker cleanup")
                    .find_map(|entry| entry.ok().map(|entry| entry.path()))
                    .expect("failure finalizer creates one evidence directory");
                let trace = fs::read_to_string(evidence.join(READYZ_READINESS_TRACE_FILE))
                    .expect("readiness trace exists before Docker cleanup");
                assert!(trace.contains("category=http_status:503"));
                assert!(trace.contains("category=cancelled"));
                assert!(trace.ends_with("terminal=cancelled"));
                self.events
                    .lock()
                    .expect("record readiness trace ordering")
                    .push("trace-present-before:docker rm".to_owned());
                self.checked_trace = true;
            }
            self.events
                .lock()
                .expect("record fake cleanup command")
                .push(format!("docker {}", args.join(" ")));
        }
    }

    /// This is the narrow execution seam for the readiness failure case. It
    /// keeps the real post-input timeout and shared case finalizer joined while
    /// tests replace only Docker command runners.
    #[cfg(test)]
    async fn run_restart_post_input_case_with_failure_finalizer<R, E, C>(
        server: &mut DockerLumen,
        deadline: tokio::time::Instant,
        timeout: Duration,
        restart_runner: &mut R,
        evidence_root: &Path,
        evidence_runner: &mut E,
        cleanup_runner: &mut C,
    ) -> Result<Duration>
    where
        R: RestartCommandRunner + Send,
        E: EvidenceCommandRunner + Send,
        C: CleanupCommandRunner,
    {
        let result = server
            .post_input_restart_step_with(deadline, timeout, restart_runner)
            .await;
        finish_case_result_with(
            server,
            result,
            evidence_root,
            evidence_runner,
            cleanup_runner,
        )
        .await
    }

    #[cfg(test)]
    async fn read_in_memory_evidence(
        bytes: Vec<u8>,
        limit: usize,
        retention: EvidenceRetention,
    ) -> BoundedEvidence {
        let (mut writer, reader) = tokio::io::duplex(8_192);
        let writer_task = async move {
            writer
                .write_all(&bytes)
                .await
                .expect("write in-memory evidence");
            writer.shutdown().await.expect("close in-memory evidence");
        };
        let (evidence, ()) =
            tokio::join!(read_bounded_reader(reader, limit, retention), writer_task);
        evidence.expect("read bounded in-memory evidence")
    }

    #[test]
    fn bounded_failure_streams_keep_success_stderr_and_http_bodies_small() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build bounded evidence unit-test runtime");
        runtime.block_on(async {
            let mut stdout_bytes = b"stdout diagnostic line\n".to_vec();
            stdout_bytes.extend(vec![b'o'; EVIDENCE_COMMAND_CHANNEL_MAX_BYTES]);
            let mut stderr_bytes = b"stderr tracing line\n".to_vec();
            stderr_bytes.extend(vec![b'e'; EVIDENCE_COMMAND_CHANNEL_MAX_BYTES]);
            let stdout = read_in_memory_evidence(
                stdout_bytes,
                EVIDENCE_COMMAND_CHANNEL_MAX_BYTES,
                EvidenceRetention::Prefix,
            )
            .await;
            let stderr = read_in_memory_evidence(
                stderr_bytes,
                EVIDENCE_COMMAND_CHANNEL_MAX_BYTES,
                EvidenceRetention::Prefix,
            )
            .await;
            assert_eq!(stdout.retained.len(), EVIDENCE_COMMAND_CHANNEL_MAX_BYTES);
            assert_eq!(stderr.retained.len(), EVIDENCE_COMMAND_CHANNEL_MAX_BYTES);
            assert!(stdout.total_bytes > stdout.retained.len() as u64);
            assert!(stderr.total_bytes > stderr.retained.len() as u64);
            assert!(stdout.truncated);
            assert!(stderr.truncated);

            let command = command_result(
                "logs --tail 2000 container-under-test",
                true,
                "exit status: 0",
                stdout,
                stderr,
            )
            .expect("successful docker logs retains both streams");
            assert!(command.contains("[stdout]"));
            assert!(command.contains("[stderr]"));
            assert!(command.contains("stderr tracing line"));
            assert!(command.contains("[stdout truncated after retaining"));
            assert!(command.contains("[stderr truncated after retaining"));
            assert!(
                command.len() <= EVIDENCE_ARTIFACT_MAX_BYTES,
                "combined successful stdout and stderr stay bounded before artifact writing"
            );

            let http_chunks = futures::stream::iter(vec![
                Ok::<Vec<u8>, String>(b"http diagnostic body\n".to_vec()),
                Ok(vec![b'h'; EVIDENCE_HTTP_BODY_MAX_BYTES]),
            ]);
            let http =
                read_bounded_chunks(http_chunks, EVIDENCE_HTTP_BODY_MAX_BYTES, |error| error)
                    .await
                    .expect("stream bounded HTTP evidence");
            assert_eq!(http.retained.len(), EVIDENCE_HTTP_BODY_MAX_BYTES);
            assert!(http.total_bytes > http.retained.len() as u64);
            assert!(http.truncated);
            let rendered_http = http.render("body");
            assert!(rendered_http.contains("http diagnostic body"));
            assert!(rendered_http.contains("[body truncated after retaining"));
            assert!(
                rendered_http.len() <= EVIDENCE_ARTIFACT_MAX_BYTES,
                "streamed HTTP evidence stays bounded before artifact writing"
            );
        });
    }

    #[test]
    fn docker_log_tail_keeps_final_stdout_and_stderr_within_artifact_cap() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build tail evidence unit-test runtime");
        runtime.block_on(async {
            let mut stdout_bytes = b"stdout early marker must be omitted\n".to_vec();
            stdout_bytes.extend(vec![b'o'; EVIDENCE_COMMAND_CHANNEL_MAX_BYTES]);
            stdout_bytes.extend_from_slice(b"\nstdout final timeout marker\n");
            let mut stderr_bytes = b"stderr early marker must be omitted\n".to_vec();
            stderr_bytes.extend(vec![b'e'; EVIDENCE_COMMAND_CHANNEL_MAX_BYTES]);
            stderr_bytes.extend_from_slice(b"\nstderr final timeout marker\n");
            let retention = docker_evidence_retention(&[
                "logs".to_owned(),
                "--tail".to_owned(),
                EVIDENCE_LOG_TAIL_LINES.to_owned(),
                "container-under-test".to_owned(),
            ]);
            assert_eq!(
                docker_evidence_retention(&[
                    "inspect".to_owned(),
                    "container-under-test".to_owned(),
                ]),
                EvidenceRetention::Prefix,
                "non-log docker evidence keeps the existing prefix retention"
            );

            let stdout = read_in_memory_evidence(
                stdout_bytes,
                EVIDENCE_COMMAND_CHANNEL_MAX_BYTES,
                retention,
            )
            .await;
            let stderr = read_in_memory_evidence(
                stderr_bytes,
                EVIDENCE_COMMAND_CHANNEL_MAX_BYTES,
                retention,
            )
            .await;
            let command = command_result(
                "logs --tail 2000 container-under-test",
                true,
                "exit status: 0",
                stdout,
                stderr,
            )
            .expect("successful docker logs retain both evidence channels");
            let artifact = bounded_evidence_text(&command);
            assert_eq!(
                artifact, command,
                "the final artifact writer preserves the already bounded docker log tail"
            );

            assert!(
                artifact.contains("stdout final timeout marker"),
                "docker log evidence keeps the final stdout line near the timeout"
            );
            assert!(
                artifact.contains("stderr final timeout marker"),
                "docker log evidence keeps the final stderr line near the timeout"
            );
            assert!(
                !artifact.contains("stdout early marker must be omitted"),
                "docker log tail omits stale stdout head data"
            );
            assert!(
                !artifact.contains("stderr early marker must be omitted"),
                "docker log tail omits stale stderr head data"
            );
            assert!(
                artifact.contains("[stdout tail]"),
                "docker log stdout carries tail provenance"
            );
            assert!(
                artifact.contains("[stderr tail]"),
                "docker log stderr carries tail provenance"
            );
            assert!(
                artifact.len() <= EVIDENCE_ARTIFACT_MAX_BYTES,
                "docker log tail stays within the existing artifact cap"
            );
        });
    }

    #[test]
    fn failure_evidence_keeps_request_chain_and_collects_before_cleanup() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build evidence unit-test runtime");
        runtime.block_on(async {
            let root = tempfile::tempdir().expect("create evidence unit-test directory");
            let error = HarnessError::RequestFailure(RequestFailure {
                display: "error sending request for url (http://127.0.0.1:7373/collections/perf-hot/index)"
                    .to_owned(),
                source_chain: vec![
                    "error sending request for url (http://127.0.0.1:7373/collections/perf-hot/index)"
                        .to_owned(),
                    "connection reset by peer".to_owned(),
                ],
                is_timeout: true,
                is_connect: false,
                is_request: false,
                is_body: false,
                is_decode: false,
                context: None,
            });
            let evidence = FailureEvidenceDirectory::create_in(
                root.path(),
                "container-under-test",
                "volume-under-test",
                &error,
            )
            .expect("create fake evidence directory");
            let events = Arc::new(StdMutex::new(Vec::new()));
            let mut runner = FakeEvidenceRunner {
                results: VecDeque::from([
                    Ok("last retained log line\n".to_owned()),
                    Err("inspect endpoint refused connection".to_owned()),
                ]),
                events: Arc::clone(&events),
            };
            let mut probe = FakeFailureProbe {
                metrics: Some(Ok("current metric sample\n".to_owned())),
                hot_stats: Some(Err("hot stats endpoint refused connection".to_owned())),
                events: Arc::clone(&events),
            };
            let mut cleanup_runner = FakeCleanupRunner { events: Arc::clone(&events) };

            finish_failed_docker_run(
                &evidence,
                &mut runner,
                &mut probe,
                &mut cleanup_runner,
                "container-under-test",
                "volume-under-test",
                "record=0 elapsed_ms=0 endpoint=POST /collections/perf-hot/index identifier=request_id=1 status=none timeout=true connect=false request=false body_err=false decode=false\n",
                Some("{\"schema_version\":3,\"kind\":\"lumen-perf-interval-trace\",\"samples\":[]}"),
            )
            .await;

            let actual_events = events
                .lock()
                .expect("read fake failure order")
                .clone();
            assert_eq!(
                actual_events,
                vec![
                    format!(
                        "docker logs --tail {} container-under-test",
                        EVIDENCE_LOG_TAIL_LINES
                    ),
                    "docker inspect container-under-test".to_owned(),
                    "GET /metrics".to_owned(),
                    "GET /collections/perf-hot/stats".to_owned(),
                    "docker rm -f container-under-test".to_owned(),
                    "docker volume rm -f volume-under-test".to_owned(),
                ],
                "the production failure finalizer must retain every probe before cleanup removes the run-owned resources"
            );
            assert_eq!(
                fs::read_to_string(evidence.path.join("interval-trace.json"))
                    .expect("trace must exist before the cleanup runner returns"),
                "{\"schema_version\":3,\"kind\":\"lumen-perf-interval-trace\",\"samples\":[]}"
            );
            assert_eq!(
                fs::read_to_string(evidence.path.join("docker-logs.txt"))
                    .expect("read retained log evidence"),
                "last retained log line\n"
            );
            assert!(
                fs::read_to_string(evidence.path.join("docker-inspect.json"))
                    .expect("read retained inspect error")
                    .contains("ERROR: inspect endpoint refused connection"),
                "a failed evidence command must be retained instead of replacing the original failure"
            );
            assert_eq!(
                fs::read_to_string(evidence.path.join("metrics.txt"))
                    .expect("read retained metrics evidence"),
                "current metric sample\n"
            );
            assert!(
                fs::read_to_string(evidence.path.join("hot-stats.json"))
                    .expect("read retained hot-stats error")
                    .contains("ERROR: hot stats endpoint refused connection")
            );
            evidence.record_result(
                "bounded-artifact.txt",
                Ok("x".repeat(EVIDENCE_ARTIFACT_MAX_BYTES + 1)),
            );
            let bounded = fs::read_to_string(evidence.path.join("bounded-artifact.txt"))
                .expect("read bounded evidence artifact");
            assert!(bounded.contains("[truncated after"));
            assert!(
                bounded.len() < EVIDENCE_ARTIFACT_MAX_BYTES + 128,
                "an untrusted Docker or HTTP response must not create an unbounded artifact"
            );
            let failure = fs::read_to_string(evidence.path.join("failure.txt"))
                .expect("read retained original failure");
            assert!(failure.contains("original_error_display=HTTP workload request failed:"));
            assert!(failure.contains("request_timeout=true"));
            assert!(failure.contains("request_connect=false"));
            assert!(failure.contains("connection reset by peer"));
        });
    }

    struct ScriptedRestartRunner {
        calls: Vec<String>,
        port_result: Option<Result<String>>,
    }

    #[async_trait]
    impl RestartCommandRunner for ScriptedRestartRunner {
        async fn restart(&mut self, container: &str) -> Result<()> {
            self.calls.push(format!("restart {container}"));
            Ok(())
        }

        async fn published_port(&mut self, container: &str) -> Result<String> {
            self.calls.push(format!("port {container} 7373/tcp"));
            self.port_result
                .take()
                .expect("resolve the port exactly once")
        }
    }

    async fn fake_ready_lumen() -> (
        DockerLumen,
        Arc<std::sync::atomic::AtomicUsize>,
        tokio::task::JoinHandle<()>,
    ) {
        fake_ready_lumen_with_delay(Duration::ZERO).await
    }

    async fn fake_ready_lumen_with_delay(
        delay: Duration,
    ) -> (
        DockerLumen,
        Arc<std::sync::atomic::AtomicUsize>,
        tokio::task::JoinHandle<()>,
    ) {
        let requests = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let app = axum::Router::new()
            .route(
                "/readyz",
                axum::routing::get(
                    |axum::extract::State((requests, delay)): axum::extract::State<(
                        Arc<std::sync::atomic::AtomicUsize>,
                        Duration,
                    )>| async move {
                        requests.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        tokio::time::sleep(delay).await;
                        axum::http::StatusCode::OK
                    },
                ),
            )
            .with_state((Arc::clone(&requests), delay));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind fake restart readiness responder");
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        (
            DockerLumen {
                container: "restart-test-owner".to_owned(),
                volume: "restart-test-volume".to_owned(),
                base: format!("http://{address}"),
                client: reqwest::Client::builder()
                    .timeout(REQUEST_TIMEOUT)
                    .build()
                    .unwrap(),
                image_reference: "restart-test-image-reference".to_owned(),
                image_id: "restart-test-image-id".to_owned(),
                cleanup_armed: false,
                recovery_observation: None,
                request_error_journal: Arc::new(Mutex::new(RequestErrorJournal::default())),
                interval_trace: Arc::new(Mutex::new(IntervalTrace::default())),
                restart_failure_trace: Arc::new(Mutex::new(None)),
                readyz_readiness_trace: Arc::new(std::sync::Mutex::new(None)),
            },
            requests,
            task,
        )
    }

    async fn fake_ready_lumen_with_statuses(
        statuses: Vec<axum::http::StatusCode>,
    ) -> (
        DockerLumen,
        Arc<std::sync::atomic::AtomicUsize>,
        tokio::task::JoinHandle<()>,
    ) {
        let requests = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let statuses = Arc::new(statuses);
        let app = axum::Router::new()
            .route(
                "/readyz",
                axum::routing::get(
                    |axum::extract::State((requests, statuses)): axum::extract::State<(
                        Arc<std::sync::atomic::AtomicUsize>,
                        Arc<Vec<axum::http::StatusCode>>,
                    )>| async move {
                        let index = requests.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        statuses
                            .get(index)
                            .copied()
                            .unwrap_or(axum::http::StatusCode::OK)
                    },
                ),
            )
            .with_state((Arc::clone(&requests), statuses));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind scripted restart readiness responder");
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        (
            DockerLumen {
                container: "restart-test-owner".to_owned(),
                volume: "restart-test-volume".to_owned(),
                base: format!("http://{address}"),
                client: reqwest::Client::builder()
                    .timeout(REQUEST_TIMEOUT)
                    .build()
                    .unwrap(),
                image_reference: "restart-test-image-reference".to_owned(),
                image_id: "restart-test-image-id".to_owned(),
                cleanup_armed: false,
                recovery_observation: None,
                request_error_journal: Arc::new(Mutex::new(RequestErrorJournal::default())),
                interval_trace: Arc::new(Mutex::new(IntervalTrace::default())),
                restart_failure_trace: Arc::new(Mutex::new(None)),
                readyz_readiness_trace: Arc::new(std::sync::Mutex::new(None)),
            },
            requests,
            task,
        )
    }

    async fn fake_ready_lumen_with_503_then_pending() -> (
        DockerLumen,
        Arc<std::sync::atomic::AtomicUsize>,
        tokio::task::JoinHandle<()>,
    ) {
        let requests = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let app = axum::Router::new()
            .route(
                "/readyz",
                axum::routing::get(
                    |axum::extract::State(requests): axum::extract::State<
                        Arc<std::sync::atomic::AtomicUsize>,
                    >| async move {
                        if requests.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
                            axum::response::IntoResponse::into_response((
                                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                                "unsafe-readyz-body-must-not-render",
                            ))
                        } else {
                            std::future::pending::<()>().await;
                            axum::response::IntoResponse::into_response(axum::http::StatusCode::OK)
                        }
                    },
                ),
            )
            .with_state(Arc::clone(&requests));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind 503 then pending readiness responder");
        let address = listener.local_addr().expect("read fake readiness address");
        let task = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("serve 503 then pending readiness responder");
        });
        (
            DockerLumen {
                container: "restart-test-owner".to_owned(),
                volume: "restart-test-volume".to_owned(),
                base: format!("http://{address}"),
                client: reqwest::Client::builder()
                    .timeout(REQUEST_TIMEOUT)
                    .build()
                    .expect("build fake readiness client"),
                image_reference: "restart-test-image-reference".to_owned(),
                image_id: "restart-test-image-id".to_owned(),
                cleanup_armed: false,
                recovery_observation: None,
                request_error_journal: Arc::new(Mutex::new(RequestErrorJournal::default())),
                interval_trace: Arc::new(Mutex::new(IntervalTrace::default())),
                restart_failure_trace: Arc::new(Mutex::new(None)),
                readyz_readiness_trace: Arc::new(std::sync::Mutex::new(None)),
            },
            requests,
            task,
        )
    }

    #[tokio::test]
    async fn restart_refreshes_published_port_before_readiness() {
        let (mut server, old_requests, old_task) = fake_ready_lumen().await;
        let (new_server, new_requests, new_task) = fake_ready_lumen().await;
        let old_identity = (
            server.container.clone(),
            server.volume.clone(),
            server.image_reference.clone(),
            server.image_id.clone(),
        );
        let mut runner = ScriptedRestartRunner {
            calls: Vec::new(),
            port_result: Some(Ok(format!(
                "{}\n",
                new_server.base.strip_prefix("http://").unwrap()
            ))),
        };
        server
            .restart_and_wait_ready_with(&mut runner)
            .await
            .unwrap();
        assert_eq!(
            server.base, new_server.base,
            "cold readbacks need the new URL"
        );
        assert_eq!(
            runner.calls,
            [
                "restart restart-test-owner",
                "port restart-test-owner 7373/tcp"
            ]
        );
        assert_eq!(
            old_requests.load(std::sync::atomic::Ordering::SeqCst),
            0,
            "a healthy responder at the stale port must not satisfy readiness"
        );
        assert_eq!(new_requests.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(
            (
                server.container.clone(),
                server.volume.clone(),
                server.image_reference.clone(),
                server.image_id.clone()
            ),
            old_identity
        );
        old_task.abort();
        new_task.abort();
    }

    #[tokio::test]
    async fn restart_port_lookup_failure_never_falls_back_to_stale_url() {
        let (mut server, requests, task) = fake_ready_lumen().await;
        let old_base = server.base.clone();
        let mut runner = ScriptedRestartRunner {
            calls: Vec::new(),
            port_result: Some(Err(HarnessError::Startup("port lookup failed".to_owned()))),
        };
        let result = server.restart_and_wait_ready_with(&mut runner).await;
        assert!(
            result.is_err(),
            "a healthy stale URL must not hide lookup failure"
        );
        assert_eq!(server.base, old_base);
        assert_eq!(requests.load(std::sync::atomic::Ordering::SeqCst), 0);
        task.abort();
    }

    #[tokio::test]
    async fn restart_rejects_invalid_published_ports_without_a_stale_probe() {
        let (mut server, requests, task) = fake_ready_lumen().await;
        let old_base = server.base.clone();
        for output in [
            "",
            "127.0.0.1:0",
            "127.0.0.1:65536",
            "127.0.0.1:not-a-port",
            "0.0.0.0:7373",
            "example.invalid:7373",
            "127.0.0.1:1234\n127.0.0.1:5678",
        ] {
            let mut runner = ScriptedRestartRunner {
                calls: Vec::new(),
                port_result: Some(Ok(output.to_owned())),
            };
            let result = server.restart_and_wait_ready_with(&mut runner).await;
            assert!(result.is_err(), "must refuse mapping {output:?}");
            assert_eq!(server.base, old_base);
        }
        assert_eq!(requests.load(std::sync::atomic::Ordering::SeqCst), 0);
        task.abort();
    }

    struct DelayedPortRunner {
        output: String,
        delay: Duration,
    }

    #[async_trait]
    impl RestartCommandRunner for DelayedPortRunner {
        async fn restart(&mut self, _container: &str) -> Result<()> {
            panic!("this fixture covers only the post-restart readiness budget")
        }

        async fn published_port(&mut self, _container: &str) -> Result<String> {
            tokio::time::sleep(self.delay).await;
            Ok(self.output.clone())
        }
    }

    struct SplitBudgetRestartRunner {
        output: String,
        restart_delay: Duration,
        port_delay: Duration,
    }

    #[async_trait]
    impl RestartCommandRunner for SplitBudgetRestartRunner {
        async fn restart(&mut self, _container: &str) -> Result<()> {
            tokio::time::sleep(self.restart_delay).await;
            Ok(())
        }

        async fn published_port(&mut self, _container: &str) -> Result<String> {
            tokio::time::sleep(self.port_delay).await;
            Ok(self.output.clone())
        }
    }

    struct DiagnosticRestartRunner {
        restart_result: Option<Result<()>>,
        restart_delay: Duration,
        port_result: Option<Result<String>>,
        port_delay: Duration,
    }

    #[async_trait]
    impl RestartCommandRunner for DiagnosticRestartRunner {
        async fn restart(&mut self, _container: &str) -> Result<()> {
            tokio::time::sleep(self.restart_delay).await;
            self.restart_result
                .take()
                .expect("restart should run once in the diagnostic fixture")
        }

        async fn published_port(&mut self, _container: &str) -> Result<String> {
            tokio::time::sleep(self.port_delay).await;
            self.port_result
                .take()
                .expect("port lookup should run once in the diagnostic fixture")
        }
    }

    #[test]
    fn restart_total_deadline_covers_restart_port_lookup_and_readiness() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build split restart-deadline regression runtime");
        runtime.block_on(async {
            let (mut server, _requests, task) = fake_ready_lumen().await;
            let mut runner = SplitBudgetRestartRunner {
                output: server.base.strip_prefix("http://").unwrap().to_owned(),
                restart_delay: Duration::from_secs(16),
                port_delay: Duration::from_secs(16),
            };
            let result = tokio::time::timeout(
                Duration::from_secs(35),
                server.restart_and_wait_ready_with(&mut runner),
            )
            .await
            .expect("the total restart deadline must finish promptly");
            assert!(
                matches!(result, Err(HarnessError::Startup(_))),
                "restart, port lookup, and readiness must share one 30-second deadline: {result:?}"
            );
            task.abort();
        });
    }

    #[test]
    fn restart_diagnostics_emit_complete_record_after_not_ready_polls() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build complete restart diagnostic runtime");
        runtime.block_on(async {
            let (mut server, requests, task) = fake_ready_lumen_with_statuses(vec![
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                axum::http::StatusCode::OK,
            ])
            .await;
            let mut runner = DiagnosticRestartRunner {
                restart_result: Some(Ok(())),
                restart_delay: Duration::ZERO,
                port_result: Some(Ok(server.base.strip_prefix("http://").unwrap().to_owned())),
                port_delay: Duration::ZERO,
            };
            let mut records = Vec::new();
            let elapsed = server
                .restart_and_wait_ready_with_diagnostic_observer(
                    &mut runner,
                    Instant::now() + Duration::from_secs(1),
                    |record, diagnostics| records.push((record.to_owned(), diagnostics)),
                )
                .await
                .expect("the third readiness response is healthy");
            assert_eq!(records.len(), 1);
            let (record, diagnostics) = &records[0];
            assert!(record.starts_with("PERF_RESTART_DIAGNOSTIC phase=complete outcome=success"));
            assert!(record.contains("total_elapsed_ms="));
            assert!(diagnostics.restart_elapsed.is_some());
            assert!(diagnostics.port_lookup_elapsed.is_some());
            assert!(diagnostics.readyz_wait_elapsed.is_some());
            assert_eq!(diagnostics.readiness_attempts, 3);
            let first_ready = diagnostics
                .first_ready_elapsed
                .expect("record first readiness");
            assert!(first_ready <= elapsed);
            assert!(diagnostics.readyz_wait_elapsed.unwrap() <= elapsed);
            assert_eq!(requests.load(std::sync::atomic::Ordering::SeqCst), 3);
            task.abort();
        });
    }

    #[test]
    fn recovery_observation_can_find_late_ready_but_restart_still_fails() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build late recovery observation runtime");
        runtime.block_on(async {
            let (mut server, requests, task) = fake_ready_lumen_with_statuses(vec![
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                axum::http::StatusCode::OK,
            ])
            .await;
            server.recovery_observation = Some(Duration::from_millis(500));
            let mut runner = DiagnosticRestartRunner {
                restart_result: Some(Ok(())),
                restart_delay: Duration::ZERO,
                port_result: Some(Ok(server.base.strip_prefix("http://").unwrap().to_owned())),
                port_delay: Duration::ZERO,
            };
            let mut records = Vec::new();
            let shared_deadline = tokio::time::Instant::now() + Duration::from_millis(80);
            let (outer_deadline, outer_timeout) = restart_post_input_window(
                shared_deadline,
                Some(Duration::from_secs(120)),
            )
            .unwrap();
            let result = post_input_step(
                outer_deadline,
                outer_timeout,
                "restart",
                server.restart_and_wait_ready_with_diagnostic_observer(
                    &mut runner,
                    Instant::now() + Duration::from_millis(80),
                    |record, _| records.push(record.to_owned()),
                ),
            )
            .await;
            assert!(
                matches!(result, Err(HarnessError::Startup(ref message)) if message.contains("30 seconds")),
                "late readiness must not change the official restart result: {result:?}"
            );
            assert_eq!(requests.load(std::sync::atomic::Ordering::SeqCst), 2);
            assert_eq!(records.len(), 1);
            assert!(records[0].starts_with("PERF_RESTART_DIAGNOSTIC phase=readyz outcome=timeout"));
            let trace = server.restart_failure_trace.lock().await;
            let trace = trace.expect("retain official restart failure trace");
            assert!(matches!(trace.phase, RestartFailureTracePhase::Readyz));
            assert!(matches!(trace.outcome, RestartFailureTraceOutcome::Timeout));
            task.abort();
        });
    }

    #[test]
    fn restart_diagnostics_keep_partial_records_for_terminal_phase_failures() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build terminal restart diagnostic runtime");
        runtime.block_on(async {
            async fn observe_terminal(
                server: &mut DockerLumen,
                runner: &mut DiagnosticRestartRunner,
                deadline: Instant,
            ) -> (Result<Duration>, Vec<(String, RestartPhaseDiagnostics)>) {
                let mut records = Vec::new();
                let result = server
                    .restart_and_wait_ready_with_diagnostic_observer(
                        runner,
                        deadline,
                        |record, diagnostics| records.push((record.to_owned(), diagnostics)),
                    )
                    .await;
                (result, records)
            }

            let (mut restart_error_server, _, restart_error_task) = fake_ready_lumen().await;
            let mut restart_error_runner = DiagnosticRestartRunner {
                restart_result: Some(Err(HarnessError::Startup("restart failed".to_owned()))),
                restart_delay: Duration::ZERO,
                port_result: None,
                port_delay: Duration::ZERO,
            };
            let (result, records) = observe_terminal(
                &mut restart_error_server,
                &mut restart_error_runner,
                Instant::now() + Duration::from_secs(1),
            )
            .await;
            assert!(result.is_err());
            assert_eq!(records.len(), 1);
            let (record, diagnostics) = &records[0];
            assert!(
                record.starts_with("PERF_RESTART_DIAGNOSTIC phase=docker-restart outcome=error")
            );
            assert!(diagnostics.restart_elapsed.is_some());
            assert!(diagnostics.port_lookup_elapsed.is_none());
            assert!(diagnostics.readyz_wait_elapsed.is_none());
            assert!(diagnostics.first_ready_elapsed.is_none());
            restart_error_task.abort();

            let (mut restart_timeout_server, _, restart_timeout_task) = fake_ready_lumen().await;
            let mut restart_timeout_runner = DiagnosticRestartRunner {
                restart_result: Some(Ok(())),
                restart_delay: Duration::from_secs(1),
                port_result: None,
                port_delay: Duration::ZERO,
            };
            let (result, records) = observe_terminal(
                &mut restart_timeout_server,
                &mut restart_timeout_runner,
                Instant::now() + Duration::from_millis(30),
            )
            .await;
            assert!(result.is_err());
            let (record, diagnostics) = &records[0];
            assert!(
                record.starts_with("PERF_RESTART_DIAGNOSTIC phase=docker-restart outcome=timeout")
            );
            assert!(diagnostics.restart_elapsed.is_some());
            assert!(diagnostics.port_lookup_elapsed.is_none());
            assert!(diagnostics.readyz_wait_elapsed.is_none());
            assert!(diagnostics.first_ready_elapsed.is_none());
            restart_timeout_task.abort();

            let (mut port_error_server, _, port_error_task) = fake_ready_lumen().await;
            let mut port_error_runner = DiagnosticRestartRunner {
                restart_result: Some(Ok(())),
                restart_delay: Duration::ZERO,
                port_result: Some(Err(HarnessError::Startup("port failed".to_owned()))),
                port_delay: Duration::ZERO,
            };
            let (result, records) = observe_terminal(
                &mut port_error_server,
                &mut port_error_runner,
                Instant::now() + Duration::from_secs(1),
            )
            .await;
            assert!(result.is_err());
            let (record, diagnostics) = &records[0];
            assert!(
                record.starts_with("PERF_RESTART_DIAGNOSTIC phase=published-port outcome=error")
            );
            assert!(diagnostics.restart_elapsed.is_some());
            assert!(diagnostics.port_lookup_elapsed.is_some());
            assert!(diagnostics.readyz_wait_elapsed.is_none());
            assert!(diagnostics.first_ready_elapsed.is_none());
            port_error_task.abort();

            let (mut port_timeout_server, _, port_timeout_task) = fake_ready_lumen().await;
            let mut port_timeout_runner = DiagnosticRestartRunner {
                restart_result: Some(Ok(())),
                restart_delay: Duration::ZERO,
                port_result: Some(Ok(port_timeout_server
                    .base
                    .strip_prefix("http://")
                    .unwrap()
                    .to_owned())),
                port_delay: Duration::from_secs(1),
            };
            let (result, records) = observe_terminal(
                &mut port_timeout_server,
                &mut port_timeout_runner,
                Instant::now() + Duration::from_millis(30),
            )
            .await;
            assert!(result.is_err());
            let (record, diagnostics) = &records[0];
            assert!(
                record.starts_with("PERF_RESTART_DIAGNOSTIC phase=published-port outcome=timeout")
            );
            assert!(diagnostics.restart_elapsed.is_some());
            assert!(diagnostics.port_lookup_elapsed.is_some());
            assert!(diagnostics.readyz_wait_elapsed.is_none());
            assert!(diagnostics.first_ready_elapsed.is_none());
            port_timeout_task.abort();

            let (mut ready_timeout_server, requests, ready_timeout_task) =
                fake_ready_lumen_with_statuses(vec![axum::http::StatusCode::SERVICE_UNAVAILABLE])
                    .await;
            let mut ready_timeout_runner = DiagnosticRestartRunner {
                restart_result: Some(Ok(())),
                restart_delay: Duration::ZERO,
                port_result: Some(Ok(ready_timeout_server
                    .base
                    .strip_prefix("http://")
                    .unwrap()
                    .to_owned())),
                port_delay: Duration::ZERO,
            };
            let (result, records) = observe_terminal(
                &mut ready_timeout_server,
                &mut ready_timeout_runner,
                Instant::now() + Duration::from_millis(30),
            )
            .await;
            assert!(result.is_err());
            let (record, diagnostics) = &records[0];
            assert!(record.starts_with("PERF_RESTART_DIAGNOSTIC phase=readyz outcome=timeout"));
            assert!(diagnostics.restart_elapsed.is_some());
            assert!(diagnostics.port_lookup_elapsed.is_some());
            assert!(diagnostics.readyz_wait_elapsed.is_some());
            assert!(diagnostics.readiness_attempts >= 1);
            assert!(diagnostics.first_ready_elapsed.is_none());
            assert!(requests.load(std::sync::atomic::Ordering::SeqCst) >= 1);
            ready_timeout_task.abort();
        });
    }

    /// A restart that fails before cold readback must leave a fixed-shape
    /// timing record. The trace must say cold readback was unavailable, not
    /// omit that lifecycle stage or infer a successful readback.
    #[test]
    fn restart_failure_trace_writes_timing_and_cold_readback_unavailable() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build restart failure trace unit-test runtime");
        runtime.block_on(async {
            let (mut server, _, task) = fake_ready_lumen().await;
            let mut runner = DiagnosticRestartRunner {
                restart_result: Some(Err(HarnessError::Startup("restart failed".to_owned()))),
                restart_delay: Duration::ZERO,
                port_result: None,
                port_delay: Duration::ZERO,
            };
            let mut records = Vec::new();
            let result = server
                .restart_and_wait_ready_with_diagnostic_observer(
                    &mut runner,
                    Instant::now() + STARTUP_TIMEOUT,
                    |record, diagnostics| records.push((record.to_owned(), diagnostics)),
                )
                .await;
            assert!(matches!(result, Err(HarnessError::Startup(_))));
            assert_eq!(records.len(), 1);
            let (record, diagnostics) = &records[0];
            assert!(
                record.starts_with("PERF_RESTART_DIAGNOSTIC phase=docker-restart outcome=error")
            );
            assert!(record.contains("total_elapsed_ms="));
            assert!(diagnostics.restart_elapsed.is_some());
            assert!(diagnostics.port_lookup_elapsed.is_none());
            assert!(diagnostics.readyz_wait_elapsed.is_none());
            task.abort();
        });

        let region = restart_diagnostic_production_region(include_str!("perf_gate.rs"));
        for expected in [
            "enum RestartFailureTracePhase {",
            "enum RestartFailureTraceOutcome {",
            "enum RestartFailureTraceColdReadback {",
            "struct RestartFailureTrace {",
            "RestartFailureTraceColdReadback::Unavailable",
            "restart-failure-trace.json",
            "STARTUP_TIMEOUT",
        ] {
            assert!(
                restart_diagnostic_production_contains(region, expected),
                "missing fixed-enum restart failure trace contract line: {expected}"
            );
        }
    }

    #[tokio::test]
    async fn restart_port_lookup_cannot_outlive_the_readiness_budget() {
        let (mut server, requests, task) = fake_ready_lumen().await;
        let old_base = server.base.clone();
        let mut runner = DelayedPortRunner {
            output: server.base.strip_prefix("http://").unwrap().to_owned(),
            delay: Duration::from_secs(1),
        };
        let result = tokio::time::timeout(
            Duration::from_secs(2),
            server
                .refresh_restart_endpoint(&mut runner, Instant::now() + Duration::from_millis(50)),
        )
        .await
        .expect("the original readiness budget must terminate lookup");
        assert!(
            matches!(result, Err(HarnessError::Startup(_))),
            "{result:?}"
        );
        assert_eq!(server.base, old_base);
        assert_eq!(requests.load(std::sync::atomic::Ordering::SeqCst), 0);
        task.abort();
    }

    #[tokio::test]
    async fn restart_port_lookup_does_not_reset_the_readiness_budget() {
        let (mut server, _requests, task) =
            fake_ready_lumen_with_delay(Duration::from_secs(1)).await;
        let mut runner = DelayedPortRunner {
            output: server.base.strip_prefix("http://").unwrap().to_owned(),
            delay: Duration::from_millis(25),
        };
        let result = tokio::time::timeout(
            Duration::from_secs(2),
            server
                .refresh_restart_endpoint(&mut runner, Instant::now() + Duration::from_millis(100)),
        )
        .await
        .expect("lookup and HTTP readiness must share the same deadline");
        assert!(
            matches!(result, Err(HarnessError::Startup(_))),
            "{result:?}"
        );
        task.abort();
    }

    #[cfg(test)]
    async fn closed_port_base_url() -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind a port to synthesize a connect error");
        let address = listener
            .local_addr()
            .expect("read the synthetic closed-port address");
        // Drop the listener immediately: nothing accepts on `address` from
        // here on, so a client that dials it observes a real OS-level
        // connection refusal instead of a fabricated error value.
        drop(listener);
        format!("http://{address}")
    }

    #[cfg(test)]
    async fn fake_non_success_handler(
        axum::extract::State(body): axum::extract::State<String>,
    ) -> (axum::http::StatusCode, String) {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, body)
    }

    #[cfg(test)]
    async fn fake_non_success_server(body: String) -> (String, tokio::sync::oneshot::Sender<()>) {
        let app = axum::Router::new()
            .route(
                "/collections/perf-hot/index",
                axum::routing::post(fake_non_success_handler),
            )
            .with_state(body);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind fake non-success server listener");
        let address = listener
            .local_addr()
            .expect("read fake non-success server listener address");
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await
                .expect("serve fake non-success server");
        });
        (format!("http://{address}"), shutdown_tx)
    }

    #[cfg(test)]
    async fn fake_always_backpressure_handler() -> axum::response::Response {
        let mut response = axum::response::Response::new(axum::body::Body::from("{}"));
        *response.status_mut() = axum::http::StatusCode::TOO_MANY_REQUESTS;
        response.headers_mut().insert(
            axum::http::header::RETRY_AFTER,
            axum::http::HeaderValue::from_static("1"),
        );
        response
    }

    #[cfg(test)]
    async fn fake_always_backpressure_server() -> (String, tokio::sync::oneshot::Sender<()>) {
        let app = axum::Router::new().route(
            "/collections/perf-hot/index",
            axum::routing::post(fake_always_backpressure_handler),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind fake setup backpressure listener");
        let address = listener
            .local_addr()
            .expect("read fake setup backpressure listener address");
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await
                .expect("serve fake setup backpressure responder");
        });
        (format!("http://{address}"), shutdown_tx)
    }

    #[test]
    fn journal_records_a_real_reqwest_connect_error_with_full_chain() {
        //! # Facets
        //!
        //! - Behavior: this test drives a real `reqwest` connect failure
        //!   through the production `post_json` and `record_request_error`
        //!   path and asserts the journaled `RequestErrorRecord` carries a
        //!   multi-level `source_chain` and `is_connect=true`
        //!   (`apps/lumen/e2e/perf_gate.rs:5535`, `:5539`) — the exact
        //!   evidence the Goal's two undiagnosed 30-minute cells lacked.
        //!   Behavior is also carried by
        //!   `request_error_journal_lands_in_the_failure_evidence_bundle`,
        //!   which proves the same journal reaches `request-errors.txt`
        //!   through `finish_failed_docker_run`
        //!   (`apps/lumen/e2e/perf_gate.rs:1863`) and asserts the on-disk
        //!   bytes equal the rendered journal and contain the endpoint,
        //!   identifier, and error text
        //!   (`apps/lumen/e2e/perf_gate.rs:5608-5611`).
        //! - Security: the change opens two new boundaries on data a peer
        //!   (the Lumen container under test) supplies and this harness now
        //!   persists to disk instead of discarding — a non-2xx response
        //!   body (`post_json`'s new branch,
        //!   `apps/lumen/e2e/perf_gate.rs:2552`) and the count of
        //!   failed-request records (`RequestErrorJournal::push`'s cap
        //!   check, `apps/lumen/e2e/perf_gate.rs:1742`). Both are closed by
        //!   a bound:
        //!   `non_success_status_record_carries_status_and_a_truncated_body`
        //!   feeds a `REQUEST_ERROR_BODY_MAX_BYTES + 200`-byte untrusted
        //!   body and asserts the retained record never contains the full
        //!   body (`apps/lumen/e2e/perf_gate.rs:5672`), only the
        //!   `truncate_evidence_body`-bounded form
        //!   (`apps/lumen/e2e/perf_gate.rs:1764`);
        //!   `request_error_journal_caps_entries_and_counts_the_overflow`
        //!   feeds `REQUEST_ERROR_JOURNAL_CAP + 5` failures and asserts the
        //!   journal stops growing at the cap and only counts the rest in
        //!   `overflow` (`apps/lumen/e2e/perf_gate.rs:5631-5632`).
        //! - Performance: every request this change touches (`post_json`,
        //!   `send_index`/`send_replace`/`send_unindex`/`send_query`)
        //!   already runs under the unchanged `REQUEST_TIMEOUT` budget
        //!   (`apps/lumen/e2e/perf_gate.rs:821`, `Duration::from_secs(5)`),
        //!   which the declared gate `cargo test --release --locked -p
        //!   lumen --test perf_gate -- --ignored --test-threads=1
        //!   --nocapture` (`apps/lumen/README.md:253`) already measures end
        //!   to end through
        //!   `durable_workload::approved_30_minute_durable_workload`. This
        //!   change adds only O(1) bookkeeping on a request whose outcome
        //!   is already decided (`record_request_error` runs after
        //!   `post_json` returns) and preserves the exact single outer
        //!   `tokio::time::timeout(REQUEST_TIMEOUT, ..)` that already wraps
        //!   the whole round trip
        //!   (`apps/lumen/e2e/perf_gate.rs:2544`), so no new user-waited
        //!   path opens; the existing gate is the account and no new case
        //!   is added.
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build connect-error unit-test runtime");
        runtime.block_on(async {
            let base = closed_port_base_url().await;
            let client = reqwest::Client::builder()
                .build()
                .expect("build closed-port client");
            let request = client
                .post(format!("{base}/collections/perf-hot/index"))
                .json(&json!({}));
            let failure = post_json(request)
                .await
                .expect_err("a closed port must classify as a PostJsonFailure");
            assert!(
                failure.error.is_connect,
                "a refused TCP connect must classify as is_connect=true, got {failure:?}"
            );
            assert_eq!(failure.outcome, Outcome::Failed);

            let journal = Arc::new(Mutex::new(RequestErrorJournal::default()));
            let outcome = record_request_error(
                &journal,
                Duration::from_millis(1),
                "POST /collections/perf-hot/index".to_owned(),
                "request_id=1".to_owned(),
                failure,
            )
            .await;
            assert_eq!(outcome, Outcome::Failed);

            let guard = journal.lock().await;
            assert_eq!(guard.records.len(), 1);
            let record = &guard.records[0];
            assert!(
                record.error.source_chain.len() >= 2,
                "a real reqwest connect error must carry the top display plus at least one nested source, got {:?}",
                record.error.source_chain
            );
            assert!(record.error.is_connect);
            assert_eq!(record.endpoint, "POST /collections/perf-hot/index");
            assert_eq!(record.identifier, "request_id=1");
        });
    }

    /// Proves the journal reaches disk through the shared
    /// `finish_failed_docker_run` finish path, following the fake-runner
    /// pattern in `failure_evidence_keeps_request_chain_and_collects_before_cleanup`.
    #[test]
    fn request_error_journal_lands_in_the_failure_evidence_bundle() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build request-error evidence unit-test runtime");
        runtime.block_on(async {
            let root = tempfile::tempdir().expect("create request-error unit-test directory");
            let error = HarnessError::RequestFailure(RequestFailure::synthetic(
                "error sending request for url (http://127.0.0.1:7373/collections/perf-hot/index)"
                    .to_owned(),
                false,
            ));
            let evidence = FailureEvidenceDirectory::create_in(
                root.path(),
                "container-under-test",
                "volume-under-test",
                &error,
            )
            .expect("create fake evidence directory");
            let events = Arc::new(StdMutex::new(Vec::new()));
            let mut runner = FakeEvidenceRunner {
                results: VecDeque::from([
                    Ok("last retained log line\n".to_owned()),
                    Ok("inspect ok\n".to_owned()),
                ]),
                events: Arc::clone(&events),
            };
            let mut probe = FakeFailureProbe {
                metrics: Some(Ok("current metric sample\n".to_owned())),
                hot_stats: Some(Ok("hot stats ok\n".to_owned())),
                events: Arc::clone(&events),
            };
            let mut cleanup_runner = FakeCleanupRunner {
                events: Arc::clone(&events),
            };
            let mut journal = RequestErrorJournal::default();
            journal.push(RequestErrorRecord {
                elapsed_since_clock_start: Duration::from_secs(42),
                endpoint: "POST /collections/perf-hot/index".to_owned(),
                identifier: "request_id=7".to_owned(),
                error: RequestFailure::synthetic("connection reset by peer".to_owned(), false),
                status: None,
                body: None,
            });
            let request_error_report = render_request_error_journal(&journal);

            finish_failed_docker_run(
                &evidence,
                &mut runner,
                &mut probe,
                &mut cleanup_runner,
                "container-under-test",
                "volume-under-test",
                &request_error_report,
                None,
            )
            .await;

            let on_disk = fs::read_to_string(evidence.path.join("request-errors.txt"))
                .expect("read retained request-error journal");
            assert_eq!(on_disk, request_error_report);
            assert!(on_disk.contains("endpoint=POST /collections/perf-hot/index"));
            assert!(on_disk.contains("identifier=request_id=7"));
            assert!(on_disk.contains("connection reset by peer"));
        });
    }

    /// The bounded journal must stop growing at the cap and only count the
    /// rest, so a pathological run cannot make the evidence bundle grow
    /// without limit.
    #[test]
    fn request_error_journal_caps_entries_and_counts_the_overflow() {
        let mut journal = RequestErrorJournal::default();
        for index in 0..REQUEST_ERROR_JOURNAL_CAP + 5 {
            journal.push(RequestErrorRecord {
                elapsed_since_clock_start: Duration::from_millis(index as u64),
                endpoint: "POST /collections/perf-hot/index".to_owned(),
                identifier: format!("request_id={index}"),
                error: RequestFailure::synthetic("connection reset by peer".to_owned(), false),
                status: None,
                body: None,
            });
        }
        assert_eq!(journal.records.len(), REQUEST_ERROR_JOURNAL_CAP);
        assert_eq!(journal.overflow, 5);
        let rendered = render_request_error_journal(&journal);
        assert!(rendered.contains("overflow=5"));
    }

    /// A non-2xx workload response is untrusted peer output; the retained
    /// record must carry the status and a bounded body, never the whole
    /// untrusted response.
    #[test]
    fn non_success_status_record_carries_status_and_a_truncated_body() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build non-success status unit-test runtime");
        runtime.block_on(async {
            let oversized_body = "e".repeat(REQUEST_ERROR_BODY_MAX_BYTES + 200);
            let (base, shutdown) = fake_non_success_server(oversized_body.clone()).await;
            let client = reqwest::Client::builder()
                .build()
                .expect("build fake non-success client");
            let request = client
                .post(format!("{base}/collections/perf-hot/index"))
                .json(&json!({}));
            let failure = post_json(request)
                .await
                .expect_err("a non-2xx workload response must classify as a PostJsonFailure");
            let _ = shutdown.send(());
            assert_eq!(failure.status, Some(500));
            let body = failure
                .body
                .expect("a non-2xx failure must carry a captured response body");
            assert!(
                body.len() <= REQUEST_ERROR_BODY_MAX_BYTES + 64,
                "the retained body must stay bounded, got {} bytes",
                body.len()
            );
            assert!(body.contains(&format!(
                "[truncated after {REQUEST_ERROR_BODY_MAX_BYTES} bytes]"
            )));
            assert!(
                !body.contains(&oversized_body),
                "the retained body must never carry the full untrusted response"
            );
        });
    }

    #[test]
    fn qualifying_matrix_keeps_every_mutation_endpoint_and_backend() {
        let cases = qualifying_matrix();
        assert_eq!(cases.len(), 16);
        assert_eq!(FIELD_COUNT, 14);
        assert_eq!(INPUT_SECONDS, 30 * 60);
        assert_eq!(DOCOPS_PER_SECOND, 100);
        assert_eq!(QUERY_QPS, 10);
        assert_eq!(
            cases
                .iter()
                .filter(|case| case.primary_endpoint == Endpoint::Index)
                .count(),
            6,
            "three Index batch cells run for both backends"
        );
        assert_eq!(
            cases
                .iter()
                .filter(|case| case.primary_endpoint == Endpoint::Replace)
                .count(),
            4,
            "two Replace batch cells run for both backends"
        );
        assert_eq!(
            cases
                .iter()
                .filter(|case| case.primary_endpoint == Endpoint::Unindex)
                .count(),
            6,
            "three Unindex batch cells run for both backends"
        );
        assert_eq!(INPUT_SECONDS % 3, 0);
        assert_eq!(
            INPUT_SECONDS / 3 * DOCOPS_PER_SECOND as u64,
            60_000,
            "each qualifying cell has equal 60,000-operation add, update, and delete thirds"
        );
    }

    fn restart_diagnostic_production_region(source: &str) -> &str {
        let start = source
            .find("const RESTART_DIAGNOSTIC_PREFIX: &str = \"PERF_RESTART_DIAGNOSTIC\";")
            .expect("restart diagnostic prefix is in the production harness");
        let end = source
            .find("fn published_loopback_base")
            .expect("restart diagnostic production region ends before port parsing");
        assert!(start < end, "restart diagnostic region is ordered");
        &source[start..end]
    }

    fn restart_diagnostic_production_contains(region: &str, expected: &str) -> bool {
        region.lines().map(str::trim).any(|line| {
            !line.starts_with("//")
                && !line.as_bytes().starts_with(&[b'/', b'*'])
                && line.contains(expected)
        })
    }

    /// The proposed private record keeps an absent phase or first successful
    /// readiness unambiguous. A zero duration must never stand in for either.
    #[test]
    fn restart_phase_diagnostics_success_reports_monotonic_phase_elapsed_and_first_ready() {
        let region = restart_diagnostic_production_region(include_str!("perf_gate.rs"));
        for expected in [
            "struct RestartPhaseDiagnostics {",
            "restart_elapsed: Option<Duration>,",
            "port_lookup_elapsed: Option<Duration>,",
            "readyz_wait_elapsed: Option<Duration>,",
            "readiness_attempts: usize,",
            "first_ready_elapsed: Option<Duration>,",
            "None => \"null\".to_owned(),",
            "eprintln!(\"{record}\");",
            "Observe: FnMut(&str, RestartPhaseDiagnostics),",
            "diagnostics.first_ready_elapsed.get_or_insert_with(|| started.elapsed());",
        ] {
            assert!(
                restart_diagnostic_production_contains(region, expected),
                "missing private restart success diagnostic contract line: {expected}"
            );
        }
    }

    /// All phase durations use the existing monotonic total start and the one
    /// unchanged deadline. The counter increments before each readiness poll.
    #[test]
    fn restart_phase_diagnostics_keeps_one_total_deadline_and_counts_readiness_attempts() {
        let region = restart_diagnostic_production_region(include_str!("perf_gate.rs"));
        for expected in [
            "Instant::now() + STARTUP_TIMEOUT,",
            "deadline.saturating_duration_since(Instant::now())",
            "let restart_started = Instant::now();",
            "diagnostics.restart_elapsed = Some(restart_started.elapsed());",
            "let port_lookup_started = Instant::now();",
            "diagnostics.port_lookup_elapsed = Some(port_lookup_started.elapsed());",
            "let readyz_wait_started = Instant::now();",
            "diagnostics.readyz_wait_elapsed = Some(readyz_wait_started.elapsed());",
            "diagnostics.readiness_attempts += 1;",
        ] {
            assert!(
                restart_diagnostic_production_contains(region, expected),
                "missing shared-deadline restart diagnostic contract line: {expected}"
            );
        }
    }

    /// Each terminal branch emits a bounded record. It names a phase and an
    /// outcome, but it does not render Docker stderr, an HTTP body, or a receipt.
    #[test]
    fn restart_phase_diagnostics_retains_bounded_terminal_records_for_every_phase() {
        let region = restart_diagnostic_production_region(include_str!("perf_gate.rs"));
        for expected in [
            "fn emit_restart_phase_diagnostics(",
            "let record = emit_restart_phase_diagnostics(diagnostics, phase, outcome);",
            "observe(&record, *diagnostics);",
            "\"complete\", \"success\"",
            "\"docker-restart\", \"error\"",
            "\"docker-restart\", \"timeout\"",
            "\"published-port\", \"error\"",
            "\"published-port\", \"timeout\"",
            "\"readyz\", \"timeout\"",
        ] {
            assert!(
                restart_diagnostic_production_contains(region, expected),
                "missing bounded terminal restart diagnostic contract line: {expected}"
            );
        }
    }

    /// A failed post-restart readiness check must leave one bounded, structured
    /// record in the job log.  Phase timing alone cannot tell an HTTP refusal
    /// from a transport failure or a stopped container.
    #[test]
    fn restart_readiness_diagnostics_keep_bounded_http_transport_process_and_deadline_evidence() {
        let region = restart_diagnostic_production_region(include_str!("perf_gate.rs"));
        for expected in [
            "const RESTART_READINESS_BODY_MAX_BYTES: usize = 4 * 1024;",
            "readyz_failure=",
            "readyz_status=",
            "readyz_body=",
            "readyz_transport_error=",
            "docker_process_state=",
            "DockerProcessState",
            "HttpStatus",
            "Transport",
            "Deadline",
            "tokio::time::timeout(remaining, runner.process_state(&self.container))",
        ] {
            assert!(
                restart_diagnostic_production_contains(region, expected),
                "missing bounded restart-readiness diagnostic contract line: {expected}"
            );
        }
    }

    #[test]
    fn restart_readiness_diagnostic_record_names_each_failure_kind_and_caps_text() {
        let capped_body = bounded_restart_diagnostic_text(
            &vec![b'x'; RESTART_READINESS_BODY_MAX_BYTES],
            (RESTART_READINESS_BODY_MAX_BYTES + 1) as u64,
            true,
        );
        let http = RestartReadinessDiagnostics {
            readiness_failure: Some(RestartReadinessFailure::HttpStatus {
                status: "503 Service Unavailable".to_owned(),
                body: capped_body.clone(),
            }),
            docker_process_state: Some(DockerProcessState {
                detail: "running=false exit_code=137 oom_killed=true error=bytes=0 truncated=false text=\"\""
                    .to_owned(),
            }),
        };
        let http_record = render_restart_phase_diagnostics(
            &RestartPhaseDiagnostics::default(),
            &http,
            "readyz",
            "timeout",
        );
        assert!(http_record.contains("readyz_failure=http_status"));
        assert!(http_record.contains("readyz_status=\"503 Service Unavailable\""));
        assert!(http_record.contains("readyz_body=\"bytes=4097 truncated=true"));
        assert!(http_record
            .contains("docker_process_state=\"running=false exit_code=137 oom_killed=true"));

        let transport = RestartReadinessDiagnostics {
            readiness_failure: Some(RestartReadinessFailure::Transport {
                error: "bytes=17 truncated=false text=\"connection refused\"".to_owned(),
            }),
            ..RestartReadinessDiagnostics::default()
        };
        assert!(render_restart_phase_diagnostics(
            &RestartPhaseDiagnostics::default(),
            &transport,
            "readyz",
            "timeout",
        )
        .contains("readyz_failure=transport"));
        let deadline = RestartReadinessDiagnostics {
            readiness_failure: Some(RestartReadinessFailure::Deadline),
            ..RestartReadinessDiagnostics::default()
        };
        assert!(render_restart_phase_diagnostics(
            &RestartPhaseDiagnostics::default(),
            &deadline,
            "readyz",
            "timeout",
        )
        .contains("readyz_failure=deadline"));
        assert!(capped_body.len() <= RESTART_READINESS_BODY_MAX_BYTES + 80);
    }

    /// A trace begins only after the first failed readiness poll. It keeps each
    /// later non-ready poll as a separate classified row. A later 2xx must not
    /// turn the retry history into a success-path artifact.
    #[test]
    fn readyz_readiness_trace_late_binds_and_distinguishes_repeated_nonready() {
        let mut trace = ReadyzReadinessTrace::new("post-restart-readyz");
        trace
            .record_nonready(Duration::from_millis(4), 503)
            .unwrap();
        trace
            .record_nonready(Duration::from_millis(9), 503)
            .unwrap();
        trace.record_cancelled(Duration::from_millis(15)).unwrap();
        let rendered = trace.render_failure().unwrap();
        assert!(rendered.contains("poll=1 elapsed_ms=4 category=http_status:503"));
        assert!(rendered.contains("poll=2 elapsed_ms=9 category=http_status:503"));
        assert!(rendered.ends_with("terminal=cancelled"));
    }

    /// Failure evidence must retain at most 512 readiness rows. The omitted
    /// counter proves that an active readiness loop did not silently drop data.
    #[test]
    fn readyz_readiness_trace_caps_rows_and_counts_omissions() {
        let mut trace = ReadyzReadinessTrace::new("post-restart-readyz");
        for poll in 0..513_u64 {
            trace
                .record_nonready(Duration::from_millis(poll), 503)
                .unwrap();
        }
        trace.record_cancelled(Duration::from_millis(513)).unwrap();
        let rendered = trace.render_failure().unwrap();
        assert_eq!(
            rendered
                .lines()
                .filter(|line| line.starts_with("poll="))
                .count(),
            512
        );
        assert!(rendered.contains("polls_omitted=2"));
    }

    /// The readiness trace is a classification record. It must not retain an
    /// HTTP body, endpoint text, or a transport error string from the failed
    /// probe.
    #[test]
    fn readyz_readiness_trace_does_not_retain_raw_errors_or_bodies() {
        let mut trace = ReadyzReadinessTrace::new("post-restart-readyz");
        trace
            .record_nonready(Duration::from_millis(4), 503)
            .unwrap();
        trace.record_transport(Duration::from_millis(9)).unwrap();
        trace.record_cancelled(Duration::from_millis(15)).unwrap();
        let rendered = trace.render_failure().unwrap();
        for forbidden in ["body=", "error=", "http://", "https://", "/readyz"] {
            assert!(!rendered.contains(forbidden));
        }
    }

    /// A real local readyz endpoint may return 503 before it becomes healthy.
    /// The later 2xx clears the shared failure-only trace without an artifact.
    #[test]
    fn readyz_readiness_trace_success_does_not_create_trace_or_change_receipt() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build readiness test runtime")
            .block_on(async {
                let (mut server, requests, task) = fake_ready_lumen_with_statuses(vec![
                    axum::http::StatusCode::SERVICE_UNAVAILABLE,
                    axum::http::StatusCode::OK,
                ])
                .await;
                let mut runner = ScriptedRestartRunner {
                    calls: Vec::new(),
                    port_result: Some(Ok(format!(
                        "{}\n",
                        server.base.strip_prefix("http://").expect("loopback base")
                    ))),
                };

                server
                    .restart_and_wait_ready_with(&mut runner)
                    .await
                    .expect("a later local 2xx satisfies restart readiness");

                assert_eq!(requests.load(std::sync::atomic::Ordering::SeqCst), 2);
                assert_eq!(
                    runner.calls,
                    [
                        "restart restart-test-owner",
                        "port restart-test-owner 7373/tcp"
                    ]
                );
                assert!(
                    server
                        .readyz_readiness_trace
                        .lock()
                        .expect("read shared readiness trace")
                        .is_none(),
                    "a real 2xx must discard earlier 503 trace rows"
                );
                task.abort();
            });
    }

    /// The failure-only trace is line-oriented evidence. It binds the stable
    /// readyz listener, keeps the poll categories in arrival order, and ends
    /// at one terminal readiness failure with monotonic elapsed times.
    #[test]
    fn readyz_readiness_trace_renders_ordered_safe_listener_bound_terminal_failure() {
        let mut trace = ReadyzReadinessTrace::new("post-restart-readyz");
        trace
            .record_nonready(Duration::from_millis(4), 503)
            .expect("first failed poll is accepted");
        trace
            .record_transport(Duration::from_millis(9))
            .expect("transport poll follows the first non-ready response");
        trace
            .record_deadline(Duration::from_millis(15))
            .expect("deadline is the terminal readiness poll");

        let rendered = trace
            .render_failure()
            .expect("terminal failure renders one readiness trace");
        let lines = rendered.lines().collect::<Vec<_>>();
        assert_eq!(lines[0], "schema_version=1");
        assert_eq!(lines[1], "phase=readyz");
        assert_eq!(lines[2], "listener=post-restart-readyz");
        assert_eq!(lines[3], "restart_command=docker restart");
        assert_eq!(lines[4], "poll=1 elapsed_ms=4 category=http_status:503");
        assert_eq!(lines[5], "poll=2 elapsed_ms=9 category=transport_error");
        assert_eq!(lines[6], "poll=3 elapsed_ms=15 category=request_timeout");
        assert_eq!(lines[7], "polls_omitted=0");
        assert_eq!(lines[8], "terminal=request_timeout");
    }

    /// A clock regression must not make a readiness trace claim a false order.
    #[test]
    fn readyz_readiness_trace_rejects_non_monotonic_poll_timing() {
        let mut trace = ReadyzReadinessTrace::new("post-restart-readyz");
        trace
            .record_nonready(Duration::from_millis(9), 503)
            .expect("first poll is accepted");
        assert!(
            trace.record_transport(Duration::from_millis(8)).is_err(),
            "a later readiness poll with an earlier elapsed time must fail closed"
        );
    }

    /// The bounded trace keeps the first 512 rows and reports every later poll
    /// through one omission counter instead of allocating unbounded evidence.
    #[test]
    fn readyz_readiness_trace_caps_rows_and_reports_all_omissions() {
        let mut trace = ReadyzReadinessTrace::new("post-restart-readyz");
        for poll in 0..521_u64 {
            trace
                .record_nonready(Duration::from_millis(poll), 503)
                .expect("monotonic non-ready poll is accepted");
        }
        trace
            .record_deadline(Duration::from_millis(521))
            .expect("terminal deadline is accepted after capped rows");
        let rendered = trace
            .render_failure()
            .expect("capped terminal failure still renders evidence");
        assert_eq!(
            rendered
                .lines()
                .filter(|line| line.starts_with("poll="))
                .count(),
            512,
            "the retained readiness evidence has a fixed 512-row cap"
        );
        assert!(
            rendered.contains("polls_omitted=10"),
            "nine capped non-ready polls plus the terminal deadline are counted"
        );
        assert!(rendered.ends_with("terminal=request_timeout"));
    }

    /// Rendered trace data is safe metadata only. It must not put a request
    /// body, a transport error, or a loopback URL in the failure artifact.
    #[test]
    fn readyz_readiness_trace_rendering_excludes_body_error_and_url_data() {
        let mut trace = ReadyzReadinessTrace::new("post-restart-readyz");
        trace
            .record_nonready(Duration::from_millis(4), 503)
            .expect("non-ready status is safe metadata");
        trace
            .record_transport(Duration::from_millis(9))
            .expect("transport category has no raw error payload");
        trace
            .record_deadline(Duration::from_millis(15))
            .expect("terminal category has no raw error payload");
        let rendered = trace
            .render_failure()
            .expect("terminal failure renders safe trace metadata");
        for forbidden in [
            "body=",
            "error=",
            "http://",
            "https://",
            "127.0.0.1",
            "/readyz",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "rendered readiness trace must not contain {forbidden:?}"
            );
        }
    }

    /// A real post-input cancellation drops the hanging restart future. The
    /// shared trace must still reach the standard failure finalizer before its
    /// Docker cleanup boundary.
    #[test]
    fn readyz_readiness_trace_is_written_before_cleanup_on_failure() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build readiness test runtime")
            .block_on(async {
                let (mut server, requests, task) = fake_ready_lumen_with_503_then_pending().await;
                let mut restart = ScriptedRestartRunner {
                    calls: Vec::new(),
                    port_result: Some(Ok(format!(
                        "{}\n",
                        server.base.strip_prefix("http://").expect("loopback base")
                    ))),
                };
                let timeout = Duration::from_millis(500);
                let evidence_root = tempfile::tempdir().expect("create failure evidence root");
                let events = Arc::new(StdMutex::new(Vec::new()));
                let mut evidence_runner = FakeEvidenceRunner {
                    results: VecDeque::from([
                        Ok("local docker logs\n".to_owned()),
                        Ok("local docker inspect\n".to_owned()),
                    ]),
                    events: Arc::clone(&events),
                };
                let mut cleanup_runner = TraceCheckingCleanupRunner {
                    evidence_root: evidence_root.path().to_owned(),
                    events: Arc::clone(&events),
                    checked_trace: false,
                };
                let result = run_restart_post_input_case_with_failure_finalizer(
                    &mut server,
                    tokio::time::Instant::now() + timeout,
                    timeout,
                    &mut restart,
                    evidence_root.path(),
                    &mut evidence_runner,
                    &mut cleanup_runner,
                )
                .await;
                assert!(matches!(
                    result,
                    Err(HarnessError::PostInputTimeout {
                        stage: "restart",
                        ..
                    })
                ));
                assert!(
                    requests.load(std::sync::atomic::Ordering::SeqCst) >= 2,
                    "the local readiness endpoint must return 503 before its hanging poll"
                );
                assert_eq!(
                    restart.calls,
                    [
                        "restart restart-test-owner",
                        "port restart-test-owner 7373/tcp"
                    ]
                );

                let evidence = fs::read_dir(evidence_root.path())
                    .expect("list real failure evidence")
                    .find_map(|entry| entry.ok().map(|entry| entry.path()))
                    .expect("one real failure evidence directory");
                let trace = fs::read_to_string(evidence.join(READYZ_READINESS_TRACE_FILE))
                    .expect("read real readiness trace artifact");
                for expected in [
                    "schema_version=1",
                    "phase=readyz",
                    "listener=post-restart-readyz",
                    "restart_command=docker restart",
                    "category=http_status:503",
                    "category=cancelled",
                    "polls_omitted=0",
                    "terminal=cancelled",
                ] {
                    assert!(trace.contains(expected), "missing {expected:?} in {trace}");
                }
                for forbidden in ["unsafe-readyz-body-must-not-render", "http://", "/readyz"] {
                    assert!(
                        !trace.contains(forbidden),
                        "the safe readiness trace must not contain {forbidden:?}"
                    );
                }
                assert_eq!(
                    events.lock().expect("read finalizer events").as_slice(),
                    [
                        format!(
                            "docker logs --tail {} restart-test-owner",
                            EVIDENCE_LOG_TAIL_LINES
                        ),
                        "docker inspect restart-test-owner".to_owned(),
                        "trace-present-before:docker rm".to_owned(),
                        "docker rm -f restart-test-owner".to_owned(),
                        "docker volume rm -f restart-test-volume".to_owned(),
                    ]
                );
                task.abort();
            });
    }

    /// A failed restart command never reaches the readyz listener. The regular
    /// failure bundle still exists, but it must not contain a readiness trace.
    #[test]
    fn readyz_readiness_trace_pre_poll_restart_failure_creates_no_trace_artifact() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build readiness test runtime")
            .block_on(async {
                let (mut server, requests, task) = fake_ready_lumen().await;
                let mut restart = DiagnosticRestartRunner {
                    restart_result: Some(Err(HarnessError::Startup("restart failed".to_owned()))),
                    restart_delay: Duration::ZERO,
                    port_result: Some(Ok("127.0.0.1:1".to_owned())),
                    port_delay: Duration::ZERO,
                };
                let evidence_root = tempfile::tempdir().expect("create pre-poll evidence root");
                let events = Arc::new(StdMutex::new(Vec::new()));
                let mut evidence_runner = FakeEvidenceRunner {
                    results: VecDeque::from([
                        Ok("local docker logs\n".to_owned()),
                        Ok("local docker inspect\n".to_owned()),
                    ]),
                    events: Arc::clone(&events),
                };
                let mut cleanup_runner = FakeCleanupRunner { events };
                let timeout = Duration::from_millis(100);
                let result = run_restart_post_input_case_with_failure_finalizer(
                    &mut server,
                    tokio::time::Instant::now() + timeout,
                    timeout,
                    &mut restart,
                    evidence_root.path(),
                    &mut evidence_runner,
                    &mut cleanup_runner,
                )
                .await;
                assert!(matches!(result, Err(HarnessError::Startup(_))));
                assert_eq!(requests.load(std::sync::atomic::Ordering::SeqCst), 0);
                let evidence = fs::read_dir(evidence_root.path())
                    .expect("list regular pre-poll failure evidence")
                    .find_map(|entry| entry.ok().map(|entry| entry.path()))
                    .expect("regular failure evidence exists");
                assert!(
                    !evidence.join(READYZ_READINESS_TRACE_FILE).exists(),
                    "a pre-poll restart failure leaves no readiness trace artifact"
                );
                task.abort();
            });
    }

    #[test]
    fn workload_slots_are_absolute_for_independent_queries_and_paced_mutations() {
        let second = 17;
        let mutation_slots = (0..DOCOPS_PER_SECOND)
            .map(|offset| mutation_slot(second, offset))
            .collect::<Vec<_>>();
        let query_slots = (0..QUERY_QPS)
            .map(|offset| query_slot(second, offset))
            .collect::<Vec<_>>();

        assert_eq!(mutation_slots[0], Duration::from_secs(second));
        assert_eq!(mutation_slots[99], Duration::from_millis(17_990));
        assert_eq!(query_slots[0], Duration::from_secs(second));
        assert_eq!(query_slots[9], Duration::from_millis(17_900));
        assert!(
            mutation_slots
                .windows(2)
                .all(|slots| slots[1] - slots[0] == MUTATION_SLOT),
            "each mutation document must keep its absolute 10 ms slot"
        );
        assert!(
            query_slots
                .windows(2)
                .all(|slots| slots[1] - slots[0] == QUERY_SLOT),
            "each query offer must keep its independent absolute 100 ms slot"
        );
    }
}
// DURABLE-WORKLOAD-END
// CODEGEN-END
