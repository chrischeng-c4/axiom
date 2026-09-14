//! Oversized Unicode Text staging contract for Lumen #4246.
//!
//! # Facets
//!
//! - Behavior: `apps/lumen/e2e/oversized_text_staging.rs:355-359` accepts
//!   legal oversized Text. Its BM25 assertion at `:314-319` runs live
//!   (`:360`) and cold (`:393`); `:366-382` checks checkpoint publication.
//!   The prepared-row case calls its BM25 helper `:443-481` at `:546-585`,
//!   `:601-624`, and `:641-703` to require ordinal last-write-wins, stale
//!   version rejection, partial-error prefix recovery, and full replacement
//!   with omitted-Text deletion, live and cold. Change points:
//!   `.aw/workitems/deliveries/lumen061-04-bounded-compaction-backpressure.md:20-25`
//!   names storage, the API, coordinator, SegmentRdbStore, the process
//!   scheduler, and metrics for this path.
//! - Security: the HTTP body boundary is pinned by
//!   `apps/lumen/e2e/http_body_limit_e2e.rs:67-119` and `:124-189`; this
//!   file proves its own bodies stay below it at `:343-347` and `:529-533`.
//!   It sends a caller-controlled unknown field after valid prepared rows
//!   and requires the closed `422 unknown_field` outcome at `:539-544`.
//!   The staged-run file boundary is closed by `apps/lumen/src/text_row_stage.rs:954-965`,
//!   which rejects a truncated prefix read from the workspace at `:165-172`
//!   and `:287-332`.
//! - Performance: `apps/lumen/ROADMAP.md:60-70` promises a 256 MiB total
//!   for active, frozen, and reserved changes. This file executes public
//!   pending-budget assertions at `:227-236` after apply (`:361-362`) and
//!   publication (`:383-384`). It requires engine-scoped Text staging
//!   counters at `:400-414`, so this request cannot be credited to another
//!   test's process-wide peak; `:73-80` serializes shared gauge reads. The gauge source is
//!   `apps/lumen/src/metrics.rs:422-428` and `:715-742`.
//!
//! The fixture is intentionally not a 30-minute benchmark. It checks the
//! current per-record pending-memory limit and the actual staging path. The
//! release workload remains responsible for latency and RSS budgets.

use axum::http::StatusCode;
use axum_test::TestServer;
use lumen::aof::AofWriter;
use lumen::api::{router, AppState, CheckpointSink};
use lumen::auth::AuthConfig;
use lumen::coordinator::{SharedAof, WriteCoordinator, WriteSink};
use lumen::segment_checkpoint::SegmentCheckpointSink;
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::Engine;
use lumen::wal::{MemWal, SharedWal};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::Mutex as AsyncMutex;

const COLLECTION: &str = "oversized-unicode-ngram";
const FIELD: &str = "body";
const OVERSIZED_ID: &str = "oversized-unicode";
const EMPTY_ID: &str = "empty-text";
const MAX_HTTP_BODY_BYTES: usize = 8 * 1024 * 1024;
const PENDING_HARD_LIMIT_BYTES: u64 = 256 * 1024 * 1024;

// A de Bruijn cycle of order three over 70 CJK scalars has 343,000 source
// scalars. `İ` expands to `i` + U+0307 during lowercasing, so default
// 2/3-gram normalization emits 686,001 windows. The current conservative
// record estimator charges more than the approved 256 MiB pending budget
// for active + frozen Text ownership, while the JSON body is about 1 MiB.
const CJK_ALPHABET_LEN: usize = 70;
const NGRAM_ORDER: usize = 3;
const CJK_BODY_SCALARS: usize = CJK_ALPHABET_LEN * CJK_ALPHABET_LEN * CJK_ALPHABET_LEN;
const NORMALIZED_SCALARS: usize = CJK_BODY_SCALARS + 2;
const EXPECTED_NGRAM_WINDOWS: usize = 2 * NORMALIZED_SCALARS - 3;

// These must be Engine metrics, not ChangeBudget process metrics. A new
// fixture Engine starts both counters at zero, so the assertion cannot be
// satisfied by another test's earlier staging work.
const TEXT_STAGE_ROWS_METRIC: &str = "lumen_text_row_stage_rows_total";
const TEXT_STAGE_INPUT_BYTES_METRIC: &str = "lumen_text_row_stage_input_bytes_total";

