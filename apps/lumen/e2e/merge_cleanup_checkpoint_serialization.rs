//! Black-box contract for serializing background merge scratch cleanup with a
//! checkpoint that owns the root save interval.
//!
//! The child creates a Keyword base and then four real sparse Keyword deltas.
//! Four deltas select one real background merge. The public `AfterPublish`
//! observer pauses after that merge has published its replacement generation,
//! dropped its normal save permit, and still owns its private `.stage-gen-*`
//! scratch directory. The child identifies that one direct scratch directory.
//!
//! A second public checkpoint then starts. Its injected `SyncFile` hold occurs
//! after root inventory but before its publication, while the checkpoint still
//! owns the same root save gate. The child releases the merge observer. A
//! correct cleanup waits for the checkpoint to leave that full protected
//! interval. The exact scratch directory stays present. A published merge can
//! also queue its next pass, so `wait_for_merges` is only a secondary
//! full-worker-idle observation. Once the checkpoint releases, cleanup
//! finishes, the scratch is removed, and live plus cold exact queries retain
//! the final value.
//!
//! This tests save-gate serialization. It does not claim to pause the smaller
//! `read_dir` to `symlink_metadata` syscall interval inside inventory. Holding
//! any later commit step is enough: cleanup that cannot run during the full
//! save-gated interval cannot remove an entry during its earlier inventory.
//!
//! The parent runs a child with a parent-owned temporary directory. The
//! watchdog only kills and reaps a stuck child before its workspace is removed.
//! It is cleanup safety, not a checkpoint latency target.
//!
//! # Facets
//!
//! - Behavior: `merge_cleanup_checkpoint_serialization.rs:342-437` drives a
//!   real base plus four Keyword deltas through public `Engine` and
//!   `SegmentRdbStore` operations, pauses one real published merge, and holds
//!   one real checkpoint commit. `:387,425-437` records the exact protected
//!   scratch directory and the secondary full-worker-idle state. `:465-491`
//!   makes the scratch-presence assertion the direct cleanup oracle, then
//!   requires checkpoint completion, cleanup, live exact search, and cold
//!   CURRENT recovery. It
//!   covers `apps/lumen/src/segment_background_merge.rs:391-429,611-624` and
//!   `apps/lumen/src/segment_rdb.rs:540-565,775-859`.
//! - Security: this change only serializes deletion of the process-created
//!   scratch path at `apps/lumen/src/segment_background_merge.rs:615-623`; it
//!   does not accept a new caller path or relax root validation. Existing
//!   `apps/lumen/e2e/segment_startup_fail_closed_e2e.rs:798-811,816-842,1119-1148`
//!   keeps symlink, unknown-root, and unpointed-generation inputs fail closed
//!   under the declared default gate. `:306-330` also refuses to treat a
//!   symlink or non-directory as this test's merge scratch.
//! - Performance: `apps/lumen/docs/indexing.md:271-276` says, verbatim,
//!   "One merge runs per process. Four delta segments request a merge, and a
//!   field retains at most 16." `:360-385` creates exactly four deltas and
//!   requires the real merge-completed counter before the pause.
//!   The bounded waits are only interleaving and cleanup controls; this case
//!   makes no latency or throughput claim.
//!
//! # Root negative control
//!
//! After the repair, move the repaired merge scratch `remove_dir_all` and
//! unpin work back outside the root save gate at
//! `apps/lumen/src/segment_background_merge.rs:615-623`. The behavior
//! scratch-presence assertion at `:465` must fail because cleanup completes while the held
//! checkpoint still owns the save interval. Restore the corrected production
//! SHA before another gate. Do not weaken the observer order or replace the
//! exact scratch identity with filesystem polling.
//!
//! Gate: cargo test -p lumen --test merge_cleanup_checkpoint_serialization -- --nocapture.
//! Full declared behavior gate: cargo test -p lumen.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use lumen::segment_rdb::{MergeObserver, MergePhase, SegmentRdbStore};
use lumen::storage::Engine;
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest, QueryNode,
    SearchRequest, TermQuery,
};
use storage_durable::{CommitStep, FailureInjector, FailurePoint};

