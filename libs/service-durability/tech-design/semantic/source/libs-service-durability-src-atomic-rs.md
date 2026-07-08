---
id: libs-service-durability-src-atomic-rs
summary: Lossless rust-source-unit coverage for `libs/service-durability/src/atomic.rs`.
capability_refs:
  - id: shared-service-durability-contract
    role: primary
    gap: shared-service-durability-contract
    claim: shared-service-durability-contract
    coverage: full
    rationale: "The source unit implements shared atomic durable file replacement."
fill_sections: [overview, source, changes]
---

# Standardized libs/service-durability/src/atomic.rs

## Overview
<!-- type: overview lang: markdown -->

Lossless rust-source-unit coverage for `libs/service-durability/src/atomic.rs`.

### Symbols

| Name | Target | Kind | Visibility | Signature |
|------|--------|------|------------|-----------|
| `atomic_write` | libs/service-durability/src/atomic.rs | function | pub | atomic_write(path: impl AsRef<Path>, bytes: &[u8], policy: FsyncPolicy) -> Result<()> |
| `sync_parent_dir` | libs/service-durability/src/atomic.rs | function | pub | sync_parent_dir(path: impl AsRef<Path>) -> Result<()> |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use anyhow::{Context, Result};

use crate::FsyncPolicy;

/// Write `bytes` to `path` through a temp file, fsync according to `policy`,
/// then atomically rename it into place.
pub fn atomic_write(path: impl AsRef<Path>, bytes: &[u8], policy: FsyncPolicy) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create durability dir {}", parent.display()))?;
        }
    }
    let tmp = temp_path(path);
    let _ = std::fs::remove_file(&tmp);
    {
        let mut file =
            File::create(&tmp).with_context(|| format!("create temp {}", tmp.display()))?;
        file.write_all(bytes)
            .with_context(|| format!("write temp {}", tmp.display()))?;
        if policy != FsyncPolicy::Os {
            file.sync_all()
                .with_context(|| format!("fsync temp {}", tmp.display()))?;
        }
    }
    std::fs::rename(&tmp, path).with_context(|| {
        format!(
            "commit durable replace {} -> {}",
            tmp.display(),
            path.display()
        )
    })?;
    if policy != FsyncPolicy::Os {
        sync_parent_dir(path)?;
    }
    Ok(())
}

/// Best-effort directory fsync on platforms that support opening directories.
pub fn sync_parent_dir(path: impl AsRef<Path>) -> Result<()> {
    let Some(parent) = path.as_ref().parent() else {
        return Ok(());
    };
    if parent.as_os_str().is_empty() {
        return Ok(());
    }
    match OpenOptions::new().read(true).open(parent) {
        Ok(dir) => {
            dir.sync_all()
                .with_context(|| format!("fsync dir {}", parent.display()))?;
            Ok(())
        }
        Err(err) if is_directory_open_unsupported(&err) => Ok(()),
        Err(err) => Err(err).with_context(|| format!("open dir {}", parent.display())),
    }
}

fn temp_path(path: &Path) -> std::path::PathBuf {
    let mut tmp = path.as_os_str().to_os_string();
    tmp.push(".tmp");
    tmp.into()
}

fn is_directory_open_unsupported(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::PermissionDenied | io::ErrorKind::Unsupported | io::ErrorKind::Other
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_replaces_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.bin");
        atomic_write(&path, b"one", FsyncPolicy::Always).unwrap();
        atomic_write(&path, b"two", FsyncPolicy::Always).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"two");
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/service-durability/src/atomic.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/service-durability/src/atomic.rs`.
```
