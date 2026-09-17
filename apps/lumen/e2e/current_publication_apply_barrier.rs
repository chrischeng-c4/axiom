//! Black-box contract for releasing the Lumen apply boundary before durable
//! CURRENT pointer file I/O.
//!
//! The child checkpoints one collection after a captured value. It pauses at
//! each real pointer durability call: SyncCurrentTemp before the atomic
//! pointer rename, and SyncRootAfterCurrent after the rename and before the
//! root directory sync. While that call is held, a real HTTP index write to a
//! different collection must enter the WAL and return success. The first cold
//! open is the captured cut and excludes the later value. Its real AOF tail
//! must replay that exact value before a second checkpoint and cold open retain
//! it without replay.
//! A separate child makes four public deltas for one field, waits for the
//! existing background merge `BeforePublish` observer, and only then arms the
//! same pointer pause. It proves the pause belongs to the merge generation,
//! then requires the other collection's retained post-cut record to survive
//! the merge cut, AOF replay, and a final cold checkpoint.
//!
//! The parent owns the child workspace. Its watchdog kills and reaps only a
//! stuck child before it removes that workspace. It is cleanup safety, not a
//! request-latency target.
//!
//! # Facets
//!
//! - Behavior: current_publication_apply_barrier.rs:499-590 requires the
//!   ordinary checkpoint's post-cut HTTP write to enter the real WAL and
//!   return before either held CURRENT I/O operation releases. Lines 594-670
//!   require its first cold CURRENT to retain the captured watermark and
//!   exclude that value, then replay the surviving post-cut AOF record before
//!   requiring the second cold CURRENT to retain it. Lines 973-1092 use the
//!   existing `MergePhase::BeforePublish` observer to prove four completed
//!   foreground checkpoints selected a real one-field background merge before
//!   the same pointer hold is armed. Lines 1094-1165 require that merge's cold
//!   cut, its retained AOF suffix, and the final cold cut. These assertions
//!   exercise apps/lumen/src/segment_rdb.rs:839-885,
//!   apps/lumen/src/segment_background_merge.rs:677-810, and the actual
//!   pointer calls at libs/storage-durable/src/generation.rs:1135-1194.
//! - Security: this change changes only the timing of process-written CURRENT
//!   publication in apps/lumen/src/segment_rdb.rs:839-885 and
//!   apps/lumen/src/segment_background_merge.rs:769-810. It opens no new
//!   caller path, parser, identifier, or disk-input boundary. The cold/AOF
//!   checks at current_publication_apply_barrier.rs:1094-1165 refuse a
//!   publication that loses the later committed record. The existing durable
//!   restore race assertion in apps/lumen/e2e/backup_restore_e2e.rs:342-433
//!   keeps an old checkpoint from replacing an observed candidate CURRENT, and
//!   the default gate runs that target.
//! - Performance: this deterministic ordering case makes no latency claim;
//!   its waits only clean up a held interleaving. The approved durable-workload
//!   gate carries this path's performance account: the current limits are 100
//!   docops/s, 10 QPS, p99 at most one second, and any request at most five
//!   seconds in apps/lumen/e2e/support/perf_workload_ledger.rs:118-124.
//!   apps/lumen/e2e/perf_gate.rs:4182-4226 runs all sixteen 30-minute cells,
//!   including checkpoint and merge observations. Its release command is
//!   `cargo test -p lumen --test perf_gate -- --ignored`.
//!
//! # Root negative control
//!
//! After the repair, retain the original CaptureLease through the merge
//! pointer calls in apps/lumen/src/segment_background_merge.rs:769-790
//! instead of converting it to the short publication pin. The assertions at
//! current_publication_apply_barrier.rs:1076-1086 must fail because the
//! merge-interleaved HTTP write cannot report success before release. Restore
//! the corrected production SHA before the next gate.
//!
//! Gate: cargo test -p lumen --test current_publication_apply_barrier -- --nocapture.
//! Full declared behavior gate: cargo test -p lumen.

use std::fs::{self, File};
use std::io;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{json, Value};

use lumen::aof::{replay_aof_into, AofWriter};
use lumen::api::{router, AppState, CheckpointSink};
use lumen::auth::AuthConfig;
use lumen::coordinator::{SharedAof, WriteCoordinator, WriteSink};
use lumen::segment_checkpoint::SegmentCheckpointSink;
use lumen::segment_rdb::{MergeObserver, MergePhase, SegmentRdbStore};
use lumen::storage::Engine;
use lumen::wal::{MemWal, SharedWal, WalLog};
use storage_durable::{CommitStep, FailureInjector, FailurePoint};

const CAPTURED_COLLECTION: &str = "current-publication-captured";
const POSTCUT_COLLECTION: &str = "current-publication-postcut";
const FIELD: &str = "kw";
const CAPTURED_BASE_ID: &str = "captured-base-id";
const POSTCUT_BASE_ID: &str = "postcut-base-id";
const CAPTURED_ID: &str = "captured-cut-id";
const POSTCUT_ID: &str = "postcut-after-current-id";
const CAPTURED_BASE_VALUE: &str = "captured-base-value";
const POSTCUT_BASE_VALUE: &str = "postcut-base-value";
const CAPTURED_VALUE: &str = "captured-value-in-first-durable-cut";
const POSTCUT_VALUE: &str = "postcut-value-must-remain-in-wal";
const READY_WATCHDOG: Duration = Duration::from_secs(30);
const INTERLEAVING_WATCHDOG: Duration = Duration::from_secs(30);
const FINISH_WATCHDOG: Duration = Duration::from_secs(30);
const CHILD_WATCHDOG: Duration = Duration::from_secs(120);
const POLL_INTERVAL: Duration = Duration::from_millis(20);
const CHILD_MODE_ENV: &str = "LUMEN_CURRENT_PUBLICATION_CHILD";
const CHILD_HANDSHAKE_ENV: &str = "LUMEN_CURRENT_PUBLICATION_HANDSHAKE";
const TEST_NAME: &str = "current_pointer_io_releases_apply_and_preserves_durable_cuts";

