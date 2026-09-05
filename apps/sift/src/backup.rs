// HANDWRITE-BEGIN gap="sift-shared-backup-runner" tracker="1605" reason="Implement Sift snapshot backup/restore composition through service-backup destinations and retention."
//! Off-node snapshot backup and restore composition for Sift.
//!
//! Scheduled runners fetch the live service's protected `/admin/backup`
//! endpoint. They never reopen a mounted journal/PVC alongside the serving
//! process, so snapshot consistency and file ownership remain with
//! [`DurableJournal`].

use std::{
    path::Path,
    time::{Duration, SystemTime},
};

use anyhow::{bail, Context, Result};
use service_backup::{
    fetch_backup_object, run_backup_once, sink_from_destination, BackupDestination,
    BackupRunResult, RetentionPolicy,
};

use crate::DurableJournal;

const LIVE_BACKUP_TIMEOUT: Duration = Duration::from_secs(300);
const MAX_ERROR_BODY_BYTES: usize = 8 * 1024;

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

/// Fetch exact snapshot bytes from a running Sift service.
///
/// `base_url` is the serving service root; this function appends the canonical
/// `/admin/backup` path. `token`, when present, is sent as a Bearer credential.
/// The full request/body transfer has a five-minute deadline, and non-200
/// diagnostics are capped at 8 KiB before being included in the error. A
/// non-200 response, timeout, transport failure, or body-read failure returns
/// an error and no bytes.
pub async fn fetch_live_snapshot(base_url: &str, token: Option<&str>) -> Result<Vec<u8>> {
    fetch_live_snapshot_authenticated(base_url, token, None, "sift.axiom.dev", "*").await
}

/// Fetch a live snapshot with either a static token or a rotating projected
/// ServiceAccount token. The project header is the SubjectAccessReview scope.
pub async fn fetch_live_snapshot_authenticated(
    base_url: &str,
    token: Option<&str>,
    token_file: Option<&Path>,
    token_audience: &str,
    project: &str,
) -> Result<Vec<u8>> {
    if token.is_some() && token_file.is_some() {
        bail!("live backup token and token file are mutually exclusive");
    }
    let transport = service_backup::AdminSnapshotTransport::with_config(
        service_backup::AdminSnapshotTransportConfig {
            operation_timeout: LIVE_BACKUP_TIMEOUT,
            max_diagnostic_bytes: MAX_ERROR_BODY_BYTES,
            ..Default::default()
        },
    )
    .context("build live snapshot HTTP transport")?;
    let mut request = service_backup::AdminSnapshotRequest::new()
        .with_header("x-sift-project", project)
        .context("build Sift live snapshot project metadata")?;
    if let Some(path) = token_file {
        request = request.with_projected_bearer(path, token_audience);
    } else if let Some(token) = token {
        request = request.with_static_bearer(token);
    }
    transport
        .fetch(base_url, &request)
        .await
        .map_err(anyhow::Error::new)
}

/// Fetch the live service snapshot and upload those exact bytes through the
/// shared destination/retention contract.
///
/// This is the safe scheduled-runner seam: it performs no direct filesystem or
/// PVC access. `destination_uri` accepts the same `file://`, `s3://`, and
/// `gs://` forms as [`backup_journal`].
pub async fn backup_live_journal(
    base_url: &str,
    token: Option<&str>,
    destination_uri: &str,
    retention_secs: Option<u64>,
) -> Result<BackupRunResult> {
    let snapshot = fetch_live_snapshot(base_url, token).await?;
    upload_snapshot(&snapshot, destination_uri, retention_secs)
}

pub async fn backup_live_journal_authenticated(
    base_url: &str,
    token: Option<&str>,
    token_file: Option<&Path>,
    token_audience: &str,
    project: &str,
    destination_uri: &str,
    retention_secs: Option<u64>,
) -> Result<BackupRunResult> {
    let snapshot =
        fetch_live_snapshot_authenticated(base_url, token, token_file, token_audience, project)
            .await?;
    upload_snapshot(&snapshot, destination_uri, retention_secs)
}

fn upload_snapshot(
    snapshot: &[u8],
    destination_uri: &str,
    retention_secs: Option<u64>,
) -> Result<BackupRunResult> {
    let destination = BackupDestination::from_uri(destination_uri)?;
    let retention = retention_secs
        .map(RetentionPolicy::max_age_seconds)
        .unwrap_or_default();
    let sink = sink_from_destination(&destination)?;
    run_backup_once(sink.as_ref(), SystemTime::now(), snapshot, &retention)
}

/// Load an exact backup object through the shared source contract and replace
/// this journal's local snapshot atomically before replay resumes.
pub fn restore_journal(journal: &DurableJournal, source_uri: &str) -> Result<()> {
    let bytes = fetch_backup_object(source_uri)?;
    journal.restore_snapshot_bytes(&bytes)
}

// HANDWRITE-END
