//! Three-phase snapshot contract for `RaftHost`.
//!
//! A state machine may first run expensive snapshot preflight without the host
//! serialization lease. The returned preparation captures one exact Raft record
//! boundary while the host lease is held. Its writer then exports that immutable
//! capture after the host lease is released.
//!
//! # Facets
//!
//! - Behavior: preflight asserts at `libs/raft-runtime/e2e/snapshot_preflight_release.rs:471`, `:490`, `:495`,
//!   `:511`, `:517`, `:527`, and `:528`; periodic capture asserts at `:557`, `:584`, `:588`, `:601`, `:605`, and `:609`.
//!   Bounded capture and export assert at `:650`, `:655`, `:659`, `:679`, `:690`, `:721`, `:726`, `:752`, `:756`, `:760`, and `:764`.
//! - Security: `libs/raft-runtime/e2e/snapshot_preflight_release.rs:819`, `:823`, `:833`, and `:837` assert callback failures; errors from the
//!   public `RaftStateMachine::preflight_snapshot`, `SnapshotPreparation`, and `PreparedSnapshot`
//!   boundary must refuse output, avoid mutation, and retain the old Raft prefix.
//! - Performance: gap — `libs/raft-runtime/src/host.rs:1923-1928` and `:2066-2129` reach
//!   `apps/lumen/src/bin/lumen.rs:3879`, but `libs/raft-runtime/README.md:74-95` has no budget.
//!
//! Declared gate: `cargo test -p raft-runtime`.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use raft_runtime::{
    FsyncPolicy, HostConfig, Index, Membership, PreparedSnapshot, RaftHost, RaftStateMachine,
    RaftStore, SnapshotPreparation, Term,
};
use tempfile::TempDir;
use tokio::sync::oneshot;

const WAIT: Duration = Duration::from_secs(3);

struct SnapshotCore {
    applied: AtomicU64,
    records: Mutex<Vec<(Index, Vec<u8>)>>,
}

impl SnapshotCore {
    fn new() -> Self {
        Self {
            applied: AtomicU64::new(0),
            records: Mutex::new(Vec::new()),
        }
    }

    fn apply(&self, index: Index, command: &[u8]) -> anyhow::Result<()> {
        let mut records = self.records.lock().expect("snapshot records mutex poisoned");
        if let Some((previous, _)) = records.last() {
            anyhow::ensure!(
                *previous < index,
                "state-machine apply indices must increase: {previous} then {index}"
            );
        }
        records.push((index, command.to_vec()));
        drop(records);
        self.applied.store(index, Ordering::Release);
        Ok(())
    }

    fn applied_index(&self) -> Index {
        self.applied.load(Ordering::Acquire)
    }

    fn bytes_through(&self, index: Index) -> anyhow::Result<Vec<u8>> {
        let records = self.records.lock().expect("snapshot records mutex poisoned");
        if index > 0 {
            let last = records
                .iter()
                .filter(|(record_index, _)| *record_index <= index)
                .last()
                .map(|(record_index, _)| *record_index);
            anyhow::ensure!(
                last == Some(index),
                "state machine cannot serialize missing prefix {index}; last retained record is {last:?}"
            );
        }

        let mut bytes = index.to_le_bytes().to_vec();
        for (record_index, command) in records.iter().filter(|(at, _)| *at <= index) {
            bytes.extend_from_slice(&record_index.to_le_bytes());
            bytes.extend_from_slice(&(command.len() as u64).to_le_bytes());
            bytes.extend_from_slice(command);
        }
        Ok(bytes)
    }

    fn snapshot(&self, writer: &mut dyn Write) -> anyhow::Result<()> {
        writer.write_all(&self.bytes_through(self.applied_index())?)?;
        Ok(())
    }

    fn restore(&self, reader: &mut dyn Read) -> anyhow::Result<()> {
        let mut index = [0_u8; 8];
        reader.read_exact(&mut index)?;
        self.applied
            .store(u64::from_le_bytes(index), Ordering::Release);
        Ok(())
    }
}

