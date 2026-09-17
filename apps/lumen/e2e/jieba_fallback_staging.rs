#![cfg(not(feature = "jieba"))]

//! Feature-off Jieba fallback staging contract for Lumen #4246.
//!
//! # Facets
//!
//! - Behavior: `jieba_fallback_oversized_row_stages_and_recovers_live_and_cold`
//!   at `:327-375` makes a legal default-feature Jieba fallback write, keeps
//!   its valid prefix after a later invalid field, and requires the fallback
//!   bigrams and their BM25 score bits to agree live and after cold reopen.
//!   Change points are `apps/lumen/src/storage/text_preparation.rs:80-184`,
//!   `apps/lumen/src/storage/staged_text_row.rs:40-186`, and
//!   `apps/lumen/src/tokenize.rs:7-27`.
//! - Security: `:327-332` feeds a caller-controlled undeclared field after
//!   the staged valid prefix and requires the closed `422 unknown_field`
//!   response. `:315-320` also proves this test input stays below the existing
//!   8 MiB HTTP body gate in `apps/lumen/e2e/http_body_limit_e2e.rs:68-122`.
//!   The new fallback stage has no caller-supplied file path: its private
//!   workspace is created by `apps/lumen/src/storage/staged_text_row.rs:94-117`.
//!   The private staged-file corruption boundary has no public e2e injection;
//!   it remains a source-unit coverage gap for the changed fallback reader.
//! - Performance: `apps/lumen/ROADMAP.md:60-70` promises a 256 MiB total for
//!   pending active, frozen, and reserved changes. `:335-338` and `:359-362`
//!   keep the public accounting within that limit. `:379-389` requires the
//!   Engine-scoped staged-row counter delta for this exact oversized fallback
//!   record, so a legacy direct-token path cannot satisfy the memory contract.
//!   `apps/lumen/src/change_record_cost.rs:114-125` and
//!   `apps/lumen/src/storage/record_admission.rs:284-289` choose the staged
//!   representation at that fixed limit.
//!
//! This ordinary case is not a latency or RSS benchmark. It pins the current
//! per-record 256 MiB admission boundary and the bounded streaming route.

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
use std::sync::{Arc, Mutex};

const COLLECTION: &str = "jieba-fallback-staging";
const FIELD: &str = "body";
const DOCUMENT_ID: &str = "large-fallback-row";
const UNKNOWN_FIELD: &str = "unknown-after-large-jieba-prefix";
const MAX_HTTP_BODY_BYTES: usize = 8 * 1024 * 1024;
const PENDING_HARD_LIMIT_BYTES: u64 = 256 * 1024 * 1024;
const TEXT_STAGE_ROWS_METRIC: &str = "lumen_text_row_stage_rows_total";
const TEXT_STAGE_INPUT_BYTES_METRIC: &str = "lumen_text_row_stage_input_bytes_total";

// The current conservative Jieba bound charges each non-whitespace scalar as
// one possible term with at most eight normalized UTF-8 bytes. A text field
// owns that term model once live and once frozen. At 608 bytes per scalar
// before dirty IDs, input, or staging workspace, this 512,004-scalar fixture
// exceeds the fixed 256 MiB pending limit and must select bounded staging.
const CJK_ALPHABET_LEN: usize = 80;
const DEBRUIJN_ORDER: usize = 3;
const CJK_BODY_SCALARS: usize = CJK_ALPHABET_LEN * CJK_ALPHABET_LEN * CJK_ALPHABET_LEN;
const FALLBACK_MARKER: &str = "北京大學";
const FIXTURE_SCALARS: usize = CJK_BODY_SCALARS + 4;
const CURRENT_JIEBA_COST_LOWER_BOUND_PER_SCALAR: u64 = 608;
const CURRENT_JIEBA_COST_LOWER_BOUND_BYTES: u64 =
    FIXTURE_SCALARS as u64 * CURRENT_JIEBA_COST_LOWER_BOUND_PER_SCALAR;

