//! Black-box contract for AOF recovery at the 16 catalog-layer field cap.
//!
//! The parent builds one real durable root with a full Keyword base and then
//! 16 real incremental catalog deltas. A run-scoped MergeObserver rejects each
//! requested compaction after the fourth delta, and the fixture waits for every
//! requested background job before it writes the next delta. This keeps the
//! catalog full without using a private layer probe or a source counter.
//!
//! A separate test process cold-opens that root with no configured checkpoint
//! driver. It replays one real AofWriter fast Index suffix above the borrowed
//! route threshold. The suffix has a 33 MiB lower-version Keyword value and a
//! small higher-version value for the same cell. The child process makes a
//! baseline replay hang safe to clean up. Its watchdog is test cleanup only.
//! It is not a product latency assertion.
//!
//! # Facets
//!
//! - Behavior: aof_recovered_catalog_layer_cap.rs:308-378 creates the forced
//!   full base and exactly 16 completed catalog deltas.
//!   aof_recovered_catalog_layer_cap.rs:540-591 requires the cold AOF suffix
//!   to advance through its committed watermark, retain the LWW winner,
//!   checkpoint, and cold reopen. aof_recovered_catalog_layer_cap.rs:627-660
//!   turns a retained baseline loop into the red without leaving a live child.
//!   These assertions cover
//!   apps/lumen/src/aof.rs:266-315,
//!   apps/lumen/src/storage/committed_index_apply.rs:288-309, and
//!   apps/lumen/src/segment_capacity.rs:261-274.
//! - Security: apps/lumen/src/segment_capacity.rs:261-274 only drives
//!   checkpoint and merge work after established Engine state. The changed AOF
//!   path at apps/lumen/src/aof.rs:266-315 passes persisted bytes through its
//!   existing frame validation. The existing closed-input case in
//!   apps/lumen/e2e/aof_oversized_committed_apply.rs:445-503 feeds a complete,
//!   CRC-valid truncated fast payload and requires refusal before its
//!   watermark. This contract supplies only validated process-written frames.
//! - Performance: apps/lumen/docs/indexing.md:264-276 says, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget."
//!   It also says "a field retains at most 16."
//!   aof_recovered_catalog_layer_cap.rs:500-507, called at
//!   aof_recovered_catalog_layer_cap.rs:556-561,
//!   aof_recovered_catalog_layer_cap.rs:570-575, and
//!   aof_recovered_catalog_layer_cap.rs:586-591,
//!   reads real public metrics after replay,
//!   checkpoint, and cold reopen and requires total and high-water pending
//!   bytes to stay within that budget. Fixture and mapped AOF source bytes are
//!   outside those counters.
//!
//! # Root negative control
//!
//! After the empty temporary-root fallback is implemented, remove its
//! merge-without-CURRENT checkpoint branch in
//! apps/lumen/src/segment_capacity.rs:261-274. The isolated-child behavior
//! assertion below must fail because a catalog-only 16-layer AOF suffix cannot
//! make capacity progress. Restore the source hash before any other gate.
//!
//! Gate: cargo test -p lumen --test aof_recovered_catalog_layer_cap -- --nocapture.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use axum_test::TestServer;

use lumen::aof::{replay_aof_into, AofWriter};
use lumen::api::{router, AppState};
use lumen::auth::AuthConfig;
use lumen::coordinator::WriteSink;
use lumen::log_entry::RaftLogEntry;
use lumen::segment_rdb::{MergeObserver, MergePhase, SegmentRdbStore};
use lumen::storage::{ApplyOutcome, Engine};
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest, QueryNode,
    SearchRequest, TermQuery,
};
use lumen::wal::WalRecord;

