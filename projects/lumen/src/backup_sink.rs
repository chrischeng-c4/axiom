// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-backup_sink-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Pluggable destination for periodic backups.
//!
//! README §7 specifies that backups can be uploaded to S3 / GCS with a
//! pluggable adapter and a retention policy. v1 ships the trait plus a
//! local-filesystem implementation; cloud-object-store adapters live as
//! optional features (`backup-s3`, `backup-gcs`) wired in a follow-up
//! once the relevant SDK crates land in the workspace.
//!
//! ## Wire shape
//!
//! Each `BackupSink::put` call receives the serialised `SnapshotV1`
//! JSON (already produced by `Engine::snapshot`) plus a timestamp the
//! caller chose for the key. Sinks are responsible for retention.
//!
//! Methods are synchronous — callers run them via
//! `tokio::task::spawn_blocking` when they live inside an async runtime.
//! This keeps the trait dep-free and means cloud-backed adapters can
//! later switch to async without churning the rest of the codebase
//! (they just provide both a sync and async path).

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};

/// A backup destination.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-backup_sink-rs.md#source
pub trait BackupSink: Send + Sync + 'static {
    /// Upload the snapshot bytes under a key derived from `timestamp`.
    /// Sinks may add a prefix / namespace; returns the key used.
    fn put(&self, timestamp: SystemTime, payload: &[u8]) -> Result<String>;

    /// Apply retention: drop backups older than `max_age_seconds`.
    /// Returns the number of objects removed.
    fn prune(&self, max_age_seconds: u64) -> Result<usize>;

    /// Sink identity for logs / metrics (`"local:/path"`, `"s3://bucket"`,
    /// `"gs://bucket"`).
    fn identity(&self) -> String;
}

/// Local-filesystem sink. Useful for dev, integration tests, and the
/// PVC-backed durable-store cohort that doesn't want a cloud dependency.
#[derive(Debug, Clone)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-backup_sink-rs.md#source
pub struct LocalFsSink {
    pub root: PathBuf,
    pub prefix: String,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-backup_sink-rs.md#source
impl LocalFsSink {
    pub fn new(root: impl Into<PathBuf>, prefix: impl Into<String>) -> Result<Self> {
        let root = root.into();
        std::fs::create_dir_all(&root)
            .with_context(|| format!("create backup dir {}", root.display()))?;
        Ok(Self {
            root,
            prefix: prefix.into(),
        })
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-backup_sink-rs.md#source
impl BackupSink for LocalFsSink {
    fn put(&self, timestamp: SystemTime, payload: &[u8]) -> Result<String> {
        let ts = timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let name = format!("{}-{ts}.json", self.prefix);
        let path = self.root.join(&name);
        std::fs::write(&path, payload).with_context(|| format!("write {}", path.display()))?;
        Ok(name)
    }

    fn prune(&self, max_age_seconds: u64) -> Result<usize> {
        let cutoff = SystemTime::now() - Duration::from_secs(max_age_seconds);
        let mut removed = 0usize;
        for entry in std::fs::read_dir(&self.root)? {
            let entry = entry?;
            let modified = entry.metadata()?.modified()?;
            if modified < cutoff {
                std::fs::remove_file(entry.path())?;
                removed += 1;
            }
        }
        Ok(removed)
    }

    fn identity(&self) -> String {
        format!("local:{}", self.root.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_fs_sink_round_trip() {
        let dir = std::env::temp_dir().join(format!("lumen-backup-test-{}", std::process::id()));
        let sink = LocalFsSink::new(&dir, "lumen").unwrap();

        let name = sink.put(SystemTime::now(), b"{\"version\":1}").unwrap();
        let path = dir.join(&name);
        assert!(path.exists());
        let contents = std::fs::read(&path).unwrap();
        assert_eq!(contents, b"{\"version\":1}");

        // Prune with one-hour age — file is brand new, nothing removed.
        assert_eq!(sink.prune(3600).unwrap(), 0);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn identity_format() {
        let dir = std::env::temp_dir().join(format!("lumen-backup-id-{}", std::process::id()));
        let sink = LocalFsSink::new(&dir, "lumen").unwrap();
        assert!(sink.identity().starts_with("local:"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn prune_removes_files_older_than_cutoff() {
        let dir = std::env::temp_dir().join(format!("lumen-backup-prune-{}", std::process::id()));
        let sink = LocalFsSink::new(&dir, "lumen").unwrap();
        sink.put(SystemTime::now(), b"{}").unwrap();
        // Cutoff = 0 seconds → file should be considered too old.
        std::thread::sleep(Duration::from_millis(5));
        let removed = sink.prune(0).unwrap();
        assert_eq!(removed, 1);
        // Directory now empty.
        let entries: Vec<_> = std::fs::read_dir(&dir).unwrap().collect();
        assert!(entries.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn new_creates_dir_if_missing() {
        let dir = std::env::temp_dir().join(format!(
            "lumen-backup-mkdir-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        assert!(!dir.exists());
        let _sink = LocalFsSink::new(&dir, "lumen").unwrap();
        assert!(dir.exists());
        std::fs::remove_dir_all(&dir).ok();
    }
}
// CODEGEN-END
