---
id: libs-service-backup-src-sink-rs
summary: Lossless rust-source-unit coverage for `libs/service-backup/src/sink.rs`.
capability_refs:
  - id: shared-service-backup-contract
    role: primary
    claim: shared-service-backup-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Service Backup library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/service-backup/src/sink.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/service-backup/src/sink.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `BackupSink` | libs/service-backup/src/sink.rs | trait | pub | 11 | pub trait BackupSink: Send + Sync + 'static { |
| `LocalFsSink` | libs/service-backup/src/sink.rs | struct | pub | 24 | pub struct LocalFsSink { |
| `new` | libs/service-backup/src/sink.rs | function | pub | 30 | pub fn new(root: impl Into<PathBuf>, prefix: impl Into<String>) -> Result<Self> { |
| `from_destination` | libs/service-backup/src/sink.rs | function | pub | 40 | pub fn from_destination(destination: &BackupDestination) -> Result<Self> { |
| `UnsupportedCloudSink` | libs/service-backup/src/sink.rs | struct | pub | 84 | pub struct UnsupportedCloudSink { |
| `sink_from_destination` | libs/service-backup/src/sink.rs | function | pub | 120 | pub fn sink_from_destination(destination: &BackupDestination) -> Result<Box<dyn BackupSink>> { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use anyhow::{bail, Context, Result};
use service_durability::{atomic_write, FsyncPolicy};

#[cfg(feature = "s3")]
use crate::s3::S3Sink;
use crate::BackupDestination;

/// Destination for snapshot bytes.
pub trait BackupSink: Send + Sync + 'static {
    /// Store bytes under a key derived from `timestamp`; returns the final key.
    fn put(&self, timestamp: SystemTime, payload: &[u8]) -> Result<String>;

    /// Apply age retention and return number of objects removed.
    fn prune(&self, max_age_seconds: u64) -> Result<usize>;

    /// Human-readable sink identity for logs/status.
    fn identity(&self) -> String;
}

/// Local filesystem sink for dev/tests/PVC-backed local deployments.
#[derive(Debug, Clone)]
pub struct LocalFsSink {
    pub root: PathBuf,
    pub prefix: String,
}

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

    pub fn from_destination(destination: &BackupDestination) -> Result<Self> {
        match destination {
            BackupDestination::Local { path, prefix } => {
                Self::new(path, prefix.clone().unwrap_or_else(|| "backup".into()))
            }
            other => bail!("{} is not a local backup destination", other.identity()),
        }
    }
}

impl BackupSink for LocalFsSink {
    fn put(&self, timestamp: SystemTime, payload: &[u8]) -> Result<String> {
        let ts = timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let name = format!("{}-{ts}.json", self.prefix);
        let path = self.root.join(&name);
        atomic_write(&path, payload, FsyncPolicy::Always)
            .with_context(|| format!("write {}", path.display()))?;
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

/// Placeholder sink for a policy whose cloud adapter is not linked into this
/// runner binary. It fails loudly instead of silently writing elsewhere.
#[derive(Debug, Clone)]
pub struct UnsupportedCloudSink {
    pub destination: BackupDestination,
}

impl UnsupportedCloudSink {
    fn action_message(&self) -> String {
        match &self.destination {
            BackupDestination::Gcs { .. } => format!(
                "backup destination {} parses as GCS, but service-backup does not yet ship a GCS sink; use file:// or s3:// for production, or remove the gs:// config until GCS support lands",
                self.destination.identity()
            ),
            BackupDestination::S3 { .. } => format!(
                "backup destination {} requires the service-backup `s3` feature in the runner; rebuild with `--features s3` or use file://",
                self.destination.identity()
            ),
            BackupDestination::Local { .. } => {
                unreachable!("local destinations never use UnsupportedCloudSink")
            }
        }
    }
}

impl BackupSink for UnsupportedCloudSink {
    fn put(&self, _timestamp: SystemTime, _payload: &[u8]) -> Result<String> {
        bail!("{}", self.action_message())
    }

    fn prune(&self, _max_age_seconds: u64) -> Result<usize> {
        bail!("{}", self.action_message())
    }

    fn identity(&self) -> String {
        self.destination.identity()
    }
}

pub fn sink_from_destination(destination: &BackupDestination) -> Result<Box<dyn BackupSink>> {
    match destination {
        BackupDestination::Local { .. } => {
            Ok(Box::new(LocalFsSink::from_destination(destination)?))
        }
        #[cfg(feature = "s3")]
        BackupDestination::S3 { .. } => Ok(Box::new(S3Sink::from_destination(destination)?)),
        #[cfg(not(feature = "s3"))]
        BackupDestination::S3 { .. } | BackupDestination::Gcs { .. } => {
            Ok(Box::new(UnsupportedCloudSink {
                destination: destination.clone(),
            }))
        }
        #[cfg(feature = "s3")]
        BackupDestination::Gcs { .. } => Ok(Box::new(UnsupportedCloudSink {
            destination: destination.clone(),
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_sink_round_trip_and_prune() {
        let dir = std::env::temp_dir().join(format!("service-backup-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let sink = LocalFsSink::new(&dir, "svc").unwrap();
        let key = sink.put(SystemTime::now(), b"snapshot").unwrap();
        assert!(dir.join(&key).exists());
        std::thread::sleep(Duration::from_millis(5));
        assert_eq!(sink.prune(0).unwrap(), 1);
        assert!(std::fs::read_dir(&dir).unwrap().next().is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn gcs_sink_reports_actionable_unsupported_message() {
        let dest = BackupDestination::from_uri("gs://bucket/prefix").unwrap();
        let sink = sink_from_destination(&dest).unwrap();
        let err = sink.put(SystemTime::now(), b"x").unwrap_err().to_string();
        assert!(err.contains("does not yet ship a GCS sink"));
        assert!(err.contains("use file:// or s3://"));
    }

    #[cfg(not(feature = "s3"))]
    #[test]
    fn s3_sink_reports_feature_action_when_unlinked() {
        let dest = BackupDestination::from_uri("s3://bucket/prefix").unwrap();
        let sink = sink_from_destination(&dest).unwrap();
        let err = sink.put(SystemTime::now(), b"x").unwrap_err().to_string();
        assert!(err.contains("`s3` feature"));
        assert!(err.contains("--features s3"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/service-backup/src/sink.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/service-backup/src/sink.rs` captured during libs codegen standardization.
```
