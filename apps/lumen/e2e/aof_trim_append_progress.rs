#![cfg(unix)]
//! This contract requires the staged trim's Unix positional reads and inode identity.
//!
//! Black-box AOF trim/apply-progress contract.
//!
//! A real checkpoint captures and publishes record `C`.  The fixture counts
//! its real covered `C` frame, then holds the test-only observer immediately
//! before the first compaction-temp `sync_all`. Record `T` was already
//! appended after the checkpoint cut, and a later HTTP write `U` must finish
//! before that initial durable I/O resumes. The first cold checkpoint
//! therefore contains only `C`; exact AOF suffix frames `T` and `U` must then
//! replay in order and restore their distinct documents.
//!
//! The observer is a test-only in-process callback.  It receives neither an
//! HTTP value nor a path, and it only pauses the actual initial temp-sync
//! branch after the stable prefix has been copied.
//! The parent test owns a child process and kills and reaps it only when a
//! fixture watchdog fires.  Those bounds clean up a deliberate interleaving;
//! they are not request-latency limits.
//!
//! # Facets
//!
//! - Behavior: `run_covered_trim_case` requires the later HTTP request to
//!   return `200 OK` before the real initial temp-sync pause releases at
//!   :688-692. It
//!   then requires the real writer to retain exactly `[T, U]` at :715-719 and
//!   requires a cold `CURRENT(C)` plus strict AOF replay to recover all three
//!   distinct IDs at :721-774. This is the black-box oracle for moving only
//!   the stable covered-prefix copy and its first temp `sync_all` out of
//!   `apps/lumen/src/segment_checkpoint.rs:180-187`'s current `SharedAof`
//!   critical section.
//! - Security: the changed reader consumes the process-written AOF file.  The
//!   exact suffix and cold replay assertions at :715-774 reject a publication
//!   that drops or reorders an acknowledged later record.  The new observer
//!   has no caller-controlled path, environment, or wire input.  Existing
//!   malformed-AOF refusal remains covered by
//!   `apps/lumen/e2e/segment_startup_fail_closed_e2e.rs:907-947`, registered
//!   by the default `cargo test -p lumen` gate.
//! - Performance: `apps/lumen/docs/indexing.md:264-276` promises, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget."
//!   `assert_pending_budget` at :498-512 reads public `/metrics` after the
//!   complete interleaving and requires total and high-water bytes within that
//!   bound at :775. This case asserts no timing budget; its channel waits are
//!   fixture cleanup only.  The approved request-latency gate is the sixteen
//!   cell workload in `apps/lumen/e2e/perf_gate.rs:4182-4226`.
//!
//! # Root negative control
//!
//! Restore the current one-phase call in
//! `apps/lumen/src/segment_checkpoint.rs:180-187`, so `SharedAof` remains
//! locked while `FramedLogWriter` performs the initial compaction-temp sync.
//! The exact pre-release assertion at :688-692 must fail because `U` reaches
//! the WAL but cannot persist and acknowledge until the initial temp sync
//! releases. Restore
//! the accepted source bytes by SHA-256 before any follow-up gate.
//!
//! Gate: `cargo test --locked -p lumen --test aof_trim_append_progress -- --nocapture`.
//! The target is normal and must also run under `--features jieba`.

use std::fmt;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc, Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{json, Value};

use lumen::aof::{replay_aof_into, AofReader, AofWriter, FramedLogTrimObserver};
use lumen::api::{router, AppState, CheckpointSink};
use lumen::auth::AuthConfig;
use lumen::coordinator::{SharedAof, WriteCoordinator, WriteSink};
use lumen::segment_checkpoint::SegmentCheckpointSink;
use lumen::segment_rdb::{MergeObserver, MergePhase, SegmentRdbStore};
use lumen::storage::Engine;
use lumen::wal::{MemWal, SharedWal, WalLog};
use storage_durable::{CommitStep, FailureInjector, FailurePoint};

