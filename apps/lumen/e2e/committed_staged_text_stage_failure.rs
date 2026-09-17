//! Black-box contract for a committed staged Ngram Text record whose private
//! stage directory cannot be created.
//!
//! The child publishes and checkpoints a valid Text schema. It then publishes
//! one valid 2 MiB Ngram Text value through the public WriteCoordinator and
//! real MemWal. A public WalLog wrapper pauses the coordinator after the log
//! assigns sequence 2. The child replaces only its parent-owned TMPDIR
//! directory with a regular file, then releases delivery. StagedTextRow then
//! gets a real create-directory I/O error after publication and before rows.
//!
//! The source string is U+20000..U+A0000. Each scalar is valid UTF-8, is not
//! whitespace, and lowercases to itself. Strictly increasing adjacent pairs and
//! triples are unique. The fixture proves the active Text lower bound from the
//! existing charge formula without allocating a token set: 1,048,573 distinct
//! terms require 327,156,816 bytes for two String owners, the ordered
//! dictionary, and postings alone. This exceeds 256 MiB. The raw source and
//! actual fast WAL record stay below 8 MiB. The record must select staged Text
//! preparation instead of ordinary owned Text apply.
//!
//! This is a native MemWal retention contract. It does not claim AOF
//! persistence. The still-unresolved delivery retains the WAL source and a
//! fresh Engine replays it only after the child restores its private temporary
//! directory. The parent-owned child workspace makes a fallback worker safe to
//! kill and reap. The watchdogs bound test cleanup only. They make no product
//! latency promise.
//!
//! # Facets
//!
//! - Behavior: committed_staged_text_stage_failure.rs:620-679 publishes the
//!   schema, checkpoints its prefix, commits the valid staged Text record, and
//!   makes the real stage-directory I/O failure only after the committed
//!   sequence exists. :681-748 requires a failed local result, old watermark,
//!   no Text row, no new CURRENT generation, retained source bytes, and valid
//!   replay after restoring the private path. It covers
//!   apps/lumen/src/coordinator.rs:80-120 and :599-605,
//!   apps/lumen/src/storage/record_admission.rs:546-621, and
//!   apps/lumen/src/storage/staged_text_row.rs:174-206.
//! - Security: :665-679 gives the environment-derived private stage root a
//!   regular file rather than a directory after the record is committed. The
//!   closed assertions at :681-740 require no acknowledgement, state,
//!   watermark, or CURRENT publication from that unusable filesystem input;
//!   :522-547 requires the real retained WAL bytes to remain intact. It covers
//!   the process I/O boundary at apps/lumen/src/storage/staged_text_row.rs:174-206.
//! - Performance: apps/lumen/docs/indexing.md:264-276 says, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget."
//!   :435-454, called after failed delivery at :734-739 and after retained-source
//!   recovery at :609-617, reads real public metrics total and high-water gauges
//!   and requires both to stay within that budget. :322-361 proves this
//!   fixture normal active Text representation exceeds that budget without an
//!   owned token dictionary.
//!
//! # Root negative control
//!
//! After the repair exists, restore the legacy PreparedLocalRecord Fallback
//! branch at apps/lumen/src/coordinator.rs:599-605. The behavior assertion at
//! :690 must fail because the stage-directory I/O error becomes a successful,
//! uncharged normal apply. Restore the corrected production SHA before another
//! gate. Do not alter the input size or expectation to manufacture this red.
//!
//! Gate: cargo test -p lumen --test committed_staged_text_stage_failure -- --nocapture.
//! Full declared behavior gate: cargo test -p lumen.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use axum_test::TestServer;
use futures::StreamExt;
use tokio::sync::Notify;

use lumen::api::{router, AppState};
use lumen::auth::AuthConfig;
use lumen::coordinator::{WriteCoordinator, WriteSink};
use lumen::log_entry::RaftLogEntry;
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::{ApplyOutcome, Engine};
use lumen::types::{
    Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    MatchOp, MatchQuery, QueryNode, SearchRequest,
};
use lumen::wal::{
    MemWal, SharedWal, WalAdmissionStream, WalLog, WalRecord, WalSourceRelease, WalStream,
};

