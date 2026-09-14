//! Checkpoint/AOF failure-recovery contract.
//!
//! Both cases use the production `SegmentCheckpointSink` through its public
//! `CheckpointSink` trait, HTTP writes through `WriteCoordinator`, the real
//! generation `SyncFile` injector, and real AOF replay.  They do not replace
//! checkpointing with a local fake sink.
//!
//! # Facets
//!
//! - Behavior: `prepublication_sync_failure_retains_frozen_cut_and_aof_suffix`
//!   requires the injected real `SyncFile` failure to leave exact `CURRENT`
//!   and AOF bytes unchanged at :466-475, then requires a cold RDB plus AOF
//!   replay to recover the committed frozen value at :478-489. Its retry
//!   starts before the newer record, then publishes that original cut while
//!   retaining the newer suffix at :494-524.
//!   `postpublication_trim_failure_retains_aof_then_reclaims_only_covered_frames`
//!   requires a real `aof.log.compact.tmp` directory to make trim fail after
//!   publication, while `CURRENT`, both AOF frames, and cold replay remain
//!   correct at :581-627. The healthy retry removes only covered frames and
//!   retains the later one at :634-669. These assertions cover
//!   `apps/lumen/src/segment_checkpoint.rs:174-189` and
//!   `libs/storage-durable/src/framed_log.rs:234-276`.
//! - Security: the compact-temp pathname is a process-read filesystem boundary.
//!   The directory obstacle is an input that trim must refuse without deleting
//!   or replacing it. Assertions at :588-598 require the object to remain a
//!   directory and the full pre-fault AOF bytes to remain exact.  The cold
//!   replay assertion at :615-627 proves that the retained file is still a
//!   valid frame stream.  The `SyncFile` path is process-owned by
//!   `libs/storage-durable/src/generation.rs:591-635`; its pre-publication
//!   failure is closed by :455-475 without moving `CURRENT` or consuming AOF.
//! - Performance: `apps/lumen/docs/indexing.md:264-276` says, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget."
//!   `assert_pending_budget` at :352-366 reads actual public `/metrics` total
//!   and high-water samples after each failed recovery path at :489 and :614,
//!   and after the complete recovery paths at :538 and :693, and
//!   requires both to stay within that budget.  The bounded channel waits are
//!   cleanup for deterministic file-I/O interleavings, not latency promises.
//!
//! # Root negative controls
//!
//! 1. Move the existing AOF trim block in
//!    `apps/lumen/src/segment_checkpoint.rs` before `save_with_sequence`.
//!    The exact-byte AOF assertion at :472-475 must fail after the injected
//!    pre-publication `SyncFile` error.
//! 2. In `libs/storage-durable/src/framed_log.rs`, make compact-temp open
//!    failure destructively rewrite `self.path` before returning the same
//!    error. The exact-byte and cold-suffix assertions at :594-627
//!    must fail.  Restore every mutated source file by SHA-256.
//!
//! Gate: `cargo test --locked -p lumen --test checkpoint_aof_failure_recovery -- --nocapture`.
//! The same target must also run under `--features jieba`.  The full declared
//! durability gate is `cargo test -p lumen --test backup_restore_e2e --test indexing_durable_oracle`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc, Arc, Mutex,
};
use std::time::Duration;

use axum_test::TestServer;
use serde_json::{json, Value};

use lumen::aof::{replay_aof_into, AofReader, AofWriter};
use lumen::api::{router, AppState, CheckpointSink};
use lumen::auth::AuthConfig;
use lumen::coordinator::{SharedAof, WriteCoordinator, WriteSink};
use lumen::segment_checkpoint::SegmentCheckpointSink;
use lumen::segment_rdb::{MergeObserver, MergePhase, SegmentRdbStore};
use lumen::storage::Engine;
use lumen::wal::{MemWal, SharedWal};
use storage_durable::{CommitStep, FailureInjector, FailurePoint};

const COLLECTION: &str = "checkpoint-aof-failure";
const FIELD: &str = "kw";
const ID: &str = "checkpoint-aof-id";
const BASE_VALUE: &str = "checkpoint-aof-base";
const FROZEN_VALUE: &str = "checkpoint-aof-frozen";
const NEWER_VALUE: &str = "checkpoint-aof-newer";
const CAPTURED_VALUE: &str = "checkpoint-aof-captured";
const FIRST_TAIL_VALUE: &str = "checkpoint-aof-first-tail";
const LATER_TAIL_VALUE: &str = "checkpoint-aof-later-tail";
const READY_WATCHDOG: Duration = Duration::from_secs(10);
const CHECKPOINT_WATCHDOG: Duration = Duration::from_secs(20);
const PENDING_HARD_LIMIT_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Default)]
struct NoopMergeObserver;