const COLLECTION: &str = "aof-recovered-catalog-layer-cap";
const FIELD: &str = "keyword";
const BOOTSTRAP_FIELD: &str = "force-full-base";
const EXTERNAL_ID: &str = "aof-recovered-catalog-layer-cap-row";
const CATALOG_LAYERS: usize = 16;
const BORROWED_ROUTE_MINIMUM_BYTES: usize = 32 * 1024 * 1024;
const LARGE_LOSER_BYTES: usize = 33 * 1024 * 1024;
const PENDING_HARD_LIMIT_BYTES: u64 = 256 * 1024 * 1024;
const CREATE_SEQUENCE: u64 = 1;
const DROP_BOOTSTRAP_SEQUENCE: u64 = 2;
const BASE_SEQUENCE: u64 = 3;
const FIRST_DELTA_SEQUENCE: u64 = BASE_SEQUENCE + 1;
const CATALOG_WATERMARK: u64 = BASE_SEQUENCE + CATALOG_LAYERS as u64;
const SUFFIX_SEQUENCE: u64 = CATALOG_WATERMARK + 1;
const MERGE_WATCHDOG: Duration = Duration::from_secs(30);
const REPLAY_WATCHDOG: Duration = Duration::from_secs(45);
const POLL_INTERVAL: Duration = Duration::from_millis(25);
const INJECTED_MERGE_FAILURE: &str = "aof recovered catalog layer cap blocks merge";
const CHILD_MODE_ENV: &str = "LUMEN_AOF_RECOVERED_CATALOG_LAYER_CAP_CHILD";
const CHILD_ROOT_ENV: &str = "LUMEN_AOF_RECOVERED_CATALOG_LAYER_CAP_ROOT";
const CHILD_AOF_ENV: &str = "LUMEN_AOF_RECOVERED_CATALOG_LAYER_CAP_AOF";
const CHILD_HANDSHAKE_ENV: &str = "LUMEN_AOF_RECOVERED_CATALOG_LAYER_CAP_HANDSHAKE";
const CHILD_CASE: &str = "replay";
const TEST_NAME: &str =
    "aof_recovered_catalog_layer_cap_replay_makes_capacity_progress_and_cold_reopens";

/// Metrics must observe the replayed Engine without starting a new coordinator.
/// A metrics request is read-only, so submit must remain closed.
struct ReadOnlyMetricsSink {
    applied_sequence: u64,
}

#[async_trait]
impl WriteSink for ReadOnlyMetricsSink {
    async fn submit(&self, _entry: RaftLogEntry) -> anyhow::Result<ApplyOutcome> {
        anyhow::bail!("the read-only metrics server must not submit a mutation");
    }

    fn applied_seq(&self) -> u64 {
        self.applied_sequence
    }
}

/// This failure is the only tolerated background result while the parent
/// creates the 16-delta catalog. The flag is disabled before recovery starts.
struct RejectCatalogMerge {
    enabled: AtomicBool,
    rejections: AtomicUsize,
}

impl RejectCatalogMerge {
    fn new() -> Self {
        Self {
            enabled: AtomicBool::new(true),
            rejections: AtomicUsize::new(0),
        }
    }

    fn allow(&self) {
        self.enabled.store(false, Ordering::Release);
    }

    fn rejections(&self) -> usize {
        self.rejections.load(Ordering::Acquire)
    }
}

impl MergeObserver for RejectCatalogMerge {
    fn observe(&self, phase: MergePhase) -> io::Result<()> {
        if phase == MergePhase::BeforeEncode && self.enabled.load(Ordering::Acquire) {
            self.rejections.fetch_add(1, Ordering::AcqRel);
            return Err(io::Error::other(INJECTED_MERGE_FAILURE));
        }
        Ok(())
    }
}

/// Owns a child until it has exited. A replay that remains retained behind the
/// old catalog-cap loop cannot leak a live process into another test.
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

fn schema_entry() -> RaftLogEntry {
    RaftLogEntry::CreateCollection {
        collection_id: COLLECTION.to_owned(),
        req: CreateCollectionRequest {
            fields: BTreeMap::from([
                (FIELD.to_owned(), keyword_spec()),
                (BOOTSTRAP_FIELD.to_owned(), keyword_spec()),
            ]),
        },
    }
}

fn drop_bootstrap_entry() -> RaftLogEntry {
    RaftLogEntry::DropField {
        collection_id: COLLECTION.to_owned(),
        field_name: BOOTSTRAP_FIELD.to_owned(),
    }
}