const COLLECTION: &str = "committed-staged-text-stage-failure";
const FIELD: &str = "body";
const EXTERNAL_ID: &str = "committed-staged-text-row";
const FIRST_SCALAR: u32 = 0x20_000;
const END_SCALAR: u32 = 0xA0_000;
const SCALAR_COUNT: usize = (END_SCALAR - FIRST_SCALAR) as usize;
const RAW_TEXT_BYTES: usize = SCALAR_COUNT * 4;
const BIGRAM_COUNT: usize = SCALAR_COUNT - 1;
const TRIGRAM_COUNT: usize = SCALAR_COUNT - 2;
const DISTINCT_TERMS: usize = BIGRAM_COUNT + TRIGRAM_COUNT;
const DISTINCT_TERM_BYTES: usize = BIGRAM_COUNT * 8 + TRIGRAM_COUNT * 12;
const TEXT_STRING_OWNERS_LOWER_BOUND: usize = 4 * DISTINCT_TERM_BYTES + 64 * DISTINCT_TERMS;
const TEXT_ORDERED_DICTIONARY_LOWER_BOUND: usize = 1024 + 192 * DISTINCT_TERMS;
const TEXT_POSTINGS_LOWER_BOUND: usize = 1024 + 16 * DISTINCT_TERMS;
const ACTIVE_TEXT_LOWER_BOUND: usize = TEXT_STRING_OWNERS_LOWER_BOUND
    + TEXT_ORDERED_DICTIONARY_LOWER_BOUND
    + TEXT_POSTINGS_LOWER_BOUND;
const PENDING_HARD_LIMIT_BYTES: u64 = 256 * 1024 * 1024;
const RAW_HTTP_LIMIT_BYTES: usize = 8 * 1024 * 1024;
const CREATE_SEQUENCE: u64 = 1;
const INDEX_SEQUENCE: u64 = 2;
const PUBLISH_WATCHDOG: Duration = Duration::from_secs(20);
const GATE_WATCHDOG: Duration = Duration::from_secs(20);
const FAILURE_WATCHDOG: Duration = Duration::from_secs(90);
const RECOVERY_WATCHDOG: Duration = Duration::from_secs(90);
const CHILD_WATCHDOG: Duration = Duration::from_secs(240);
const POLL_INTERVAL: Duration = Duration::from_millis(20);
const CHILD_MODE_ENV: &str = "LUMEN_COMMITTED_STAGED_TEXT_STAGE_FAILURE_CHILD";
const CHILD_ROOT_ENV: &str = "LUMEN_COMMITTED_STAGED_TEXT_STAGE_FAILURE_ROOT";
const CHILD_HANDSHAKE_ENV: &str = "LUMEN_COMMITTED_STAGED_TEXT_STAGE_FAILURE_HANDSHAKE";
const CHILD_CASE: &str = "stage-directory-io-failure";
const TEST_NAME: &str =
    "committed_staged_text_stage_directory_failure_retains_source_and_never_falls_back";

struct DeliveryGate {
    held_sequence: AtomicU64,
    arrived: AtomicBool,
    released: AtomicBool,
    arrived_notify: Notify,
    release_notify: Notify,
}

impl DeliveryGate {
    fn new() -> Self {
        Self {
            held_sequence: AtomicU64::new(0),
            arrived: AtomicBool::new(false),
            released: AtomicBool::new(false),
            arrived_notify: Notify::new(),
            release_notify: Notify::new(),
        }
    }

    fn hold(&self, sequence: u64) {
        assert_eq!(
            self.held_sequence.swap(sequence, Ordering::AcqRel),
            0,
            "fixture must arm one committed WAL delivery exactly once",
        );
        self.arrived.store(false, Ordering::Release);
        self.released.store(false, Ordering::Release);
    }

    fn holds(&self, sequence: u64) -> bool {
        self.held_sequence.load(Ordering::Acquire) == sequence
    }

    async fn wait_until_arrived(&self) {
        let arrived = tokio::time::timeout(GATE_WATCHDOG, async {
            loop {
                let notified = self.arrived_notify.notified();
                if self.arrived.load(Ordering::Acquire) {
                    return;
                }
                notified.await;
            }
        })
        .await;
        assert!(
            arrived.is_ok(),
            "the coordinator must receive the committed Text sequence before TMPDIR changes",
        );
    }

