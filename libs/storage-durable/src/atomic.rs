// CODEGEN-BEGIN
//! Replace a file's contents so that a crash leaves either the old bytes or
//! the new ones, never a prefix of the new ones.
//!
//! The order is what buys that, and all four steps are load-bearing: write the
//! temp file, fsync it, rename it over the target, then fsync the target's
//! parent directory. Skipping the last one leaves the rename itself in the
//! directory's write-back cache, so the commit -- not the payload -- is what a
//! crash loses.
//!
//! Two deliberate limits a caller has to know about:
//!
//! - The temp file is the fixed sibling `<path>.tmp`, not a randomised name, and
//!   a leftover one is removed rather than recovered. Two concurrent
//!   `atomic_write` calls to the same path therefore race, and one of them wins
//!   silently. **One writer per path** is a precondition, not a hint.
//! - [`sync_parent_dir`] treats `PermissionDenied`, `Unsupported` and `Other`
//!   from opening the directory as success, because some platforms and sandboxes
//!   do not allow opening a directory as a file. On those, the parent fsync is
//!   quietly not happening and the durability of the commit is whatever the OS
//!   orders by itself -- there is no error to observe.
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

/// Strictly fsync the parent directory of `path`.
///
/// Unlike [`sync_parent_dir`], this function never treats an unsupported or
/// denied directory open as success. Use it for a commit protocol that must not
/// report success without a proven directory fsync.
pub fn strict_sync_parent_dir(path: impl AsRef<Path>) -> Result<()> {
    let parent = strict_parent(path.as_ref());
    let dir = OpenOptions::new()
        .read(true)
        .open(parent)
        .with_context(|| format!("open dir {} for strict fsync", parent.display()))?;
    dir.sync_all()
        .with_context(|| format!("strict fsync dir {}", parent.display()))
}

fn strict_parent(path: &Path) -> &Path {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
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

    #[test]
    fn strict_parent_sync_accepts_a_real_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.bin");
        std::fs::write(&path, b"state").unwrap();
        strict_sync_parent_dir(&path).unwrap();
    }

    #[test]
    fn strict_parent_sync_resolves_a_bare_filename_to_the_working_directory() {
        assert_eq!(strict_parent(Path::new("state.bin")), Path::new("."));
    }
}
// CODEGEN-END