fn index_entry(value: String, version: u64) -> RaftLogEntry {
    RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items: vec![IndexItem {
                external_id: EXTERNAL_ID.to_owned(),
                field: FIELD.to_owned(),
                value: FieldValue::String(value),
                version: Some(version),
            }],
            request_id: None,
        },
    }
}

fn base_value() -> String {
    "aof-recovered-catalog-base".to_owned()
}

fn catalog_value(round: usize) -> String {
    format!("aof-recovered-catalog-delta-{round:02}")
}

fn winner_value() -> String {
    "aof-recovered-catalog-suffix-winner".to_owned()
}

fn borrowed_loser_value() -> String {
    let marker = "aof-recovered-catalog-suffix-loser:";
    assert!(
        marker.len() < LARGE_LOSER_BYTES,
        "fixture marker must leave a real 33 MiB borrowed Keyword source",
    );
    let mut bytes = vec![b'x'; LARGE_LOSER_BYTES];
    bytes[..marker.len()].copy_from_slice(marker.as_bytes());
    String::from_utf8(bytes).expect("borrowed losing Keyword fixture is valid UTF-8")
}

fn suffix_entry() -> RaftLogEntry {
    let items = vec![
        IndexItem {
            external_id: EXTERNAL_ID.to_owned(),
            field: FIELD.to_owned(),
            value: FieldValue::String(borrowed_loser_value()),
            version: Some(CATALOG_LAYERS as u64 + 2),
        },
        IndexItem {
            external_id: EXTERNAL_ID.to_owned(),
            field: FIELD.to_owned(),
            value: FieldValue::String(winner_value()),
            version: Some(CATALOG_LAYERS as u64 + 3),
        },
    ];
    assert_eq!(
        items.len(),
        2,
        "the real fast Index suffix stays within the public 1,000-item cap",
    );
    RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items,
            request_id: None,
        },
    }
}

fn append_and_apply(
    writer: &mut AofWriter,
    engine: &Arc<Engine>,
    sequence: u64,
    entry: RaftLogEntry,
    phase: &str,
) {
    let record = WalRecord::new(entry.clone());
    writer
        .append(sequence, &record)
        .unwrap_or_else(|error| panic!("{phase}: append real AOF record {sequence}: {error:#}"));
    engine
        .apply_raft_entry(entry)
        .unwrap_or_else(|error| panic!("{phase}: apply real committed record {sequence}: {error:#}"));
}

fn wait_for_expected_merge_result(store: &SegmentRdbStore, round: usize) {
    let result = store.wait_for_merges(MERGE_WATCHDOG);
    if round < 4 {
        assert!(
            result.is_ok(),
            "catalog delta {round}: fewer than four deltas must not have a background merge failure: {result:?}",
        );
        return;
    }
    let error = result
        .err()
        .map(|error| format!("{error:#}"))
        .unwrap_or_default();
    assert!(
        error.contains(INJECTED_MERGE_FAILURE),
        "catalog delta {round}: only the run-scoped injected merge rejection is tolerated after waiting for its job: {error}",
    );
}

