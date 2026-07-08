// SPEC-MANAGED: libs/service-backup/tech-design/semantic/source/libs-service-backup-src-lib-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! `service-backup` — shared backup contract for axiom services.
//!
//! The data plane owns snapshot consistency: each service state machine produces
//! bytes at a concrete applied index, and `raft-host` handles snapshot install
//! plus log compaction. This crate owns the cross-service backup shape around
//! those bytes: destination/policy schema, sink trait, local sink, and a small
//! runner primitive.
//!
//! Operator code should render/manage a backup runner from the policy. The
//! runner calls the service's admin backup endpoint or CLI, then writes the
//! returned bytes through a [`BackupSink`]. Local is always available, S3 is
//! feature-gated here, and `gs://` remains schema-compatible until this crate
//! grows a real GCS sink. Bootstrap/restore flows can read exact snapshot
//! object URIs through [`fetch_backup_object`].

mod destination;
pub mod llm;
mod policy;
mod runner;
#[cfg(feature = "s3")]
mod s3;
mod sink;
mod source;

pub use destination::BackupDestination;
pub use policy::{BackupPolicy, RetentionPolicy};
pub use runner::{run_backup_once, BackupObject, BackupRunResult};
pub use sink::{sink_from_destination, BackupSink, LocalFsSink, UnsupportedCloudSink};
pub use source::fetch_backup_object;
// CODEGEN-END
