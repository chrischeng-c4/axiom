// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-backup_sink-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Lumen backup compatibility exports.
//!
//! Lumen produces a consistent snapshot from its engine / raft state machine.
//! Destination schema, sink implementations, retention, and runner primitives
//! live in `libs/service-backup` so Lumen, Keep, Relay, and Loom share one
//! backup contract instead of each carrying a bespoke sink.
//!
pub use service_backup::{
    run_backup_once, sink_from_destination, BackupDestination, BackupObject, BackupPolicy,
    BackupRunResult, BackupSink, LocalFsSink, RetentionPolicy, UnsupportedCloudSink,
};
// CODEGEN-END
