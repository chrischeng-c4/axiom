// HANDWRITE-BEGIN gap="missing-generator:logic:a80dd388" tracker="pending-tracker" reason="keep's backup adoption: re-export service_backup types (mirrors lumen's backup_sink), a KeepSnapshot payload built from engine.dump_values, snapshot_bytes/snapshot_bytes_from_data_dir, and run_backup that recovers the engine, serializes a consistent snapshot, and calls sink_from_destination + run_backup_once."
//! keep backup adoption of the shared `libs/service-backup` contract.
//!
//! keep owns snapshot consistency: [`RecoveryManager`] rebuilds a full engine
//! from the on-disk snapshot + WAL, and [`KeepSnapshot`] serializes that
//! engine's live key/value set into one coherent payload. Destination / policy /
//! sink / runner primitives live in `libs/service-backup` so keep, lumen, relay,
//! and loom share a single backup contract instead of each carrying a bespoke
//! sink (this mirrors lumen's `backup_sink` re-export module).
//!
//! The `keep backup` CLI verb and the operator-rendered backup CronJob both
//! drive [`run_backup`]: recover → serialize → `sink_from_destination` →
//! `run_backup_once`. Cloud sinks (`s3://` / `gs://`) fail loud until a cloud
//! adapter feature is linked into `service-backup`; `file://` is the must-work
//! local path.
//!
//! @spec projects/keep/tech-design/logic/adopt-libs-service-backup-snapshot-sink-keep-backup-verb.md

use std::path::Path;
use std::time::SystemTime;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::engine::KvEngine;
use crate::persistence::recovery::RecoveryManager;
use crate::types::KvValue;

pub use service_backup::{
    run_backup_once, sink_from_destination, BackupDestination, BackupObject, BackupPolicy,
    BackupRunResult, BackupSink, LocalFsSink, RetentionPolicy, UnsupportedCloudSink,
};

/// Snapshot payload format version (bump on an incompatible layout change).
pub const KEEP_SNAPSHOT_VERSION: u32 = 1;

/// A consistent, self-describing snapshot of keep's full key/value state.
///
/// TTL metadata is intentionally dropped — same as [`KvEngine::dump_values`]; a
/// restore re-imports values without expiry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeepSnapshot {
    /// Payload format version.
    pub version: u32,
    /// Every live (non-expired) key→value pair across all shards.
    pub values: Vec<(String, KvValue)>,
}

impl KeepSnapshot {
    /// Build a snapshot from a live engine (R1).
    pub fn from_engine(engine: &KvEngine) -> Self {
        Self {
            version: KEEP_SNAPSHOT_VERSION,
            values: engine.dump_values(),
        }
    }
}

/// Serialize a consistent snapshot of `engine` to JSON bytes (R1).
pub fn snapshot_bytes(engine: &KvEngine) -> Result<Vec<u8>> {
    serde_json::to_vec(&KeepSnapshot::from_engine(engine)).context("serialize keep snapshot")
}

/// Recover keep's on-disk state (latest snapshot + full WAL replay) into an
/// engine, then serialize a consistent full-state snapshot payload. This is the
/// self-contained source of bytes for the `keep backup` verb / CronJob runner,
/// so it never needs a live server.
pub fn snapshot_bytes_from_data_dir(data_dir: impl AsRef<Path>, shards: usize) -> Result<Vec<u8>> {
    let (engine, _stats) =
        RecoveryManager::recover(data_dir, shards).context("recover engine from data dir")?;
    snapshot_bytes(&engine)
}

/// One backup run (R2): recover a consistent snapshot from `data_dir`, write it
/// to `dest`, and apply `retention`. `file://` is the must-work local path;
/// `s3://` / `gs://` route to the shared fail-loud placeholder sink until a
/// cloud adapter feature is linked.
pub fn run_backup(
    data_dir: impl AsRef<Path>,
    shards: usize,
    dest: &BackupDestination,
    retention: &RetentionPolicy,
) -> Result<BackupRunResult> {
    let payload = snapshot_bytes_from_data_dir(data_dir, shards)?;
    let sink = sink_from_destination(dest)?;
    run_backup_once(sink.as_ref(), SystemTime::now(), &payload, retention)
}
// HANDWRITE-END
