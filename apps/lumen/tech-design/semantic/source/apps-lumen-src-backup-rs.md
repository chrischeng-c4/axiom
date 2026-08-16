<!-- HANDWRITE-BEGIN gap="missing-generator:source:c3730096" tracker="1646" reason="New SPEC-MANAGED tech-design doc for the new backup.rs module (rust-source-unit), mirroring the format of the other projects-lumen-src-operator-*-rs.md docs." -->
---
id: projects-lumen-src-backup-rs
capability_refs:
  - id: "long-running-stability"
    role: primary
    claim: "lumen-crd-reconcile-loop-kube-rs-operator"
    coverage: partial
    rationale: >
      New hand-written module (#808): schedules/transports lumen's existing
      /admin/backup snapshot to a service-backup destination sink for the
      operator's optional backup CronJob (spec.serving.backup) or ad hoc use.
      No generator exists for this HTTP-client + service-backup-sink shape
      yet, so it stays HANDWRITE per CLAUDE.md ("no skip state for source
      ownership") until a generator primitive covers it.
  - id: "replica-sync-bootstrap"
    role: primary
    gap: "external-backup-disaster-recovery-seed"
    claim: "external-backup-disaster-recovery-seed"
    coverage: full
    rationale: >
      This module delegates backup snapshots to shared service-backup sinks,
      making external object storage the disaster-recovery seed surface.
fill_sections: [overview, source, changes]
---

# Standardized apps/lumen/src/backup.rs

## Overview
<!-- type: overview lang: markdown -->

`lumen backup` (#808) support module: fetches a consistent snapshot from a
running serving fleet's already-existing `GET /admin/backup` endpoint and
hands the bytes to a `libs/service-backup` destination sink
(`sink_from_destination` + `run_backup_once`). No new snapshot/quiesce
mechanism — `Engine::snapshot()` behind that endpoint is the same
quiesce-free call the raft snapshotter itself uses. Gated
`#[cfg(feature = "backup")]`, pulled in transitively by the `operator`
feature; the default (no-feature) build links no HTTP client.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `run_backup` | apps/lumen/src/backup.rs | function | pub async | 21 | run_backup(base_url: &str, token: Option<&str>, dest: &BackupDestination, retention: &RetentionPolicy) -> Result<BackupRunResult> |

## Source
<!-- type: rust-source-unit lang: rust -->


```rust
// HANDWRITE-BEGIN gap="missing-generator:logic:ff759770" tracker="1646" reason="Lumen owns only its restore endpoint; service-backup owns the shared authenticated GET /admin/backup fetch and sink upload contract, re-exported under Lumen's compatible helper names."
//! `lumen backup` (#808): fetch a consistent snapshot from a running serving
//! fleet's already-existing `GET /admin/backup` endpoint and hand the bytes to
//! a `libs/service-backup` destination sink. This module owns no new
//! snapshot/quiesce logic — `Engine::snapshot()` behind that endpoint is the
//! same quiesce-free call the raft snapshotter itself uses; this is transport
//! + scheduling only, meant to be driven by the operator's optional backup
//! CronJob (`spec.serving.backup`, see `service_k8s::render::backup_cron_job`)
//! or invoked ad hoc via the CLI.

use anyhow::{bail, Context, Result};
pub use service_backup::{
    fetch_admin_snapshot as fetch_snapshot_bytes, run_admin_snapshot_backup as run_backup,
};

/// POST exact SnapshotV1 JSON bytes to `{base_url}/admin/restore` (Bearer
/// `token` when set).
/// @spec apps/lumen/tech-design/interfaces/cli/lumen-cli-add-dump-load-export-import-snapshot-verbs.md#logic
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

#[cfg(test)]
mod tests {
    use super::*;
    use service_backup::{BackupDestination, RetentionPolicy};
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
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/src/backup.rs
    action: create
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      New module (#808), hand-written: no generator primitive exists yet for
      an HTTP-client admin-backup fetch + service-backup destination-sink
      shape. run_backup(base_url, token, dest, retention) fetches
      {base_url}/admin/backup (Bearer auth when token is Some) and hands the
      bytes to service_backup::run_backup_once against
      sink_from_destination(dest) and the given RetentionPolicy.
```
<!-- HANDWRITE-END -->
