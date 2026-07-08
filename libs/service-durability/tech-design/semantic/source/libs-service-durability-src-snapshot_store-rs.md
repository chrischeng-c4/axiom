---
id: libs-service-durability-src-snapshot-store-rs
summary: Lossless rust-source-unit coverage for `libs/service-durability/src/snapshot_store.rs`.
capability_refs:
  - id: shared-service-durability-contract
    role: primary
    gap: shared-service-durability-contract
    claim: shared-service-durability-contract
    coverage: full
    rationale: "The source unit implements sequence-named local snapshot persistence."
fill_sections: [overview, source, changes]
---

# Standardized libs/service-durability/src/snapshot_store.rs

## Overview
<!-- type: overview lang: markdown -->

Lossless rust-source-unit coverage for `libs/service-durability/src/snapshot_store.rs`.

### Symbols

| Name | Target | Kind | Visibility | Signature |
|------|--------|------|------------|-----------|
| `SnapshotFile` | libs/service-durability/src/snapshot_store.rs | struct | pub | SnapshotFile { seq, path } |
| `SnapshotFileStore` | libs/service-durability/src/snapshot_store.rs | struct | pub | SnapshotFileStore |
| `new` | libs/service-durability/src/snapshot_store.rs | method | pub | new(root, prefix, extension, policy) -> Result<Self> |
| `save` | libs/service-durability/src/snapshot_store.rs | method | pub | save(&self, seq, bytes) -> Result<PathBuf> |
| `load_latest` | libs/service-durability/src/snapshot_store.rs | method | pub | load_latest(&self) -> Result<Option<Vec<u8>>> |
| `snapshots` | libs/service-durability/src/snapshot_store.rs | method | pub | snapshots(&self) -> Result<Vec<SnapshotFile>> |
| `prune` | libs/service-durability/src/snapshot_store.rs | method | pub | prune(&self, keep) -> Result<usize> |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::{atomic_write, FsyncPolicy};

/// One sequence-named snapshot file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotFile {
    pub seq: u64,
    pub path: PathBuf,
}

/// Local sequence-named snapshot store.
#[derive(Debug, Clone)]
pub struct SnapshotFileStore {
    root: PathBuf,
    prefix: String,
    extension: String,
    policy: FsyncPolicy,
}

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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/service-durability/src/snapshot_store.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/service-durability/src/snapshot_store.rs`.
```
