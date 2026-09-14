//! Whole-record admission before the apply boundary.
//!
//! The transport retains the returned reservation across publication. Repricing
//! returns that same ownership, so its raw payload does not become uncharged
//! while the apply worker waits or stages it. The apply guard stays alive until
//! the caller has persisted AOF and advanced its watermark.
//!
//! [`Engine::try_reserve_record`] never returns a record with **zero**
//! reservation. Exact domain pricing that needs context it cannot resolve
//! before publication (`RecordAdmissionError::NeedsPreparation`) still gets
//! the raw+transport floor apply already knows how to charge — see
//! [`Engine::minimal_context_pending_reservation`] — so the publisher never
//! hands its post-publication apply path (the WAL delivery loop's unbounded
//! `wait_reserve_record_ram`, or `begin_admitted_record`'s bounded repricing
//! `wait_grow_to`) a record with nothing reserved to grow from. A Full
//! budget at that minimal floor is refused here, before publication, exactly
//! like a Full at the fully-priced bound.

use super::Engine;
use crate::capture_barrier::ApplyLease;
use crate::change_budget::{
    AdmissionError, BudgetWake, ChangeBudget, Owner, OwnerCapacityState, Reservation,
    RetainedCharge,
};
use crate::change_record_cost::{NormalizeError, RecordEstimate};
use crate::log_entry::RaftLogEntry;
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};

use super::record_charges::RecordChargeJournal;

static NEXT_ENGINE: AtomicU64 = AtomicU64::new(1);

pub(super) struct EngineChanges {
    pub(super) records: RecordChargeJournal,
    pub(super) owner: Owner,
    pub(super) budget: ChangeBudget,
    id: u64,
}

impl Default for EngineChanges {
    fn default() -> Self {
        let budget = ChangeBudget::process_shared();
        let owner = budget.owner();
        let id = NEXT_ENGINE
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .expect("Engine admission identity exhausted");
        Self {
            records: RecordChargeJournal::new(),
            owner,
            budget,
            id,
        }
    }
}

/// An allocation-free price retained while the delivered working copy is
/// released. The committed WAL source remains pinned during the wait.
pub(crate) struct RecordRamRequest {
    runtime: u64,
    bytes: usize,
    extra_owned: usize,
}

/// All sizes are conservative owned-memory bounds. `extra_owned` comes from
/// the transport's raw-record/codec ownership; normalized field data is priced
/// separately from the current collection schema and live coverage.
pub(crate) struct RecordReservation {
    runtime: u64,
    reservation: Reservation,
    extra_owned: usize,
    ngram_cost_workspace_bytes: usize,
    staged_text: bool,
    preparation_bytes: usize,
    prepared_text: Option<super::text_preparation::PreparedTextRows>,
    wait_for_preparation: bool,
}

impl RecordReservation {
    pub(crate) fn source_retention(&mut self) -> crate::change_budget::SourceRetention {
        self.reservation.source_retention()
    }

    /// Release one transport-owned raw copy after its caller has dropped it.
    /// This does not affect the decoded entry or any normalized change charge.
    /// It must run with no apply lease.
    pub(crate) fn release_transport_bytes(&mut self) -> Result<(), AdmissionError> {
        let released = self.extra_owned;
        if released == 0 {
            return Ok(());
        }
        let remaining = self
            .bytes()
            .checked_sub(released)
            .expect("transport bytes are part of this reservation");
        self.reservation.shrink_to(remaining)?;
        self.extra_owned = 0;
        Ok(())
    }

    pub(super) fn grow_preparation(&mut self, additional: usize) -> Result<(), AdmissionError> {
        let overflow = AdmissionError::Oversized {
            requested: usize::MAX,
            hard_limit: crate::change_budget::HARD_LIMIT,
        };
        let required = self.bytes().checked_add(additional).ok_or(overflow)?;
        if self.wait_for_preparation {
            self.wait_grow_to(required)?;
        } else {
            self.try_grow_to(required)?;
        }
        self.preparation_bytes = self
            .preparation_bytes
            .checked_add(additional)
            .ok_or(overflow)?;
        Ok(())
    }
    /// Release helper workspace only after its temporary owners have dropped.
    /// This runs outside apply and preserves other prepared rows and transport.
    pub(super) fn release_preparation_workspace(
        &mut self,
        bytes: usize,
    ) -> Result<(), AdmissionError> {
        let prepared = self
            .preparation_bytes
            .checked_sub(bytes)
            .expect("released workspace was admitted as preparation");
        let remaining = self
            .bytes()
            .checked_sub(bytes)
            .expect("prepared workspace is part of this reservation");
        self.reservation.shrink_to(remaining)?;
        self.preparation_bytes = prepared;
        Ok(())
    }

    pub(crate) fn bytes(&self) -> usize {
        self.reservation.bytes()
    }

    /// Retain only the prepared borrowed representation. The prepared rows
    /// are held by the caller, so no owned decoder or staging scratch remains.
    pub(super) fn finish_borrowed_preparation(
        &mut self,
        bytes: usize,
    ) -> Result<(), AdmissionError> {
        debug_assert!(self.prepared_text.is_none());
        debug_assert_eq!(self.extra_owned, 0);
        self.reservation.shrink_to(bytes)?;
        self.preparation_bytes = 0;
        Ok(())
    }

    pub(super) fn shrink_borrowed_to(&mut self, bytes: usize) -> Result<(), AdmissionError> {
        debug_assert_eq!(self.preparation_bytes, 0);
        self.reservation.shrink_to(bytes)
    }

    /// This must run with no apply lease. Failure retains the original bytes.
    pub(crate) fn try_grow_to(&mut self, required: usize) -> Result<(), AdmissionError> {
        self.reservation.try_grow_to(required)
    }

    pub(crate) fn wait_grow_to(&mut self, required: usize) -> Result<(), AdmissionError> {
        self.reservation.wait_grow_to(required)
    }

    pub(super) fn before_publication(&mut self) {
        self.wait_for_preparation = false;
    }