// ChangeBudget is process-wide. These two large-row tests each use a fresh
// Engine, but must not overlap their pending-budget measurements in one test
// process.
static OVERSIZED_TEXT_PENDING_GAUGE_LOCK: OnceLock<AsyncMutex<()>> = OnceLock::new();

fn oversized_text_pending_gauge_lock() -> &'static AsyncMutex<()> {
    OVERSIZED_TEXT_PENDING_GAUGE_LOCK.get_or_init(|| AsyncMutex::new(()))
}

struct OversizedTextFixture {
    _dir: tempfile::TempDir,
    engine: Arc<Engine>,
    store: Arc<SegmentRdbStore>,
    server: TestServer,
}

fn oversized_text_fixture() -> OversizedTextFixture {
    let dir = tempfile::tempdir().expect("oversized Text fixture directory");
    let root = dir.path().join("segments");
    let store = Arc::new(SegmentRdbStore::new(&root).expect("open oversized Text store"));
    let aof: SharedAof = Arc::new(Mutex::new(
        AofWriter::open(dir.path().join("aof.log")).expect("open oversized Text AOF"),
    ));
    let engine = Arc::new(Engine::new());
    let wal: SharedWal = Arc::new(MemWal::new());
    let writer = WriteCoordinator::start_from_with_aof(wal, engine.clone(), 0, aof.clone());
    let sink_writer: Arc<dyn WriteSink> = writer;
    let checkpoint = Arc::new(SegmentCheckpointSink {
        engine: engine.clone(),
        store: store.clone(),
        writer: sink_writer.clone(),
        aof: Some(aof),
    });
    let checkpoint_api: Arc<dyn CheckpointSink> = checkpoint.clone();
    let state =
        AppState::with_components(engine.clone(), Arc::new(AuthConfig::open()), sink_writer)
            .with_checkpoint(checkpoint_api);
    let server = TestServer::new(router(state)).expect("oversized Text HTTP server");
    OversizedTextFixture {
        _dir: dir,
        engine,
        store,
        server,
    }
}

/// Emit a de Bruijn sequence without storing a large set of String keys.
/// Every cyclic three-scalar word occurs exactly once. The linear body has
/// `CJK_BODY_SCALARS - 2` distinct three-scalar windows, enough to avoid a
/// repeated-token fixture that the normalizer could cheaply deduplicate.
fn oversized_unicode_text() -> (String, String) {
    fn visit(
        position: usize,
        period: usize,
        alphabet_len: usize,
        order: usize,
        work: &mut [usize],
        output: &mut Vec<usize>,
    ) {
        if position > order {
            if order % period == 0 {
                output.extend_from_slice(&work[1..=period]);
            }
            return;
        }
        let prior = work[position - period];
        work[position] = prior;
        visit(position + 1, period, alphabet_len, order, work, output);
        for next in prior + 1..alphabet_len {
            work[position] = next;
            visit(position + 1, position, alphabet_len, order, work, output);
        }
    }

    let alphabet = (0..CJK_ALPHABET_LEN)
        .map(|offset| {
            char::from_u32(0x4E00 + u32::try_from(offset).expect("CJK alphabet offset"))
                .expect("CJK alphabet scalar")
        })
        .collect::<Vec<_>>();
    let mut work = vec![0usize; CJK_ALPHABET_LEN * NGRAM_ORDER + 1];
    let mut cycle = Vec::with_capacity(CJK_BODY_SCALARS);
    visit(1, 1, CJK_ALPHABET_LEN, NGRAM_ORDER, &mut work, &mut cycle);
    assert_eq!(
        cycle.len(),
        CJK_BODY_SCALARS,
        "the Unicode fixture must contain one cycle position per three-gram",
    );
    let body = cycle
        .iter()
        .map(|index| alphabet[*index])
        .collect::<String>();
    let probe = body.chars().skip(12_345).take(3).collect::<String>();
    assert_eq!(
        probe.chars().count(),
        NGRAM_ORDER,
        "the match probe must contain one complete distinct CJK trigram",
    );
    let mut text = String::with_capacity("İ".len() + body.len());
    text.push('İ');
    text.push_str(&body);
    assert_eq!(
        text.chars().count(),
        CJK_BODY_SCALARS + 1,
        "source scalar count excludes only the lowercase expansion",
    );
    assert_eq!(
        EXPECTED_NGRAM_WINDOWS, 686_001,
        "fixture normalization count must stay pinned to the intended oversized path",
    );
    (text, probe)
}

