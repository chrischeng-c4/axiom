//! A checkpoint cut excludes whole synchronous apply records. Nested Engine
//! methods reuse the current thread's apply interval. Readers need no lease.
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Condvar, Mutex};
use std::thread::{self, ThreadId};
use std::time::Instant;

#[derive(Default)]
pub(crate) struct CaptureBarrier {
    state: Mutex<State>,
    changed: Condvar,
}

#[derive(Default)]
struct State {
    owner: Option<ThreadId>,
    depth: usize,
    capturing: bool,
    publication_pins: usize,
    #[cfg(test)]
    publication_wait_observer: Option<std::sync::mpsc::Sender<()>>,
    waiting_captures: usize,
    sequence: Option<u64>,
    epoch: u64,
    apply_revision: u64,
    restore_inhibitions: usize,
    uncertain: bool,
    apply_started: Option<Instant>,
    timing: CaptureTiming,
}

/// Completed lease intervals. Nested apply guards form one record interval.
/// Wait time is deliberately excluded: these values measure how long work
/// holds the apply/capture boundary after admission.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HoldTiming {
    pub(crate) count: u64,
    pub(crate) total_ns: u64,
    pub(crate) max_ns: u64,
}
impl HoldTiming {
    fn record(&mut self, started: Instant) {
        let elapsed = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
        self.count = self.count.saturating_add(1);
        self.total_ns = self.total_ns.saturating_add(elapsed);
        self.max_ns = self.max_ns.max(elapsed);
    }
}
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CaptureTiming {
    pub(crate) apply: HoldTiming,
    pub(crate) capture: HoldTiming,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CaptureStamp {
    pub(crate) sequence: u64,
    pub(crate) epoch: u64,
}

/// A process-local identity for the live mutation state. A cache seal records
/// this under the writer fence and may only be reused while it is unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MutationStamp {
    pub(crate) epoch: u64,
    pub(crate) apply_revision: u64,
}

pub(crate) struct ApplyLease<'a> {
    barrier: &'a CaptureBarrier,
    _thread_affine: PhantomData<Rc<()>>,
}
pub(crate) struct CaptureLease<'a> {
    barrier: &'a CaptureBarrier,
    stamp: CaptureStamp,
    started: Instant,
    _thread_affine: PhantomData<Rc<()>>,
}
/// Keeps a validated capture epoch stable while durable pointer I/O runs.
/// Unlike [`CaptureLease`], this does not exclude ordinary apply.
pub(crate) struct PublicationPin<'a> {
    barrier: &'a CaptureBarrier,
}
/// Refuses captures while a durable restore owns the interval between
/// publishing CURRENT and activating the replacement. The guard has no
/// apply lease and is Send because it borrows a Sync CaptureBarrier.
pub(crate) struct RestoreInhibition<'a> {
    barrier: &'a CaptureBarrier,
}

