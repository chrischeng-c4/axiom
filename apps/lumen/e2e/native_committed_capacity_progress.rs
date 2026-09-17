//! Black-box contract for a fitting generic native committed record at a full
//! pending-change budget.
//!
//! The isolated child first publishes a real schema record through `MemWal`,
//! makes a real checkpoint, and then creates checkpointable local
//! `docs:replace` work until at most 1 MiB remains in the Engine-owned pending
//! budget. It publishes one valid 2 MiB generic-CBOR `ReplaceDocs` record
//! straight to the same `MemWal`. It never calls `WriteCoordinator::submit`,
//! starts no `PendingChangeSpill` or `SegmentCheckpointDriver`, and makes no
//! caller checkpoint until the native subscriber has either made progress or
//! the bounded cleanup proves that it was waiting.
//!
//! The target stays below the 32 MiB borrowed mapped-record route. It reaches
//! the normal foreign-delivery pre-decode reservation in the native subscriber.
//! A correct subscriber starts independent capacity maintenance before that
//! reservation waits. It retains the committed source, advances exactly one
//! watermark, writes the exact generic source to its AOF, preserves the prior
//! local row, and leaves state that a normal checkpoint and cold open can read.
//! The observation and child watchdog only bound test cleanup. They are not a
//! product latency promise.
//!
//! # Facets
//!
//! - Behavior: `native_committed_capacity_progress.rs:195-270` creates the
//!   schema and fitting native source. `:353-382` and `:501-579` establish
//!   real checkpointable pressure and require the subscriber to return before
//!   a caller checkpoint. `:624-714` requires the retained source's exact AOF
//!   wire, one applied watermark, an independent maintenance checkpoint, both
//!   live rows, and final cold state. These assertions cover
//!   `apps/lumen/src/coordinator.rs:449-488`,
//!   `apps/lumen/src/coordinator.rs:554-710`, and
//!   `apps/lumen/src/segment_capacity.rs:410-443`.
//! - Security: the changed wait at `apps/lumen/src/coordinator.rs:461-488`
//!   receives a typed `WalDelivery::Deferred` from
//!   `apps/lumen/src/wal.rs:1060-1092`; it does not accept a new raw byte,
//!   path, or authorization input. A future fallback uses the existing
//!   temporary-spill filesystem boundary in
//!   `apps/lumen/src/segment_checkpoint.rs:345-386`. That collision boundary
//!   remains covered by
//!   `apps/lumen/e2e/aof_standalone_replay_capacity_progress.rs:573-580` and
//!   `:749-755`, which pre-creates a predictable path and requires its
//!   sentinel to remain unchanged. Generic frame refusal remains covered by
//!   `apps/lumen/e2e/aof_oversized_committed_replace.rs:794-868`.
//! - Performance: `apps/lumen/docs/indexing.md:264-276` says, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget."
//!   Assertions at `:337-351`, called at `:630`, `:695`, and `:714`, read the
//!   real public pending total and high-water gauges and require both to stay
//!   within that budget. Fixture source, query, and AOF bytes are outside those
//!   gauges. This path has no documented latency budget.
//!
//! # Root negative control
//!
//! Remove the native generic capacity-owner ensure before the Full wait in
//! `apps/lumen/src/coordinator.rs:461-488`. The behavior assertion at
//! `:574-577` must fail after its cleanup checkpoint proves that the valid
//! committed source stayed retained until caller-driven capacity release.
//! Restore changed production bytes by SHA-256 before another gate.
//!
//! Target gate: `cargo test -p lumen --test native_committed_capacity_progress -- --nocapture`.
//! Full declared behavior gate: `cargo test -p lumen`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum_test::TestServer;

use lumen::aof::{AofReader, AofWriter};
use lumen::api::{router, AppState, CheckpointSink};
use lumen::auth::AuthConfig;
use lumen::coordinator::{SharedAof, WriteCoordinator, WriteSink};
use lumen::log_entry::RaftLogEntry;
use lumen::segment_checkpoint::SegmentCheckpointSink;
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::Engine;
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, QueryNode, ReplaceDocItem,
    ReplaceDocsRequest, SearchRequest, TermQuery,
};
use lumen::wal::{MemWal, SharedWal, WalLog, WalRecord};