struct PreflightGate {
    entered: Mutex<Option<oneshot::Sender<()>>>,
    released: Mutex<bool>,
    wake: Condvar,
}

impl PreflightGate {
    fn new() -> (Arc<Self>, oneshot::Receiver<()>) {
        let (entered_tx, entered_rx) = oneshot::channel();
        (
            Arc::new(Self {
                entered: Mutex::new(Some(entered_tx)),
                released: Mutex::new(false),
                wake: Condvar::new(),
            }),
            entered_rx,
        )
    }

    fn enter_and_wait(&self) {
        self.entered
            .lock()
            .expect("preflight entered mutex poisoned")
            .take()
            .expect("preflight is entered once")
            .send(())
            .expect("test keeps the preflight observation receiver alive");
        let mut released = self.released.lock().expect("preflight release mutex poisoned");
        while !*released {
            released = self.wake.wait(released).expect("preflight release mutex poisoned");
        }
    }

    fn release(&self) {
        *self
            .released
            .lock()
            .expect("preflight release mutex poisoned") = true;
        self.wake.notify_all();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CaptureObservation {
    requested: Index,
    observed_applied: Index,
}

struct CaptureGate {
    entered: Mutex<Option<oneshot::Sender<CaptureObservation>>>,
    released: Mutex<bool>,
    wake: Condvar,
}

impl CaptureGate {
    fn new() -> (Arc<Self>, oneshot::Receiver<CaptureObservation>) {
        let (entered_tx, entered_rx) = oneshot::channel();
        (
            Arc::new(Self {
                entered: Mutex::new(Some(entered_tx)),
                released: Mutex::new(false),
                wake: Condvar::new(),
            }),
            entered_rx,
        )
    }

    fn capture_and_wait(&self, observation: CaptureObservation) {
        self.entered
            .lock()
            .expect("capture entered mutex poisoned")
            .take()
            .expect("capture is entered once")
            .send(observation)
            .expect("test keeps the capture observation receiver alive");
        let mut released = self.released.lock().expect("capture release mutex poisoned");
        while !*released {
            released = self.wake.wait(released).expect("capture release mutex poisoned");
        }
    }

    fn release(&self) {
        *self.released.lock().expect("capture release mutex poisoned") = true;
        self.wake.notify_all();
    }
}

struct ExportGate {
    entered: Mutex<Option<oneshot::Sender<()>>>,
    released: Mutex<bool>,
    wake: Condvar,
}

impl ExportGate {
    fn new() -> (Arc<Self>, oneshot::Receiver<()>) {
        let (entered_tx, entered_rx) = oneshot::channel();
        (
            Arc::new(Self {
                entered: Mutex::new(Some(entered_tx)),
                released: Mutex::new(false),
                wake: Condvar::new(),
            }),
            entered_rx,
        )
    }

    fn export_and_wait(&self) {
        self.entered
            .lock()
            .expect("export entered mutex poisoned")
            .take()
            .expect("prepared export is entered once")
            .send(())
            .expect("test keeps the export observation receiver alive");
        let mut released = self.released.lock().expect("export release mutex poisoned");
        while !*released {
            released = self.wake.wait(released).expect("export release mutex poisoned");
        }
    }

    fn release(&self) {
        *self.released.lock().expect("export release mutex poisoned") = true;
        self.wake.notify_all();
    }
}

/// Opens a blocked fixture on unwinding so a failed assertion cannot strand a
/// host worker on a test-owned condition variable.
struct ReleasePreflight(Arc<PreflightGate>);

impl Drop for ReleasePreflight {
    fn drop(&mut self) {
        self.0.release();
    }
}

/// Opens a blocked fixture on unwinding so a failed assertion cannot strand a
/// host worker on a test-owned condition variable.
struct ReleaseCapture(Arc<CaptureGate>);

impl Drop for ReleaseCapture {
    fn drop(&mut self) {
        self.0.release();
    }
}

/// Opens a blocked fixture on unwinding so a failed assertion cannot strand a
/// host worker on a test-owned condition variable.
struct ReleaseExport(Arc<ExportGate>);

impl Drop for ReleaseExport {
    fn drop(&mut self) {
        self.0.release();
    }
}

#[derive(Clone)]
enum PreflightMode {
    Immediate,
    Block(Arc<PreflightGate>),
    Fail,
}

#[derive(Clone)]
enum CaptureMode {
    Immediate,
    Block(Arc<CaptureGate>),
    Fail,
}

#[derive(Clone)]
enum ExportMode {
    Immediate,
    Block(Arc<ExportGate>),
    Fail,
}

struct PreparedSnapshotStateMachine {
    core: Arc<SnapshotCore>,
    preflight_mode: PreflightMode,
    capture_mode: CaptureMode,
    export_mode: ExportMode,
    capture_requests: Arc<Mutex<Vec<Index>>>,
}

impl PreparedSnapshotStateMachine {
    fn new(
        preflight_mode: PreflightMode,
        capture_mode: CaptureMode,
        export_mode: ExportMode,
    ) -> Arc<Self> {
        Arc::new(Self {
            core: Arc::new(SnapshotCore::new()),
            preflight_mode,
            capture_mode,
            export_mode,
            capture_requests: Arc::new(Mutex::new(Vec::new())),
        })
    }
}

struct PreparedSnapshotPreparation {
    core: Arc<SnapshotCore>,
    capture_mode: CaptureMode,
    export_mode: ExportMode,
    capture_requests: Arc<Mutex<Vec<Index>>>,
}

struct CapturedSnapshot {
    bytes: Vec<u8>,
    export_mode: ExportMode,
}

impl PreparedSnapshot for CapturedSnapshot {
    fn write_to(self: Box<Self>, writer: &mut dyn Write) -> anyhow::Result<()> {
        match &self.export_mode {
            ExportMode::Immediate => {}
            ExportMode::Block(gate) => gate.export_and_wait(),
            ExportMode::Fail => anyhow::bail!("injected prepared snapshot export failure"),
        }
        writer.write_all(&self.bytes)?;
        Ok(())
    }
}

impl SnapshotPreparation for PreparedSnapshotPreparation {
    fn capture_at(self: Box<Self>, index: Index) -> anyhow::Result<Box<dyn PreparedSnapshot>> {
        if matches!(&self.capture_mode, CaptureMode::Fail) {
            anyhow::bail!("injected prepared snapshot capture failure");
        }

        let observed_applied = self.core.applied_index();
        anyhow::ensure!(
            observed_applied == index,
            "prepared snapshot must capture exact prefix {index}; current applied index is {observed_applied}"
        );
        let bytes = self.core.bytes_through(index)?;
        self.capture_requests
            .lock()
            .expect("capture requests mutex poisoned")
            .push(index);

        if let CaptureMode::Block(gate) = &self.capture_mode {
            gate.capture_and_wait(CaptureObservation {
                requested: index,
                observed_applied,
            });
        }

        Ok(Box::new(CapturedSnapshot {
            bytes,
            export_mode: self.export_mode.clone(),
        }))
    }
}

impl RaftStateMachine for PreparedSnapshotStateMachine {
    fn apply(&self, index: Index, command: &[u8]) -> anyhow::Result<()> {
        self.core.apply(index, command)
    }

    fn snapshot(&self, writer: &mut dyn Write) -> anyhow::Result<()> {
        self.core.snapshot(writer)
    }

    fn snapshot_at(&self, _index: Index, _writer: &mut dyn Write) -> anyhow::Result<()> {
        anyhow::bail!("prepared snapshot opt-in must not fall back to snapshot_at")
    }

    fn preflight_snapshot(&self) -> anyhow::Result<Option<Box<dyn SnapshotPreparation>>> {
        match &self.preflight_mode {
            PreflightMode::Immediate => {}
            PreflightMode::Block(gate) => gate.enter_and_wait(),
            PreflightMode::Fail => anyhow::bail!("injected snapshot preflight failure"),
        }
        Ok(Some(Box::new(PreparedSnapshotPreparation {
            core: Arc::clone(&self.core),
            capture_mode: self.capture_mode.clone(),
            export_mode: self.export_mode.clone(),
            capture_requests: Arc::clone(&self.capture_requests),
        })))
    }

    fn restore(&self, reader: &mut dyn Read) -> anyhow::Result<()> {
        self.core.restore(reader)
    }

    fn applied_index(&self) -> Index {
        self.core.applied_index()
    }
}

fn spawn_host<S>(state_machine: Arc<S>) -> (TempDir, Arc<RaftHost>)
where
    S: RaftStateMachine,
{
    let data = tempfile::tempdir().expect("create raft store directory");
    let host = Arc::new(RaftHost::spawn(
        0,
        Membership {
            voters: vec![0],
            learners: Vec::new(),
        },
        HashMap::new(),
        RaftStore::open(data.path().to_str().expect("temporary path is UTF-8"), 0, FsyncPolicy::Always)
            .expect("open raft store"),
        state_machine as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));
    (data, host)
}

async fn wait_for_committed(host: &RaftHost, expected: Index) {
    tokio::time::timeout(WAIT, async {
        loop {
            let persisted = host
                .store()
                .load()
                .expect("read raft store")
                .expect("host persists a Raft state");
            if persisted.commit_index >= expected {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("the later proposal commits while the test gate is held");
}

async fn wait_for_signal<T>(receiver: oneshot::Receiver<T>) -> Option<T> {
    tokio::time::timeout(WAIT, receiver).await.ok()?.ok()
}

fn resident_term_at(host: &RaftHost, index: Index) -> Term {
    host.store()
        .load()
        .expect("read Raft state for the requested snapshot term")
        .expect("host persists a Raft state")
        .log
        .iter()
        .find(|entry| entry.index == index)
        .map(|entry| entry.term)
        .expect("the requested snapshot index remains a resident Raft log entry")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn preflight_does_not_hold_apply_and_stale_explicit_prefix_refuses() {
    let (preflight_gate, preflight_rx) = PreflightGate::new();
    let state_machine = PreparedSnapshotStateMachine::new(
        PreflightMode::Block(Arc::clone(&preflight_gate)),
        CaptureMode::Immediate,
        ExportMode::Immediate,
    );
    let (_data, host) = spawn_host(Arc::clone(&state_machine));
    let requested = host
        .propose(b"requested-prefix".to_vec())
        .await
        .expect("requested prefix applies");
    let _release_preflight = ReleasePreflight(Arc::clone(&preflight_gate));

    let snapshot_host = Arc::clone(&host);
    let snapshot = tokio::spawn(async move {
        snapshot_host
            .snapshot_and_compact_through(requested)
            .await
    });
    assert!(
        wait_for_signal(preflight_rx).await.is_some(),
        "the host must call the snapshot preflight hook before capture"
    );
    assert!(
        !snapshot.is_finished(),
        "the snapshot remains in the explicitly blocked preflight"
    );

    let later_host = Arc::clone(&host);
    let later = tokio::time::timeout(
        WAIT,
        tokio::spawn(async move { later_host.propose(b"later-record".to_vec()).await }),
    )
    .await
    .expect("preflight must not hold host apply serialization")
    .expect("later proposal task does not panic")
    .expect("later proposal applies while preflight is blocked");
    assert!(later > requested, "the later record receives a later index");
    assert_eq!(
        state_machine.applied_index(),
        later,
        "state-machine apply proceeds while preflight is blocked"
    );
    assert_eq!(
        *host.applied_watch().borrow(),
        later,
        "the public applied-index read advances while preflight is blocked"
    );
    assert!(
        !snapshot.is_finished(),
        "preflight remains blocked after later apply and its public read complete"
    );

    preflight_gate.release();
    let error = tokio::time::timeout(WAIT, snapshot)
        .await
        .expect("stale explicit snapshot returns after preflight opens")
        .expect("snapshot task does not panic")
        .expect_err("a later applied record must not be captured under the older requested index");
    assert!(
        error
            .to_string()
            .contains("prepared snapshot must capture exact prefix"),
        "the stale explicit target fails closed instead of stamping newer bytes with its older index: {error:#}"
    );
    assert_eq!(
        host.snapshot_index().await,
        0,
        "a stale explicit target does not publish a snapshot"
    );
    let persisted = host
        .store()
        .load()
        .expect("read persisted state after stale explicit target")
        .expect("host persists a Raft state");
    assert_eq!(persisted.snapshot_index, 0);
    assert_eq!(
        persisted
            .log
            .iter()
            .map(|entry| entry.index)
            .collect::<Vec<_>>(),
        vec![requested, later],
        "the stale requested prefix remains recoverable in the Raft log"
    );
    host.shutdown().await.expect("shutdown host after stale refusal");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn periodic_snapshot_samples_current_after_preflight_and_compacts_that_exact_prefix() {
    let (preflight_gate, preflight_rx) = PreflightGate::new();
    let state_machine = PreparedSnapshotStateMachine::new(
        PreflightMode::Block(Arc::clone(&preflight_gate)),
        CaptureMode::Immediate,
        ExportMode::Immediate,
    );
    let (_data, host) = spawn_host(Arc::clone(&state_machine));
    let old = host
        .propose(b"before-periodic-preflight".to_vec())
        .await
        .expect("initial periodic command applies");
    let _release_preflight = ReleasePreflight(Arc::clone(&preflight_gate));

    let snapshot_host = Arc::clone(&host);
    let snapshot = tokio::spawn(async move { snapshot_host.snapshot_and_compact().await });
    assert!(
        wait_for_signal(preflight_rx).await.is_some(),
        "periodic snapshot calls preflight before choosing its capture index"
    );

    let later = host
        .propose(b"after-periodic-preflight".to_vec())
        .await
        .expect("later command applies while periodic preflight is blocked");
    assert!(later > old, "the later command has a later index");
    assert_eq!(
        state_machine.applied_index(),
        later,
        "the later command becomes the current applied head before periodic capture"
    );
    let expected_bytes = state_machine
        .core
        .bytes_through(later)
        .expect("latest periodic prefix bytes are available");
    let expected_term = resident_term_at(&host, later);

    preflight_gate.release();
    let compacted = tokio::time::timeout(WAIT, snapshot)
        .await
        .expect("periodic snapshot completes after preflight opens")
        .expect("periodic snapshot task does not panic")
        .expect("periodic snapshot captures the current applied prefix");
    assert_eq!(
        compacted, later,
        "periodic snapshot uses the current index sampled after preflight, not its old preflight index"
    );
    assert_eq!(
        *state_machine
            .capture_requests
            .lock()
            .expect("capture requests mutex poisoned"),
        vec![later],
        "periodic capture receives the exact current index"
    );
    let persisted = host
        .store()
        .load()
        .expect("read periodic snapshot state")
        .expect("host persists a Raft state");
    assert_eq!(
        persisted.snapshot_index, later,
        "the stored periodic snapshot index is the post-preflight current index"
    );
    assert_eq!(
        persisted.snapshot_term, expected_term,
        "the stored periodic term belongs to the same post-preflight capture index"
    );
    assert_eq!(
        persisted.snapshot, expected_bytes,
        "the stored periodic bytes match the same post-preflight current prefix"
    );
    assert!(
        persisted.log.is_empty(),
        "compacting the current periodic head leaves no older Raft log prefix"
    );
    host.shutdown().await.expect("shutdown periodic host");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn capture_keeps_its_exact_cut_and_releases_apply_before_export() {
    let (capture_gate, capture_rx) = CaptureGate::new();
    let (export_gate, export_rx) = ExportGate::new();
    let state_machine = PreparedSnapshotStateMachine::new(
        PreflightMode::Immediate,
        CaptureMode::Block(Arc::clone(&capture_gate)),
        ExportMode::Block(Arc::clone(&export_gate)),
    );
    let (_data, host) = spawn_host(Arc::clone(&state_machine));
    let capture_index = host
        .propose(b"captured-prefix".to_vec())
        .await
        .expect("the capture command applies");
    let expected_bytes = state_machine
        .core
        .bytes_through(capture_index)
        .expect("capture bytes are available");
    let capture_term = resident_term_at(&host, capture_index);
    let _release_capture = ReleaseCapture(Arc::clone(&capture_gate));
    let _release_export = ReleaseExport(Arc::clone(&export_gate));

    let snapshot_host = Arc::clone(&host);
    let snapshot = tokio::spawn(async move {
        snapshot_host
            .snapshot_and_compact_through(capture_index)
            .await
    });

    let observed = wait_for_signal(capture_rx).await;
    assert!(
        observed.is_some(),
        "the host must call capture_at for a prepared snapshot"
    );
    let observed = observed.expect("the assertion above proves capture_at was called");
    assert_eq!(
        observed.requested, capture_index,
        "the host passes the explicit compaction target to capture_at"
    );
    assert_eq!(
        observed.observed_applied, capture_index,
        "capture sees the exact applied cut before a later record can interleave"
    );
    assert_eq!(
        *state_machine
            .capture_requests
            .lock()
            .expect("capture requests mutex poisoned"),
        vec![capture_index],
        "one prepared capture receives one exact target"
    );

    let during_capture_host = Arc::clone(&host);
    let during_capture = tokio::spawn(async move {
        during_capture_host
            .propose(b"committed-during-capture".to_vec())
            .await
    });
    wait_for_committed(&host, capture_index + 1).await;
    // A durable commit can precede dispatch to the ordered apply worker. An
    // immediate floor read would miss a host that released its lease too soon.
    // Keep capture blocked for the confirmation interval, then assert the
    // recorded observation only after both gates and the host are cleaned up.
    let mut applied = host.applied_watch();
    let applied_during_capture = tokio::time::timeout(WAIT, async {
        loop {
            if *applied.borrow_and_update() > capture_index {
                return;
            }
            applied.changed().await.expect("host keeps its applied watch");
        }
    })
    .await
    .is_ok();
    let capture_floor = state_machine.applied_index();
    let completed_during_capture = during_capture.is_finished();

    capture_gate.release();
    assert!(
        wait_for_signal(export_rx).await.is_some(),
        "prepared export starts after the capture lease is released"
    );
    let during_capture_index = tokio::time::timeout(WAIT, during_capture)
        .await
        .expect("the fixed capture releases the queued apply even after export starts")
        .expect("queued proposal task does not panic")
        .expect("queued proposal applies after capture");
    assert!(
        !snapshot.is_finished(),
        "the export gate still holds the snapshot operation after its exact capture"
    );

    let during_export_host = Arc::clone(&host);
    let during_export_index = tokio::time::timeout(
        WAIT,
        tokio::spawn(async move {
            during_export_host
                .propose(b"committed-during-export".to_vec())
                .await
        }),
    )
    .await
    .expect("a later proposal applies while prepared export is held")
    .expect("later proposal task does not panic")
    .expect("later proposal completes while prepared export is held");
    assert!(
        during_export_index > during_capture_index,
        "the later proposal receives a later Raft index"
    );
    assert_eq!(
        state_machine.applied_index(),
        during_export_index,
        "the state machine applies the later record before export unblocks"
    );
    assert_eq!(
        *host.applied_watch().borrow(),
        during_export_index,
        "the public applied-index read observes the later committed record before export unblocks"
    );
    assert!(
        !snapshot.is_finished(),
        "the export remains blocked even though later apply and its public read complete"
    );

    export_gate.release();
    let compacted = tokio::time::timeout(WAIT, snapshot)
        .await
        .expect("snapshot completes when export unblocks")
        .expect("snapshot task does not panic")
        .expect("prepared snapshot compacts its exact captured prefix");
    assert_eq!(
        compacted, capture_index,
        "the host compacts the same index that capture_at fixed"
    );

    let persisted = host
        .store()
        .load()
        .expect("read persisted host state")
        .expect("host persists a Raft state");
    assert_eq!(
        persisted.snapshot_index, capture_index,
        "the stored snapshot index remains the captured cut"
    );
    assert_eq!(
        persisted.snapshot_term, capture_term,
        "the stored snapshot term remains paired with the captured cut"
    );
    assert_eq!(
        persisted.snapshot, expected_bytes,
        "later apply cannot change bytes captured before export"
    );
    assert_eq!(
        persisted
            .log
            .iter()
            .map(|entry| entry.index)
            .collect::<Vec<_>>(),
        vec![during_capture_index, during_export_index],
        "compaction keeps the later suffix outside the captured prefix"
    );
    host.shutdown().await.expect("shutdown healthy host");
    assert!(
        !applied_during_capture,
        "a committed later record cannot apply while capture_at holds the capture lease"
    );
    assert_eq!(capture_floor, capture_index, "capture excludes later applied records");
    assert!(
        !completed_during_capture,
        "read-your-write propose remains pending until the capture lease is released"
    );
}

#[derive(Clone, Copy)]
enum FailureStage {
    Preflight,
    Capture,
    Export,
}

async fn assert_failed_prepared_snapshot_retains_prefix(stage: FailureStage) {
    let (preflight_mode, capture_mode, export_mode, expected_error) = match stage {
        FailureStage::Preflight => (
            PreflightMode::Fail,
            CaptureMode::Immediate,
            ExportMode::Immediate,
            "injected snapshot preflight failure",
        ),
        FailureStage::Capture => (
            PreflightMode::Immediate,
            CaptureMode::Fail,
            ExportMode::Immediate,
            "injected prepared snapshot capture failure",
        ),
        FailureStage::Export => (
            PreflightMode::Immediate,
            CaptureMode::Immediate,
            ExportMode::Fail,
            "injected prepared snapshot export failure",
        ),
    };
    let state_machine = PreparedSnapshotStateMachine::new(preflight_mode, capture_mode, export_mode);
    let (_data, host) = spawn_host(state_machine);
    let first = host
        .propose(b"prefix-retained-on-failure".to_vec())
        .await
        .expect("first command applies");
    let second = host
        .propose(b"suffix-retained-on-failure".to_vec())
        .await
        .expect("second command applies");

    let error = host
        .snapshot_and_compact_through(second)
        .await
        .expect_err("a failed prepared snapshot phase must refuse compaction");
    assert!(
        error.to_string().contains(expected_error),
        "the host exposes the prepared snapshot phase failure: {error:#}"
    );
    assert_eq!(
        host.snapshot_index().await,
        0,
        "failed preflight, capture, or export does not publish a snapshot index"
    );
    let persisted = host
        .store()
        .load()
        .expect("read persisted state after prepared snapshot failure")
        .expect("host persists a Raft state");
    assert_eq!(
        persisted.snapshot_index, 0,
        "failed preflight, capture, or export does not compact a partial snapshot"
    );
    assert_eq!(
        persisted
            .log
            .iter()
            .map(|entry| entry.index)
            .collect::<Vec<_>>(),
        vec![first, second],
        "the unresolved old prefix remains recoverable in the Raft log"
    );
    host.shutdown().await.expect("shutdown host after refusal");
}

#[tokio::test]
async fn prepared_snapshot_failures_refuse_compaction_and_keep_the_old_prefix() {
    assert_failed_prepared_snapshot_retains_prefix(FailureStage::Preflight).await;
    assert_failed_prepared_snapshot_retains_prefix(FailureStage::Capture).await;
    assert_failed_prepared_snapshot_retains_prefix(FailureStage::Export).await;
}
