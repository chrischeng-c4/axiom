// HANDWRITE-BEGIN gap="missing-generator:logic:f9ee58f0" tracker="pending-tracker" reason="New module, cfg(feature = backup): fetch_snapshot_bytes(base_url, token) GETs {base_url}/admin/backup via reqwest (Bearer when set, non-2xx bails with status + body); run_backup(base_url, token, dest, retention) hands the exact bytes to service_backup::run_backup_once against sink_from_destination — lumen src/backup.rs pattern minus the restore POST (relay restore is load_live merge, library-side)."
//! `relay backup` (WI #1209): fetch a consistent snapshot from a running
//! node's `GET /admin/backup` endpoint and hand the exact bytes to a
//! `libs/service-backup` destination sink. This module owns NO snapshot
//! logic — the endpoint serves the same `raft::snapshot_bytes` serialization
//! (`EngineSnapshot` = `dump_live` + applied index) the raft snapshotter
//! uses; this is transport + shipping only, meant to be driven by the
//! operator's optional backup CronJob (`spec.backup`, see
//! `operator::render::backup_cron_job`) or invoked ad hoc via the CLI
//! (lumen #808 pattern).
//!
//! Restore is a library-side `load_live` MERGE: feed the artifact to
//! `crate::raft::load_snapshot_bytes` on a fresh node — idempotent per
//! `message_id`, leases/acks are node-local and not in the snapshot, so
//! restored work redelivers (at-least-once).

use std::time::SystemTime;

use anyhow::{bail, Context, Result};
use service_backup::{
    run_backup_once, sink_from_destination, BackupDestination, BackupRunResult, RetentionPolicy,
};

/// Fetch `{base_url}/admin/backup` (Bearer `token` when set — the endpoint
/// needs `admin` on `*` when the node runs `--auth required`) and return the
/// exact snapshot response bytes.
pub async fn fetch_snapshot_bytes(base_url: &str, token: Option<&str>) -> Result<Vec<u8>> {
    let url = format!("{}/admin/backup", base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let mut req = client.get(&url);
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let resp = req.send().await.with_context(|| format!("GET {url}"))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        bail!("GET {url} returned {status}: {body}");
    }
    let payload = resp
        .bytes()
        .await
        .with_context(|| format!("read response body from {url}"))?;
    Ok(payload.to_vec())
}

/// Fetch `{base_url}/admin/backup` and ship the returned bytes to `dest` via
/// `service_backup::run_backup_once`, applying `retention` afterward.
/// `file://` always works; `s3://` needs the lib's `s3` feature (the `backup`
/// feature enables it); `gs://` parses but the lib's sink fails loudly.
pub async fn run_backup(
    base_url: &str,
    token: Option<&str>,
    dest: &BackupDestination,
    retention: &RetentionPolicy,
) -> Result<BackupRunResult> {
    let payload = fetch_snapshot_bytes(base_url, token).await?;
    let sink = sink_from_destination(dest)?;
    run_backup_once(sink.as_ref(), SystemTime::now(), &payload, retention)
}
// HANDWRITE-END
