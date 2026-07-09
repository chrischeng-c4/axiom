// HANDWRITE-BEGIN gap="missing-generator:logic:adf117ff" tracker="pending-tracker" reason="New module (feature backup): fetch_snapshot_bytes(base_url, token) GETs {base_url}/admin/backup via reqwest (Bearer when set, non-2xx bails with status+body); run_backup(base_url, token, dest, retention) hands the exact bytes to service_backup::run_backup_once against sink_from_destination -- relay's src/backup.rs pattern verbatim (transport + shipping only, no snapshot logic)."
//! `tape backup` (WI #1329): fetch a consistent snapshot from a running
//! node's `GET /admin/backup` endpoint and hand the exact bytes to a
//! `libs/service-backup` destination sink. This module owns NO snapshot
//! logic — the endpoint serves the same `raft::snapshot_bytes` serialization
//! (the whole [`crate::TapeJournal`] + applied index) the raft state
//! machine's own `snapshot`/`restore` round-trip; this is transport +
//! shipping only (relay #1209's `src/backup.rs` pattern verbatim).
//!
//! Restore is the existing raft-side `TapeStateMachine::restore` merge path
//! (loaded offline/out of band); no restore CLI verb is added here, matching
//! relay's scope.

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

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use service_backup::BackupDestination;

    use crate::server::{router, AppState};
    use crate::TapeJournal;

    async fn start_server() -> (SocketAddr, AppState) {
        let mut journal = TapeJournal::default();
        journal.append("orders", None, serde_json::json!({"n": 1}), Some(100));
        let state = AppState::new(journal, None);
        let app = router(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(service_http::serve(
            listener,
            app,
            std::future::pending::<()>(),
        ));
        (addr, state)
    }

    /// R2: `run_backup` fetches the live `/admin/backup` bytes unmodified and
    /// hands them to the destination sink — the artifact on disk is exactly
    /// `raft::snapshot_bytes` for the same journal.
    #[tokio::test]
    async fn run_backup_ships_fetched_bytes_to_sink() {
        let (addr, state) = start_server().await;
        let dir = tempfile::tempdir().unwrap();
        let dest =
            BackupDestination::from_uri(&format!("file://{}", dir.path().display())).unwrap();

        let result = super::run_backup(
            &format!("http://{addr}"),
            None,
            &dest,
            &service_backup::RetentionPolicy::default(),
        )
        .await
        .unwrap();

        assert!(result.object.bytes > 0);
        let artifact = std::fs::read(dir.path().join(&result.object.key)).unwrap();
        let expected = crate::raft::snapshot_bytes(&state.journal_handle(), 0).unwrap();
        assert_eq!(&artifact[..], &expected[..]);
    }
}
// HANDWRITE-END