fn metric_u64(metrics: &str, name: &str) -> u64 {
    let values = metrics
        .lines()
        .filter_map(|line| {
            let (metric, value) = line.split_once(|character: char| character.is_whitespace())?;
            (metric == name).then_some(value.trim())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        values.len(),
        1,
        "{name} must appear exactly once in public /metrics: {metrics}",
    );
    values[0].parse::<u64>().unwrap_or_else(|_| {
        panic!(
            "{name} must be an unsigned byte/count metric: {}",
            values[0]
        )
    })
}

#[derive(Debug)]
struct PendingBudget {
    reserved: u64,
    active: u64,
    frozen: u64,
    total: u64,
}

async fn pending_budget(server: &TestServer) -> PendingBudget {
    let response = server.get("/metrics").await;
    response.assert_status_ok();
    let metrics = response.text();
    PendingBudget {
        reserved: metric_u64(&metrics, "lumen_pending_change_reserved_bytes"),
        active: metric_u64(&metrics, "lumen_pending_change_active_bytes"),
        frozen: metric_u64(&metrics, "lumen_pending_change_frozen_bytes"),
        total: metric_u64(&metrics, "lumen_pending_change_total_bytes"),
    }
}

fn assert_pending_budget_is_bounded(snapshot: &PendingBudget, phase: &str) {
    assert_eq!(
        snapshot.total,
        snapshot.reserved + snapshot.active + snapshot.frozen,
        "{phase}: public pending total must equal its three documented ownership states: {snapshot:?}",
    );
    assert!(
        snapshot.total <= PENDING_HARD_LIMIT_BYTES,
        "{phase}: one oversized Text record must never make public pending accounting exceed the approved 256 MiB limit: {snapshot:?}",
    );
}

#[derive(Debug)]
struct TextStageCounters {
    rows: u64,
    input_bytes: u64,
}

fn optional_metric_u64(metrics: &str, name: &str) -> Option<u64> {
    let values = metrics
        .lines()
        .filter_map(|line| {
            let (metric, value) = line.split_once(|character: char| character.is_whitespace())?;
            (metric == name).then_some(value.trim())
        })
        .collect::<Vec<_>>();
    match values.as_slice() {
        [] => None,
        [value] => {
            Some(value.parse::<u64>().unwrap_or_else(|_| {
                panic!("{name} must be an unsigned byte/count metric: {value}")
            }))
        }
        _ => panic!("{name} must appear at most once in public /metrics: {metrics}"),
    }
}

async fn text_stage_counters(server: &TestServer) -> (Option<u64>, Option<u64>) {
    let response = server.get("/metrics").await;
    response.assert_status_ok();
    let metrics = response.text();
    (
        optional_metric_u64(&metrics, TEXT_STAGE_ROWS_METRIC),
        optional_metric_u64(&metrics, TEXT_STAGE_INPUT_BYTES_METRIC),
    )
}

async fn match_ids(server: &TestServer, text: &str, context: &str) -> Vec<String> {
    let response = server
        .post(&format!("/collections/{COLLECTION}/search"))
        .json(&json!({
            "query": { "match": { "field": FIELD, "text": text, "op": "and" } },
            "limit": 8,
            "track_total": true,
        }))
        .await;
    response.assert_status_ok();
    let body: Value = response.json();
    let hits = body["hits"].as_array().expect("Text Match hits array");
    assert_eq!(
        body["total"].as_u64(),
        Some(hits.len() as u64),
        "{context}: bounded fixture must return every Text Match hit: {body}",
    );
    let mut ids = hits
        .iter()
        .map(|hit| {
            assert!(
                hit["score"].as_f64().is_some(),
                "{context}: Text Match must expose a numeric BM25 score: {body}",
            );
            hit["external_id"]
                .as_str()
                .expect("Text Match external ID")
                .to_owned()
        })
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

async fn assert_text_semantics(server: &TestServer, cjk_probe: &str, phase: &str) {
    for (query, name) in [
        ("İ", "uppercase Unicode marker"),
        ("i\u{307}", "lowercase Unicode marker"),
        (cjk_probe, "distinct CJK trigram"),
    ] {
        assert_eq!(
            match_ids(server, query, &format!("{phase} {name}")).await,
            vec![OVERSIZED_ID.to_owned()],
            "{phase}: {name} must match only the oversized Text document; the explicit empty Text row must not match",
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn oversized_unicode_ngram_text_is_staged_bounded_and_cold_recoverable() {
    let _pending_gauge_lock = oversized_text_pending_gauge_lock().lock().await;
    let fixture = oversized_text_fixture();
    fixture
        .server
        .put(&format!("/collections/{COLLECTION}"))
        .json(&json!({
            "fields": { FIELD: { "type": "text", "analyzer": "ngram" } }
        }))
        .await
        .assert_status_ok();

    let (text, cjk_probe) = oversized_unicode_text();
    let request = json!({
        "items": [
            { "external_id": OVERSIZED_ID, "field": FIELD, "value": text.clone() },
            { "external_id": EMPTY_ID, "field": FIELD, "value": "" },
        ]
    });
    let encoded = serde_json::to_vec(&request).expect("serialize legal oversized Text request");
    assert!(
        encoded.len() < MAX_HTTP_BODY_BYTES,
        "the staged Text request must remain below the existing 8 MiB HTTP body limit: {} bytes",
        encoded.len(),
    );

    let response = fixture
        .server
        .post(&format!("/collections/{COLLECTION}/index"))
        .json(&request)
        .await;
    response.assert_status_ok();
    assert_eq!(
        response.json::<Value>()["indexed"].as_u64(),
        Some(2),
        "one oversized Text row and one empty Text row must both be accepted",
    );
    assert_text_semantics(&fixture.server, &cjk_probe, "live").await;
    let live_pending = pending_budget(&fixture.server).await;
    assert_pending_budget_is_bounded(&live_pending, "after oversized Text apply");

    let checkpoint = fixture.server.post("/admin/checkpoint").await;
    checkpoint.assert_status_ok();
    assert_eq!(
        checkpoint.json::<Value>()["persisted"],
        true,
        "the real checkpoint endpoint must publish the oversized Text generation",
    );
    let (tokens_after_checkpoint, has_segment) = fixture
        .engine
        .segment_field_probe(COLLECTION, FIELD)
        .expect("probe oversized Text segment");
    assert_eq!(
        tokens_after_checkpoint, 0,
        "published Text must release its live token driver after durable staging",
    );
    assert!(
        has_segment,
        "published Text must attach its durable segment after checkpoint",
    );
    let checkpoint_pending = pending_budget(&fixture.server).await;
    assert_pending_budget_is_bounded(&checkpoint_pending, "after oversized Text publication");

    let cold = fixture
        .store
        .load_current_generation()
        .expect("load oversized Text CURRENT")
        .expect("oversized Text generation");
    let cold_server = TestServer::new(router(AppState::open(cold.engine)))
        .expect("cold oversized Text HTTP server");
    assert_text_semantics(&cold_server, &cjk_probe, "cold").await;

    // Keep performance checks last. A legacy oversized-request bypass can
    // be functionally correct for this fixture, so it must not hide a missing
    // durable Text-row staging path behind process-wide pending metrics.
    // Engine-scoped counters bind this request to that staged path.
    let (rows, input_bytes) = text_stage_counters(&fixture.server).await;
    assert!(
        rows.is_some() && input_bytes.is_some(),
        "public /metrics must expose Engine-scoped {TEXT_STAGE_ROWS_METRIC} and {TEXT_STAGE_INPUT_BYTES_METRIC} so this request cannot hide behind a process-wide peak",
    );
    let stage = TextStageCounters {
        rows: rows.expect("stage rows asserted above"),
        input_bytes: input_bytes.expect("stage bytes asserted above"),
    };
    assert!(
        stage.rows >= 1,
        "the oversized Text request must use at least one engine-scoped durable Text-row stage: {stage:?}",
    );
    assert!(
        stage.input_bytes >= u64::try_from(text.len()).expect("Text byte count"),
        "durable Text-row staging must account for the oversized caller bytes: {stage:?}",
    );
}

const PREPARED_ROWS_ID: &str = "oversized-prepared-row";
const SECONDARY_TEXT_FIELD: &str = "secondary";
const BASE_MARKER: &str = "\u{9fa5}\u{9fa6}\u{9fa7}";
const FIRST_MARKER: &str = "\u{9fa8}\u{9fa9}\u{9faa}";
const FINAL_MARKER: &str = "\u{9fab}\u{9fac}\u{9fad}";
const STALE_MARKER: &str = "\u{9fae}\u{9faf}\u{9fb0}";
const REPLACEMENT_MARKER: &str = "\u{9fb1}\u{9fb2}\u{9fb3}";
const SECONDARY_MARKER: &str = "\u{9fb4}\u{9fb5}\u{9fb6}";

fn marked_oversized_text(corpus: &str, marker: &str) -> String {
    let mut text = String::with_capacity(marker.len() + corpus.len());
    text.push_str(marker);
    text.push_str(corpus);
    text
}

async fn assert_field_match_ids(
    server: &TestServer,
    field: &str,
    text: &str,
    expected: &[&str],
    context: &str,
) {
    let response = server
        .post(&format!("/collections/{COLLECTION}/search"))
        .json(&json!({
            "query": { "match": { "field": field, "text": text, "op": "and" } },
            "limit": 8,
            "track_total": true,
        }))
        .await;
    response.assert_status_ok();
    let body: Value = response.json();
    let hits = body["hits"]
        .as_array()
        .expect("prepared Text Match hits array");
    assert_eq!(
        body["total"].as_u64(),
        Some(hits.len() as u64),
        "{context}: prepared Text Match must report every returned hit: {body}",
    );
    let mut ids = hits
        .iter()
        .map(|hit| {
            assert!(
                hit["score"].as_f64().is_some(),
                "{context}: prepared Text Match must expose a numeric BM25 score: {body}",
            );
            hit["external_id"]
                .as_str()
                .expect("prepared Text Match external ID")
                .to_owned()
        })
        .collect::<Vec<_>>();
    ids.sort();
    let mut expected = expected
        .iter()
        .map(|external_id| (*external_id).to_owned())
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(
        ids, expected,
        "{context}: unexpected prepared Text Match IDs"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn oversized_text_prepared_rows_preserve_update_stale_partial_error_and_replace_deletion() {
    let _pending_gauge_lock = oversized_text_pending_gauge_lock().lock().await;
    let fixture = oversized_text_fixture();
    fixture
        .server
        .put(&format!("/collections/{COLLECTION}"))
        .json(&json!({
            "fields": {
                FIELD: { "type": "text", "analyzer": "ngram" },
                SECONDARY_TEXT_FIELD: { "type": "text", "analyzer": "ngram" },
            }
        }))
        .await
        .assert_status_ok();

    fixture
        .server
        .post(&format!("/collections/{COLLECTION}/index"))
        .json(&json!({ "items": [
            { "external_id": PREPARED_ROWS_ID, "field": FIELD, "value": BASE_MARKER },
            { "external_id": PREPARED_ROWS_ID, "field": SECONDARY_TEXT_FIELD, "value": SECONDARY_MARKER },
        ] }))
        .await
        .assert_status_ok();
    let base_checkpoint = fixture.server.post("/admin/checkpoint").await;
    base_checkpoint.assert_status_ok();
    assert_eq!(
        base_checkpoint.json::<Value>()["persisted"],
        true,
        "the ordinary Text base must publish before prepared-row updates",
    );

    let (corpus, _) = oversized_unicode_text();
    let first = marked_oversized_text(&corpus, FIRST_MARKER);
    let final_value = marked_oversized_text(&corpus, FINAL_MARKER);
    let stale = marked_oversized_text(&corpus, STALE_MARKER);
    let partial_request = json!({ "items": [
        { "external_id": PREPARED_ROWS_ID, "field": FIELD, "value": first, "version": 41 },
        { "external_id": PREPARED_ROWS_ID, "field": FIELD, "value": final_value, "version": 42 },
        { "external_id": PREPARED_ROWS_ID, "field": FIELD, "value": stale, "version": 41 },
        { "external_id": PREPARED_ROWS_ID, "field": "unknown-after-oversized-prefix", "value": "must-error" },
    ] });
    let encoded =
        serde_json::to_vec(&partial_request).expect("serialize prepared Text update request");
    assert!(
        encoded.len() < MAX_HTTP_BODY_BYTES,
        "the valid oversized prepared-row request must stay below the HTTP body limit: {} bytes",
        encoded.len(),
    );
    let partial = fixture
        .server
        .post(&format!("/collections/{COLLECTION}/index"))
        .json(&partial_request)
        .await;
    partial.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    let partial_body: Value = partial.json();
    assert_eq!(
        partial_body["error"], "unknown_field",
        "a malformed later item must keep the public unknown-field refusal: {partial_body}",
    );

    assert_field_match_ids(
        &fixture.server,
        FIELD,
        FINAL_MARKER,
        &[PREPARED_ROWS_ID],
        "live final ordinal",
    )
    .await;
    assert_field_match_ids(
        &fixture.server,
        FIELD,
        FIRST_MARKER,
        &[],
        "live replaced earlier ordinal",
    )
    .await;
    assert_field_match_ids(
        &fixture.server,
        FIELD,
        STALE_MARKER,
        &[],
        "live stale ordinal",
    )
    .await;
    assert_field_match_ids(
        &fixture.server,
        FIELD,
        BASE_MARKER,
        &[],
        "live replaced ordinary base",
    )
    .await;
    assert_field_match_ids(
        &fixture.server,
        SECONDARY_TEXT_FIELD,
        SECONDARY_MARKER,
        &[PREPARED_ROWS_ID],
        "live untouched secondary field after partial error",
    )
    .await;

    let prefix_checkpoint = fixture.server.post("/admin/checkpoint").await;
    prefix_checkpoint.assert_status_ok();
    assert_eq!(
        prefix_checkpoint.json::<Value>()["persisted"],
        true,
        "the checkpoint must publish the valid oversized prefix before the error",
    );
    let prefix_cold = fixture
        .store
        .load_current_generation()
        .expect("load partial-error prepared Text CURRENT")
        .expect("partial-error prepared Text generation");
    let prefix_cold_server = TestServer::new(router(AppState::open(prefix_cold.engine)))
        .expect("partial-error prepared Text cold server");
    assert_field_match_ids(
        &prefix_cold_server,
        FIELD,
        FINAL_MARKER,
        &[PREPARED_ROWS_ID],
        "cold final ordinal after partial error",
    )
    .await;
    assert_field_match_ids(
        &prefix_cold_server,
        FIELD,
        STALE_MARKER,
        &[],
        "cold stale ordinal after partial error",
    )
    .await;
    assert_field_match_ids(
        &prefix_cold_server,
        SECONDARY_TEXT_FIELD,
        SECONDARY_MARKER,
        &[PREPARED_ROWS_ID],
        "cold untouched secondary field after partial error",
    )
    .await;

    let replacement_value = marked_oversized_text(&corpus, REPLACEMENT_MARKER);
    let replace = fixture
        .server
        .put(&format!("/collections/{COLLECTION}/docs:replace"))
        .json(&json!({ "docs": [{
            "external_id": PREPARED_ROWS_ID,
            "fields": { FIELD: replacement_value },
        }] }))
        .await;
    replace.assert_status_ok();
    let replace_body: Value = replace.json();
    assert_eq!(
        replace_body["results"][0]["status"], "ok",
        "full replacement of the prepared Text row must report its ordinary success: {replace_body}",
    );
    assert_field_match_ids(
        &fixture.server,
        FIELD,
        REPLACEMENT_MARKER,
        &[PREPARED_ROWS_ID],
        "live replacement Text",
    )
    .await;
    assert_field_match_ids(
        &fixture.server,
        FIELD,
        FINAL_MARKER,
        &[],
        "live removed final Text after replacement",
    )
    .await;
    assert_field_match_ids(
        &fixture.server,
        SECONDARY_TEXT_FIELD,
        SECONDARY_MARKER,
        &[],
        "live omitted Text field after replacement",
    )
    .await;

    let replacement_checkpoint = fixture.server.post("/admin/checkpoint").await;
    replacement_checkpoint.assert_status_ok();
    assert_eq!(
        replacement_checkpoint.json::<Value>()["persisted"],
        true,
        "the checkpoint must publish changed and omitted prepared Text fields",
    );
    let replacement_cold = fixture
        .store
        .load_current_generation()
        .expect("load replacement prepared Text CURRENT")
        .expect("replacement prepared Text generation");
    let replacement_cold_server = TestServer::new(router(AppState::open(replacement_cold.engine)))
        .expect("replacement prepared Text cold server");
    assert_field_match_ids(
        &replacement_cold_server,
        FIELD,
        REPLACEMENT_MARKER,
        &[PREPARED_ROWS_ID],
        "cold replacement Text",
    )
    .await;
    assert_field_match_ids(
        &replacement_cold_server,
        FIELD,
        FINAL_MARKER,
        &[],
        "cold removed final Text after replacement",
    )
    .await;
    assert_field_match_ids(
        &replacement_cold_server,
        SECONDARY_TEXT_FIELD,
        SECONDARY_MARKER,
        &[],
        "cold omitted Text field after replacement",
    )
    .await;
}

/// The optional dictionary analyzer must use the same bounded preparation as
/// other Text analyzers. The corpus crosses the fixed 256 MiB normalized cost
/// while keeping a legal HTTP body. The long ASCII run must remain one token;
/// splitting it at a staging page boundary would change search semantics.
#[cfg(feature = "jieba")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn jieba_dictionary_oversized_row_stages_without_changing_live_or_cold_tokens() {
    let _pending_gauge_lock = oversized_text_pending_gauge_lock().lock().await;
    let fixture = oversized_text_fixture();
    fixture
        .server
        .put(&format!("/collections/{COLLECTION}"))
        .json(&json!({"fields": {FIELD: {"type": "text", "analyzer": "jieba"}}}))
        .await
        .assert_status_ok();
    let before = text_stage_counters(&fixture.server).await;
    let (corpus, _) = oversized_unicode_text();
    let ascii = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".repeat(7000);
    let text = format!("南京市长江大桥, ΣΟΣ {ascii} {corpus}");
    assert!(text.chars().filter(|c| !c.is_whitespace()).count() * 608 > 256 * 1024 * 1024);
    let request = json!({"items": [
        {"external_id": OVERSIZED_ID, "field": FIELD, "value": text},
        {"external_id": OVERSIZED_ID, "field": "undeclared", "value": "refuse"}
    ]});
    assert!(serde_json::to_vec(&request).unwrap().len() < MAX_HTTP_BODY_BYTES);
    let response = fixture
        .server
        .post(&format!("/collections/{COLLECTION}/index"))
        .json(&request)
        .await;
    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(response.json::<Value>()["error"], "unknown_field");

    let probes = ["南京市长江大桥", "ΣΟΣ", ascii.as_str()];
    let mut live = Vec::new();
    for probe in probes {
        let response = fixture.server.post(&format!("/collections/{COLLECTION}/search"))
            .json(&json!({"query": {"match": {"field": FIELD, "text": probe, "op": "and"}}, "track_total": true}))
            .await;
        response.assert_status_ok();
        let body = response.json::<Value>();
        assert_eq!(body["total"], 1);
        assert_eq!(body["hits"][0]["external_id"], OVERSIZED_ID);
        live.push(body["hits"][0]["score"].as_f64().unwrap().to_bits());
    }
    assert_pending_budget_is_bounded(&pending_budget(&fixture.server).await, "dictionary apply");
    fixture
        .server
        .post("/admin/checkpoint")
        .await
        .assert_status_ok();
    assert_eq!(
        fixture
            .engine
            .segment_field_probe(COLLECTION, FIELD)
            .unwrap(),
        (0, true)
    );
    let cold = fixture.store.load_current_generation().unwrap().unwrap();
    let cold_server = TestServer::new(router(AppState::open(cold.engine))).unwrap();
    for (probe, expected_score) in probes.into_iter().zip(live) {
        let response = cold_server.post(&format!("/collections/{COLLECTION}/search"))
            .json(&json!({"query": {"match": {"field": FIELD, "text": probe, "op": "and"}}, "track_total": true}))
            .await;
        response.assert_status_ok();
        let body = response.json::<Value>();
        assert_eq!(body["total"], 1);
        assert_eq!(body["hits"][0]["external_id"], OVERSIZED_ID);
        assert_eq!(
            body["hits"][0]["score"].as_f64().unwrap().to_bits(),
            expected_score
        );
    }
    let after = text_stage_counters(&fixture.server).await;
    assert_eq!(
        after.0.unwrap_or(0),
        before.0.unwrap_or(0) + 1,
        "this dictionary row must use bounded staging, including the optional jieba feature"
    );
    assert_eq!(
        after.1.unwrap_or(0),
        before.1.unwrap_or(0) + request["items"][0]["value"].as_str().unwrap().len() as u64
    );
}