    async fn wait_until_released(&self) {
        loop {
            let notified = self.release_notify.notified();
            if self.released.load(Ordering::Acquire) {
                return;
            }
            notified.await;
        }
    }

    fn release(&self) {
        assert!(
            self.arrived.load(Ordering::Acquire),
            "fixture must not release before coordinator owns the committed sequence",
        );
        self.released.store(true, Ordering::Release);
        self.release_notify.notify_waiters();
    }
}

struct GateWal {
    inner: Arc<MemWal>,
    gate: Arc<DeliveryGate>,
}

impl GateWal {
    fn new() -> Self {
        Self {
            inner: Arc::new(MemWal::new()),
            gate: Arc::new(DeliveryGate::new()),
        }
    }
}

#[async_trait]
impl WalLog for GateWal {
    async fn publish(&self, record: WalRecord) -> anyhow::Result<u64> {
        self.inner.publish(record).await
    }

    async fn subscribe(&self, from_seq: u64) -> anyhow::Result<WalStream> {
        self.inner.subscribe(from_seq).await
    }

    async fn subscribe_admitted(&self, from_seq: u64) -> anyhow::Result<WalAdmissionStream> {
        let stream = self.inner.subscribe_admitted(from_seq).await?;
        let gate = self.gate.clone();
        Ok(Box::pin(futures::stream::unfold(
            (stream, gate),
            |(mut stream, gate)| async move {
                let item = stream.next().await?;
                if let Ok((sequence, _)) = &item {
                    if gate.holds(*sequence) {
                        gate.arrived.store(true, Ordering::Release);
                        gate.arrived_notify.notify_waiters();
                        gate.wait_until_released().await;
                    }
                }
                Some((item, (stream, gate)))
            },
        )))
    }

    async fn stage_source(&self, sequence: u64) -> anyhow::Result<Option<WalSourceRelease>> {
        self.inner.stage_source(sequence).await
    }

    async fn latest_seq(&self) -> anyhow::Result<u64> {
        self.inner.latest_seq().await
    }
}

struct MetricsSink {
    applied_sequence: u64,
}

#[async_trait]
impl WriteSink for MetricsSink {
    async fn submit(&self, _entry: RaftLogEntry) -> anyhow::Result<ApplyOutcome> {
        anyhow::bail!("the metrics-only sink must not accept a mutation")
    }

    fn applied_seq(&self) -> u64 {
        self.applied_sequence
    }
}

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

struct Fixture {
    engine: Arc<Engine>,
    store: Arc<SegmentRdbStore>,
    wal: Arc<GateWal>,
    writer: Arc<WriteCoordinator>,
}

fn fixture(root: &Path) -> Fixture {
    let engine = Arc::new(Engine::new());
    let store = Arc::new(
        SegmentRdbStore::new(root.join("segments"))
            .expect("open committed staged Text segment store"),
    );
    let wal = Arc::new(GateWal::new());
    let shared_wal: SharedWal = wal.clone();
    let writer = WriteCoordinator::start(shared_wal, engine.clone());
    Fixture {
        engine,
        store,
        wal,
        writer,
    }
}

fn text_schema() -> CreateCollectionRequest {
    CreateCollectionRequest {
        fields: BTreeMap::from([(
            FIELD.to_owned(),
            FieldSpec {
                field_type: FieldType::Text,
                analyzer: Some(Analyzer::Ngram),
                multi: None,
                dim: None,
                metric: None,
                backend: None,
                quantize: None,
            },
        )]),
    }
}

fn create_entry() -> RaftLogEntry {
    RaftLogEntry::CreateCollection {
        collection_id: COLLECTION.to_owned(),
        req: text_schema(),
    }
}

