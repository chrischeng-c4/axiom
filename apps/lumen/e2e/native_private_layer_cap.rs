//! Black-box contract for native committed scalar progress at the private-layer cap.
//!
//! The fixture puts records directly into the real `MemWal`; it does not call
//! `WriteCoordinator::submit`, stage a source, or call a private layer API.
//! Each of the first 16 records has a 33 MiB losing Keyword item and a small,
//! later LWW winner for the same cell. The losing item makes the native path
//! use its borrowed source handoff. The winner keeps each resulting private
//! scalar layer small, so the normal 128 MiB byte trigger cannot explain the
//! capacity checkpoint.
//!
//! One case configures the existing checkpoint owner and holds its real
//! `SyncFile` write. The 17th record must stay unapplied and budget-owned until
//! that checkpoint releases. A second case has no configured owner. It requires
//! lazy capacity maintenance, then uses the normal public checkpoint and cold
//! open to retain the final delete watermark. Each outer case runs its actual
//! fixture in a bounded child process, because native coordinator and budget
//! work can outlive an async test guard. The transport source bytes are
//! fixture-owned before `MemWal::publish` and then WAL-owned. They are outside
//! Lumen pending-change accounting.
//!
//! # Facets
//!
//! - Behavior: `native_private_layer_cap.rs:474-495`, called from `:693-697`,
//!   `:825-829`, and `:907-911`, requires ordered LWW replacement without row
//!   revival. `:618-651`, reached from `:805-815`, is the configured-owner red:
//!   the 17th committed record must reach a real `SyncFile` checkpoint while its
//!   watermark and reservation remain held. `:731-741`, reached from `:895-906`,
//!   requires the no-owner native path to lazily make the same real checkpoint
//!   progress. `:744-789` retains the delete and watermark through a public
//!   checkpoint and cold open. `:215-292` and `:922-937` isolate each real
//!   fixture so a retained worker cannot alter the other case. These assertions
//!   exercise direct native delivery
//!   at `apps/lumen/src/coordinator/committed_scalar.rs:16-86`, private scalar
//!   attachment at `apps/lumen/src/storage/committed_index_apply.rs:436-505`,
//!   and checkpoint publication at `apps/lumen/src/segment_checkpoint.rs:220-268`.
//! - Security: this cap coordination adds no caller-controlled parser, path,
//!   identity, or authorization decision in
//!   `apps/lumen/src/coordinator/committed_scalar.rs:16-86` or
//!   `apps/lumen/src/segment_checkpoint.rs:128-190`. The existing typed WAL
//!   input boundary remains fail-closed in
//!   `apps/lumen/e2e/raft_oversized_committed_apply.rs:453-484`. The existing
//!   process-written checkpoint/AOF recovery boundary remains fail-closed in
//!   `apps/lumen/e2e/segment_startup_fail_closed_e2e.rs:908-947`; this case's
//!   cold-open behavior assertions are at `:773-789`.
//! - Performance: `apps/lumen/docs/indexing.md:264-276` says, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget." It
//!   also promises "a field retains at most 16." The public ownership and
//!   budget assertions at `:543-557`, called at `:708`, `:765`, `:871`, and
//!   `:912-916`, require every current ownership state and high-water value to stay
//!   within that budget. The watchdogs bound test cleanup only. They make no
//!   product latency claim.
//!
//! # Root negative control
//!
//! After the capacity-owner implementation exists, replace its 17th-layer
//! wait-and-maintain branch with the current unconditional private append. The
//! configured-owner assertion at `:618-627` must fail because no checkpoint
//! reaches the held `SyncFile`. Restore the implementation bytes before any
//! other check.
//!
//! Gate: `cargo test -p lumen --test native_private_layer_cap -- --nocapture`.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io;
use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

use axum_test::TestServer;