const COLLECTION: &str = "native-committed-capacity-progress";
const FILLER_FIELD: &str = "filler";
const TARGET_FIELD: &str = "target";
const FILLER_ID: &str = "checkpointable-local-predecessor";
const TARGET_ID: &str = "committed-generic-native-target";
const CREATE_SEQUENCE: u64 = 1;
const TARGET_SEQUENCE: u64 = 2;
const PENDING_HARD_LIMIT_BYTES: u64 = 256 * 1024 * 1024;
const MAPPED_ROUTE_MINIMUM_BYTES: usize = 32 * 1024 * 1024;
const TARGET_TOKEN_BYTES: usize = 2 * 1024 * 1024;
const FILLER_VALUE_BYTES: usize = 64 * 1024;
const INITIAL_MAX_FREE_BYTES: u64 = 1024 * 1024;
const MAX_FILLER_WRITES: usize = 6_000;
const PUBLISH_WATCHDOG: Duration = Duration::from_secs(15);
const PROGRESS_OBSERVATION: Duration = Duration::from_secs(5);
const CLEANUP_WAIT: Duration = Duration::from_secs(30);
const CHECKPOINT_WATCHDOG: Duration = Duration::from_secs(60);
const CHILD_WATCHDOG: Duration = Duration::from_secs(120);
const POLL_INTERVAL: Duration = Duration::from_millis(25);
const CHILD_MODE_ENV: &str = "LUMEN_NATIVE_CAPACITY_PROGRESS_CHILD";
const CHILD_ROOT_ENV: &str = "LUMEN_NATIVE_CAPACITY_PROGRESS_ROOT";
const CHILD_HANDSHAKE_ENV: &str = "LUMEN_NATIVE_CAPACITY_PROGRESS_HANDSHAKE";
const CHILD_CASE: &str = "native-generic-predecode-capacity";
const TEST_NAME: &str =
    "native_committed_generic_apply_makes_capacity_progress_before_any_caller_checkpoint";

struct Fixture {
    engine: Arc<Engine>,
    server: TestServer,
    store: Arc<SegmentRdbStore>,
    aof: SharedAof,
    aof_path: PathBuf,
    wal: Arc<MemWal>,
    writer: Arc<WriteCoordinator>,
}

/// Reap a child before its parent-owned temporary directory disappears.
struct ChildCleanup(Option<Child>);

impl Drop for ChildCleanup {
    fn drop(&mut self) {
        let Some(child) = self.0.as_mut() else {
            return;
        };
        if !matches!(child.try_wait(), Ok(Some(_))) {
            let _ = child.kill();
        }
        let _ = child.wait();
    }
}

fn fixture(root: &Path) -> Fixture {
    let aof_path = root.join("native-capacity-progress.aof");
    let aof: SharedAof = Arc::new(Mutex::new(
        AofWriter::open(&aof_path).expect("open native capacity-progress AOF"),
    ));
    let engine = Arc::new(Engine::new());
    let store = Arc::new(
        SegmentRdbStore::new(root.join("segments"))
            .expect("open native capacity-progress segment store"),
    );
    let wal = Arc::new(MemWal::new());
    let shared_wal: SharedWal = wal.clone();
    let writer = WriteCoordinator::start_from_with_aof(shared_wal, engine.clone(), 0, aof.clone());
    let sink_writer: Arc<dyn WriteSink> = writer.clone();
    let checkpoint: Arc<dyn CheckpointSink> = Arc::new(SegmentCheckpointSink {
        engine: engine.clone(),
        store: store.clone(),
        writer: sink_writer.clone(),
        aof: Some(aof.clone()),
    });
    let state =
        AppState::with_components(engine.clone(), Arc::new(AuthConfig::open()), sink_writer)
            .with_checkpoint(checkpoint);
    let server = TestServer::new(router(state)).expect("open native capacity-progress HTTP server");

    Fixture {
        engine,
        server,
        store,
        aof,
        aof_path,
        wal,
        writer,
    }
}

