//! One process worker compacts immutable fields outside the checkpoint lock.
//! A root token owns its source/staging pins until publication or refusal.
//! Publication rebases exact inputs onto the newest complete CURRENT catalog.

use super::compaction::confined;
use super::*;
use std::sync::{mpsc, Condvar};
use std::time::{Duration, Instant};

#[derive(Default)]
struct WorkState {
    queued: bool,
    running: bool,
    requested: bool,
    ordinary_retry_permits: u8,
    retryable_stale: bool,
    published_revision: u64,
    publication_revision_overflowed: bool,
    #[cfg(test)]
    wait_entries: u64,
    next_owner: Option<(
        Weak<Engine>,
        Option<crate::segment_capacity::PublicationFence>,
    )>,
    error: Option<String>,
    protected: BTreeMap<String, usize>,
    // A proved predecessor deferred only because a reader still owned it.
    // Later pruning may have shortened the visible chain in the meantime.
    retired: BTreeSet<String>,
    readers: Vec<(String, Vec<Weak<crate::segment::SegmentReader>>)>,
}

impl WorkState {
    // Called under the root mutex, with the save permit still held. An idle
    // observation expires when a later publication changes this revision.
    fn validate_capacity_retry(
        &self,
        idle_revision: Option<u64>,
        deadline: Option<Instant>,
    ) -> Result<()> {
        if self.publication_revision_overflowed {
            bail!("background segment merge publication revision overflowed");
        }
        if (idle_revision.is_some() || deadline.is_some()) && !self.queued && !self.running {
            if let Some(error) = &self.error {
                bail!("background segment merge failed: {error}");
            }
        }
        // A ready predicate may wake after its deadline. Preflight is allowed
        // to use the freed capacity, but a still-full field cannot request more
        // work after that same deadline.
        if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            bail!("background segment merge capacity wait timed out");
        }
        if idle_revision
            .is_some_and(|revision| self.published_revision <= revision && !self.retryable_stale)
        {
            bail!("background segment merge made no capacity progress");
        }
        Ok(())
    }
}

#[derive(Default)]
pub(super) struct RootWork {
    state: Mutex<WorkState>,
    changed: Condvar,
}

/// One point-in-time view of the root-local merge scheduler. It is copied
/// into the checkpoint diagnostic trace after a durable publication.
#[derive(Clone, Copy)]
pub(super) struct TraceState {
    pub(super) queued: bool,
    pub(super) running: bool,
    pub(super) requested: bool,
    pub(super) published_revision: u64,
}

pub(super) fn shared(root: &Path) -> Result<Arc<RootWork>> {
    // A loaded Engine may outlive all store handles. Keep its weak reader pins
    // discoverable by a later handle for the same canonical root.
    static ROOTS: OnceLock<Mutex<HashMap<PathBuf, Arc<RootWork>>>> = OnceLock::new();
    let root = std::fs::canonicalize(root)?;
    let mut roots = ROOTS
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    roots.retain(|_, state| Arc::strong_count(state) > 1 || state.has_live_ownership());
    if let Some(state) = roots.get(&root) {
        return Ok(state.clone());
    }
    let state = Arc::new(RootWork::default());
    roots.insert(root, state.clone());
    Ok(state)
}

impl RootWork {
    pub(super) fn trace_state(&self) -> TraceState {
        let state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        TraceState {
            queued: state.queued,
            running: state.running,
            requested: state.requested,
            published_revision: state.published_revision,
        }
    }

    fn has_live_ownership(&self) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        state
            .readers
            .retain(|(_, readers)| readers.iter().any(|reader| reader.strong_count() != 0));
        state.queued
            || state.running
            || !state.protected.is_empty()
            || !state.retired.is_empty()
            || !state.readers.is_empty()
    }
    pub(super) fn defer_reclaim(&self, name: &str) {
        self.state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .retired
            .insert(name.to_owned());
    }
    pub(super) fn known_retired(&self, name: &str) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .retired
            .contains(name)
    }
    pub(super) fn reclaimed(&self, name: &str) {
        self.state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .retired
            .remove(name);
    }
    pub(super) fn pin_loaded_engine(&self, generation: String, engine: &Engine) -> Result<()> {
        let _capture = engine
            .capture_barrier
            .capture(0)
            .map_err(anyhow::Error::msg)?;
        let expected = engine
            .checkpoint_dirty_fields()?
            .into_iter()
            .map(|(name, (generation, schema_version, _))| {
                (
                    name,
                    crate::storage::CheckpointCollectionIdentity {
                        generation,
                        schema_version,
                        data_version: 0,
                    },
                )
            })
            .collect();
        let capture = engine.capture_background_merge(expected)?;
        let readers = capture
            .live_base_inputs
            .values()
            .flat_map(|fields| fields.values())
            .chain(
                capture
                    .live_delta_inputs
                    .values()
                    .flat_map(|fields| fields.values())
                    .flatten(),
            )
            .map(Arc::downgrade)
            .collect::<Vec<_>>();
        if !readers.is_empty() {
            self.state
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .readers
                .push((generation, readers));
        }
        Ok(())
    }
    pub(super) fn protects(&self, name: &str) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        state
            .readers
            .retain(|(_, readers)| readers.iter().any(|reader| reader.strong_count() != 0));
        state.protected.contains_key(name)
            || state
                .readers
                .iter()
                .any(|(generation, _)| generation == name)
    }

    pub(super) fn pin(&self, name: String) {
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        *state.protected.entry(name).or_default() += 1;
    }

    pub(super) fn unpin(&self, name: &str) {
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        let Some(refs) = state.protected.get_mut(name) else {
            return;
        };
        *refs -= 1;
        if *refs == 0 {
            state.protected.remove(name);
        }
    }

    fn pin_readers(&self, generation: String, capture: &crate::storage::CheckpointCapture) {
        let readers = capture
            .prepared_compactions
            .values()
            .flatten()
            .flat_map(|field| {
                field
                    .base
                    .iter()
                    .chain(field.inputs.iter())
                    .map(Arc::downgrade)
            })
            .collect::<Vec<_>>();
        if !readers.is_empty() {
            self.state
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .readers
                .push((generation, readers));
        }
    }

    fn wait(&self, timeout: Duration) -> Result<()> {
        let deadline = Instant::now() + timeout;
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        #[cfg(test)]
        {
            state.wait_entries = state
                .wait_entries
                .checked_add(1)
                .expect("test merge-wait entry counter overflowed");
            self.changed.notify_all();
        }
        while state.queued || state.running {
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                bail!("background segment merge wait timed out");
            }
            state = self
                .changed
                .wait_timeout(state, left)
                .unwrap_or_else(|p| p.into_inner())
                .0;
        }
        if let Some(error) = &state.error {
            bail!("background segment merge failed: {error}");
        }
        Ok(())
    }

    pub(super) fn wait_for_capacity_progress_after(
        &self,
        revision: u64,
        deadline: Instant,
    ) -> Result<CapacityWait> {
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        #[cfg(test)]
        {
            state.wait_entries = state
                .wait_entries
                .checked_add(1)
                .expect("test merge-wait entry counter overflowed");
            self.changed.notify_all();
        }
        loop {
            if !state.queued && !state.running {
                if let Some(error) = &state.error {
                    bail!("background segment merge failed: {error}");
                }
            }
            if state.published_revision > revision {
                return Ok(CapacityWait::Published);
            }
            if !state.queued && !state.running {
                return Ok(CapacityWait::Idle);
            }
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                bail!("background segment merge capacity wait timed out");
            }
            state = self
                .changed
                .wait_timeout(state, left)
                .unwrap_or_else(|p| p.into_inner())
                .0;
        }
    }

    fn finish_job(&self, result: &mut Result<MergeOutcome>, engine_alive: bool) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        state.running = false;
        if result
            .as_ref()
            .is_ok_and(|outcome| *outcome == MergeOutcome::Published)
        {
            match state.published_revision.checked_add(1) {
                Some(revision) => state.published_revision = revision,
                None => {
                    state.publication_revision_overflowed = true;
                    *result = Err(anyhow!(
                        "background segment merge publication revision overflowed"
                    ));
                }
            }
        }
        if state.publication_revision_overflowed {
            state.requested = false;
            state.queued = false;
            state.error = Some("background segment merge publication revision overflowed".into());
            self.changed.notify_all();
            return false;
        }
        let retryable = result.as_ref().is_err()
            || result
                .as_ref()
                .is_ok_and(|outcome| *outcome == MergeOutcome::RetryableStale);
        if retryable && state.ordinary_retry_permits > 0 {
            state.ordinary_retry_permits -= 1;
            state.requested = true;
        } else if result.as_ref().is_ok_and(|outcome| {
            matches!(
                outcome,
                MergeOutcome::Published | MergeOutcome::NoEligibleWork
            )
        }) {
            state.ordinary_retry_permits = 0;
        }
        // A successful pair merge must not immediately schedule another pass
        // against the same immutable catalog. The scheduler will request the
        // next pass when a later checkpoint creates another delta layer, or
        // when a request arrived while this job was running. This preserves
        // the selected two-layer window instead of draining a four-layer
        // stack through automatic follow-up jobs.
        // A failed job leaves its source and pins intact.  Keep that state
        // retryable so a later durable checkpoint can submit one fresh job
        // after the worker has become idle.
        state.retryable_stale = retryable;
        let again = state.requested;
        state.error = result.as_ref().err().map(|error| format!("{error:#}"));
        // Keep the token logically queued across re-enqueue so a waiter cannot
        // observe a false idle gap between two ready fields.
        state.queued = again && engine_alive;
        self.changed.notify_all();
        again
    }

    #[cfg(test)]
    fn test_wait_entries(&self) -> u64 {
        self.state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .wait_entries
    }

    #[cfg(test)]
    fn wait_for_test_wait_entry_after(&self, baseline: u64, timeout: Duration) -> Result<()> {
        let deadline = Instant::now() + timeout;
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        while state.wait_entries <= baseline {
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                bail!("target checkpoint did not enter RootWork::wait");
            }
            state = self
                .changed
                .wait_timeout(state, left)
                .unwrap_or_else(|p| p.into_inner())
                .0;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum CapacityWait {
    Published,
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MergeOutcome {
    Published,
    RetryableStale,
    NoEligibleWork,
}

struct Job {
    store: SegmentRdbStore,
    engine: Weak<Engine>,
}

fn queue_sender() -> Result<&'static mpsc::Sender<Job>> {
    static WORKER: OnceLock<std::result::Result<mpsc::Sender<Job>, String>> = OnceLock::new();
    WORKER
        .get_or_init(|| {
            let (sender, receiver) = mpsc::channel::<Job>();
            std::thread::Builder::new()
                .name("lumen-segment-merge".into())
                .spawn(move || {
                    while let Ok(mut job) = receiver.recv() {
                        let work = job.store.background.clone();
                        {
                            let mut state = work.state.lock().unwrap_or_else(|p| p.into_inner());
                            if let Some((engine, fence)) = state.next_owner.take() {
                                job.engine = engine;
                                job.store.publication_fence = fence;
                            }
                            state.queued = false;
                            state.running = true;
                            state.requested = false;
                        }
                        let engine = job.engine.upgrade();
                        let mut result =
                            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                engine
                                    .as_ref()
                                    .map_or(Ok(MergeOutcome::NoEligibleWork), |engine| {
                                        job.store.merge_one(engine)
                                    })
                            }))
                            .unwrap_or_else(|_| {
                                Err(anyhow!("background merge worker panicked; files retained"))
                            });
                        let again = work.finish_job(&mut result, engine.is_some());
                        if let Err(error) = &result {
                            tracing::warn!(%error, "background segment merge failed");
                        }
                        if again {
                            if let Some(engine) = engine {
                                if let Ok(queue) = queue_sender() {
                                    if queue
                                        .send(Job {
                                            store: job.store,
                                            engine: Arc::downgrade(&engine),
                                        })
                                        .is_ok()
                                    {
                                        continue;
                                    }
                                }
                                let mut state =
                                    work.state.lock().unwrap_or_else(|p| p.into_inner());
                                state.queued = false;
                                state.error = Some("background merge queue stopped".into());
                                work.changed.notify_all();
                            }
                        }
                    }
                })
                .map_err(|error| error.to_string())?;
            Ok(sender)
        })
        .as_ref()
        .map_err(|error| anyhow!("start background merge worker: {error}"))
}