fn staged_text_value() -> String {
    assert_eq!(SCALAR_COUNT, 524_288, "fixture scalar count");
    assert_eq!(RAW_TEXT_BYTES, 2 * 1024 * 1024, "fixture raw source bytes");
    assert_eq!(DISTINCT_TERMS, 1_048_573, "fixture distinct Ngram windows");
    assert_eq!(
        ACTIVE_TEXT_LOWER_BOUND, 327_156_816,
        "fixture lower-bound arithmetic must stay tied to the cost formula",
    );
    assert!(
        ACTIVE_TEXT_LOWER_BOUND > PENDING_HARD_LIMIT_BYTES as usize,
        "active Text lower bound must exceed the 256 MiB budget: {ACTIVE_TEXT_LOWER_BOUND}",
    );

    let mut value = String::with_capacity(RAW_TEXT_BYTES);
    for scalar in FIRST_SCALAR..END_SCALAR {
        let character = char::from_u32(scalar)
            .unwrap_or_else(|| panic!("fixture scalar U+{scalar:04X} must be valid Unicode"));
        assert!(
            !character.is_whitespace(),
            "fixture scalar U+{scalar:04X} must not disappear from Ngram normalization",
        );
        let mut lowered = character.to_lowercase();
        assert_eq!(
            lowered.next(),
            Some(character),
            "fixture scalar U+{scalar:04X} must lowercase to itself",
        );
        assert_eq!(
            lowered.next(),
            None,
            "fixture scalar U+{scalar:04X} must yield one lowercase scalar",
        );
        value.push(character);
    }
    assert_eq!(
        value.len(),
        RAW_TEXT_BYTES,
        "every fixture scalar must use four UTF-8 bytes",
    );
    value
}

fn target_entry() -> RaftLogEntry {
    RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items: vec![IndexItem {
                external_id: EXTERNAL_ID.to_owned(),
                field: FIELD.to_owned(),
                value: FieldValue::String(staged_text_value()),
                version: Some(7),
            }],
            request_id: Some("committed-staged-text-stage-failure".to_owned()),
        },
    }
}

fn target_query() -> String {
    let mut query = String::with_capacity(8);
    for scalar in FIRST_SCALAR..FIRST_SCALAR + 2 {
        query.push(
            char::from_u32(scalar)
                .unwrap_or_else(|| panic!("query scalar U+{scalar:04X} must be valid")),
        );
    }
    query
}

fn match_request(text: String) -> SearchRequest {
    SearchRequest {
        query: QueryNode::Match(MatchQuery {
            field: FIELD.to_owned(),
            text,
            op: MatchOp::And,
        }),
        limit: 16,
        offset: 0,
        cursor: None,
        routing_key: None,
        sort: None,
        track_total: true,
        collapse: None,
    }
}

fn match_ids(engine: &Engine, phase: &str) -> BTreeSet<String> {
    engine
        .search(COLLECTION, match_request(target_query()))
        .unwrap_or_else(|error| panic!("{phase}: public Ngram Match query failed: {error:#}"))
        .hits
        .into_iter()
        .map(|hit| hit.external_id)
        .collect()
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
        "public metrics must publish exactly one {name} sample: {metrics}",
    );
    values[0]
        .parse::<u64>()
        .unwrap_or_else(|_| panic!("{name} must be an unsigned byte value: {}", values[0]))
}

async fn assert_public_pending_budget(engine: Arc<Engine>, sequence: u64, phase: &str) {
    let sink: Arc<dyn WriteSink> = Arc::new(MetricsSink {
        applied_sequence: sequence,
    });
    let state = AppState::with_components(engine, Arc::new(AuthConfig::open()), sink);
    let server = TestServer::new(router(state)).expect("open public staged Text metrics server");
    let response = server.get("/metrics").await;
    response.assert_status_ok();
    let metrics = response.text();
    let total = metric_u64(&metrics, "lumen_pending_change_total_bytes");
    let high_water = metric_u64(&metrics, "lumen_pending_change_high_water_bytes");
    assert!(
        total <= PENDING_HARD_LIMIT_BYTES,
        "{phase}: public pending total must stay within 256 MiB: total={total}",
    );
    assert!(
        high_water <= PENDING_HARD_LIMIT_BYTES,
        "{phase}: public pending high water must stay within 256 MiB: high_water={high_water}",
    );
}