impl CaptureBarrier {
    fn wait_for_publication_pins<'a>(
        &self,
        mut state: std::sync::MutexGuard<'a, State>,
    ) -> std::sync::MutexGuard<'a, State> {
        while state.publication_pins != 0 {
            #[cfg(test)]
            if let Some(observer) = &state.publication_wait_observer {
                observer
                    .send(())
                    .expect("publication wait observer dropped");
            }
            state = self.changed.wait(state).expect("capture barrier poisoned");
        }
        state
    }
    /// Read the replacement epoch without acquiring an apply or capture lease.
    /// Pending checkpoint ownership uses this only to discard a detached cut
    /// that a completed restore has already made impossible to publish.
    pub(crate) fn epoch(&self) -> u64 {
        self.state.lock().expect("capture barrier poisoned").epoch
    }

    /// Preparation observes this while holding the Engine state read lock.
    /// Recheck under apply and state write before publishing detached work.
    /// Capture completion also advances it: checkpoint binding may replace a
    /// reader view without changing the collection's document data version.
    pub(crate) fn apply_revision(&self) -> u64 {
        self.state
            .lock()
            .expect("capture barrier poisoned")
            .apply_revision
    }

    /// Read both cache-seal invalidation values while holding one state lock.
    /// Separate reads could combine an old epoch with a new apply revision.
    pub(crate) fn mutation_stamp(&self) -> MutationStamp {
        let state = self.state.lock().expect("capture barrier poisoned");
        MutationStamp {
            epoch: state.epoch,
            apply_revision: state.apply_revision,
        }
    }

    pub(crate) fn is_uncertain(&self) -> bool {
        self.state
            .lock()
            .expect("capture barrier poisoned")
            .uncertain
    }

    pub(crate) fn timing(&self) -> CaptureTiming {
        self.state.lock().expect("capture barrier poisoned").timing
    }

    pub(crate) fn apply(&self) -> ApplyLease<'_> {
        self.acquire_apply(false)
    }

    /// Acquire an apply interval. Only a [`RestoreInhibition`] may set
    /// `allow_restore_inhibition`: this prevents ordinary WAL apply from
    /// mutating old live state after durable `CURRENT` publication, while
    /// still letting the restoring task activate its replacement.
    fn acquire_apply(&self, allow_restore_inhibition: bool) -> ApplyLease<'_> {
        let owner = thread::current().id();
        let mut state = self.state.lock().expect("capture barrier poisoned");
        loop {
            if state.owner == Some(owner) {
                state.depth = state.depth.checked_add(1).expect("apply depth overflow");
                break;
            }
            if state.owner.is_none()
                && !state.capturing
                && state.waiting_captures == 0
                && (allow_restore_inhibition || state.restore_inhibitions == 0)
            {
                state.owner = Some(owner);
                state.depth = 1;
                state.apply_started = Some(Instant::now());
                break;
            }
            state = self.changed.wait(state).expect("capture barrier poisoned");
        }
        ApplyLease {
            barrier: self,
            _thread_affine: PhantomData,
        }
    }

    pub(crate) fn capture(&self, fallback: u64) -> Result<CaptureLease<'_>, &'static str> {
        let mut state = self.state.lock().expect("capture barrier poisoned");
        assert_ne!(
            state.owner,
            Some(thread::current().id()),
            "capture inside apply interval"
        );
        if state.restore_inhibitions != 0 {
            return Err("checkpoint refused: durable restore is in progress");
        }
        state.waiting_captures = state
            .waiting_captures
            .checked_add(1)
            .expect("capture queue overflow");
        self.changed.notify_all();
        while state.owner.is_some() || state.capturing {
            state = self.changed.wait(state).expect("capture barrier poisoned");
        }
        state.waiting_captures -= 1;
        // A restore can consume the apply lease that we were waiting behind.
        // Recheck before granting this capture so a queued caller cannot cross
        // its publication inhibition.
        if state.restore_inhibitions != 0 {
            self.changed.notify_all();
            return Err("checkpoint refused: durable restore is in progress");
        }
        if state.uncertain {
            self.changed.notify_all();
            return Err("checkpoint refused: durability is uncertain; restart required");
        }
        state.capturing = true;
        let stamp = CaptureStamp {
            sequence: state.sequence.unwrap_or(fallback),
            epoch: state.epoch,
        };
        Ok(CaptureLease {
            barrier: self,
            stamp,
            started: Instant::now(),
            _thread_affine: PhantomData,
        })
    }
}

