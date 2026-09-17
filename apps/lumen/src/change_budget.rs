//! Owner-scoped accounting for durable Lumen changes.
//!
//! A caller reserves bytes before it starts an operation.  A reservation can
//! be cancelled by dropping it, or committed into active change memory.  An
//! active charge is intentionally not released when its original caller is
//! dropped: it remains charged until a checkpoint batch that captured it is
//! published.  Freezing creates such a batch.  Dropping a frozen batch handle
//! represents a failed checkpoint attempt and retains its charge.
//!
//! Admission must happen before a caller takes a future CaptureBarrier.  The
//! blocking admission method waits for publication or owner retirement, while
//! an input larger than the hard limit returns `Oversized` immediately so the
//! caller can choose a spill path.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock, Weak};
use std::time::{Duration, Instant};

pub const HARD_LIMIT: usize = 256 * 1024 * 1024;
pub const CHECKPOINT_TRIGGER: usize = 128 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionError {
    /// The requested object cannot ever fit and must use a spill path.
    Oversized { requested: usize, hard_limit: usize },
    /// The request fits alone but current reserved, active, and frozen work
    /// leaves insufficient room.  Callers may retry or use blocking admission.
    Full {
        requested: usize,
        used: usize,
        hard_limit: usize,
    },
    /// The owner was retired while an operation still held its handle.
    Retired,
    /// A frozen batch belongs to a different shared budget instance.
    WrongBudget,
    /// The batch belongs to another owner in this shared budget.
    WrongOwner,
    /// A reservation can release bytes but cannot use shrinking to grow.
    CannotShrink { current: usize, requested: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Snapshot {
    pub reserved: usize,
    pub active: usize,
    pub frozen: usize,
    pub total: usize,
    /// Monotonic revision of new checkpointable work.
    pub work_revision: u64,
    /// A blocked-admission request tied to work that was already active or
    /// frozen when the request was made.
    pub checkpoint_request_revision: Option<u64>,
}

impl Snapshot {
    pub fn checkpoint_needed(self) -> bool {
        self.total >= CHECKPOINT_TRIGGER
    }
}

#[derive(Default)]
struct OwnerState {
    reserved: usize,
    // Committed record payloads that have not entered an apply interval.
    // A checkpoint cannot capture these bytes, so freeze must leave them here.
    committed_ram: usize,
    active: usize,
    frozen: BTreeMap<u64, usize>,
    /// Payload ownership retained by a journal or reclaimer. The key is the
    /// capture epoch at which the payload became resident.
    resident: BTreeMap<u64, usize>,
    current_capture_epoch: u64,
    work_revision: u64,
    checkpoint_request_revision: Option<u64>,
    retired: bool,
}

#[derive(Default)]
struct State {
    next_owner: u64,
    next_batch: u64,
    next_work_revision: u64,
    next_checkpoint_request_revision: u64,
    checkpoint_request_revision: Option<u64>,
    high_water_bytes: usize,
    owners: BTreeMap<u64, OwnerState>,
}

fn request_revision(state: &mut State, work_revision: u64) -> u64 {
    if let Some(revision) = state.checkpoint_request_revision {
        return revision;
    }
    let revision = state
        .next_checkpoint_request_revision
        .checked_add(1)
        .expect("checkpoint request revision exhausted")
        .max(work_revision);
    state.next_checkpoint_request_revision = revision;
    state.checkpoint_request_revision = Some(revision);
    revision
}

struct Inner {
    hard_limit: usize,
    capacity_waiters: AtomicUsize,
    state: Mutex<State>,
    changed: Condvar,
    wake: Arc<BudgetWake>,
}

/// A checkpoint driver can wait for accounting changes without registering a
/// callback. No user code runs while the accounting mutex is held.
///
/// Mutators take the accounting mutex before this mutex. Waiters release this
/// mutex before they may query accounting, so the reverse order is forbidden.
pub struct BudgetWake {
    epoch: AtomicU64,
    lock: Mutex<()>,
    changed: Condvar,
}
impl BudgetWake {
    pub fn epoch(&self) -> u64 {
        self.epoch.load(Ordering::Acquire)
    }

    pub fn wait_for_change(&self, observed: u64) {
        let mut guard = self.lock.lock().expect("budget wake lock poisoned");
        while self.epoch() == observed {
            guard = self.changed.wait(guard).expect("budget wake lock poisoned");
        }
    }

    /// Returns true when a new epoch arrives before `timeout` expires.
    /// The wake mutex is dropped before this method returns.
    pub fn wait_for_change_timeout(&self, observed: u64, timeout: Duration) -> bool {
        let deadline = Instant::now().checked_add(timeout);
        let mut guard = self.lock.lock().expect("budget wake lock poisoned");
        while self.epoch() == observed {
            let remaining = deadline
                .and_then(|deadline| deadline.checked_duration_since(Instant::now()))
                .unwrap_or_default();
            if remaining.is_zero() {
                return false;
            }
            let (next, timeout) = self
                .changed
                .wait_timeout(guard, remaining)
                .expect("budget wake lock poisoned");
            guard = next;
            if timeout.timed_out() {
                return self.epoch() != observed;
            }
        }
        true
    }

    fn signal(&self) {
        // This shares the waiter predicate mutex, so notify cannot race a
        // predicate check immediately before Condvar::wait.
        let _guard = self.lock.lock().expect("budget wake lock poisoned");
        self.epoch.fetch_add(1, Ordering::Release);
        self.changed.notify_all();
    }
}

/// Shared process accounting.  Its owners are independent engines or restore
/// candidates; publishing one owner never clears another owner's work.
#[derive(Clone)]
pub struct ChangeBudget(Arc<Inner>);

/// A waiter owns this marker only until admission finishes or fails. It does
/// not hold an apply lease, and it does not invoke user code or callbacks.
struct CapacityWaiter(ChangeBudget);

impl Drop for CapacityWaiter {
    fn drop(&mut self) {
        let previous = self.0.0.capacity_waiters.fetch_sub(1, Ordering::AcqRel);
        assert!(previous > 0, "capacity waiter accounting lost");
        self.0.0.wake.signal();
    }
}

/// Pending checkpointable work for one Engine owner. Process-wide snapshots
/// cannot choose a checkpoint target because they aggregate other Engines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OwnerCapacityState {
    pub(crate) active: usize,
    pub(crate) frozen: usize,
    pub(crate) work_revision: u64,
    pub(crate) checkpoint_request_revision: Option<u64>,
}

/// One engine or restore candidate's accounting namespace. Retire only after
/// ordinary reservations and committed RAM have left this owner's live path.
/// Retained payload handles may outlive retirement and stay charged until
/// their actual final drop.
pub struct Owner {
    budget: ChangeBudget,
    id: u64,
    lifetime: Arc<OwnerLifetime>,
}

/// Every handle that can use an owner's namespace keeps this token alive.
/// Ending an Engine cannot remove reservations or payloads that escaped it.
/// The last handle removes only empty accounting; unpublished committed bytes
/// still require publication or explicit retirement, even without a handle.
struct OwnerLifetime {
    budget: ChangeBudget,
    id: u64,
}

impl Drop for OwnerLifetime {
    fn drop(&mut self) {
        let mut state = self
            .budget
            .0
            .state
            .lock()
            .expect("change budget lock poisoned");
        let empty = state.owners.get(&self.id).is_some_and(|owner| {
            owner.reserved == 0
                && owner.committed_ram == 0
                && owner.active == 0
                && owner.frozen.values().all(|&bytes| bytes == 0)
                && owner.resident.values().all(|&bytes| bytes == 0)
        });
        if empty {
            state.owners.remove(&self.id);
        }
    }
}

/// Bytes held before a change is committed.  Dropping releases only this
/// reservation.
pub struct Reservation {
    budget: ChangeBudget,
    owner: u64,
    lifetime: Arc<OwnerLifetime>,
    bytes: usize,
    committed: bool,
    source_retention: Option<Arc<Mutex<Option<SourceRetentionHeld>>>>,
}

enum SourceRetentionHeld {
    Reservation(Reservation),
    Retained(RetainedCharge),
}

/// Keeps one source-backed reservation charged while a failed worker still
/// owns the source but cannot safely resume application.
#[derive(Clone)]
pub(crate) struct SourceRetention(Arc<Mutex<Option<SourceRetentionHeld>>>);

/// A committed charge.  Dropping it deliberately does not release memory.
pub struct ActiveCharge {
    budget: ChangeBudget,
    owner: u64,
    _lifetime: Arc<OwnerLifetime>,
    bytes: usize,
}

/// Pre-apply RAM ownership. Unlike [`ActiveCharge`], this may be released only
/// by the private durable-stage proof emitted after its payload is destroyed.
pub(crate) struct RamCharge {
    budget: ChangeBudget,
    owner: u64,
    _lifetime: Arc<OwnerLifetime>,
    bytes: usize,
    sequence: u64,
    engine_epoch: u64,
    source: crate::committed_stage::SourceIdentity,
}

/// Shared ownership of payload bytes retained outside the checkpoint batch.
/// Cloning this handle shares one accounting entry; only the last drop frees
/// its bytes.
#[derive(Clone)]
pub(crate) struct RetainedCharge(Arc<RetainedChargeInner>);

struct RetainedChargeInner {
    budget: ChangeBudget,
    owner: u64,
    _lifetime: Arc<OwnerLifetime>,
    capture_epoch: u64,
    bytes: usize,
}

/// A frozen checkpoint capture.  Dropping it deliberately retains memory.
pub struct FrozenBatch {
    budget: ChangeBudget,
    owner: u64,
    _lifetime: Arc<OwnerLifetime>,
    id: u64,
}

impl ChangeBudget {
    pub fn new() -> Self {
        Self::new_inner(HARD_LIMIT)
    }

    fn new_inner(hard_limit: usize) -> Self {
        assert!(hard_limit > 0);
        Self(Arc::new(Inner {
            hard_limit,
            capacity_waiters: AtomicUsize::new(0),
            state: Mutex::new(State::default()),
            changed: Condvar::new(),
            wake: Arc::new(BudgetWake {
                epoch: AtomicU64::new(0),
                lock: Mutex::new(()),
                changed: Condvar::new(),
            }),
        }))
    }

    /// Process default. Every live caller gets the same 256 MiB accounting
    /// instance; tests use `with_hard_limit` instead.
    pub fn process_shared() -> Self {
        static REGISTRY: OnceLock<Mutex<Weak<Inner>>> = OnceLock::new();
        let registry = REGISTRY.get_or_init(|| Mutex::new(Weak::new()));
        let mut weak = registry.lock().expect("change budget registry poisoned");
        if let Some(inner) = weak.upgrade() {
            return Self(inner);
        }
        let budget = Self::new();
        *weak = Arc::downgrade(&budget.0);
        budget
    }

    pub fn checkpoint_wake(&self) -> Arc<BudgetWake> {
        self.0.wake.clone()
    }

    #[cfg(test)]
    pub(crate) fn with_hard_limit(hard_limit: usize) -> Self {
        Self::new_inner(hard_limit)
    }

    pub fn owner(&self) -> Owner {
        let mut state = self.0.state.lock().expect("change budget lock poisoned");
        state.next_owner = state.next_owner.checked_add(1).expect("owner id exhausted");
        let id = state.next_owner;
        state.owners.insert(id, OwnerState::default());
        Owner {
            budget: self.clone(),
            id,
            lifetime: Arc::new(OwnerLifetime {
                budget: self.clone(),
                id,
            }),
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        self.snapshot_with_high_water().0
    }

    /// Read the live pending totals and their lifetime peak under one budget
    /// mutex acquisition. Prometheus rendering needs this pair to avoid
    /// combining totals from different concurrent accounting states.
    pub(crate) fn snapshot_with_high_water(&self) -> (Snapshot, usize) {
        let state = self.0.state.lock().expect("change budget lock poisoned");
        (snapshot(&state), state.high_water_bytes)
    }

    /// Largest successful pending-change total since this budget was created.
    /// Releases and rejected reservations never lower or raise this value.
    pub fn high_water_bytes(&self) -> usize {
        self.snapshot_with_high_water().1
    }

    /// True only while an admitted apply/replay operation waits for capacity.
    /// A coordinator uses this to park future WAL payloads outside the apply
    /// barrier. A pre-publication refusal does not request such work.
    pub(crate) fn has_capacity_waiters(&self) -> bool {
        self.0.capacity_waiters.load(Ordering::Acquire) != 0
    }

    fn mark_capacity_waiter(&self) -> CapacityWaiter {
        self.0
            .capacity_waiters
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                count.checked_add(1)
            })
            .expect("capacity waiter count overflow");
        self.0.wake.signal();
        CapacityWaiter(self.clone())
    }

    /// Ask the checkpoint driver to make one early attempt for current
    /// checkpointable work. This never makes a reservation or pre-apply RAM
    /// eligible by itself, and it does not advance the work revision.
    pub fn request_checkpoint(&self) -> bool {
        let mut state = self.0.state.lock().expect("change budget lock poisoned");
        let pending = snapshot(&state);
        if pending.active == 0 && pending.frozen == 0 {
            return false;
        }
        let work_revision = state.next_work_revision;
        request_revision(&mut state, work_revision);
        self.0.wake.signal();
        true
    }

    fn reserve(
        &self,
        lifetime: &Arc<OwnerLifetime>,
        bytes: usize,
        wait: bool,
    ) -> Result<Reservation, AdmissionError> {
        let owner = lifetime.id;
        if bytes > self.0.hard_limit {
            return Err(AdmissionError::Oversized {
                requested: bytes,
                hard_limit: self.0.hard_limit,
            });
        }
        // Declared before the state guard so marker drop never holds that
        // mutex. A spurious wake reuses this marker instead of counting twice.
        let mut waiting = None;
        let mut state = self.0.state.lock().expect("change budget lock poisoned");
        loop {
            let used = snapshot(&state).total;
            let owner_state = state.owners.get(&owner).ok_or(AdmissionError::Retired)?;
            if owner_state.retired {
                return Err(AdmissionError::Retired);
            }
            if bytes <= self.0.hard_limit - used {
                state
                    .owners
                    .get_mut(&owner)
                    .expect("owner checked above")
                    .reserved = state
                    .owners
                    .get(&owner)
                    .expect("owner checked above")
                    .reserved
                    .checked_add(bytes)
                    .expect("reservation accounting overflow");
                update_high_water(&mut state);
                self.0.wake.signal();
                return Ok(Reservation {
                    budget: self.clone(),
                    owner,
                    lifetime: lifetime.clone(),
                    bytes,
                    committed: false,
                    source_retention: None,
                });
            }
            if !wait {
                return Err(AdmissionError::Full {
                    requested: bytes,
                    used,
                    hard_limit: self.0.hard_limit,
                });
            }
            waiting.get_or_insert_with(|| self.mark_capacity_waiter());
            state = self
                .0
                .changed
                .wait(state)
                .expect("change budget lock poisoned");
        }
    }

    /// Publish every frozen batch for this owner through `captured`, inclusive.
    /// Newer frozen batches, active work, and reservations stay charged.
    pub fn publish_through(&self, captured: &FrozenBatch) -> Result<usize, AdmissionError> {
        if !Arc::ptr_eq(&self.0, &captured.budget.0) {
            return Err(AdmissionError::WrongBudget);
        }
        let mut state = self.0.state.lock().expect("change budget lock poisoned");
        let owner = state
            .owners
            .get_mut(&captured.owner)
            .ok_or(AdmissionError::Retired)?;
        let ids: Vec<u64> = owner
            .frozen
            .range(..=captured.id)
            .map(|(&id, _)| id)
            .collect();
        let released = ids
            .into_iter()
            .map(|id| owner.frozen.remove(&id).unwrap())
            .sum();
        self.0.changed.notify_all();
        self.0.wake.signal();
        Ok(released)
    }
}

