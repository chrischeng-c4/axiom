// HANDWRITE-BEGIN gap="sift-shared-backup-runner" tracker="1605" reason="Implement Sift snapshot backup/restore composition through service-backup destinations and retention."
//! Off-node snapshot backup and restore composition for Sift.
//!
//! Scheduled runners fetch the live service's protected `/admin/backup`
//! endpoint. They never reopen a mounted journal/PVC alongside the serving
//! process, so snapshot consistency and file ownership remain with
//! [`DurableJournal`].

use std::time::{Duration, SystemTime};

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
    let url = format!("{}/admin/backup", base_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(LIVE_BACKUP_TIMEOUT)
        .build()
        .context("build live snapshot HTTP client")?;
    let mut request = client.get(&url);
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    let mut response = request.send().await.with_context(|| format!("GET {url}"))?;
    let status = response.status();
    if status != reqwest::StatusCode::OK {
        let mut diagnostic = Vec::new();
        while diagnostic.len() < MAX_ERROR_BODY_BYTES {
            let Some(chunk) = response
                .chunk()
                .await
                .with_context(|| format!("read bounded error response from {url}"))?
            else {
                break;
            };
            let remaining = MAX_ERROR_BODY_BYTES - diagnostic.len();
            diagnostic.extend_from_slice(&chunk[..chunk.len().min(remaining)]);
            if chunk.len() > remaining {
                break;
            }
        }
        let diagnostic = String::from_utf8_lossy(&diagnostic);
        bail!("GET {url} returned {status}: {diagnostic}");
    }
    response
        .bytes()
        .await
        .map(|bytes| bytes.to_vec())
        .with_context(|| format!("read snapshot response body from {url}"))
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
    let destination = BackupDestination::from_uri(destination_uri)?;
    let retention = retention_secs
        .map(RetentionPolicy::max_age_seconds)
        .unwrap_or_default();
    let sink = sink_from_destination(&destination)?;
    run_backup_once(sink.as_ref(), SystemTime::now(), &snapshot, &retention)
}

/// Load an exact backup object through the shared source contract and replace
/// this journal's local snapshot atomically before replay resumes.
pub fn restore_journal(journal: &DurableJournal, source_uri: &str) -> Result<()> {
    let bytes = fetch_backup_object(source_uri)?;
    journal.restore_snapshot_bytes(&bytes)
}

// HANDWRITE-END