async fn submit_schema(fixture: &Fixture) {
    let result =
        tokio::time::timeout(PUBLISH_WATCHDOG, fixture.writer.submit(create_entry())).await;
    assert!(
        result.is_ok(),
        "schema submission must finish before staged Text fault setup",
    );
    let result = result.expect("schema timeout assertion checked completion");
    assert!(
        result.is_ok(),
        "valid Text schema submission must not fail: {}",
        result
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"))
            .unwrap_or_default(),
    );
    assert_eq!(
        fixture.writer.applied_seq(),
        CREATE_SEQUENCE,
        "schema must establish the exact committed prefix",
    );
    assert_eq!(
        fixture
            .store
            .save_with_sequence(&fixture.engine, CREATE_SEQUENCE)
            .expect("save schema-only staged Text checkpoint"),
        CREATE_SEQUENCE,
        "schema checkpoint must publish the prefix watermark",
    );
    let cold = fixture
        .store
        .load_current_generation()
        .expect("cold-open schema-only staged Text checkpoint")
        .expect("schema checkpoint must publish CURRENT");
    assert_eq!(
        cold.sequence, CREATE_SEQUENCE,
        "schema checkpoint must retain the prefix watermark",
    );
    assert_eq!(
        cold.engine
            .stats(COLLECTION)
            .expect("schema-only staged Text cold stats")
            .documents_indexed,
        0,
        "schema checkpoint must not contain a hidden row",
    );
}