impl Default for ChangeBudget {
    fn default() -> Self {
        Self::new()
    }
}

impl Owner {
    pub(crate) fn request_checkpoint(&self) -> bool {
        let mut state = self
            .budget
            .0
            .state
            .lock()
            .expect("change budget lock poisoned");
        let Some(owner) = state.owners.get(&self.id) else {
            return false;
        };
        if owner.active == 0 && owner.frozen.is_empty() {
            return false;
        }
        let work_revision = owner.work_revision;
        let revision = request_revision(&mut state, work_revision);
        state
            .owners
            .get_mut(&self.id)
            .expect("owner checked above")
            .checkpoint_request_revision = Some(revision);
        self.budget.0.wake.signal();
        true
    }

    pub(crate) fn consume_checkpoint_request(&self, revision: Option<u64>) {
        let Some(revision) = revision else {
            return;
        };
        let mut state = self
            .budget
            .0
            .state
            .lock()
            .expect("change budget lock poisoned");
        if let Some(owner) = state.owners.get_mut(&self.id) {
            if owner.checkpoint_request_revision == Some(revision) {
                owner.checkpoint_request_revision = None;
            }
        }
        if state.checkpoint_request_revision == Some(revision) {
            state.checkpoint_request_revision = None;
        }
    }

    pub fn try_reserve(&self, bytes: usize) -> Result<Reservation, AdmissionError> {
        self.budget.reserve(&self.lifetime, bytes, false)
    }