const COLLECTION: &str = "aof-trim-append-progress";
const FIELD: &str = "kw";
const BASE_ID: &str = "aof-trim-base-id";
const COVERED_ID: &str = "aof-trim-covered-id";
const FIRST_TAIL_ID: &str = "aof-trim-first-tail-id";
const LATER_TAIL_ID: &str = "aof-trim-later-tail-id";
const BASE_VALUE: &str = "aof-trim-base-value";
const COVERED_VALUE: &str = "aof-trim-covered-value";
const FIRST_TAIL_VALUE: &str = "aof-trim-first-tail-value";
const LATER_TAIL_VALUE: &str = "aof-trim-later-tail-value";
const READY_WATCHDOG: Duration = Duration::from_secs(10);
const INTERLEAVING_WATCHDOG: Duration = Duration::from_secs(10);
const FINISH_WATCHDOG: Duration = Duration::from_secs(30);
const CHILD_WATCHDOG: Duration = Duration::from_secs(120);
const POLL_INTERVAL: Duration = Duration::from_millis(20);
const SUBMIT_FLUSH_DEADLINE: Duration = Duration::from_secs(1);
const PENDING_HARD_LIMIT_BYTES: u64 = 256 * 1024 * 1024;
const CHILD_MODE_ENV: &str = "LUMEN_AOF_TRIM_APPEND_PROGRESS_CHILD";
const CHILD_HANDSHAKE_ENV: &str = "LUMEN_AOF_TRIM_APPEND_PROGRESS_HANDSHAKE";
const TEST_NAME: &str = "covered_trim_releases_later_http_append_and_retains_exact_suffix";

#[derive(Default)]
struct NoopMergeObserver;

impl MergeObserver for NoopMergeObserver {
    fn observe(&self, _: MergePhase) -> io::Result<()> {
        Ok(())
    }
}

/// Holds one production `SyncFile` call after the checkpoint has captured its
/// cut.  The first tail record is therefore unambiguously post-cut before the
/// real trim starts.
#[derive(Default)]
struct CheckpointSyncHold {
    hold: Mutex<Option<(mpsc::SyncSender<()>, mpsc::Receiver<()>)>>,
}

impl CheckpointSyncHold {
    fn arm(&self, entered: mpsc::SyncSender<()>, release: mpsc::Receiver<()>) {
        assert!(
            self.hold
                .lock()
                .expect("checkpoint SyncFile hold mutex")
                .replace((entered, release))
                .is_none(),
            "fixture may hold only one real SyncFile operation",
        );
    }
}

impl FailureInjector for CheckpointSyncHold {
    fn check(&self, point: &FailurePoint) -> io::Result<()> {
        if point.step != CommitStep::SyncFile {
            return Ok(());
        }
        let Some((entered, release)) = self
            .hold
            .lock()
            .expect("checkpoint SyncFile hold mutex")
            .take()
        else {
            return Ok(());
        };
        entered.send(()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                "checkpoint SyncFile readiness receiver dropped",
            )
        })?;
        release.recv().map_err(|_| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                "checkpoint SyncFile release sender dropped",
            )
        })?;
        Ok(())
    }
}

/// The future shared observer counts the actual covered scan frame, then
/// pauses only at the first compaction-temp sync. It is deliberately
/// in-process and has no path, environment, or HTTP configuration surface.
#[derive(Default)]
struct PauseCoveredFrame {
    expected: Mutex<Option<CoveredFrameExpectation>>,
    temp_sync: Mutex<Option<TempSyncPause>>,
    background_sync: Mutex<Option<BackgroundSyncPause>>,
    exact_hits: AtomicUsize,
    unexpected_hits: AtomicUsize,
    temp_sync_hits: AtomicUsize,
}

struct CoveredFrameExpectation {
    through: u64,
    sequence: u64,
}

struct TempSyncPause {
    through: u64,
    entered: mpsc::SyncSender<()>,
    release: mpsc::Receiver<()>,
}

struct BackgroundSyncPause {
    entered: mpsc::SyncSender<()>,
    release: mpsc::Receiver<()>,
}

