// CODEGEN-BEGIN
//! The sink trait, the local filesystem sink, and the sink that refuses.
//!
//! `sink_from_destination` is the only place a destination becomes a sink, which
//! is what lets [`UnsupportedCloudSink`] carry two
//! `unreachable!()` arms honestly: local always maps to `LocalFsSink` and GCS
//! always maps to `GcsSink`, so only the S3 variant can ever reach it. GCS is
//! always linked and S3 is behind the `s3` feature -- that asymmetry is the
//! reason this file has a refusing sink at all. Without the feature, an `s3://`
//! destination still parses and still constructs; it fails on `put`/`prune` with
//! a message naming the rebuild flag, rather than quietly writing somewhere else.
//!
//! [`LocalFsSink`] has two behaviours worth knowing before
//! pointing it at a directory:
//!
//! - Its key is `<prefix>-<unix_seconds>.json`, at second resolution, written
//!   through `atomic_write`. Two puts in the same second therefore replace each
//!   other -- atomically, and without any error to observe. (The S3 sink spells
//!   the same policy's key differently, as `<prefix>/backup-<unix>.json`.)
//! - `prune` is **indiscriminate**. It compares each directory entry's
//!   filesystem mtime against `now - max_age_seconds` and removes anything older,
//!   without looking at the name at all: not the prefix, not the `.json`
//!   extension, not the timestamp in the key. So two sinks sharing a root will
//!   delete each other's objects, and any unrelated file in that root is
//!   deleted too. **One sink per directory.** (The S3 sink's prune is the
//!   opposite: it only touches keys matching its own pattern, and it reads the
//!   age out of the key.)
//!
//! A `prune` that fails a `remove_file` propagates immediately, so the returned
//! count is only meaningful on `Ok` -- on `Err`, some files are already gone and
//! how many is not reported.
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use anyhow::{bail, Context, Result};
use storage_durable::{atomic_write, FsyncPolicy};

#[cfg(feature = "s3")]
use crate::s3::S3Sink;
use crate::{BackupDestination, GcsSink};

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
            BackupDestination::Gcs { .. } => unreachable!("GCS always uses GcsSink"),
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
        BackupDestination::Gcs { .. } => Ok(Box::new(GcsSink::from_destination(destination)?)),
        #[cfg(feature = "s3")]
        BackupDestination::S3 { .. } => Ok(Box::new(S3Sink::from_destination(destination)?)),
        #[cfg(not(feature = "s3"))]
        BackupDestination::S3 { .. } => Ok(Box::new(UnsupportedCloudSink {
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
    fn gcs_destination_constructs_real_sink_without_network_io() {
        let dest = BackupDestination::from_uri("gs://bucket/prefix").unwrap();
        let sink = sink_from_destination(&dest).unwrap();
        assert_eq!(sink.identity(), "gs://bucket/prefix");
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
// CODEGEN-END