    /// Wait for admission but keep the bytes reserved until the caller crosses
    /// its apply boundary and explicitly commits this reservation.
    pub fn wait_reserve(&self, bytes: usize) -> Result<Reservation, AdmissionError> {
        self.budget.reserve(&self.lifetime, bytes, true)
    }

    pub(crate) fn work_revision(&self) -> Result<u64, AdmissionError> {
        let state = self
            .budget
            .0
            .state
            .lock()
            .expect("change budget lock poisoned");
        let owner = state.owners.get(&self.id).ok_or(AdmissionError::Retired)?;
        if owner.retired {
            return Err(AdmissionError::Retired);
        }
        Ok(owner.work_revision)
    }

    /// Read only this owner's publishable work. `ChangeBudget::snapshot` is
    /// process-global and therefore cannot safely select an Engine to save.
    pub(crate) fn capacity_state(&self) -> Result<OwnerCapacityState, AdmissionError> {
        let state = self
            .budget
            .0
            .state
            .lock()
            .expect("change budget lock poisoned");
        let owner = state.owners.get(&self.id).ok_or(AdmissionError::Retired)?;
        if owner.retired {
            return Err(AdmissionError::Retired);
        }
        let resident_active = owner
            .resident
            .get(&owner.current_capture_epoch)
            .copied()
            .unwrap_or_default();
        let resident_frozen = owner
            .resident
            .iter()
            .filter(|(epoch, _)| **epoch != owner.current_capture_epoch)
            .try_fold(0usize, |total, (_, bytes)| total.checked_add(*bytes))
            .expect("resident frozen overflow");
        let frozen = owner
            .frozen
            .values()
            .copied()
            .try_fold(resident_frozen, |total, bytes| total.checked_add(bytes))
            .expect("frozen overflow");
        Ok(OwnerCapacityState {
            active: owner
                .active
                .checked_add(resident_active)
                .expect("active overflow"),
            frozen,
            work_revision: owner.work_revision,
            checkpoint_request_revision: owner.checkpoint_request_revision,
        })
    }

    /// Move all active work into one durable checkpoint batch.
    pub fn freeze(&self) -> Result<FrozenBatch, AdmissionError> {
        let mut state = self
            .budget
            .0
            .state
            .lock()
            .expect("change budget lock poisoned");
        let active = {
            let owner = state
                .owners
                .get_mut(&self.id)
                .ok_or(AdmissionError::Retired)?;
            if owner.retired {
                return Err(AdmissionError::Retired);
            }
            let active = std::mem::take(&mut owner.active);
            // Retained payload handles carry their old epoch. Advancing this
            // scalar moves them into the frozen view without visiting them.
            owner.current_capture_epoch = owner
                .current_capture_epoch
                .checked_add(1)
                .expect("capture epoch exhausted");
            active
        };
        state.next_batch = state.next_batch.checked_add(1).expect("batch id exhausted");
        let id = state.next_batch;
        state
            .owners
            .get_mut(&self.id)
            .expect("owner checked above")
            .frozen
            .insert(id, active);
        self.budget.0.wake.signal();
        Ok(FrozenBatch {
            budget: self.budget.clone(),
            owner: self.id,
            _lifetime: self.lifetime.clone(),
            id,
        })
    }

    /// Release only this engine's accounting.  Restore candidates and other
    /// live engines remain charged.
    pub fn retire(&self) {
        let mut state = self
            .budget
            .0
            .state
            .lock()
            .expect("change budget lock poisoned");
        if let Some(owner) = state.owners.get_mut(&self.id) {
            owner.reserved = 0;
            owner.committed_ram = 0;
            owner.active = 0;
            owner.frozen.clear();
            // `resident` belongs to retained payload handles. Those handles
            // can outlive this owner, so retirement must leave their bytes
            // visible until the final handle drops.
            owner.retired = true;
            self.budget.0.changed.notify_all();
            self.budget.0.wake.signal();
        }
    }

    /// Publish only this owner's frozen batch.
    pub fn publish_through(&self, batch: &FrozenBatch) -> Result<usize, AdmissionError> {
        if self.id != batch.owner {
            return Err(AdmissionError::WrongOwner);
        }
        self.budget.publish_through(batch)
    }
}

impl Reservation {
    /// Divide one already-admitted reservation without changing process
    /// accounting. The original half keeps its source-retention bridge; the
    /// new half is ordinary transient ownership. This is deliberately a pure
    /// handle split: it must not wake capacity waiters or open a second
    /// admission race.
    pub(crate) fn split_off(&mut self, bytes: usize) -> Reservation {
        assert!(
            bytes <= self.bytes,
            "cannot split {bytes} bytes from {} reserved bytes",
            self.bytes
        );
        self.bytes = self
            .bytes
            .checked_sub(bytes)
            .expect("split bytes were checked against reservation");
        Reservation {
            budget: self.budget.clone(),
            owner: self.owner,
            lifetime: self.lifetime.clone(),
            bytes,
            committed: false,
            // The source must stay with the apply half. The transient half
            // protects only temporary encode/AOF ownership.
            source_retention: None,
        }
    }

    pub(crate) fn source_retention(&mut self) -> SourceRetention {
        let retained = self
            .source_retention
            .get_or_insert_with(|| Arc::new(Mutex::new(None)));
        SourceRetention(retained.clone())
    }

    pub(crate) fn bytes(&self) -> usize {
        self.bytes
    }

    /// Release unused bytes before this reservation becomes a charge.
    ///
    /// This only changes the mutable reservation.  Charges created by a
    /// commit retain their existing lifetime and accounting.
    pub(crate) fn shrink_to(&mut self, required: usize) -> Result<(), AdmissionError> {
        if required > self.bytes {
            return Err(AdmissionError::CannotShrink {
                current: self.bytes,
                requested: required,
            });
        }
        if required == self.bytes {
            return Ok(());
        }

        let released = self
            .bytes
            .checked_sub(required)
            .expect("required is smaller than the reservation");
        let mut state = self.budget.0.state.lock().expect("change budget poisoned");
        let owner = state
            .owners
            .get_mut(&self.owner)
            .ok_or(AdmissionError::Retired)?;
        if owner.retired {
            return Err(AdmissionError::Retired);
        }
        owner.reserved = owner
            .reserved
            .checked_sub(released)
            .expect("reservation accounting lost");
        self.bytes = required;
        drop(state);
        self.budget.0.changed.notify_all();
        self.budget.0.wake.signal();
        Ok(())
    }

    /// Extend this live reservation to the requested total; it never shrinks.
    /// Call before an apply or CaptureBarrier lease. The waiting form may block
    /// until another owner publishes or retires.
    pub(crate) fn try_grow_to(&mut self, required_total: usize) -> Result<(), AdmissionError> {
        self.grow_to(required_total, false)
    }

    /// Like try_grow_to, but waits only for the positive delta.
    /// Call before an apply or CaptureBarrier lease.
    pub(crate) fn wait_grow_to(&mut self, required_total: usize) -> Result<(), AdmissionError> {
        self.grow_to(required_total, true)
    }

    fn grow_to(&mut self, required_total: usize, wait: bool) -> Result<(), AdmissionError> {
        if required_total <= self.bytes {
            return Ok(());
        }
        if required_total > self.budget.0.hard_limit {
            return Err(AdmissionError::Oversized {
                requested: required_total,
                hard_limit: self.budget.0.hard_limit,
            });
        }
        let delta = required_total
            .checked_sub(self.bytes)
            .expect("required total checked above");
        let mut waiting = None;
        let mut state = self
            .budget
            .0
            .state
            .lock()
            .expect("change budget lock poisoned");
        loop {
            let used = snapshot(&state).total;
            let owner = state
                .owners
                .get(&self.owner)
                .ok_or(AdmissionError::Retired)?;
            if owner.retired {
                return Err(AdmissionError::Retired);
            }
            if delta <= self.budget.0.hard_limit - used {
                let owner = state
                    .owners
                    .get_mut(&self.owner)
                    .expect("owner checked above");
                owner.reserved = owner
                    .reserved
                    .checked_add(delta)
                    .expect("reservation accounting overflow");
                self.bytes = required_total;
                update_high_water(&mut state);
                self.budget.0.wake.signal();
                return Ok(());
            }
            if !wait {
                return Err(AdmissionError::Full {
                    requested: required_total,
                    used,
                    hard_limit: self.budget.0.hard_limit,
                });
            }
            waiting.get_or_insert_with(|| self.budget.mark_capacity_waiter());
            state = self
                .budget
                .0
                .changed
                .wait(state)
                .expect("change budget lock poisoned");
        }
    }

