//! Capacity maintenance has its own native worker. Committed apply retains its
//! source and reservation while waiting outside every Engine/apply lock.
//! A weak registry avoids an Engine -> checkpoint sink -> Engine ownership cycle.

use crate::segment_checkpoint::SegmentCheckpointSink;
use crate::segment_rdb::SegmentRdbStore;
use crate::storage::Engine;
use anyhow::{Result, anyhow};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, Weak};
use std::time::Duration;

#[path = "segment_save_gate.rs"]
mod publication_gate;

#[derive(Default)]
pub(crate) struct Registry {
    publication: Arc<publication_gate::SaveGate>,
    registration: Mutex<Registration>,
    frozen_windows: AtomicUsize,
}

#[derive(Default)]
struct Registration {
    token: u64,
    endpoint: Weak<Endpoint>,
    configured: bool,
}

/// At most one extra catalog delta may replace a captured scalar prefix. While
/// any cut is frozen, leave a slot for that delta. The marker is installed under
/// capture and lives with frozen ownership, including a failed save's retry.
pub(crate) struct FrozenWindow(Arc<Registry>);
impl Drop for FrozenWindow {
    fn drop(&mut self) {
        self.0.frozen_windows.fetch_sub(1, Ordering::AcqRel);
    }
}

#[derive(Clone)]
pub(crate) struct PublicationFence {
    registry: Weak<Registry>,
    token: u64,
}

#[derive(Debug, thiserror::Error)]
#[error("checkpoint maintenance owner was superseded before CURRENT publication")]
pub(crate) struct Superseded;

pub(crate) struct PublicationPermit {
    _permit: publication_gate::SavePermit,
}