    fn discard_preparation(&mut self) -> Result<(), AdmissionError> {
        // Drop every mmap owner before releasing its reserved representation.
        // Raw entry and transport reservations remain owned by this record.
        drop(self.prepared_text.take());
        let remaining = self
            .bytes()
            .checked_sub(self.preparation_bytes)
            .expect("prepared bytes are part of this reservation");
        self.reservation.shrink_to(remaining)?;
        self.preparation_bytes = 0;
        Ok(())
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub(crate) enum RecordAdmissionError {
    #[error("record preparation failed: {0}")]
    Preparation(String),
    #[error("pending record admission: {0:?}")]
    Capacity(AdmissionError),
    #[error("record preparation needs its durable source: {0:?}")]
    NeedsPreparation(NormalizeError),
    #[error("record memory bound overflow")]
    Overflow,
    #[error("record reservation belongs to another Engine")]
    WrongEngine,
}

/// A failed prepare never owns an apply lease and returns the raw-record
/// reservation. The transport still owns the exact log entry as well.
pub(crate) struct RepriceRecord {
    pub(crate) entry: RaftLogEntry,
    pub(crate) reservation: RecordReservation,
    pub(crate) required: Option<usize>,
    pub(crate) error: RecordAdmissionError,
}

/// The guard is thread-affine because the capture barrier is thread-affine.
/// It cannot be forged or reused for a second record or another Engine.
pub(crate) struct RecordApplyGuard<'a> {
    entry: Option<RaftLogEntry>,
    prepared_text: Option<super::text_preparation::PreparedTextRows>,
    charge: RetainedCharge,
    runtime: u64,
    used: Cell<bool>,
    apply: ApplyLease<'a>,
}

impl RecordApplyGuard<'_> {
    pub(crate) fn entry(&self) -> Option<&RaftLogEntry> {
        self.entry.as_ref()
    }
    pub(crate) fn apply_lease(&self) -> &ApplyLease<'_> {
        &self.apply
    }
}

impl Engine {
    /// The caller owns apply and has rechecked its borrowed plan. No source or
    /// codec workspace remains in this reservation, only attached metadata.
    pub(super) fn retain_borrowed_reservation(
        &self,
        reserved: RecordReservation,
    ) -> Result<RetainedCharge, RecordAdmissionError> {
        if reserved.runtime != self.changes.id {
            return Err(RecordAdmissionError::WrongEngine);
        }
        let charge = reserved
            .reservation
            .commit_retained()
            .map_err(RecordAdmissionError::Capacity)?;
        self.changes.records.retain(charge.clone());
        Ok(charge)
    }

    pub(crate) fn has_capacity_waiters(&self) -> bool {
        self.changes.budget.has_capacity_waiters()
    }

    #[cfg(test)]
    pub(crate) fn with_change_budget(budget: ChangeBudget) -> Self {
        let mut engine = Self::new();
        engine.changes.owner = budget.owner();
        engine.changes.budget = budget;
        engine
    }

    pub(crate) fn request_pending_checkpoint(&self) {
        self.changes.budget.request_checkpoint();
    }

    /// Fallback checkpoint maintenance selects only this Engine's work, while
    /// its waiter demand remains process-global to relieve other Engines too.
    pub(crate) fn capacity_owner_state(&self) -> Option<OwnerCapacityState> {
        self.changes.owner.capacity_state().ok()
    }

    pub(crate) fn checkpoint_wake(&self) -> std::sync::Arc<BudgetWake> {
        self.changes.budget.checkpoint_wake()
    }

    pub(crate) fn record_owned_bytes(entry: &RaftLogEntry) -> Result<usize, RecordAdmissionError> {
        super::record_ram::estimate_record_ram(entry).map_err(|_| RecordAdmissionError::Overflow)
    }

    pub(crate) fn record_decode_peak(entry: &RaftLogEntry) -> Result<usize, RecordAdmissionError> {
        super::record_ram::estimate_record_decode_peak(entry)
            .map_err(|_| RecordAdmissionError::Overflow)
    }

    /// Replace the scanning workspace price after a bounded wire decoder has
    /// produced the record. The same reservation keeps both the decoded peak
    /// and the transport source alive; this never acquires an apply lease.
    pub(crate) fn price_decoded_record(
        &self,
        entry: &RaftLogEntry,
        reserved: &mut RecordReservation,
        decoded_peak: usize,
        transport_bytes: usize,
        before_publication: bool,
    ) -> Result<(), RecordAdmissionError> {
        if reserved.runtime != self.changes.id {
            return Err(RecordAdmissionError::WrongEngine);
        }
        let raw = Self::record_owned_bytes(entry)?;
        let overhead = decoded_peak
            .checked_sub(raw)
            .ok_or(RecordAdmissionError::Overflow)?;
        reserved.extra_owned = transport_bytes
            .checked_add(overhead)
            .ok_or(RecordAdmissionError::Overflow)?;
        if before_publication {
            self.price_before_publication(entry, reserved)?;
        } else {
            // A foreign committed delivery does not reserve a cost table until
            // begin_admitted_record can return ownership for a capacity wait.
            // Keep any workspace already supplied by local admission.
            let required = decoded_peak
                .checked_add(transport_bytes)
                .and_then(|n| n.checked_add(reserved.ngram_cost_workspace_bytes))
                .ok_or(RecordAdmissionError::Overflow)?;
            reserved
                .reservation
                .shrink_to(required)
                .map_err(RecordAdmissionError::Capacity)?;
        }
        Ok(())
    }

    /// Reserve the fixed cost table before calling its estimator. RAM-only
    /// delivery keeps its old scanner/decoder bound until this point. A failed
    /// growth returns the same record ownership for waiting outside apply.
    fn ensure_ngram_cost_workspace(
        &self,
        entry: &RaftLogEntry,
        reserved: &mut RecordReservation,
    ) -> Result<(), (Option<usize>, RecordAdmissionError)> {
        if reserved.staged_text
            || (reserved.ngram_cost_workspace_bytes == 0
                && !self.may_need_default_ngram_workspace(entry))
        {
            return Ok(());
        }
        let workspace = crate::change_record_cost::DEFAULT_NGRAM_COST_WORKSPACE_BYTES;
        let minimum = Self::record_owned_bytes(entry)
            .and_then(|raw| {
                raw.checked_add(reserved.extra_owned)
                    .and_then(|n| n.checked_add(reserved.preparation_bytes))
                    .and_then(|n| n.checked_add(workspace))
                    .ok_or(RecordAdmissionError::Overflow)
            })
            .map_err(|error| (None, error))?;
        reserved
            .try_grow_to(minimum)
            .map_err(|error| (Some(minimum), RecordAdmissionError::Capacity(error)))?;
        reserved.ngram_cost_workspace_bytes = workspace;
        Ok(())
    }

    /// Only this reservation-owning path may construct an exact Ngram table.
    /// Its input and table bounds have already been reserved. The table is
    /// destroyed before this helper returns; generic estimators stay unchanged.
    fn record_memory_bound_with_workspace(
        &self,
        entry: &RaftLogEntry,
        reserved: &RecordReservation,
        staged_text: bool,
    ) -> Result<usize, RecordAdmissionError> {
        if staged_text || reserved.ngram_cost_workspace_bytes == 0 {
            return self.record_memory_bound(entry, reserved.extra_owned, staged_text);
        }
        let raw = Self::record_owned_bytes(entry)?;
        let floor = raw
            .checked_add(reserved.extra_owned)
            .and_then(|n| n.checked_add(reserved.preparation_bytes))
            .and_then(|n| n.checked_add(reserved.ngram_cost_workspace_bytes))
            .ok_or(RecordAdmissionError::Overflow)?;
        assert!(
            reserved.bytes() >= floor,
            "exact Ngram workspace must be reserved before pricing"
        );
        match self.estimate_record_exact_default_ngram_cost(entry) {
            RecordEstimate::Ready(cost) => cost
                .active
                .checked_add(cost.frozen)
                .and_then(|n| n.checked_add(cost.prepublish))
                .and_then(|n| n.checked_add(raw))
                .and_then(|n| n.checked_add(reserved.extra_owned))
                .ok_or(RecordAdmissionError::Overflow),
            RecordEstimate::Retain { cause } => Err(RecordAdmissionError::NeedsPreparation(cause)),
        }
    }