use lumen::api::{router, AppState, CheckpointSink};
use lumen::auth::AuthConfig;
use lumen::coordinator::{WriteCoordinator, WriteSink};
use lumen::log_entry::RaftLogEntry;
use lumen::segment_checkpoint::{PendingChangeSpill, SegmentCheckpointSink};
use lumen::segment_rdb::{MergeObserver, MergePhase, SegmentRdbStore};
use lumen::storage::Engine;
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest, QueryNode,
    SearchRequest, TermQuery,
};
use lumen::wal::{MemWal, SharedWal, WalLog, WalRecord};
use storage_durable::{CommitStep, FailureInjector, FailurePoint};

const COLLECTION: &str = "native-private-layer-cap";
const FIELD: &str = "keyword";
const EXTERNAL_ID: &str = "native-private-layer-cap-row";
const PRIVATE_LAYER_CAP: usize = 16;
const BORROWED_LOSER_BYTES: usize = 33 * 1024 * 1024;
const PENDING_HARD_LIMIT_BYTES: u64 = 256 * 1024 * 1024;
const CREATE_SEQUENCE: u64 = 1;
const FIRST_LAYER_SEQUENCE: u64 = CREATE_SEQUENCE + 1;
const PRESSURE_SEQUENCE: u64 = FIRST_LAYER_SEQUENCE + PRIVATE_LAYER_CAP as u64;
const DELETE_SEQUENCE: u64 = PRESSURE_SEQUENCE + 1;
const PUBLISH_WATCHDOG: Duration = Duration::from_secs(20);
const APPLY_WATCHDOG: Duration = Duration::from_secs(90);
const CHECKPOINT_WATCHDOG: Duration = Duration::from_secs(90);
const CAPACITY_READY_WATCHDOG: Duration = Duration::from_secs(30);
const ISOLATED_CASE_WATCHDOG: Duration = Duration::from_secs(240);
const POLL_INTERVAL: Duration = Duration::from_millis(5);
const CHILD_CASE_ENV: &str = "LUMEN_NATIVE_PRIVATE_LAYER_CAP_CHILD";
const CHILD_HANDSHAKE_ENV: &str = "LUMEN_NATIVE_PRIVATE_LAYER_CAP_HANDSHAKE";
const CONFIGURED_CASE: &str = "configured";
const LAZY_CASE: &str = "lazy";
const CONFIGURED_TEST_NAME: &str =
    "native_borrowed_scalar_private_layer_cap_maintains_before_progress_and_cold_reopens";
const LAZY_TEST_NAME: &str =
    "native_borrowed_scalar_private_layer_cap_lazily_maintains_and_cold_reopens";

#[derive(Default)]
struct NoopMergeObserver;

impl MergeObserver for NoopMergeObserver {
    fn observe(&self, _: MergePhase) -> io::Result<()> {
        Ok(())
    }
}

/// Holds one actual `SyncFile` write without holding an engine apply lease.
#[derive(Default)]
struct HoldNextSyncFile {
    hold: Mutex<Option<(mpsc::SyncSender<()>, mpsc::Receiver<()>)>>,
}

impl HoldNextSyncFile {
    fn arm(&self, entered: mpsc::SyncSender<()>, release: mpsc::Receiver<()>) {
        assert!(
            self.hold
                .lock()
                .expect("private-layer-cap SyncFile hold mutex")
                .replace((entered, release))
                .is_none(),
            "private-layer-cap fixture arms exactly one checkpoint SyncFile hold",
        );
    }
}

impl FailureInjector for HoldNextSyncFile {
    fn check(&self, point: &FailurePoint) -> io::Result<()> {
        if point.step != CommitStep::SyncFile {
            return Ok(());
        }
        let Some((entered, release)) = self
            .hold
            .lock()
            .expect("private-layer-cap SyncFile hold mutex")
            .take()
        else {
            return Ok(());
        };
        entered.send(()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                "private-layer-cap checkpoint readiness receiver dropped",
            )
        })?;
        release.recv().map_err(|_| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                "private-layer-cap checkpoint release sender dropped",
            )
        })?;
        Ok(())
    }
}

/// Releases the checkpoint during assertion unwind as well as normal exit.
struct SyncRelease(Option<mpsc::SyncSender<()>>);