fn build_sixteen_catalog_deltas(
    store: &Arc<SegmentRdbStore>,
    observer: &Arc<RejectCatalogMerge>,
    engine: &Arc<Engine>,
    writer: &mut AofWriter,
) {
    append_and_apply(
        writer,
        engine,
        CREATE_SEQUENCE,
        schema_entry(),
        "create collection with temporary bootstrap field",
    );
    append_and_apply(
        writer,
        engine,
        DROP_BOOTSTRAP_SEQUENCE,
        drop_bootstrap_entry(),
        "drop bootstrap field to force the first full base checkpoint",
    );
    append_and_apply(
        writer,
        engine,
        BASE_SEQUENCE,
        index_entry(base_value(), 1),
        "apply base Keyword row",
    );
    writer
        .sync_strict()
        .expect("strict-sync durable AOF prefix before base checkpoint");
    let base_saved = store
        .save_with_sequence(engine, BASE_SEQUENCE)
        .expect("publish real full base checkpoint");
    assert_eq!(
        base_saved, BASE_SEQUENCE,
        "the forced full base checkpoint must retain its exact watermark",
    );

    for round in 1..=CATALOG_LAYERS {
        let sequence = FIRST_DELTA_SEQUENCE + (round - 1) as u64;
        append_and_apply(
            writer,
            engine,
            sequence,
            index_entry(catalog_value(round), round as u64 + 1),
            "apply catalog delta row",
        );
        let saved = store
            .save_with_sequence(engine, sequence)
            .unwrap_or_else(|error| panic!("catalog delta {round}: save real checkpoint: {error:#}"));
        assert_eq!(
            saved, sequence,
            "catalog delta {round}: checkpoint must publish its exact next sequence",
        );
        wait_for_expected_merge_result(store, round);
    }
    assert_eq!(
        observer.rejections(),
        CATALOG_LAYERS - 3,
        "the observer must reject one finished merge job for every fourth-through-sixteenth catalog delta",
    );
    assert_eq!(
        store
            .load_current_generation()
            .expect("cold-open catalog-cap checkpoint before suffix")
            .expect("catalog-cap checkpoint must publish CURRENT")
            .sequence,
        CATALOG_WATERMARK,
        "the catalog-cap checkpoint must cold-open at its exact sixteenth-delta watermark",
    );
}

fn append_oversized_suffix(writer: &mut AofWriter, aof_path: &Path) {
    let before = fs::metadata(aof_path)
        .expect("measure AOF before real oversized suffix")
        .len();
    let record = WalRecord::new(suffix_entry());
    writer
        .append(SUFFIX_SEQUENCE, &record)
        .expect("append real oversized fast Index suffix through AofWriter");
    drop(record);
    writer
        .sync_strict()
        .expect("strict-sync real oversized AOF suffix");
    let suffix_bytes = fs::metadata(aof_path)
        .expect("measure AOF after real oversized suffix")
        .len()
        .checked_sub(before)
        .expect("AOF suffix length must not move backwards");
    assert!(
        suffix_bytes > BORROWED_ROUTE_MINIMUM_BYTES as u64,
        "the real AofWriter suffix must cross the >32 MiB borrowed replay route: suffix_bytes={suffix_bytes}",
    );
}

fn term_query(value: &str) -> SearchRequest {
    SearchRequest {
        query: QueryNode::Term(TermQuery {
            field: FIELD.to_owned(),
            value: FieldValue::String(value.to_owned()),
        }),
        limit: 10,
        offset: 0,
        cursor: None,
        routing_key: None,
        sort: None,
        track_total: true,
        collapse: None,
    }
}

fn search_ids(engine: &Engine, value: &str) -> Vec<String> {
    engine
        .search(COLLECTION, term_query(value))
        .unwrap_or_else(|error| panic!("Keyword query for {value} failed: {error:#}"))
        .hits
        .into_iter()
        .map(|hit| hit.external_id)
        .collect()
}

fn assert_catalog_winner(engine: &Engine, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("catalog-cap collection stats")
            .documents_indexed,
        1,
        "{phase}: the catalog checkpoint must retain one live document",
    );
    assert_eq!(
        search_ids(engine, &catalog_value(CATALOG_LAYERS)),
        vec![EXTERNAL_ID.to_owned()],
        "{phase}: cold checkpoint must retain its sixteenth catalog winner",
    );
    assert_eq!(
        search_ids(engine, &catalog_value(CATALOG_LAYERS - 1)),
        Vec::<String>::new(),
        "{phase}: an older catalog delta must not revive",
    );
}

fn assert_suffix_winner(engine: &Engine, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("suffix collection stats")
            .documents_indexed,
        1,
        "{phase}: the LWW suffix must retain one live document",
    );
    assert_eq!(
        search_ids(engine, &winner_value()),
        vec![EXTERNAL_ID.to_owned()],
        "{phase}: the small higher-version suffix winner must be query-visible",
    );
    assert_eq!(
        search_ids(engine, &catalog_value(CATALOG_LAYERS)),
        Vec::<String>::new(),
        "{phase}: the pre-replay catalog winner must not revive after the suffix",
    );
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

