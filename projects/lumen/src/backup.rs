// HANDWRITE-BEGIN gap="missing-generator:logic:ff759770" tracker="pending-tracker" reason="New module gated #[cfg(feature = 'backup')]: run_backup(base_url, token, dest, retention) fetches {base_url}/admin/backup via reqwest (Bearer auth when token is Some), then hands the response bytes to service_backup::run_backup_once against sink_from_destination(dest) and the given RetentionPolicy, returning a BackupRunResult; unit tests use wiremock to stand in for the admin API and a tempdir + file:// destination for the sink."
//! `lumen backup` (#808): fetch a consistent snapshot from a running serving
//! fleet's already-existing `GET /admin/backup` endpoint and hand the bytes to
//! a `libs/service-backup` destination sink. This module owns no new
//! snapshot/quiesce logic — `Engine::snapshot()` behind that endpoint is the
//! same quiesce-free call the raft snapshotter itself uses; this is transport
//! + scheduling only, meant to be driven by the operator's optional backup
//! CronJob (`spec.serving.backup`, see `operator::render::backup_cron_job`)
//! or invoked ad hoc via the CLI.

use std::time::SystemTime;

use anyhow::{bail, Context, Result};
use service_backup::{
    run_backup_once, sink_from_destination, BackupDestination, BackupRunResult, RetentionPolicy,
};

/// Fetch `{base_url}/admin/backup` (Bearer `token` when set) and return the
/// exact snapshot response bytes.
/// @spec projects/lumen/tech-design/interfaces/cli/lumen-cli-add-dump-load-export-import-snapshot-verbs.md#logic
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

/// POST exact SnapshotV1 JSON bytes to `{base_url}/admin/restore` (Bearer
/// `token` when set).
/// @spec projects/lumen/tech-design/interfaces/cli/lumen-cli-add-dump-load-export-import-snapshot-verbs.md#logic
pub async fn restore_snapshot_bytes(
    base_url: &str,
    token: Option<&str>,
    payload: &[u8],
) -> Result<()> {
    let url = format!("{}/admin/restore", base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let mut req = client
        .post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
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

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn tmp_dir(tag: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("lumen-backup-test-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[tokio::test]
    async fn fetches_snapshot_and_writes_local_sink() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/admin/backup"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"snapshot-bytes".to_vec()))
            .mount(&server)
            .await;

        let dir = tmp_dir("basic");
        let dest = BackupDestination::Local {
            path: dir.to_string_lossy().into_owned(),
            prefix: Some("lumen".to_string()),
        };

        let result = run_backup(&server.uri(), None, &dest, &RetentionPolicy::default())
            .await
            .expect("run_backup succeeds");

        assert_eq!(result.object.bytes, "snapshot-bytes".len());
        assert!(dir.join(&result.object.key).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn sends_bearer_token_when_set() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/admin/backup"))
            .and(header("authorization", "Bearer secret-token"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"ok".to_vec()))
            .mount(&server)
            .await;

        let dir = tmp_dir("token");
        let dest = BackupDestination::Local {
            path: dir.to_string_lossy().into_owned(),
            prefix: None,
        };

        let result = run_backup(
            &server.uri(),
            Some("secret-token"),
            &dest,
            &RetentionPolicy::default(),
        )
        .await
        .expect("run_backup succeeds with a bearer token");

        assert_eq!(result.object.bytes, 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn non_success_status_bails_with_body() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/admin/backup"))
            .respond_with(ResponseTemplate::new(503).set_body_string("engine not ready"))
            .mount(&server)
            .await;

        let dir = tmp_dir("err");
        let dest = BackupDestination::Local {
            path: dir.to_string_lossy().into_owned(),
            prefix: None,
        };

        let err = run_backup(&server.uri(), None, &dest, &RetentionPolicy::default())
            .await
            .expect_err("non-2xx must bail");
        assert!(err.to_string().contains("503"));
        assert!(err.to_string().contains("engine not ready"));
    }

    #[tokio::test]
    async fn restore_posts_snapshot_with_bearer_token() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/admin/restore"))
            .and(header("authorization", "Bearer restore-token"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        restore_snapshot_bytes(
            &server.uri(),
            Some("restore-token"),
            br#"{"version":1,"collections":{}}"#,
        )
        .await
        .expect("restore succeeds");
    }

    #[tokio::test]
    async fn restore_non_success_status_bails_with_body() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/admin/restore"))
            .respond_with(ResponseTemplate::new(400).set_body_string("bad snapshot"))
            .mount(&server)
            .await;

        let err = restore_snapshot_bytes(&server.uri(), None, b"{}")
            .await
            .expect_err("non-2xx must bail");
        assert!(err.to_string().contains("400"));
        assert!(err.to_string().contains("bad snapshot"));
    }

    #[tokio::test]
    async fn applies_retention_after_upload() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/admin/backup"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"snap".to_vec()))
            .mount(&server)
            .await;

        let dir = tmp_dir("retention");
        std::fs::create_dir_all(&dir).unwrap();
        // A pre-existing object old enough that a 1-second retention window
        // prunes it, while the object this run just writes (age ~0s) survives.
        std::fs::write(dir.join("lumen-1.json"), b"old").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));

        let dest = BackupDestination::Local {
            path: dir.to_string_lossy().into_owned(),
            prefix: Some("lumen".to_string()),
        };
        let result = run_backup(
            &server.uri(),
            None,
            &dest,
            &RetentionPolicy::max_age_seconds(1),
        )
        .await
        .expect("run_backup succeeds");

        assert_eq!(result.pruned, 1);
        assert!(dir.join(&result.object.key).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
// HANDWRITE-END
