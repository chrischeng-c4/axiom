//! Ownership bridge for a committed in-memory record and its durable stage.
//!
//! A committed budget charge stays attached to its payload while staging reads
//! it. A failed stage returns that same carrier. A successful stage drops the
//! payload before it returns the durable receipt, then consumes the narrowly
//! typed pre-apply RAM charge. The durable stage keeps only disk metadata.

use std::fmt;
use std::io::{self, Read, Seek, SeekFrom, Write};

use crate::change_budget::{AdmissionError, RamCharge, Reservation};
use crate::committed_stage::{DurableStage, SourceIdentity, StageStore};

/// A local write could not reserve the pending-change budget before it was
/// published to the WAL. Full, oversized, and unrepresentable local requests
/// are refused here. Committed records never use this classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PendingChangeCapacity {
    Local {
        requested: usize,
        used: usize,
        hard_limit: usize,
    },
    Oversized {
        requested: usize,
        hard_limit: usize,
    },
    Overflow,
    #[cfg(feature = "raft-wal")]
    Raft {
        reason: String,
    },
}

impl PendingChangeCapacity {
    /// Classify every local admission failure before publication.
    pub(crate) fn from_prepublication(error: AdmissionError) -> Result<Self, AdmissionError> {
        match error {
            AdmissionError::Full {
                requested,
                used,
                hard_limit,
            } => Ok(Self::Local {
                requested,
                used,
                hard_limit,
            }),
            AdmissionError::Oversized {
                requested,
                hard_limit,
            } => Ok(Self::Oversized {
                requested,
                hard_limit,
            }),
            other => Err(other),
        }
    }

    pub(crate) fn from_record_prepublication(
        error: &crate::storage::RecordAdmissionError,
    ) -> Option<Self> {
        match error {
            crate::storage::RecordAdmissionError::Capacity(error) => {
                Self::from_prepublication(*error).ok()
            }
            crate::storage::RecordAdmissionError::Overflow => Some(Self::Overflow),
            _ => None,
        }
    }
}

impl fmt::Display for PendingChangeCapacity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Local {
                requested,
                used,
                hard_limit,
            } => write!(
                formatter,
                "pending change capacity is full: requested {} bytes with {} of {} bytes in use",
                requested, used, hard_limit
            ),
            Self::Oversized {
                requested,
                hard_limit,
            } => write!(
                formatter,
                "pending change is too large: requested {} bytes exceeds {} bytes",
                requested, hard_limit
            ),
            Self::Overflow => formatter.write_str("pending change size overflow"),
            #[cfg(feature = "raft-wal")]
            Self::Raft { reason } => formatter.write_str(reason),
        }
    }
}

impl std::error::Error for PendingChangeCapacity {}

#[cfg(test)]
mod prepublication_tests {
    use super::*;

    #[test]
    fn prepublication_capacity_classifies_full_and_oversized_without_used_bytes() {
        assert!(matches!(
            PendingChangeCapacity::from_prepublication(AdmissionError::Full {
                requested: 8,
                used: 7,
                hard_limit: 10,
            }),
            Ok(PendingChangeCapacity::Local { .. })
        ));
        assert!(matches!(
            PendingChangeCapacity::from_prepublication(AdmissionError::Oversized {
                requested: 11,
                hard_limit: 10,
            }),
            Ok(PendingChangeCapacity::Oversized { .. })
        ));
        assert!(matches!(
            PendingChangeCapacity::from_record_prepublication(
                &crate::storage::RecordAdmissionError::Overflow
            ),
            Some(PendingChangeCapacity::Overflow)
        ));
        assert!(PendingChangeCapacity::from_record_prepublication(
            &crate::storage::RecordAdmissionError::Capacity(AdmissionError::Retired)
        )
        .is_none());
        assert!(PendingChangeCapacity::from_record_prepublication(
            &crate::storage::RecordAdmissionError::WrongEngine
        )
        .is_none());
    }
}

pub(crate) struct CommittedRecord<P> {
    charge: RamCharge,
    payload: P,
    sequence: u64,
    engine_epoch: u64,
    source: SourceIdentity,
}

pub(crate) struct StagedCommitted {
    stage: DurableStage,
}

pub(crate) struct StageFailure<P> {
    error: io::Error,
    record: CommittedRecord<P>,
}

/// Admission can race a restore retiring the old budget owner. Return the
/// committed bytes and their identity so the apply worker can route or stage
/// them under the replacement owner. An error must not destroy this payload.
pub(crate) struct AdmissionFailure<P> {
    error: AdmissionError,
    payload: P,
    sequence: u64,
    engine_epoch: u64,
    source: SourceIdentity,
}