    /// Finish complete local admission before a transport may publish the WAL.
    /// The table is gone; keep its reserved workspace for the later state recheck.
    fn price_before_publication(
        &self,
        entry: &RaftLogEntry,
        reserved: &mut RecordReservation,
    ) -> Result<(), RecordAdmissionError> {
        self.ensure_ngram_cost_workspace(entry, reserved)
            .map_err(|(_, error)| error)?;
        let ordinary = self.record_memory_bound_with_workspace(entry, reserved, false)?;
        let ordinary_with_workspace = ordinary
            .checked_add(reserved.ngram_cost_workspace_bytes)
            .ok_or(RecordAdmissionError::Overflow)?;
        reserved.staged_text = ordinary_with_workspace > crate::change_budget::HARD_LIMIT;
        let final_bound = if reserved.staged_text {
            self.record_memory_bound(entry, reserved.extra_owned, true)?
        } else {
            ordinary
        };
        let required = final_bound
            .checked_add(reserved.ngram_cost_workspace_bytes)
            .ok_or(RecordAdmissionError::Overflow)?;
        reserved
            .try_grow_to(required)
            .map_err(RecordAdmissionError::Capacity)?;
        reserved
            .reservation
            .shrink_to(required)
            .map_err(RecordAdmissionError::Capacity)?;
        Ok(())
    }

    fn record_memory_bound(
        &self,
        entry: &RaftLogEntry,
        extra_owned: usize,
        staged_text: bool,
    ) -> Result<usize, RecordAdmissionError> {
        let raw = Self::record_owned_bytes(entry)?;
        if staged_text {
            return self
                .prepared_text_bound(entry)?
                .checked_add(raw)
                .and_then(|n| n.checked_add(extra_owned))
                .ok_or(RecordAdmissionError::Overflow);
        }
        match self.estimate_record_cost(entry) {
            RecordEstimate::Ready(cost) => cost
                .active
                .checked_add(cost.frozen)
                .and_then(|n| n.checked_add(cost.prepublish))
                .and_then(|n| n.checked_add(raw))
                .and_then(|n| n.checked_add(extra_owned))
                .ok_or(RecordAdmissionError::Overflow),
            RecordEstimate::Retain { cause } => Err(RecordAdmissionError::NeedsPreparation(cause)),
        }
    }

    /// Price only the one owned, decoded delivery. Normalized changes and
    /// transport clones do not exist yet. `begin_admitted_record` must grow this
    /// reservation before creating either kind of payload.
    pub(crate) fn record_ram_request(
        &self,
        entry: &RaftLogEntry,
        extra_owned: usize,
    ) -> Result<RecordRamRequest, RecordAdmissionError> {
        Ok(self.record_ram_request_from_bound(Self::record_owned_bytes(entry)?, extra_owned))
    }

    /// The native WAL computes this bound while it owns the source record.
    /// Using it does not clone or decode the pending delivery.
    pub(crate) fn record_ram_request_from_bound(
        &self,
        bytes: usize,
        extra_owned: usize,
    ) -> RecordRamRequest {
        RecordRamRequest {
            runtime: self.changes.id,
            bytes,
            extra_owned,
        }
    }

    pub(crate) fn try_reserve_record_ram(
        &self,
        request: &RecordRamRequest,
    ) -> Result<RecordReservation, RecordAdmissionError> {
        self.reserve_record_ram(request, false)
    }

    /// The caller has released the delivered working copy and pinned its WAL
    /// source before this wait. No Engine apply lease may be held here.
    pub(crate) fn wait_reserve_record_ram(
        &self,
        request: &RecordRamRequest,
    ) -> Result<RecordReservation, RecordAdmissionError> {
        self.request_pending_checkpoint();
        self.reserve_record_ram(request, true)
    }

    fn reserve_record_ram(
        &self,
        request: &RecordRamRequest,
        wait: bool,
    ) -> Result<RecordReservation, RecordAdmissionError> {
        if request.runtime != self.changes.id {
            return Err(RecordAdmissionError::WrongEngine);
        }
        let bytes = request
            .bytes
            .checked_add(request.extra_owned)
            .ok_or(RecordAdmissionError::Overflow)?;
        let reservation = if wait {
            self.changes.owner.wait_reserve(bytes)
        } else {
            self.changes.owner.try_reserve(bytes)
        }
        .map_err(RecordAdmissionError::Capacity)?;
        Ok(RecordReservation {
            runtime: request.runtime,
            reservation,
            extra_owned: request.extra_owned,
            ngram_cost_workspace_bytes: 0,
            staged_text: false,
            preparation_bytes: 0,
            prepared_text: None,
            wait_for_preparation: true,
        })
    }