const COLLECTION: &str = "merge-cleanup-checkpoint-serialization";
const FIELD: &str = "kw";
const EXTERNAL_ID: &str = "merge-cleanup-row";
const BASE_VALUE: &str = "merge-cleanup-base";
const BASE_SEQUENCE: u64 = 7_100;
const DELTA_COUNT: u64 = 4;
const CHECKPOINT_SEQUENCE: u64 = BASE_SEQUENCE + DELTA_COUNT + 1;
const READY_WATCHDOG: Duration = Duration::from_secs(30);
const EARLY_CLEANUP_OBSERVATION: Duration = Duration::from_secs(2);
const FINISH_WATCHDOG: Duration = Duration::from_secs(30);
const CHILD_WATCHDOG: Duration = Duration::from_secs(120);
const POLL_INTERVAL: Duration = Duration::from_millis(20);
const CHILD_MODE_ENV: &str = "LUMEN_MERGE_CLEANUP_SERIALIZATION_CHILD";
const CHILD_ROOT_ENV: &str = "LUMEN_MERGE_CLEANUP_SERIALIZATION_ROOT";
const CHILD_HANDSHAKE_ENV: &str = "LUMEN_MERGE_CLEANUP_SERIALIZATION_HANDSHAKE";
const CHILD_CASE: &str = "save-gate-cleanup";
const TEST_NAME: &str = "background_merge_cleanup_waits_for_checkpoint_save_gate";

/// Releases a blocked test seam on normal completion and panic unwinding.
struct ReleaseSignal(Option<mpsc::Sender<()>>);

impl ReleaseSignal {
    fn release(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}

impl Drop for ReleaseSignal {
    fn drop(&mut self) {
        self.release();
    }
}

/// Holds exactly one checkpoint `SyncFile` operation. The generation commit is
/// still inside `SegmentRdbStore`'s save gate at this public failure seam.
#[derive(Default)]
struct HoldNextSyncFile {
    hold: Mutex<Option<(mpsc::Sender<()>, mpsc::Receiver<()>)>>,
}

impl HoldNextSyncFile {
    fn arm(&self, entered: mpsc::Sender<()>, release: mpsc::Receiver<()>) {
        assert!(
            self.hold
                .lock()
                .expect("checkpoint SyncFile hold mutex")
                .replace((entered, release))
                .is_none(),
            "fixture arms exactly one checkpoint SyncFile hold",
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

/// Pauses after the selected merge has published but before its source scratch
/// cleanup. Repeated worker wakeups are intentionally not held.
struct HoldAfterPublish {
    hold: Mutex<Option<(mpsc::Sender<()>, mpsc::Receiver<()>)>>,
}

impl HoldAfterPublish {
    fn new(entered: mpsc::Sender<()>, release: mpsc::Receiver<()>) -> Self {
        Self {
            hold: Mutex::new(Some((entered, release))),
        }
    }
}

impl MergeObserver for HoldAfterPublish {
    fn observe(&self, phase: MergePhase) -> io::Result<()> {
        if phase != MergePhase::AfterPublish {
            return Ok(());
        }
        let Some((entered, release)) = self
            .hold
            .lock()
            .expect("merge AfterPublish hold mutex")
            .take()
        else {
            return Ok(());
        };
        entered.send(()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                "merge AfterPublish readiness receiver dropped",
            )
        })?;
        release.recv().map_err(|_| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                "merge AfterPublish release sender dropped",
            )
        })?;
        Ok(())
    }
}

/// Kills and reaps a hung child before its parent-owned workspace disappears.
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

fn round_value(round: u64) -> String {
    format!("merge-cleanup-round-{round}")
}