fn identities(
    manifest: &SegmentGenerationManifest,
) -> BTreeMap<String, crate::storage::CheckpointCollectionIdentity> {
    manifest
        .collections
        .iter()
        .map(|collection| {
            (
                collection.collection_id.clone(),
                crate::storage::CheckpointCollectionIdentity {
                    generation: collection.collection_generation,
                    data_version: collection.data_version,
                    schema_version: collection.schema_version,
                },
            )
        })
        .collect()
}

pub(super) fn needs_merge(manifest: &SegmentGenerationManifest) -> bool {
    manifest.collections.iter().any(|collection| {
        let mut counts = BTreeMap::new();
        collection.segments.iter().any(|segment| {
            if segment.role != SegmentRole::Field || segment.kind != SegmentKind::Delta {
                return false;
            }
            let count = counts.entry(segment.field.as_deref()).or_insert(0);
            *count += 1;
            *count >= 4
        })
    })
}

impl SegmentRdbStore {
    /// Remove only this merge's owned scratch under the root save gate. On a
    /// removal error, leave both pins in place so no later sweep can treat the
    /// incomplete merge as abandoned work.
    fn cleanup_owned_merge_staging(
        &self,
        scratch_path: &Path,
        scratch_name: &str,
        source_name: &str,
        scratch: StagedGeneration,
    ) -> Result<()> {
        self.cleanup_owned_merge_staging_with(
            scratch_path,
            scratch_name,
            source_name,
            scratch,
            |path| std::fs::remove_dir_all(path),
        )
    }