impl PublicationFence {
    pub(crate) fn acquire(&self) -> Result<PublicationPermit> {
        let registry = self.registry.upgrade().ok_or(Superseded)?;
        let permit = registry.publication.lock_owned();
        let current = registry
            .registration
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .token;
        if current != self.token {
            return Err(Superseded.into());
        }
        Ok(PublicationPermit { _permit: permit })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Work {
    Checkpoint,
    Merge,
}

#[derive(Default)]
struct Requests {
    requested: u64,
    completed: u64,
    checkpoint: bool,
    merge: bool,
    stopped: bool,
    error: Option<String>,
    #[cfg(test)]
    operations: Vec<Work>,
}

pub(crate) struct Endpoint {
    requests: Mutex<Requests>,
    changed: Condvar,
    fence: PublicationFence,
}

impl Endpoint {
    fn stop(&self) {
        self.requests
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .stopped = true;
        self.changed.notify_all();
    }

    fn is_stopped(&self) -> bool {
        self.requests
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .stopped
    }

    /// `Ok` means recheck the live view. Another owner may have replaced this
    /// endpoint; no record result or watermark is manufactured here.
    pub(crate) fn wait_for(&self, work: Work) -> Result<()> {
        let mut state = self.requests.lock().unwrap_or_else(|p| p.into_inner());
        if state.stopped {
            return Ok(());
        }
        state.requested = state
            .requested
            .checked_add(1)
            .ok_or_else(|| anyhow!("capacity work revision overflow"))?;
        let revision = state.requested;
        match work {
            Work::Checkpoint => state.checkpoint = true,
            Work::Merge => state.merge = true,
        }
        self.changed.notify_all();
        while !state.stopped && state.completed < revision {
            state = self.changed.wait(state).unwrap_or_else(|p| p.into_inner());
        }
        if !state.stopped {
            if let Some(error) = &state.error {
                return Err(anyhow!(error.clone()));
            }
        }
        Ok(())
    }
}

impl Registry {
    pub(crate) fn owner(&self) -> Option<Arc<Endpoint>> {
        let owner = self
            .registration
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .endpoint
            .upgrade()?;
        let stopped = owner
            .requests
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .stopped;
        (!stopped).then_some(owner)
    }

    pub(crate) fn freeze(self: &Arc<Self>) -> FrozenWindow {
        self.frozen_windows.fetch_add(1, Ordering::AcqRel);
        FrozenWindow(self.clone())
    }

    pub(crate) fn append_limit(&self) -> usize {
        crate::composed_segment::MAX_INCREMENTAL_LAYERS
            - usize::from(self.frozen_windows.load(Ordering::Acquire) != 0)
    }
}

/// Held by the caller, never by Engine. The worker holds its endpoint and sink
/// only until stop and any already started operation finish.
pub(crate) struct Owner {
    endpoint: Arc<Endpoint>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl Owner {
    pub(crate) fn start(
        sink: Arc<SegmentCheckpointSink>,
        configured: bool,
    ) -> Result<Option<Self>> {
        let registry = &sink.engine.layer_maintenance;
        let _publication = registry.publication.lock_owned();
        let mut registration = registry
            .registration
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if let Some(active) = registration.endpoint.upgrade() {
            if !active
                .requests
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .stopped
                && !configured
            {
                return Ok(None);
            }
        }
        let token = registration
            .token
            .checked_add(1)
            .ok_or_else(|| anyhow!("capacity owner revision overflow"))?;
        let fence = PublicationFence {
            registry: Arc::downgrade(registry),
            token,
        };
        let endpoint = Arc::new(Endpoint {
            requests: Mutex::new(Requests::default()),
            changed: Condvar::new(),
            fence: fence.clone(),
        });
        let worker_endpoint = endpoint.clone();
        let worker_sink = sink.clone();
        let worker = std::thread::Builder::new()
            .name("lumen-layer-capacity".into())
            .spawn(move || run(worker_sink, worker_endpoint))?;
        if let Some(old) = registration.endpoint.upgrade() {
            old.stop();
        }
        registration.token = token;
        registration.endpoint = Arc::downgrade(&endpoint);
        registration.configured = configured;
        Ok(Some(Self {
            endpoint,
            worker: Some(worker),
        }))
    }

    pub(crate) fn fence(&self) -> PublicationFence {
        self.endpoint.fence.clone()
    }

    fn endpoint(&self) -> Arc<Endpoint> {
        self.endpoint.clone()
    }

    pub(crate) fn stop(&mut self) {
        self.endpoint.stop();
    }

    pub(crate) fn join(&mut self) -> Result<()> {
        self.stop();
        if let Some(worker) = self.worker.take() {
            worker
                .join()
                .map_err(|_| anyhow!("capacity worker panicked"))?;
        }
        Ok(())
    }
}

impl Drop for Owner {
    fn drop(&mut self) {
        // Never block an async executor on held file IO. The native worker owns
        // the sink/root until completion. Explicit shutdown may join it.
        self.stop();
    }
}

fn run(sink: Arc<SegmentCheckpointSink>, endpoint: Arc<Endpoint>) {
    loop {
        let (revision, checkpoint, merge) = {
            let mut state = endpoint.requests.lock().unwrap_or_else(|p| p.into_inner());
            while !state.stopped && state.completed == state.requested {
                state = endpoint
                    .changed
                    .wait(state)
                    .unwrap_or_else(|p| p.into_inner());
            }
            if state.stopped {
                return;
            }
            let work = (state.requested, state.checkpoint, state.merge);
            state.checkpoint = false;
            state.merge = false;
            work
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<()> {
            #[cfg(test)]
            {
                let mut state = endpoint.requests.lock().unwrap_or_else(|p| p.into_inner());
                if checkpoint {
                    state.operations.push(Work::Checkpoint);
                }
                if merge {
                    state.operations.push(Work::Merge);
                }
            }
            let store = sink
                .store
                .as_ref()
                .clone()
                .with_publication_fence(endpoint.fence.clone());
            if checkpoint || (merge && !store.has_current_generation()?) {
                sink.checkpoint_sync(&store)?;
            }
            if merge {
                let revision = store.request_capacity_merge(&sink.engine)?;
                store.wait_for_capacity_merge_progress(revision, Duration::from_secs(60))?;
            }
            Ok(())
        }))
        .unwrap_or_else(|_| {
            sink.engine.capture_barrier.apply().mark_uncertain();
            Err(anyhow!(
                "capacity worker panicked; committed source remains retained"
            ))
        });
        let superseded = endpoint.fence.acquire().is_err();
        let retry = result.is_err() && !superseded && !sink.engine.capture_barrier.is_uncertain();
        let mut state = endpoint.requests.lock().unwrap_or_else(|p| p.into_inner());
        if retry && !state.stopped {
            // A failed pre-publication save still owns the frozen cut. Keep the
            // request outstanding and the apply waiter asleep while this same
            // independent worker retries. New requests coalesce into its flags.
            state.checkpoint |= checkpoint;
            state.merge |= merge;
            tracing::warn!(error = %result.as_ref().unwrap_err(), "capacity maintenance will retry with retained committed input");
            let _ = endpoint
                .changed
                .wait_timeout(state, Duration::from_secs(1))
                .unwrap_or_else(|p| p.into_inner());
            continue;
        }
        state.completed = revision;
        state.error = result.as_ref().err().map(|error| format!("{error:#}"));
        // A superseded owner can return without a record error. Its waiters
        // re-read the registry and submit to the replacement owner.
        if superseded {
            state.stopped = true;
        }
        endpoint.changed.notify_all();
        if state.stopped {
            return;
        }
    }
}

/// A fallback needs no Tokio runtime. This native relay only requests the
/// existing `Owner` worker; `run` remains the sole checkpoint/save path.
struct BudgetRelay {
    stop: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl BudgetRelay {
    fn start(engine: Arc<Engine>, endpoint: Arc<Endpoint>) -> Result<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = stop.clone();
        let wake = engine.checkpoint_wake();
        let worker = std::thread::Builder::new()
            .name("lumen-capacity-relay".into())
            .spawn(move || run_budget_relay(engine, wake, endpoint, worker_stop))?;
        Ok(Self {
            stop,
            worker: Some(worker),
        })
    }

    fn stop(&self) {
        self.stop.store(true, Ordering::Release);
    }
}

impl Drop for BudgetRelay {
    fn drop(&mut self) {
        // Do not join on an async caller's drop path. Fallback stops the
        // endpoint too, so a relay waiting for owner work wakes and exits.
        self.stop();
        let _ = self.worker.take();
    }
}

fn run_budget_relay(
    engine: Arc<Engine>,
    wake: Arc<crate::change_budget::BudgetWake>,
    endpoint: Arc<Endpoint>,
    stop: Arc<AtomicBool>,
) {
    let mut observed = wake.epoch();
    let mut last_requested_revision = None;
    loop {
        if stop.load(Ordering::Acquire) || endpoint.is_stopped() {
            return;
        }
        // Work belongs to `engine`, while waiter demand is process-global.
        // A live Engine with publishable work can relieve another Engine's
        // waiter, but an empty fallback never writes for that waiter.
        let Some(state) = engine.capacity_owner_state() else {
            return;
        };
        let demand_revision = state
            .checkpoint_request_revision
            .or_else(|| engine.has_capacity_waiters().then_some(state.work_revision));
        if demand_revision.is_some()
            && (state.active != 0
                || state.frozen != 0
                // Explicit requests may be backed only by resident bytes.
                || state.checkpoint_request_revision.is_some())
            && last_requested_revision != demand_revision
        {
            last_requested_revision = demand_revision;
            // `wait_for` blocks this independent native thread until the one
            // existing owner completes, retries, or is superseded. No Engine,
            // accounting, or apply lock is held here.
            if relay_cycle(&engine, &endpoint, state.checkpoint_request_revision).is_err() {
                return;
            }
            continue;
        }
        // Sample after checking state. A change between the check and this
        // sample is handled immediately; a later change is observed by the
        // BudgetWake predicate before the Condvar can sleep.
        let current = wake.epoch();
        if current != observed {
            observed = current;
            continue;
        }
        let _ = wake.wait_for_change_timeout(observed, Duration::from_millis(50));
        observed = wake.epoch();
    }
}

fn relay_cycle(engine: &Engine, endpoint: &Endpoint, request_revision: Option<u64>) -> Result<()> {
    endpoint.wait_for(Work::Checkpoint)?;
    // A local capacity refusal needs both publication and compaction. The
    // checkpoint frees only the frozen layer; without the merge, repeated
    // refusals can fill the segment budget again before the next request.
    endpoint.wait_for(Work::Merge)?;
    if let Some(request_revision) = request_revision {
        engine.consume_checkpoint_request(request_revision);
    }
    Ok(())
}

/// Caller-owned lazy fallback. Constructor and file IO run before apply; its
/// EngineWatermarkSink does not retain a coordinator or Raft state machine.
pub(crate) struct Fallback {
    owner: Option<Owner>,
    relay: Option<BudgetRelay>,
    _store: Option<Arc<SegmentRdbStore>>,
}

impl Fallback {
    pub(crate) fn ensure(
        slot: &mut Option<Self>,
        engine: &Arc<Engine>,
        configured: Option<Arc<SegmentRdbStore>>,
    ) -> Result<()> {
        if let Some(endpoint) = engine.layer_maintenance.owner() {
            // A configured bootstrap owner may already exist before replay
            // creates its caller-owned fallback.  Keep the owner as the sole
            // checkpoint writer, but attach the native relay to that same
            // endpoint so capacity waits can submit work to it.
            let relay = BudgetRelay::start(engine.clone(), endpoint)?;
            *slot = Some(Self {
                owner: None,
                relay: Some(relay),
                _store: configured,
            });
            return Ok(());
        }
        let is_configured = configured.is_some();
        let store = match configured {
            Some(store) => store,
            None => crate::segment_checkpoint::temporary_spill_store()?,
        };
        let sink = Arc::new(SegmentCheckpointSink {
            engine: engine.clone(),
            store: store.clone(),
            writer: Arc::new(crate::segment_checkpoint::EngineWatermarkSink::new(
                engine.clone(),
            )),
            aof: None,
        });
        let owner = Owner::start(sink, is_configured)?;
        let relay = owner
            .as_ref()
            .map(|owner| BudgetRelay::start(engine.clone(), owner.endpoint()))
            .transpose()?;
        *slot = Some(Self {
            owner,
            relay,
            _store: Some(store),
        });
        Ok(())
    }
}

impl Drop for Fallback {
    fn drop(&mut self) {
        if let Some(relay) = &self.relay {
            relay.stop();
        }
        if let Some(owner) = &mut self.owner {
            owner.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change_budget::ChangeBudget;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc;
    use storage_durable::{CommitStep, FailureInjector, FailurePoint};

    fn engine() -> Arc<Engine> {
        let engine = Arc::new(Engine::new());
        engine
            .create_collection(
                "docs",
                serde_json::from_value(serde_json::json!({
                    "fields": {"kw":{"type":"keyword"}}
                }))
                .unwrap(),
            )
            .unwrap();
        engine
            .index(
                "docs",
                serde_json::from_value(serde_json::json!({
                    "items":[{"external_id":"one","field":"kw","value":"retained"}]
                }))
                .unwrap(),
            )
            .unwrap();
        engine
    }

    fn sink(engine: Arc<Engine>, store: Arc<SegmentRdbStore>) -> Arc<SegmentCheckpointSink> {
        Arc::new(SegmentCheckpointSink {
            writer: Arc::new(crate::segment_checkpoint::EngineWatermarkSink::new(
                engine.clone(),
            )),
            engine,
            store,
            aof: None,
        })
    }

    struct FailSync(AtomicBool);
    impl FailureInjector for FailSync {
        fn check(&self, point: &FailurePoint) -> std::io::Result<()> {
            if point.step == CommitStep::SyncFile && self.0.swap(false, Ordering::AcqRel) {
                return Err(std::io::Error::other("capacity frozen retry test"));
            }
            Ok(())
        }
    }

    #[test]
    fn failed_checkpoint_keeps_its_layer_slot_until_successful_retry() {
        let engine = engine();
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new_with_failure_injector(
            dir.path(),
            Arc::new(FailSync(AtomicBool::new(true))),
        )
        .unwrap();
        assert_eq!(engine.layer_maintenance.append_limit(), 16);
        assert!(
            store
                .save(&engine, 1)
                .unwrap_err()
                .to_string()
                .contains("activate segment generation")
        );
        assert_eq!(
            engine.layer_maintenance.append_limit(),
            15,
            "failed frozen ownership must still reserve the publication slot"
        );
        store.save(&engine, 1).unwrap();
        assert_eq!(
            engine.layer_maintenance.append_limit(),
            16,
            "successful publication and binding release the slot"
        );
        let (cold, sequence) = store.load_latest().unwrap().unwrap();
        assert_eq!(sequence, 1);
        assert_eq!(cold.stats("docs").unwrap().documents_indexed, 1);
    }

    #[test]
    fn fallback_attaches_relay_to_existing_configured_owner() {
        let engine = engine();
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SegmentRdbStore::new(dir.path()).unwrap());
        let owner = Owner::start(sink(engine.clone(), store), true)
            .unwrap()
            .expect("configured owner");
        let mut fallback = None;
        Fallback::ensure(&mut fallback, &engine, None).unwrap();
        let fallback = fallback.expect("existing owner must receive a relay");
        assert!(fallback.owner.is_none());
        assert!(fallback.relay.is_some());
        drop(fallback);
        let mut owner = owner;
        owner.join().unwrap();
    }

    struct HoldSync {
        entered: Mutex<Option<mpsc::Sender<()>>>,
        release: Mutex<mpsc::Receiver<()>>,
    }
    impl FailureInjector for HoldSync {
        fn check(&self, point: &FailurePoint) -> std::io::Result<()> {
            if point.step == CommitStep::SyncFile {
                if let Some(entered) = self.entered.lock().unwrap().take() {
                    entered.send(()).unwrap();
                    self.release.lock().unwrap().recv().unwrap();
                }
            }
            Ok(())
        }
    }
    struct Release(Option<mpsc::Sender<()>>);
    impl Drop for Release {
        fn drop(&mut self) {
            if let Some(tx) = self.0.take() {
                let _ = tx.send(());
            }
        }
    }

    #[test]
    fn configured_owner_fences_detached_temporary_save_before_current() {
        let engine = engine();
        let old_dir = tempfile::tempdir().unwrap();
        let new_dir = tempfile::tempdir().unwrap();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let release = Release(Some(release_tx));
        let old_store = Arc::new(
            SegmentRdbStore::new_with_failure_injector(
                old_dir.path(),
                Arc::new(HoldSync {
                    entered: Mutex::new(Some(entered_tx)),
                    release: Mutex::new(release_rx),
                }),
            )
            .unwrap(),
        );
        let mut old = Owner::start(sink(engine.clone(), old_store.clone()), false)
            .unwrap()
            .unwrap();
        let fenced = old_store
            .as_ref()
            .clone()
            .with_publication_fence(old.fence());
        let saving_engine = engine.clone();
        let saving = std::thread::spawn(move || fenced.save(&saving_engine, 1));
        entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        let new_store = Arc::new(SegmentRdbStore::new(new_dir.path()).unwrap());
        let mut new = Owner::start(sink(engine.clone(), new_store.clone()), true)
            .unwrap()
            .unwrap();
        assert!(old.fence().acquire().is_err());
        assert!(
            Owner::start(sink(engine.clone(), old_store.clone()), false)
                .unwrap()
                .is_none(),
            "temporary owner cannot displace configured owner"
        );
        drop(release);
        let error = saving.join().unwrap().unwrap_err();
        assert!(
            format!("{error:#}").contains("superseded before CURRENT"),
            "{error:#}"
        );
        assert!(
            old_store.load_latest().unwrap().is_none(),
            "superseded output must not change CURRENT"
        );
        assert!(
            engine.capture_barrier.capture(0).is_ok(),
            "pre-publication supersession is not an uncertain commit"
        );
        new_store
            .as_ref()
            .clone()
            .with_publication_fence(new.fence())
            .save(&engine, 1)
            .unwrap();
        assert_eq!(new_store.load_latest().unwrap().unwrap().1, 1);
        old.join().unwrap();
        new.join().unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn manual_checkpoint_keeps_capacity_relief_on_its_publication_owner() {
        for configured in [false, true] {
            let engine = engine();
            let dir = tempfile::tempdir().unwrap();
            let (entered_tx, entered_rx) = mpsc::channel();
            let (release_tx, release_rx) = mpsc::channel();
            let release = Release(Some(release_tx));
            let store = Arc::new(
                SegmentRdbStore::new_with_failure_injector(
                    dir.path(),
                    Arc::new(HoldSync {
                        entered: Mutex::new(Some(entered_tx)),
                        release: Mutex::new(release_rx),
                    }),
                )
                .unwrap(),
            );
            let sink = sink(engine.clone(), store.clone());
            let mut configured_owner =
                configured.then(|| Owner::start(sink.clone(), true).unwrap().unwrap());
            let prior = engine.layer_maintenance.owner();
            let saving = tokio::spawn(async move {
                crate::api::CheckpointSink::checkpoint_now(sink.as_ref()).await
            });
            let entered = tokio::task::spawn_blocking(move || {
                entered_rx.recv_timeout(Duration::from_secs(10))
            })
            .await
            .unwrap();
            let during = engine.layer_maintenance.owner();
            let mut fallback = None;
            if entered.is_ok() && during.is_some() {
                Fallback::ensure(&mut fallback, &engine, None).unwrap();
            }
            let reused = fallback
                .as_ref()
                .is_some_and(|fallback| fallback.owner.is_none() && fallback.relay.is_some());
            drop(release);
            let result = saving.await.unwrap();
            drop(fallback);
            if let Some(owner) = &mut configured_owner {
                owner.join().unwrap();
            }
            entered.expect("manual checkpoint reached file sync");
            assert!(result.unwrap());
            assert!(
                during.is_some(),
                "capacity refusal during manual publication must reuse its owner"
            );
            assert!(
                reused,
                "capacity relief must not open a second checkpoint root"
            );
            if let Some(prior) = prior {
                assert!(
                    Arc::ptr_eq(&prior, during.as_ref().unwrap()),
                    "manual publication must preserve the configured driver's owner"
                );
            }
            assert_eq!(
                store
                    .load_latest()
                    .unwrap()
                    .unwrap()
                    .0
                    .stats("docs")
                    .unwrap()
                    .documents_indexed,
                1
            );
        }
    }

    #[test]
    fn relay_cycle_checkpoints_merges_then_consumes_request() {
        let engine = engine();
        let root = tempfile::tempdir().unwrap();
        let store = Arc::new(SegmentRdbStore::new(root.path()).unwrap());
        let mut owner = Owner::start(sink(engine.clone(), store), true).unwrap().unwrap();
        engine.request_pending_checkpoint();
        let revision = engine
            .capacity_owner_state()
            .and_then(|state| state.checkpoint_request_revision)
            .unwrap();
        relay_cycle(&engine, &owner.endpoint(), Some(revision)).unwrap();
        assert!(engine
            .capacity_owner_state()
            .and_then(|state| state.checkpoint_request_revision)
            .is_none());
        let requests = owner.endpoint.requests.lock().unwrap();
        assert_eq!(requests.requested, 2);
        assert_eq!(requests.completed, 2);
        assert_eq!(requests.operations, [Work::Checkpoint, Work::Merge]);
        drop(requests);
        owner.join().unwrap();
    }

    #[test]
    fn stale_relay_revision_cannot_consume_successor_request() {
        let engine = engine();
        let root = tempfile::tempdir().unwrap();
        let store = Arc::new(SegmentRdbStore::new(root.path()).unwrap());
        let mut owner = Owner::start(sink(engine.clone(), store), true).unwrap().unwrap();
        engine.request_pending_checkpoint();
        let first = engine
            .capacity_owner_state()
            .and_then(|state| state.checkpoint_request_revision)
            .unwrap();
        engine.consume_checkpoint_request(first);
        engine.request_pending_checkpoint();
        let successor = engine
            .capacity_owner_state()
            .and_then(|state| state.checkpoint_request_revision)
            .unwrap();
        assert!(successor > first);
        relay_cycle(&engine, &owner.endpoint(), Some(first)).unwrap();
        assert_eq!(
            engine
                .capacity_owner_state()
                .and_then(|state| state.checkpoint_request_revision),
            Some(successor)
        );
        owner.join().unwrap();
    }

    #[test]
    fn relay_cycle_error_keeps_request_pending() {
        let engine = engine();
        engine.request_pending_checkpoint();
        let revision = engine
            .capacity_owner_state()
            .and_then(|state| state.checkpoint_request_revision)
            .unwrap();
        let endpoint = Endpoint {
            requests: Mutex::new(Requests {
                completed: 1,
                error: Some("terminal checkpoint failure".into()),
                ..Requests::default()
            }),
            changed: Condvar::new(),
            fence: PublicationFence {
                registry: Weak::new(),
                token: 0,
            },
        };
        assert!(relay_cycle(&engine, &endpoint, Some(revision)).is_err());
        assert_eq!(
            engine
                .capacity_owner_state()
                .and_then(|state| state.checkpoint_request_revision),
            Some(revision)
        );
    }

    #[test]
    fn replacement_owner_waits_until_current_and_live_binding_permit_releases() {
        let engine = engine();
        let old_dir = tempfile::tempdir().unwrap();
        let new_dir = tempfile::tempdir().unwrap();
        let old_store = Arc::new(SegmentRdbStore::new(old_dir.path()).unwrap());
        let mut old = Owner::start(sink(engine.clone(), old_store), false)
            .unwrap()
            .unwrap();
        let publication = old.fence().acquire().unwrap();
        let new_sink = sink(
            engine.clone(),
            Arc::new(SegmentRdbStore::new(new_dir.path()).unwrap()),
        );
        let (attempt_tx, attempt_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let replacement = std::thread::spawn(move || {
            attempt_tx.send(()).unwrap();
            let mut new = Owner::start(new_sink, true).unwrap().unwrap();
            done_tx.send(()).unwrap();
            new.join().unwrap();
        });
        attempt_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let early = done_rx.recv_timeout(Duration::from_millis(50)).is_ok();
        drop(publication);
        if !early {
            done_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        }
        replacement.join().unwrap();
        old.join().unwrap();
        assert!(
            !early,
            "configured replacement crossed an active durable/live publication permit"
        );
    }
    #[test]
    fn capacity_worker_retries_prepublication_failure_without_failing_waiter() {
        let engine = engine();
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(
            SegmentRdbStore::new_with_failure_injector(
                dir.path(),
                Arc::new(FailSync(AtomicBool::new(true))),
            )
            .unwrap(),
        );
        let mut owner = Owner::start(sink(engine.clone(), store.clone()), true)
            .unwrap()
            .unwrap();
        let result = owner.endpoint.wait_for(Work::Checkpoint);
        owner.join().unwrap();
        result.expect("a pre-publication capacity failure must retry while the committed waiter retains ownership");
        assert_eq!(store.load_latest().unwrap().unwrap().1, 0);
        assert!(!engine.capture_barrier.is_uncertain());
    }

    #[test]
    fn fallback_replaces_a_stopped_owner_slot() {
        let engine = engine();
        let mut fallback = None;
        Fallback::ensure(&mut fallback, &engine, None).unwrap();
        let old = engine.layer_maintenance.owner().unwrap();
        old.stop();
        assert!(engine.layer_maintenance.owner().is_none());

        Fallback::ensure(&mut fallback, &engine, None).unwrap();
        let replacement = engine.layer_maintenance.owner().unwrap();
        assert!(
            !Arc::ptr_eq(&old, &replacement) && !replacement.is_stopped(),
            "a stopped fallback slot must not suppress a replacement owner"
        );
    }

    #[test]
    fn local_capacity_requests_schedule_two_checkpoint_merge_cycles() {
        let engine = engine();
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SegmentRdbStore::new(dir.path()).unwrap());
        let mut fallback = None;
        Fallback::ensure(&mut fallback, &engine, Some(store)).unwrap();
        let endpoint = engine.layer_maintenance.owner().unwrap();
        for id in ["one", "two"] {
            while engine
                .capacity_owner_state()
                .and_then(|state| state.checkpoint_request_revision)
                .is_some()
            {
                std::thread::sleep(Duration::from_millis(10));
            }
            engine
                .index(
                    "docs",
                    serde_json::from_value(serde_json::json!({
                        "items":[{"external_id":id,"field":"kw","value":"next"}]
                    }))
                    .unwrap(),
                )
                .unwrap();
            let state = engine.capacity_owner_state().unwrap();
            assert!(state.active != 0 || state.frozen != 0);
            let start = endpoint.requests.lock().unwrap().requested;
            engine.request_pending_checkpoint();
            let expected = start + 2;
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            while std::time::Instant::now() < deadline {
                let requests = endpoint.requests.lock().unwrap();
                let request_cleared = engine
                    .capacity_owner_state()
                    .and_then(|state| state.checkpoint_request_revision)
                    .is_none();
                if requests.requested >= expected
                    && requests.completed >= expected
                    && request_cleared
                {
                    break;
                }
                drop(requests);
                std::thread::sleep(Duration::from_millis(10));
            }
            let requests = endpoint.requests.lock().unwrap();
            assert!(requests.requested >= expected && requests.completed >= expected);
            drop(requests);
            assert!(engine
                .capacity_owner_state()
                .and_then(|state| state.checkpoint_request_revision)
                .is_none());
        }
        drop(fallback);
    }

    #[test]
    fn resident_only_capacity_request_reaches_checkpoint_and_merge_relay() {
        let engine = engine();
        let raw = engine.raw_capacity_state_for_test().unwrap();
        assert_eq!(raw.active, 0);
        assert_eq!(raw.frozen_batches, 0);
        assert!(raw.resident_active > 0);
        assert_eq!(raw.resident_frozen, 0);
        let dir = tempfile::tempdir().unwrap();
        engine.request_pending_checkpoint();
        assert!(engine
            .capacity_owner_state()
            .unwrap()
            .checkpoint_request_revision
            .is_some());
        let mut fallback = None;
        Fallback::ensure(
            &mut fallback,
            &engine,
            Some(Arc::new(SegmentRdbStore::new(dir.path()).unwrap())),
        )
        .unwrap();
        let endpoint = engine.layer_maintenance.owner().unwrap();

        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            let request = engine
                .capacity_owner_state()
                .and_then(|state| state.checkpoint_request_revision);
            let requests = endpoint.requests.lock().unwrap();
            if request.is_none() && requests.requested >= 2 && requests.completed >= 2 {
                break;
            }
            drop(requests);
            std::thread::sleep(Duration::from_millis(10));
        }
        let requests = endpoint.requests.lock().unwrap();
        assert!(requests.requested >= 2 && requests.completed >= 2);
        drop(requests);
        assert!(engine
            .capacity_owner_state()
            .and_then(|state| state.checkpoint_request_revision)
            .is_none());
        drop(fallback);
    }

    #[test]
    fn relay_publishes_own_work_for_other_engine_waiter_without_empty_save() {
        const HARD: usize = 64 * 1024;
        let budget = ChangeBudget::with_hard_limit(HARD);
        let engine_a = Arc::new(Engine::with_change_budget(budget.clone()));
        engine_a
            .create_collection(
                "docs",
                serde_json::from_value(serde_json::json!({
                    "fields": {"kw":{"type":"keyword"}}
                }))
                .unwrap(),
            )
            .unwrap();
        engine_a
            .index(
                "docs",
                serde_json::from_value(serde_json::json!({
                    "items":[{"external_id":"one","field":"kw","value":"retained"}]
                }))
                .unwrap(),
            )
            .unwrap();
        let engine_b = Arc::new(Engine::with_change_budget(budget.clone()));
        let mut fallback_a = None;
        let mut fallback_b = None;
        Fallback::ensure(&mut fallback_a, &engine_a, None).unwrap();
        Fallback::ensure(&mut fallback_b, &engine_b, None).unwrap();
        let used = budget.snapshot().total;
        assert!(used > 0 && used < HARD, "fixture needs publishable A work");
        let held = budget.owner().try_reserve(HARD - used).unwrap();
        // The budget is full before the request starts. A fast publication may
        // finish before this thread can observe the transient waiter counter.
        assert_eq!(budget.snapshot().total, HARD);
        let request = engine_b.record_ram_request_from_bound(1, 0);
        let (done_tx, done_rx) = mpsc::channel();
        let waiting_engine = engine_b.clone();
        std::thread::spawn(move || {
            done_tx
                .send(waiting_engine.wait_reserve_record_ram(&request).map(drop))
                .unwrap();
        });
        done_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("Engine A fallback must publish work for Engine B waiter")
            .expect("Engine B admission must succeed after Engine A publication");
        assert!(
            engine_a.metrics().segment_checkpoint_completed_total.get() > 0,
            "Engine A owns the only publishable work"
        );
        assert_eq!(
            engine_b.metrics().segment_checkpoint_completed_total.get(),
            0,
            "empty Engine B must not checkpoint for its own capacity wait"
        );
        drop(held);
    }

    #[test]
    fn merge_request_bootstraps_a_fresh_fallback_root_from_live_state() {
        let engine = engine();
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SegmentRdbStore::new(dir.path()).unwrap());
        let mut owner = Owner::start(sink(engine, store.clone()), false)
            .unwrap()
            .unwrap();
        owner.endpoint.wait_for(Work::Merge).unwrap();
        owner.join().unwrap();
        assert!(
            store.load_latest().unwrap().is_some(),
            "merge capacity on an empty fallback root must first publish the live engine"
        );
    }
}