fn index_keyword(engine: &Engine, value: &str) {
    engine
        .index(
            COLLECTION,
            IndexRequest {
                items: vec![IndexItem {
                    external_id: EXTERNAL_ID.to_owned(),
                    field: FIELD.to_owned(),
                    value: FieldValue::String(value.to_owned()),
                    version: None,
                }],
                request_id: None,
            },
        )
        .expect("apply valid public Keyword update");
}

fn exact_ids(engine: &Engine, value: &str) -> Vec<String> {
    let mut ids = engine
        .search(
            COLLECTION,
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
            },
        )
        .expect("run exact Keyword query")
        .hits
        .into_iter()
        .map(|hit| hit.external_id)
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

fn assert_visible_final_value(engine: &Engine, phase: &str) {
    assert_eq!(
        exact_ids(engine, &round_value(DELTA_COUNT)),
        vec![EXTERNAL_ID.to_owned()],
        "{phase}: the final Keyword delta must stay query-visible",
    );
    assert_eq!(
        exact_ids(engine, BASE_VALUE),
        Vec::<String>::new(),
        "{phase}: the base Keyword value must stay masked by the final delta",
    );
}

/// Returns the exact direct staging directory still owned by the paused merge.
fn selected_merge_scratch(root: &Path) -> PathBuf {
    let mut matches = fs::read_dir(root)
        .expect("list checkpoint root after paused merge publication")
        .map(|entry| entry.expect("read direct checkpoint-root entry"))
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_str()?;
            name.starts_with(".stage-gen-").then_some(entry.path())
        })
        .collect::<Vec<_>>();
    matches.sort();
    assert_eq!(
        matches.len(),
        1,
        "one paused merge must own exactly one direct .stage-gen-* scratch directory: {matches:?}",
    );
    let scratch = matches.pop().expect("one selected merge scratch path");
    let metadata = fs::symlink_metadata(&scratch).expect("inspect selected merge scratch");
    assert!(
        metadata.is_dir() && !metadata.file_type().is_symlink(),
        "the selected merge scratch must be a real directory, never a symlink: {}",
        scratch.display(),
    );
    scratch
}

fn wait_receiver<T>(
    receiver: mpsc::Receiver<T>,
    timeout: Duration,
    phase: &str,
) -> T {
    receiver.recv_timeout(timeout).unwrap_or_else(|error| {
        panic!("{phase} did not finish before bounded cleanup watchdog {timeout:?}: {error}")
    })
}

