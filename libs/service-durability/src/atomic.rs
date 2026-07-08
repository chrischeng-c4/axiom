// SPEC-MANAGED: libs/service-durability/tech-design/semantic/source/libs-service-durability-src-atomic-rs.md#rust-source-unit
// CODEGEN-BEGIN
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use anyhow::{Context, Result};

use crate::FsyncPolicy;

/// Write `bytes` to `path` through a temp file, fsync according to `policy`,
/// then atomically rename it into place.
/// @spec libs/service-durability/tech-design/semantic/source/libs-service-durability-src-atomic-rs.md#source
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
/// @spec libs/service-durability/tech-design/semantic/source/libs-service-durability-src-atomic-rs.md#source
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
// CODEGEN-END
