use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use anyhow::{bail, Context, Result};

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

/// Placeholder sink for a policy whose cloud adapter is not linked into this
/// runner binary. It fails loudly instead of silently writing elsewhere.
#[derive(Debug, Clone)]
pub struct UnsupportedCloudSink {
    pub destination: BackupDestination,
}

impl BackupSink for UnsupportedCloudSink {
    fn put(&self, _timestamp: SystemTime, _payload: &[u8]) -> Result<String> {
        bail!(
            "backup destination {} requires a cloud adapter feature in the runner",
            self.destination.identity()
        )
    }

    fn prune(&self, _max_age_seconds: u64) -> Result<usize> {
        bail!(
            "backup destination {} requires a cloud adapter feature in the runner",
            self.destination.identity()
        )
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
        BackupDestination::S3 { .. } | BackupDestination::Gcs { .. } => {
            Ok(Box::new(UnsupportedCloudSink {
                destination: destination.clone(),
            }))
        }
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
    fn cloud_sink_is_explicitly_unsupported_without_adapter() {
        let dest = BackupDestination::from_uri("s3://bucket/prefix").unwrap();
        let sink = sink_from_destination(&dest).unwrap();
        assert!(sink.put(SystemTime::now(), b"x").is_err());
    }
}