fn run_child(root: PathBuf, handshake: PathBuf) {
    fs::write(&handshake, CHILD_CASE).expect("record merge-cleanup child entry");

    let (merge_entered_tx, merge_entered_rx) = mpsc::channel();
    let (merge_release_tx, merge_release_rx) = mpsc::channel();
    let mut merge_release = ReleaseSignal(Some(merge_release_tx));
    let sync_hold = Arc::new(HoldNextSyncFile::default());
    let observer = Arc::new(HoldAfterPublish::new(merge_entered_tx, merge_release_rx));
    let store = Arc::new(
        SegmentRdbStore::with_failure_injector_and_merge_observer(
            &root,
            sync_hold.clone(),
            observer,
        )
        .expect("open public observed segment store"),
    );
    let engine = Arc::new(Engine::new());

    engine
        .create_collection(COLLECTION, keyword_schema())
        .expect("create public Keyword collection");
    index_keyword(&engine, BASE_VALUE);
    store
        .save(&engine, BASE_SEQUENCE)
        .expect("publish public Keyword base");
    for round in 1..=DELTA_COUNT {
        index_keyword(&engine, &round_value(round));
        store
            .save(&engine, BASE_SEQUENCE + round)
            .expect("publish public Keyword delta");
    }

    let merge_entered = merge_entered_rx.recv_timeout(READY_WATCHDOG);
    if merge_entered.is_err() {
        merge_release.release();
    }
    assert!(
        merge_entered.is_ok(),
        "exactly four public Keyword deltas must reach one real merge AfterPublish pause: {merge_entered:?}",
    );
    assert_eq!(
        engine.metrics().segment_merge_completed_total.get(),
        1,
        "four delta segments must publish exactly one real merge before cleanup is released",
    );
    let scratch = selected_merge_scratch(&root);

    let (checkpoint_entered_tx, checkpoint_entered_rx) = mpsc::channel();
    let (checkpoint_release_tx, checkpoint_release_rx) = mpsc::channel();
    let mut checkpoint_release = ReleaseSignal(Some(checkpoint_release_tx));
    sync_hold.arm(checkpoint_entered_tx, checkpoint_release_rx);
    let (checkpoint_done_tx, checkpoint_done_rx) = mpsc::channel();
    let checkpoint_store = store.clone();
    let checkpoint_engine = engine.clone();
    let checkpoint_thread = thread::Builder::new()
        .name("lumen-merge-cleanup-held-checkpoint".into())
        .spawn(move || {
            let _ = checkpoint_done_tx.send(
                checkpoint_store.save_with_sequence(&checkpoint_engine, CHECKPOINT_SEQUENCE),
            );
        })
        .expect("start held public checkpoint thread");

    let checkpoint_entered = checkpoint_entered_rx.recv_timeout(READY_WATCHDOG);
    if checkpoint_entered.is_err() {
        checkpoint_release.release();
        merge_release.release();
    }
    assert!(
        checkpoint_entered.is_ok(),
        "public checkpoint must reach its held SyncFile while owning the root save interval: {checkpoint_entered:?}",
    );

    merge_release.release();
    let (merge_done_tx, merge_done_rx) = mpsc::channel();
    let merge_store = store.clone();
    let merge_wait_thread = thread::Builder::new()
        .name("lumen-merge-cleanup-wait".into())
        .spawn(move || {
            let _ = merge_done_tx.send(merge_store.wait_for_merges(FINISH_WATCHDOG));
        })
        .expect("start background-merge completion observer");

    // A published merge may queue a follow-up pass before this worker returns.
    // This is therefore a full-root-worker-idle observation only. The exact
    // scratch path below is the direct old-cleanup oracle.
    let worker_became_idle_before_checkpoint_release =
        match merge_done_rx.recv_timeout(EARLY_CLEANUP_OBSERVATION) {
        Ok(Ok(())) => Some("completed successfully".to_owned()),
        Ok(Err(error)) => Some(format!("completed with error: {error:#}")),
        Err(mpsc::RecvTimeoutError::Timeout) => None,
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            Some("completion observer disconnected".to_owned())
        }
    };
    let scratch_survived_held_checkpoint = scratch.exists();

    checkpoint_release.release();
    let checkpoint_result = wait_receiver(
        checkpoint_done_rx,
        FINISH_WATCHDOG,
        "held public checkpoint after merge cleanup release",
    );
    checkpoint_thread
        .join()
        .expect("held public checkpoint thread must not panic");

    let merge_result = match worker_became_idle_before_checkpoint_release.as_ref() {
        Some(_) => Ok(()),
        None => wait_receiver(
            merge_done_rx,
            FINISH_WATCHDOG,
            "background merge cleanup after checkpoint release",
        ),
    };
    merge_wait_thread
        .join()
        .expect("background-merge completion observer must not panic");

    assert!(
        worker_became_idle_before_checkpoint_release.is_none(),
        "background merge work must not become fully idle while the checkpoint owns its save-gated root interval; worker_became_idle_before_checkpoint_release={worker_became_idle_before_checkpoint_release:?}",
    );
    assert!(
        scratch_survived_held_checkpoint,
        "the exact paused merge scratch must stay present while the checkpoint owns its save-gated root interval: {}",
        scratch.display(),
    );
    assert_eq!(
        checkpoint_result.expect("held public checkpoint must publish after release"),
        CHECKPOINT_SEQUENCE,
        "held public checkpoint must publish its requested watermark",
    );
    merge_result.expect("background merge must finish after checkpoint releases save gate");
    assert!(
        !scratch.exists(),
        "the exact merge scratch must be removed only after the held checkpoint releases: {}",
        scratch.display(),
    );
    assert_visible_final_value(&engine, "live after serialized cleanup");

    let cold = store
        .load_current_generation()
        .expect("cold-open CURRENT after serialized cleanup")
        .expect("serialized cleanup must leave a complete CURRENT generation");
    assert_eq!(
        cold.sequence, CHECKPOINT_SEQUENCE,
        "cold CURRENT must retain the checkpoint watermark after serialized cleanup",
    );
    assert_visible_final_value(&cold.engine, "cold after serialized cleanup");
}