async fn assert_public_pending_budget(engine: Arc<Engine>, sequence: u64, phase: &str) {
    let sink: Arc<dyn WriteSink> = Arc::new(ReadOnlyMetricsSink {
        applied_sequence: sequence,
    });
    let state = AppState::with_components(engine, Arc::new(AuthConfig::open()), sink);
    let server = TestServer::new(router(state)).expect("open read-only public metrics server");
    let response = server.get("/metrics").await;
    response.assert_status_ok();
    let metrics = response.text();
    let total = metric_u64(&metrics, "lumen_pending_change_total_bytes");
    let high_water = metric_u64(&metrics, "lumen_pending_change_high_water_bytes");
    assert!(
        total <= PENDING_HARD_LIMIT_BYTES,
        "{phase}: public pending-change total must stay within the documented 256 MiB budget: total={total}",
    );
    assert!(
        high_water <= PENDING_HARD_LIMIT_BYTES,
        "{phase}: public pending-change high water must stay within the documented 256 MiB budget: high_water={high_water}",
    );
}

fn child_paths() -> Option<(PathBuf, PathBuf, PathBuf)> {
    if std::env::var(CHILD_MODE_ENV).ok().as_deref() != Some(CHILD_CASE) {
        return None;
    }
    let root = std::env::var_os(CHILD_ROOT_ENV)
        .map(PathBuf::from)
        .expect("isolated replay child needs its durable root");
    let aof = std::env::var_os(CHILD_AOF_ENV)
        .map(PathBuf::from)
        .expect("isolated replay child needs its AOF path");
    let handshake = std::env::var_os(CHILD_HANDSHAKE_ENV)
        .map(PathBuf::from)
        .expect("isolated replay child needs a handshake path");
    Some((root, aof, handshake))
}

async fn run_replay_child(root: PathBuf, aof_path: PathBuf, handshake: PathBuf) {
    fs::write(&handshake, CHILD_CASE).expect("record exact isolated replay child entry");
    let store = SegmentRdbStore::new(root.join("segments"))
        .expect("open cold catalog-cap segment root without a configured driver");
    let cold = store
        .load_current_generation()
        .expect("cold-open real sixteen-delta catalog checkpoint")
        .expect("real sixteen-delta catalog checkpoint must publish CURRENT");
    assert_eq!(
        cold.sequence, CATALOG_WATERMARK,
        "cold recovery must start exactly after the catalog-cap checkpoint",
    );
    assert_catalog_winner(&cold.engine, "cold catalog-cap checkpoint before AOF suffix");

    let replay = replay_aof_into(&cold.engine, &aof_path, cold.sequence);
    assert!(
        replay.is_ok(),
        "a valid >32 MiB fast Index AOF suffix after 16 catalog layers must make independent capacity progress instead of remaining retained: {}",
        replay
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"))
            .unwrap_or_default(),
    );
    assert_eq!(
        replay.expect("behavior assertion above checked suffix replay"),
        SUFFIX_SEQUENCE,
        "the valid AOF suffix must advance the replay watermark",
    );
    assert_suffix_winner(&cold.engine, "live cold-AOF suffix replay");
    assert_public_pending_budget(
        cold.engine.clone(),
        SUFFIX_SEQUENCE,
        "cold-AOF suffix replay",
    )
    .await;

    let saved = store
        .save_with_sequence(&cold.engine, SUFFIX_SEQUENCE)
        .expect("checkpoint replayed suffix after independent capacity maintenance");
    assert_eq!(
        saved, SUFFIX_SEQUENCE,
        "checkpoint after replay must retain the suffix watermark",
    );
    assert_public_pending_budget(
        cold.engine.clone(),
        SUFFIX_SEQUENCE,
        "checkpoint after cold-AOF suffix replay",
    )
    .await;

    let final_cold = store
        .load_current_generation()
        .expect("cold-open replayed suffix checkpoint")
        .expect("replayed suffix checkpoint must publish CURRENT");
    assert_eq!(
        final_cold.sequence, SUFFIX_SEQUENCE,
        "final cold checkpoint must retain the AOF suffix watermark",
    );
    assert_suffix_winner(&final_cold.engine, "final cold reopen after AOF suffix");
    assert_public_pending_budget(
        final_cold.engine.clone(),
        SUFFIX_SEQUENCE,
        "final cold reopen after AOF suffix",
    )
    .await;
}