fn keyword_spec() -> FieldSpec {
    FieldSpec {
        field_type: FieldType::Keyword,
        analyzer: None,
        multi: None,
        dim: None,
        metric: None,
        backend: None,
        quantize: None,
    }
}

fn schema() -> CreateCollectionRequest {
    CreateCollectionRequest {
        fields: BTreeMap::from([
            (FILLER_FIELD.to_owned(), keyword_spec()),
            (TARGET_FIELD.to_owned(), keyword_spec()),
        ]),
    }
}

fn create_record() -> WalRecord {
    WalRecord::new(RaftLogEntry::CreateCollection {
        collection_id: COLLECTION.to_owned(),
        req: schema(),
    })
}

fn marked_ascii(prefix: &str, ordinal: usize, bytes: usize) -> String {
    let head = format!("{prefix}-{ordinal:04}:");
    let tail = format!(":end-{prefix}-{ordinal:04}");
    assert!(
        head.len() + tail.len() < bytes,
        "fixture markers must leave a real valid Keyword value",
    );
    let mut value = vec![b'x'; bytes];
    value[..head.len()].copy_from_slice(head.as_bytes());
    value[bytes - tail.len()..].copy_from_slice(tail.as_bytes());
    String::from_utf8(value).expect("native capacity fixture Keyword must be valid ASCII")
}

fn filler_value(ordinal: usize) -> String {
    marked_ascii("checkpointable-local-filler", ordinal, FILLER_VALUE_BYTES)
}

fn target_value() -> String {
    marked_ascii("committed-generic-native-token", 0, TARGET_TOKEN_BYTES)
}

fn local_filler_request(ordinal: usize) -> ReplaceDocsRequest {
    ReplaceDocsRequest {
        docs: vec![ReplaceDocItem {
            external_id: FILLER_ID.to_owned(),
            version: Some(ordinal as u64 + 1),
            fields: BTreeMap::from([(
                FILLER_FIELD.to_owned(),
                FieldValue::String(filler_value(ordinal)),
            )]),
        }],
    }
}

fn target_record() -> WalRecord {
    WalRecord::new(RaftLogEntry::ReplaceDocs {
        collection_id: COLLECTION.to_owned(),
        req: ReplaceDocsRequest {
            docs: vec![ReplaceDocItem {
                external_id: TARGET_ID.to_owned(),
                version: Some(100),
                fields: BTreeMap::from([(
                    TARGET_FIELD.to_owned(),
                    FieldValue::String(target_value()),
                )]),
            }],
        },
    })
}

fn assert_generic_target_source(record: &WalRecord) -> (usize, u32) {
    let wire = record
        .encode()
        .expect("encode valid fitting committed native generic source");
    assert!(
        !wire.starts_with(b"LWAL"),
        "target must use its real generic-CBOR native WAL form, not the fast Index wire",
    );
    assert!(
        wire.len() >= TARGET_TOKEN_BYTES,
        "target must retain its real 2 MiB Keyword token: wire_bytes={}",
        wire.len(),
    );
    assert!(
        wire.len() < MAPPED_ROUTE_MINIMUM_BYTES,
        "target must stay below the 32 MiB borrowed mapped-record route: wire_bytes={}",
        wire.len(),
    );
    (wire.len(), crc32fast::hash(&wire))
}

fn search_request(query: QueryNode) -> SearchRequest {
    SearchRequest {
        query,
        limit: 16,
        offset: 0,
        cursor: None,
        routing_key: None,
        sort: None,
        track_total: true,
        collapse: None,
    }
}

fn term_ids(engine: &Engine, field: &str, value: String, phase: &str) -> BTreeSet<String> {
    engine
        .search(
            COLLECTION,
            search_request(QueryNode::Term(TermQuery {
                field: field.to_owned(),
                value: FieldValue::String(value),
            })),
        )
        .unwrap_or_else(|error| panic!("{phase}: public exact Keyword query failed: {error:#}"))
        .hits
        .into_iter()
        .map(|hit| hit.external_id)
        .collect()
}

