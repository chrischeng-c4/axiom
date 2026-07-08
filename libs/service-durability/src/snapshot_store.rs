// SPEC-MANAGED: libs/service-durability/tech-design/semantic/source/libs-service-durability-src-snapshot_store-rs.md#rust-source-unit
// CODEGEN-BEGIN
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::{atomic_write, FsyncPolicy};

/// One sequence-named snapshot file.
#[derive(Debug, Clone, PartialEq, Eq)]
/// @spec libs/service-durability/tech-design/semantic/source/libs-service-durability-src-snapshot_store-rs.md#source
pub struct SnapshotFile {
    pub seq: u64,
    pub path: PathBuf,
}

/// Local sequence-named snapshot store.
#[derive(Debug, Clone)]
/// @spec libs/service-durability/tech-design/semantic/source/libs-service-durability-src-snapshot_store-rs.md#source
pub struct SnapshotFileStore {
    root: PathBuf,
    prefix: String,
    extension: String,
    policy: FsyncPolicy,
}

/// @spec libs/service-durability/tech-design/semantic/source/libs-service-durability-src-snapshot_store-rs.md#source
impl SnapshotFileStore {
    pub fn new(
        root: impl Into<PathBuf>,
        prefix: impl Into<String>,
        extension: impl Into<String>,
        policy: FsyncPolicy,
    ) -> Result<Self> {
        let root = root.into();
        std::fs::create_dir_all(&root)
            .with_context(|| format!("create snapshot dir {}", root.display()))?;
        Ok(Self {
            root,
            prefix: prefix.into(),
            extension: extension.into(),
            policy,
        })
    }

    pub fn save(&self, seq: u64, bytes: &[u8]) -> Result<PathBuf> {
        let path = self.path_for(seq);
        atomic_write(&path, bytes, self.policy)?;
        Ok(path)
    }

    pub fn load_latest(&self) -> Result<Option<Vec<u8>>> {
        let Some(snapshot) = self.snapshots()?.into_iter().last() else {
            return Ok(None);
        };
        std::fs::read(&snapshot.path)
            .with_context(|| format!("read snapshot {}", snapshot.path.display()))
            .map(Some)
    }

    pub fn snapshots(&self) -> Result<Vec<SnapshotFile>> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(&self.root)
            .with_context(|| format!("read snapshot dir {}", self.root.display()))?
        {
            let path = entry?.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some(self.extension.as_str()) {
                if let Some(seq) = self.seq_of(&path) {
                    out.push(SnapshotFile { seq, path });
                }
            }
        }
        out.sort_by_key(|snapshot| snapshot.seq);
        Ok(out)
    }

    pub fn prune(&self, keep: usize) -> Result<usize> {
        let all = self.snapshots()?;
        if all.len() <= keep {
            return Ok(0);
        }
        let to_drop = all.len() - keep;
        let mut removed = 0usize;
        for snapshot in all.into_iter().take(to_drop) {
            if std::fs::remove_file(&snapshot.path).is_ok() {
                removed += 1;
            }
        }
        Ok(removed)
    }

    fn path_for(&self, seq: u64) -> PathBuf {
        self.root
            .join(format!("{}-{}.{}", self.prefix, seq, self.extension))
    }

    fn seq_of(&self, path: &Path) -> Option<u64> {
        path.file_stem()?
            .to_str()?
            .strip_prefix(&format!("{}-", self.prefix))?
            .parse()
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_latest_and_prune_use_sequence_order() {
        let dir = tempfile::tempdir().unwrap();
        let store = SnapshotFileStore::new(dir.path(), "snap", "bin", FsyncPolicy::Always).unwrap();
        store.save(1, b"one").unwrap();
        store.save(3, b"three").unwrap();
        store.save(2, b"two").unwrap();
        assert_eq!(store.load_latest().unwrap().unwrap(), b"three");
        assert_eq!(store.prune(1).unwrap(), 2);
        assert_eq!(store.snapshots().unwrap()[0].seq, 3);
    }
}
// CODEGEN-END