impl SyncRelease {
    fn release(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}

impl Drop for SyncRelease {
    fn drop(&mut self) {
        self.release();
    }
}

/// Owns an isolated child until it has exited. A blocked native coordinator
/// cannot survive its fixture and charge the next public case's process budget.
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

/// The outer test runs only this exact body in a fresh child. The handshake
/// makes an empty or wrong filtered child run fail rather than look green.
fn child_case_enters(case_key: &str) -> bool {
    if std::env::var(CHILD_CASE_ENV).ok().as_deref() != Some(case_key) {
        return false;
    }
    let handshake = std::env::var_os(CHILD_HANDSHAKE_ENV)
        .unwrap_or_else(|| panic!("{case_key}: isolated child needs a handshake path"));
    fs::write(&handshake, case_key)
        .unwrap_or_else(|error| panic!("{case_key}: write isolated child handshake: {error}"));
    true
}

/// Runs one capacity fixture in another process. Its temporary directory is
/// parent-owned, so killing a blocked child still removes fallback-owner and
/// temporary scalar files after the child has been reaped.
async fn run_isolated_case(case_key: &'static str, test_name: &'static str) {
    let child_root = tempfile::tempdir().expect("native private-layer-cap child workspace");
    let child_tmp = child_root.path().join("child-tmp");
    fs::create_dir(&child_tmp).expect("create parent-owned native child temporary directory");
    let handshake = child_root.path().join("entered-case");
    let stdout_path = child_root.path().join("child.stdout");
    let stderr_path = child_root.path().join("child.stderr");
    let executable =
        std::env::current_exe().expect("current native private-layer-cap test executable");
    let stdout = File::create(&stdout_path).expect("create native private-layer-cap child stdout");
    let stderr = File::create(&stderr_path).expect("create native private-layer-cap child stderr");
    let child = Command::new(executable)
        .env(CHILD_CASE_ENV, case_key)
        .env(CHILD_HANDSHAKE_ENV, &handshake)
        .env("TMPDIR", &child_tmp)
        .env("TEMP", &child_tmp)
        .env("TMP", &child_tmp)
        .arg(test_name)
        .arg("--exact")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .expect("spawn isolated native private-layer-cap child");
    let mut child = ChildCleanup(Some(child));
    let deadline = Instant::now() + ISOLATED_CASE_WATCHDOG;
    loop {
        match child
            .0
            .as_mut()
            .expect("native private-layer-cap child remains owned until it exits")
            .try_wait()
        {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => {
                tokio::time::sleep(POLL_INTERVAL).await;
            }
            Ok(None) => {
                let mut raw = child
                    .0
                    .take()
                    .expect("timed-out native private-layer-cap child remains owned");
                let _ = raw.kill();
                let status = raw
                    .wait()
                    .expect("wait for killed native private-layer-cap child");
                let stdout = fs::read_to_string(&stdout_path)
                    .expect("read killed native private-layer-cap child stdout");
                let stderr = fs::read_to_string(&stderr_path)
                    .expect("read killed native private-layer-cap child stderr");
                panic!(
                    "{case_key}: native private-layer-cap child exceeded cleanup watchdog {ISOLATED_CASE_WATCHDOG:?}; status={status}; stdout={stdout}; stderr={stderr}",
                );
            }
            Err(error) => {
                panic!("{case_key}: poll isolated native private-layer-cap child: {error}")
            }
        }
    }
    let status = child
        .0
        .take()
        .expect("exited native private-layer-cap child remains owned")
        .wait()
        .expect("wait for exited native private-layer-cap child");
    let stdout =
        fs::read_to_string(&stdout_path).expect("read native private-layer-cap child stdout");
    let stderr =
        fs::read_to_string(&stderr_path).expect("read native private-layer-cap child stderr");
    let entered = fs::read_to_string(&handshake).unwrap_or_else(|error| {
        panic!(
            "{case_key}: isolated child did not enter exact {test_name}: {error}; stdout={stdout}; stderr={stderr}",
        )
    });
    assert_eq!(
        entered, case_key,
        "{case_key}: isolated child must enter its intended native capacity body",
    );
    assert!(
        status.success(),
        "{case_key}: isolated native private-layer-cap child failed: status={status}; stdout={stdout}; stderr={stderr}",
    );
}

struct Fixture {
    _root: tempfile::TempDir,
    engine: Arc<Engine>,
    server: TestServer,
    store: Arc<SegmentRdbStore>,
    wal: Arc<MemWal>,
    writer: Arc<WriteCoordinator>,
    hold: Arc<HoldNextSyncFile>,
    spill: Option<PendingChangeSpill>,
}

fn fixture(configured_owner: bool) -> Fixture {
    let root = tempfile::tempdir().expect("native private-layer-cap fixture directory");
    let engine = Arc::new(Engine::new());
    let hold = Arc::new(HoldNextSyncFile::default());
    let store = Arc::new(
        SegmentRdbStore::with_failure_injector_and_merge_observer(
            root.path().join("segments"),
            hold.clone(),
            Arc::new(NoopMergeObserver),
        )
        .expect("open observed native private-layer-cap segment store"),
    );
    let wal = Arc::new(MemWal::new());
    let shared_wal: SharedWal = wal.clone();
    let writer = WriteCoordinator::start(shared_wal, engine.clone());
    let sink_writer: Arc<dyn WriteSink> = writer.clone();
    let checkpoint: Arc<dyn CheckpointSink> = Arc::new(SegmentCheckpointSink {
        engine: engine.clone(),
        store: store.clone(),
        writer: sink_writer.clone(),
        aof: None,
    });
    let state =
        AppState::with_components(engine.clone(), Arc::new(AuthConfig::open()), sink_writer)
            .with_checkpoint(checkpoint);
    let server = TestServer::new(router(state)).expect("open native private-layer-cap HTTP server");
    // The configured case must reuse this existing bootstrap owner. The lazy
    // case deliberately has no owner, so native delivery must install one only
    // when the 17th layer needs capacity.
    let spill = configured_owner.then(|| {
        PendingChangeSpill::configured_replay(
            engine.clone(),
            store.clone(),
            Duration::from_secs(3600),
        )
    });

    Fixture {
        _root: root,
        engine,
        server,
        store,
        wal,
        writer,
        hold,
        spill,
    }
}

fn keyword_schema() -> CreateCollectionRequest {
    CreateCollectionRequest {
        fields: BTreeMap::from([(
            FIELD.to_owned(),
            FieldSpec {
                field_type: FieldType::Keyword,
                analyzer: None,
                multi: None,
                dim: None,
                metric: None,
                backend: None,
                quantize: None,
            },
        )]),
    }
}

fn create_record() -> WalRecord {
    WalRecord::new(RaftLogEntry::CreateCollection {
        collection_id: COLLECTION.to_owned(),
        req: keyword_schema(),
    })
}

fn winner_value(layer: usize) -> String {
    format!("native-private-layer-cap-winner-{layer:02}")
}

fn borrowed_loser_value(layer: usize) -> String {
    let marker = format!("native-private-layer-cap-loser-{layer:02}:");
    assert!(
        marker.len() < BORROWED_LOSER_BYTES,
        "fixture marker must leave a real borrowed source span",
    );
    let mut bytes = vec![b'x'; BORROWED_LOSER_BYTES];
    bytes[..marker.len()].copy_from_slice(marker.as_bytes());
    String::from_utf8(bytes).expect("borrowed losing Keyword value is valid UTF-8")
}

fn layer_record(layer: usize) -> WalRecord {
    assert!(
        (1..=PRIVATE_LAYER_CAP + 1).contains(&layer),
        "fixture creates exactly the 16 private layers and one pressure record",
    );
    assert!(
        BORROWED_LOSER_BYTES > (PENDING_HARD_LIMIT_BYTES as usize / 8),
        "fixture must cross the native borrowed delivery threshold",
    );
    let first_version = (layer as u64)
        .checked_mul(2)
        .expect("fixture LWW version multiplication");
    let final_version = first_version
        .checked_add(1)
        .expect("fixture LWW final version");
    let items = vec![
        IndexItem {
            external_id: EXTERNAL_ID.to_owned(),
            field: FIELD.to_owned(),
            value: FieldValue::String(borrowed_loser_value(layer)),
            version: Some(first_version),
        },
        IndexItem {
            external_id: EXTERNAL_ID.to_owned(),
            field: FIELD.to_owned(),
            value: FieldValue::String(winner_value(layer)),
            version: Some(final_version),
        },
    ];
    assert_eq!(
        items.len(),
        2,
        "fixture keeps its valid fast Index under the public 1,000-item cap",
    );
    WalRecord::new(RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: IndexRequest {
            items,
            request_id: None,
        },
    })
}