    fn cleanup_owned_merge_staging_with(
        &self,
        scratch_path: &Path,
        scratch_name: &str,
        source_name: &str,
        scratch: StagedGeneration,
        remove: impl FnOnce(&Path) -> std::io::Result<()>,
    ) -> Result<()> {
        let _cleanup_guard = self.save_gate.lock_owned();
        match std::fs::symlink_metadata(scratch_path) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    bail!(
                        "owned background merge staging must be a real directory: {}",
                        scratch_path.display()
                    );
                }
                if let Err(error) = remove(scratch_path) {
                    return Err(error).context("remove owned background merge staging");
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "inspect owned background merge staging {}",
                        scratch_path.display()
                    )
                });
            }
        }
        self.background.unpin(scratch_name);
        self.background.unpin(source_name);
        drop(scratch);
        Ok(())
    }

    pub(super) fn request_merge(&self, engine: &Arc<Engine>) -> Result<()> {
        self.request_merge_with_revision(engine).map(|_| ())
    }

    pub(super) fn request_merge_with_revision(&self, engine: &Arc<Engine>) -> Result<u64> {
        let queue = queue_sender()?;
        let mut state = self
            .background
            .state
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let revision = state.published_revision;
        if state.publication_revision_overflowed {
            bail!("background segment merge publication revision overflowed");
        }
        if state.running || state.queued {
            state.ordinary_retry_permits = state.ordinary_retry_permits.saturating_add(1);
            return Ok(revision);
        }
        state.retryable_stale = false;
        state.error = None;
        state.next_owner = Some((Arc::downgrade(engine), self.publication_fence.clone()));
        state.queued = true;
        if queue
            .send(Job {
                store: self.clone(),
                engine: Arc::downgrade(engine),
            })
            .is_err()
        {
            state.queued = false;
            state.error = Some("background merge queue stopped".into());
            self.background.changed.notify_all();
            bail!("background merge queue stopped");
        }
        Ok(revision)
    }

    pub(super) fn request_merge_for_capacity_retry(
        &self,
        engine: &Arc<Engine>,
        idle_revision: Option<u64>,
        deadline: Option<Instant>,
    ) -> Result<u64> {
        let queue = queue_sender()?;
        let mut state = self
            .background
            .state
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        state.validate_capacity_retry(idle_revision, deadline)?;
        state.retryable_stale = false;
        let revision = state.published_revision;
        state.next_owner = Some((Arc::downgrade(engine), self.publication_fence.clone()));
        if state.running {
            state.requested = true;
            return Ok(revision);
        }
        if state.queued {
            return Ok(revision);
        }
        state.error = None;
        state.queued = true;
        if queue
            .send(Job {
                store: self.clone(),
                engine: Arc::downgrade(engine),
            })
            .is_err()
        {
            state.queued = false;
            state.error = Some("background merge queue stopped".into());
            self.background.changed.notify_all();
            bail!("background merge queue stopped");
        }
        Ok(revision)
    }

    /// Wait for all currently requested merges on this root. Checkpoint save
    /// completion alone does not imply background compaction completion.
    #[doc(hidden)]
    pub fn wait_for_merges(&self, timeout: Duration) -> Result<()> {
        self.background.wait(timeout)
    }

    pub(super) fn needs_delta_capacity(
        &self,
        engine: &Arc<Engine>,
        manifest: &SegmentGenerationManifest,
        current_root: &Path,
    ) -> Result<bool> {
        let dirty = engine.checkpoint_reusable_dirty_fields(current_root)?;
        for collection in &manifest.collections {
            let Some((generation, schema, fields)) = dirty.get(&collection.collection_id) else {
                continue;
            };
            if *generation != collection.collection_generation
                || *schema != collection.schema_version
            {
                continue;
            }
            for field in fields {
                if collection
                    .segments
                    .iter()
                    .filter(|segment| {
                        segment.role == SegmentRole::Field
                            && segment.kind == SegmentKind::Delta
                            && segment.field.as_ref() == Some(field)
                    })
                    .count()
                    >= 16
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn merge_one(&self, engine: &Arc<Engine>) -> Result<MergeOutcome> {
        let job_started = Instant::now();
        let mut costs = MergeStepCosts::default();
        let mut save_gate_held = Duration::ZERO;
        let mut compacted_fields = 0u64;
        if self
            .publication_fence
            .as_ref()
            .is_some_and(|fence| fence.acquire().is_err())
        {
            return Ok(MergeOutcome::RetryableStale);
        }
        let guard = self.save_gate.lock_owned();
        // A failed checkpoint owns an older cut. It must publish before a
        // background rewrite can change that cut's predecessor.
        if self
            .pending_frozen
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .is_some()
        {
            return Ok(MergeOutcome::RetryableStale);
        }
        let Some(source) = self.current_record()? else {
            return Ok(MergeOutcome::NoEligibleWork);
        };
        if source.legacy {
            return Ok(MergeOutcome::NoEligibleWork);
        }
        let prior = read_generation_manifest(&source.path)?;
        // v3 is a legacy flat layout that the normal reader can reopen, but
        // background merge must never copy it into a newly published
        // generation. The checkpoint writer is the only migration path to v2.
        if !background_merge_supports_manifest(prior.schema_version) {
            return Ok(MergeOutcome::NoEligibleWork);
        }
        if !needs_merge(&prior) {
            return Ok(MergeOutcome::NoEligibleWork);
        }
        self.verify_predecessor_catalog(&source, &prior)?;
        let stamp = engine
            .capture_barrier
            .capture(source.sequence)
            .map_err(anyhow::Error::msg)?;
        let captured_stamp = stamp.stamp();
        let capture_before_started = Instant::now();
        let mut capture = engine.capture_background_merge(identities(&prior))?;
        costs.record(
            crate::metrics::MergeStep::CaptureBefore,
            capture_before_started.elapsed(),
            0,
        );
        drop(stamp);
        let scratch = self.begin_background_merge_stage(source.sequence)?;
        let scratch_path = scratch.path().to_path_buf();
        let scratch_name = scratch_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        self.background.pin(source.name.as_str().to_owned());
        self.background.pin(scratch_name.clone());
        drop(guard);

        let result = (|| -> Result<MergeOutcome> {
            let mut collections = prior.collections.clone();
            // Selection reads only sizes, so it runs against the read-only
            // source generation. Knowing the collection first is what keeps
            // the scratch link below proportional to the job's payload: every
            // candidate returned shares that one collection, so one link
            // pass covers every field this job will compact.
            let candidates = select_staged_delta_window(&source.path, &collections)?;
            if candidates.is_empty() {
                return Ok(MergeOutcome::NoEligibleWork);
            }
            let collection_id = candidates[0].collection_id.clone();
            let selected_fields: Vec<String> = candidates
                .iter()
                .map(|candidate| candidate.field.clone())
                .collect();
            let selected_inputs: BTreeMap<String, (Vec<SegmentReference>, SegmentReference, bool)> =
                candidates
                    .iter()
                    .map(|candidate| {
                        (
                            candidate.field.clone(),
                            (
                                candidate.inputs.clone(),
                                candidate.base.clone(),
                                candidate.includes_base,
                            ),
                        )
                    })
                    .collect();
            let link_scratch_started = Instant::now();
            let scratch_links =
                link_collection(&source.path, &scratch_path, collection_id.as_str())?;
            costs.record(
                crate::metrics::MergeStep::LinkScratch,
                link_scratch_started.elapsed(),
                scratch_links as u64,
            );
            let compact_started = Instant::now();
            // The selector chooses the collection and the tied field set.
            // Each admitted field is then drained once from its complete
            // captured stack. This keeps one publication while avoiding
            // repeated scratch rounds and repeated encode setup.
            let pending_candidates = candidates;
            let mut scratch_delta_readers = capture
                .live_delta_inputs
                .get(collection_id.as_str())
                .cloned()
                .unwrap_or_default();
            let mut final_outputs = BTreeMap::new();
            let compacted_input_count = pending_candidates
                .iter()
                .map(|candidate| candidate.inputs.len() as u64)
                .sum::<u64>();
            let outputs = compact_staged_delta_windows(
                &scratch_path,
                source.sequence,
                &mut collections,
                &mut capture,
                &mut scratch_delta_readers,
                self.merge_observer.as_ref(),
                pending_candidates,
            )?;
            for output in outputs {
                final_outputs.insert(output.output.field.clone().unwrap_or_default(), output);
            }
            costs.record(
                crate::metrics::MergeStep::Compact,
                compact_started.elapsed(),
                compacted_input_count,
            );
            compacted_fields = final_outputs.len() as u64;
            if final_outputs.is_empty() {
                return Ok(MergeOutcome::NoEligibleWork);
            }
            // One job publishes the selected field window. The fold is kept
            // as a sequence so the output identity check stays per-output.
            let mut merges = Vec::with_capacity(final_outputs.len());
            for field in &selected_fields {
                let mut output = final_outputs
                    .remove(field)
                    .ok_or_else(|| anyhow!("bounded merge produced no final output"))?;
                let (selected_inputs, selected_base, includes_base) = selected_inputs
                    .get(field)
                    .ok_or_else(|| anyhow!("bounded merge lost original inputs"))?;
                let old_collection = prior
                    .collections
                    .iter()
                    .find(|collection| collection.segments.contains(&selected_inputs[0]))
                    .ok_or_else(|| anyhow!("merge inputs have no source collection"))?;
                // The scratch output was produced from one window per round.
                // Publication must carry the complete original durable
                // identity set so the rebase compares against the captured
                // live layer, rather than the shortened scratch catalog.
                output.inputs = selected_inputs.clone();
                let base = includes_base.then(|| selected_base.clone());
                let selection = merge_rebase::MergeSelection {
                    collection_id: old_collection.collection_id.clone(),
                    collection_generation: old_collection.collection_generation,
                    schema_version: old_collection.schema_version,
                    schema: old_collection.schema.clone(),
                    field: field.clone(),
                    inputs: selected_inputs.clone(),
                    vector_sidecar: base.as_ref().and_then(|base| {
                        old_collection
                            .segments
                            .iter()
                            .find(|segment| {
                                segment.role == SegmentRole::VectorEids
                                    && segment.field == base.field
                            })
                            .cloned()
                    }),
                    base,
                };
                merges.push((selection, output));
            }
            engine.prepare_checkpoint_compactions(&mut capture)?;
            self.background
                .pin_readers(source.name.as_str().to_owned(), &capture);
            self.merge_observer.observe(MergePhase::BeforePublish)?;
            let guard = self.save_gate.lock_owned();
            let save_gate_acquired = Instant::now();
            if self
                .pending_frozen
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .is_some()
            {
                return Ok(MergeOutcome::RetryableStale);
            }
            if engine.capture_barrier.epoch() != captured_stamp.epoch {
                return Ok(MergeOutcome::RetryableStale);
            }
            let Some(latest) = self.current_record()? else {
                return Ok(MergeOutcome::RetryableStale);
            };
            if latest.legacy {
                return Ok(MergeOutcome::RetryableStale);
            }
            let latest_manifest = read_generation_manifest(&latest.path)?;
            // CURRENT may have changed while compaction ran. Refuse a v3
            // predecessor before staging or publishing any merge output.
            if !background_merge_supports_manifest(latest_manifest.schema_version) {
                return Ok(MergeOutcome::RetryableStale);
            }
            // Each rebase refuses unless its exact inputs still occur in the
            // catalog it is applied to, so folding the outputs in order keeps
            // every field's identity check as strict as a single-field job.
            let mut manifest = latest_manifest.clone();
            for (selection, output) in &merges {
                manifest = match merge_rebase::rebase_manifest(
                    &manifest,
                    selection,
                    merge_rebase::VerifiedOutput {
                        field: output.output.clone(),
                        vector_sidecar: output.vector_eids.clone(),
                    },
                ) {
                    Ok(manifest) => manifest,
                    Err(_) => return Ok(MergeOutcome::RetryableStale),
                };
            }
            let (revision, mut staged) = self.begin_next_generation_selected(
                latest.sequence,
                StagingSelection::CurrentIfDurable,
            )?;
            let path = staged.path().to_path_buf();
            let link_generation_started = Instant::now();
            let mut inherited_current_files =
                link_collections_with_paths(&latest.path, &path, &latest_manifest)?;
            costs.record(
                crate::metrics::MergeStep::LinkGeneration,
                link_generation_started.elapsed(),
                inherited_current_files.len() as u64,
            );
            for (_, output) in &merges {
                for reference in std::iter::once(&output.output).chain(output.vector_eids.iter()) {
                    for relative in std::iter::once(&reference.path)
                        .chain(reference.local_rows.iter().map(|rows| &rows.path))
                    {
                        let destination = confined(&path, relative)?;
                        if destination.exists() {
                            std::fs::remove_file(&destination)?;
                        }
                        std::fs::hard_link(confined(&scratch_path, relative)?, destination)?;
                        inherited_current_files.remove(relative);
                    }
                }
            }
            // Remove replaced input links which no longer occur in the full catalog.
            let retained: BTreeSet<_> = manifest
                .collections
                .iter()
                .flat_map(|collection| collection.segments.iter())
                .flat_map(|segment| {
                    std::iter::once(segment.path.as_str())
                        .chain(segment.local_rows.iter().map(|rows| rows.path.as_str()))
                })
                .collect();
            for (_, output) in &merges {
                for reference in &output.inputs {
                    for relative in std::iter::once(&reference.path)
                        .chain(reference.local_rows.iter().map(|rows| &rows.path))
                    {
                        if !retained.contains(relative.as_str()) {
                            std::fs::remove_file(confined(&path, relative)?)?;
                            inherited_current_files.remove(relative);
                        }
                    }
                }
            }
            manifest.revision = revision;
            manifest.previous = Some(latest.name.as_str().to_owned());
            let manifest_write_started = Instant::now();
            write_generation_manifest(&path, &manifest)?;
            costs.record(
                crate::metrics::MergeStep::ManifestWrite,
                manifest_write_started.elapsed(),
                1,
            );
            let record = GenerationRecord {
                name: staged.generation().clone(),
                path: path.clone(),
                sequence: latest.sequence,
                revision,
                legacy: false,
                previous: Some(latest.name.clone()),
            };
            let validate_started = Instant::now();
            validate_generation_layout_with_prior(&record, Some((&latest.path, &latest_manifest)))?;
            costs.record(
                crate::metrics::MergeStep::ValidateLayout,
                validate_started.elapsed(),
                0,
            );
            let inherit_started = Instant::now();
            let mut inherited = 0u64;
            if let GenerationStaging::Current(current_stage) = &mut staged {
                for relative in &inherited_current_files {
                    current_stage.inherit_current_file(relative)?;
                    inherited += 1;
                }
            }
            costs.record(
                crate::metrics::MergeStep::InheritFiles,
                inherit_started.elapsed(),
                inherited,
            );
            let pending_started = Instant::now();
            let pending = telemetry::pending_deltas(&path, &manifest.collections)?;
            costs.record(
                crate::metrics::MergeStep::PendingDeltas,
                pending_started.elapsed(),
                0,
            );
            capture.collections = identities(&manifest);
            let mut owner_publication = None;
            let mut capture_publish_elapsed = Duration::ZERO;
            let commit = staged.commit_with_publication_guard(&self.generations, || {
                owner_publication = self
                    .publication_fence
                    .as_ref()
                    .map(|fence| fence.acquire())
                    .transpose()
                    .map_err(std::io::Error::other)?;
                let publication = engine
                    .capture_barrier
                    .capture(latest.sequence)
                    .map_err(std::io::Error::other)?;
                // The catalog can lag a live schema/drop mutation that has not
                // checkpointed yet. Do not publish a merge of that old identity.
                let capture_publish_started = Instant::now();
                engine
                    .capture_background_merge(identities(&manifest))
                    .map_err(std::io::Error::other)?;
                capture_publish_elapsed += capture_publish_started.elapsed();
                let pin = publication
                    .publication_pin(captured_stamp)
                    .map_err(std::io::Error::other)?;
                drop(publication);
                Ok(pin)
            });
            if let Err(error) = commit {
                if error.class() == storage_durable::CommitFailureClass::CommitUncertain {
                    engine.capture_barrier.apply().mark_uncertain();
                }
                return Err(anyhow::Error::new(error)).context("publish rebased background merge");
            }
            *self
                .verified_catalog
                .lock()
                .unwrap_or_else(|p| p.into_inner()) =
                Some((record.name.clone(), serde_json::to_vec(&manifest)?));
            let binding = engine
                .capture_barrier
                .capture(latest.sequence)
                .map_err(anyhow::Error::msg)?;
            binding
                .validate_publish(captured_stamp)
                .map_err(anyhow::Error::msg)?;
            engine.bind_background_merge(&self.root.join(record.name.as_str()), &mut capture)?;
            drop(binding);
            drop(owner_publication);
            for (_, output) in &merges {
                engine
                    .metrics()
                    .observe_segment_merge(output.logical_read_bytes, output.logical_write_bytes);
            }
            engine
                .metrics()
                .set_segment_pending_delta(pending.0, pending.1);
            costs.record(
                crate::metrics::MergeStep::CapturePublish,
                capture_publish_elapsed,
                0,
            );
            save_gate_held = save_gate_acquired.elapsed();
            drop(guard);
            self.merge_observer.observe(MergePhase::AfterPublish)?;
            Ok(MergeOutcome::Published)
        })();
        // Source files and scratch stay protected through all detached I/O.
        // On panic this cleanup is not reached and the worker retains the pins.
        self.cleanup_owned_merge_staging(
            &scratch_path,
            &scratch_name,
            source.name.as_str(),
            scratch,
        )?;
        if matches!(result, Ok(MergeOutcome::Published)) {
            costs.publish(
                engine.metrics(),
                job_started.elapsed(),
                save_gate_held,
                compacted_fields,
            );
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
        QueryNode, SearchRequest, TermQuery,
    };
    use std::sync::mpsc;

    const TEST_TIMEOUT: Duration = Duration::from_secs(2);
    const CASE_TIMEOUT: Duration = Duration::from_secs(10);

    #[test]
    fn v2_generation_manifests_use_directory_layout() {
        assert!(!uses_flat_generation_layout(GENERATION_MANIFEST_V2));
        assert!(uses_flat_generation_layout(GENERATION_MANIFEST_V3));
    }

    #[test]
    fn background_merge_does_not_publish_v3_manifest() {
        assert!(background_merge_supports_manifest(GENERATION_MANIFEST_V2));
        assert!(!background_merge_supports_manifest(GENERATION_MANIFEST_V3));
    }

    #[derive(Default)]
    struct BlockingMergeState {
        before_encode: u64,
        before_publish: u64,
        after_publish: u64,
        release_first_encode: bool,
        release_second_publish: bool,
    }

    #[derive(Default)]
    struct BlockingMergeObserver {
        state: Mutex<BlockingMergeState>,
        changed: Condvar,
    }

    struct ObserverRelease(Arc<BlockingMergeObserver>);

    impl Drop for ObserverRelease {
        fn drop(&mut self) {
            self.0.release_all();
        }
    }

    impl BlockingMergeObserver {
        fn wait_until(&self, predicate: impl Fn(&BlockingMergeState) -> bool, message: &str) {
            let deadline = Instant::now() + CASE_TIMEOUT;
            let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
            while !predicate(&state) {
                let left = deadline.saturating_duration_since(Instant::now());
                assert!(!left.is_zero(), "{message}");
                state = self
                    .changed
                    .wait_timeout(state, left)
                    .unwrap_or_else(|p| p.into_inner())
                    .0;
            }
        }

        fn release_first_encode(&self) {
            let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
            state.release_first_encode = true;
            self.changed.notify_all();
        }

        fn release_all(&self) {
            let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
            state.release_first_encode = true;
            state.release_second_publish = true;
            self.changed.notify_all();
        }

        fn held_observations(&self) -> (u64, u64, u64) {
            let state = self.state.lock().unwrap_or_else(|p| p.into_inner());
            (
                state.before_encode,
                state.before_publish,
                state.after_publish,
            )
        }
    }

    impl MergeObserver for BlockingMergeObserver {
        fn observe(&self, phase: MergePhase) -> std::io::Result<()> {
            let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
            match phase {
                MergePhase::BeforeEncode => {
                    state.before_encode += 1;
                    self.changed.notify_all();
                    if state.before_encode == 1 {
                        while !state.release_first_encode {
                            state = self.changed.wait(state).unwrap_or_else(|p| p.into_inner());
                        }
                    }
                }
                MergePhase::BeforePublish => {
                    state.before_publish += 1;
                    self.changed.notify_all();
                    if state.before_publish >= 2 {
                        while !state.release_second_publish {
                            state = self.changed.wait(state).unwrap_or_else(|p| p.into_inner());
                        }
                    }
                }
                MergePhase::AfterPublish => {
                    state.after_publish += 1;
                    self.changed.notify_all();
                }
            }
            Ok(())
        }
    }

    fn capacity_schema() -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
        for field in ["a_capacity", "z_unrelated"] {
            fields.insert(
                field.to_owned(),
                FieldSpec {
                    field_type: FieldType::Keyword,
                    analyzer: None,
                    multi: None,
                    dim: None,
                    metric: None,
                    backend: None,
                    quantize: None,
                },
            );
        }
        CreateCollectionRequest { fields }
    }

    fn index_fields(engine: &Engine, a_value: &str, z_value: Option<&str>) {
        let mut items = vec![IndexItem {
            external_id: "row".into(),
            field: "a_capacity".into(),
            value: FieldValue::String(a_value.into()),
            version: None,
        }];
        if let Some(z_value) = z_value {
            items.push(IndexItem {
                external_id: "row".into(),
                field: "z_unrelated".into(),
                value: FieldValue::String(z_value.into()),
                version: None,
            });
        }
        engine
            .index(
                "u",
                IndexRequest {
                    items,
                    request_id: None,
                },
            )
            .unwrap();
    }

    fn current_manifest(store: &SegmentRdbStore) -> SegmentGenerationManifest {
        let CurrentTarget::Generation(name) = store.generations.read_current().unwrap() else {
            panic!("capacity fixture must publish CURRENT")
        };
        read_generation_manifest(&store.root.join(name.as_str())).unwrap()
    }

    fn field_delta_count(manifest: &SegmentGenerationManifest, field: &str) -> usize {
        manifest.collections[0]
            .segments
            .iter()
            .filter(|segment| {
                segment.role == SegmentRole::Field
                    && segment.kind == SegmentKind::Delta
                    && segment.field.as_deref() == Some(field)
            })
            .count()
    }

    fn has_capacity_value(engine: &Engine, value: &str) -> bool {
        !engine
            .search(
                "u",
                SearchRequest {
                    query: QueryNode::Term(TermQuery {
                        field: "a_capacity".into(),
                        value: FieldValue::String(value.into()),
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
            .unwrap()
            .hits
            .is_empty()
    }

    fn vector_capacity_schema(
        backend: Option<crate::types::VectorBackend>,
    ) -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
        fields.insert(
            "v_capacity".to_owned(),
            FieldSpec {
                field_type: FieldType::Vector,
                analyzer: None,
                multi: None,
                dim: Some(4),
                metric: Some(crate::types::VectorMetric::L2),
                backend,
                quantize: None,
            },
        );
        CreateCollectionRequest { fields }
    }

    fn put_vector(engine: &Engine, id: &str, value: Vec<f32>) {
        engine
            .index(
                "u",
                IndexRequest {
                    items: vec![IndexItem {
                        external_id: id.into(),
                        field: "v_capacity".into(),
                        value: FieldValue::Vector(value),
                        version: None,
                    }],
                    request_id: None,
                },
            )
            .unwrap();
    }

    fn put_keyword_row(engine: &Engine, field: &str, id: &str, value: &str) {
        engine
            .index(
                "u",
                IndexRequest {
                    items: vec![IndexItem {
                        external_id: id.into(),
                        field: field.into(),
                        value: FieldValue::String(value.into()),
                        version: None,
                    }],
                    request_id: None,
                },
            )
            .unwrap();
    }

    /// Build one field a two-phase way: phase A folds a wide row set (plus a
    /// few tiny spacer rows) into a real, sizable base via one background
    /// merge; phase B then writes exactly four small deltas on top of that
    /// base (nothing else touches the store while they land) and drains
    /// exactly one background merge job. `wide_rows` and `spacer` build phase
    /// A's base; `small_delta` writes phase B's small per-sequence change.
    fn assert_whole_delta_stack_compacts_in_one_job(
        schema: CreateCollectionRequest,
        field: &str,
        wide_rows: impl Fn(&Engine),
        spacer: impl Fn(&Engine, u64),
        small_delta: impl Fn(&Engine, u64),
    ) {
        let directory = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(directory.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", schema).unwrap();
        wide_rows(&engine);
        store.save_required(&engine, 1).unwrap();
        for sequence in 2..=4 {
            spacer(&engine, sequence);
            store.save_required(&engine, sequence).unwrap();
        }
        store
            .wait_for_merges(Duration::from_secs(10))
            .expect("phase A background merge worker must drain before phase B");
        let based = current_manifest(&store);
        assert_eq!(
            field_delta_count(&based, field),
            0,
            "phase A must fold the wide row set into the base before phase B"
        );

        for sequence in 5..=8 {
            small_delta(&engine, sequence);
            store.save_required(&engine, sequence).unwrap();
        }
        store
            .wait_for_merges(Duration::from_secs(10))
            .expect("phase B background merge worker must drain before assertions");
        let merged = current_manifest(&store);
        assert_eq!(
            field_delta_count(&merged, field),
            3,
            "one background merge job folds one adjacent pair and leaves the remaining deltas"
        );
    }

    #[test]
    fn background_merge_compacts_the_whole_delta_stack_in_one_job() {
        assert_whole_delta_stack_compacts_in_one_job(
            capacity_schema(),
            "a_capacity",
            |engine| {
                for row in 0..8000 {
                    put_keyword_row(
                        engine,
                        "a_capacity",
                        &format!("wide-{row}"),
                        &format!("wide-value-with-extra-padding-bytes-{row}"),
                    );
                }
            },
            |engine, sequence| {
                put_keyword_row(engine, "a_capacity", &format!("spacer-{sequence}"), "s");
            },
            |engine, sequence| {
                put_keyword_row(engine, "a_capacity", "hot", &format!("a-{sequence}"));
            },
        );
    }

    #[test]
    fn background_merge_compacts_the_whole_delta_stack_for_a_flat_cpu_vector_field() {
        assert_whole_delta_stack_compacts_in_one_job(
            vector_capacity_schema(Some(crate::types::VectorBackend::FlatCpu)),
            "v_capacity",
            |engine| {
                for row in 0..2000 {
                    put_vector(
                        engine,
                        &format!("wide-{row}"),
                        vec![row as f32, 0.0, 0.0, 0.0],
                    );
                }
            },
            |engine, sequence| {
                put_vector(
                    engine,
                    &format!("spacer-{sequence}"),
                    vec![0.0, 0.0, 0.0, 0.0],
                );
            },
            |engine, sequence| {
                put_vector(engine, "hot", vec![sequence as f32, 1.0, 0.0, 0.0]);
            },
        );
    }

    #[test]
    fn background_merge_compacts_the_whole_delta_stack_for_an_hnsw_vector_field() {
        assert_whole_delta_stack_compacts_in_one_job(
            vector_capacity_schema(Some(crate::types::VectorBackend::HnswCpu)),
            "v_capacity",
            |engine| {
                for row in 0..2000 {
                    put_vector(
                        engine,
                        &format!("wide-{row}"),
                        vec![row as f32, 0.0, 0.0, 0.0],
                    );
                }
            },
            |engine, sequence| {
                put_vector(
                    engine,
                    &format!("spacer-{sequence}"),
                    vec![0.0, 0.0, 0.0, 0.0],
                );
            },
            |engine, sequence| {
                put_vector(engine, "hot", vec![sequence as f32, 1.0, 0.0, 0.0]);
            },
        );
    }

    fn keyword_schema(fields: &[&str]) -> CreateCollectionRequest {
        let mut map = BTreeMap::new();
        for field in fields {
            map.insert(
                (*field).to_owned(),
                FieldSpec {
                    field_type: FieldType::Keyword,
                    analyzer: None,
                    multi: None,
                    dim: None,
                    metric: None,
                    backend: None,
                    quantize: None,
                },
            );
        }
        CreateCollectionRequest { fields: map }
    }

    fn put_row(engine: &Engine, collection: &str, field: &str, id: &str, value: &str) {
        engine
            .index(
                collection,
                IndexRequest {
                    items: vec![IndexItem {
                        external_id: id.into(),
                        field: field.into(),
                        value: FieldValue::String(value.into()),
                        version: None,
                    }],
                    request_id: None,
                },
            )
            .unwrap();
    }

    fn published_merge_jobs(store: &SegmentRdbStore) -> u64 {
        store
            .background
            .state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .published_revision
    }

    /// `select_staged_delta_window` is the whole of a job's scope decision, so
    /// its ordering is asserted directly rather than through the worker: the
    /// collection holding the deepest stack wins, an equal-depth tie keeps
    /// catalog order, and every field tied at that collection's deepest
    /// eligible depth is returned in deterministic field-name order with its
    /// own merge window.
    #[test]
    fn select_staged_delta_window_selects_all_eligible_fields_of_deepest_collection() {
        fn write(root: &Path, relative: &str, bytes: usize) {
            let path = root.join(relative);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, vec![b'x'; bytes]).unwrap();
        }

        fn segment(
            root: &Path,
            collection: &str,
            field: &str,
            kind: SegmentKind,
            ordinal: u32,
            bytes: usize,
        ) -> SegmentReference {
            let relative = format!(
                "{}/{field}-{ordinal}-{}.lseg",
                collection_checkpoint_dir_name(collection),
                match kind {
                    SegmentKind::Base => "base",
                    SegmentKind::Delta => "delta",
                }
            );
            write(root, &relative, bytes);
            SegmentReference {
                role: SegmentRole::Field,
                field: Some(field.to_owned()),
                ordinal,
                kind,
                format: SegmentFormat::LsegV1,
                path: relative,
                local_rows: None,
                applied_seq: None,
                payload_sha256: None,
            }
        }

        fn catalog(
            root: &Path,
            collection: &str,
            depths: &[(&str, u32)],
            base_bytes: usize,
        ) -> CollectionCatalog {
            let mut segments = Vec::new();
            for (field, depth) in depths {
                segments.push(segment(
                    root,
                    collection,
                    field,
                    SegmentKind::Base,
                    0,
                    base_bytes,
                ));
                for ordinal in 1..=*depth {
                    segments.push(segment(
                        root,
                        collection,
                        field,
                        SegmentKind::Delta,
                        ordinal,
                        1,
                    ));
                }
            }
            CollectionCatalog {
                collection_id: collection.to_owned(),
                collection_generation: 1,
                schema_version: 1,
                data_version: 1,
                schema: serde_json::json!({}),
                segments,
            }
        }

        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        // A base far larger than the deltas keeps every candidate a
        // delta-only merge, so only the stack depth decides which collection
        // is selected.
        let deepest = vec![
            catalog(root, "first", &[("a", 4), ("b", 4)], 4096),
            catalog(root, "second", &[("a", 6), ("b", 3)], 4096),
        ];
        let selected = select_staged_delta_window(root, &deepest).unwrap();
        assert_eq!(selected.len(), 1, "the deepest field is the only candidate");
        assert_eq!(selected[0].collection_id, "second");
        assert_eq!(selected[0].field, "a");
        assert_eq!(selected[0].inputs.len(), 2);
        assert!(
            !selected[0].includes_base,
            "deltas smaller than the base must not fold the base in"
        );

        // Only fields at the selected collection depth are admitted. The
        // shallower eligible field remains for a later scheduling pass.
        let both_eligible = vec![
            catalog(root, "third", &[("a", 4), ("b", 4)], 4096),
            catalog(root, "fourth", &[("a", 7), ("b", 6)], 4096),
        ];
        let selected = select_staged_delta_window(root, &both_eligible).unwrap();
        assert_eq!(
            selected
                .iter()
                .map(|candidate| (candidate.collection_id.as_str(), candidate.field.as_str()))
                .collect::<Vec<_>>(),
            vec![("fourth", "a")],
            "the deepest collection and deepest field are selected deterministically"
        );
        assert!(selected
            .iter()
            .all(|candidate| candidate.inputs.len() == 2 && !candidate.includes_base));

        let tied = vec![
            catalog(root, "first", &[("a", 7), ("b", 7)], 4096),
            catalog(root, "second", &[("a", 7), ("b", 7)], 4096),
        ];
        let selected = select_staged_delta_window(root, &tied).unwrap();
        assert_eq!(
            selected
                .iter()
                .map(|candidate| (candidate.collection_id.as_str(), candidate.field.as_str()))
                .collect::<Vec<_>>(),
            vec![("first", "a"), ("first", "b")],
            "an equal-depth tie keeps catalog order and selects all eligible fields"
        );

        let below_threshold = vec![catalog(root, "first", &[("a", 3)], 4096)];
        assert!(
            select_staged_delta_window(root, &below_threshold)
                .unwrap()
                .is_empty(),
            "a stack below the merge threshold is not a candidate"
        );

        // Deltas that have reached the base size fold the base in.
        let over_base = vec![catalog(root, "first", &[("a", 4)], 2)];
        let selected = select_staged_delta_window(root, &over_base).unwrap();
        assert_eq!(selected.len(), 1);
        assert!(
            selected[0].includes_base,
            "a delta stack at or past the base size must fold the base in"
        );
    }

    #[test]
    fn select_staged_delta_window_uses_smallest_adjacent_pair() {
        fn write(root: &Path, relative: &str, bytes: usize) {
            std::fs::write(root.join(relative), vec![b'x'; bytes]).unwrap();
        }

        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let collection = {
            let mut collections = vec![{
                let mut collection = CollectionCatalog {
                    collection_id: "pair".to_owned(),
                    collection_generation: 1,
                    schema_version: 1,
                    data_version: 1,
                    schema: serde_json::json!({}),
                    segments: Vec::new(),
                };
                collection.segments.push(SegmentReference {
                    role: SegmentRole::Field,
                    field: Some("f".to_owned()),
                    ordinal: 0,
                    kind: SegmentKind::Base,
                    format: SegmentFormat::LsegV1,
                    path: "base.lseg".to_owned(),
                    local_rows: None,
                    applied_seq: None,
                    payload_sha256: None,
                });
                for ordinal in 1..=5 {
                    collection.segments.push(SegmentReference {
                        role: SegmentRole::Field,
                        field: Some("f".to_owned()),
                        ordinal,
                        kind: SegmentKind::Delta,
                        format: SegmentFormat::LsegV1,
                        path: format!("delta-{ordinal}.lseg"),
                        local_rows: None,
                        applied_seq: None,
                        payload_sha256: None,
                    });
                }
                collection
            }];
            write(root, "base.lseg", 1000);
            for (ordinal, bytes) in [20, 1, 2, 30, 4].into_iter().enumerate() {
                write(root, &format!("delta-{}.lseg", ordinal + 1), bytes);
            }
            collections.pop().unwrap()
        };
        let selected = select_staged_delta_window(root, &[collection.clone()]).unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(
            selected[0]
                .inputs
                .iter()
                .map(|segment| segment.ordinal)
                .collect::<Vec<_>>(),
            vec![2, 3]
        );
    }

    /// The shape of one drained merge job: what each hard-link pass paid for,
    /// and what the published generation holds afterwards.
    struct MergeJobShape {
        jobs: u64,
        scratch_files: u64,
        generation_link_files: u64,
        generation_files: usize,
        hot_files: usize,
    }

    fn term_hits(engine: &Engine, collection: &str, field: &str, value: &str) -> usize {
        engine
            .search(
                collection,
                SearchRequest {
                    query: QueryNode::Term(TermQuery {
                        field: field.into(),
                        value: FieldValue::String(value.into()),
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
            .unwrap()
            .hits
            .len()
    }

    /// Drive one hot field over the merge threshold in a root that also holds
    /// `idle_collections` collections nothing ever writes to again, drain the
    /// background worker, and report what the single published job cost.
    /// Every assertion here is invariant in `idle_collections`, so the caller
    /// can compare two runs directly.
    fn drain_one_merge_job(idle_collections: usize) -> MergeJobShape {
        use crate::metrics::MergeStep;

        const HOT_FIELD: &str = "h_alpha";
        const IDLE_FIELDS: [&str; 2] = ["i_alpha", "i_beta"];

        let directory = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(directory.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine
            .create_collection("hot", keyword_schema(&[HOT_FIELD]))
            .unwrap();
        let idle: Vec<String> = (0..idle_collections)
            .map(|index| format!("idle-{index:03}"))
            .collect();
        for collection in &idle {
            engine
                .create_collection(collection, keyword_schema(&IDLE_FIELDS))
                .unwrap();
        }
        put_row(&engine, "hot", HOT_FIELD, "seed", "seed-value");
        for collection in &idle {
            for field in IDLE_FIELDS {
                put_row(&engine, collection, field, "seed", "seed-value");
            }
        }
        store.save_required(&engine, 1).unwrap();
        for sequence in 2..=4u64 {
            put_row(&engine, "hot", HOT_FIELD, "hot", &format!("hot-{sequence}"));
            store.save_required(&engine, sequence).unwrap();
        }
        store
            .wait_for_merges(Duration::from_secs(120))
            .expect("background merge worker must drain");

        let metrics = engine.metrics();
        let (_, scratch_observations, scratch_files) =
            metrics.segment_merge_step_observation(MergeStep::LinkScratch);
        let (_, _, generation_link_files) =
            metrics.segment_merge_step_observation(MergeStep::LinkGeneration);
        assert_eq!(
            scratch_observations, 1,
            "this fixture must publish exactly one merge job"
        );

        let CurrentTarget::Generation(name) = store.generations.read_current().unwrap() else {
            panic!("the merge fixture must publish CURRENT")
        };
        let record = store.record_for_name(name.clone()).unwrap();
        validate_generation_layout(&record)
            .expect("the published generation must keep a valid layout");
        let generation = store.root.join(name.as_str());
        let (generation_files, _) = count_tree(&generation);
        let (hot_files, _) = count_tree(&generation.join(collection_checkpoint_dir_name("hot")));

        // Cold-open the published generation: every collection, hot and idle,
        // must still answer for the rows it last wrote.
        let (cold, sequence) = store.load_latest().unwrap().unwrap();
        assert_eq!(sequence, 4);
        assert_eq!(
            term_hits(&cold, "hot", HOT_FIELD, "hot-4"),
            1,
            "the compacted field must answer for its newest layer"
        );
        assert_eq!(
            term_hits(&cold, "hot", HOT_FIELD, "seed-value"),
            1,
            "the compacted field must answer for its oldest layer"
        );
        for collection in &idle {
            for field in IDLE_FIELDS {
                assert_eq!(
                    term_hits(&cold, collection, field, "seed-value"),
                    1,
                    "an idle collection must survive the merge: {collection}/{field}"
                );
            }
        }

        MergeJobShape {
            jobs: published_merge_jobs(&store),
            scratch_files,
            generation_link_files,
            generation_files,
            hot_files,
        }
    }

    /// A merge job's scratch stage is job-private — it never carries a
    /// generation manifest and no reader ever opens it — so linking every
    /// idle collection into it buys nothing and costs one hard link per file
    /// in the whole root. The oracle is structural: the same fixture run at
    /// two idle-collection counts must link the same number of files into
    /// scratch, bounded by the compacted collection's own file count.
    #[test]
    fn scratch_link_cost_is_independent_of_idle_collection_count() {
        let small = drain_one_merge_job(4);
        let large = drain_one_merge_job(20);

        assert_eq!(small.jobs, 1, "the small fixture must publish one job");
        assert_eq!(large.jobs, 1, "the large fixture must publish one job");
        assert!(
            large.generation_files > small.generation_files,
            "the fixtures must differ in root size: {} vs {}",
            small.generation_files,
            large.generation_files
        );
        assert_eq!(
            large.scratch_files, small.scratch_files,
            "the scratch link pass must cost the compacted collection, not the root: \
             {} files at 20 idle collections vs {} at 4",
            large.scratch_files, small.scratch_files
        );
        assert!(
            large.scratch_files <= large.hot_files as u64 + 8,
            "the scratch link pass must stay within the compacted collection: \
             {} files linked, compacted collection holds {}",
            large.scratch_files,
            large.hot_files
        );
    }

    /// The publication-side pass is a different contract from the scratch
    /// pass: the new generation inherits every file of the one it replaces, so
    /// its link count must still cover the whole root and grow with it.
    #[test]
    fn generation_link_pass_still_covers_the_whole_root() {
        let small = drain_one_merge_job(4);
        let large = drain_one_merge_job(20);

        assert!(
            large.generation_link_files >= (large.generation_files - large.hot_files) as u64,
            "the generation link pass must cover every idle file: linked {} of {} idle files",
            large.generation_link_files,
            large.generation_files - large.hot_files
        );
        assert!(
            large.generation_link_files > small.generation_link_files,
            "the generation link pass is proportional to the root: {} vs {}",
            large.generation_link_files,
            small.generation_link_files
        );
    }

    /// Walk one generation directory, returning `(files, directories)` under
    /// the subtree rooted at `path`.
    fn count_tree(path: &Path) -> (usize, usize) {
        let mut files = 0;
        let mut dirs = 0;
        let Ok(entries) = std::fs::read_dir(path) else {
            return (0, 0);
        };
        for entry in entries.flatten() {
            let metadata = std::fs::symlink_metadata(entry.path()).unwrap();
            if metadata.is_dir() {
                dirs += 1;
                let (nested_files, nested_dirs) = count_tree(&entry.path());
                files += nested_files;
                dirs += nested_dirs;
            } else {
                files += 1;
            }
        }
        (files, dirs)
    }

    /// A published merge job must leave a per-step cost breakdown in the same
    /// metric registry `/metrics` renders from. Without it the only readable
    /// fact about a multi-second job is that it finished, which cannot say
    /// whether the whole-root link/inherit passes or the per-field payload
    /// dominated it.
    #[test]
    fn a_published_merge_job_records_every_phase_step_in_the_metric_registry() {
        use crate::metrics::MergeStep;

        const IDLE_COLLECTIONS: usize = 8;
        const HOT_FIELDS: [&str; 1] = ["h_alpha"];
        const IDLE_FIELDS: [&str; 2] = ["i_alpha", "i_beta"];

        let directory = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(directory.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine
            .create_collection("hot", keyword_schema(&HOT_FIELDS))
            .unwrap();
        let idle: Vec<String> = (0..IDLE_COLLECTIONS)
            .map(|index| format!("idle-{index:03}"))
            .collect();
        for collection in &idle {
            engine
                .create_collection(collection, keyword_schema(&IDLE_FIELDS))
                .unwrap();
        }
        for field in HOT_FIELDS {
            put_row(&engine, "hot", field, "seed", "seed-value");
        }
        for collection in &idle {
            for field in IDLE_FIELDS {
                put_row(&engine, collection, field, "seed", "seed-value");
            }
        }
        store.save_required(&engine, 1).unwrap();
        for sequence in 2..=4u64 {
            for field in HOT_FIELDS {
                put_row(&engine, "hot", field, "hot", &format!("hot-{sequence}"));
            }
            store.save_required(&engine, sequence).unwrap();
        }
        store
            .wait_for_merges(Duration::from_secs(120))
            .expect("background merge worker must drain");
        assert_eq!(
            published_merge_jobs(&store),
            1,
            "this fixture must publish exactly one merge job"
        );

        let metrics = engine.metrics();
        for step in MergeStep::ALL {
            let (_, count, _) = metrics.segment_merge_step_observation(step);
            assert_eq!(
                count,
                1,
                "one published job must observe phase {} exactly once",
                step.name()
            );
        }
        let (total_us, _, _) = metrics.segment_merge_step_observation(MergeStep::Total);
        let parts: u64 = MergeStep::ALL
            .iter()
            .filter(|step| **step != MergeStep::Total)
            .map(|step| metrics.segment_merge_step_observation(*step).0)
            .sum();
        assert!(
            parts <= total_us,
            "the named phases must be disjoint spans inside the job total: \
             parts={parts}us total={total_us}us"
        );
        for step in [MergeStep::LinkScratch, MergeStep::LinkGeneration] {
            let (_, _, files) = metrics.segment_merge_step_observation(step);
            assert!(
                files > 0,
                "{} must report the files it hard-linked",
                step.name()
            );
        }
        assert!(
            metrics.segment_merge_linked_files_total.get()
                >= metrics
                    .segment_merge_step_observation(MergeStep::LinkScratch)
                    .2,
            "the linked-file counter must cover both whole-root link passes"
        );
        assert_eq!(
            metrics.segment_merge_fields_total.get(),
            1,
            "this fixture's collection has one eligible field, and the job \
             must report it"
        );
        assert_eq!(
            metrics.segment_merge_save_gate_count.get(),
            1,
            "the publication-side save_gate hold must be observed once per job"
        );
    }

    /// MEASUREMENT, not a gate: prints one background merge job's cost broken
    /// down by phase for a root of `LUMEN_MEASURE_COLLECTIONS` idle
    /// collections (default 1) plus one hot collection. It asserts nothing
    /// about wall-clock — a timing assertion here would fail on a loaded
    /// machine for reasons that have nothing to do with lumen — so it stays
    /// `#[ignore]`d and is run by hand:
    ///
    /// ```text
    /// LUMEN_MEASURE_COLLECTIONS=182 cargo test -p lumen --lib -- --ignored \
    ///   --exact segment_rdb::background::tests::\
    ///   measure_background_merge_job_cost_by_collection_count --nocapture
    /// ```
    ///
    /// The phase numbers are read back from the same metric registry
    /// `/metrics` renders, so the breakdown cannot drift from what a
    /// production scrape of the same build would report.
    #[test]
    #[ignore = "measurement: prints a per-phase cost breakdown, asserts no wall-clock budget"]
    fn measure_background_merge_job_cost_by_collection_count() {
        use crate::metrics::MergeStep;

        const HOT_FIELDS: [&str; 3] = ["h_alpha", "h_beta", "h_gamma"];
        // The durable perf probe's schema width, so an idle collection here
        // costs the same number of field files it costs in production.
        const IDLE_FIELD_COUNT: usize = 14;

        let idle_collections: usize = std::env::var("LUMEN_MEASURE_COLLECTIONS")
            .ok()
            .and_then(|raw| raw.trim().parse().ok())
            .unwrap_or(1);
        let idle_field_names: Vec<String> = (0..IDLE_FIELD_COUNT)
            .map(|index| format!("f{index:02}"))
            .collect();
        let idle_fields: Vec<&str> = idle_field_names.iter().map(String::as_str).collect();

        let directory = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(directory.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine
            .create_collection("hot", keyword_schema(&HOT_FIELDS))
            .unwrap();
        let idle: Vec<String> = (0..idle_collections)
            .map(|index| format!("idle-{index:04}"))
            .collect();
        for collection in &idle {
            engine
                .create_collection(collection, keyword_schema(&idle_fields))
                .unwrap();
        }
        for field in HOT_FIELDS {
            put_row(&engine, "hot", field, "seed", "seed-value");
        }
        for collection in &idle {
            for field in &idle_fields {
                put_row(&engine, collection, field, "seed", "seed-value");
            }
        }
        store.save_required(&engine, 1).unwrap();
        for sequence in 2..=4u64 {
            for field in HOT_FIELDS {
                put_row(&engine, "hot", field, "hot", &format!("hot-{sequence}"));
            }
            store.save_required(&engine, sequence).unwrap();
        }
        store
            .wait_for_merges(Duration::from_secs(600))
            .expect("background merge worker must drain");

        let n = idle_collections;
        let jobs = published_merge_jobs(&store);
        let metrics = engine.metrics();
        // A job publishes exactly one compacted field, so this fixture's
        // eligible fields cost one job each. Every line therefore carries the
        // per-job value as well as the sum: what a waiting checkpoint pays is
        // one job's save-gate hold, not the whole drain's.
        for step in MergeStep::ALL {
            let (micros, count, files) = metrics.segment_merge_step_observation(step);
            let seconds = micros as f64 / 1_000_000.0;
            let per_job = seconds / jobs.max(1) as f64;
            println!(
                "MEASURE n={n} phase={} seconds={seconds:.6} per_job_seconds={per_job:.6} \
                 files={files} per_job_files={} observations={count}",
                step.name(),
                files / jobs.max(1),
            );
        }
        let gate_seconds = metrics.segment_merge_save_gate_us_sum.get() as f64 / 1_000_000.0;
        println!(
            "MEASURE n={n} phase=save_gate_held seconds={gate_seconds:.6} \
             per_job_seconds={:.6} files=0",
            gate_seconds / jobs.max(1) as f64
        );
        println!(
            "MEASURE n={n} jobs={jobs} fields={} linked_files={}",
            metrics.segment_merge_fields_total.get(),
            metrics.segment_merge_linked_files_total.get()
        );

        let CurrentTarget::Generation(name) = store.generations.read_current().unwrap() else {
            panic!("the measurement fixture must publish CURRENT")
        };
        let generation = store.root.join(name.as_str());
        let (files, dirs) = count_tree(&generation);
        let hot_dir = generation.join(collection_checkpoint_dir_name("hot"));
        let (hot_files, hot_dirs) = count_tree(&hot_dir);
        println!(
            "MEASURE n={n} generation_files={files} generation_dirs={dirs} \
             hot_files={hot_files} hot_dirs={hot_dirs} idle_files={} idle_dirs={}",
            files - hot_files,
            dirs - hot_dirs
        );
    }

    #[test]
    fn capacity_wait_returns_after_a_published_merge() {
        let work = Arc::new(RootWork::default());
        {
            let mut state = work.state.lock().unwrap_or_else(|p| p.into_inner());
            state.queued = true;
        }
        let waiting = work.clone();
        let waiter = std::thread::spawn(move || {
            waiting.wait_for_capacity_progress_after(0, Instant::now() + TEST_TIMEOUT)
        });
        work.wait_for_test_wait_entry_after(0, TEST_TIMEOUT)
            .unwrap();
        {
            let mut state = work.state.lock().unwrap_or_else(|p| p.into_inner());
            state.published_revision = 1;
            state.queued = false;
            work.changed.notify_all();
        }
        assert_eq!(waiter.join().unwrap().unwrap(), CapacityWait::Published);
    }

    #[test]
    fn capacity_wait_ignores_stale_error_while_retry_is_queued() {
        let work = Arc::new(RootWork::default());
        {
            let mut state = work.state.lock().unwrap_or_else(|p| p.into_inner());
            state.queued = true;
            state.error = Some("stale failed merge".into());
        }
        let waiting = work.clone();
        let waiter = std::thread::spawn(move || {
            waiting.wait_for_capacity_progress_after(0, Instant::now() + TEST_TIMEOUT)
        });
        work.wait_for_test_wait_entry_after(0, TEST_TIMEOUT)
            .unwrap();
        {
            let mut state = work.state.lock().unwrap_or_else(|p| p.into_inner());
            state.published_revision = 1;
            state.queued = false;
            state.error = None;
            work.changed.notify_all();
        }
        assert_eq!(waiter.join().unwrap().unwrap(), CapacityWait::Published);
    }

    #[test]
    fn capacity_wait_reports_terminal_error_before_published_revision() {
        let work = RootWork::default();
        {
            let mut state = work.state.lock().unwrap_or_else(|p| p.into_inner());
            state.published_revision = 1;
            state.error = Some("terminal merge failure".into());
        }
        let error = work
            .wait_for_capacity_progress_after(0, Instant::now() + TEST_TIMEOUT)
            .expect_err("terminal error must take precedence over an older publication");
        assert!(format!("{error:#}").contains("terminal merge failure"));
    }

    #[test]
    fn capacity_wait_deadline_survives_a_spurious_wake() {
        let work = Arc::new(RootWork::default());
        {
            let mut state = work.state.lock().unwrap_or_else(|p| p.into_inner());
            state.running = true;
        }
        let deadline = Instant::now() + Duration::from_millis(50);
        let waiting = work.clone();
        let waiter =
            std::thread::spawn(move || waiting.wait_for_capacity_progress_after(0, deadline));
        work.wait_for_test_wait_entry_after(0, TEST_TIMEOUT)
            .unwrap();
        work.changed.notify_all();
        let error = waiter
            .join()
            .expect("deadline waiter must not panic")
            .expect_err("unchanged work must not turn a spurious wake into progress");
        assert!(format!("{error:#}").contains("capacity wait timed out"));
    }

    #[test]
    fn capacity_wait_observes_ready_state_even_when_scheduled_after_deadline() {
        let work = RootWork::default();
        let expired = Instant::now() - Duration::from_secs(1);
        assert_eq!(
            work.wait_for_capacity_progress_after(0, expired).unwrap(),
            CapacityWait::Idle
        );
        work.state.lock().unwrap().published_revision = 1;
        assert_eq!(
            work.wait_for_capacity_progress_after(0, expired).unwrap(),
            CapacityWait::Published
        );
    }

    #[test]
    fn capacity_request_rechecks_the_idle_revision_before_rejecting() {
        let mut state = WorkState::default();
        state.published_revision = 7;
        let deadline = Some(Instant::now() + TEST_TIMEOUT);
        assert!(state
            .validate_capacity_retry(Some(7), deadline)
            .unwrap_err()
            .to_string()
            .contains("no capacity progress"));
        state.published_revision = 8;
        state.validate_capacity_retry(Some(7), deadline).unwrap();
    }

    #[test]
    fn stale_retry_is_allowed_to_retake_capacity_request_without_self_requeue() {
        let work = RootWork::default();
        {
            let mut state = work.state.lock().unwrap();
            state.running = true;
        }
        let mut result = Ok(MergeOutcome::RetryableStale);
        assert!(!work.finish_job(&mut result, true));
        {
            let state = work.state.lock().unwrap();
            assert!(!state.queued);
            assert!(state.retryable_stale);
        }
        let mut state = work.state.lock().unwrap();
        state
            .validate_capacity_retry(Some(0), Some(Instant::now() + TEST_TIMEOUT))
            .unwrap();
        state.retryable_stale = false;
        assert!(state
            .validate_capacity_retry(Some(0), Some(Instant::now() + TEST_TIMEOUT))
            .is_err());
    }

    #[test]
    fn a_new_publication_cannot_restart_an_expired_capacity_deadline() {
        let mut state = WorkState::default();
        state.published_revision = 8;
        let expired = Some(Instant::now() - Duration::from_secs(1));
        assert!(state
            .validate_capacity_retry(Some(7), expired)
            .unwrap_err()
            .to_string()
            .contains("capacity wait timed out"));
        assert!(state
            .validate_capacity_retry(None, expired)
            .unwrap_err()
            .to_string()
            .contains("capacity wait timed out"));
    }

    #[test]
    fn an_in_progress_capacity_retry_keeps_a_new_terminal_failure_visible() {
        let mut state = WorkState::default();
        state.published_revision = 8;
        state.error = Some("new terminal failure".into());
        let expired = Some(Instant::now() - Duration::from_secs(1));
        assert!(state
            .validate_capacity_retry(Some(7), expired)
            .unwrap_err()
            .to_string()
            .contains("new terminal failure"));
        // A later independent checkpoint may request the existing normal retry.
        state.validate_capacity_retry(None, None).unwrap();
    }

    #[test]
    fn capacity_wait_reports_idle_without_spinning() {
        let work = RootWork::default();
        assert_eq!(
            work.wait_for_capacity_progress_after(0, Instant::now() + TEST_TIMEOUT)
                .unwrap(),
            CapacityWait::Idle
        );
    }

    #[test]
    fn publication_revision_overflow_stops_requested_requeue() {
        let work = RootWork::default();
        {
            let mut state = work.state.lock().unwrap_or_else(|p| p.into_inner());
            state.running = true;
            state.requested = true;
            state.published_revision = u64::MAX;
        }
        let mut result = Ok(MergeOutcome::Published);
        assert!(!work.finish_job(&mut result, true));
        assert!(result.is_err());
        {
            let state = work.state.lock().unwrap_or_else(|p| p.into_inner());
            assert!(state.publication_revision_overflowed);
            assert!(!state.running && !state.queued && !state.requested);
            assert!(state.error.as_deref().unwrap().contains("overflowed"));
        }
        {
            let mut state = work.state.lock().unwrap_or_else(|p| p.into_inner());
            state.running = true;
            state.requested = true;
        }
        let mut later = Ok(MergeOutcome::NoEligibleWork);
        assert!(!work.finish_job(&mut later, true));
        let state = work.state.lock().unwrap_or_else(|p| p.into_inner());
        assert!(state.publication_revision_overflowed);
        assert!(!state.running && !state.queued && !state.requested);
        assert!(state.error.as_deref().unwrap().contains("overflowed"));
    }

    #[test]
    fn publication_revision_overflow_rejects_later_request() {
        let directory = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(directory.path()).unwrap();
        let engine = Arc::new(Engine::new());
        {
            let mut state = store
                .background
                .state
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            state.publication_revision_overflowed = true;
            state.error = Some("background segment merge publication revision overflowed".into());
        }
        let error = store
            .request_merge_with_revision(&engine)
            .expect_err("a terminal publication revision overflow must reject a new request");
        assert!(format!("{error:#}").contains("overflowed"));
        let state = store
            .background
            .state
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        assert!(state.publication_revision_overflowed);
        assert!(state.error.as_deref().unwrap().contains("overflowed"));
    }

    fn capacity_progress_case_runs_in_its_own_process(test_name: &str) -> bool {
        const CHILD: &str = "LUMEN_CHECKPOINT_CAPACITY_UNIT_CHILD";
        if std::env::var_os(CHILD).as_deref() == Some(std::ffi::OsStr::new("1")) {
            return false;
        }
        // This case intentionally parks the process-wide merge worker. A child
        // keeps that pause from blocking unrelated, parallel library tests.
        let mut output = tempfile::tempfile().unwrap();
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", test_name, "--nocapture"])
            .env(CHILD, "1")
            .stdout(std::process::Stdio::from(output.try_clone().unwrap()))
            .stderr(std::process::Stdio::from(output.try_clone().unwrap()))
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(120);
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => {}
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("could not observe isolated checkpoint capacity case: {error}");
                }
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("isolated checkpoint capacity case timed out");
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        use std::io::{Read as _, Seek as _};
        output.rewind().unwrap();
        let mut diagnostic = String::new();
        output
            .take(64 * 1024)
            .read_to_string(&mut diagnostic)
            .unwrap();
        assert!(
            status.success(),
            "isolated checkpoint capacity case failed: {diagnostic}"
        );
        assert!(
            diagnostic.contains("1 passed; 0 failed"),
            "isolated checkpoint capacity case did not execute exactly its test: {diagnostic}"
        );
        true
    }

    #[test]
    fn checkpoint_capacity_wait_yields_after_its_field_merge_while_follow_up_merge_runs() {
        if capacity_progress_case_runs_in_its_own_process(
            "segment_rdb::background::tests::checkpoint_capacity_wait_yields_after_its_field_merge_while_follow_up_merge_runs",
        ) {
            return;
        }
        let directory = tempfile::tempdir().unwrap();
        let observer = Arc::new(BlockingMergeObserver::default());
        let store =
            SegmentRdbStore::with_merge_observer(directory.path(), observer.clone()).unwrap();
        let engine = Arc::new(Engine::new());
        let _release_on_drop = ObserverRelease(observer.clone());
        engine.create_collection("u", capacity_schema()).unwrap();
        // Publish an empty base: first-save indexed rows already form a delta.
        store.save_required(&engine, 1).unwrap();

        // `z_unrelated` never receives a write, so it never becomes an
        // eligible delta field even though it shares "u" with `a_capacity`:
        // one job now compacts every eligible field of its selected
        // collection, so leaving `z_unrelated` ineligible (rather than merely
        // shallower) is what keeps it untouched by the merge below.
        for sequence in 2..=17 {
            index_fields(&engine, &format!("a-{sequence}"), None);
            store.save_required(&engine, sequence).unwrap();
            if sequence == 5 {
                observer.wait_until(
                    |state| state.before_encode >= 1,
                    "first background merge must pause before encode",
                );
            }
        }
        let setup = current_manifest(&store);
        assert_eq!(field_delta_count(&setup, "a_capacity"), 16);
        assert_eq!(field_delta_count(&setup, "z_unrelated"), 0);

        let baseline_wait_entries = store.background.test_wait_entries();
        index_fields(&engine, "a-target", None);
        let (target_tx, target_rx) = mpsc::channel();
        let target_store = store.clone();
        let target_engine = engine.clone();
        let target = std::thread::spawn(move || {
            let _ = target_tx.send(target_store.save_required(&target_engine, 18));
        });
        store
            .background
            .wait_for_test_wait_entry_after(baseline_wait_entries, CASE_TIMEOUT)
            .expect("target checkpoint must reach the existing root-wide capacity wait");

        observer.release_first_encode();
        observer.wait_until(
            |state| state.before_publish >= 2 && state.after_publish >= 1,
            "first merge must publish before the follow-up merge pauses",
        );
        let held = observer.held_observations();
        assert!(
            held.0 >= 2,
            "independent eligible fields must reach the bounded parallel encode stage"
        );
        let after_first_merge = current_manifest(&store);

        let held_result = target_rx.recv_timeout(CASE_TIMEOUT);
        let completed_while_follow_up_merge_is_held = held_result.is_ok();
        let held_checkpoint = matches!(&held_result, Ok(Ok(_)))
            .then(|| (current_manifest(&store), store.load_latest()));
        observer.release_all();
        let target_result = match held_result {
            Ok(result) => result,
            Err(mpsc::RecvTimeoutError::Timeout) => target_rx
                .recv_timeout(Duration::from_secs(10))
                .expect("target checkpoint must finish after cleanup release"),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                panic!("target checkpoint thread stopped before returning a result")
            }
        };
        target
            .join()
            .expect("target checkpoint thread must not panic");
        target_result.expect("target checkpoint must publish after root work drains");
        store
            .wait_for_merges(Duration::from_secs(10))
            .expect("background merge worker must drain before fixture teardown");
        assert!(held.0 >= 2 && held.1 >= 2 && held.2 >= 1);
        // The blocking field must be drained by the merge the waiting
        // checkpoint resumed on: the unrelated field is never eligible, so it
        // stays untouched regardless of how many eligible fields one job
        // compacts.
        assert!(field_delta_count(&after_first_merge, "a_capacity") < 16);
        assert_eq!(field_delta_count(&after_first_merge, "z_unrelated"), 0);
        if let Some((manifest, cold_result)) = held_checkpoint {
            assert_eq!(manifest.checkpoint_sequence, 18);
            for field in ["a_capacity", "z_unrelated"] {
                assert!(field_delta_count(&manifest, field) <= 16);
            }
            let (cold, sequence) = cold_result.unwrap().unwrap();
            assert_eq!(sequence, 18);
            assert!(has_capacity_value(&cold, "a-target"));
        }
        let final_manifest = current_manifest(&store);
        assert_eq!(final_manifest.checkpoint_sequence, 18);
        let (cold, sequence) = store.load_latest().unwrap().unwrap();
        assert_eq!(sequence, 18);
        assert!(has_capacity_value(&cold, "a-target"));

        assert!(
            completed_while_follow_up_merge_is_held,
            "checkpoint did not resume after capacity-field merge while follow-up root merge remained queued"
        );
    }

    #[test]
    fn capacity_owner_merge_completes_after_capacity_progress_while_follow_up_merge_is_held() {
        if capacity_progress_case_runs_in_its_own_process(
            "segment_rdb::background::tests::capacity_owner_merge_completes_after_capacity_progress_while_follow_up_merge_is_held",
        ) {
            return;
        }
        let directory = tempfile::tempdir().unwrap();
        let observer = Arc::new(BlockingMergeObserver::default());
        let store = Arc::new(
            SegmentRdbStore::with_merge_observer(directory.path(), observer.clone()).unwrap(),
        );
        let engine = Arc::new(Engine::new());
        let _release_on_drop = ObserverRelease(observer.clone());
        engine.create_collection("u", capacity_schema()).unwrap();
        store.save_required(&engine, 1).unwrap();
        for sequence in 2..=17 {
            index_fields(&engine, &format!("a-{sequence}"), None);
            store.save_required(&engine, sequence).unwrap();
            if sequence == 5 {
                observer.wait_until(
                    |state| state.before_encode >= 1,
                    "first background merge must pause before encode",
                );
            }
        }

        let mut fallback = None;
        crate::segment_capacity::Fallback::ensure(&mut fallback, &engine, Some(store.clone()))
            .unwrap();
        let endpoint = engine.layer_maintenance.owner().unwrap();
        let baseline_wait_entries = store.background.test_wait_entries();
        let (merge_tx, merge_rx) = mpsc::channel();
        let merge_endpoint = endpoint.clone();
        let merge = std::thread::spawn(move || {
            let _ = merge_tx.send(merge_endpoint.wait_for(crate::segment_capacity::Work::Merge));
        });
        store
            .background
            .wait_for_test_wait_entry_after(baseline_wait_entries, CASE_TIMEOUT)
            .expect("capacity owner must enter its merge wait");

        observer.release_first_encode();
        observer.wait_until(
            |state| state.before_publish >= 2 && state.after_publish >= 1,
            "first merge must publish before the follow-up merge pauses",
        );
        let held_result = merge_rx.recv_timeout(CASE_TIMEOUT);
        let completed_while_follow_up_merge_is_held = held_result.is_ok();
        observer.release_all();
        let merge_result = match held_result {
            Ok(result) => result,
            Err(mpsc::RecvTimeoutError::Timeout) => merge_rx
                .recv_timeout(CASE_TIMEOUT)
                .expect("capacity owner merge must finish after cleanup release"),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                panic!("capacity owner merge thread stopped before returning a result")
            }
        };
        merge
            .join()
            .expect("capacity owner merge thread must not panic");
        merge_result.expect("capacity owner merge must not fail");
        store
            .wait_for_merges(Duration::from_secs(10))
            .expect("background merge worker must drain before fixture teardown");
        assert!(
            completed_while_follow_up_merge_is_held,
            "capacity owner waited for an unrelated follow-up root merge after capacity progress"
        );
        drop(fallback);
    }

    #[test]
    fn failed_owned_scratch_cleanup_retains_both_pins() {
        let directory = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(directory.path()).unwrap();
        let (_, scratch) = store.begin_next_generation(1).unwrap();
        let scratch_path = scratch.path().to_path_buf();
        let scratch_name = scratch_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        let source_name = "gen-0-rev-0";
        store.background.pin(scratch_name.clone());
        store.background.pin(source_name.to_owned());

        let error = store
            .cleanup_owned_merge_staging_with(
                &scratch_path,
                &scratch_name,
                source_name,
                scratch,
                |_| Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied)),
            )
            .expect_err("a failed owned scratch removal must remain an error");

        assert!(format!("{error:#}").contains("remove owned background merge staging"));
        assert!(
            store.background.protects(&scratch_name),
            "cleanup failure must retain the scratch pin"
        );
        assert!(
            store.background.protects(source_name),
            "cleanup failure must retain the source pin"
        );
    }

    #[test]
    fn missing_owned_scratch_cleanup_releases_both_pins() {
        let directory = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(directory.path()).unwrap();
        let (_, scratch) = store.begin_next_generation(1).unwrap();
        let scratch_path = scratch.path().to_path_buf();
        let scratch_name = scratch_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        let source_name = "gen-0-rev-0";
        store.background.pin(scratch_name.clone());
        store.background.pin(source_name.to_owned());

        std::fs::remove_dir_all(&scratch_path).unwrap();
        store
            .cleanup_owned_merge_staging(&scratch_path, &scratch_name, source_name, scratch)
            .expect("an already-absent owned scratch is equivalent to prior absence");

        assert!(
            !store.background.protects(&scratch_name),
            "successful cleanup must release the scratch pin"
        );
        assert!(
            !store.background.protects(source_name),
            "successful cleanup must release the source pin"
        );
    }
}

/// One merge job's accumulated per-[`MergeStep`] cost.
///
/// Steps are buffered rather than published as they finish because a merge
/// attempt may abandon at any of its refusal points (a frozen checkpoint, a
/// moved capture epoch, a rebase whose inputs no longer occur). Publishing a
/// half-finished attempt's phases would leave the `phase` rows with
/// different observation counts, and the breakdown would no longer describe
/// any single job. [`MergeStepCosts::publish`] therefore runs only on a job
/// that actually published, so every `phase` row shares one denominator.
#[derive(Default)]
struct MergeStepCosts {
    entries: [(Duration, u64); crate::metrics::MERGE_STEP_COUNT],
}

impl MergeStepCosts {
    fn record(&mut self, step: crate::metrics::MergeStep, elapsed: Duration, files: u64) {
        let entry = &mut self.entries[step.index()];
        entry.0 += elapsed;
        entry.1 += files;
    }

    fn publish(
        &self,
        metrics: &crate::metrics::Metrics,
        total: Duration,
        save_gate_held: Duration,
        fields: u64,
    ) {
        for step in crate::metrics::MergeStep::ALL {
            if step == crate::metrics::MergeStep::Total {
                continue;
            }
            let (elapsed, files) = self.entries[step.index()];
            metrics.observe_segment_merge_step(step, elapsed, files);
        }
        metrics.observe_segment_merge_step(crate::metrics::MergeStep::Total, total, 0);
        metrics.observe_segment_merge_save_gate(save_gate_held);
        metrics.incr_segment_merge_fields(fields);
    }
}

/// Hard-link one collection's directory from `source` into `destination`,
/// returning the number of files linked.
///
/// A merge job's scratch stage is job-private: it carries no generation
/// manifest, is never published, and is removed by
/// [`SegmentRdbStore::cleanup_owned_merge_staging`] when the job ends, so no
/// reader ever opens it as a generation. The compaction reads and writes only
/// the compacted collection's own directory, so that is all the stage needs —
/// and the count
/// `lumen_segment_merge_phase_files_total{phase="link_scratch"}` publishes is
/// proportional to the job's payload rather than to the whole root.
fn link_collection(source: &Path, destination: &Path, collection_id: &str) -> Result<usize> {
    let name = collection_checkpoint_dir_name(collection_id);
    if source.join(FLAT_PAYLOAD_DIR).is_dir() {
        // v3 flat names keep the collection ID as their leading byte string,
        // then hex-encode only the former collection-relative path. The old
        // directory name is hexadecimal and cannot select those files.
        let prefix = flat_payload_name(collection_id, Path::new(""));
        return link_flat_collection(
            &source.join(FLAT_PAYLOAD_DIR),
            &destination.join(FLAT_PAYLOAD_DIR),
            &prefix,
        );
    }
    link_tree(&source.join(&name), &destination.join(&name))
}

fn link_flat_collection(source: &Path, destination: &Path, prefix: &str) -> Result<usize> {
    std::fs::create_dir_all(destination)?;
    let mut count = 0;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !format!("payload/{name}").starts_with(prefix) {
            continue;
        }
        let metadata = std::fs::symlink_metadata(entry.path())?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            bail!("flat merge source contains a nonregular file");
        }
        std::fs::hard_link(entry.path(), destination.join(entry.file_name()))?;
        count += 1;
    }
    Ok(count)
}

/// Hard-link every collection of `manifest` from `source` into `destination`,
/// returning every linked path relative to the generation root.
///
/// This is the publication-side pass: the generation being staged inherits
/// every file of the one it replaces, so its cost is proportional to the whole
/// root and it is paid while the job holds the save gate. Collections are
/// disjoint subtrees, so they are linked on a bounded worker set
/// ([`map_generation_pass`]) and folded back in manifest order, which keeps
/// both the reported path set and the first reported error identical to a
/// sequential pass.
fn link_collections_with_paths(
    source: &Path,
    destination: &Path,
    manifest: &SegmentGenerationManifest,
) -> Result<BTreeSet<String>> {
    if uses_flat_generation_layout(manifest.schema_version) {
        let mut linked = BTreeSet::new();
        link_tree_with_paths(
            source.join(FLAT_PAYLOAD_DIR).as_path(),
            destination.join(FLAT_PAYLOAD_DIR).as_path(),
            Path::new(FLAT_PAYLOAD_DIR),
            &mut linked,
        )?;
        return Ok(linked);
    }
    let names: Vec<String> = manifest
        .collections
        .iter()
        .map(|collection| collection_checkpoint_dir_name(&collection.collection_id))
        .collect();
    let per_collection = map_generation_pass(&names, GENERATION_LINK_WORKERS, |name| {
        let mut linked = BTreeSet::new();
        link_tree_with_paths(
            &source.join(name),
            &destination.join(name),
            Path::new(name),
            &mut linked,
        )?;
        Ok(linked)
    });
    let mut linked = BTreeSet::new();
    for collection in per_collection {
        linked.extend(collection?);
    }
    Ok(linked)
}

#[inline]
fn uses_flat_generation_layout(schema_version: u32) -> bool {
    schema_version == GENERATION_MANIFEST_V3
}

fn background_merge_supports_manifest(schema_version: u32) -> bool {
    schema_version == GENERATION_MANIFEST_V2
}

fn link_tree_with_paths(
    source: &Path,
    destination: &Path,
    relative: &Path,
    linked: &mut BTreeSet<String>,
) -> Result<()> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let metadata = std::fs::symlink_metadata(entry.path())?;
        let target = destination.join(entry.file_name());
        let relative = relative.join(entry.file_name());
        if metadata.file_type().is_symlink() {
            bail!("merge source contains a symlink");
        }
        if metadata.is_dir() {
            link_tree_with_paths(&entry.path(), &target, &relative, linked)?;
        } else if metadata.is_file() {
            std::fs::hard_link(entry.path(), target)?;
            linked.insert(
                relative
                    .to_str()
                    .ok_or_else(|| anyhow!("merge source path is not UTF-8"))?
                    .to_owned(),
            );
        } else {
            bail!("merge source contains a nonregular file");
        }
    }
    Ok(())
}

/// Returns the number of files hard-linked, including nested directories.
fn link_tree(source: &Path, destination: &Path) -> Result<usize> {
    let mut linked = 0;
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let metadata = std::fs::symlink_metadata(entry.path())?;
        let target = destination.join(entry.file_name());
        if metadata.file_type().is_symlink() {
            bail!("merge source contains a symlink");
        }
        if metadata.is_dir() {
            linked += link_tree(&entry.path(), &target)?;
        } else if metadata.is_file() {
            std::fs::hard_link(entry.path(), target)?;
            linked += 1;
        } else {
            bail!("merge source contains a nonregular file");
        }
    }
    Ok(linked)
}
