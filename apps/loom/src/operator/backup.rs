//! `loom backup` — upload a raft snapshot to a backup destination via the shared
//! `service-backup` sink.
//!
//! The archetype's backup-runner shape: the service produces consistent snapshot
//! bytes (`LoomSm`'s persisted `runs.snapshot.json`), the runner uploads them
//! (local / S3 / GCS) and prunes by retention, and the operator schedules the
//! runner (a CronJob). loom does not serialize its data here — it reads the
//! snapshot the state machine already wrote.

use std::path::Path;
use std::time::SystemTime;

use service_backup::{run_backup_once, sink_from_destination, BackupDestination, RetentionPolicy};

/// Upload `source` (a snapshot file) to `destination` (`file:///path`,
/// `s3://bucket/prefix`, `gs://bucket/prefix`), pruning objects older than
/// `max_age_secs` when set.
pub fn run(source: &Path, destination: &str, max_age_secs: Option<u64>) -> anyhow::Result<()> {
    let payload = std::fs::read(source)
        .map_err(|e| anyhow::anyhow!("read snapshot {}: {e}", source.display()))?;
    let dest = BackupDestination::from_uri(destination)?;
    let sink = sink_from_destination(&dest)?;
    let retention = RetentionPolicy { max_age_seconds: max_age_secs };
    let result = run_backup_once(sink.as_ref(), SystemTime::now(), &payload, &retention)?;
    println!(
        "backed up {} bytes → {} (key {}); pruned {} old object(s)",
        result.object.bytes, result.object.sink, result.object.key, result.pruned
    );
    Ok(())
}