fn delete_record() -> WalRecord {
    WalRecord::new(RaftLogEntry::Delete {
        collection_id: COLLECTION.to_owned(),
        external_id: EXTERNAL_ID.to_owned(),
        field: None,
    })
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
        .unwrap_or_else(|error| panic!("native private-layer-cap exact query failed: {error}"))
        .hits
        .into_iter()
        .map(|hit| hit.external_id)
        .collect()
}

fn assert_current_winner(engine: &Engine, layer: usize, phase: &str) {
    assert_eq!(
        engine
            .stats(COLLECTION)
            .expect("native private-layer-cap collection stats")
            .documents_indexed,
        1,
        "{phase}: repeated committed updates must retain one live document",
    );
    assert_eq!(
        search_ids(engine, &winner_value(layer)),
        vec![EXTERNAL_ID.to_owned()],
        "{phase}: the later LWW winner must be query-visible",
    );
    if layer > 1 {
        assert_eq!(
            search_ids(engine, &winner_value(layer - 1)),
            Vec::<String>::new(),
            "{phase}: an earlier private-layer winner must not revive",
        );
    }
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
        .unwrap_or_else(|_| panic!("{name} must be an unsigned integer: {}", values[0]))
}

async fn public_metric(server: &TestServer, name: &str) -> u64 {
    let response = server.get("/metrics").await;
    response.assert_status_ok();
    metric_u64(&response.text(), name)
}