fn run_isolated_case() {
    let workspace = tempfile::tempdir().expect("create parent-owned merge-cleanup workspace");
    let child_root = workspace.path().join("segments");
    let child_tmp = workspace.path().join("child-tmp");
    fs::create_dir(&child_tmp).expect("create parent-owned merge-cleanup child TMPDIR");
    let handshake = workspace.path().join("entered-case");
    let stdout_path = workspace.path().join("child.stdout");
    let stderr_path = workspace.path().join("child.stderr");
    let executable = std::env::current_exe().expect("locate merge-cleanup test executable");
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
            File::create(&stdout_path).expect("create merge-cleanup child stdout"),
        ))
        .stderr(Stdio::from(
            File::create(&stderr_path).expect("create merge-cleanup child stderr"),
        ))
        .spawn()
        .expect("spawn isolated merge-cleanup child");
    let mut child = ChildCleanup(Some(child));
    let deadline = Instant::now() + CHILD_WATCHDOG;
    loop {
        match child
            .0
            .as_mut()
            .expect("merge-cleanup child remains owned until exit")
            .try_wait()
        {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => thread::sleep(POLL_INTERVAL),
            Ok(None) => {
                let mut raw = child.0.take().expect("timed-out child remains owned");
                let _ = raw.kill();
                let status = raw.wait().expect("reap killed merge-cleanup child");
                let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
                let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
                panic!(
                    "merge-cleanup child exceeded cleanup watchdog {CHILD_WATCHDOG:?}; status={status}; stdout={stdout}; stderr={stderr}",
                );
            }
            Err(error) => panic!("poll isolated merge-cleanup child: {error}"),
        }
    }
    let status = child
        .0
        .take()
        .expect("exited merge-cleanup child remains owned")
        .wait()
        .expect("wait for exited merge-cleanup child");
    let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
    let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
    let entered = fs::read_to_string(&handshake).unwrap_or_else(|error| {
        panic!(
            "isolated merge-cleanup child did not enter {TEST_NAME}: {error}; stdout={stdout}; stderr={stderr}",
        )
    });
    assert_eq!(
        entered, CHILD_CASE,
        "isolated merge-cleanup child must enter the intended behavior body",
    );
    assert!(
        status.success(),
        "isolated merge-cleanup child failed: status={status}; stdout={stdout}; stderr={stderr}",
    );
}

#[test]
fn background_merge_cleanup_waits_for_checkpoint_save_gate() {
    if std::env::var(CHILD_MODE_ENV).ok().as_deref() == Some(CHILD_CASE) {
        let root = PathBuf::from(
            std::env::var_os(CHILD_ROOT_ENV)
                .expect("merge-cleanup child must receive parent-owned segment root"),
        );
        let handshake = PathBuf::from(
            std::env::var_os(CHILD_HANDSHAKE_ENV)
                .expect("merge-cleanup child must receive parent-owned handshake path"),
        );
        run_child(root, handshake);
        return;
    }

    run_isolated_case();
}