fn singleton(id: &str) -> BTreeSet<String> {
    BTreeSet::from([id.to_owned()])
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
        "public /metrics must publish exactly one {name} sample: {metrics}",
    );
    values[0]
        .parse::<u64>()
        .unwrap_or_else(|_| panic!("{name} must be an unsigned byte value: {}", values[0]))
}

async fn public_metric(server: &TestServer, name: &str) -> u64 {
    let response = server.get("/metrics").await;
    response.assert_status_ok();
    metric_u64(&response.text(), name)
}

fn pending_total_from_engine(engine: &Engine) -> u64 {
    metric_u64(
        &engine.metrics().render(),
        "lumen_pending_change_total_bytes",
    )
}

async fn assert_public_pending_budget(server: &TestServer, phase: &str) {
    let response = server.get("/metrics").await;
    response.assert_status_ok();
    let metrics = response.text();
    let total = metric_u64(&metrics, "lumen_pending_change_total_bytes");
    let high_water = metric_u64(&metrics, "lumen_pending_change_high_water_bytes");
    assert!(
        total <= PENDING_HARD_LIMIT_BYTES,
        "{phase}: public pending total must stay within the documented 256 MiB budget: total={total}",
    );
    assert!(
        high_water <= PENDING_HARD_LIMIT_BYTES,
        "{phase}: public pending high water must stay within the documented 256 MiB budget: high_water={high_water}",
    );
}

fn fill_checkpointable_local_work(engine: &Arc<Engine>) -> usize {
    for ordinal in 0..MAX_FILLER_WRITES {
        let total = pending_total_from_engine(engine);
        assert!(
            total <= PENDING_HARD_LIMIT_BYTES,
            "direct local setup must never exceed the documented pending budget: total={total}",
        );
        let free = PENDING_HARD_LIMIT_BYTES - total;
        if free <= INITIAL_MAX_FREE_BYTES {
            return ordinal
                .checked_sub(1)
                .expect("schema-only base must not start within the selected free-space interval");
        }
        let response = engine
            .replace_docs(COLLECTION, local_filler_request(ordinal))
            .unwrap_or_else(|error| {
                panic!(
                    "direct local setup must create checkpointable work before capacity becomes full: {error:#}",
                )
            });
        assert_eq!(
            response.results.len(),
            1,
            "each direct local setup replacement must produce one public result",
        );
    }
    panic!("setup did not leave at most 1 MiB free after {MAX_FILLER_WRITES} real local records",);
}

fn assert_live_state(engine: &Engine, filler_ordinal: usize, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("native capacity collection stats")
            .documents_indexed,
        2,
        "{phase}: one checkpointable local predecessor and one committed target must be live",
    );
    assert_eq!(
        term_ids(engine, FILLER_FIELD, filler_value(filler_ordinal), phase),
        singleton(FILLER_ID),
        "{phase}: the checkpointable local predecessor must survive capacity maintenance",
    );
    if filler_ordinal != 0 {
        assert_eq!(
            term_ids(engine, FILLER_FIELD, filler_value(0), phase),
            BTreeSet::new(),
            "{phase}: superseded local filler values must not revive",
        );
    }
    assert_eq!(
        term_ids(engine, TARGET_FIELD, target_value(), phase),
        singleton(TARGET_ID),
        "{phase}: the whole valid committed generic native Keyword must be query-visible",
    );
}

async fn publish_committed(wal: &MemWal, record: WalRecord, phase: &str) -> u64 {
    let published = tokio::time::timeout(PUBLISH_WATCHDOG, wal.publish(record)).await;
    assert!(
        published.is_ok(),
        "{phase}: native MemWal publish must return a committed sequence before test cleanup",
    );
    let published = published.expect("behavior assertion above checked native publish watchdog");
    let diagnostic = published
        .as_ref()
        .err()
        .map(|error| format!("{error:#}"))
        .unwrap_or_default();
    assert!(
        published.is_ok(),
        "{phase}: a valid native committed record must not receive an error acknowledgement: {diagnostic}",
    );
    published.expect("behavior assertion above checked native committed acknowledgement")
}