struct Fixture {
    _dir: tempfile::TempDir,
    engine: Arc<Engine>,
    store: Arc<SegmentRdbStore>,
    server: TestServer,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().expect("Jieba fallback staging fixture directory");
    let root = dir.path().join("segments");
    let store = Arc::new(SegmentRdbStore::new(&root).expect("open fallback segment store"));
    let aof: SharedAof = Arc::new(Mutex::new(
        AofWriter::open(dir.path().join("aof.log")).expect("open fallback AOF"),
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
    let checkpoint_api: Arc<dyn CheckpointSink> = checkpoint;
    let state =
        AppState::with_components(engine.clone(), Arc::new(AuthConfig::open()), sink_writer)
            .with_checkpoint(checkpoint_api);
    let server = TestServer::new(router(state)).expect("Jieba fallback HTTP server");
    Fixture {
        _dir: dir,
        engine,
        store,
        server,
    }
}

/// A deterministic CJK corpus. Every cyclic three-scalar word occurs once,
/// so this is not a repeated-token shortcut. The leading marker carries the
/// small fallback queries whose result and score must survive cold reopen.
fn forced_fallback_text() -> String {
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
            char::from_u32(0x4E00 + u32::try_from(offset).expect("CJK offset")).expect("CJK scalar")
        })
        .collect::<Vec<_>>();
    let mut work = vec![0usize; DEBRUIJN_ORDER + 1];
    let mut cycle = Vec::with_capacity(CJK_BODY_SCALARS);
    visit(
        1,
        1,
        CJK_ALPHABET_LEN,
        DEBRUIJN_ORDER,
        &mut work,
        &mut cycle,
    );
    assert_eq!(
        cycle.len(),
        CJK_BODY_SCALARS,
        "fixture must retain one position per CJK de Bruijn cycle member",
    );

    let mut text = String::with_capacity(FALLBACK_MARKER.len() + cycle.len() * 3);
    text.push_str(FALLBACK_MARKER);
    for index in cycle {
        text.push(alphabet[index]);
    }
    assert_eq!(
        text.chars().count(),
        FIXTURE_SCALARS,
        "fixture scalar count must continue to force the fixed pending-budget path",
    );
    assert!(
        CURRENT_JIEBA_COST_LOWER_BOUND_BYTES > PENDING_HARD_LIMIT_BYTES,
        "fixture must exceed the current 256 MiB Jieba conservative admission lower bound: {CURRENT_JIEBA_COST_LOWER_BOUND_BYTES}",
    );
    text
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
    values[0]
        .parse::<u64>()
        .unwrap_or_else(|_| panic!("{name} must be an unsigned metric: {}", values[0]))
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
        "{phase}: pending total must equal the three public ownership states: {snapshot:?}",
    );
    assert!(
        snapshot.total <= PENDING_HARD_LIMIT_BYTES,
        "{phase}: fallback staging must not exceed the approved 256 MiB pending budget: {snapshot:?}",
    );
}

async fn stage_counters(server: &TestServer) -> (u64, u64) {
    let response = server.get("/metrics").await;
    response.assert_status_ok();
    let metrics = response.text();
    (
        metric_u64(&metrics, TEXT_STAGE_ROWS_METRIC),
        metric_u64(&metrics, TEXT_STAGE_INPUT_BYTES_METRIC),
    )
}

#[derive(Debug, PartialEq, Eq)]
struct MatchResult {
    ids: Vec<String>,
    score_bits: Vec<u64>,
}

async fn fallback_match(server: &TestServer, query: &str, phase: &str) -> MatchResult {
    let response = server
        .post(&format!("/collections/{COLLECTION}/search"))
        .json(&json!({
            "query": { "match": { "field": FIELD, "text": query, "op": "and" } },
            "limit": 8,
            "track_total": true,
        }))
        .await;
    response.assert_status_ok();
    let body: Value = response.json();
    let hits = body["hits"].as_array().expect("Jieba Match hits array");
    assert_eq!(
        body["total"].as_u64(),
        Some(hits.len() as u64),
        "{phase} {query:?}: Match must report every returned hit: {body}",
    );
    let mut hits = hits
        .iter()
        .map(|hit| {
            let id = hit["external_id"]
                .as_str()
                .expect("Jieba Match external ID")
                .to_owned();
            let score = hit["score"].as_f64().unwrap_or_else(|| {
                panic!("{phase} {query:?}: Match must expose a numeric BM25 score: {body}")
            });
            (id, score.to_bits())
        })
        .collect::<Vec<_>>();
    hits.sort_by(|left, right| left.0.cmp(&right.0));
    MatchResult {
        ids: hits.iter().map(|(id, _)| id.clone()).collect(),
        score_bits: hits.iter().map(|(_, score)| *score).collect(),
    }
}