impl<'a> ApplyLease<'a> {
    pub(crate) fn initialize_sequence(&self, sequence: u64) {
        let mut state = self.barrier.state.lock().expect("capture barrier poisoned");
        if let Some(old) = state.sequence {
            assert_eq!(old, sequence, "applied sequence initialization mismatch");
        }
        state.sequence = Some(sequence);
    }
    pub(crate) fn advance_sequence(&self, sequence: u64) {
        let mut state = self.barrier.state.lock().expect("capture barrier poisoned");
        assert!(
            state.sequence.is_none_or(|old| sequence >= old),
            "applied sequence regressed"
        );
        state.sequence = Some(sequence);
    }
    pub(crate) fn replace_epoch(&self) {
        let mut state = self.barrier.state.lock().expect("capture barrier poisoned");
        state = self.barrier.wait_for_publication_pins(state);
        state.epoch = state.epoch.checked_add(1).expect("engine epoch overflow");
        state.sequence = None;
    }
    pub(crate) fn mark_uncertain(&self) {
        self.barrier
            .state
            .lock()
            .expect("capture barrier poisoned")
            .uncertain = true;
    }
    /// Bump the engine epoch while preserving its applied sequence. No capture
    /// can be active because this consumes the apply lease; the returned guard
    /// then releases apply before durable I/O starts.
    pub(crate) fn inhibit_checkpoints_for_restore(self) -> RestoreInhibition<'a> {
        let barrier = self.barrier;
        {
            let mut state = barrier.state.lock().expect("capture barrier poisoned");
            state = barrier.wait_for_publication_pins(state);
            state.epoch = state.epoch.checked_add(1).expect("engine epoch overflow");
            state.restore_inhibitions = state
                .restore_inhibitions
                .checked_add(1)
                .expect("restore inhibition overflow");
        }
        drop(self);
        RestoreInhibition { barrier }
    }
}
impl<'a> RestoreInhibition<'a> {
    /// Take the short apply interval that installs the already-prepared
    /// replacement. This is the only path allowed through this inhibition.
    pub(crate) fn activation_apply(&self) -> ApplyLease<'_> {
        self.barrier.acquire_apply(true)
    }

    /// Latch uncertainty without acquiring ordinary apply, which is blocked
    /// until this restore inhibition drops.
    pub(crate) fn mark_uncertain(&self) {
        self.barrier
            .state
            .lock()
            .expect("capture barrier poisoned")
            .uncertain = true;
    }
}
impl Drop for ApplyLease<'_> {
    fn drop(&mut self) {
        let mut state = self.barrier.state.lock().expect("capture barrier poisoned");
        assert_eq!(state.owner, Some(thread::current().id()));
        if thread::panicking() {
            state.uncertain = true;
        }
        state.depth -= 1;
        if state.depth == 0 {
            state.apply_revision = state
                .apply_revision
                .checked_add(1)
                .expect("capture barrier preparation revision exhausted");
            let started = state.apply_started.take().expect("apply start missing");
            state.timing.apply.record(started);
            state.owner = None;
            self.barrier.changed.notify_all();
        }
    }
}
impl<'a> CaptureLease<'a> {
    pub(crate) fn stamp(&self) -> CaptureStamp {
        self.stamp
    }
    /// Check this capture against the original cut and retain only an epoch
    /// pin for durable publication. Drop this lease before filesystem I/O.
    pub(crate) fn publication_pin(
        &self,
        captured: CaptureStamp,
    ) -> Result<PublicationPin<'a>, &'static str> {
        let mut state = self.barrier.state.lock().expect("capture barrier poisoned");
        if self.stamp.epoch != captured.epoch {
            return Err("checkpoint refused: engine was restored after capture");
        }
        state.publication_pins = state
            .publication_pins
            .checked_add(1)
            .ok_or("checkpoint publication pin overflow")?;
        Ok(PublicationPin {
            barrier: self.barrier,
        })
    }