async fn wait_for_committed_apply(fixture: &Fixture, sequence: u64, phase: &str) {
    let applied = tokio::time::timeout(CLEANUP_WAIT, async {
        loop {
            if fixture.writer.applied_seq() >= sequence {
                return;
            }
            if fixture.writer.is_restart_required() {
                return;
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    })
    .await;
    let wal_head = fixture
        .wal
        .latest_seq()
        .await
        .expect("read native MemWal head after committed publication");
    assert!(
        applied.is_ok() && fixture.writer.applied_seq() >= sequence,
        "{phase}: valid retained native committed source must reach sequence {sequence} after cleanup; applied_sequence={}, wal_head={wal_head}, restart_required={}",
        fixture.writer.applied_seq(),
        fixture.writer.is_restart_required(),
    );
}

async fn checkpoint_now(server: &TestServer) {
    let response = tokio::time::timeout(CHECKPOINT_WATCHDOG, async {
        server.post("/admin/checkpoint").await
    })
    .await;
    assert!(
        response.is_ok(),
        "the real on-demand checkpoint must finish before test cleanup",
    );
    let response = response.expect("behavior assertion above checked checkpoint watchdog");
    response.assert_status_ok();
    assert_eq!(
        response.json::<serde_json::Value>()["persisted"],
        true,
        "the real on-demand checkpoint must publish durable state",
    );
}

fn assert_aof_target_wire(
    fixture: &Fixture,
    expected_bytes: usize,
    expected_crc: u32,
    phase: &str,
) {
    fixture
        .aof
        .lock()
        .expect("native capacity AOF lock")
        .sync_strict()
        .unwrap_or_else(|error| panic!("{phase}: strict-sync native committed AOF: {error:#}"));
    let mut frames = Vec::new();
    let maximum = AofReader::replay(&fixture.aof_path, CREATE_SEQUENCE, |sequence, record| {
        let wire = record
            .encode()
            .expect("re-encode persisted native generic AOF record");
        frames.push((sequence, wire.len(), crc32fast::hash(&wire)));
    })
    .unwrap_or_else(|error| {
        panic!("{phase}: read persisted native committed AOF source: {error:#}")
    });
    assert_eq!(
        frames,
        vec![(TARGET_SEQUENCE, expected_bytes, expected_crc)],
        "{phase}: the AOF must retain exactly the committed generic source wire after the schema checkpoint",
    );
    assert_eq!(
        maximum, TARGET_SEQUENCE,
        "{phase}: the retained AOF source must keep the exact native committed watermark",
    );
}

async fn require_progress_before_caller_checkpoint(
    fixture: &Fixture,
    filler_ordinal: usize,
    source_bytes: usize,
    source_crc: u32,
) {
    let progress = tokio::time::timeout(PROGRESS_OBSERVATION, async {
        loop {
            if fixture.writer.applied_seq() >= TARGET_SEQUENCE {
                return true;
            }
            if fixture.writer.is_restart_required() {
                return false;
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    })
    .await;
    match progress {
        Ok(true) => return,
        Ok(false) => panic!(
            "valid fitting native committed source must not mark the coordinator restart-required while capacity maintenance is pending",
        ),
        Err(_) => {}
    }

    assert_eq!(
        fixture.writer.applied_seq(),
        CREATE_SEQUENCE,
        "the retained native source must remain unacknowledged while capacity maintenance is absent",
    );
    assert_eq!(
        fixture
            .wal
            .latest_seq()
            .await
            .expect("read native MemWal head while committed source waits"),
        TARGET_SEQUENCE,
        "the native WAL must retain the committed generic source while the subscriber waits",
    );
    assert!(
        !fixture.writer.is_restart_required(),
        "waiting for capacity must not turn a valid retained native source into restart-required",
    );

    // This checkpoint is cleanup after the behavior observation only. It must
    // never be needed on the correct path, but it lets the retained source
    // resume so the test proves exact source retention before it reports red.
    let cleanup_sequence = fixture
        .store
        .save_with_sequence(&fixture.engine, CREATE_SEQUENCE)
        .expect("caller cleanup checkpoint must release retained local work");
    assert_eq!(
        cleanup_sequence, CREATE_SEQUENCE,
        "cleanup checkpoint must retain the schema watermark while it only releases local setup work",
    );
    wait_for_committed_apply(
        fixture,
        TARGET_SEQUENCE,
        "valid retained native committed source after cleanup checkpoint",
    )
    .await;
    assert_aof_target_wire(
        fixture,
        source_bytes,
        source_crc,
        "cleanup-resumed native committed source",
    );
    assert_live_state(
        &fixture.engine,
        filler_ordinal,
        "cleanup-resumed native committed source",
    );
    assert!(
        false,
        "valid fitting generic native committed apply waited for a caller checkpoint instead of starting independent capacity maintenance; cleanup checkpoint released the retained source",
    );
    unreachable!("the behavior assertion above always panics");
}

fn child_paths() -> Option<(PathBuf, PathBuf)> {
    if std::env::var(CHILD_MODE_ENV).ok().as_deref() != Some(CHILD_CASE) {
        return None;
    }
    let root = std::env::var_os(CHILD_ROOT_ENV)
        .map(PathBuf::from)
        .expect("native capacity child needs a parent-owned durable root");
    let handshake = std::env::var_os(CHILD_HANDSHAKE_ENV)
        .map(PathBuf::from)
        .expect("native capacity child needs a handshake path");
    Some((root, handshake))
}

async fn run_child(root: PathBuf, handshake: PathBuf) {
    fs::write(&handshake, CHILD_CASE).expect("record exact native capacity child entry");
    let fixture = fixture(&root);

    let create_sequence =
        publish_committed(&fixture.wal, create_record(), "collection schema").await;
    assert_eq!(
        create_sequence, CREATE_SEQUENCE,
        "the schema must be the first native committed MemWal record",
    );
    wait_for_committed_apply(&fixture, create_sequence, "collection schema").await;
    checkpoint_now(&fixture.server).await;
    let schema_cold = fixture
        .store
        .load_current_generation()
        .expect("cold-open schema-only native capacity checkpoint")
        .expect("schema-only native capacity checkpoint must publish CURRENT");
    assert_eq!(
        schema_cold.sequence, CREATE_SEQUENCE,
        "schema checkpoint must retain the initial native committed watermark",
    );
    assert_eq!(
        schema_cold
            .engine
            .stats(COLLECTION)
            .expect("schema-only native capacity collection stats")
            .documents_indexed,
        0,
        "schema checkpoint must not retain a hidden document",
    );

    let filler_ordinal = fill_checkpointable_local_work(&fixture.engine);
    let free_after_setup = PENDING_HARD_LIMIT_BYTES - pending_total_from_engine(&fixture.engine);
    assert!(
        free_after_setup <= INITIAL_MAX_FREE_BYTES,
        "public pending accounting must establish the selected pre-decode pressure: free={free_after_setup}",
    );
    assert_public_pending_budget(&fixture.server, "native direct local capacity setup").await;
    let checkpoints_before =
        public_metric(&fixture.server, "lumen_segment_checkpoint_completed_total").await;

    let target = target_record();
    let (source_bytes, source_crc) = assert_generic_target_source(&target);
    let target_sequence = publish_committed(
        &fixture.wal,
        target,
        "fitting generic native committed target",
    )
    .await;
    assert_eq!(
        target_sequence, TARGET_SEQUENCE,
        "the fitting generic target must become the next committed native MemWal record",
    );
    require_progress_before_caller_checkpoint(&fixture, filler_ordinal, source_bytes, source_crc)
        .await;

    assert_eq!(
        fixture.writer.applied_seq(),
        TARGET_SEQUENCE,
        "one completed fitting foreign source must advance the native watermark exactly once",
    );
    assert_eq!(
        fixture
            .wal
            .latest_seq()
            .await
            .expect("read native MemWal after capacity progress"),
        TARGET_SEQUENCE,
        "the native WAL head must remain aligned with the applied committed source",
    );
    assert!(
        !fixture.writer.is_restart_required(),
        "independent maintenance for a valid retained native source must not leave the coordinator restart-required",
    );
    assert!(
        public_metric(
            &fixture.server,
            "lumen_segment_checkpoint_completed_total",
        )
        .await
            > checkpoints_before,
        "native capacity progress must complete a real maintenance checkpoint before generic apply returns",
    );
    assert_aof_target_wire(
        &fixture,
        source_bytes,
        source_crc,
        "live native capacity progress",
    );
    assert_live_state(
        &fixture.engine,
        filler_ordinal,
        "live native capacity progress",
    );
    assert_public_pending_budget(&fixture.server, "live native capacity progress").await;

    // This ordinary public checkpoint comes only after foreign delivery has
    // completed. It proves configured durability, not capacity progress.
    checkpoint_now(&fixture.server).await;
    let cold = fixture
        .store
        .load_current_generation()
        .expect("cold-open final native capacity checkpoint")
        .expect("final native capacity checkpoint must publish CURRENT");
    assert_eq!(
        cold.sequence, TARGET_SEQUENCE,
        "cold native checkpoint must retain the completed committed watermark",
    );
    assert_live_state(
        &cold.engine,
        filler_ordinal,
        "cold native capacity progress",
    );
    assert_public_pending_budget(&fixture.server, "native capacity final checkpoint").await;
}

async fn run_isolated_child(root: &Path) {
    let child_workspace =
        tempfile::tempdir().expect("create parent-owned native capacity child workspace");
    let child_tmp = child_workspace.path().join("tmp");
    fs::create_dir(&child_tmp).expect("create parent-owned native capacity TMPDIR");
    let handshake = child_workspace.path().join("entered-case");
    let stdout_path = child_workspace.path().join("child.stdout");
    let stderr_path = child_workspace.path().join("child.stderr");
    let executable =
        std::env::current_exe().expect("locate current native capacity e2e executable");
    let child = Command::new(executable)
        .env(CHILD_MODE_ENV, CHILD_CASE)
        .env(CHILD_ROOT_ENV, root)
        .env(CHILD_HANDSHAKE_ENV, &handshake)
        .env("TMPDIR", &child_tmp)
        .env("TEMP", &child_tmp)
        .env("TMP", &child_tmp)
        .arg(TEST_NAME)
        .arg("--exact")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .stdout(Stdio::from(
            File::create(&stdout_path).expect("create isolated native capacity child stdout"),
        ))
        .stderr(Stdio::from(
            File::create(&stderr_path).expect("create isolated native capacity child stderr"),
        ))
        .spawn()
        .expect("spawn isolated native committed capacity child");
    let mut child = ChildCleanup(Some(child));
    let deadline = Instant::now() + CHILD_WATCHDOG;
    loop {
        match child
            .0
            .as_mut()
            .expect("child remains owned until it exits")
            .try_wait()
        {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => tokio::time::sleep(POLL_INTERVAL).await,
            Ok(None) => {
                let mut raw = child.0.take().expect("timed-out child remains owned");
                let _ = raw.kill();
                let status = raw.wait().expect("reap killed native capacity child");
                let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
                let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
                panic!(
                    "valid fitting committed native child did not finish before cleanup watchdog; status={status}; stdout={stdout}; stderr={stderr}",
                );
            }
            Err(error) => panic!("poll isolated committed native capacity child: {error}"),
        }
    }
    let status = child
        .0
        .take()
        .expect("exited child remains owned")
        .wait()
        .expect("reap exited committed native capacity child");
    let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
    let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
    let entered = fs::read_to_string(&handshake).unwrap_or_else(|error| {
        panic!(
            "isolated committed native capacity child never entered its test body: {error}; stdout={stdout}; stderr={stderr}",
        )
    });
    assert_eq!(
        entered, CHILD_CASE,
        "isolated child must run the intended committed native capacity case",
    );
    assert!(
        status.success(),
        "isolated committed native capacity child failed: status={status}; stdout={stdout}; stderr={stderr}",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn native_committed_generic_apply_makes_capacity_progress_before_any_caller_checkpoint() {
    if let Some((root, handshake)) = child_paths() {
        run_child(root, handshake).await;
        return;
    }

    let root = tempfile::tempdir().expect("create run-scoped committed native capacity root");
    run_isolated_child(root.path()).await;
}