async fn wait_for_applied(writer: &WriteCoordinator, sequence: u64, phase: &str) {
    let applied = tokio::time::timeout(RECOVERY_WATCHDOG, async {
        loop {
            if writer.applied_seq() >= sequence {
                return;
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    })
    .await;
    assert!(
        applied.is_ok(),
        "{phase}: retained source must apply through sequence {sequence}; applied={}",
        writer.applied_seq(),
    );
}

async fn assert_retained_source(wal: &GateWal, expected_wire: &[u8]) {
    let mut source = wal
        .subscribe(CREATE_SEQUENCE)
        .await
        .expect("open fresh public WAL reader after unresolved failure");
    let delivered = tokio::time::timeout(GATE_WATCHDOG, source.next()).await;
    assert!(
        delivered.is_ok(),
        "committed staged Text source must remain readable before cleanup",
    );
    let delivered = delivered
        .expect("source watchdog assertion checked completion")
        .expect("retained MemWal source stream must not end")
        .expect("retained MemWal source must not become an error");
    assert_eq!(
        delivered.0, INDEX_SEQUENCE,
        "fresh source reader must receive unresolved committed Text sequence",
    );
    let actual_wire = delivered
        .1
        .encode()
        .expect("encode retained valid staged Text WAL source");
    assert_eq!(
        actual_wire, expected_wire,
        "stage-directory I/O failure must retain exact committed WAL bytes",
    );
}

async fn assert_recovery_after_restoring_stage_directory(fixture: &Fixture) {
    let base = fixture
        .store
        .load_current_generation()
        .expect("cold-open schema prefix before retained-source recovery")
        .expect("schema prefix CURRENT must remain available for recovery");
    assert_eq!(
        base.sequence, CREATE_SEQUENCE,
        "recovery must begin from the unchanged schema-only checkpoint prefix",
    );
    let recovered = base.engine;
    let shared_wal: SharedWal = fixture.wal.clone();
    let recovery_writer =
        WriteCoordinator::start_from(shared_wal, recovered.clone(), base.sequence);
    wait_for_applied(
        recovery_writer.as_ref(),
        INDEX_SEQUENCE,
        "fresh native recovery from retained staged Text source",
    )
    .await;
    assert!(
        !recovery_writer.is_restart_required(),
        "restored private stage directory must let retained-source recovery finish",
    );
    assert_eq!(
        recovered
            .stats(COLLECTION)
            .expect("recovered staged Text stats")
            .documents_indexed,
        1,
        "fresh recovery must apply the retained committed staged Text row",
    );
    assert_eq!(
        match_ids(&recovered, "fresh retained-source recovery"),
        BTreeSet::from([EXTERNAL_ID.to_owned()]),
        "fresh recovery must expose retained staged Text through public Match",
    );
    assert_eq!(
        fixture
            .store
            .save_with_sequence(&recovered, INDEX_SEQUENCE)
            .expect("checkpoint recovered staged Text source"),
        INDEX_SEQUENCE,
        "post-recovery checkpoint must advance through retained source",
    );
    let cold = fixture
        .store
        .load_current_generation()
        .expect("cold-open recovered staged Text checkpoint")
        .expect("recovered staged Text checkpoint must publish CURRENT");
    assert_eq!(
        cold.sequence, INDEX_SEQUENCE,
        "cold recovered checkpoint must retain source watermark",
    );
    assert_eq!(
        match_ids(&cold.engine, "cold retained-source recovery"),
        BTreeSet::from([EXTERNAL_ID.to_owned()]),
        "cold recovered checkpoint must retain staged Text Match result",
    );
    assert_public_pending_budget(recovered, INDEX_SEQUENCE, "fresh retained-source recovery").await;
}

async fn run_child(root: PathBuf, child_tmp: PathBuf, handshake: PathBuf) {
    fs::write(&handshake, CHILD_CASE).expect("record exact staged Text child entry");
    assert!(
        child_tmp.is_dir(),
        "child must begin with parent-owned TMPDIR as a directory",
    );

    let fixture = fixture(&root);
    submit_schema(&fixture).await;
    fixture.wal.gate.hold(INDEX_SEQUENCE);

    let entry = target_entry();
    let expected_wire = WalRecord::new(entry.clone())
        .encode()
        .expect("encode valid staged Text fast WAL source");
    assert!(
        expected_wire.starts_with(b"LWAL"),
        "fixture must use real fast Index WAL encoding",
    );
    assert!(
        expected_wire.len() < RAW_HTTP_LIMIT_BYTES,
        "valid staged Text WAL source must stay below 8 MiB: wire_bytes={}",
        expected_wire.len(),
    );
    assert!(
        expected_wire.len() >= RAW_TEXT_BYTES,
        "valid staged Text WAL source must retain full 2 MiB value: wire_bytes={}",
        expected_wire.len(),
    );

    let mut submitted = tokio::spawn({
        let writer = fixture.writer.clone();
        async move { writer.submit(entry).await }
    });
    fixture.wal.gate.wait_until_arrived().await;
    assert_eq!(
        fixture
            .wal
            .latest_seq()
            .await
            .expect("read MemWal head after target publish"),
        INDEX_SEQUENCE,
        "target must become committed before stage I/O failure",
    );

    let entries = fs::read_dir(&child_tmp)
        .expect("read valid empty child TMPDIR before I/O fault")
        .count();
    assert_eq!(
        entries, 0,
        "fixture must not place unrelated files in stage parent",
    );
    fs::remove_dir(&child_tmp).expect("replace child TMPDIR after WAL publish");
    fs::write(&child_tmp, b"not-a-directory")
        .expect("make child stage-directory root a regular file");
    assert!(
        child_tmp.is_file(),
        "stage root must be a regular file when delivery reaches StageDirectory create",
    );
    fixture.wal.gate.release();

    let result = tokio::time::timeout(FAILURE_WATCHDOG, &mut submitted).await;
    assert!(
        result.is_ok(),
        "committed stage-directory I/O error must report unresolved failure before cleanup",
    );
    let result = result
        .expect("stage failure timeout assertion checked completion")
        .expect("staged Text submit worker must not panic");
    assert!(
        result.is_err(),
        "a committed staged Text record with unusable private stage directory must not acknowledge successful uncharged normal apply",
    );
    assert!(
        fixture.writer.is_restart_required(),
        "stage-directory I/O failure must latch unresolved restart state instead of silent apply",
    );
    assert_eq!(
        fixture.writer.applied_seq(),
        CREATE_SEQUENCE,
        "failed staged Text record must not advance applied watermark",
    );
    assert_eq!(
        fixture
            .engine
            .stats(COLLECTION)
            .expect("failed staged Text collection stats")
            .documents_indexed,
        0,
        "failed staged Text record must not create a live document",
    );
    assert_eq!(
        match_ids(&fixture.engine, "failed staged Text stage-directory I/O"),
        BTreeSet::new(),
        "failed staged Text record must not become query-visible",
    );
    let current = fixture
        .store
        .load_current_generation()
        .expect("read CURRENT after failed staged Text record")
        .expect("schema checkpoint CURRENT must remain available");
    assert_eq!(
        current.sequence, CREATE_SEQUENCE,
        "failed staged Text record must not publish a generation",
    );
    assert_eq!(
        current
            .engine
            .stats(COLLECTION)
            .expect("CURRENT schema-only staged Text stats")
            .documents_indexed,
        0,
        "CURRENT after failed staged Text record must remain schema-only",
    );
    assert_public_pending_budget(
        fixture.engine.clone(),
        CREATE_SEQUENCE,
        "failed committed staged Text stage-directory I/O",
    )
    .await;
    assert_retained_source(fixture.wal.as_ref(), &expected_wire).await;

    fs::remove_file(&child_tmp).expect("remove only child regular-file TMPDIR fault");
    fs::create_dir(&child_tmp).expect("restore child TMPDIR before source recovery");
    assert!(
        child_tmp.is_dir(),
        "fresh recovery must receive writable private stage root",
    );
    assert_recovery_after_restoring_stage_directory(&fixture).await;
}

async fn run_isolated_case() {
    let child_workspace =
        tempfile::tempdir().expect("create parent-owned staged Text child workspace");
    let child_root = child_workspace.path().join("child-root");
    let child_tmp = child_root.join("tmp");
    fs::create_dir(&child_root).expect("create parent-owned staged Text child root");
    fs::create_dir(&child_tmp).expect("create parent-owned staged Text child TMPDIR");
    let handshake = child_workspace.path().join("entered-case");
    let stdout_path = child_workspace.path().join("child.stdout");
    let stderr_path = child_workspace.path().join("child.stderr");
    let executable = std::env::current_exe().expect("locate committed staged Text e2e executable");
    let child = Command::new(executable)
        .env(CHILD_MODE_ENV, CHILD_CASE)
        .env(CHILD_ROOT_ENV, &child_root)
        .env(CHILD_HANDSHAKE_ENV, &handshake)
        .env("TMPDIR", &child_tmp)
        .env("TEMP", &child_tmp)
        .env("TMP", &child_tmp)
        .arg(TEST_NAME)
        .arg("--exact")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .stdout(Stdio::from(
            File::create(&stdout_path).expect("create staged Text child stdout"),
        ))
        .stderr(Stdio::from(
            File::create(&stderr_path).expect("create staged Text child stderr"),
        ))
        .spawn()
        .expect("spawn isolated committed staged Text child");
    let mut child = ChildCleanup(Some(child));
    let deadline = Instant::now() + CHILD_WATCHDOG;
    loop {
        match child
            .0
            .as_mut()
            .expect("committed staged Text child remains owned until exit")
            .try_wait()
        {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => tokio::time::sleep(POLL_INTERVAL).await,
            Ok(None) => {
                let mut raw = child.0.take().expect("timed-out child remains owned");
                let _ = raw.kill();
                let status = raw.wait().expect("reap killed staged Text child");
                let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
                let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
                panic!(
                    "committed staged Text child exceeded cleanup watchdog {CHILD_WATCHDOG:?}; status={status}; stdout={stdout}; stderr={stderr}",
                );
            }
            Err(error) => panic!("poll committed staged Text child: {error}"),
        }
    }
    let status = child
        .0
        .take()
        .expect("exited committed staged Text child remains owned")
        .wait()
        .expect("wait for committed staged Text child");
    let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
    let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
    let entered = fs::read_to_string(&handshake).unwrap_or_else(|error| {
        panic!(
            "isolated staged Text child did not enter {TEST_NAME}: {error}; stdout={stdout}; stderr={stderr}",
        )
    });
    assert_eq!(
        entered, CHILD_CASE,
        "isolated staged Text child must enter intended behavior body",
    );
    assert!(
        status.success(),
        "isolated staged Text child failed: status={status}; stdout={stdout}; stderr={stderr}",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn committed_staged_text_stage_directory_failure_retains_source_and_never_falls_back() {
    if std::env::var(CHILD_MODE_ENV).ok().as_deref() == Some(CHILD_CASE) {
        let root = PathBuf::from(
            std::env::var_os(CHILD_ROOT_ENV)
                .expect("staged Text child must receive parent-owned root"),
        );
        let child_tmp = PathBuf::from(
            std::env::var_os("TMPDIR").expect("staged Text child must receive parent-owned TMPDIR"),
        );
        let handshake = PathBuf::from(
            std::env::var_os(CHILD_HANDSHAKE_ENV)
                .expect("staged Text child must receive handshake path"),
        );
        run_child(root, child_tmp, handshake).await;
        return;
    }

    run_isolated_case().await;
}