    /// Check a short binding capture after durable publication. Later apply
    /// intervals may advance the cut, but only a restore invalidates it.
    pub(crate) fn validate_publish(&self, captured: CaptureStamp) -> Result<(), &'static str> {
        if self.stamp.epoch != captured.epoch {
            Err("checkpoint refused: engine was restored after capture")
        } else {
            Ok(())
        }
    }
}
impl Drop for RestoreInhibition<'_> {
    fn drop(&mut self) {
        let mut state = self.barrier.state.lock().expect("capture barrier poisoned");
        if thread::panicking() {
            state.uncertain = true;
        }
        state.restore_inhibitions = state
            .restore_inhibitions
            .checked_sub(1)
            .expect("restore inhibition underflow");
        self.barrier.changed.notify_all();
    }
}
impl Drop for CaptureLease<'_> {
    fn drop(&mut self) {
        let mut state = self.barrier.state.lock().expect("capture barrier poisoned");
        assert!(state.capturing);
        if thread::panicking() {
            state.uncertain = true;
        }
        state.timing.capture.record(self.started);
        state.apply_revision = state
            .apply_revision
            .checked_add(1)
            .expect("capture barrier preparation revision exhausted");
        state.capturing = false;
        self.barrier.changed.notify_all();
    }
}
impl Drop for PublicationPin<'_> {
    fn drop(&mut self) {
        let mut state = self.barrier.state.lock().expect("capture barrier poisoned");
        if thread::panicking() {
            state.uncertain = true;
        }
        state.publication_pins = state
            .publication_pins
            .checked_sub(1)
            .expect("checkpoint publication pin underflow");
        self.barrier.changed.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{mpsc, Arc};
    use std::time::Duration;
    const LIMIT: Duration = Duration::from_secs(2);

    #[test]
    fn timing_counts_one_completed_apply_interval_for_nested_guards() {
        let barrier = CaptureBarrier::default();
        let outer = barrier.apply();
        let inner = barrier.apply();
        assert_eq!(barrier.timing().apply.count, 0);
        drop(outer);
        assert_eq!(barrier.timing().apply.count, 0);
        drop(inner);
        let timing = barrier.timing();
        assert_eq!(timing.apply.count, 1);
        assert_eq!(timing.apply.total_ns, timing.apply.max_ns);
        assert_eq!(timing.capture, HoldTiming::default());
    }

    #[test]
    fn timing_records_only_granted_completed_captures() {
        let barrier = CaptureBarrier::default();
        let capture = barrier.capture(0).unwrap();
        assert_eq!(barrier.timing().capture.count, 0);
        drop(capture);
        assert_eq!(barrier.timing().capture.count, 1);
        let inhibition = barrier.apply().inhibit_checkpoints_for_restore();
        assert!(barrier.capture(0).is_err());
        assert_eq!(barrier.timing().capture.count, 1);
        drop(inhibition);
        drop(barrier.capture(0).unwrap());
        let timing = barrier.timing();
        assert_eq!(timing.capture.count, 2);
        assert!(timing.capture.total_ns >= timing.capture.max_ns);
    }

    #[test]
    fn mutation_stamp_reads_epoch_and_apply_revision_together() {
        let barrier = CaptureBarrier::default();
        assert_eq!(
            barrier.mutation_stamp(),
            MutationStamp {
                epoch: 0,
                apply_revision: 0,
            }
        );
        drop(barrier.apply());
        assert_eq!(
            barrier.mutation_stamp(),
            MutationStamp {
                epoch: 0,
                apply_revision: 1,
            }
        );
        barrier.apply().replace_epoch();
        assert_eq!(
            barrier.mutation_stamp(),
            MutationStamp {
                epoch: 1,
                apply_revision: 2,
            }
        );
    }

    #[test]
    fn nested_apply_finishes_a_record_before_queued_capture() {
        let barrier = Arc::new(CaptureBarrier::default());
        let apply = barrier.apply();
        apply.initialize_sequence(8);
        let other = barrier.clone();
        let (tx, rx) = mpsc::channel();
        let capture = thread::spawn(move || tx.send(other.capture(0).unwrap().stamp()).unwrap());
        // Wait for the actual queued capture, not merely a thread start.
        let mut state = barrier.state.lock().unwrap();
        while state.waiting_captures == 0 {
            let (next, timeout) = barrier.changed.wait_timeout(state, LIMIT).unwrap();
            assert!(!timeout.timed_out(), "capture never queued");
            state = next;
        }
        drop(state);
        let nested = barrier.apply();
        nested.advance_sequence(9);
        drop(nested);
        assert!(rx.try_recv().is_err(), "capture crossed unfinished record");
        drop(apply);
        assert_eq!(rx.recv_timeout(LIMIT).unwrap().sequence, 9);
        capture.join().unwrap();
    }

    #[test]
    fn capture_excludes_new_apply() {
        let barrier = Arc::new(CaptureBarrier::default());
        let capture = barrier.capture(17).unwrap();
        let other = barrier.clone();
        let (started_tx, started_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let apply = thread::spawn(move || {
            started_tx.send(()).unwrap();
            let _apply = other.apply();
            done_tx.send(()).unwrap();
        });
        started_rx.recv_timeout(LIMIT).unwrap();
        assert!(done_rx.recv_timeout(Duration::from_millis(20)).is_err());
        drop(capture);
        done_rx.recv_timeout(LIMIT).unwrap();
        apply.join().unwrap();
    }

    #[test]
    fn publication_rejects_stamp_from_before_restore() {
        let barrier = CaptureBarrier::default();
        let old = barrier.capture(4).unwrap().stamp();
        barrier.apply().replace_epoch();
        let publication = barrier.capture(5).unwrap();
        assert!(publication.validate_publish(old).is_err());
    }

    #[test]
    fn publication_pin_allows_apply_and_wakes_epoch_replacement() {
        let barrier = Arc::new(CaptureBarrier::default());
        let initialization = barrier.apply();
        initialization.initialize_sequence(9);
        drop(initialization);
        let capture = barrier.capture(4).unwrap();
        let pin = capture.publication_pin(capture.stamp()).unwrap();
        drop(capture);

        let apply_barrier = barrier.clone();
        let (apply_tx, apply_rx) = mpsc::channel();
        let apply = thread::spawn(move || {
            let _apply = apply_barrier.apply();
            apply_tx.send(()).unwrap();
        });
        apply_rx.recv_timeout(LIMIT).unwrap();
        apply.join().unwrap();

        let (wait_tx, wait_rx) = mpsc::channel();
        barrier.state.lock().unwrap().publication_wait_observer = Some(wait_tx);
        let replacement_barrier = barrier.clone();
        let (done_tx, done_rx) = mpsc::channel();
        let replacement = thread::spawn(move || {
            let apply = replacement_barrier.apply();
            apply.replace_epoch();
            done_tx.send(()).unwrap();
        });
        wait_rx.recv_timeout(LIMIT).unwrap();
        let state = barrier.state.lock().unwrap();
        assert_eq!(
            state.epoch, 0,
            "epoch changed before the publication pin dropped"
        );
        assert_eq!(state.sequence, Some(9));
        assert!(
            done_rx.try_recv().is_err(),
            "epoch replacement crossed a live publication pin"
        );
        drop(state);
        drop(pin);
        done_rx.recv_timeout(LIMIT).unwrap();
        replacement.join().unwrap();
        let state = barrier.state.lock().unwrap();
        assert_eq!(state.epoch, 1);
        assert_eq!(
            state.sequence, None,
            "epoch replacement must clear the sequence"
        );
        assert_eq!(state.publication_pins, 0);
    }

    #[test]
    fn publication_pin_blocks_restore_inhibition_until_drop() {
        let barrier = Arc::new(CaptureBarrier::default());
        let initialization = barrier.apply();
        initialization.initialize_sequence(9);
        drop(initialization);
        let capture = barrier.capture(4).unwrap();
        let pin = capture.publication_pin(capture.stamp()).unwrap();
        drop(capture);
        let (wait_tx, wait_rx) = mpsc::channel();
        barrier.state.lock().unwrap().publication_wait_observer = Some(wait_tx);
        let other = barrier.clone();
        let (done_tx, done_rx) = mpsc::channel();
        let restore = thread::spawn(move || {
            let apply = other.apply();
            let inhibition = apply.inhibit_checkpoints_for_restore();
            done_tx.send(()).unwrap();
            drop(inhibition);
        });
        wait_rx.recv_timeout(LIMIT).unwrap();
        let state = barrier.state.lock().unwrap();
        assert_eq!(
            state.epoch, 0,
            "restore changed epoch before the publication pin dropped"
        );
        assert_eq!(state.restore_inhibitions, 0);
        assert_eq!(
            state.sequence,
            Some(9),
            "restore inhibition must preserve sequence"
        );
        assert!(
            done_rx.try_recv().is_err(),
            "restore inhibition crossed a live publication pin"
        );
        drop(state);
        drop(pin);
        done_rx.recv_timeout(LIMIT).unwrap();
        restore.join().unwrap();
        let state = barrier.state.lock().unwrap();
        assert_eq!(state.epoch, 1);
        assert_eq!(state.restore_inhibitions, 0);
        assert_eq!(state.sequence, Some(9));
        assert_eq!(state.publication_pins, 0);
    }

    #[test]
    fn publication_pin_rejects_a_stamp_invalidated_before_conversion() {
        let barrier = CaptureBarrier::default();
        let capture = barrier.capture(4).unwrap();
        let stamp = capture.stamp();
        drop(capture);
        barrier.apply().replace_epoch();
        let publication = barrier.capture(5).unwrap();
        assert!(publication.publication_pin(stamp).is_err());
    }

    #[test]
    fn panicking_publication_pin_latches_uncertainty() {
        fn assert_send<T: Send>() {}
        assert_send::<PublicationPin<'_>>();

        let barrier = Arc::new(CaptureBarrier::default());
        let capture = barrier.capture(4).unwrap();
        let pin = capture.publication_pin(capture.stamp()).unwrap();
        drop(capture);
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _pin = pin;
            panic!("pointer publication panicked");
        }))
        .is_err());
        assert!(barrier.is_uncertain());
        assert_eq!(barrier.state.lock().unwrap().publication_pins, 0);
    }

    #[test]
    fn publication_pin_overflow_refuses_without_changing_the_count() {
        let barrier = CaptureBarrier::default();
        let capture = barrier.capture(4).unwrap();
        {
            let mut state = barrier.state.lock().unwrap();
            state.publication_pins = usize::MAX;
        }
        assert_eq!(
            capture.publication_pin(capture.stamp()).err(),
            Some("checkpoint publication pin overflow")
        );
        assert_eq!(barrier.state.lock().unwrap().publication_pins, usize::MAX);
        barrier.state.lock().unwrap().publication_pins = 0;
    }

    #[test]
    fn ordinary_apply_keeps_epoch_and_advances_capture_cut() {
        let barrier = CaptureBarrier::default();
        let old = barrier.capture(4).unwrap().stamp();
        barrier.apply().advance_sequence(6);
        let publication = barrier.capture(99).unwrap();
        assert_eq!(publication.stamp().sequence, 6);
        assert!(publication.validate_publish(old).is_ok());
    }

    #[test]
    fn uncertain_apply_blocks_capture_even_after_restore() {
        let barrier = CaptureBarrier::default();
        let apply = barrier.apply();
        apply.mark_uncertain();
        apply.replace_epoch();
        drop(apply);
        assert!(barrier.capture(4).is_err());
    }
    #[test]
    fn panic_in_apply_latches_uncertainty_before_lease_opens() {
        let barrier = Arc::new(CaptureBarrier::default());
        let other = barrier.clone();
        assert!(thread::spawn(move || {
            let _apply = other.apply();
            panic!("partial apply");
        })
        .join()
        .is_err());
        assert!(barrier.capture(4).is_err());
    }

    #[test]
    fn queued_capture_refuses_when_held_apply_becomes_restore_inhibition() {
        let barrier = Arc::new(CaptureBarrier::default());
        let apply = barrier.apply();
        apply.initialize_sequence(8);
        let other = barrier.clone();
        let (result_tx, result_rx) = mpsc::channel();
        let capture = thread::spawn(move || {
            let result = other.capture(0).map(|_| ()).map_err(str::to_owned);
            result_tx.send(result).unwrap();
        });
        let mut state = barrier.state.lock().unwrap();
        while state.waiting_captures == 0 {
            let (next, timeout) = barrier.changed.wait_timeout(state, LIMIT).unwrap();
            assert!(!timeout.timed_out(), "capture never queued");
            state = next;
        }
        drop(state);

        let inhibition = apply.inhibit_checkpoints_for_restore();
        assert_eq!(
            result_rx.recv_timeout(LIMIT).unwrap().err().unwrap(),
            "checkpoint refused: durable restore is in progress"
        );
        capture.join().unwrap();
        drop(inhibition);
    }

    #[test]
    fn restore_inhibition_keeps_apply_open_preserves_sequence_and_clears_on_drop() {
        fn assert_send<T: Send>() {}
        assert_send::<RestoreInhibition<'_>>();

        let barrier = CaptureBarrier::default();
        let initial = barrier.apply();
        initial.initialize_sequence(8);
        let inhibition = initial.inhibit_checkpoints_for_restore();
        let apply = inhibition.activation_apply();
        apply.advance_sequence(9);
        drop(apply);
        assert_eq!(
            barrier.capture(0).err().unwrap(),
            "checkpoint refused: durable restore is in progress"
        );
        drop(inhibition);
        assert_eq!(barrier.capture(0).unwrap().stamp().sequence, 9);
    }

    #[test]
    fn restore_inhibition_blocks_ordinary_apply_but_allows_its_activation_lease() {
        let barrier = Arc::new(CaptureBarrier::default());
        let inhibition = barrier.apply().inhibit_checkpoints_for_restore();
        let other = barrier.clone();
        let (started_tx, started_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let waiter = thread::spawn(move || {
            started_tx.send(()).unwrap();
            let _apply = other.apply();
            done_tx.send(()).unwrap();
        });
        started_rx.recv_timeout(LIMIT).unwrap();
        assert!(
            done_rx.recv_timeout(Duration::from_millis(20)).is_err(),
            "ordinary apply crossed a durable restore inhibition"
        );

        let activation = inhibition.activation_apply();
        let nested = barrier.apply();
        drop(nested);
        drop(activation);
        assert!(
            done_rx.recv_timeout(Duration::from_millis(20)).is_err(),
            "dropping only the activation lease reopened ordinary apply"
        );
        drop(inhibition);
        done_rx.recv_timeout(LIMIT).unwrap();
        waiter.join().unwrap();
    }

    #[test]
    fn restore_inhibition_invalidates_prior_capture_stamp() {
        let barrier = CaptureBarrier::default();
        barrier.apply().initialize_sequence(8);
        let old = barrier.capture(0).unwrap().stamp();
        let inhibition = barrier.apply().inhibit_checkpoints_for_restore();
        drop(inhibition);
        let publication = barrier.capture(0).unwrap();
        assert!(publication.validate_publish(old).is_err());
    }

    #[test]
    fn panic_in_restore_inhibition_latches_uncertainty_before_reopening_capture() {
        let barrier = Arc::new(CaptureBarrier::default());
        let other = barrier.clone();
        assert!(thread::spawn(move || {
            let _inhibition = other.apply().inhibit_checkpoints_for_restore();
            panic!("durable restore panicked");
        })
        .join()
        .is_err());
        assert_eq!(
            barrier.capture(0).err().unwrap(),
            "checkpoint refused: durability is uncertain; restart required"
        );
    }

    #[test]
    fn preparation_revision_changes_only_after_complete_mutation_intervals() {
        let barrier = CaptureBarrier::default();
        assert_eq!(barrier.apply_revision(), 0);
        let outer = barrier.apply();
        let nested = barrier.apply();
        drop(nested);
        assert_eq!(
            barrier.apply_revision(),
            0,
            "nested calls must not invalidate their own record"
        );
        drop(outer);
        assert_eq!(
            barrier.apply_revision(),
            1,
            "empty metadata mutation intervals must invalidate detached preparation"
        );
        let capture = barrier.capture(0).unwrap();
        assert_eq!(barrier.apply_revision(), 1);
        drop(capture);
        assert_eq!(
            barrier.apply_revision(),
            2,
            "checkpoint binding can replace immutable readers without changing document version"
        );
    }
}