    pub fn commit(mut self) -> Result<ActiveCharge, AdmissionError> {
        let mut state = self
            .budget
            .0
            .state
            .lock()
            .expect("change budget lock poisoned");
        let owner = state
            .owners
            .get_mut(&self.owner)
            .ok_or(AdmissionError::Retired)?;
        if owner.retired {
            return Err(AdmissionError::Retired);
        }
        owner.reserved = owner
            .reserved
            .checked_sub(self.bytes)
            .expect("reservation accounting lost");
        owner.active = owner
            .active
            .checked_add(self.bytes)
            .expect("active accounting overflow");
        state.next_work_revision = state
            .next_work_revision
            .checked_add(1)
            .expect("work revision exhausted");
        let revision = state.next_work_revision;
        state
            .owners
            .get_mut(&self.owner)
            .expect("owner checked above")
            .work_revision = revision;
        self.budget.0.wake.signal();
        self.committed = true;
        Ok(ActiveCharge {
            budget: self.budget.clone(),
            owner: self.owner,
            _lifetime: self.lifetime.clone(),
            bytes: self.bytes,
        })
    }

    /// Commit journal or reclaimer payload bytes that stay resident until the
    /// actual shared payload owner releases its final handle.
    pub(crate) fn commit_retained(mut self) -> Result<RetainedCharge, AdmissionError> {
        let mut state = self
            .budget
            .0
            .state
            .lock()
            .expect("change budget lock poisoned");
        let capture_epoch = {
            let owner = state
                .owners
                .get_mut(&self.owner)
                .ok_or(AdmissionError::Retired)?;
            if owner.retired {
                return Err(AdmissionError::Retired);
            }
            owner.reserved = owner
                .reserved
                .checked_sub(self.bytes)
                .expect("reservation accounting lost");
            let capture_epoch = owner.current_capture_epoch;
            if self.bytes != 0 {
                let resident = owner.resident.entry(capture_epoch).or_default();
                *resident = resident
                    .checked_add(self.bytes)
                    .expect("resident accounting overflow");
            }
            capture_epoch
        };
        state.next_work_revision = state
            .next_work_revision
            .checked_add(1)
            .expect("work revision exhausted");
        let revision = state.next_work_revision;
        state
            .owners
            .get_mut(&self.owner)
            .expect("owner checked above")
            .work_revision = revision;
        self.budget.0.wake.signal();
        self.committed = true;
        let charge = RetainedCharge(Arc::new(RetainedChargeInner {
            budget: self.budget.clone(),
            owner: self.owner,
            _lifetime: self.lifetime.clone(),
            capture_epoch,
            bytes: self.bytes,
        }));
        if let Some(retention) = self.source_retention.take() {
            *retention.lock().expect("source retention poisoned") =
                Some(SourceRetentionHeld::Retained(charge.clone()));
        }
        Ok(charge)
    }

    /// Commit pre-apply RAM that is eligible only for a verified durable-stage
    /// transition. Applied journal work must use [`Self::commit`] instead.
    pub(crate) fn commit_ram(
        mut self,
        sequence: u64,
        engine_epoch: u64,
        source: crate::committed_stage::SourceIdentity,
    ) -> Result<RamCharge, AdmissionError> {
        let mut state = self
            .budget
            .0
            .state
            .lock()
            .expect("change budget lock poisoned");
        let owner = state
            .owners
            .get_mut(&self.owner)
            .ok_or(AdmissionError::Retired)?;
        if owner.retired {
            return Err(AdmissionError::Retired);
        }
        owner.reserved = owner
            .reserved
            .checked_sub(self.bytes)
            .expect("reservation accounting lost");
        owner.committed_ram = owner
            .committed_ram
            .checked_add(self.bytes)
            .expect("committed RAM accounting overflow");
        self.budget.0.wake.signal();
        self.committed = true;
        Ok(RamCharge {
            budget: self.budget.clone(),
            owner: self.owner,
            _lifetime: self.lifetime.clone(),
            bytes: self.bytes,
            sequence,
            engine_epoch,
            source,
        })
    }
}

impl Drop for Reservation {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        if let Some(retention) = self.source_retention.take() {
            let held = Reservation {
                budget: self.budget.clone(),
                owner: self.owner,
                lifetime: self.lifetime.clone(),
                bytes: self.bytes,
                committed: false,
                source_retention: None,
            };
            *retention.lock().expect("source retention poisoned") =
                Some(SourceRetentionHeld::Reservation(held));
            self.committed = true;
            return;
        }
        let mut state = self
            .budget
            .0
            .state
            .lock()
            .expect("change budget lock poisoned");
        if let Some(owner) = state.owners.get_mut(&self.owner) {
            if !owner.retired {
                owner.reserved = owner
                    .reserved
                    .checked_sub(self.bytes)
                    .expect("reservation accounting lost");
            }
            self.budget.0.changed.notify_all();
            self.budget.0.wake.signal();
        }
    }
}

impl ActiveCharge {
    pub fn bytes(&self) -> usize {
        self.bytes
    }
    pub fn owner_id(&self) -> u64 {
        self.owner
    }
}

impl Drop for ActiveCharge {
    fn drop(&mut self) {
        // Deliberate: a committed mutation outlives its request handle.
        let _ = &self.budget;
    }
}

impl RamCharge {
    pub(crate) fn bytes(&self) -> usize {
        self.bytes
    }

    /// Consume this RAM charge only after the unforgeable marker made by the
    /// carrier confirms the same source, epoch, and sequence reached durable
    /// staging after the payload was dropped.
    pub(crate) fn release_after_stage(
        self,
        proof: crate::change_admission::RamReleased,
    ) -> Result<(), (AdmissionError, RamCharge)> {
        if !proof.matches(self.sequence, self.engine_epoch, &self.source) {
            return Err((AdmissionError::WrongOwner, self));
        }
        let mut state = self
            .budget
            .0
            .state
            .lock()
            .expect("change budget lock poisoned");
        let retired = match state.owners.get_mut(&self.owner) {
            Some(owner) if !owner.retired => false,
            _ => true,
        };
        if retired {
            drop(state);
            return Err((AdmissionError::Retired, self));
        }
        let owner = state
            .owners
            .get_mut(&self.owner)
            .expect("live RAM charge owner checked");
        owner.committed_ram = owner
            .committed_ram
            .checked_sub(self.bytes)
            .expect("RAM charge accounting lost");
        self.budget.0.changed.notify_all();
        self.budget.0.wake.signal();
        Ok(())
    }
}

impl Drop for RamCharge {
    fn drop(&mut self) {
        let _ = &self.budget;
    }
}

impl RetainedCharge {
    pub(crate) fn bytes(&self) -> usize {
        self.0.bytes
    }
}

impl Drop for RetainedChargeInner {
    fn drop(&mut self) {
        let mut state = self
            .budget
            .0
            .state
            .lock()
            .expect("change budget lock poisoned");
        if let Some(owner) = state.owners.get_mut(&self.owner) {
            if self.bytes != 0 {
                let remove_epoch = {
                    let resident = owner
                        .resident
                        .get_mut(&self.capture_epoch)
                        .expect("retained charge accounting lost");
                    *resident = resident
                        .checked_sub(self.bytes)
                        .expect("retained charge accounting lost");
                    *resident == 0
                };
                if remove_epoch {
                    owner.resident.remove(&self.capture_epoch);
                }
            }
            self.budget.0.changed.notify_all();
            self.budget.0.wake.signal();
        }
    }
}

impl FrozenBatch {
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn publish(self) -> Result<usize, AdmissionError> {
        self.budget.publish_through(&self)
    }
}

