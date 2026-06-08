// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
// CODEGEN-BEGIN
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Store garbage collector — removes unreferenced packages from ~/.jet-store/.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub struct StoreGc {
    store_path: PathBuf,
}

/// GC result summary.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug)]
pub struct GcResult {
    pub removed: usize,
    pub reclaimed_bytes: u64,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
impl StoreGc {
    pub fn new(store_path: PathBuf) -> Self {
        Self { store_path }
    }

    /// Prune unreferenced packages from the store.
    ///
    /// 1. Scan all jet-lock.yaml files under known project directories
    /// 2. Build a set of referenced package@version keys
    /// 3. Walk the store and delete any directory not in the ref set
    pub fn prune(&self, project_roots: &[PathBuf]) -> Result<GcResult> {
        let referenced = self.collect_references(project_roots)?;
        let stored = self.list_store_entries()?;

        let mut removed = 0;
        let mut reclaimed_bytes = 0u64;

        for entry_name in &stored {
            if !referenced.contains(entry_name) {
                let entry_path = self.store_path.join(entry_name);
                let size = dir_size(&entry_path);
                tracing::info!("Removing orphan: {} ({} bytes)", entry_name, size);

                if let Err(e) = std::fs::remove_dir_all(&entry_path) {
                    tracing::warn!("Failed to remove {}: {}", entry_name, e);
                    continue;
                }
                removed += 1;
                reclaimed_bytes += size;
            }
        }

        Ok(GcResult {
            removed,
            reclaimed_bytes,
        })
    }

    /// Collect all referenced packages from lockfiles.
    fn collect_references(&self, project_roots: &[PathBuf]) -> Result<HashSet<String>> {
        let mut refs = HashSet::new();

        for root in project_roots {
            let lockfile_path = root.join("jet-lock.yaml");
            if !lockfile_path.exists() {
                continue;
            }

            let content = std::fs::read_to_string(&lockfile_path)?;
            let lockfile: serde_yaml::Value = serde_yaml::from_str(&content)?;

            if let Some(packages) = lockfile.get("packages").and_then(|p| p.as_mapping()) {
                for (key, entry) in packages {
                    if let (Some(key_str), Some(version)) =
                        (key.as_str(), entry.get("version").and_then(|v| v.as_str()))
                    {
                        // Key format: /name@version or /@scope/name@version
                        let name = key_str
                            .trim_start_matches('/')
                            .rsplit_once('@')
                            .map(|(n, _)| n)
                            .unwrap_or(key_str.trim_start_matches('/'));
                        refs.insert(format!("{}@{}", name, version));
                    }
                }
            }
        }

        Ok(refs)
    }

    /// List all directories in the store.
    fn list_store_entries(&self) -> Result<Vec<String>> {
        let mut entries = Vec::new();
        if !self.store_path.exists() {
            return Ok(entries);
        }

        for entry in std::fs::read_dir(&self.store_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    entries.push(name.to_string());
                }
            }
        }

        Ok(entries)
    }
}

/// Format the warning emitted when a per-entry walkdir failure during
/// `dir_size` silently drops bytes from the GC reclaim total. Names the
/// offending path verbatim, preserves the underlying walkdir error, and
/// tags `GH #3534` so users grepping for "freed X bytes doesn't match
/// reality" can land on this line. Extracted for unit-test pinning.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_gc_walkdir_warn(root: &Path, err: &walkdir::Error) -> String {
    // `walkdir::Error::path()` reports the path the error happened at; fall
    // back to the root when walkdir can't tell us (rare — typically only
    // when the *root* itself is unreadable, which `.path()` returns None
    // for).
    let blamed = err
        .path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| root.display().to_string());
    format!(
        "GH #3534 walkdir entry skipped under {} at {}: {}; bytes in this entry will NOT be counted toward the GC reclaim total. The actual on-disk free space may exceed jet's reported `freed` value.",
        root.display(),
        blamed,
        err
    )
}

/// Format the warning emitted when a per-file `metadata()` failure during
/// `dir_size` silently drops bytes from the GC reclaim total. Same shape
/// as `format_gc_walkdir_warn`. Tagged `GH #3534`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_gc_metadata_warn(path: &Path, err: &walkdir::Error) -> String {
    format!(
        "GH #3534 walkdir metadata read skipped at {}: {}; this file's bytes will NOT be counted toward the GC reclaim total. The actual on-disk free space may exceed jet's reported `freed` value.",
        path.display(),
        err
    )
}

