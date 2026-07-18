use std::time::SystemTime;

use anyhow::{bail, Context, Result};
use service_backup::{
    run_backup_once, sink_from_destination, BackupDestination, BackupRunResult, RetentionPolicy,
};

/// Fetch `{base_url}/admin/backup` (Bearer `token` when set) and return the
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

/// POST exact Snapshot bytes to `{base_url}/admin/restore` (Bearer `token` when set).
pub async fn restore_snapshot_bytes(
    base_url: &str,
    token: Option<&str>,
    payload: &[u8],
) -> Result<()> {
    let url = format!("{}/admin/restore", base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let mut req = client
        .post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
        .body(payload.to_vec());
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let resp = req.send().await.with_context(|| format!("POST {url}"))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        bail!("POST {url} returned {status}: {body}");
    }
    Ok(())
}

/// Fetch `{base_url}/admin/backup` (Bearer `token` when set) and ship the
/// returned bytes to `dest` via `service_backup::run_backup_once`, applying
/// `retention` afterward.
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