#[derive(Debug)]
struct PendingBudget {
    reserved: u64,
    active: u64,
    frozen: u64,
    total: u64,
    high_water: u64,
}

async fn public_pending_budget(server: &TestServer) -> PendingBudget {
    let response = server.get("/metrics").await;
    response.assert_status_ok();
    let metrics = response.text();
    PendingBudget {
        reserved: metric_u64(&metrics, "lumen_pending_change_reserved_bytes"),
        active: metric_u64(&metrics, "lumen_pending_change_active_bytes"),
        frozen: metric_u64(&metrics, "lumen_pending_change_frozen_bytes"),
        total: metric_u64(&metrics, "lumen_pending_change_total_bytes"),
        high_water: metric_u64(&metrics, "lumen_pending_change_high_water_bytes"),
    }
}

async fn assert_public_pending_budget(server: &TestServer, phase: &str) {
    let budget = public_pending_budget(server).await;
    assert_eq!(
        budget.total,
        budget.reserved + budget.active + budget.frozen,
        "{phase}: public pending-change total must equal its reserved, active, and frozen owners: {budget:?}",
    );
    assert!(
        budget.total <= PENDING_HARD_LIMIT_BYTES,
        "{phase}: public pending-change total must stay within the documented 256 MiB budget: {budget:?}",
    );
    assert!(
        budget.high_water <= PENDING_HARD_LIMIT_BYTES,
        "{phase}: public pending-change high water must stay within the documented 256 MiB budget: {budget:?}",
    );
}