impl MergeObserver for NoopMergeObserver {
    fn observe(&self, _: MergePhase) -> io::Result<()> {
        Ok(())
    }
}

/// One fixture-owned injector supplies either a real pre-publication failure
/// or a real post-capture `SyncFile` hold.  It never supplies a fake
/// checkpoint result.
#[derive(Default)]
struct CheckpointFaults {
    fail_next_sync: AtomicBool,
    failure_hits: AtomicUsize,
    hold: Mutex<Option<(mpsc::SyncSender<()>, mpsc::Receiver<()>)>>,
}

impl CheckpointFaults {
    fn arm_sync_failure(&self) {
        assert!(
            !self.fail_next_sync.swap(true, Ordering::AcqRel),
            "fixture may arm only one SyncFile failure at a time"
        );
    }

    fn failure_hits(&self) -> usize {
        self.failure_hits.load(Ordering::Acquire)
    }

    fn arm_sync_hold(&self, entered: mpsc::SyncSender<()>, release: mpsc::Receiver<()>) {
        assert!(
            self.hold
                .lock()
                .expect("checkpoint fault hold mutex")
                .replace((entered, release))
                .is_none(),
            "fixture may hold only one real SyncFile call at a time"
        );
    }
}

impl FailureInjector for CheckpointFaults {
    fn check(&self, point: &FailurePoint) -> io::Result<()> {
        if point.step != CommitStep::SyncFile {
            return Ok(());
        }
        if self.fail_next_sync.swap(false, Ordering::AcqRel) {
            self.failure_hits.fetch_add(1, Ordering::AcqRel);
            return Err(io::Error::other("injected checkpoint SyncFile failure"));
        }
        let Some((entered, release)) = self
            .hold
            .lock()
            .expect("checkpoint fault hold mutex")
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

/// Releases a blocked production callback on every assertion or task unwind.
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

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    aof_path: PathBuf,
    aof: SharedAof,
    server: TestServer,
    store: Arc<SegmentRdbStore>,
    checkpoint: Arc<SegmentCheckpointSink>,
    writer: Arc<WriteCoordinator>,
    faults: Arc<CheckpointFaults>,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().expect("checkpoint/AOF fixture directory");
    let root = dir.path().join("segments");
    let aof_path = dir.path().join("aof.log");
    let faults = Arc::new(CheckpointFaults::default());
    let store = Arc::new(
        SegmentRdbStore::with_failure_injector_and_merge_observer(
            &root,
            faults.clone(),
            Arc::new(NoopMergeObserver),
        )
        .expect("open production observed segment store"),
    );
    let aof: SharedAof = Arc::new(Mutex::new(
        AofWriter::open(&aof_path).expect("open production AOF"),
    ));
    let engine = Arc::new(Engine::new());
    let wal: SharedWal = Arc::new(MemWal::new());
    let writer = WriteCoordinator::start_from_with_aof(wal, engine.clone(), 0, aof.clone());
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
        root,
        aof_path,
        aof,
        server: TestServer::new(router(state)).expect("production checkpoint HTTP server"),
        store,
        checkpoint,
        writer,
        faults,
    }
}

async fn create_collection(server: &TestServer) {
    server
        .put(&format!("/collections/{COLLECTION}"))
        .json(&json!({ "fields": { FIELD: { "type": "keyword" } } }))
        .await
        .assert_status_ok();
}