async fn assert_fallback_semantics(server: &TestServer, phase: &str) -> Vec<MatchResult> {
    let mut results = Vec::new();
    for query in ["北京", "京大", "大學", FALLBACK_MARKER] {
        let result = fallback_match(server, query, phase).await;
        assert_eq!(
            result.ids,
            vec![DOCUMENT_ID.to_owned()],
            "{phase} {query:?}: feature-off Jieba fallback bigrams must find the valid staged prefix",
        );
        results.push(result);
    }
    results
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn jieba_fallback_oversized_row_stages_and_recovers_live_and_cold() {
    assert!(
        !cfg!(feature = "jieba"),
        "this target must run with the default feature set so it exercises the CJK-bigram fallback",
    );

    let fixture = fixture();
    fixture
        .server
        .put(&format!("/collections/{COLLECTION}"))
        .json(&json!({
            "fields": { FIELD: { "type": "text", "analyzer": "jieba" } }
        }))
        .await
        .assert_status_ok();
    let before_stage = stage_counters(&fixture.server).await;
    assert_eq!(
        before_stage,
        (0, 0),
        "a fresh Engine must start with no staged Text rows credited to another request",
    );

    let text = forced_fallback_text();
    let request = json!({ "items": [
        { "external_id": DOCUMENT_ID, "field": FIELD, "value": text.clone() },
        { "external_id": DOCUMENT_ID, "field": UNKNOWN_FIELD, "value": "must-refuse" },
    ] });
    let encoded = serde_json::to_vec(&request).expect("serialize oversized fallback request");
    assert!(
        encoded.len() < MAX_HTTP_BODY_BYTES,
        "the staged fallback request must stay below the existing 8 MiB HTTP body limit: {} bytes",
        encoded.len(),
    );

    let response = fixture
        .server
        .post(&format!("/collections/{COLLECTION}/index"))
        .json(&request)
        .await;
    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    let body: Value = response.json();
    assert_eq!(
        body["error"], "unknown_field",
        "the later caller-controlled field must be refused after the valid oversized Jieba prefix: {body}",
    );

    let live = assert_fallback_semantics(&fixture.server, "live").await;
    assert_pending_budget_is_bounded(
        &pending_budget(&fixture.server).await,
        "after fallback apply",
    );

    let checkpoint = fixture.server.post("/admin/checkpoint").await;
    checkpoint.assert_status_ok();
    assert_eq!(
        checkpoint.json::<Value>()["persisted"],
        true,
        "the real checkpoint endpoint must publish the oversized fallback generation",
    );
    let (tokens_after_checkpoint, has_segment) = fixture
        .engine
        .segment_field_probe(COLLECTION, FIELD)
        .expect("probe staged fallback field");
    assert_eq!(
        tokens_after_checkpoint, 0,
        "published staged fallback Text must release its live token driver",
    );
    assert!(
        has_segment,
        "published staged fallback Text must attach a durable segment",
    );
    assert_pending_budget_is_bounded(
        &pending_budget(&fixture.server).await,
        "after fallback checkpoint publication",
    );

    let cold = fixture
        .store
        .load_current_generation()
        .expect("load fallback CURRENT")
        .expect("published fallback generation");
    let cold_server =
        TestServer::new(router(AppState::open(cold.engine))).expect("cold fallback HTTP server");
    assert_eq!(
        assert_fallback_semantics(&cold_server, "cold").await,
        live,
        "cold fallback Match results and BM25 score bits must equal live results",
    );

    // Keep the staging evidence last. A direct legacy fallback path can keep
    // query behavior correct, but it must not satisfy this fixed-budget path.
    let after_stage = stage_counters(&fixture.server).await;
    assert_eq!(
        after_stage.0,
        before_stage.0 + 1,
        "this exact oversized fallback row must create one Engine-scoped durable staged row",
    );
    assert_eq!(
        after_stage.1,
        before_stage.1 + u64::try_from(text.len()).expect("fallback input bytes"),
        "the staged fallback counter must charge the exact caller input bytes",
    );
}