/// Calculate total size of a directory.
///
/// GH #3534 — the prior `.filter_map(|e| e.ok())` chain silently dropped
/// per-entry and per-metadata errors, leaving the GC's reported
/// reclaimed-bytes total out of sync with the actual on-disk free space
/// when the store contained a broken symlink or an unreadable file. We
/// now match on each step so an operator sees a structured warn per
/// skip and can chase the offending entry (typical causes: broken
/// symlinks pointing at a deleted package, EACCES on a misowned file in
/// the store).
fn dir_size(path: &Path) -> u64 {
    let mut total: u64 = 0;
    for entry_res in walkdir::WalkDir::new(path) {
        let entry = match entry_res {
            Ok(e) => e,
            Err(err) => {
                tracing::warn!(
                    target: "jet::pkg_manager::gc",
                    root = %path.display(),
                    error = %err,
                    "{}",
                    format_gc_walkdir_warn(path, &err)
                );
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }
        match entry.metadata() {
            Ok(meta) => total = total.saturating_add(meta.len()),
            Err(err) => {
                tracing::warn!(
                    target: "jet::pkg_manager::gc",
                    path = %entry.path().display(),
                    error = %err,
                    "{}",
                    format_gc_metadata_warn(entry.path(), &err)
                );
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// GH #3534 — produce a real `walkdir::Error` by walking a path that
    /// doesn't exist. This is the only portable way to fabricate the
    /// error type for message-shape tests.
    fn make_walkdir_err() -> walkdir::Error {
        walkdir::WalkDir::new("/nonexistent/path/for/gh3534/test")
            .into_iter()
            .next()
            .unwrap()
            .unwrap_err()
    }

    #[test]
    fn gh3534_format_gc_walkdir_warn_names_root_and_error_and_issue() {
        let root = PathBuf::from("/home/user/.jet-store");
        let err = make_walkdir_err();
        let msg = format_gc_walkdir_warn(&root, &err);
        assert!(
            msg.contains("/home/user/.jet-store"),
            "warning must name the store root verbatim: {msg}"
        );
        assert!(
            msg.contains("GH #3534"),
            "warning must carry the GH #3534 tag so users can grep their logs: {msg}"
        );
        assert!(
            msg.contains("reclaim total") || msg.contains("reclaim"),
            "warning must mention the GC reclaim total so users debugging 'jet freed less than df shows' can find this line: {msg}"
        );
    }

    #[test]
    fn gh3534_format_gc_metadata_warn_names_path_and_error_and_issue() {
        let path = PathBuf::from("/home/user/.jet-store/pkg@1.0.0/index.js");
        let err = make_walkdir_err();
        let msg = format_gc_metadata_warn(&path, &err);
        assert!(
            msg.contains("/home/user/.jet-store/pkg@1.0.0/index.js"),
            "warning must name the offending file verbatim: {msg}"
        );
        assert!(
            msg.contains("GH #3534"),
            "warning must carry the GH #3534 tag: {msg}"
        );
        assert!(
            msg.contains("reclaim total") || msg.contains("reclaim"),
            "warning must mention the GC reclaim total: {msg}"
        );
    }

    /// GH #3534 — end-to-end: dir_size against a tree where one subdir
    /// is chmod-000 must not panic and must return a partial sum (the
    /// readable siblings' bytes) instead of either zeroing out or
    /// aborting.
    #[cfg(unix)]
    #[test]
    fn gh3534_dir_size_partial_sum_on_unreadable_subdir() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let root = dir.path();

        // Readable file at the root contributes 7 bytes.
        std::fs::write(root.join("keep.bin"), b"keep me").unwrap();

        // Unreadable subdir — walkdir will see a per-entry Err while
        // trying to descend into it.
        let locked = root.join("locked");
        std::fs::create_dir(&locked).unwrap();
        std::fs::write(locked.join("hidden.bin"), b"hidden bytes").unwrap();
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();

        // Skip cleanly when running as root (chmod has no effect).
        if std::fs::read_dir(&locked).is_ok() {
            std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
            return;
        }

        let size = dir_size(root);

        // Restore perms so tempdir cleanup can succeed.
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();

        // The readable file's bytes must survive; the hidden bytes are
        // intentionally absent. The partial sum is the user-visible
        // contract this PR pins (operator sees the warn line).
        assert!(
            size >= 7,
            "readable sibling's bytes must survive an unreadable subdir: got {size}, expected at least 7"
        );
    }

    #[test]
    fn test_prune_empty_store() {
        let store_dir = tempdir().unwrap();
        let gc = StoreGc::new(store_dir.path().to_path_buf());
        let result = gc.prune(&[]).unwrap();
        assert_eq!(result.removed, 0);
    }

    #[test]
    fn test_prune_orphans() {
        let store_dir = tempdir().unwrap();

        // Create a fake store entry
        let orphan = store_dir.path().join("orphan-pkg@1.0.0");
        std::fs::create_dir_all(&orphan).unwrap();
        std::fs::write(orphan.join("index.js"), "// orphan").unwrap();

        let gc = StoreGc::new(store_dir.path().to_path_buf());
        let result = gc.prune(&[]).unwrap();
        assert_eq!(result.removed, 1);
        assert!(!orphan.exists());
    }

    #[test]
    fn test_prune_keeps_referenced() {
        let store_dir = tempdir().unwrap();
        let project_dir = tempdir().unwrap();

        // Create store entry
        let pkg = store_dir.path().join("lodash@4.17.21");
        std::fs::create_dir_all(&pkg).unwrap();

        // Create lockfile referencing it
        std::fs::write(
            project_dir.path().join("jet-lock.yaml"),
            "lockfileVersion: '2.0'\npackages:\n  /lodash@4.17.21:\n    version: '4.17.21'\n    resolution:\n      tarball: https://example.com/lodash.tgz\n      shasum: abc\n",
        )
        .unwrap();

        let gc = StoreGc::new(store_dir.path().to_path_buf());
        let result = gc.prune(&[project_dir.path().to_path_buf()]).unwrap();
        assert_eq!(result.removed, 0);
        assert!(pkg.exists());
    }
}
// CODEGEN-END