fn snapshot(state: &State) -> Snapshot {
    let mut out = Snapshot {
        reserved: 0,
        active: 0,
        frozen: 0,
        total: 0,
        work_revision: state.next_work_revision,
        checkpoint_request_revision: state.checkpoint_request_revision,
    };
    for owner in state.owners.values() {
        // Retained payloads remain process-owned after their Engine owner is
        // retired. The current epoch is active; earlier epochs are frozen.
        let resident_active = owner
            .resident
            .get(&owner.current_capture_epoch)
            .copied()
            .unwrap_or_default();
        let resident_frozen = owner
            .resident
            .iter()
            .filter(|(epoch, _)| **epoch != owner.current_capture_epoch)
            .try_fold(0usize, |total, (_, bytes)| total.checked_add(*bytes))
            .expect("resident frozen overflow");
        out.active = out
            .active
            .checked_add(resident_active)
            .expect("active overflow");
        out.frozen = out
            .frozen
            .checked_add(resident_frozen)
            .expect("frozen overflow");
        if owner.retired {
            continue;
        }
        out.reserved = out
            .reserved
            .checked_add(owner.reserved)
            .and_then(|bytes| bytes.checked_add(owner.committed_ram))
            .expect("reserved overflow");
        out.active = out
            .active
            .checked_add(owner.active)
            .expect("active overflow");
        let frozen: usize = owner.frozen.values().copied().sum();
        out.frozen = out.frozen.checked_add(frozen).expect("frozen overflow");
    }
    out.total = out
        .reserved
        .checked_add(out.active)
        .and_then(|v| v.checked_add(out.frozen))
        .expect("total overflow");
    out
}