impl<P> fmt::Debug for AdmissionFailure<P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AdmissionFailure")
            .field("error", &self.error)
            .field("sequence", &self.sequence)
            .field("engine_epoch", &self.engine_epoch)
            .finish_non_exhaustive()
    }
}

impl<P> AdmissionFailure<P> {
    pub(crate) fn into_parts(self) -> (AdmissionError, P, u64, u64, SourceIdentity) {
        (
            self.error,
            self.payload,
            self.sequence,
            self.engine_epoch,
            self.source,
        )
    }
}

/// A retryable owned payload that writes its exact wire representation into a
/// bounded staging sink. `Record` can implement this with `serde_json::to_writer`
/// and therefore never needs a full encoded `Vec`.
pub(crate) trait StagePayload {
    fn write_stage(&mut self, output: &mut dyn Write) -> io::Result<()>;
}

/// Adapter for existing rewindable readers. New production record payloads can
/// implement [`StagePayload`] directly with their serializer.
pub(crate) struct RewindablePayload<P>(pub(crate) P);
impl<P: Read + Seek> StagePayload for RewindablePayload<P> {
    fn write_stage(&mut self, output: &mut dyn Write) -> io::Result<()> {
        self.0.seek(SeekFrom::Start(0))?;
        io::copy(&mut self.0, output).map(|_| ())
    }
}

/// Private proof emitted only after `CommittedRecord::stage` has obtained a
/// durable receipt and destroyed the RAM payload. Its fields cannot be forged
/// outside this module.
pub(crate) struct RamReleased {
    sequence: u64,
    engine_epoch: u64,
    source: SourceIdentity,
}

impl RamReleased {
    fn after_payload_drop(stage: &DurableStage) -> Self {
        Self {
            sequence: stage.sequence(),
            engine_epoch: stage.engine_epoch(),
            source: stage.source().clone(),
        }
    }
    pub(crate) fn matches(
        &self,
        sequence: u64,
        engine_epoch: u64,
        source: &SourceIdentity,
    ) -> bool {
        self.sequence == sequence && self.engine_epoch == engine_epoch && &self.source == source
    }
}

impl<P> fmt::Debug for StageFailure<P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StageFailure")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

impl<P> CommittedRecord<P> {
    /// Commit an already admitted reservation and attach it to the payload.
    pub(crate) fn new(
        reservation: Reservation,
        payload: P,
        sequence: u64,
        engine_epoch: u64,
        source: SourceIdentity,
    ) -> Result<Self, AdmissionFailure<P>> {
        let charge = match reservation.commit_ram(sequence, engine_epoch, source.clone()) {
            Ok(charge) => charge,
            Err(error) => {
                return Err(AdmissionFailure {
                    error,
                    payload,
                    sequence,
                    engine_epoch,
                    source,
                })
            }
        };
        Ok(Self {
            charge,
            payload,
            sequence,
            engine_epoch,
            source,
        })
    }

    pub(crate) fn charge_bytes(&self) -> usize {
        self.charge.bytes()
    }
}

impl<P: StagePayload> CommittedRecord<P> {
    /// Stream the owned payload. Failure returns the exact carrier for retry.
    pub(crate) fn stage(self, store: &StageStore) -> Result<StagedCommitted, StageFailure<P>> {
        let Self {
            charge,
            mut payload,
            sequence,
            engine_epoch,
            source,
        } = self;
        match store.stage_with_writer(sequence, engine_epoch, source.clone(), |output| {
            payload.write_stage(output)
        }) {
            Ok(stage) => {
                drop(payload);
                let proof = RamReleased::after_payload_drop(&stage);
                if charge.release_after_stage(proof).is_err() {
                    panic!(
                        "the carrier's own durable proof must release its matching live RAM charge"
                    );
                }
                Ok(StagedCommitted { stage })
            }
            Err(error) => Err(StageFailure {
                error,
                record: Self {
                    charge,
                    payload,
                    sequence,
                    engine_epoch,
                    source,
                },
            }),
        }
    }
}

impl<P> StageFailure<P> {
    pub(crate) fn error(&self) -> &io::Error {
        &self.error
    }
    pub(crate) fn into_record(self) -> CommittedRecord<P> {
        self.record
    }
}