async fn index_keyword(server: &TestServer, value: &str) {
    server
        .post(&format!("/collections/{COLLECTION}/index"))
        .json(&json!({ "items": [{
            "external_id": ID,
            "field": FIELD,
            "value": value,
        }] }))
        .await
        .assert_status_ok();
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
    let mut actual = response
        .json::<Value>()["hits"]
        .as_array()
        .expect("Keyword query hits")
        .iter()
        .map(|hit| {
            hit["external_id"]
                .as_str()
                .expect("Keyword query external ID")
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
        "{context}: exact Keyword query must retain only the expected IDs"
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
        "{context}: CURRENT watermark must equal the captured durable cut"
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
    .expect("cold recovery HTTP server")
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
        "public /metrics must publish exactly one {name} sample: {metrics}"
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
        "{context}: public pending total must stay within the documented 256 MiB budget: total={total}"
    );
    assert!(
        high_water <= PENDING_HARD_LIMIT_BYTES,
        "{context}: public pending high water must stay within the documented 256 MiB budget: high_water={high_water}"
    );
}

async fn seed_durable_base(fixture: &Fixture, context: &str) -> u64 {
    create_collection(&fixture.server).await;
    index_keyword(&fixture.server, BASE_VALUE).await;
    let sequence = fixture.writer.applied_seq();
    checkpoint_success(fixture.checkpoint.as_ref(), context).await;
    sync_aof(fixture, context);
    assert_eq!(
        aof_sequences(&fixture.aof_path, context),
        Vec::<u64>::new(),
        "{context}: initial durable checkpoint must reclaim only its fully covered AOF frames"
    );
    sequence
}

async fn start_held_checkpoint(
    fixture: &Fixture,
    context: &str,
) -> (tokio::task::JoinHandle<anyhow::Result<bool>>, SyncRelease) {
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    fixture.faults.arm_sync_hold(entered_tx, release_rx);
    let mut release = SyncRelease(Some(release_tx));
    let mut checkpoint = tokio::spawn({
        let checkpoint = fixture.checkpoint.clone();
        async move { CheckpointSink::checkpoint_now(checkpoint.as_ref()).await }
    });
    let entered = tokio::task::spawn_blocking(move || entered_rx.recv_timeout(READY_WATCHDOG))
        .await
        .expect("checkpoint SyncFile readiness task must join");
    if !matches!(entered, Ok(())) {
        release.release();
        checkpoint.abort();
        let _ = checkpoint.await;
        panic!("{context}: checkpoint did not reach its real SyncFile hold: {entered:?}");
    }
    (checkpoint, release)
}

async fn finish_held_checkpoint(
    mut checkpoint: tokio::task::JoinHandle<anyhow::Result<bool>>,
    mut release: SyncRelease,
    context: &str,
) {
    release.release();
    let joined = match tokio::time::timeout(CHECKPOINT_WATCHDOG, &mut checkpoint).await {
        Ok(joined) => joined,
        Err(_) => {
            checkpoint.abort();
            let _ = checkpoint.await;
            panic!("{context}: held checkpoint exceeded bounded cleanup watchdog");
        }
    };
    let persisted = joined
        .unwrap_or_else(|error| panic!("{context}: checkpoint task panicked: {error}"))
        .unwrap_or_else(|error| panic!("{context}: checkpoint failed: {error:#}"));
    assert!(persisted, "{context}: configured checkpoint must persist");
}

fn compact_tmp_path(aof_path: &Path) -> PathBuf {
    let mut path = aof_path.as_os_str().to_os_string();
    path.push(".compact.tmp");
    PathBuf::from(path)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn prepublication_sync_failure_retains_frozen_cut_and_aof_suffix() {
    let fixture = fixture();
    let base_sequence = seed_durable_base(&fixture, "seed pre-publication durable base").await;

    index_keyword(&fixture.server, FROZEN_VALUE).await;
    let frozen_sequence = fixture.writer.applied_seq();
    assert!(
        frozen_sequence > base_sequence,
        "the committed frozen record must advance the real coordinator watermark"
    );
    sync_aof(&fixture, "snapshot pre-failure AOF");
    let current_before_failure = fs::read(fixture.root.join("CURRENT"))
        .expect("read CURRENT before injected pre-publication failure");
    let aof_before_failure = fs::read(&fixture.aof_path)
        .expect("read committed AOF before injected pre-publication failure");
    assert_eq!(
        aof_sequences(&fixture.aof_path, "pre-failure AOF"),
        vec![frozen_sequence],
        "the committed frozen record must be the only suffix after the durable base"
    );

    fixture.faults.arm_sync_failure();
    let failed = CheckpointSink::checkpoint_now(fixture.checkpoint.as_ref()).await;
    assert!(
        failed.is_err(),
        "the injected real SyncFile fault must fail before checkpoint publication"
    );
    assert_eq!(
        fixture.faults.failure_hits(),
        1,
        "the pre-publication failure must consume exactly one real SyncFile callback"
    );
    sync_aof(&fixture, "inspect AOF after pre-publication failure");
    assert_eq!(
        fs::read(fixture.root.join("CURRENT")).expect("read CURRENT after failed checkpoint"),
        current_before_failure,
        "pre-publication checkpoint failure must retain the exact old CURRENT bytes"
    );
    assert_eq!(
        fs::read(&fixture.aof_path).expect("read AOF after failed checkpoint"),
        aof_before_failure,
        "pre-publication checkpoint failure must retain every committed AOF byte"
    );

    let failed_cold_engine = cold_engine(&fixture, base_sequence, "failed checkpoint cold base");
    let failed_cold = cold_server(failed_cold_engine.clone(), base_sequence);
    assert_term_ids(&failed_cold, BASE_VALUE, &[ID], "failed checkpoint cold base").await;
    assert_term_ids(&failed_cold, FROZEN_VALUE, &[], "failed checkpoint excludes frozen suffix").await;
    assert_eq!(
        replay_aof_into(&failed_cold_engine, &fixture.aof_path, base_sequence)
            .expect("replay retained pre-publication AOF suffix"),
        frozen_sequence,
        "cold replay must recover the committed frozen record after the failed checkpoint"
    );
    assert_term_ids(&failed_cold, FROZEN_VALUE, &[ID], "failed checkpoint AOF recovery").await;
    assert_pending_budget(&fixture.server, "failed pre-publication recovery").await;

    // Start the retry before the later write. The production retry must reuse
    // the retained sequence, so a real SyncFile hold gives the later AOF frame
    // an unambiguous post-capture position.
    let (checkpoint, release) =
        start_held_checkpoint(&fixture, "hold retained frozen retry before newer suffix").await;
    index_keyword(&fixture.server, NEWER_VALUE).await;
    let newer_sequence = fixture.writer.applied_seq();
    assert_eq!(
        newer_sequence,
        frozen_sequence + 1,
        "the later committed suffix must follow the frozen record without a synthetic sequence"
    );
    finish_held_checkpoint(
        checkpoint,
        release,
        "release retained frozen checkpoint after newer suffix",
    )
    .await;
    sync_aof(&fixture, "inspect AOF after frozen-cut retry");
    let retry_cold_engine = cold_engine(&fixture, frozen_sequence, "retry frozen durable cut");
    let retry_cold = cold_server(retry_cold_engine.clone(), frozen_sequence);
    assert_term_ids(&retry_cold, FROZEN_VALUE, &[ID], "retry durable frozen cut").await;
    assert_eq!(
        aof_sequences(&fixture.aof_path, "retry frozen-cut suffix"),
        vec![newer_sequence],
        "retry must reclaim only the frozen cut and retain the newer committed suffix"
    );
    assert_eq!(
        replay_aof_into(&retry_cold_engine, &fixture.aof_path, frozen_sequence)
            .expect("replay newer suffix after frozen-cut retry"),
        newer_sequence,
        "retry cold replay must advance exactly through the later suffix"
    );
    assert_term_ids(&retry_cold, NEWER_VALUE, &[ID], "retry AOF recovery").await;

    checkpoint_success(fixture.checkpoint.as_ref(), "publish final newer durable cut").await;
    sync_aof(&fixture, "inspect AOF after final newer checkpoint");
    assert_eq!(
        aof_sequences(&fixture.aof_path, "final newer checkpoint"),
        Vec::<u64>::new(),
        "final checkpoint must reclaim its now-covered newer suffix"
    );
    let final_cold = cold_server(
        cold_engine(&fixture, newer_sequence, "final newer cold checkpoint"),
        newer_sequence,
    );
    assert_term_ids(&final_cold, NEWER_VALUE, &[ID], "final newer cold checkpoint").await;
    assert_pending_budget(&fixture.server, "pre-publication recovery").await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn postpublication_trim_failure_retains_aof_then_reclaims_only_covered_frames() {
    let fixture = fixture();
    let base_sequence = seed_durable_base(&fixture, "seed post-publication durable base").await;

    index_keyword(&fixture.server, CAPTURED_VALUE).await;
    let captured_sequence = fixture.writer.applied_seq();
    assert!(
        captured_sequence > base_sequence,
        "the checkpoint capture record must advance the real coordinator watermark"
    );
    let compact_tmp = compact_tmp_path(&fixture.aof_path);
    fs::create_dir(&compact_tmp).expect("create fixture-owned AOF compact-temp directory obstacle");
    assert!(
        fs::symlink_metadata(&compact_tmp)
            .expect("inspect compact-temp obstacle")
            .file_type()
            .is_dir(),
        "the trim obstacle must be a directory before checkpoint publication"
    );

    let (checkpoint, release) =
        start_held_checkpoint(&fixture, "hold first checkpoint after captured cut").await;
    index_keyword(&fixture.server, FIRST_TAIL_VALUE).await;
    let first_tail_sequence = fixture.writer.applied_seq();
    assert_eq!(
        first_tail_sequence,
        captured_sequence + 1,
        "the first tail record must arrive after the held capture without a synthetic sequence"
    );
    sync_aof(&fixture, "snapshot AOF before failed trim");
    let aof_before_failed_trim = fs::read(&fixture.aof_path)
        .expect("read full AOF before releasing failed-trim checkpoint");
    assert_eq!(
        aof_sequences(&fixture.aof_path, "full AOF before failed trim"),
        vec![captured_sequence, first_tail_sequence],
        "the full old AOF must contain both the covered capture and later suffix"
    );
    finish_held_checkpoint(checkpoint, release, "release first checkpoint with trim obstacle").await;

    sync_aof(&fixture, "inspect AOF after post-publication trim failure");
    let first_cold_engine = cold_engine(
        &fixture,
        captured_sequence,
        "post-publication trim-failure durable cut",
    );
    assert!(
        fs::symlink_metadata(&compact_tmp)
            .expect("inspect compact-temp obstacle after failed trim")
            .file_type()
            .is_dir(),
        "trim must refuse the directory obstacle without deleting or replacing it"
    );
    assert_eq!(
        fs::read(&fixture.aof_path).expect("read AOF after failed trim"),
        aof_before_failed_trim,
        "post-publication trim failure must retain the exact full old AOF bytes"
    );
    let first_cold = cold_server(first_cold_engine.clone(), captured_sequence);
    assert_term_ids(
        &first_cold,
        CAPTURED_VALUE,
        &[ID],
        "post-publication trim-failure cold durable cut",
    )
    .await;
    assert_term_ids(
        &first_cold,
        FIRST_TAIL_VALUE,
        &[],
        "post-publication trim-failure cold cut excludes later suffix",
    )
    .await;
    assert_pending_budget(&fixture.server, "failed post-publication trim recovery").await;
    assert_eq!(
        replay_aof_into(&first_cold_engine, &fixture.aof_path, captured_sequence)
            .expect("replay retained full AOF after trim failure"),
        first_tail_sequence,
        "trim failure must leave the later committed suffix replayable"
    );
    assert_term_ids(
        &first_cold,
        FIRST_TAIL_VALUE,
        &[ID],
        "post-publication trim-failure AOF recovery",
    )
    .await;

    fs::remove_dir(&compact_tmp).expect("remove only the fixture-owned trim obstacle");
    assert!(
        !compact_tmp.exists(),
        "fixture-owned compact-temp obstacle must be gone before healthy retry"
    );
    let (checkpoint, release) =
        start_held_checkpoint(&fixture, "hold healthy retry after first tail cut").await;
    index_keyword(&fixture.server, LATER_TAIL_VALUE).await;
    let later_tail_sequence = fixture.writer.applied_seq();
    assert_eq!(
        later_tail_sequence,
        first_tail_sequence + 1,
        "the later tail record must follow the healthy retry capture"
    );
    finish_held_checkpoint(checkpoint, release, "release healthy retry checkpoint").await;
    sync_aof(&fixture, "inspect AOF after healthy retry trim");
    assert_eq!(
        aof_sequences(&fixture.aof_path, "healthy retry suffix"),
        vec![later_tail_sequence],
        "healthy retry must reclaim covered frames and retain only the later suffix"
    );
    let retry_cold_engine = cold_engine(
        &fixture,
        first_tail_sequence,
        "healthy retry durable cut",
    );
    let retry_cold = cold_server(retry_cold_engine.clone(), first_tail_sequence);
    assert_term_ids(
        &retry_cold,
        FIRST_TAIL_VALUE,
        &[ID],
        "healthy retry cold durable cut",
    )
    .await;
    assert_eq!(
        replay_aof_into(&retry_cold_engine, &fixture.aof_path, first_tail_sequence)
            .expect("replay later suffix after healthy retry"),
        later_tail_sequence,
        "healthy retry must retain the later suffix for strict replay"
    );
    assert_term_ids(&retry_cold, LATER_TAIL_VALUE, &[ID], "healthy retry AOF recovery").await;

    checkpoint_success(fixture.checkpoint.as_ref(), "publish final later-tail checkpoint").await;
    sync_aof(&fixture, "inspect AOF after final later-tail checkpoint");
    assert_eq!(
        aof_sequences(&fixture.aof_path, "final later-tail checkpoint"),
        Vec::<u64>::new(),
        "final checkpoint must reclaim the fully covered later suffix"
    );
    let final_cold = cold_server(
        cold_engine(
            &fixture,
            later_tail_sequence,
            "final post-publication cold checkpoint",
        ),
        later_tail_sequence,
    );
    assert_term_ids(
        &final_cold,
        LATER_TAIL_VALUE,
        &[ID],
        "final post-publication cold checkpoint",
    )
    .await;
    assert_pending_budget(&fixture.server, "post-publication trim recovery").await;
}