async fn run_isolated_replay(root: &Path, aof_path: &Path) {
    let child_root = tempfile::tempdir().expect("isolated AOF replay child directory");
    let handshake = child_root.path().join("entered-case");
    let stdout_path = child_root.path().join("child.stdout");
    let stderr_path = child_root.path().join("child.stderr");
    let executable = std::env::current_exe().expect("current AOF catalog-cap test executable");
    let stdout = File::create(&stdout_path).expect("create isolated replay child stdout");
    let stderr = File::create(&stderr_path).expect("create isolated replay child stderr");
    let child = Command::new(executable)
        .env(CHILD_MODE_ENV, CHILD_CASE)
        .env(CHILD_ROOT_ENV, root)
        .env(CHILD_AOF_ENV, aof_path)
        .env(CHILD_HANDSHAKE_ENV, &handshake)
        .arg(TEST_NAME)
        .arg("--exact")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .expect("spawn isolated cold-AOF replay child");
    let mut child = ChildCleanup(Some(child));
    let deadline = Instant::now() + REPLAY_WATCHDOG;
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
                let mut raw = child.0.take().expect("timed out child remains owned");
                let _ = raw.kill();
                let status = raw.wait().expect("wait for killed cold-AOF replay child");
                let stdout = fs::read_to_string(&stdout_path)
                    .expect("read killed cold-AOF replay child stdout");
                let stderr = fs::read_to_string(&stderr_path)
                    .expect("read killed cold-AOF replay child stderr");
                panic!(
                    "a valid cold AOF suffix blocked only by the 16 catalog-layer cap must replay and checkpoint before test cleanup; isolated child was killed after {REPLAY_WATCHDOG:?}: status={status}; stdout={stdout}; stderr={stderr}",
                );
            }
            Err(error) => panic!("poll isolated cold-AOF replay child: {error}"),
        }
    }
    let status = child
        .0
        .take()
        .expect("exited child remains owned")
        .wait()
        .expect("wait for exited cold-AOF replay child");
    let stdout = fs::read_to_string(&stdout_path).expect("read cold-AOF replay child stdout");
    let stderr = fs::read_to_string(&stderr_path).expect("read cold-AOF replay child stderr");
    let entered = fs::read_to_string(&handshake).unwrap_or_else(|error| {
        panic!(
            "isolated child did not enter exact {TEST_NAME}: {error}; stdout={stdout}; stderr={stderr}",
        )
    });
    assert_eq!(
        entered, CHILD_CASE,
        "isolated child must enter the intended AOF replay body",
    );
    assert!(
        status.success(),
        "isolated cold-AOF replay child failed: status={status}; stdout={stdout}; stderr={stderr}",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn aof_recovered_catalog_layer_cap_replay_makes_capacity_progress_and_cold_reopens() {
    if let Some((root, aof_path, handshake)) = child_paths() {
        run_replay_child(root, aof_path, handshake).await;
        return;
    }

    let root = tempfile::tempdir().expect("AOF recovered catalog-cap fixture root");
    let aof_path = root.path().join("aof.log");
    let observer = Arc::new(RejectCatalogMerge::new());
    let store = Arc::new(
        SegmentRdbStore::with_merge_observer(root.path().join("segments"), observer.clone())
            .expect("open observed catalog-cap segment root"),
    );
    let engine = Arc::new(Engine::new());
    let mut writer = AofWriter::open(&aof_path).expect("open real AOF writer");

    build_sixteen_catalog_deltas(&store, &observer, &engine, &mut writer);
    let cold = store
        .load_current_generation()
        .expect("cold-open parent catalog-cap checkpoint")
        .expect("parent catalog-cap checkpoint must publish CURRENT");
    assert_catalog_winner(&cold.engine, "parent cold catalog-cap checkpoint");
    observer.allow();
    append_oversized_suffix(&mut writer, &aof_path);
    drop(writer);

    run_isolated_replay(root.path(), &aof_path).await;
}
