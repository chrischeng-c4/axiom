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
//! returned bytes through a [`BackupSink`]. Cloud sinks are feature work on this
//! crate, not bespoke code in each service or in the operator.

mod destination;
mod policy;
mod runner;
mod sink;

pub use destination::BackupDestination;
pub use policy::{BackupPolicy, RetentionPolicy};
pub use runner::{run_backup_once, BackupObject, BackupRunResult};
pub use sink::{sink_from_destination, BackupSink, LocalFsSink, UnsupportedCloudSink};