#[derive(Clone, Copy, Debug)]
enum PointerIoStep {
    CurrentTemp,
    RootAfterCurrent,
}

impl PointerIoStep {
    fn commit_step(self) -> CommitStep {
        match self {
            Self::CurrentTemp => CommitStep::SyncCurrentTemp,
            Self::RootAfterCurrent => CommitStep::SyncRootAfterCurrent,
        }
    }

    fn expected_relative_path(self) -> PathBuf {
        match self {
            Self::CurrentTemp => PathBuf::from("CURRENT.tmp"),
            Self::RootAfterCurrent => PathBuf::from("."),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::CurrentTemp => "SyncCurrentTemp",
            Self::RootAfterCurrent => "SyncRootAfterCurrent",
        }
    }
}

#[derive(Default)]
struct NoopMergeObserver;

impl MergeObserver for NoopMergeObserver {
    fn observe(&self, _: MergePhase) -> io::Result<()> {
        Ok(())
    }
}

struct PointerIoPause {
    step: PointerIoStep,
    entered: mpsc::SyncSender<()>,
    release: mpsc::Receiver<()>,
}

/// Holds exactly one expected pointer operation. A different file operation
/// cannot satisfy the readiness channel.
#[derive(Default)]
struct HoldPointerIo {
    pause: Mutex<Option<PointerIoPause>>,
}

impl HoldPointerIo {
    fn arm(&self, step: PointerIoStep, entered: mpsc::SyncSender<()>, release: mpsc::Receiver<()>) {
        assert!(
            self.pause
                .lock()
                .expect("pointer I/O pause mutex")
                .replace(PointerIoPause {
                    step,
                    entered,
                    release,
                })
                .is_none(),
            "fixture arms exactly one CURRENT I/O pause",
        );
    }
}

impl FailureInjector for HoldPointerIo {
    fn check(&self, point: &FailurePoint) -> io::Result<()> {
        let pause = {
            let mut state = self.pause.lock().expect("pointer I/O pause mutex");
            if state
                .as_ref()
                .is_none_or(|pause| pause.step.commit_step() != point.step)
            {
                None
            } else {
                state.take()
            }
        };
        let Some(pause) = pause else {
            return Ok(());
        };
        if point.relative_path != pause.step.expected_relative_path() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "expected {} at {:?}, got {:?}",
                    pause.step.label(),
                    pause.step.expected_relative_path(),
                    point.relative_path,
                ),
            ));
        }
        pause.entered.send(()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                "CURRENT I/O readiness receiver dropped",
            )
        })?;
        pause.release.recv().map_err(|_| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                "CURRENT I/O release sender dropped",
            )
        })?;
        Ok(())
    }
}

/// Releases the pause on success and panic paths.
struct PointerIoRelease(Option<mpsc::SyncSender<()>>);