fn update_high_water(state: &mut State) {
    let total = snapshot(state).total;
    state.high_water_bytes = state.high_water_bytes.max(total);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    fn owner_count(budget: &ChangeBudget) -> usize {
        budget.0.state.lock().unwrap().owners.len()
    }

    #[test]
    fn source_retention_guard_keeps_a_panicking_reservation_charged() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let mut reservation = owner.try_reserve(7).unwrap();
        let guard = reservation.source_retention();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _reservation = reservation;
            panic!("injected apply-worker panic");
        }));
        assert!(result.is_err());
        assert_eq!(budget.snapshot().reserved, 7);
        drop(guard);
        assert_eq!(budget.snapshot().total, 0);
    }

    #[test]
    fn source_retention_guard_releases_the_final_reserved_bytes_on_drop() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let mut reservation = owner.try_reserve(7).unwrap();
        let guard = reservation.source_retention();
        drop(reservation);
        assert_eq!(budget.snapshot().reserved, 7);
        drop(guard);
        assert_eq!(budget.snapshot().total, 0);
    }

    #[test]
    fn source_retention_guard_bridges_commit_before_journal_adoption() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let mut reservation = owner.try_reserve(7).unwrap();
        let guard = reservation.source_retention();
        let charge = reservation.commit_retained().unwrap();
        drop(charge);
        assert_eq!(budget.snapshot().total, 7);
        drop(guard);
        assert_eq!(budget.snapshot().total, 0);
    }

    #[test]
    fn source_retention_commit_after_guard_drop_does_not_deadlock() {
        let budget = ChangeBudget::with_hard_limit(16);
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let owner = budget.owner();
            let mut reservation = owner.try_reserve(7).unwrap();
            let guard = reservation.source_retention();
            drop(guard);
            let charge = reservation.commit_retained().unwrap();
            drop(charge);
            done_tx.send(()).unwrap();
        });
        assert!(
            done_rx
                .recv_timeout(std::time::Duration::from_secs(2))
                .is_ok(),
            "commit_retained must not drop a retained charge while holding the budget lock"
        );
    }

    #[test]
    fn source_retention_guard_uses_the_reservation_final_size() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let mut reservation = owner.try_reserve(4).unwrap();
        let guard = reservation.source_retention();
        reservation.try_grow_to(9).unwrap();
        reservation.shrink_to(6).unwrap();
        drop(reservation);
        assert_eq!(budget.snapshot().reserved, 6);
        drop(guard);
        assert_eq!(budget.snapshot().total, 0);
    }

    #[test]
    fn split_off_keeps_owner_total_and_source_retention_on_apply_half() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let mut apply = owner.try_reserve(10).unwrap();
        let guard = apply.source_retention();
        let wake = budget.checkpoint_wake();
        let epoch = wake.epoch();
        let transient = apply.split_off(4);

        assert_eq!(apply.bytes(), 6);
        assert_eq!(transient.bytes(), 4);
        assert_eq!(budget.snapshot().reserved, 10);
        assert_eq!(
            wake.epoch(),
            epoch,
            "a handle-only split must not wake capacity waiters"
        );

        // The transient half has no source bridge and releases independently.
        drop(transient);
        assert_eq!(budget.snapshot().reserved, 6);
        drop(apply);
        assert_eq!(
            budget.snapshot().reserved,
            6,
            "source retention must stay on the apply half"
        );
        drop(guard);
        assert_eq!(budget.snapshot().total, 0);
    }

    #[test]
    fn reservation_without_source_retention_still_releases_on_drop() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let reservation = owner.try_reserve(7).unwrap();
        drop(reservation);
        assert_eq!(budget.snapshot().total, 0);
    }

    #[test]
    fn blocking_admission_announces_pressure_only_until_capacity_is_acquired() {
        let budget = ChangeBudget::with_hard_limit(8);
        let owner = Arc::new(budget.owner());
        let occupied = owner.try_reserve(8).unwrap();
        assert!(owner.try_reserve(1).is_err());
        assert!(!budget.has_capacity_waiters());
        let worker = std::thread::spawn({
            let owner = owner.clone();
            move || owner.wait_reserve(1).unwrap()
        });
        let deadline = Instant::now() + Duration::from_millis(200);
        while !budget.has_capacity_waiters() && Instant::now() < deadline {
            std::thread::yield_now();
        }
        let reported = budget.has_capacity_waiters();
        // Always unblock before asserting, including against a broken signal.
        drop(occupied);
        let admitted = worker.join().unwrap();
        assert!(!budget.has_capacity_waiters());
        drop(admitted);
        assert!(
            reported,
            "blocked admission must announce capacity pressure"
        );
    }

    #[test]
    fn blocking_growth_announces_pressure_without_an_apply_lease() {
        let budget = ChangeBudget::with_hard_limit(8);
        let owner = budget.owner();
        let mut growing = owner.try_reserve(1).unwrap();
        let occupied = owner.try_reserve(7).unwrap();
        let worker = std::thread::spawn(move || {
            growing.wait_grow_to(8).unwrap();
            growing
        });
        let deadline = Instant::now() + Duration::from_millis(200);
        while !budget.has_capacity_waiters() && Instant::now() < deadline {
            std::thread::yield_now();
        }
        let reported = budget.has_capacity_waiters();
        drop(occupied);
        let admitted = worker.join().unwrap();
        assert_eq!(admitted.bytes(), 8);
        assert!(!budget.has_capacity_waiters());
        drop(admitted);
        assert!(
            reported,
            "blocked repricing must announce capacity pressure"
        );
    }

    #[test]
    fn ended_owner_namespaces_do_not_accumulate_in_a_live_process() {
        let budget = ChangeBudget::with_hard_limit(16);
        let _live = budget.owner();
        for _ in 0..4096 {
            let ended = budget.owner();
            let retained = ended.try_reserve(7).unwrap().commit_retained().unwrap();
            drop(ended);
            assert_eq!(budget.snapshot().total, 7);
            drop(retained);
        }
        assert_eq!(budget.snapshot().total, 0);
        assert_eq!(owner_count(&budget), 1);
    }

    #[test]
    fn escaped_reservations_keep_the_namespace_until_their_last_handle_drops() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let reserved = owner.try_reserve(7).unwrap();
        let empty = owner.try_reserve(0).unwrap();
        drop(owner);
        assert_eq!(budget.snapshot().reserved, 7);
        let retained = reserved.commit_retained().unwrap();
        assert_eq!(budget.snapshot().active, 7);
        drop(retained);
        assert_eq!(budget.snapshot().total, 0);
        assert_eq!(owner_count(&budget), 1);
        let retained_empty = empty.commit_retained().unwrap();
        drop(retained_empty);
        assert_eq!(owner_count(&budget), 0);
    }

    #[test]
    fn escaped_frozen_batch_can_publish_after_owner_drop_and_then_be_reclaimed() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let active = owner.try_reserve(7).unwrap().commit().unwrap();
        let batch = owner.freeze().unwrap();
        drop(owner);
        drop(active);
        assert_eq!(budget.snapshot().frozen, 7);
        assert_eq!(batch.publish().unwrap(), 7);
        assert_eq!(budget.snapshot().total, 0);
        assert_eq!(owner_count(&budget), 0);
    }

    #[test]
    fn dropping_all_handles_cannot_forgive_unpublished_committed_bytes() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let charge = owner.try_reserve(7).unwrap().commit().unwrap();
        drop(owner);
        drop(charge);
        assert_eq!(budget.snapshot().active, 7);
        assert_eq!(owner_count(&budget), 1);
    }

    #[test]
    fn atomic_limit_rejects_before_overflow_and_soft_trigger_is_visible() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let _a = owner.try_reserve(10).unwrap();
        assert!(matches!(
            owner.try_reserve(7),
            Err(AdmissionError::Full {
                requested: 7,
                used: 10,
                hard_limit: 16
            })
        ));
        assert!(matches!(
            owner.try_reserve(17),
            Err(AdmissionError::Oversized {
                requested: 17,
                hard_limit: 16
            })
        ));
        assert!(!budget.snapshot().checkpoint_needed());
        let normal = ChangeBudget::new();
        let normal_owner = normal.owner();
        let _trigger = normal_owner.try_reserve(CHECKPOINT_TRIGGER).unwrap();
        assert!(normal.snapshot().checkpoint_needed());
    }

    #[test]
    fn high_water_keeps_the_largest_successful_pending_change_total() {
        let budget = ChangeBudget::with_hard_limit(32);
        let owner = budget.owner();
        let mut reservation = owner.try_reserve(8).unwrap();
        assert_eq!(budget.high_water_bytes(), 8);

        reservation.try_grow_to(12).unwrap();
        assert_eq!(budget.high_water_bytes(), 12);
        assert!(matches!(
            owner.try_reserve(21),
            Err(AdmissionError::Full {
                requested: 21,
                used: 12,
                hard_limit: 32,
            })
        ));
        assert_eq!(budget.high_water_bytes(), 12);

        let charge = reservation.commit().unwrap();
        let frozen = owner.freeze().unwrap();
        drop(charge);
        assert_eq!(budget.snapshot().frozen, 12);
        assert_eq!(frozen.publish().unwrap(), 12);
        assert_eq!(budget.snapshot().total, 0);
        assert_eq!(budget.high_water_bytes(), 12);
    }

    #[test]
    fn high_water_is_race_safe_for_concurrent_reservations() {
        let budget = Arc::new(ChangeBudget::with_hard_limit(128));
        let gate = Arc::new(std::sync::Barrier::new(9));
        let (tx, rx) = mpsc::channel();
        let mut workers = Vec::new();
        for _ in 0..8 {
            let budget = budget.clone();
            let gate = gate.clone();
            let tx = tx.clone();
            workers.push(std::thread::spawn(move || {
                let owner = budget.owner();
                gate.wait();
                tx.send(owner.try_reserve(8).unwrap()).unwrap();
            }));
        }
        drop(tx);
        gate.wait();
        let reservations: Vec<_> = rx.into_iter().collect();
        for worker in workers {
            worker.join().unwrap();
        }

        assert_eq!(budget.snapshot().total, 64);
        assert_eq!(budget.high_water_bytes(), 64);
        drop(reservations);
        assert_eq!(budget.snapshot().total, 0);
        assert_eq!(budget.high_water_bytes(), 64);
    }

    #[test]
    fn work_revision_tracks_applied_work_not_reservations_or_checkpoint_progress() {
        let budget = ChangeBudget::with_hard_limit(64);
        let one = budget.owner();
        let two = budget.owner();
        let reservation = one.try_reserve(16).unwrap();
        assert_eq!(budget.snapshot().work_revision, 0);
        reservation.commit().unwrap();
        assert_eq!(one.work_revision().unwrap(), 1);
        assert_eq!(two.work_revision().unwrap(), 0);
        let frozen = one.freeze().unwrap();
        one.publish_through(&frozen).unwrap();
        assert_eq!(budget.snapshot().work_revision, 1);
        drop(two.try_reserve(8).unwrap());
        assert_eq!(budget.snapshot().work_revision, 1);
        two.try_reserve(4).unwrap().commit().unwrap();
        assert_eq!(budget.snapshot().work_revision, 2);
        assert_eq!(one.work_revision().unwrap(), 1);
        assert_eq!(two.work_revision().unwrap(), 2);
        two.retire();
        assert_eq!(budget.snapshot().work_revision, 2);
        assert_eq!(two.work_revision(), Err(AdmissionError::Retired));
    }

    #[test]
    fn checkpoint_request_requires_applied_work_and_keeps_its_revision() {
        let budget = ChangeBudget::with_hard_limit(64);
        assert!(!budget.request_checkpoint());
        assert_eq!(budget.snapshot().checkpoint_request_revision, None);

        let owner = budget.owner();
        let _charge = owner.try_reserve(16).unwrap().commit().unwrap();
        assert_eq!(budget.snapshot().work_revision, 1);
        assert!(budget.request_checkpoint());
        assert_eq!(budget.snapshot().checkpoint_request_revision, Some(1));

        // A blocked waiter can repeat its hint, but it must not manufacture
        // a fresh work revision and bypass checkpoint completion suppression.
        assert!(budget.request_checkpoint());
        let snapshot = budget.snapshot();
        assert_eq!(snapshot.work_revision, 1);
        assert_eq!(snapshot.checkpoint_request_revision, Some(1));
    }

    #[test]
    fn owner_checkpoint_request_advances_after_consumption_without_new_work() {
        let budget = ChangeBudget::with_hard_limit(64);
        let owner = budget.owner();
        let _charge = owner.try_reserve(16).unwrap().commit().unwrap();

        assert!(owner.request_checkpoint());
        let first = owner.capacity_state().unwrap().checkpoint_request_revision;
        owner.consume_checkpoint_request(first);
        assert_eq!(budget.snapshot().checkpoint_request_revision, None);

        assert!(owner.request_checkpoint());
        let second = owner.capacity_state().unwrap().checkpoint_request_revision;
        assert!(second > first);
    }

    #[test]
    fn cancelled_reservation_releases_only_reserved_bytes() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let reservation = owner.try_reserve(7).unwrap();
        assert_eq!(budget.snapshot().reserved, 7);
        drop(reservation);
        assert_eq!(
            budget.snapshot(),
            Snapshot {
                reserved: 0,
                active: 0,
                frozen: 0,
                total: 0,
                work_revision: 0,
                checkpoint_request_revision: None,
            }
        );
    }

    #[test]
    fn failed_growth_keeps_the_original_reservation() {
        let budget = ChangeBudget::with_hard_limit(10);
        let owner = budget.owner();
        let other = budget.owner();
        let mut reservation = owner.try_reserve(4).unwrap();
        let _full = other.try_reserve(6).unwrap();
        assert_eq!(
            reservation.try_grow_to(5),
            Err(AdmissionError::Full {
                requested: 5,
                used: 10,
                hard_limit: 10
            })
        );
        assert_eq!(reservation.bytes(), 4);
        assert_eq!(budget.snapshot().reserved, 10);
    }

    #[test]
    fn growth_charges_only_the_delta_not_a_second_reservation() {
        let budget = ChangeBudget::with_hard_limit(10);
        let owner = budget.owner();
        let other = budget.owner();
        let mut reservation = owner.try_reserve(4).unwrap();
        let _other = other.try_reserve(4).unwrap();
        reservation.try_grow_to(6).unwrap();
        assert_eq!(reservation.bytes(), 6);
        assert_eq!(budget.snapshot().reserved, 10);
    }

    #[test]
    fn waiting_growth_wakes_when_only_its_delta_becomes_available() {
        let budget = ChangeBudget::with_hard_limit(10);
        let owner = Arc::new(budget.owner());
        let other = budget.owner();
        let release_owner = budget.owner();
        let reservation = owner.try_reserve(4).unwrap();
        let _other = other.try_reserve(4).unwrap();
        let charge = release_owner.try_reserve(2).unwrap().commit().unwrap();
        let frozen = release_owner.freeze().unwrap();
        let (started_tx, started_rx) = mpsc::channel();
        let (result_tx, result_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut reservation = reservation;
            started_tx.send(()).unwrap();
            let result = reservation.wait_grow_to(6).map(|()| reservation);
            result_tx.send(result).unwrap();
        });
        started_rx.recv().unwrap();
        assert!(result_rx.recv_timeout(Duration::from_millis(20)).is_err());
        drop(charge);
        assert_eq!(frozen.publish().unwrap(), 2);
        let grown = result_rx
            .recv_timeout(Duration::from_secs(1))
            .unwrap()
            .unwrap();
        assert_eq!(grown.bytes(), 6);
        assert_eq!(budget.snapshot().reserved, 10);
    }

    #[test]
    fn grown_reservation_commits_and_drops_exactly_once() {
        let budget = ChangeBudget::with_hard_limit(12);
        let owner = budget.owner();
        let mut reservation = owner.try_reserve(3).unwrap();
        reservation.try_grow_to(7).unwrap();
        let charge = reservation.commit().unwrap();
        assert_eq!(budget.snapshot().reserved, 0);
        assert_eq!(budget.snapshot().active, 7);
        drop(charge);
        assert_eq!(budget.snapshot().active, 7);
    }

    #[test]
    fn oversized_growth_retains_old_reservation() {
        let budget = ChangeBudget::with_hard_limit(8);
        let owner = budget.owner();
        let mut reservation = owner.try_reserve(3).unwrap();
        assert_eq!(
            reservation.try_grow_to(9),
            Err(AdmissionError::Oversized {
                requested: 9,
                hard_limit: 8
            })
        );
        assert_eq!(reservation.bytes(), 3);
        assert_eq!(budget.snapshot().reserved, 3);
    }

    #[test]
    fn shrinking_reservation_wakes_a_competing_waiter_without_lowering_high_water() {
        let budget = ChangeBudget::with_hard_limit(20);
        let owner = budget.owner();
        let waiter_owner = Arc::new(budget.owner());
        let mut reservation = owner.try_reserve(16).unwrap();
        assert_eq!(budget.high_water_bytes(), 16);

        let (started_tx, started_rx) = mpsc::channel();
        let (result_tx, result_rx) = mpsc::channel();
        let waiter = waiter_owner.clone();
        std::thread::spawn(move || {
            started_tx.send(()).unwrap();
            result_tx.send(waiter.wait_reserve(8)).unwrap();
        });
        started_rx.recv().unwrap();
        assert!(result_rx.recv_timeout(Duration::from_millis(20)).is_err());

        reservation.shrink_to(12).unwrap();
        assert_eq!(reservation.bytes(), 12);
        // Shrinking wakes the other thread. Its eight bytes may already be
        // reserved here, so observe the process total at the channel handoff.

        let competing = result_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("shrink must wake capacity waiter")
            .unwrap();
        assert_eq!(budget.snapshot().reserved, 20);
        assert_eq!(budget.high_water_bytes(), 20);
        drop(competing);
        assert_eq!(budget.snapshot().reserved, 12);
        drop(reservation);
        assert_eq!(budget.snapshot().reserved, 0);
        assert_eq!(budget.high_water_bytes(), 20);
    }

    #[test]
    fn shrinking_reservation_preserves_other_owner_and_drop_releases_only_remainder() {
        let budget = ChangeBudget::with_hard_limit(20);
        let first = budget.owner();
        let second = budget.owner();
        let mut reservation = first.try_reserve(12).unwrap();
        let other = second.try_reserve(4).unwrap();

        reservation.shrink_to(5).unwrap();
        assert_eq!(budget.snapshot().reserved, 9);
        assert_eq!(budget.high_water_bytes(), 16);
        drop(reservation);
        assert_eq!(budget.snapshot().reserved, 4);
        drop(other);
        assert_eq!(budget.snapshot().reserved, 0);
        assert_eq!(budget.high_water_bytes(), 16);
    }

    #[test]
    fn shrinking_refuses_a_larger_total_without_changing_the_reservation() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let mut reservation = owner.try_reserve(5).unwrap();
        assert_eq!(
            reservation.shrink_to(6),
            Err(AdmissionError::CannotShrink {
                current: 5,
                requested: 6,
            })
        );
        assert_eq!(reservation.bytes(), 5);
        assert_eq!(budget.snapshot().reserved, 5);
    }

    #[test]
    fn committed_charge_survives_request_handle_drop() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let charge = owner.try_reserve(7).unwrap().commit().unwrap();
        drop(charge);
        assert_eq!(budget.snapshot().active, 7);
        assert!(matches!(
            owner.try_reserve(10),
            Err(AdmissionError::Full {
                requested: 10,
                used: 7,
                hard_limit: 16
            })
        ));
    }

    #[test]
    fn failed_freeze_handle_keeps_committed_bytes() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let _charge = owner.try_reserve(7).unwrap().commit().unwrap();
        let frozen = owner.freeze().unwrap();
        drop(frozen);
        assert_eq!(
            budget.snapshot(),
            Snapshot {
                reserved: 0,
                active: 0,
                frozen: 7,
                total: 7,
                work_revision: 1,
                checkpoint_request_revision: None,
            }
        );
    }

    #[test]
    fn publication_through_batch_preserves_newer_work() {
        let budget = ChangeBudget::with_hard_limit(32);
        let owner = budget.owner();
        let _old = owner.try_reserve(5).unwrap().commit().unwrap();
        let first = owner.freeze().unwrap();
        let _new_active = owner.try_reserve(6).unwrap().commit().unwrap();
        let second = owner.freeze().unwrap();
        let _reserved = owner.try_reserve(3).unwrap();
        assert_eq!(budget.publish_through(&first).unwrap(), 5);
        assert_eq!(
            budget.snapshot(),
            Snapshot {
                reserved: 3,
                active: 0,
                frozen: 6,
                total: 9,
                work_revision: 2,
                checkpoint_request_revision: None,
            }
        );
        assert_eq!(budget.publish_through(&second).unwrap(), 6);
    }

    #[test]
    fn blocked_committed_admission_wakes_after_publish() {
        let budget = ChangeBudget::with_hard_limit(8);
        let owner = Arc::new(budget.owner());
        let _charge = owner.try_reserve(8).unwrap().commit().unwrap();
        let frozen = owner.freeze().unwrap();
        let (started_tx, started_rx) = mpsc::channel();
        let (result_tx, result_rx) = mpsc::channel();
        let waiter = owner.clone();
        std::thread::spawn(move || {
            started_tx.send(()).unwrap();
            result_tx
                .send(
                    waiter
                        .wait_reserve(1)
                        .and_then(Reservation::commit)
                        .map(|_| ()),
                )
                .unwrap();
        });
        started_rx.recv().unwrap();
        assert!(result_rx.recv_timeout(Duration::from_millis(20)).is_err());
        assert_eq!(frozen.publish().unwrap(), 8);
        assert_eq!(
            result_rx.recv_timeout(Duration::from_secs(1)).unwrap(),
            Ok(())
        );
    }

    #[test]
    fn blocked_admission_wakes_with_retired_when_its_owner_is_retired() {
        let budget = ChangeBudget::with_hard_limit(8);
        let owner = Arc::new(budget.owner());
        let _charge = owner.try_reserve(8).unwrap().commit().unwrap();
        let (started_tx, started_rx) = mpsc::channel();
        let (result_tx, result_rx) = mpsc::channel();
        let waiter = owner.clone();
        std::thread::spawn(move || {
            started_tx.send(()).unwrap();
            result_tx
                .send(
                    waiter
                        .wait_reserve(1)
                        .and_then(Reservation::commit)
                        .map(|_| ()),
                )
                .unwrap();
        });
        started_rx.recv().unwrap();
        assert!(result_rx.recv_timeout(Duration::from_millis(20)).is_err());
        owner.retire();
        assert_eq!(
            result_rx.recv_timeout(Duration::from_secs(1)).unwrap(),
            Err(AdmissionError::Retired)
        );
    }

    #[test]
    fn a_frozen_batch_from_another_budget_cannot_release_matching_ids() {
        let left = ChangeBudget::with_hard_limit(16);
        let right = ChangeBudget::with_hard_limit(16);
        let left_owner = left.owner();
        let right_owner = right.owner();
        let _left_charge = left_owner.try_reserve(5).unwrap().commit().unwrap();
        let _right_charge = right_owner.try_reserve(7).unwrap().commit().unwrap();
        let left_batch = left_owner.freeze().unwrap();
        let right_batch = right_owner.freeze().unwrap();
        assert_eq!(
            left.publish_through(&right_batch),
            Err(AdmissionError::WrongBudget)
        );
        assert_eq!(left.snapshot().frozen, 5);
        assert_eq!(right.snapshot().frozen, 7);
        assert_eq!(left.publish_through(&left_batch).unwrap(), 5);
    }

    #[test]
    fn retiring_one_owner_does_not_clear_another() {
        let budget = ChangeBudget::with_hard_limit(32);
        let live = budget.owner();
        let restore = budget.owner();
        let _live = live.try_reserve(7).unwrap().commit().unwrap();
        let _restore = restore.try_reserve(9).unwrap().commit().unwrap();
        restore.retire();
        assert_eq!(
            budget.snapshot(),
            Snapshot {
                reserved: 0,
                active: 7,
                frozen: 0,
                total: 7,
                work_revision: 2,
                checkpoint_request_revision: None,
            }
        );
        assert!(matches!(
            live.try_reserve(26),
            Err(AdmissionError::Full {
                requested: 26,
                used: 7,
                hard_limit: 32
            })
        ));
    }

    #[test]
    fn waiting_reservation_survives_freeze_and_publish_until_commit() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let reserved = owner.wait_reserve(5).unwrap();
        assert_eq!(budget.snapshot().reserved, 5);
        let empty = owner.freeze().unwrap();
        assert_eq!(empty.publish().unwrap(), 0);
        assert_eq!(budget.snapshot().reserved, 5);
        let _charge = reserved.commit().unwrap();
        let frozen = owner.freeze().unwrap();
        assert_eq!(frozen.publish().unwrap(), 5);
        assert_eq!(budget.snapshot().total, 0);
    }

    #[test]
    fn process_registry_shares_live_default_budget() {
        let first = ChangeBudget::process_shared();
        let second = ChangeBudget::process_shared();
        assert!(
            Arc::ptr_eq(&first.0, &second.0),
            "live callers must share one process budget"
        );
        let owner = first.owner();
        let reserved = owner.try_reserve(3).unwrap();
        // Other Engine tests use this same process-wide budget in parallel.
        // Observe this owner through the second handle instead of requiring
        // every unrelated reservation in the process to be absent.
        {
            let state = second.0.state.lock().unwrap();
            assert_eq!(state.owners.get(&owner.id).unwrap().reserved, 3);
        }
        drop(reserved);
        let state = second.0.state.lock().unwrap();
        assert_eq!(state.owners.get(&owner.id).unwrap().reserved, 0);
    }

    #[test]
    fn owner_cannot_publish_another_owners_batch() {
        let budget = ChangeBudget::with_hard_limit(16);
        let one = budget.owner();
        let two = budget.owner();
        let _charge = one.try_reserve(3).unwrap().commit().unwrap();
        let batch = one.freeze().unwrap();
        assert_eq!(two.publish_through(&batch), Err(AdmissionError::WrongOwner));
        assert_eq!(one.publish_through(&batch).unwrap(), 3);
    }

    #[test]
    fn retained_charge_clones_share_one_resident_entry_and_one_revision() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let charge = owner.try_reserve(7).unwrap().commit_retained().unwrap();
        assert_eq!(charge.bytes(), 7);
        assert_eq!(budget.snapshot().active, 7);
        assert_eq!(budget.snapshot().work_revision, 1);

        let clone = charge.clone();
        assert_eq!(budget.snapshot().total, 7);
        drop(charge);
        assert_eq!(budget.snapshot().total, 7);
        assert_eq!(budget.snapshot().work_revision, 1);
        drop(clone);
        assert_eq!(budget.snapshot().total, 0);
        assert_eq!(budget.snapshot().work_revision, 1);
    }

    #[test]
    fn publication_does_not_free_retained_payload_from_a_frozen_epoch() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let charge = owner.try_reserve(7).unwrap().commit_retained().unwrap();
        let frozen = owner.freeze().unwrap();
        assert_eq!(budget.snapshot().active, 0);
        assert_eq!(budget.snapshot().frozen, 7);
        assert_eq!(frozen.publish().unwrap(), 0);
        assert_eq!(budget.snapshot().total, 7);
        drop(charge);
        assert_eq!(budget.snapshot().total, 0);
    }

    #[test]
    fn dropping_an_older_retained_epoch_keeps_later_active_payload_charged() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let old = owner.try_reserve(5).unwrap().commit_retained().unwrap();
        let _frozen = owner.freeze().unwrap();
        let current = owner.try_reserve(6).unwrap().commit_retained().unwrap();
        assert_eq!(budget.snapshot().active, 6);
        assert_eq!(budget.snapshot().frozen, 5);
        drop(old);
        assert_eq!(budget.snapshot().active, 6);
        assert_eq!(budget.snapshot().frozen, 0);
        assert_eq!(budget.snapshot().total, 6);
        drop(current);
        assert_eq!(budget.snapshot().total, 0);
    }

    #[test]
    fn retiring_an_owner_preserves_retained_payload_until_its_last_drop() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let charge = owner.try_reserve(7).unwrap().commit_retained().unwrap();
        owner.retire();
        assert_eq!(budget.snapshot().total, 7);
        assert!(matches!(owner.try_reserve(1), Err(AdmissionError::Retired)));
        drop(charge);
        assert_eq!(budget.snapshot().total, 0);
    }

    #[test]
    fn blocked_admission_wakes_only_after_the_last_retained_handle_drops() {
        let budget = ChangeBudget::with_hard_limit(8);
        let owner = Arc::new(budget.owner());
        let charge = owner.try_reserve(8).unwrap().commit_retained().unwrap();
        let clone = charge.clone();
        let (started_tx, started_rx) = mpsc::channel();
        let (result_tx, result_rx) = mpsc::channel();
        let waiter = owner.clone();
        std::thread::spawn(move || {
            started_tx.send(()).unwrap();
            result_tx.send(waiter.wait_reserve(1).map(|_| ())).unwrap();
        });
        started_rx.recv().unwrap();
        assert!(result_rx.recv_timeout(Duration::from_millis(20)).is_err());
        drop(charge);
        assert!(result_rx.recv_timeout(Duration::from_millis(20)).is_err());
        drop(clone);
        assert_eq!(
            result_rx.recv_timeout(Duration::from_secs(1)).unwrap(),
            Ok(())
        );
    }

    #[test]
    fn retained_commit_advances_revision_once_and_never_on_clone_or_drop() {
        let budget = ChangeBudget::with_hard_limit(16);
        let owner = budget.owner();
        let charge = owner.try_reserve(3).unwrap().commit_retained().unwrap();
        assert_eq!(owner.work_revision().unwrap(), 1);
        let clone = charge.clone();
        drop(clone);
        drop(charge);
        assert_eq!(owner.work_revision().unwrap(), 1);
        let _next = owner.try_reserve(2).unwrap().commit_retained().unwrap();
        assert_eq!(owner.work_revision().unwrap(), 2);
    }

    #[test]
    fn wake_epoch_advances_on_reservation_cancel() {
        let budget = ChangeBudget::with_hard_limit(16);
        let wake = budget.checkpoint_wake();
        let epoch = wake.epoch();
        let reservation = budget.owner().try_reserve(3).unwrap();
        drop(reservation);
        assert!(wake.epoch() > epoch);
    }

    #[test]
    fn wake_observes_signal_before_waiter_sleeps() {
        let budget = ChangeBudget::with_hard_limit(16);
        let wake = budget.checkpoint_wake();
        let observed = wake.epoch();
        wake.signal();
        assert!(wake.wait_for_change_timeout(observed, Duration::from_millis(1)));
    }

    #[test]
    fn wake_timeout_is_bounded_without_a_signal() {
        let budget = ChangeBudget::with_hard_limit(16);
        let wake = budget.checkpoint_wake();
        let started = Instant::now();
        assert!(!wake.wait_for_change_timeout(wake.epoch(), Duration::from_millis(10)));
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn owner_capacity_state_selects_only_its_publishable_work() {
        let budget = ChangeBudget::with_hard_limit(64);
        let engine_a = budget.owner();
        let engine_b = budget.owner();
        let a_active = engine_a.try_reserve(9).unwrap().commit().unwrap();
        let b_reservation = engine_b.try_reserve(7).unwrap();

        assert_eq!(
            engine_b.capacity_state().unwrap(),
            OwnerCapacityState {
                active: 0,
                frozen: 0,
                work_revision: 0,
                checkpoint_request_revision: None,
            },
            "another Engine's active work and local reservations are not publishable here"
        );
        let frozen = engine_a.freeze().unwrap();
        assert_eq!(
            engine_a.capacity_state().unwrap(),
            OwnerCapacityState {
                active: 0,
                frozen: 9,
                work_revision: 1,
                checkpoint_request_revision: None,
            }
        );
        drop(b_reservation);
        frozen.publish().unwrap();
        drop(a_active);
    }

    #[test]
    fn signal_waits_for_the_predicate_mutex_before_notifying() {
        let budget = ChangeBudget::with_hard_limit(16);
        let wake = budget.checkpoint_wake();
        let guard = wake.lock.lock().unwrap();
        let (started_tx, started_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let signal = wake.clone();
        std::thread::spawn(move || {
            started_tx.send(()).unwrap();
            signal.signal();
            done_tx.send(()).unwrap();
        });
        started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(done_rx.recv_timeout(Duration::from_millis(20)).is_err());
        drop(guard);
        done_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    }
}
