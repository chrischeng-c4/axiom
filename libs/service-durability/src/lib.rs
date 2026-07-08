// SPEC-MANAGED: libs/service-durability/tech-design/semantic/source/libs-service-durability-src-lib-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Shared durable local storage primitives for axiom services.
//!
//! Services still own their domain record and snapshot codecs. This crate owns
//! the repeated mechanics around durable local files: fsync policy, atomic
//! replacement, CRC-framed append logs, and sequence-named snapshot stores.

mod atomic;
mod framed_log;
mod fsync;
mod snapshot_store;

pub use atomic::{atomic_write, sync_parent_dir};
pub use framed_log::{FramedLogReader, FramedLogWriter, LogFrame};
pub use fsync::FsyncPolicy;
pub use snapshot_store::{SnapshotFile, SnapshotFileStore};
// CODEGEN-END