impl fmt::Debug for PauseCoveredFrame {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PauseCoveredFrame")
            .field("exact_hits", &self.exact_hits.load(Ordering::Acquire))
            .field(
                "unexpected_hits",
                &self.unexpected_hits.load(Ordering::Acquire),
            )
            .field(
                "temp_sync_hits",
                &self.temp_sync_hits.load(Ordering::Acquire),
            )
            .finish_non_exhaustive()
    }
}

impl PauseCoveredFrame {
    fn arm(
        &self,
        through: u64,
        sequence: u64,
        entered: mpsc::SyncSender<()>,
        release: mpsc::Receiver<()>,
    ) {
        assert!(
            self.expected
                .lock()
                .expect("trim observer expectation mutex")
                .replace(CoveredFrameExpectation { through, sequence })
                .is_none(),
            "fixture may arm only one covered-frame expectation",
        );
        assert!(
            self.temp_sync
                .lock()
                .expect("trim temp-sync observer mutex")
                .replace(TempSyncPause {
                    through,
                    entered,
                    release,
                })
                .is_none(),
            "fixture may arm only one real initial temp-sync pause",
        );
    }

    fn arm_background_sync(&self, entered: mpsc::SyncSender<()>, release: mpsc::Receiver<()>) {
        assert!(
            self.background_sync
                .lock()
                .expect("trim background-sync observer mutex")
                .replace(BackgroundSyncPause { entered, release })
                .is_none(),
            "fixture may arm only one background-sync pause",
        );
    }
}

impl FramedLogTrimObserver for PauseCoveredFrame {
    fn covered_frame(&self, through: u64, sequence: u64) {
        match self
            .expected
            .lock()
            .expect("trim observer expectation mutex")
            .as_ref()
        {
            Some(expected) if expected.through == through && expected.sequence == sequence => {
                self.exact_hits.fetch_add(1, Ordering::AcqRel);
            }
            Some(_) => {
                self.unexpected_hits.fetch_add(1, Ordering::AcqRel);
            }
            None => {}
        }
    }

    fn before_temp_sync(&self, through: u64) {
        let pause = {
            let mut state = self
                .temp_sync
                .lock()
                .expect("trim temp-sync observer mutex");
            match state.as_ref() {
                Some(expected) if expected.through == through => state.take(),
                Some(_) => {
                    self.unexpected_hits.fetch_add(1, Ordering::AcqRel);
                    None
                }
                None => None,
            }
        };
        let Some(pause) = pause else {
            return;
        };
        self.temp_sync_hits.fetch_add(1, Ordering::AcqRel);
        let _ = pause.entered.send(());
        let _ = pause.release.recv();
    }

    fn before_background_sync(&self) {
        let pause = self
            .background_sync
            .lock()
            .expect("trim background-sync observer mutex")
            .take();
        let Some(pause) = pause else {
            return;
        };
        let _ = pause.entered.send(());
        let _ = pause.release.recv();
    }
}

/// Releases a deliberately blocked production callback during ordinary
/// completion and test unwinding.
struct ReleaseOnDrop(Option<mpsc::SyncSender<()>>);