impl StagedCommitted {
    pub(crate) fn stage(&self) -> &DurableStage {
        &self.stage
    }
    pub(crate) fn into_stage(self) -> DurableStage {
        self.stage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change_budget::ChangeBudget;
    use crate::committed_stage::{SourceKind, StageFailureInjector, StageFailurePoint};
    use std::io::Cursor;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    struct FailOnce(Mutex<Option<StageFailurePoint>>);
    impl StageFailureInjector for FailOnce {
        fn check(&self, point: StageFailurePoint) -> io::Result<()> {
            let mut wanted = self.0.lock().unwrap();
            if *wanted == Some(point) {
                *wanted = None;
                return Err(io::Error::other("injected stage failure"));
            }
            Ok(())
        }
    }

    struct DropReader {
        bytes: Cursor<Vec<u8>>,
        drops: Arc<AtomicUsize>,
    }

    /// Deliberately lacks `Read` and `Seek`: a production `Record` can stream
    /// its serializer directly into staging without an encoded-record buffer.
    struct WriterOnlyPayload(Vec<u8>);
    impl StagePayload for WriterOnlyPayload {
        fn write_stage(&mut self, output: &mut dyn Write) -> io::Result<()> {
            output.write_all(&self.0)
        }
    }
    impl Read for DropReader {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            self.bytes.read(output)
        }
    }
    impl Seek for DropReader {
        fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
            self.bytes.seek(position)
        }
    }
    impl Drop for DropReader {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }

    struct DropChecksCharge {
        bytes: Cursor<Vec<u8>>,
        budget: ChangeBudget,
        observed_reserved: Arc<AtomicUsize>,
    }
    impl Read for DropChecksCharge {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            self.bytes.read(output)
        }
    }
    impl Seek for DropChecksCharge {
        fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
            self.bytes.seek(position)
        }
    }
    impl Drop for DropChecksCharge {
        fn drop(&mut self) {
            self.observed_reserved
                .store(self.budget.snapshot().reserved, Ordering::SeqCst);
        }
    }

    fn source(name: &str) -> SourceIdentity {
        SourceIdentity::new(SourceKind::External, name).unwrap()
    }

    #[test]
    fn checkpoint_never_releases_committed_records_waiting_for_apply() {
        let root = tempfile::tempdir().unwrap();
        let store = StageStore::new(root.path()).unwrap();
        let budget = ChangeBudget::with_hard_limit(256);
        let owner = budget.owner();
        let record = CommittedRecord::new(
            owner.try_reserve(240).unwrap(),
            RewindablePayload(Cursor::new(vec![7; 240])),
            2,
            1,
            source("orders"),
        )
        .unwrap();
        let _applied = owner.try_reserve(16).unwrap().commit().unwrap();
        let frozen = owner.freeze().unwrap();
        assert_eq!(
            owner.publish_through(&frozen).unwrap(),
            16,
            "checkpoint can release only applied journal data"
        );
        assert_eq!(
            budget.snapshot().total,
            240,
            "committed pre-apply RAM must survive unrelated checkpoint publication"
        );
        assert!(owner.try_reserve(64).is_err());
        record.stage(&store).unwrap();
        assert_eq!(budget.snapshot().total, 0);
        assert!(owner.try_reserve(64).is_ok());
    }

    #[test]
    fn retired_admission_returns_the_committed_payload_to_its_owner() {
        let budget = ChangeBudget::new();
        let owner = budget.owner();
        let reservation = owner.try_reserve(7).unwrap();
        let drops = Arc::new(AtomicUsize::new(0));
        owner.retire();
        let refusal = CommittedRecord::new(
            reservation,
            RewindablePayload(DropReader {
                bytes: Cursor::new(b"payload".to_vec()),
                drops: drops.clone(),
            }),
            2,
            1,
            source("orders"),
        );
        assert!(refusal.is_err());
        assert_eq!(
            drops.load(Ordering::SeqCst),
            0,
            "admission failure must return ownership of the committed payload"
        );
        drop(refusal);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn stage_fault_retains_payload_and_charge_for_repeat_attempt() {
        let root = tempfile::tempdir().unwrap();
        let budget = ChangeBudget::new();
        let owner = budget.owner();
        let record = CommittedRecord::new(
            owner.try_reserve(7).unwrap(),
            RewindablePayload(Cursor::new(b"payload".to_vec())),
            4,
            2,
            source("orders"),
        )
        .unwrap();
        let store = StageStore::with_injector(
            root.path(),
            Arc::new(FailOnce(Mutex::new(Some(StageFailurePoint::SyncMarker)))),
        )
        .unwrap();
        let failed = match record.stage(&store) {
            Ok(_) => panic!("injected durability fault unexpectedly staged the payload"),
            Err(failed) => failed,
        };
        assert_eq!(failed.error().kind(), io::ErrorKind::Other);
        assert_eq!(budget.snapshot().reserved, 7);
        let staged = failed.into_record().stage(&store).unwrap();
        assert_eq!(staged.stage().sequence(), 4);
        assert_eq!(budget.snapshot().reserved, 0);
    }

    #[test]
    fn dropped_carrier_cannot_cancel_its_committed_charge() {
        let budget = ChangeBudget::new();
        let owner = budget.owner();
        let drops = Arc::new(AtomicUsize::new(0));
        let record = CommittedRecord::new(
            owner.try_reserve(7).unwrap(),
            RewindablePayload(DropReader {
                bytes: Cursor::new(b"payload".to_vec()),
                drops: drops.clone(),
            }),
            5,
            2,
            source("orders"),
        )
        .unwrap();
        drop(record);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        assert_eq!(budget.snapshot().reserved, 7);
    }

    #[test]
    fn durable_success_drops_ram_after_exact_recovery_and_releases_charge() {
        let root = tempfile::tempdir().unwrap();
        let store = StageStore::new(root.path()).unwrap();
        let budget = ChangeBudget::new();
        let owner = budget.owner();
        let drops = Arc::new(AtomicUsize::new(0));
        let record = CommittedRecord::new(
            owner.try_reserve(7).unwrap(),
            RewindablePayload(DropReader {
                bytes: Cursor::new(b"payload".to_vec()),
                drops: drops.clone(),
            }),
            6,
            3,
            source("orders"),
        )
        .unwrap();
        let staged = record.stage(&store).unwrap();
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        let mut recovered = Vec::new();
        staged
            .stage()
            .open()
            .unwrap()
            .read_to_end(&mut recovered)
            .unwrap();
        assert_eq!(recovered, b"payload");
        assert_eq!(budget.snapshot().reserved, 0);
    }

    #[test]
    fn durable_stage_drops_payload_before_releasing_its_ram_charge() {
        let root = tempfile::tempdir().unwrap();
        let store = StageStore::new(root.path()).unwrap();
        let budget = ChangeBudget::new();
        let owner = budget.owner();
        let observed_reserved = Arc::new(AtomicUsize::new(usize::MAX));
        CommittedRecord::new(
            owner.try_reserve(7).unwrap(),
            RewindablePayload(DropChecksCharge {
                bytes: Cursor::new(b"payload".to_vec()),
                budget: budget.clone(),
                observed_reserved: observed_reserved.clone(),
            }),
            8,
            3,
            source("orders"),
        )
        .unwrap()
        .stage(&store)
        .unwrap();
        assert_eq!(observed_reserved.load(Ordering::SeqCst), 7);
        assert_eq!(budget.snapshot().reserved, 0);
    }

    #[test]
    fn foreign_epoch_source_and_proof_refuse_cleanup() {
        let root = tempfile::tempdir().unwrap();
        let store = StageStore::new(root.path()).unwrap();
        let budget = ChangeBudget::new();
        let owner = budget.owner();
        let staged = CommittedRecord::new(
            owner.try_reserve(7).unwrap(),
            RewindablePayload(Cursor::new(b"payload".to_vec())),
            7,
            4,
            source("orders"),
        )
        .unwrap()
        .stage(&store)
        .unwrap();
        let stage = staged.into_stage();
        assert!(store
            .remove_after_verified_watermark(stage, store.cleanup_proof(source("other"), 4, 7))
            .is_err());
        let stage = store.recover().unwrap().pop().unwrap();
        assert!(store
            .remove_after_verified_watermark(stage, store.cleanup_proof(source("orders"), 5, 7))
            .is_err());
    }

    #[test]
    fn durable_stage_frees_ram_for_a_later_reservation_without_checkpoint() {
        let root = tempfile::tempdir().unwrap();
        let store = StageStore::new(root.path()).unwrap();
        let budget = ChangeBudget::with_hard_limit(256);
        let owner = budget.owner();
        let record = CommittedRecord::new(
            owner.try_reserve(240).unwrap(),
            RewindablePayload(Cursor::new(vec![7; 240])),
            9,
            1,
            source("orders"),
        )
        .unwrap();
        assert!(owner.try_reserve(64).is_err());
        record.stage(&store).unwrap();
        assert_eq!(budget.snapshot().reserved, 0);
        assert!(owner.try_reserve(64).is_ok());
    }

    #[test]
    fn serializer_callback_payload_needs_neither_read_nor_seek() {
        let root = tempfile::tempdir().unwrap();
        let store = StageStore::new(root.path()).unwrap();
        let budget = ChangeBudget::with_hard_limit(32);
        let owner = budget.owner();
        let staged = CommittedRecord::new(
            owner.try_reserve(9).unwrap(),
            WriterOnlyPayload(b"serializer".to_vec()),
            10,
            1,
            source("orders"),
        )
        .unwrap()
        .stage(&store)
        .unwrap();
        let mut recovered = Vec::new();
        staged
            .stage()
            .open()
            .unwrap()
            .read_to_end(&mut recovered)
            .unwrap();
        assert_eq!(recovered, b"serializer");
        assert_eq!(budget.snapshot().reserved, 0);
    }
}