async fn publish_committed(wal: &MemWal, record: WalRecord, phase: &str) -> u64 {
    let published = tokio::time::timeout(PUBLISH_WATCHDOG, wal.publish(record)).await;
    assert!(
        published.is_ok(),
        "{phase}: native MemWal publish must return a committed sequence before test cleanup",
    );
    let published = published.expect("behavior assertion above checked publish watchdog");
    let diagnostic = published
        .as_ref()
        .err()
        .map(|error| format!("{error:#}"))
        .unwrap_or_default();
    assert!(
        published.is_ok(),
        "{phase}: a valid native committed record must not receive an error acknowledgement: {diagnostic}",
    );
    published.expect("behavior assertion above checked native MemWal acknowledgement")
}

async fn wait_for_committed_apply(fixture: &Fixture, sequence: u64, phase: &str) {
    let applied = tokio::time::timeout(APPLY_WATCHDOG, async {
        loop {
            if fixture.writer.applied_seq() >= sequence {
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
        applied.is_ok(),
        "{phase}: a valid native committed record must reach sequence {sequence} before test cleanup; applied_sequence={}, wal_head={wal_head}, restart_required={}",
        fixture.writer.applied_seq(),
        fixture.writer.is_restart_required(),
    );
}

async fn require_capacity_checkpoint_pause(
    fixture: &Fixture,
    pressure_sequence: u64,
    entered_rx: mpsc::Receiver<()>,
    release: &mut SyncRelease,
) {
    let entered =
        tokio::task::spawn_blocking(move || entered_rx.recv_timeout(CAPACITY_READY_WATCHDOG)).await;
    let ready = matches!(&entered, Ok(Ok(())));
    if !ready {
        // The current source has no cap checkpoint. A future failing path may
        // have a blocked save, and this keeps its run-scoped worker releasable.
        release.release();
    }
    assert!(
        ready,
        "the 17th native committed private scalar layer must start a real capacity checkpoint and reach its SyncFile before apply; monitor={entered:?}, applied_sequence={}, wal_head={}",
        fixture.writer.applied_seq(),
        fixture
            .wal
            .latest_seq()
            .await
            .expect("read native MemWal head after capacity readiness watchdog"),
    );
    assert_eq!(
        fixture.writer.applied_seq(),
        pressure_sequence - 1,
        "the retained 17th native source must remain unacknowledged while its real capacity checkpoint SyncFile is paused",
    );
    assert_eq!(
        fixture
            .wal
            .latest_seq()
            .await
            .expect("read native MemWal head while capacity checkpoint is paused"),
        pressure_sequence,
        "the native WAL must retain the committed pressure source while apply waits for capacity maintenance",
    );
    let budget = public_pending_budget(&fixture.server).await;
    assert!(
        budget.reserved > 0,
        "the paused 17th native source reservation must remain budget-owned until capacity publication releases it: {budget:?}",
    );
    assert_eq!(
        budget.total,
        budget.reserved + budget.active + budget.frozen,
        "the paused capacity checkpoint must report every retained source owner: {budget:?}",
    );
}

async fn checkpoint_now(server: &TestServer) {
    let response = tokio::time::timeout(CHECKPOINT_WATCHDOG, async {
        server.post("/admin/checkpoint").await
    })
    .await;
    assert!(
        response.is_ok(),
        "the real final on-demand checkpoint must finish before test cleanup",
    );
    let response = response.expect("behavior assertion above checked final checkpoint watchdog");
    response.assert_status_ok();
    assert_eq!(
        response.json::<serde_json::Value>()["persisted"],
        true,
        "the real final on-demand checkpoint must publish durable state",
    );
}

async fn apply_pre_cap_layers(fixture: &Fixture) {
    let create_sequence =
        publish_committed(&fixture.wal, create_record(), "collection schema").await;
    assert_eq!(
        create_sequence, CREATE_SEQUENCE,
        "the schema must be the first native committed MemWal record",
    );
    wait_for_committed_apply(fixture, create_sequence, "collection schema").await;

    for layer in 1..=PRIVATE_LAYER_CAP {
        let sequence = publish_committed(
            &fixture.wal,
            layer_record(layer),
            "pre-cap borrowed scalar layer",
        )
        .await;
        assert_eq!(
            sequence,
            FIRST_LAYER_SEQUENCE + (layer - 1) as u64,
            "each distinct pre-cap record must receive the next native sequence",
        );
        wait_for_committed_apply(fixture, sequence, "pre-cap borrowed scalar layer").await;
        assert_current_winner(
            &fixture.engine,
            layer,
            "live pre-cap native borrowed scalar update",
        );
    }
    assert_eq!(
        public_metric(
            &fixture.server,
            "lumen_segment_checkpoint_completed_total",
        )
        .await,
        0,
        "the first 16 small retained private layers must not hit the ordinary byte checkpoint trigger",
    );
    assert_public_pending_budget(&fixture.server, "16 live private scalar layers").await;
}

async fn require_capacity_checkpoint_completion(
    fixture: &Fixture,
    checkpoints_before: u64,
    phase: &str,
) {
    let completed = tokio::time::timeout(CAPACITY_READY_WATCHDOG, async {
        loop {
            if public_metric(&fixture.server, "lumen_segment_checkpoint_completed_total").await
                > checkpoints_before
            {
                return;
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    })
    .await;
    assert!(
        completed.is_ok(),
        "{phase}: the direct native caller with no configured owner must lazily install capacity maintenance and complete a real checkpoint; completed={completed:?}, applied_sequence={}, wal_head={}, restart_required={}",
        fixture.writer.applied_seq(),
        fixture
            .wal
            .latest_seq()
            .await
            .expect("read native MemWal head after lazy capacity maintenance watchdog"),
        fixture.writer.is_restart_required(),
    );
}

async fn final_delete_checkpoint_and_cold_open(fixture: &Fixture) {
    let delete_sequence =
        publish_committed(&fixture.wal, delete_record(), "full-document delete").await;
    assert_eq!(
        delete_sequence, DELETE_SEQUENCE,
        "the full-document delete must be ordered after the retained capacity record",
    );
    wait_for_committed_apply(fixture, delete_sequence, "full-document delete").await;
    assert_eq!(
        search_ids(&fixture.engine, &winner_value(PRIVATE_LAYER_CAP + 1)),
        Vec::<String>::new(),
        "the post-cap full-document delete must not revive the prior private scalar winner",
    );
    assert_eq!(
        fixture
            .engine
            .stats(COLLECTION)
            .expect("live post-cap delete collection stats")
            .documents_indexed,
        0,
        "the post-cap full-document delete must remove the only document",
    );
    assert_public_pending_budget(&fixture.server, "post-cap native full-document delete").await;

    checkpoint_now(&fixture.server).await;
    let cold = fixture
        .store
        .load_current_generation()
        .expect("cold-open final native private-layer-cap checkpoint")
        .expect("final native private-layer-cap checkpoint must publish CURRENT");
    assert_eq!(
        cold.sequence, delete_sequence,
        "final cold checkpoint must retain the native delete watermark after capacity maintenance",
    );
    assert_eq!(
        search_ids(&cold.engine, &winner_value(PRIVATE_LAYER_CAP + 1)),
        Vec::<String>::new(),
        "cold reopen must retain the post-cap delete and never revive the private scalar winner",
    );
    assert_eq!(
        cold.engine
            .stats(COLLECTION)
            .expect("cold post-cap delete collection stats")
            .documents_indexed,
        0,
        "cold reopen must retain the deleted document count",
    );
}

async fn configured_capacity_case() {
    let mut fixture = fixture(true);
    apply_pre_cap_layers(&fixture).await;

    let checkpoints_before =
        public_metric(&fixture.server, "lumen_segment_checkpoint_completed_total").await;
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    fixture.hold.arm(entered_tx, release_rx);
    let mut release = SyncRelease(Some(release_tx));
    let pressure_sequence = publish_committed(
        &fixture.wal,
        layer_record(PRIVATE_LAYER_CAP + 1),
        "17th borrowed scalar layer at the private-layer cap",
    )
    .await;
    assert_eq!(
        pressure_sequence, PRESSURE_SEQUENCE,
        "the 17th record must become the next committed native MemWal head",
    );
    require_capacity_checkpoint_pause(&fixture, pressure_sequence, entered_rx, &mut release).await;
    assert_eq!(
        public_metric(
            &fixture.server,
            "lumen_segment_checkpoint_completed_total",
        )
        .await,
        checkpoints_before,
        "a checkpoint paused before SyncFile completion must not publish its metric or release the pressure record",
    );
    assert_current_winner(
        &fixture.engine,
        PRIVATE_LAYER_CAP,
        "live native state while capacity checkpoint SyncFile is paused",
    );
    release.release();
    wait_for_committed_apply(
        &fixture,
        pressure_sequence,
        "17th borrowed scalar layer after capacity checkpoint release",
    )
    .await;
    fixture
        .spill
        .as_mut()
        .expect("configured capacity case owns a configured checkpoint driver")
        .stop_bootstrap()
        .await
        .expect("configured capacity checkpoint driver must stop after its started save");
    assert!(
        public_metric(
            &fixture.server,
            "lumen_segment_checkpoint_completed_total",
        )
        .await
            > checkpoints_before,
        "the released capacity checkpoint must publish before the retained source is allowed to make later progress",
    );
    assert_eq!(
        fixture
            .wal
            .latest_seq()
            .await
            .expect("read native MemWal head after capacity maintenance"),
        pressure_sequence,
        "the retained native source head must remain aligned with the applied capacity record",
    );
    assert!(
        !fixture.writer.is_restart_required(),
        "capacity maintenance for a valid retained source must not leave native delivery restart-required",
    );
    assert_current_winner(
        &fixture.engine,
        PRIVATE_LAYER_CAP + 1,
        "live post-cap native borrowed scalar update",
    );
    assert_public_pending_budget(&fixture.server, "post-cap native borrowed scalar update").await;

    final_delete_checkpoint_and_cold_open(&fixture).await;
}

async fn lazy_capacity_case() {
    let fixture = fixture(false);
    apply_pre_cap_layers(&fixture).await;

    let checkpoints_before =
        public_metric(&fixture.server, "lumen_segment_checkpoint_completed_total").await;
    let pressure_sequence = publish_committed(
        &fixture.wal,
        layer_record(PRIVATE_LAYER_CAP + 1),
        "17th borrowed scalar layer with no configured capacity owner",
    )
    .await;
    assert_eq!(
        pressure_sequence, PRESSURE_SEQUENCE,
        "the 17th no-owner record must become the next committed native MemWal head",
    );
    wait_for_committed_apply(
        &fixture,
        pressure_sequence,
        "17th borrowed scalar layer after lazy capacity owner bootstrap",
    )
    .await;
    require_capacity_checkpoint_completion(
        &fixture,
        checkpoints_before,
        "17th native private scalar layer",
    )
    .await;
    assert_current_winner(
        &fixture.engine,
        PRIVATE_LAYER_CAP + 1,
        "live post-cap native borrowed scalar update after lazy owner bootstrap",
    );
    assert_public_pending_budget(
        &fixture.server,
        "post-cap native borrowed scalar update after lazy owner bootstrap",
    )
    .await;

    final_delete_checkpoint_and_cold_open(&fixture).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn native_borrowed_scalar_private_layer_cap_maintains_before_progress_and_cold_reopens() {
    if child_case_enters(CONFIGURED_CASE) {
        configured_capacity_case().await;
        return;
    }
    run_isolated_case(CONFIGURED_CASE, CONFIGURED_TEST_NAME).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn native_borrowed_scalar_private_layer_cap_lazily_maintains_and_cold_reopens() {
    if child_case_enters(LAZY_CASE) {
        lazy_capacity_case().await;
        return;
    }
    run_isolated_case(LAZY_CASE, LAZY_TEST_NAME).await;
}
