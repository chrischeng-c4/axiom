//! Shared HTTP transport for a service's standard admin snapshot endpoint.

use std::time::SystemTime;

use anyhow::{bail, Context, Result};

use crate::{
    run_backup_once, sink_from_destination, BackupDestination, BackupRunResult, RetentionPolicy,
};

/// Fetch exact snapshot bytes from the standard `GET /admin/backup` endpoint.
///
/// Services retain the domain-specific snapshot encoding and restore policy.
/// This helper owns the common Bearer request, non-success diagnostic, and
/// byte-preserving response read used by service backup CLIs and CronJobs.
pub async fn fetch_admin_snapshot(base_url: &str, token: Option<&str>) -> Result<Vec<u8>> {
    let url = format!("{}/admin/backup", base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let mut request = client.get(&url);
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    let response = request.send().await.with_context(|| format!("GET {url}"))?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        bail!("GET {url} returned {status}: {body}");
    }
    let payload = response
        .bytes()
        .await
        .with_context(|| format!("read response body from {url}"))?;
    Ok(payload.to_vec())
}

/// Fetch an admin snapshot and ship the exact bytes to `dest`.
///
/// `file://` always works. `s3://` requires the crate's `s3` feature; `gs://`
/// remains schema-compatible and fails loudly until a GCS sink exists.
pub async fn run_admin_snapshot_backup(
    base_url: &str,
    token: Option<&str>,
    dest: &BackupDestination,
    retention: &RetentionPolicy,
) -> Result<BackupRunResult> {
    let payload = fetch_admin_snapshot(base_url, token).await?;
    let sink = sink_from_destination(dest)?;
    run_backup_once(sink.as_ref(), SystemTime::now(), &payload, retention)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn fetches_exact_snapshot_bytes_with_bearer_auth() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/admin/backup"))
            .and(header("authorization", "Bearer registry-token"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"snapshot".to_vec()))
            .mount(&server)
            .await;

        let bytes = fetch_admin_snapshot(&server.uri(), Some("registry-token"))
            .await
            .expect("admin snapshot fetch succeeds");
        assert_eq!(bytes, b"snapshot");
    }

    #[tokio::test]
    async fn keeps_non_success_status_and_body_in_the_diagnostic() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/admin/backup"))
            .respond_with(ResponseTemplate::new(503).set_body_string("not ready"))
            .mount(&server)
            .await;

        let error = fetch_admin_snapshot(&server.uri(), None)
            .await
            .expect_err("non-success status must fail");
        assert!(error.to_string().contains("503"));
        assert!(error.to_string().contains("not ready"));
    }
}