    /// No apply lease is taken here. Full local admission maps to HTTP429;
    /// oversized input is handed to durable preparation.
    ///
    /// Exact domain pricing can need context this door does not have yet
    /// (`RecordAdmissionError::NeedsPreparation`, e.g. an unresolved schema
    /// or live-coverage read) — that is never a reason to discard a valid
    /// record, but it also must never become a **publish with zero
    /// reservation**: the WAL delivery loop's post-publication path
    /// (`try_reserve_record_ram` / `wait_reserve_record_ram`) has no bound
    /// on how long it waits for capacity, and a record with no reservation
    /// at all reaches exactly that unbounded wait. So on `NeedsPreparation`
    /// this reserves the same raw+transport floor apply already knows how
    /// to charge before it can price exactly — the same bound
    /// `raft_sm::decode_admitted` reserves before it decodes — via
    /// [`Self::minimal_context_pending_reservation`]. Any growth beyond that
    /// floor is priced later, with real context, by `begin_admitted_record`;
    /// a Full observed even at this minimal floor is refused here, before
    /// publication, like any other capacity failure.
    pub(crate) fn try_reserve_record(
        &self,
        entry: &RaftLogEntry,
        extra_owned: usize,
    ) -> Result<RecordReservation, RecordAdmissionError> {
        if self.may_need_default_ngram_workspace(entry) {
            // Raw input is retained while exact pricing runs. Returning this
            // reservation is permission to publish, so finish normalized pricing
            // before returning it, not only later in begin_admitted_record.
            let request = self.record_ram_request(entry, extra_owned)?;
            let mut reserved = self.try_reserve_record_ram(&request)?;
            return match self.price_before_publication(entry, &mut reserved) {
                Ok(()) => Ok(reserved),
                Err(RecordAdmissionError::NeedsPreparation(_)) => {
                    // `reserved` already holds exactly the raw+transport
                    // floor; keep that ownership rather than dropping and
                    // re-acquiring it, which would open a capacity race.
                    Ok(reserved)
                }
                Err(error) => Err(error),
            };
        }
        let ordinary = match self.record_memory_bound(entry, extra_owned, false) {
            Ok(bound) => bound,
            Err(RecordAdmissionError::NeedsPreparation(_)) => {
                return self.minimal_context_pending_reservation(entry, extra_owned);
            }
            Err(error) => return Err(error),
        };
        let staged_text = ordinary > crate::change_budget::HARD_LIMIT;
        let bytes = if staged_text {
            self.record_memory_bound(entry, extra_owned, true)?
        } else {
            ordinary
        };
        let reservation = self
            .changes
            .owner
            .try_reserve(bytes)
            .map_err(RecordAdmissionError::Capacity)?;
        Ok(RecordReservation {
            runtime: self.changes.id,
            reservation,
            extra_owned,
            ngram_cost_workspace_bytes: 0,
            staged_text,
            preparation_bytes: 0,
            prepared_text: None,
            wait_for_preparation: true,
        })
    }

    /// The raw+transport floor for a record whose exact price needs
    /// context this door cannot resolve. See [`Self::try_reserve_record`].
    /// A Full here is a genuine capacity refusal — classified through the
    /// same [`RecordAdmissionError::Capacity`] path as every other
    /// prepublication admission failure — never a silent zero-reservation
    /// publish.
    fn minimal_context_pending_reservation(
        &self,
        entry: &RaftLogEntry,
        extra_owned: usize,
    ) -> Result<RecordReservation, RecordAdmissionError> {
        let request = self.record_ram_request(entry, extra_owned)?;
        self.try_reserve_record_ram(&request)
    }

    /// Revalidate against the state that will be changed. A stale schema or
    /// larger deletion coverage returns ownership for growth outside this lease.
    pub(crate) fn begin_admitted_record(
        &self,
        entry: RaftLogEntry,
        mut reserved: RecordReservation,
    ) -> Result<RecordApplyGuard<'_>, RepriceRecord> {
        if reserved.runtime != self.changes.id {
            return Err(RepriceRecord {
                entry,
                reservation: reserved,
                required: None,
                error: RecordAdmissionError::WrongEngine,
            });
        }
        loop {
            if let Err((required, error)) = self.ensure_ngram_cost_workspace(&entry, &mut reserved)
            {
                return Err(RepriceRecord {
                    entry,
                    reservation: reserved,
                    required,
                    error,
                });
            }
            // Decide representation before taking the apply lease. External WAL
            // delivery starts with only its raw reservation and reaches this path too.
            if !reserved.staged_text
                && self
                    .record_memory_bound_with_workspace(&entry, &reserved, false)
                    .is_ok_and(|bytes| {
                        bytes.saturating_add(reserved.ngram_cost_workspace_bytes)
                            > crate::change_budget::HARD_LIMIT
                    })
            {
                reserved.staged_text = true;
            }
            if reserved.staged_text {
                let minimum = match self.record_memory_bound(&entry, reserved.extra_owned, true) {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        return Err(RepriceRecord {
                            entry,
                            reservation: reserved,
                            required: None,
                            error,
                        })
                    }
                };
                if reserved.bytes() < minimum {
                    return Err(RepriceRecord {
                        entry,
                        reservation: reserved,
                        required: Some(minimum),
                        error: RecordAdmissionError::Capacity(AdmissionError::Full {
                            requested: minimum,
                            used: self.changes.budget.snapshot().total,
                            hard_limit: crate::change_budget::HARD_LIMIT,
                        }),
                    });
                }
                if reserved.prepared_text.is_none() {
                    match self.prepare_text_rows(&entry, &mut reserved) {
                        Ok(rows) => reserved.prepared_text = Some(rows),
                        Err(error) => {
                            let error = error
                                .downcast_ref::<RecordAdmissionError>()
                                .cloned()
                                .unwrap_or_else(|| {
                                    RecordAdmissionError::Preparation(error.to_string())
                                });
                            let error = reserved
                                .discard_preparation()
                                .err()
                                .map(RecordAdmissionError::Capacity)
                                .unwrap_or(error);
                            return Err(RepriceRecord {
                                entry,
                                reservation: reserved,
                                required: None,
                                error,
                            });
                        }
                    }
                }
            }
            let apply = self.capture_barrier.apply();
            if reserved
                .prepared_text
                .as_ref()
                .is_some_and(|rows| !rows.matches(self))
            {
                drop(apply);
                if let Err(error) = reserved.discard_preparation() {
                    return Err(RepriceRecord {
                        entry,
                        reservation: reserved,
                        required: None,
                        error: RecordAdmissionError::Capacity(error),
                    });
                }
                continue;
            }
            let required = match self
                .record_memory_bound_with_workspace(&entry, &reserved, reserved.staged_text)
                .and_then(|bytes| {
                    bytes
                        .checked_add(reserved.preparation_bytes)
                        .ok_or(RecordAdmissionError::Overflow)
                }) {
                Ok(required) => required,
                Err(error) => {
                    drop(apply);
                    return Err(RepriceRecord {
                        entry,
                        reservation: reserved,
                        required: None,
                        error,
                    });
                }
            };
            // Keep room for another exact state recheck when ownership must
            // return to the apply caller for growth outside this lease.
            let retry_required = match required.checked_add(reserved.ngram_cost_workspace_bytes) {
                Some(bytes) => bytes,
                None => {
                    drop(apply);
                    return Err(RepriceRecord {
                        entry,
                        reservation: reserved,
                        required: None,
                        error: RecordAdmissionError::Overflow,
                    });
                }
            };
            if required > reserved.bytes() {
                let error = RecordAdmissionError::Capacity(AdmissionError::Full {
                    requested: retry_required,
                    used: self.changes.budget.snapshot().total,
                    hard_limit: crate::change_budget::HARD_LIMIT,
                });
                drop(apply);
                return Err(RepriceRecord {
                    entry,
                    reservation: reserved,
                    required: Some(retry_required),
                    error,
                });
            }
            // Exact pricing's fixed table is no longer alive. Remove its
            // workspace and any old normalized overestimate before retention.
            if reserved.ngram_cost_workspace_bytes != 0 {
                reserved
                    .reservation
                    .shrink_to(required)
                    .expect("exact final bound was checked against reserved ownership");
                reserved.ngram_cost_workspace_bytes = 0;
            }
            // EngineChanges never retires an owner while it still has reservations.
            // Thus this conversion cannot lose ownership to a concurrent restore.
            let charge = reserved
                .reservation
                .commit_retained()
                .expect("live Engine reservation owner cannot retire during apply preparation");
            return Ok(RecordApplyGuard {
                entry: Some(entry),
                prepared_text: reserved.prepared_text,
                charge,
                runtime: self.changes.id,
                used: Cell::new(false),
                apply,
            });
        }
    }

    pub(crate) fn apply_prepared_raft_entry(
        &self,
        prepared: &mut RecordApplyGuard<'_>,
    ) -> anyhow::Result<super::ApplyOutcome> {
        anyhow::ensure!(
            prepared.runtime == self.changes.id,
            "prepared record belongs to another Engine"
        );
        anyhow::ensure!(
            !prepared.used.replace(true),
            "prepared record was already applied"
        );
        // Retain before dispatch: a normal validation error can follow earlier
        // mutations or metadata changes, and must not release their charge.
        self.changes.records.retain(prepared.charge.clone());
        self.dispatch_raft_entry(
            prepared
                .entry
                .take()
                .expect("prepared entry already applied"),
            Some(&prepared.charge),
            prepared.prepared_text.as_ref(),
        )
    }
}