impl PointerIoRelease {
    fn release(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}

impl Drop for PointerIoRelease {
    fn drop(&mut self) {
        self.release();
    }
}

struct Fixture {
    server: Arc<TestServer>,
    store: Arc<SegmentRdbStore>,
    checkpoint: Arc<SegmentCheckpointSink>,
    writer: Arc<WriteCoordinator>,
    wal: Arc<MemWal>,
    aof_path: PathBuf,
    pointer_hold: Arc<HoldPointerIo>,
    // Drop the driver-facing handles before removing the child-owned files.
    _dir: tempfile::TempDir,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().expect("create current-publication fixture directory");
    let pointer_hold = Arc::new(HoldPointerIo::default());
    let store = Arc::new(
        SegmentRdbStore::with_failure_injector_and_merge_observer(
            dir.path().join("segments"),
            pointer_hold.clone(),
            Arc::new(NoopMergeObserver),
        )
        .expect("open failure-injected segment store"),
    );
    let aof_path = dir.path().join("aof.log");
    let aof: SharedAof = Arc::new(Mutex::new(
        AofWriter::open(&aof_path).expect("open current-publication AOF"),
    ));
    let engine = Arc::new(Engine::new());
    let wal = Arc::new(MemWal::new());
    let shared_wal: SharedWal = wal.clone();
    let writer = WriteCoordinator::start_from_with_aof(shared_wal, engine.clone(), 0, aof.clone());
    let writer_sink: Arc<dyn WriteSink> = writer.clone();
    let checkpoint = Arc::new(SegmentCheckpointSink {
        engine: engine.clone(),
        store: store.clone(),
        writer: writer_sink.clone(),
        aof: Some(aof),
    });
    let checkpoint_api: Arc<dyn CheckpointSink> = checkpoint.clone();
    let state = AppState::with_components(engine, Arc::new(AuthConfig::open()), writer_sink)
        .with_checkpoint(checkpoint_api);
    Fixture {
        server: Arc::new(TestServer::new(router(state)).expect("open current-publication server")),
        store,
        checkpoint,
        writer,
        wal,
        aof_path,
        pointer_hold,
        _dir: dir,
    }
}

async fn create_keyword_collection(server: &TestServer, collection: &str) {
    server
        .put(&format!("/collections/{collection}"))
        .json(&json!({ "fields": { FIELD: { "type": "keyword" } } }))
        .await
        .assert_status_ok();
}

async fn index_keyword(server: &TestServer, collection: &str, external_id: &str, value: &str) {
    server
        .post(&format!("/collections/{collection}/index"))
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
    collection: &'static str,
    external_id: &'static str,
    value: &'static str,
) -> StatusCode {
    server
        .post(&format!("/collections/{collection}/index"))
        .json(&json!({ "items": [{
            "external_id": external_id,
            "field": FIELD,
            "value": value,
        }] }))
        .await
        .status_code()
}

async fn term_ids(server: &TestServer, collection: &str, value: &str) -> Vec<String> {
    let response = server
        .post(&format!("/collections/{collection}/search"))
        .json(&json!({
            "query": { "term": { "field": FIELD, "value": value } },
            "limit": 16,
        }))
        .await;
    response.assert_status_ok();
    let mut ids = response.json::<Value>()["hits"]
        .as_array()
        .expect("exact term-query hits")
        .iter()
        .map(|hit| {
            hit["external_id"]
                .as_str()
                .expect("exact term-query external ID")
                .to_owned()
        })
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

async fn assert_term_ids(
    server: &TestServer,
    collection: &str,
    value: &str,
    expected: &[&str],
    context: &str,
) {
    let mut expected = expected
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(
        term_ids(server, collection, value).await,
        expected,
        "{context}: exact Keyword query must retain only expected IDs",
    );
}

async fn checkpoint_now(checkpoint: &SegmentCheckpointSink, context: &str) {
    assert!(
        CheckpointSink::checkpoint_now(checkpoint)
            .await
            .unwrap_or_else(|error| panic!("{context}: real checkpoint failed: {error:#}")),
        "{context}: configured checkpoint must persist",
    );
}

async fn wait_for_wal_sequence(wal: &MemWal, sequence: u64, context: &str) {
    let wait = tokio::time::timeout(INTERLEAVING_WATCHDOG, async {
        loop {
            if wal
                .latest_seq()
                .await
                .expect("read current-publication MemWal sequence")
                >= sequence
            {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await;
    assert!(
        wait.is_ok(),
        "{context}: HTTP write did not reach the real WAL before cleanup",
    );
}

fn cold_engine(store: &SegmentRdbStore, sequence: u64, context: &str) -> Arc<Engine> {
    let loaded = store
        .load_current_generation()
        .unwrap_or_else(|error| panic!("{context}: cold-open CURRENT failed: {error:#}"))
        .unwrap_or_else(|| panic!("{context}: checkpoint CURRENT must name a generation"));
    assert_eq!(
        loaded.sequence, sequence,
        "{context}: CURRENT watermark must equal the durable cut",
    );
    loaded.engine
}

fn cold_server(engine: Arc<Engine>, sequence: u64) -> TestServer {
    // Querying a checkpoint must not initialize its apply boundary at zero
    // before the surviving AOF suffix is replayed at the checkpoint cut.
    let writer: Arc<dyn WriteSink> =
        WriteCoordinator::start_from(Arc::new(MemWal::new()), engine.clone(), sequence);
    let state = AppState::with_components(engine, Arc::new(AuthConfig::open()), writer);
    TestServer::new(router(state)).expect("open cold current-publication server")
}

async fn assert_checkpoint_completion(
    checkpoint: &mut tokio::task::JoinHandle<anyhow::Result<bool>>,
    context: &str,
) {
    let result = tokio::time::timeout(FINISH_WATCHDOG, checkpoint)
        .await
        .unwrap_or_else(|_| panic!("{context}: checkpoint exceeded cleanup watchdog"))
        .unwrap_or_else(|error| panic!("{context}: checkpoint task panicked: {error}"))
        .unwrap_or_else(|error| panic!("{context}: checkpoint returned error: {error:#}"));
    assert!(
        result,
        "{context}: configured checkpoint returned persisted=false"
    );
}

async fn run_pointer_io_case(step: PointerIoStep) {
    let fixture = fixture();
    create_keyword_collection(&fixture.server, CAPTURED_COLLECTION).await;
    create_keyword_collection(&fixture.server, POSTCUT_COLLECTION).await;
    index_keyword(
        &fixture.server,
        CAPTURED_COLLECTION,
        CAPTURED_BASE_ID,
        CAPTURED_BASE_VALUE,
    )
    .await;
    index_keyword(
        &fixture.server,
        POSTCUT_COLLECTION,
        POSTCUT_BASE_ID,
        POSTCUT_BASE_VALUE,
    )
    .await;
    checkpoint_now(
        fixture.checkpoint.as_ref(),
        "publish initial current-publication baseline",
    )
    .await;

    index_keyword(
        &fixture.server,
        CAPTURED_COLLECTION,
        CAPTURED_ID,
        CAPTURED_VALUE,
    )
    .await;
    let captured_sequence = fixture.writer.applied_seq();
    assert_eq!(
        fixture
            .wal
            .latest_seq()
            .await
            .expect("read captured MemWal sequence"),
        captured_sequence,
        "{}: captured write must own the WAL head before checkpoint",
        step.label(),
    );

    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    fixture.pointer_hold.arm(step, entered_tx, release_rx);
    let mut release = PointerIoRelease(Some(release_tx));
    let mut paused_checkpoint = tokio::spawn({
        let checkpoint = fixture.checkpoint.clone();
        async move { CheckpointSink::checkpoint_now(checkpoint.as_ref()).await }
    });
    let entered = tokio::task::spawn_blocking(move || entered_rx.recv_timeout(READY_WATCHDOG))
        .await
        .expect("CURRENT I/O readiness task must join");
    if !matches!(entered, Ok(())) {
        release.release();
        paused_checkpoint.abort();
        let _ = paused_checkpoint.await;
        panic!(
            "{}: checkpoint did not reach real CURRENT I/O pause: {entered:?}",
            step.label(),
        );
    }

    let postcut_sequence = captured_sequence + 1;
    // axum_test's request future is deliberately local. Poll it beside the
    // WAL wait instead of sending it to another Tokio worker.
    let postcut_http = post_keyword_status(
        fixture.server.clone(),
        POSTCUT_COLLECTION,
        POSTCUT_ID,
        POSTCUT_VALUE,
    );
    tokio::pin!(postcut_http);
    let wal_context = format!("{} post-cut HTTP write", step.label());
    let wal_ready = wait_for_wal_sequence(fixture.wal.as_ref(), postcut_sequence, &wal_context);
    tokio::pin!(wal_ready);
    let response_before_wal = tokio::select! {
        () = &mut wal_ready => None,
        status = &mut postcut_http => Some(status),
    };
    if response_before_wal.is_some() {
        (&mut wal_ready).await;
    }
    assert_eq!(
        fixture
            .wal
            .latest_seq()
            .await
            .expect("read post-cut MemWal sequence"),
        postcut_sequence,
        "{}: post-cut record must remain the one next WAL record",
        step.label(),
    );

    // This proves order, not a latency limit. The pointer hook has entered and
    // the real WAL contains the post-cut record before this response is tested.
    let response_before_release = match response_before_wal {
        Some(status) => Some(status),
        None => tokio::time::timeout(INTERLEAVING_WATCHDOG, &mut postcut_http)
            .await
            .ok(),
    };

    release.release();
    assert_checkpoint_completion(
        &mut paused_checkpoint,
        &format!("{} paused checkpoint after release", step.label()),
    )
    .await;
    assert_eq!(
        fixture
            .wal
            .latest_seq()
            .await
            .expect("read WAL after first durable cut"),
        postcut_sequence,
        "{}: first durable cut must retain the later committed WAL record",
        step.label(),
    );
    let response_after_release = if response_before_release.is_none() {
        Some(
            tokio::time::timeout(FINISH_WATCHDOG, &mut postcut_http)
                .await
                .unwrap_or_else(|_| {
                    panic!(
                        "{}: retained post-cut HTTP task exceeded cleanup after release",
                        step.label()
                    )
                }),
        )
    } else {
        None
    };
    let response_status = response_before_release
        .or(response_after_release)
        .expect("post-cut HTTP task returns before or after controlled cleanup");
    assert_eq!(
        response_status,
        StatusCode::OK,
        "{}: retained post-cut HTTP request must eventually report success",
        step.label(),
    );
    assert!(
        response_before_release.is_some(),
        "{}: post-cut write that reached WAL must complete before CURRENT I/O releases",
        step.label(),
    );
    assert_eq!(
        fixture.writer.applied_seq(),
        postcut_sequence,
        "{}: only the post-cut record advances the live watermark",
        step.label(),
    );

    let first_cold_engine = cold_engine(
        fixture.store.as_ref(),
        captured_sequence,
        &format!("{} first durable cut", step.label()),
    );
    let first_cold = cold_server(first_cold_engine.clone(), captured_sequence);
    assert_term_ids(
        &first_cold,
        CAPTURED_COLLECTION,
        CAPTURED_VALUE,
        &[CAPTURED_ID],
        &format!("{} first cold cut keeps captured value", step.label()),
    )
    .await;
    assert_term_ids(
        &first_cold,
        POSTCUT_COLLECTION,
        POSTCUT_VALUE,
        &[],
        &format!(
            "{} first cold cut excludes post-cut WAL value",
            step.label()
        ),
    )
    .await;
    let replayed = replay_aof_into(&first_cold_engine, &fixture.aof_path, captured_sequence)
        .unwrap_or_else(|error| {
            panic!(
            "{}: AOF after first checkpoint must replay the retained post-cut record: {error:#}",
            step.label(),
        )
        });
    assert_eq!(
        replayed,
        postcut_sequence,
        "{}: checkpoint truncation must retain the exact post-cut AOF record",
        step.label(),
    );
    assert_term_ids(
        &first_cold,
        POSTCUT_COLLECTION,
        POSTCUT_VALUE,
        &[POSTCUT_ID],
        &format!(
            "{} AOF replay recovers the exact post-cut value before second checkpoint",
            step.label(),
        ),
    )
    .await;
    assert_term_ids(
        &fixture.server,
        POSTCUT_COLLECTION,
        POSTCUT_VALUE,
        &[POSTCUT_ID],
        &format!("{} live engine applies post-cut value", step.label()),
    )
    .await;

    checkpoint_now(
        fixture.checkpoint.as_ref(),
        &format!("{} publish second durable cut", step.label()),
    )
    .await;
    let second_cold = cold_server(
        cold_engine(
            fixture.store.as_ref(),
            postcut_sequence,
            &format!("{} second durable cut", step.label()),
        ),
        postcut_sequence,
    );
    assert_term_ids(
        &second_cold,
        POSTCUT_COLLECTION,
        POSTCUT_VALUE,
        &[POSTCUT_ID],
        &format!(
            "{} second cold cut retains post-cut WAL value",
            step.label()
        ),
    )
    .await;
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
    let workspace = tempfile::tempdir().expect("create parent-owned publication workspace");
    let child_tmp = workspace.path().join("child-tmp");
    fs::create_dir(&child_tmp).expect("create parent-owned child TMPDIR");
    let handshake = workspace.path().join("entered");
    let stdout_path = workspace.path().join("child.stdout");
    let stderr_path = workspace.path().join("child.stderr");
    let executable = std::env::current_exe().expect("locate current-publication test executable");
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
            File::create(&stdout_path).expect("create current-publication child stdout"),
        ))
        .stderr(Stdio::from(
            File::create(&stderr_path).expect("create current-publication child stderr"),
        ))
        .spawn()
        .expect("spawn isolated current-publication child");
    let mut child = ChildCleanup(Some(child));
    let deadline = Instant::now() + CHILD_WATCHDOG;
    loop {
        match child
            .0
            .as_mut()
            .expect("current-publication child remains owned")
            .try_wait()
        {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => thread::sleep(POLL_INTERVAL),
            Ok(None) => {
                let mut raw = child.0.take().expect("timed-out child remains owned");
                let _ = raw.kill();
                let status = raw.wait().expect("reap killed current-publication child");
                let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
                let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
                panic!(
                    "current-publication child exceeded cleanup watchdog {CHILD_WATCHDOG:?}; status={status}; stdout={stdout}; stderr={stderr}",
                );
            }
            Err(error) => panic!("poll current-publication child: {error}"),
        }
    }
    let status = child
        .0
        .take()
        .expect("exited current-publication child remains owned")
        .wait()
        .expect("wait for current-publication child");
    let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
    let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
    let entered = fs::read_to_string(&handshake).unwrap_or_else(|error| {
        panic!(
            "current-publication child did not enter intended body: {error}; stdout={stdout}; stderr={stderr}",
        )
    });
    assert_eq!(
        entered, TEST_NAME,
        "current-publication child must run intended exact test",
    );
    assert!(
        status.success(),
        "current-publication child failed: status={status}; stdout={stdout}; stderr={stderr}",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn current_pointer_io_releases_apply_and_preserves_durable_cuts() {
    if std::env::var_os(CHILD_MODE_ENV).is_some() {
        let handshake = PathBuf::from(
            std::env::var_os(CHILD_HANDSHAKE_ENV)
                .expect("current-publication child receives handshake path"),
        );
        fs::write(&handshake, TEST_NAME).expect("record current-publication child entry");
        for step in [PointerIoStep::CurrentTemp, PointerIoStep::RootAfterCurrent] {
            run_pointer_io_case(step).await;
        }
        return;
    }
    tokio::task::spawn_blocking(run_isolated_child)
        .await
        .expect("current-publication child controller must join");
}

const BACKGROUND_MERGE_COLLECTION: &str = "current-publication-merge";
const BACKGROUND_POSTCUT_COLLECTION: &str = "current-publication-merge-postcut";
const BACKGROUND_MERGE_ID: &str = "current-publication-merge-id";
const BACKGROUND_POSTCUT_ID: &str = "current-publication-merge-postcut-id";
const BACKGROUND_MERGE_BASE_VALUE: &str = "current-publication-merge-base";
const BACKGROUND_POSTCUT_BASE_VALUE: &str = "current-publication-merge-postcut-base";
const BACKGROUND_POSTCUT_VALUE: &str = "current-publication-merge-postcut-retained";
const BACKGROUND_MERGE_DELTA_COUNT: u64 = 4;
const BACKGROUND_CHILD_WATCHDOG: Duration = Duration::from_secs(120);
const BACKGROUND_CHILD_MODE_ENV: &str = "LUMEN_CURRENT_PUBLICATION_MERGE_CHILD";
const BACKGROUND_CHILD_HANDSHAKE_ENV: &str = "LUMEN_CURRENT_PUBLICATION_MERGE_HANDSHAKE";
const BACKGROUND_CHILD_STEP_ENV: &str = "LUMEN_CURRENT_PUBLICATION_MERGE_STEP";
const BACKGROUND_TEST_NAME: &str =
    "background_merge_current_pointer_io_releases_apply_and_preserves_durable_cuts";

fn background_step_name(step: PointerIoStep) -> &'static str {
    match step {
        PointerIoStep::CurrentTemp => "current-temp",
        PointerIoStep::RootAfterCurrent => "root-after-current",
    }
}

fn background_step_from_environment() -> PointerIoStep {
    match std::env::var(BACKGROUND_CHILD_STEP_ENV)
        .expect("background-merge child receives pointer step")
        .as_str()
    {
        "current-temp" => PointerIoStep::CurrentTemp,
        "root-after-current" => PointerIoStep::RootAfterCurrent,
        value => panic!("background-merge child received unknown pointer step {value:?}"),
    }
}

/// Pauses only an actual background merge after its compaction input exists
/// and before it begins durable publication. The pointer injector is armed
/// only after this public observer reports `BeforePublish`; all foreground
/// checkpoints have already returned by then.
#[derive(Default)]
struct PauseBeforeMergeCurrentPublish {
    control: Mutex<Option<(mpsc::SyncSender<()>, mpsc::Receiver<()>)>>,
}

impl PauseBeforeMergeCurrentPublish {
    fn arm(&self) -> (mpsc::Receiver<()>, MergeCurrentPublishRelease) {
        let (entered_tx, entered_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::channel();
        assert!(
            self.control
                .lock()
                .expect("merge CURRENT observer mutex")
                .replace((entered_tx, release_rx))
                .is_none(),
            "fixture arms exactly one background merge BeforePublish pause",
        );
        (entered_rx, MergeCurrentPublishRelease(Some(release_tx)))
    }
}

impl MergeObserver for PauseBeforeMergeCurrentPublish {
    fn observe(&self, phase: MergePhase) -> io::Result<()> {
        if phase != MergePhase::BeforePublish {
            return Ok(());
        }
        let Some((entered, release)) = self
            .control
            .lock()
            .expect("merge CURRENT observer mutex")
            .take()
        else {
            return Ok(());
        };
        entered.send(()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                "background merge BeforePublish readiness receiver dropped",
            )
        })?;
        release.recv().map_err(|_| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                "background merge BeforePublish release sender dropped",
            )
        })?;
        Ok(())
    }
}

/// Releases the merge observer on ordinary and unwinding paths.
struct MergeCurrentPublishRelease(Option<mpsc::Sender<()>>);

impl MergeCurrentPublishRelease {
    fn release(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}

impl Drop for MergeCurrentPublishRelease {
    fn drop(&mut self) {
        self.release();
    }
}

fn background_merge_fixture(observer: Arc<PauseBeforeMergeCurrentPublish>) -> Fixture {
    let dir = tempfile::tempdir().expect("create background-merge publication fixture directory");
    let pointer_hold = Arc::new(HoldPointerIo::default());
    let store = Arc::new(
        SegmentRdbStore::with_failure_injector_and_merge_observer(
            dir.path().join("segments"),
            pointer_hold.clone(),
            observer,
        )
        .expect("open background-merge observed segment store"),
    );
    let aof_path = dir.path().join("aof.log");
    let aof: SharedAof = Arc::new(Mutex::new(
        AofWriter::open(&aof_path).expect("open background-merge AOF"),
    ));
    let engine = Arc::new(Engine::new());
    let wal = Arc::new(MemWal::new());
    let shared_wal: SharedWal = wal.clone();
    let writer = WriteCoordinator::start_from_with_aof(shared_wal, engine.clone(), 0, aof.clone());
    let writer_sink: Arc<dyn WriteSink> = writer.clone();
    let checkpoint = Arc::new(SegmentCheckpointSink {
        engine: engine.clone(),
        store: store.clone(),
        writer: writer_sink.clone(),
        aof: Some(aof),
    });
    let checkpoint_api: Arc<dyn CheckpointSink> = checkpoint.clone();
    let state = AppState::with_components(engine, Arc::new(AuthConfig::open()), writer_sink)
        .with_checkpoint(checkpoint_api);
    Fixture {
        server: Arc::new(
            TestServer::new(router(state)).expect("open background-merge publication server"),
        ),
        store,
        checkpoint,
        writer,
        wal,
        aof_path,
        pointer_hold,
        _dir: dir,
    }
}

fn background_merge_value(round: u64) -> String {
    format!("current-publication-merge-v{round}")
}

async fn wait_for_background_merge_pause(entered_rx: mpsc::Receiver<()>, step: PointerIoStep) {
    let entered = tokio::task::spawn_blocking(move || entered_rx.recv_timeout(READY_WATCHDOG))
        .await
        .expect("background merge readiness task must join");
    assert!(
        matches!(entered, Ok(())),
        "{}: four completed foreground checkpoints must reach the real background merge BeforePublish pause: {entered:?}",
        step.label(),
    );
}

async fn wait_for_background_merge_completion(store: Arc<SegmentRdbStore>, context: &str) {
    let result = tokio::task::spawn_blocking(move || store.wait_for_merges(FINISH_WATCHDOG))
        .await
        .expect("background merge completion task must join");
    result.unwrap_or_else(|error| panic!("{context}: background merge failed: {error:#}"));
}

async fn run_background_merge_pointer_io_case(step: PointerIoStep) {
    let observer = Arc::new(PauseBeforeMergeCurrentPublish::default());
    let fixture = background_merge_fixture(observer.clone());
    create_keyword_collection(&fixture.server, BACKGROUND_MERGE_COLLECTION).await;
    create_keyword_collection(&fixture.server, BACKGROUND_POSTCUT_COLLECTION).await;
    index_keyword(
        &fixture.server,
        BACKGROUND_MERGE_COLLECTION,
        BACKGROUND_MERGE_ID,
        BACKGROUND_MERGE_BASE_VALUE,
    )
    .await;
    index_keyword(
        &fixture.server,
        BACKGROUND_POSTCUT_COLLECTION,
        BACKGROUND_POSTCUT_ID,
        BACKGROUND_POSTCUT_BASE_VALUE,
    )
    .await;
    checkpoint_now(
        fixture.checkpoint.as_ref(),
        "publish background-merge CURRENT baseline",
    )
    .await;

    let (merge_entered, mut merge_release) = observer.arm();
    for round in 1..=BACKGROUND_MERGE_DELTA_COUNT {
        let value = background_merge_value(round);
        index_keyword(
            &fixture.server,
            BACKGROUND_MERGE_COLLECTION,
            BACKGROUND_MERGE_ID,
            &value,
        )
        .await;
        checkpoint_now(
            fixture.checkpoint.as_ref(),
            &format!("publish background-merge delta {round}"),
        )
        .await;
    }
    let merge_sequence = fixture.writer.applied_seq();
    wait_for_background_merge_pause(merge_entered, step).await;

    // `MergePhase::BeforePublish` is emitted only by `merge_one`; the four
    // synchronous foreground checkpoints above completed before the failure
    // injector is armed. Thus this exact pointer pause belongs to the merge
    // generation, not to a foreground trigger checkpoint.
    let (pointer_entered_tx, pointer_entered_rx) = mpsc::sync_channel(1);
    let (pointer_release_tx, pointer_release_rx) = mpsc::sync_channel(1);
    fixture
        .pointer_hold
        .arm(step, pointer_entered_tx, pointer_release_rx);
    let mut pointer_release = PointerIoRelease(Some(pointer_release_tx));
    merge_release.release();
    let pointer_entered =
        tokio::task::spawn_blocking(move || pointer_entered_rx.recv_timeout(READY_WATCHDOG))
            .await
            .expect("background merge CURRENT I/O readiness task must join");
    if !matches!(pointer_entered, Ok(())) {
        pointer_release.release();
        panic!(
            "{}: injected CURRENT pointer pause did not occur after the real merge BeforePublish gate: {pointer_entered:?}",
            step.label(),
        );
    }

    let postcut_sequence = merge_sequence + 1;
    let postcut_http = post_keyword_status(
        fixture.server.clone(),
        BACKGROUND_POSTCUT_COLLECTION,
        BACKGROUND_POSTCUT_ID,
        BACKGROUND_POSTCUT_VALUE,
    );
    tokio::pin!(postcut_http);
    let wal_context = format!("{} background-merge post-cut HTTP write", step.label());
    let wal_ready = wait_for_wal_sequence(fixture.wal.as_ref(), postcut_sequence, &wal_context);
    tokio::pin!(wal_ready);
    let response_before_wal = tokio::select! {
        () = &mut wal_ready => None,
        status = &mut postcut_http => Some(status),
    };
    if response_before_wal.is_some() {
        (&mut wal_ready).await;
    }
    assert_eq!(
        fixture
            .wal
            .latest_seq()
            .await
            .expect("read background-merge post-cut MemWal sequence"),
        postcut_sequence,
        "{}: merge-interleaved post-cut record must own the next real WAL sequence",
        step.label(),
    );

    // This is an ordering assertion. The timeout releases test-owned I/O; it
    // does not define a service latency budget.
    let response_before_release = match response_before_wal {
        Some(status) => Some(status),
        None => tokio::time::timeout(INTERLEAVING_WATCHDOG, &mut postcut_http)
            .await
            .ok(),
    };

    pointer_release.release();
    wait_for_background_merge_completion(
        fixture.store.clone(),
        &format!(
            "{} background merge after CURRENT I/O release",
            step.label()
        ),
    )
    .await;
    let response_after_release = if response_before_release.is_none() {
        Some(
            tokio::time::timeout(FINISH_WATCHDOG, &mut postcut_http)
                .await
                .unwrap_or_else(|_| {
                    panic!(
                        "{}: retained merge-interleaved HTTP request exceeded cleanup after release",
                        step.label(),
                    )
                }),
        )
    } else {
        None
    };
    let response_status = response_before_release
        .or(response_after_release)
        .expect("merge-interleaved post-cut HTTP request returns before or after cleanup");
    assert_eq!(
        response_status,
        StatusCode::OK,
        "{}: merge-interleaved retained HTTP request must eventually report success",
        step.label(),
    );
    assert!(
        response_before_release.is_some(),
        "{}: post-cut write that reached WAL must complete before background merge CURRENT I/O releases",
        step.label(),
    );
    assert_eq!(
        fixture.writer.applied_seq(),
        postcut_sequence,
        "{}: the later record alone advances the live watermark after the merge cut",
        step.label(),
    );

    let merged_cold_engine = cold_engine(
        fixture.store.as_ref(),
        merge_sequence,
        &format!("{} background merge durable cut", step.label()),
    );
    let merged_cold = cold_server(merged_cold_engine.clone(), merge_sequence);
    let final_merge_value = background_merge_value(BACKGROUND_MERGE_DELTA_COUNT);
    assert_term_ids(
        &merged_cold,
        BACKGROUND_MERGE_COLLECTION,
        &final_merge_value,
        &[BACKGROUND_MERGE_ID],
        &format!("{} merge cold cut retains compaction output", step.label()),
    )
    .await;
    assert_term_ids(
        &merged_cold,
        BACKGROUND_POSTCUT_COLLECTION,
        BACKGROUND_POSTCUT_VALUE,
        &[],
        &format!("{} merge cold cut excludes later WAL value", step.label()),
    )
    .await;
    let replayed = replay_aof_into(&merged_cold_engine, &fixture.aof_path, merge_sequence)
        .unwrap_or_else(|error| {
            panic!(
                "{}: AOF after background merge must replay the retained post-cut record: {error:#}",
                step.label(),
            )
        });
    assert_eq!(
        replayed,
        postcut_sequence,
        "{}: merge durable cut must retain exactly the post-cut AOF record",
        step.label(),
    );
    assert_term_ids(
        &merged_cold,
        BACKGROUND_POSTCUT_COLLECTION,
        BACKGROUND_POSTCUT_VALUE,
        &[BACKGROUND_POSTCUT_ID],
        &format!(
            "{} AOF replay recovers merge-interleaved post-cut value",
            step.label()
        ),
    )
    .await;
    assert_term_ids(
        &fixture.server,
        BACKGROUND_POSTCUT_COLLECTION,
        BACKGROUND_POSTCUT_VALUE,
        &[BACKGROUND_POSTCUT_ID],
        &format!(
            "{} live engine applies merge-interleaved post-cut value",
            step.label()
        ),
    )
    .await;

    checkpoint_now(
        fixture.checkpoint.as_ref(),
        &format!("{} publish final post-merge durable cut", step.label()),
    )
    .await;
    let final_cold = cold_server(
        cold_engine(
            fixture.store.as_ref(),
            postcut_sequence,
            &format!("{} final post-merge cold cut", step.label()),
        ),
        postcut_sequence,
    );
    assert_term_ids(
        &final_cold,
        BACKGROUND_POSTCUT_COLLECTION,
        BACKGROUND_POSTCUT_VALUE,
        &[BACKGROUND_POSTCUT_ID],
        &format!("{} final cold cut retains post-cut value", step.label()),
    )
    .await;
}

fn run_isolated_background_merge_child() {
    let workspace = tempfile::tempdir().expect("create parent-owned background-merge workspace");
    let executable = std::env::current_exe().expect("locate background-merge test executable");
    let mut failures = Vec::new();
    for step in [PointerIoStep::CurrentTemp, PointerIoStep::RootAfterCurrent] {
        let step_name = background_step_name(step);
        let child_root = workspace.path().join(step_name);
        fs::create_dir(&child_root).expect("create parent-owned background-merge child root");
        let child_tmp = child_root.join("tmp");
        fs::create_dir(&child_tmp).expect("create parent-owned background-merge TMPDIR");
        let handshake = child_root.join("entered");
        let stdout_path = child_root.join("child.stdout");
        let stderr_path = child_root.join("child.stderr");
        let child = Command::new(&executable)
            .env(BACKGROUND_CHILD_MODE_ENV, "1")
            .env(BACKGROUND_CHILD_HANDSHAKE_ENV, &handshake)
            .env(BACKGROUND_CHILD_STEP_ENV, step_name)
            .env("TMPDIR", &child_tmp)
            .env("TEMP", &child_tmp)
            .env("TMP", &child_tmp)
            .arg(BACKGROUND_TEST_NAME)
            .arg("--exact")
            .arg("--nocapture")
            .arg("--test-threads=1")
            .stdout(Stdio::from(
                File::create(&stdout_path).expect("create background-merge child stdout"),
            ))
            .stderr(Stdio::from(
                File::create(&stderr_path).expect("create background-merge child stderr"),
            ))
            .spawn()
            .expect("spawn isolated background-merge child");
        let mut child = ChildCleanup(Some(child));
        let deadline = Instant::now() + BACKGROUND_CHILD_WATCHDOG;
        loop {
            match child
                .0
                .as_mut()
                .expect("background-merge child remains owned")
                .try_wait()
            {
                Ok(Some(_)) => break,
                Ok(None) if Instant::now() < deadline => thread::sleep(POLL_INTERVAL),
                Ok(None) => {
                    let mut raw = child
                        .0
                        .take()
                        .expect("timed-out background-merge child remains owned");
                    let _ = raw.kill();
                    let status = raw.wait().expect("reap killed background-merge child");
                    let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
                    let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
                    panic!(
                        "{step_name}: background-merge child exceeded cleanup watchdog: status={status}; stdout={stdout}; stderr={stderr}",
                    );
                }
                Err(error) => panic!("{step_name}: poll background-merge child: {error}"),
            }
        }
        let status = child
            .0
            .take()
            .expect("exited background-merge child remains owned")
            .wait()
            .expect("wait for background-merge child");
        let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
        let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
        let expected_handshake = format!("{BACKGROUND_TEST_NAME}:{step_name}");
        match fs::read_to_string(&handshake) {
            Ok(entered) if entered == expected_handshake && status.success() => {}
            Ok(entered) => failures.push(format!(
                "{step_name}: status={status}; entered={entered:?}; expected={expected_handshake:?}; stdout={stdout}; stderr={stderr}",
            )),
            Err(error) => failures.push(format!(
                "{step_name}: missing intended-child handshake: {error}; status={status}; stdout={stdout}; stderr={stderr}",
            )),
        }
    }
    assert!(
        failures.is_empty(),
        "background-merge pointer cases failed: {}",
        failures.join("\n\n"),
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn background_merge_current_pointer_io_releases_apply_and_preserves_durable_cuts() {
    if std::env::var_os(BACKGROUND_CHILD_MODE_ENV).is_some() {
        let handshake = PathBuf::from(
            std::env::var_os(BACKGROUND_CHILD_HANDSHAKE_ENV)
                .expect("background-merge child receives handshake path"),
        );
        let step = background_step_from_environment();
        fs::write(
            &handshake,
            format!("{BACKGROUND_TEST_NAME}:{}", background_step_name(step)),
        )
        .expect("record background-merge child entry");
        run_background_merge_pointer_io_case(step).await;
        return;
    }
    tokio::task::spawn_blocking(run_isolated_background_merge_child)
        .await
        .expect("background-merge child controller must join");
}
