// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-src.md#schema
// CODEGEN-BEGIN
//! Copy-on-write workspace + filesystem diffing.
//!
//! A vat's `rootfs` is a copy-on-write clone of its base. On APFS (macOS) this
//! is `clonefile(2)`: cloning a whole directory tree is a near-instant
//! metadata operation that shares blocks until written. On Linux we try a
//! reflink copy (`cp --reflink=auto`) and fall back to a plain recursive copy.
//!
//! Diffing is how an agent learns "what did my run change". At clone time we
//! capture a [`Manifest`] (path → size + mtime) as the baseline; after a run
//! we re-walk the rootfs and compare. v1 uses size+mtime (cheap, good enough
//! to spot changes); content hashing is a tracked refinement.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::UNIX_EPOCH;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::state::ChangeSet;

/// Per-file stat used for cheap change detection.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-overlay-rs.md#source
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileStat {
    pub size: u64,
    /// Modification time, ms since the Unix epoch.
    pub mtime_ms: i64,
}

/// Map of rootfs-relative path → stat. Sorted for stable diffs and output.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-overlay-rs.md#source
pub type Manifest = BTreeMap<String, FileStat>;

/// Copy-on-write clone of `src` into `dst`. `dst` must not already exist.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-overlay-rs.md#source
pub fn clone_tree(src: &Path, dst: &Path) -> Result<()> {
    if dst.exists() {
        bail!("clone target already exists: {}", dst.display());
    }
    if !src.exists() {
        bail!("clone source does not exist: {}", src.display());
    }
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create parent of {}", dst.display()))?;
    }

    if destination_is_inside_source(src, dst) {
        return copy_recursive(src, dst);
    }

    #[cfg(target_os = "macos")]
    {
        clonefile_macos(src, dst)
    }
    #[cfg(not(target_os = "macos"))]
    {
        clone_tree_portable(src, dst)
    }
}

/// macOS: one `clonefile(2)` clones the entire tree, copy-on-write.
#[cfg(target_os = "macos")]
fn clonefile_macos(src: &Path, dst: &Path) -> Result<()> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let c_src = CString::new(src.as_os_str().as_bytes()).context("src path has NUL byte")?;
    let c_dst = CString::new(dst.as_os_str().as_bytes()).context("dst path has NUL byte")?;
    // clonefile(const char *src, const char *dst, int flags)
    let rc = unsafe { libc::clonefile(c_src.as_ptr(), c_dst.as_ptr(), 0) };
    if rc != 0 {
        let err = std::io::Error::last_os_error();
        // Fall back to a portable copy if the volume isn't APFS or clonefile
        // is otherwise unhappy — correctness over speed.
        eprintln!(
            "vat: clonefile failed ({err}); falling back to recursive copy. \
             (COW disabled — is the workspace on a non-APFS volume?)"
        );
        return copy_recursive(src, dst);
    }
    Ok(())
}

/// Portable clone: reflink via `cp` if available, else a plain recursive copy.
#[cfg(not(target_os = "macos"))]
fn clone_tree_portable(src: &Path, dst: &Path) -> Result<()> {
    // Try reflink first (instant COW on btrfs/xfs); `--reflink=auto` degrades
    // to a normal copy when unsupported, so this single call is enough on
    // Linux. We still keep a manual fallback for hosts without GNU cp.
    let reflink = std::process::Command::new("cp")
        .args(["-a", "--reflink=auto"])
        .arg(src)
        .arg(dst)
        .status();
    match reflink {
        Ok(s) if s.success() => return Ok(()),
        _ => {}
    }
    copy_recursive(src, dst)
}

/// Last-resort recursive copy (used as the universal fallback on every host).
fn copy_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in WalkDir::new(src)
        .min_depth(1)
        .into_iter()
        .filter_entry(|entry| should_copy_entry(src, dst, entry.path()))
    {
        let entry = entry?;
        let rel = entry.path().strip_prefix(src)?;
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn destination_is_inside_source(src: &Path, dst: &Path) -> bool {
    match (
        std::fs::canonicalize(src),
        dst.parent()
            .and_then(|parent| std::fs::canonicalize(parent).ok()),
    ) {
        (Ok(src), Some(dst_parent)) => dst_parent.starts_with(src),
        _ => false,
    }
}

fn should_copy_entry(src: &Path, dst: &Path, path: &Path) -> bool {
    let Ok(rel) = path.strip_prefix(src) else {
        return true;
    };
    if rel
        .components()
        .any(|component| component.as_os_str() == ".vat")
    {
        return false;
    }
    !(path.starts_with(dst) || dst.starts_with(path))
}

/// Walk `root` and record a stat manifest of every regular file. Symlinks are
/// not followed (we record the link's own stat); directories are implied by
/// their files.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-overlay-rs.md#source
pub fn manifest_of(root: &Path) -> Result<Manifest> {
    let mut m = Manifest::new();
    for entry in WalkDir::new(root).min_depth(1).follow_links(false) {
        let entry = entry.with_context(|| format!("walk {}", root.display()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(root)
            .context("strip rootfs prefix")?
            .to_string_lossy()
            .into_owned();
        let meta = entry.metadata().context("stat file")?;
        let mtime_ms = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        m.insert(
            rel,
            FileStat {
                size: meta.len(),
                mtime_ms,
            },
        );
    }
    Ok(m)
}

/// Diff a current manifest against the captured baseline.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-overlay-rs.md#source
pub fn diff(base: &Manifest, now: &Manifest) -> ChangeSet {
    let mut cs = ChangeSet::default();
    for (path, stat) in now {
        match base.get(path) {
            None => cs.added.push(path.clone()),
            Some(old) if old != stat => cs.modified.push(path.clone()),
            Some(_) => {}
        }
    }
    for path in base.keys() {
        if !now.contains_key(path) {
            cs.deleted.push(path.clone());
        }
    }
    cs
}

/// Persist a manifest as pretty JSON.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-overlay-rs.md#source
pub fn save_manifest(path: &Path, m: &Manifest) -> Result<()> {
    let json = serde_json::to_vec_pretty(m).context("serialize manifest")?;
    std::fs::write(path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

/// Load a previously saved manifest.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-overlay-rs.md#source
pub fn load_manifest(path: &Path) -> Result<Manifest> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&bytes).context("parse manifest")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_local_vat_store_is_not_cloned_into_rootfs() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        std::fs::create_dir_all(repo.join(".git")).expect("git dir");
        std::fs::write(repo.join("source.txt"), "kept").expect("source file");
        std::fs::create_dir_all(repo.join(".vat/vats/old/rootfs")).expect("old vat");
        std::fs::write(repo.join(".vat/vats/old/rootfs/leak.txt"), "skip").expect("vat file");

        let dst = repo.join(".vat/vats/new/rootfs");
        clone_tree(&repo, &dst).expect("clone tree");

        assert_eq!(
            std::fs::read_to_string(dst.join("source.txt")).expect("copied file"),
            "kept"
        );
        assert!(
            !dst.join(".vat").exists(),
            "repo-local vat state must never be copied into a vat rootfs"
        );
    }
}
// CODEGEN-END