impl ReleaseOnDrop {
    fn release(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}

impl Drop for ReleaseOnDrop {
    fn drop(&mut self) {
        self.release();
    }
}

/// Aborts a checkpoint task during a failed child body.  Normal completion
/// takes and joins the handle through `finish_checkpoint` below.
struct CheckpointTask(Option<tokio::task::JoinHandle<anyhow::Result<bool>>>);

impl Drop for CheckpointTask {
    fn drop(&mut self) {
        if let Some(task) = self.0.take() {
            task.abort();
        }
    }
}

struct Fixture {
    _dir: tempfile::TempDir,
    aof_path: PathBuf,
    aof: SharedAof,
    server: Arc<TestServer>,
    store: Arc<SegmentRdbStore>,
    checkpoint: Arc<SegmentCheckpointSink>,
    writer: Arc<WriteCoordinator>,
    wal: Arc<MemWal>,
    sync_hold: Arc<CheckpointSyncHold>,
    trim_pause: Arc<PauseCoveredFrame>,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().expect("create AOF trim fixture directory");
    let root = dir.path().join("segments");
    let sync_hold = Arc::new(CheckpointSyncHold::default());
    let trim_pause = Arc::new(PauseCoveredFrame::default());
    let store = Arc::new(
        SegmentRdbStore::with_failure_injector_and_merge_observer(
            &root,
            sync_hold.clone(),
            Arc::new(NoopMergeObserver),
        )
        .expect("open production segment store"),
    );
    let aof_path = dir.path().join("aof.log");
    let aof: SharedAof = Arc::new(Mutex::new(
        AofWriter::open(&aof_path)
            .expect("open production AOF")
            .with_trim_observer(trim_pause.clone()),
    ));
    let engine = Arc::new(Engine::new());
    let wal = Arc::new(MemWal::new());
    let shared_wal: SharedWal = wal.clone();
    let writer = WriteCoordinator::start_from_with_aof(shared_wal, engine.clone(), 0, aof.clone());
    let sink_writer: Arc<dyn WriteSink> = writer.clone();
    let checkpoint = Arc::new(SegmentCheckpointSink {
        engine: engine.clone(),
        store: store.clone(),
        writer: sink_writer.clone(),
        aof: Some(aof.clone()),
    });
    let checkpoint_api: Arc<dyn CheckpointSink> = checkpoint.clone();
    let state = AppState::with_components(engine, Arc::new(AuthConfig::open()), sink_writer)
        .with_checkpoint(checkpoint_api);
    Fixture {
        _dir: dir,
        aof_path,
        aof,
        server: Arc::new(TestServer::new(router(state)).expect("open production HTTP server")),
        store,
        checkpoint,
        writer,
        wal,
        sync_hold,
        trim_pause,
    }
}

async fn create_keyword_collection(server: &TestServer) {
    server
        .put(&format!("/collections/{COLLECTION}"))
        .json(&json!({ "fields": { FIELD: { "type": "keyword" } } }))
        .await
        .assert_status_ok();
}

async fn index_keyword(server: &TestServer, external_id: &str, value: &str) {
    server
        .post(&format!("/collections/{COLLECTION}/index"))
        .json(&json!({ "items": [{
            "external_id": external_id,
            "field": FIELD,
            "value": value,
        }] }))
        .await
        .assert_status_ok();
}

async fn post_keyword_status(
    server: Arc<TestServer>,
    external_id: &'static str,
    value: &'static str,
) -> StatusCode {
    server
        .post(&format!("/collections/{COLLECTION}/index"))
        .json(&json!({ "items": [{
            "external_id": external_id,
            "field": FIELD,
            "value": value,
        }] }))
        .await
        .status_code()
}

async fn assert_term_ids(server: &TestServer, value: &str, expected: &[&str], context: &str) {
    let response = server
        .post(&format!("/collections/{COLLECTION}/search"))
        .json(&json!({
            "query": { "term": { "field": FIELD, "value": value } },
            "limit": 16,
        }))
        .await;
    response.assert_status_ok();
    let mut actual = response.json::<Value>()["hits"]
        .as_array()
        .expect("exact Keyword query hits")
        .iter()
        .map(|hit| {
            hit["external_id"]
                .as_str()
                .expect("Keyword hit external ID")
                .to_owned()
        })
        .collect::<Vec<_>>();
    actual.sort();
    let mut expected = expected
        .iter()
        .map(|id| (*id).to_owned())
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(
        actual, expected,
        "{context}: exact Keyword query must retain only expected IDs",
    );
}

async fn checkpoint_success(checkpoint: &SegmentCheckpointSink, context: &str) {
    let persisted = CheckpointSink::checkpoint_now(checkpoint)
        .await
        .unwrap_or_else(|error| panic!("{context}: production checkpoint failed: {error:#}"));
    assert!(persisted, "{context}: configured checkpoint must persist");
}

fn sync_aof(fixture: &Fixture, context: &str) {
    fixture
        .aof
        .lock()
        .expect("AOF writer lock")
        .sync_strict()
        .unwrap_or_else(|error| panic!("{context}: sync AOF bytes: {error:#}"));
}

fn aof_sequences(path: &Path, context: &str) -> Vec<u64> {
    let mut sequences = Vec::new();
    AofReader::replay(path, 0, |sequence, _| sequences.push(sequence))
        .unwrap_or_else(|error| panic!("{context}: read AOF frames: {error:#}"));
    sequences
}

fn cold_engine(fixture: &Fixture, sequence: u64, context: &str) -> Arc<Engine> {
    let loaded = fixture
        .store
        .load_current_generation()
        .unwrap_or_else(|error| panic!("{context}: cold-open CURRENT: {error:#}"))
        .unwrap_or_else(|| panic!("{context}: expected a durable CURRENT generation"));
    assert_eq!(
        loaded.sequence, sequence,
        "{context}: CURRENT watermark must equal captured durable cut",
    );
    loaded.engine
}

fn cold_server(engine: Arc<Engine>, sequence: u64) -> TestServer {
    let writer: Arc<dyn WriteSink> =
        WriteCoordinator::start_from(Arc::new(MemWal::new()), engine.clone(), sequence);
    TestServer::new(router(AppState::with_components(
        engine,
        Arc::new(AuthConfig::open()),
        writer,
    )))
    .expect("open cold recovery HTTP server")
}

fn metric_u64(metrics: &str, name: &str) -> u64 {
    let values = metrics
        .lines()
        .filter_map(|line| {
            let (metric, value) = line.split_once(|byte: char| byte.is_whitespace())?;
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

async fn assert_pending_budget(server: &TestServer, context: &str) {
    let response = server.get("/metrics").await;
    response.assert_status_ok();
    let metrics = response.text();
    let total = metric_u64(&metrics, "lumen_pending_change_total_bytes");
    let high_water = metric_u64(&metrics, "lumen_pending_change_high_water_bytes");
    assert!(
        total <= PENDING_HARD_LIMIT_BYTES,
        "{context}: public pending total must stay within documented 256 MiB budget: total={total}",
    );
    assert!(
        high_water <= PENDING_HARD_LIMIT_BYTES,
        "{context}: public pending high water must stay within documented 256 MiB budget: high_water={high_water}",
    );
}

async fn wait_for_wal_sequence(wal: &MemWal, sequence: u64, context: &str) {
    let result = tokio::time::timeout(INTERLEAVING_WATCHDOG, async {
        loop {
            if wal.latest_seq().await.expect("read MemWal sequence") >= sequence {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await;
    assert!(
        result.is_ok(),
        "{context}: later HTTP write did not reach the real WAL before cleanup",
    );
}

async fn start_held_checkpoint(
    fixture: &Fixture,
    context: &str,
) -> (CheckpointTask, ReleaseOnDrop) {
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    fixture.sync_hold.arm(entered_tx, release_rx);
    let mut release = ReleaseOnDrop(Some(release_tx));
    let mut task = tokio::spawn({
        let checkpoint = fixture.checkpoint.clone();
        async move { CheckpointSink::checkpoint_now(checkpoint.as_ref()).await }
    });
    let entered = tokio::task::spawn_blocking(move || entered_rx.recv_timeout(READY_WATCHDOG))
        .await
        .expect("checkpoint SyncFile readiness task joins");
    if !matches!(entered, Ok(())) {
        release.release();
        task.abort();
        let _ = task.await;
        panic!("{context}: checkpoint did not reach real SyncFile hold: {entered:?}");
    }
    (CheckpointTask(Some(task)), release)
}

async fn wait_for_temp_sync_pause(
    entered_rx: mpsc::Receiver<()>,
    trim_release: &mut ReleaseOnDrop,
    checkpoint: &mut CheckpointTask,
    context: &str,
) {
    let entered = tokio::task::spawn_blocking(move || entered_rx.recv_timeout(READY_WATCHDOG))
        .await
        .expect("trim readiness task joins");
    if !matches!(entered, Ok(())) {
        trim_release.release();
        if let Some(task) = checkpoint.0.take() {
            task.abort();
            let _ = task.await;
        }
        panic!("{context}: trim did not reach real initial temp-sync pause: {entered:?}");
    }
}

async fn wait_for_background_sync_pause(
    entered_rx: mpsc::Receiver<()>,
    trim_release: &mut ReleaseOnDrop,
    checkpoint: &mut CheckpointTask,
    context: &str,
) {
    let entered = tokio::task::spawn_blocking(move || entered_rx.recv_timeout(READY_WATCHDOG))
        .await
        .expect("background-sync readiness task joins");
    if !matches!(entered, Ok(())) {
        trim_release.release();
        if let Some(task) = checkpoint.0.take() {
            task.abort();
            let _ = task.await;
        }
        panic!("{context}: background sync did not reach observer hook: {entered:?}");
    }
}

async fn finish_checkpoint(mut task: CheckpointTask, context: &str) {
    let mut task = task.0.take().expect("checkpoint task remains owned");
    let joined = match tokio::time::timeout(FINISH_WATCHDOG, &mut task).await {
        Ok(joined) => joined,
        Err(_) => {
            task.abort();
            let _ = task.await;
            panic!("{context}: checkpoint exceeded fixture cleanup watchdog");
        }
    };
    let persisted = joined
        .unwrap_or_else(|error| panic!("{context}: checkpoint task panicked: {error}"))
        .unwrap_or_else(|error| panic!("{context}: checkpoint failed: {error:#}"));
    assert!(persisted, "{context}: configured checkpoint must persist");
}

async fn run_covered_trim_case() {
    let fixture = fixture();
    create_keyword_collection(&fixture.server).await;
    index_keyword(&fixture.server, BASE_ID, BASE_VALUE).await;
    let base_sequence = fixture.writer.applied_seq();
    checkpoint_success(&fixture.checkpoint, "publish durable base checkpoint").await;
    sync_aof(&fixture, "inspect durable base AOF");
    assert_eq!(
        aof_sequences(&fixture.aof_path, "durable base AOF"),
        Vec::<u64>::new(),
        "durable base checkpoint must reclaim its fully covered AOF frame",
    );

    index_keyword(&fixture.server, COVERED_ID, COVERED_VALUE).await;
    let covered_sequence = fixture.writer.applied_seq();
    assert_eq!(
        covered_sequence,
        base_sequence + 1,
        "covered checkpoint record must be the next real coordinator sequence",
    );

    let (mut checkpoint, mut sync_release) =
        start_held_checkpoint(&fixture, "hold checkpoint after covered record capture").await;
    index_keyword(&fixture.server, FIRST_TAIL_ID, FIRST_TAIL_VALUE).await;
    let first_tail_sequence = fixture.writer.applied_seq();
    assert_eq!(
        first_tail_sequence,
        covered_sequence + 1,
        "first tail record must arrive after captured record without synthetic ordering",
    );
    sync_aof(&fixture, "inspect AOF before covered-prefix trim");
    assert_eq!(
        aof_sequences(&fixture.aof_path, "pre-trim AOF"),
        vec![covered_sequence, first_tail_sequence],
        "real trim source must contain its covered frame and first retained suffix",
    );

    let (trim_entered_tx, trim_entered_rx) = mpsc::sync_channel(1);
    let (trim_release_tx, trim_release_rx) = mpsc::sync_channel(1);
    fixture.trim_pause.arm(
        covered_sequence,
        covered_sequence,
        trim_entered_tx,
        trim_release_rx,
    );
    let mut trim_release = ReleaseOnDrop(Some(trim_release_tx));
    sync_release.release();
    wait_for_temp_sync_pause(
        trim_entered_rx,
        &mut trim_release,
        &mut checkpoint,
        "hold real initial AOF compaction temp sync after covered scan",
    )
    .await;

    let (background_entered_tx, background_entered_rx) = mpsc::sync_channel(1);
    let (background_release_tx, background_release_rx) = mpsc::sync_channel(1);
    fixture
        .trim_pause
        .arm_background_sync(background_entered_tx, background_release_rx);
    let mut background_release = ReleaseOnDrop(Some(background_release_tx));
    wait_for_background_sync_pause(
        background_entered_rx,
        &mut trim_release,
        &mut checkpoint,
        "observe real background AOF sync before later append",
    )
    .await;
    let later_tail_sequence = first_tail_sequence + 1;
    let later_http = post_keyword_status(fixture.server.clone(), LATER_TAIL_ID, LATER_TAIL_VALUE);
    tokio::pin!(later_http);
    let wal_ready = wait_for_wal_sequence(
        fixture.wal.as_ref(),
        later_tail_sequence,
        "later HTTP write while initial trim temp sync pauses",
    );
    tokio::pin!(wal_ready);
    let response_before_wal = tokio::select! {
        () = &mut wal_ready => None,
        status = &mut later_http => Some(status),
    };
    if response_before_wal.is_some() {
        (&mut wal_ready).await;
    }
    assert_eq!(
        fixture
            .wal
            .latest_seq()
            .await
            .expect("read later-tail MemWal sequence"),
        later_tail_sequence,
        "later record must retain its real WAL sequence before response ordering is checked",
    );
    let response_before_release = match response_before_wal {
        Some(status) => Some(status),
        None => tokio::time::timeout(SUBMIT_FLUSH_DEADLINE, &mut later_http)
            .await
            .ok(),
    };

    // Release first so the child can cleanly finish if this deliberately-red
    // assertion fails against the current one-phase temp-sync lock.
    trim_release.release();
    assert_eq!(
        response_before_release,
        Some(StatusCode::OK),
        "later HTTP write must return 200 while background sync is held at its pre-sync hook",
    );
    background_release.release();
    finish_checkpoint(checkpoint, "finish checkpoint after later append").await;
    assert_eq!(
        fixture.trim_pause.exact_hits.load(Ordering::Acquire),
        1,
        "fixture must observe exactly the captured covered AOF frame",
    );
    assert_eq!(
        fixture.trim_pause.unexpected_hits.load(Ordering::Acquire),
        0,
        "trim observer must not treat an uncovered suffix as the covered frame",
    );
    assert_eq!(
        fixture.trim_pause.temp_sync_hits.load(Ordering::Acquire),
        1,
        "fixture must pause exactly the real initial compaction temp sync",
    );
    assert_eq!(
        fixture.writer.applied_seq(),
        later_tail_sequence,
        "later acknowledged write must advance the live applied watermark",
    );
    sync_aof(&fixture, "inspect compacted AOF suffix");
    assert_eq!(
        aof_sequences(&fixture.aof_path, "compacted exact suffix"),
        vec![first_tail_sequence, later_tail_sequence],
        "trim must retain exactly both post-cut AOF frames in sequence order",
    );
    let mut restarted_aof =
        AofWriter::open(&fixture.aof_path).expect("restart must reopen the retained AOF suffix");
    restarted_aof
        .sync_strict()
        .expect("restart must strict-sync the retained AOF suffix");

    let cold_engine = cold_engine(
        &fixture,
        covered_sequence,
        "first cold durable checkpoint cut",
    );
    let cold = cold_server(cold_engine.clone(), covered_sequence);
    assert_term_ids(
        &cold,
        COVERED_VALUE,
        &[COVERED_ID],
        "cold CURRENT cut retains covered record",
    )
    .await;
    assert_term_ids(
        &cold,
        FIRST_TAIL_VALUE,
        &[],
        "cold CURRENT cut excludes first post-cut suffix",
    )
    .await;
    assert_term_ids(
        &cold,
        LATER_TAIL_VALUE,
        &[],
        "cold CURRENT cut excludes later concurrent suffix",
    )
    .await;
    assert_eq!(
        replay_aof_into(&cold_engine, &fixture.aof_path, covered_sequence)
            .expect("strict replay exact retained suffix"),
        later_tail_sequence,
        "strict replay must advance through the final later suffix sequence",
    );
    assert_term_ids(
        &cold,
        COVERED_VALUE,
        &[COVERED_ID],
        "strict replay must preserve covered checkpoint document",
    )
    .await;
    assert_term_ids(
        &cold,
        FIRST_TAIL_VALUE,
        &[FIRST_TAIL_ID],
        "strict replay recovers first post-cut document",
    )
    .await;
    assert_term_ids(
        &cold,
        LATER_TAIL_VALUE,
        &[LATER_TAIL_ID],
        "strict replay recovers later concurrent document",
    )
    .await;
    assert_pending_budget(&fixture.server, "completed trim/apply interleaving").await;
}

struct ChildCleanup(Option<Child>);

impl Drop for ChildCleanup {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn run_isolated_child() {
    let workspace = tempfile::tempdir().expect("create parent-owned trim workspace");
    let child_tmp = workspace.path().join("child-tmp");
    fs::create_dir(&child_tmp).expect("create parent-owned child TMPDIR");
    let handshake = workspace.path().join("entered");
    let stdout_path = workspace.path().join("child.stdout");
    let stderr_path = workspace.path().join("child.stderr");
    let executable = std::env::current_exe().expect("locate trim test executable");
    let child = Command::new(executable)
        .env(CHILD_MODE_ENV, "1")
        .env(CHILD_HANDSHAKE_ENV, &handshake)
        .env("TMPDIR", &child_tmp)
        .env("TEMP", &child_tmp)
        .env("TMP", &child_tmp)
        .arg(TEST_NAME)
        .arg("--exact")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .stdout(Stdio::from(
            File::create(&stdout_path).expect("create trim child stdout"),
        ))
        .stderr(Stdio::from(
            File::create(&stderr_path).expect("create trim child stderr"),
        ))
        .spawn()
        .expect("spawn isolated trim child");
    let mut child = ChildCleanup(Some(child));
    let deadline = Instant::now() + CHILD_WATCHDOG;
    loop {
        match child
            .0
            .as_mut()
            .expect("trim child remains owned")
            .try_wait()
        {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => thread::sleep(POLL_INTERVAL),
            Ok(None) => {
                let mut raw = child.0.take().expect("timed-out trim child remains owned");
                let _ = raw.kill();
                let status = raw.wait().expect("reap killed trim child");
                let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
                let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
                panic!(
                    "trim child exceeded fixture cleanup watchdog {CHILD_WATCHDOG:?}; status={status}; stdout={stdout}; stderr={stderr}",
                );
            }
            Err(error) => panic!("poll trim child: {error}"),
        }
    }
    let status = child
        .0
        .take()
        .expect("exited trim child remains owned")
        .wait()
        .expect("wait for trim child");
    let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
    let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
    let entered = fs::read_to_string(&handshake).unwrap_or_else(|error| {
        panic!("trim child did not enter intended body: {error}; stdout={stdout}; stderr={stderr}",)
    });
    assert_eq!(
        entered, TEST_NAME,
        "trim child must run intended exact test"
    );
    assert!(
        status.success(),
        "trim child failed; stdout={stdout}; stderr={stderr}",
    );
}

#[test]
fn covered_trim_releases_later_http_append_and_retains_exact_suffix() {
    if std::env::var_os(CHILD_MODE_ENV).is_some() {
        let handshake = std::env::var_os(CHILD_HANDSHAKE_ENV)
            .map(PathBuf::from)
            .expect("child handshake path");
        fs::write(&handshake, TEST_NAME).expect("write child entry handshake");
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("build trim child Tokio runtime")
            .block_on(run_covered_trim_case());
        return;
    }
    run_isolated_child();
}