/// One exact captured ownership cut. A background merge has no record cut.
pub(crate) struct RecordCut {
    records: super::record_charges::FrozenRecordCharges,
    budget: crate::change_budget::FrozenBatch,
}

impl Engine {
    pub(super) fn freeze_record_charges(&self) -> anyhow::Result<std::sync::Arc<RecordCut>> {
        let budget = self
            .changes
            .owner
            .freeze()
            .map_err(|error| anyhow::anyhow!("freeze pending record budget: {error:?}"))?;
        Ok(std::sync::Arc::new(RecordCut {
            records: self.changes.records.freeze(),
            budget,
        }))
    }

    /// Caller has durably published and bound this exact capture. The pending
    /// checkpoint and escaped row handles still retain their payload charges.
    pub(crate) fn acknowledge_record_charges(
        &self,
        capture: &super::CheckpointCapture,
    ) -> anyhow::Result<()> {
        if let Some(cut) = &capture.record_cut {
            self.changes
                .owner
                .publish_through(&cut.budget)
                .map_err(|error| anyhow::anyhow!("publish pending record budget: {error:?}"))?;
            self.changes.records.acknowledge_through(&cut.records);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CreateCollectionRequest, FieldSpec, FieldValue, IndexItem, IndexRequest};
    use std::collections::BTreeMap;

    fn engine(budget: &ChangeBudget) -> Engine {
        let mut engine = Engine::new();
        engine.changes.budget = budget.clone();
        engine.changes.owner = budget.owner();
        // These ownership units start from an existing schema baseline. The
        // direct API admission case below exercises the charged write route.
        engine
            .create_collection_inner(
                "c",
                CreateCollectionRequest {
                    fields: BTreeMap::from([(
                        "k".into(),
                        serde_json::from_str::<FieldSpec>(r#"{"type":"keyword"}"#).unwrap(),
                    )]),
                },
            )
            .unwrap();
        engine
    }

    fn entry() -> RaftLogEntry {
        RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: "one".into(),
                    field: "k".into(),
                    value: FieldValue::String("payload".repeat(100)),
                    version: None,
                }],
                request_id: None,
            },
        }
    }

    fn apply(engine: &Engine, entry: RaftLogEntry) {
        let reservation = engine.try_reserve_record(&entry, 0).unwrap();
        let mut guard = engine
            .begin_admitted_record(entry, reservation)
            .ok()
            .unwrap();
        engine.apply_prepared_raft_entry(&mut guard).unwrap();
    }

    #[test]
    fn admission_direct_engine_rejects_before_mutation_when_capacity_is_reserved() {
        let budget = ChangeBudget::with_hard_limit(1 << 20);
        let engine = engine(&budget);
        let remaining = (1 << 20) - budget.snapshot().total;
        let competing = budget.owner();
        let held = competing.try_reserve(remaining).unwrap();
        let RaftLogEntry::Index { req, .. } = entry() else {
            unreachable!()
        };
        let result = engine.index("c", req.clone());
        assert!(
            result.is_err(),
            "direct Engine mutation must reserve capacity before changing state"
        );
        assert!(result
            .unwrap_err()
            .downcast_ref::<crate::change_admission::PendingChangeCapacity>()
            .is_some());
        assert!(engine.state.read().unwrap().collections["c"]
            .interner
            .id("one")
            .is_none());
        drop(held);
        let before = budget.snapshot().active;
        engine.index("c", req).unwrap();
        assert!(
            budget.snapshot().active > before,
            "direct Engine changes must retain their actual pending charge"
        );
        let root = tempfile::tempdir().unwrap();
        let store = crate::segment_rdb::SegmentRdbStore::new(root.path()).unwrap();
        store.save(&std::sync::Arc::new(engine), 0).unwrap();
        assert_eq!(budget.snapshot().total, 0);
        let (cold, _) = store.load_latest().unwrap().unwrap();
        assert!(cold.state.read().unwrap().collections["c"]
            .interner
            .id("one")
            .is_some());
    }

    #[test]
    fn admission_stale_prepared_text_releases_the_discarded_reader_charge() {
        let budget = ChangeBudget::with_hard_limit(64 * 1024 * 1024);
        let engine = engine(&budget);
        engine
            .add_field(
                "c",
                "body",
                serde_json::from_str(r#"{"type":"text","analyzer":"ngram"}"#).unwrap(),
            )
            .unwrap();
        let entry = RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: "text".into(),
                    field: "body".into(),
                    value: FieldValue::String("ababa".into()),
                    version: None,
                }],
                request_id: None,
            },
        };
        let mut reserved = engine.try_reserve_record(&entry, 0).unwrap();
        reserved.staged_text = true;
        reserved
            .try_grow_to(engine.record_memory_bound(&entry, 0, true).unwrap())
            .unwrap();
        reserved.prepared_text = Some(engine.prepare_text_rows(&entry, &mut reserved).unwrap());
        assert!(reserved.preparation_bytes > 0);
        let once = reserved.preparation_bytes;
        engine
            .add_field(
                "c",
                "later",
                serde_json::from_str(r#"{"type":"keyword"}"#).unwrap(),
            )
            .unwrap();
        // Price one prepared row after the schema change. The cost table is
        // temporary; its reservation must not become retained data. A stale
        // preparation retry must retain exactly one reader charge.
        let expected = engine.record_memory_bound(&entry, 0, true).unwrap() + once;
        let mut apply = engine.begin_admitted_record(entry, reserved).ok().unwrap();
        assert_eq!(
            apply.charge.bytes(),
            expected,
            "discarded prepared Text rows must not accumulate reserved bytes on retry"
        );
        engine.apply_prepared_raft_entry(&mut apply).unwrap();
    }

    #[test]
    fn prepared_text_dispatch_keeps_payload_mapped_and_uses_sparse_document_rows() {
        let budget = ChangeBudget::with_hard_limit(64 * 1024 * 1024);
        let engine = engine(&budget);
        engine
            .add_field(
                "c",
                "body",
                serde_json::from_str(r#"{"type":"text","analyzer":"ngram"}"#).unwrap(),
            )
            .unwrap();
        // An unrelated field can establish a large runtime ID before Text is written.
        {
            let mut state = engine.state.write().unwrap();
            for id in 0..10_000 {
                state
                    .collections
                    .get_mut("c")
                    .unwrap()
                    .interner
                    .intern(&format!("unused-{id}"));
            }
        }
        let entry = RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: "last".into(),
                    field: "body".into(),
                    value: FieldValue::String("ababa".into()),
                    version: None,
                }],
                request_id: None,
            },
        };
        let mut reserved = engine.try_reserve_record(&entry, 0).unwrap();
        reserved.staged_text = true;
        let required = engine.record_memory_bound(&entry, 0, true).unwrap();
        reserved.wait_grow_to(required).unwrap();
        let mut guard = engine.begin_admitted_record(entry, reserved).ok().unwrap();
        engine.apply_prepared_raft_entry(&mut guard).unwrap();
        drop(guard);
        {
            let state = engine.state.read().unwrap();
            let super::super::FieldIndex::Text { idx, .. } = &state.collections["c"].fields["body"]
            else {
                unreachable!()
            };
            assert!(
                idx.tokens.is_empty(),
                "prepared Text must not rebuild normalized postings in RAM"
            );
            assert!(
                idx.lens.is_empty() && idx.distinct.is_empty(),
                "sparse Text must not allocate the global document ID prefix"
            );
            assert_eq!(idx.staged_rows.len(), 1);
            assert_eq!(idx.tok_postings("ab").unwrap().tfs(), &[2]);
            assert_eq!(idx.doc_len(10_000), 7);
        }
        engine
            .index(
                "c",
                IndexRequest {
                    items: vec![IndexItem {
                        external_id: "last".into(),
                        field: "body".into(),
                        value: FieldValue::String("bcdef".into()),
                        version: None,
                    }],
                    request_id: None,
                },
            )
            .unwrap();
        let state = engine.state.read().unwrap();
        let super::super::FieldIndex::Text { idx, .. } = &state.collections["c"].fields["body"]
        else {
            unreachable!()
        };
        assert!(idx.staged_rows.is_empty());
        assert!(idx.lens.is_empty() && idx.distinct.is_empty());
        assert_eq!(idx.delta_docs.len(), 1);
        assert!(idx.tok_postings("ab").is_none());
    }

    #[test]
    fn real_row_and_frozen_file_ownership_survives_durable_publication() {
        let budget = ChangeBudget::with_hard_limit(1 << 20);
        let engine = engine(&budget);
        apply(&engine, entry());
        let charged = budget.snapshot().total;
        assert!(charged > 700);
        let escaped = {
            let state = engine.state.read().unwrap();
            let frozen = state.collections["c"].change_journal.freeze();
            let value = frozen.rows().next().unwrap().2.value().unwrap().clone();
            value
        };
        let root = tempfile::tempdir().unwrap();
        let store = crate::segment_rdb::SegmentRdbStore::new(root.path()).unwrap();
        let engine = std::sync::Arc::new(engine);
        store.save(&engine, 0).unwrap();
        assert_eq!(
            budget.snapshot().total,
            charged,
            "an escaped row must retain its record charge after CURRENT"
        );
        drop(escaped);
        assert_eq!(
            budget.snapshot().total,
            0,
            "publication and last payload drop must free capacity"
        );
    }

    #[test]
    fn frozen_payload_keeps_capacity_after_metadata_acknowledgement() {
        let budget = ChangeBudget::with_hard_limit(1 << 20);
        let engine = engine(&budget);
        apply(&engine, entry());
        let charged = budget.snapshot().total;
        let frozen = engine.freeze_checkpoint_collections(None).unwrap();
        assert_eq!(budget.snapshot().active, 0);
        assert_eq!(budget.snapshot().frozen, charged);
        engine.acknowledge_record_charges(&frozen.capture).unwrap();
        drop(engine);
        assert_eq!(budget.snapshot().total, charged);
        drop(frozen);
        assert_eq!(budget.snapshot().total, 0);
    }

    #[test]
    fn a_reservation_cannot_be_applied_to_a_second_engine() {
        let budget = ChangeBudget::with_hard_limit(1 << 20);
        let first = engine(&budget);
        let second = engine(&budget);
        let entry = entry();
        let reservation = first.try_reserve_record(&entry, 0).unwrap();
        let charged = budget.snapshot().reserved;
        let rejected = second
            .begin_admitted_record(entry, reservation)
            .err()
            .unwrap();
        assert!(matches!(rejected.error, RecordAdmissionError::WrongEngine));
        assert_eq!(budget.snapshot().reserved, charged);
        assert!(second.state.read().unwrap().collections["c"]
            .interner
            .id("one")
            .is_none());
        drop(rejected);
        assert_eq!(budget.snapshot().reserved, 0);
    }
    #[test]
    fn replacement_adopts_charged_candidate_and_releases_old_live_metadata() {
        let budget = ChangeBudget::with_hard_limit(1 << 20);
        let active = std::sync::Arc::new(engine(&budget));
        let candidate = engine(&budget);
        apply(&active, entry());
        let old = budget.snapshot().total;
        apply(&candidate, entry());
        assert!(budget.snapshot().total > old);
        active.activate_replacement(candidate).unwrap();
        assert_eq!(
            budget.snapshot().total,
            old,
            "candidate remains charged; replaced live state has dropped"
        );
        let root = tempfile::tempdir().unwrap();
        let store = crate::segment_rdb::SegmentRdbStore::new(root.path()).unwrap();
        store.save(&active, 0).unwrap();
        assert_eq!(
            budget.snapshot().total,
            0,
            "live publication releases imported candidate metadata and row handles"
        );
    }

    #[test]
    fn restore_preserves_pending_reservation_and_retired_frozen_payload() {
        let budget = ChangeBudget::with_hard_limit(1 << 20);
        let active = engine(&budget);
        apply(&active, entry());
        let frozen = active.freeze_checkpoint_collections(None).unwrap();
        let old = budget.snapshot().frozen;
        let replacement = engine(&budget).snapshot().unwrap();
        let record = entry();
        let reserved = active.try_reserve_record(&record, 0).unwrap();
        let pending = reserved.bytes();
        active.restore(replacement).unwrap();
        assert_eq!(budget.snapshot().total, old + pending);
        let mut reprice = active
            .begin_admitted_record(record, reserved)
            .err()
            .unwrap();
        let required = reprice
            .required
            .expect("restored row needs a new interner and coverage entry");
        assert!(required > pending);
        assert_eq!(
            budget.snapshot().reserved,
            pending,
            "reprice retains its original reservation"
        );
        reprice.reservation.try_grow_to(required).unwrap();
        let mut apply = active
            .begin_admitted_record(reprice.entry, reprice.reservation)
            .ok()
            .unwrap();
        active.apply_prepared_raft_entry(&mut apply).unwrap();
        drop(apply);
        assert_eq!(budget.snapshot().total, old + required);
        drop(frozen);
        assert_eq!(
            budget.snapshot().total,
            required,
            "old capture releases only its original payload"
        );
    }
    #[test]
    fn raw_delivery_reservation_charges_then_releases_dropped_frame_bytes() {
        let entry = entry();
        let raw = Engine::record_owned_bytes(&entry).unwrap();
        let encoded = 4096;
        let budget = ChangeBudget::with_hard_limit(raw + encoded + 1);
        let engine = engine(&budget);
        let request = engine.record_ram_request(&entry, encoded).unwrap();
        let mut reservation = engine.try_reserve_record_ram(&request).unwrap();

        assert_eq!(
            budget.snapshot().reserved,
            raw + encoded,
            "the decoded entry and encoded frame coexist until replay drops the frame"
        );
        reservation.release_transport_bytes().unwrap();
        assert_eq!(
            budget.snapshot().reserved,
            raw,
            "only the explicitly dropped encoded frame may release capacity"
        );
        drop(reservation);
        assert_eq!(budget.snapshot().reserved, 0);
    }

    // A generous test-only workspace margin keeps this admission test independent
    // of the private table layout. The available budget still stays below the old
    // all-window estimate.
    const NGRAM_TEST_WORKSPACE_HEADROOM: usize = 64 * 1024;

    #[test]
    fn repeated_default_ngram_admits_exact_bound_plus_reserved_workspace() {
        const HARD: usize = 512 * 1024;
        let budget = ChangeBudget::with_hard_limit(HARD);
        let engine = engine(&budget);
        engine
            .add_field(
                "c",
                "body",
                serde_json::from_str(r#"{"type":"text","analyzer":"ngram"}"#).unwrap(),
            )
            .unwrap();
        let input = "durable search token ".repeat(16);
        let entry = RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: "repeated".into(),
                    field: "body".into(),
                    value: FieldValue::String(input.clone()),
                    version: None,
                }],
                request_id: None,
            },
        };
        let old_bound = engine.record_memory_bound(&entry, 0, false).unwrap();

        // Derive the intended Text inputs independently from the shared stream.
        // Do not duplicate tokenizer windows or cost multipliers in this test.
        let mut unique = std::collections::BTreeSet::new();
        crate::ngram_stream::stream_default_ngrams(&input, |token| {
            unique.insert(token.as_bytes().to_vec());
            Ok::<_, ()>(())
        })
        .unwrap();
        let exact_field = crate::change_memory_cost::FieldCost::Text {
            distinct_terms: unique.len(),
            total_term_bytes: unique.iter().map(Vec::len).sum(),
        };
        let index =
            crate::change_memory_cost::estimate_change(&crate::change_memory_cost::Change::Index {
                external_id_bytes: "repeated".len(),
                new_document: true,
                field: exact_field,
                volatile_metadata_bytes: 0,
            })
            .unwrap();
        let direct = crate::change_memory_cost::estimate_change(
            &crate::change_memory_cost::Change::Direct {
                // Existing record-cost contract: field bytes plus direct metadata.
                metadata_bytes: "body".len() + 96,
            },
        )
        .unwrap();
        let exact_bound =
            Engine::record_owned_bytes(&entry).unwrap() + index.total() + direct.total();
        let available = exact_bound + NGRAM_TEST_WORKSPACE_HEADROOM;
        assert!(
            available < old_bound,
            "fixture including workspace must fit below old all-window bound"
        );
        let baseline = budget.snapshot().total;
        assert!(baseline + available < HARD);
        let held = budget
            .owner()
            .try_reserve(HARD - baseline - available)
            .unwrap();

        // Baseline fails here because it reserves `old_bound`. The implemented
        // route must reserve raw+workspace, exact-price under that reservation,
        // then retain `exact_bound` only.
        let reservation = engine.try_reserve_record(&entry, 0);
        assert!(
            reservation.is_ok(),
            "exact Ngram admission must fit below old bound"
        );
        drop(reservation);
        drop(held);
    }
    fn ngram_entry(engine: &Engine) -> RaftLogEntry {
        engine
            .add_field(
                "c",
                "body",
                serde_json::from_str(r#"{"type":"text","analyzer":"ngram"}"#).unwrap(),
            )
            .unwrap();
        RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: "ngram-document".into(),
                    field: "body".into(),
                    value: FieldValue::String("durable search token ".repeat(16)),
                    version: None,
                }],
                request_id: None,
            },
        }
    }

    #[test]
    fn exact_ngram_local_admission_reserves_full_normalized_cost_before_return() {
        const HARD: usize = 512 * 1024;
        let budget = ChangeBudget::with_hard_limit(HARD);
        let engine = engine(&budget);
        let entry = ngram_entry(&engine);
        let accepted = engine.try_reserve_record(&entry, 0).unwrap();
        let full = accepted.bytes();
        let floor = Engine::record_owned_bytes(&entry).unwrap()
            + crate::change_record_cost::DEFAULT_NGRAM_COST_WORKSPACE_BYTES;
        assert!(
            full > floor + 1,
            "raw input and workspace alone cannot cover normalized changes"
        );
        drop(accepted);
        let baseline = budget.snapshot().total;
        let held = budget
            .owner()
            .try_reserve(HARD - baseline - floor - 1)
            .unwrap();
        let before = budget.snapshot();
        let result = engine.try_reserve_record(&entry, 0);
        assert!(
            matches!(
                result,
                Err(RecordAdmissionError::Capacity(AdmissionError::Full { .. }))
            ),
            "local admission must refuse before publication when only raw input and workspace fit"
        );
        assert_eq!(
            budget.snapshot().total,
            before.total,
            "failed admission must release all temporary reservation"
        );
        assert_eq!(budget.snapshot().reserved, before.reserved);
        assert!(engine.state.read().unwrap().collections["c"]
            .interner
            .id("ngram-document")
            .is_none());
        drop(held);
    }

    #[test]
    fn exact_ngram_decoded_local_admission_prices_changes_before_publication() {
        const HARD: usize = 512 * 1024;
        let budget = ChangeBudget::with_hard_limit(HARD);
        let engine = engine(&budget);
        let entry = ngram_entry(&engine);
        let raw = Engine::record_owned_bytes(&entry).unwrap();
        let workspace = crate::change_record_cost::DEFAULT_NGRAM_COST_WORKSPACE_BYTES;
        let accepted = engine.try_reserve_record(&entry, 0).unwrap();
        assert!(accepted.bytes() > raw + workspace + 1);
        drop(accepted);
        let baseline = budget.snapshot().total;
        let request = engine.record_ram_request(&entry, 0).unwrap();
        let mut reserved = engine.try_reserve_record_ram(&request).unwrap();
        let held = budget
            .owner()
            .try_reserve(HARD - budget.snapshot().total - workspace - 1)
            .unwrap();
        let held_bytes = held.bytes();
        let result = engine.price_decoded_record(&entry, &mut reserved, raw, 0, true);
        assert!(matches!(result, Err(RecordAdmissionError::Capacity(AdmissionError::Full { .. }))),
            "decoded local admission must not return success after reserving only decoder and table bytes");
        drop(reserved);
        assert_eq!(budget.snapshot().total, baseline + held_bytes);
        assert!(engine.state.read().unwrap().collections["c"]
            .interner
            .id("ngram-document")
            .is_none());
        drop(held);
    }

    #[test]
    fn exact_ngram_committed_workspace_wait_retains_source_and_releases_scratch_before_apply() {
        const HARD: usize = 512 * 1024;
        let budget = ChangeBudget::with_hard_limit(HARD);
        let engine = engine(&budget);
        let entry = ngram_entry(&engine);
        let raw = Engine::record_owned_bytes(&entry).unwrap();
        let workspace = crate::change_record_cost::DEFAULT_NGRAM_COST_WORKSPACE_BYTES;
        let accepted = engine.try_reserve_record(&entry, 0).unwrap();
        let final_cost = accepted.bytes() - workspace;
        drop(accepted);
        let baseline = budget.snapshot().total;
        let request = engine.record_ram_request(&entry, 0).unwrap();
        let reserved = engine.try_reserve_record_ram(&request).unwrap();
        assert_eq!(reserved.bytes(), raw);
        let held = budget
            .owner()
            .try_reserve(HARD - budget.snapshot().total - workspace + 1)
            .unwrap();
        let mut reprice = engine
            .begin_admitted_record(entry, reserved)
            .err()
            .expect("unreserved table must return source ownership before it is constructed");
        assert_eq!(reprice.required, Some(raw + workspace));
        assert_eq!(reprice.reservation.bytes(), raw);
        assert_eq!(reprice.reservation.ngram_cost_workspace_bytes, 0);
        assert!(matches!(
            reprice.error,
            RecordAdmissionError::Capacity(AdmissionError::Full { .. })
        ));
        // A capture can start while the caller owns the returned source. No
        // capacity wait may retain the apply boundary.
        let snapshot = engine.snapshot().unwrap();
        drop(snapshot);
        drop(held);
        reprice
            .reservation
            .try_grow_to(reprice.required.unwrap())
            .unwrap();
        let mut reprice = engine
            .begin_admitted_record(reprice.entry, reprice.reservation)
            .err()
            .expect("raw plus table still needs the whole normalized change price");
        assert_eq!(reprice.required, Some(final_cost + workspace));
        reprice
            .reservation
            .try_grow_to(reprice.required.unwrap())
            .unwrap();
        let mut guard = engine
            .begin_admitted_record(reprice.entry, reprice.reservation)
            .ok()
            .unwrap();
        assert_eq!(
            guard.charge.bytes(),
            final_cost,
            "temporary cost table must not remain in retained changes"
        );
        assert_eq!(budget.snapshot().reserved, 0);
        assert_eq!(budget.snapshot().total, baseline + final_cost);
        engine.apply_prepared_raft_entry(&mut guard).unwrap();
        drop(guard);
        assert!(engine.state.read().unwrap().collections["c"]
            .interner
            .id("ngram-document")
            .is_some());
    }

    /// Poison the schema/coverage lock so `estimate_record_cost` and
    /// `estimate_record_exact_default_ngram_cost` cannot resolve context and
    /// return `RecordEstimate::Retain` (#lumen-pending-change-capacity-door).
    fn poison_state(engine: &Engine) {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = engine.state.write().unwrap();
            panic!("poison state for test");
        }));
        assert!(result.is_err(), "poisoning thread must panic");
        assert!(engine.state.is_poisoned());
    }

    /// Door pricing that needs context it cannot resolve (a poisoned lock
    /// stands in for "unknown schema/live coverage") must never publish a
    /// record with zero reservation. It must reserve at least the
    /// raw+transport bound apply already knows how to charge — the same
    /// bound `raft_sm::decode_admitted` reserves before it decodes — so a
    /// later local `submit()` never reaches the WAL delivery loop's
    /// unbounded `wait_reserve_record_ram` with no reservation to grow.
    #[test]
    fn try_reserve_record_reserves_the_minimal_bound_when_pricing_needs_context() {
        let budget = ChangeBudget::with_hard_limit(1 << 20);
        let engine = engine(&budget);
        poison_state(&engine);

        let entry = entry();
        let raw = Engine::record_owned_bytes(&entry).unwrap();
        let reservation = engine.try_reserve_record(&entry, 0).unwrap_or_else(|error| {
            panic!(
                "a record whose exact price needs unavailable context must still be \
                 reserved at its known minimal bound before publication, not left \
                 unreserved: {error:?}"
            )
        });
        assert_eq!(
            reservation.bytes(),
            raw,
            "the minimal reservation must match the raw+transport bound, the same \
             floor apply uses before it can price exactly"
        );
    }

    /// The same context-missing pricing failure, but with capacity already
    /// full even for the minimal raw+transport bound: the door must refuse
    /// with a capacity error (#429 upstream) before publication rather than
    /// publish the record with no reservation and let it hang later.
    #[test]
    fn try_reserve_record_refuses_before_publication_when_even_the_minimal_bound_is_full() {
        let budget = ChangeBudget::with_hard_limit(1 << 20);
        let engine = engine(&budget);
        poison_state(&engine);

        let remaining = (1 << 20) - budget.snapshot().total;
        let competing = budget.owner();
        let _held = competing.try_reserve(remaining).unwrap();

        let entry = entry();
        let error = match engine.try_reserve_record(&entry, 0) {
            Ok(_) => panic!("a full budget must not silently publish an unreserved record"),
            Err(error) => error,
        };
        assert!(
            matches!(error, RecordAdmissionError::Capacity(AdmissionError::Full { .. })),
            "a Full minimal bound must classify as capacity, not a bare domain error \
             a caller would otherwise treat as safe to publish unreserved: {error:?}"
        );
    }
}
