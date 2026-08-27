// CODEGEN-BEGIN
//! Shared durable local storage primitives for axiom services.
//!
//! Services still own their domain record and snapshot codecs. This crate owns
//! the repeated mechanics around durable local files: fsync policy, atomic
//! replacement, CRC-framed append logs, and sequence-named snapshot stores.

mod atomic;
mod framed_log;
mod fsync;
mod generation;
mod snapshot_store;

pub use atomic::{atomic_write, sync_parent_dir};
pub use framed_log::{FramedLogReader, FramedLogWriter, LogFrame};
pub use fsync::FsyncPolicy;
pub use generation::{
    CommitError, CommitFailureClass, CommitStep, CurrentReadError, CurrentReadErrorKind,
    CurrentTarget, FailureInjector, FailurePoint, GenerationName, GenerationNameError,
    GenerationNameErrorKind, GenerationStore, NoFailures, StagedGeneration, CURRENT_FILE_NAME,
    CURRENT_TEMP_FILE_NAME, EMPTY_CURRENT_BYTES,
};
pub use snapshot_store::{SnapshotFile, SnapshotFileStore};
// CODEGEN-END
