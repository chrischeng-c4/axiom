// HANDWRITE-BEGIN gap="sift-shared-backup-runner" tracker="1605" reason="Implement Sift snapshot backup/restore composition through service-backup destinations and retention."
//! Off-node snapshot backup and restore composition for Sift.

use std::time::SystemTime;

use anyhow::Result;
use service_backup::{
    fetch_backup_object, run_backup_once, sink_from_destination, BackupDestination, BackupRunResult,
    RetentionPolicy,
};

use crate::DurableJournal;

/// Serialize a point-in-time Sift journal snapshot and ship it to the shared
/// destination contract. Cloud destinations remain explicit: if this build
/// lacks a configured sink, `service-backup` fails rather than pretending a
/// local path is a production backup.
pub fn backup_journal(
    journal: &DurableJournal,
    destination_uri: &str,
    retention_secs: Option<u64>,
) -> Result<BackupRunResult> {
    let destination = BackupDestination::from_uri(destination_uri)?;
    let retention = retention_secs
        .map(RetentionPolicy::max_age_seconds)
        .unwrap_or_default();
    let sink = sink_from_destination(&destination)?;
    let snapshot = journal.snapshot_bytes()?;
    run_backup_once(sink.as_ref(), SystemTime::now(), &snapshot, &retention)
}

/// Load an exact backup object through the shared source contract and replace
/// this journal's local snapshot atomically before replay resumes.
pub fn restore_journal(journal: &DurableJournal, source_uri: &str) -> Result<()> {
    let bytes = fetch_backup_object(source_uri)?;
    journal.restore_snapshot_bytes(&bytes)
}

<!-- marker: sift-shared-backup-runner path: projects/sift/src/backup.rs reason: Implement Sift snapshot backup/restore composition through service-backup destinations and retention. -->
// HANDWRITE-END
