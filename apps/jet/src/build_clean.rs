// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-src.md#schema
// CODEGEN-BEGIN
//! Safe `--empty-out-dir` implementation for `jet build`.
//!
//! Vite/Webpack-style clean of the output directory before writing the new
//! bundle. Refuses to delete dangerous targets (empty path, filesystem root,
//! project root, current working directory) and requires explicit opt-in
//! (`force = true`) before touching anything outside the project root.

use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};

/// Reason a clean request was rejected.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CleanRejection {
    EmptyPath,
    FilesystemRoot,
    ProjectRoot,
    CurrentWorkingDirectory,
    OutsideProjectRootWithoutForce {
        resolved: PathBuf,
    },
    /// `target.canonicalize()` failed. The underlying io::Error message is
    /// carried in `reason` so a user debugging "why was my clean rejected?"
    /// gets the actual OS error (EACCES, ELOOP, EIO, ENOENT mid-traversal,
    /// EMFILE) instead of a fixed "symlink traversal" message that hid the
    /// real cause (GH #3538).
    CannotCanonicalizeTarget {
        path: PathBuf,
        reason: String,
    },
    /// `project_root` could not be canonicalized; refusing to compare a
    /// canonical \`target\` against a non-canonical reference would
    /// silently defeat the project-root/cwd safety guards (GH #3110).
    /// `reason` carries the underlying io::Error (GH #3538).
    CannotCanonicalizeProjectRoot {
        path: PathBuf,
        reason: String,
    },
    /// `cwd` could not be canonicalized; same reason as above.
    /// `reason` carries the underlying io::Error (GH #3538).
    CannotCanonicalizeCwd {
        path: PathBuf,
        reason: String,
    },
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
impl std::fmt::Display for CleanRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CleanRejection::EmptyPath => write!(f, "refusing to clean empty path"),
            CleanRejection::FilesystemRoot => write!(f, "refusing to clean filesystem root"),
            CleanRejection::ProjectRoot => write!(f, "refusing to clean the project root"),
            CleanRejection::CurrentWorkingDirectory => {
                write!(f, "refusing to clean the current working directory")
            }
            CleanRejection::OutsideProjectRootWithoutForce { resolved } => write!(
                f,
                "refusing to clean {} because it is outside the project root; \
                 re-run with --force to opt in",
                resolved.display()
            ),
            CleanRejection::CannotCanonicalizeTarget { path, reason } => write!(
                f,
                "refusing to clean {}: canonicalize failed ({}). This is usually \
                 a symlink loop, missing parent, EACCES on a parent directory, or \
                 an EIO on a flaky disk — not necessarily symlink traversal. \
                 (GH #3538)",
                path.display(),
                reason
            ),
            CleanRejection::CannotCanonicalizeProjectRoot { path, reason } => write!(
                f,
                "refusing to clean: cannot canonicalize project root {} ({}); \
                 safety guards require a canonical reference path (GH #3110)",
                path.display(),
                reason
            ),
            CleanRejection::CannotCanonicalizeCwd { path, reason } => write!(
                f,
                "refusing to clean: cannot canonicalize cwd {} ({}); \
                 safety guards require a canonical reference path (GH #3110)",
                path.display(),
                reason
            ),
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
impl std::error::Error for CleanRejection {}

/// Decide whether `target` may be emptied given `project_root` and `force`.
///
/// `project_root` and `target` are expected to be absolute paths from the
/// caller. `cwd` is the current working directory (passed in for testability).
///
/// Returns:
/// - `Ok(Some(canonical))` — safe to empty, with the canonicalized path.
/// - `Ok(None)` — target does not exist; nothing to clean (caller continues).
/// - `Err(CleanRejection)` — refused.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub fn assess_clean(
    target: &Path,
    project_root: &Path,
    cwd: &Path,
    force: bool,
) -> Result<Option<PathBuf>, CleanRejection> {
    if target.as_os_str().is_empty() {
        return Err(CleanRejection::EmptyPath);
    }

    if !target.exists() {
        return Ok(None);
    }

    // GH #3538 — preserve the underlying io::Error message on each
    // canonicalize failure. Previously `.map_err(|_| ...)` dropped the
    // OS error, so EACCES/ELOOP/EIO/EMFILE all surfaced as a fixed
    // "symlink traversal" message that misdirected operators.
    let canonical_target =
        target
            .canonicalize()
            .map_err(|err| CleanRejection::CannotCanonicalizeTarget {
                path: target.to_path_buf(),
                reason: err.to_string(),
            })?;
    // GH #3110 — both sides of the safety comparisons must be in
    // canonical form. Falling back to the raw symlink-form path made
    // the `ProjectRoot` / `CurrentWorkingDirectory` / `starts_with`
    // guards silently false-negative when the user's project lives
    // under a symlink and canonicalize failed for `project_root`.
    let canonical_root = project_root.canonicalize().map_err(|err| {
        CleanRejection::CannotCanonicalizeProjectRoot {
            path: project_root.to_path_buf(),
            reason: err.to_string(),
        }
    })?;
    let canonical_cwd =
        cwd.canonicalize()
            .map_err(|err| CleanRejection::CannotCanonicalizeCwd {
                path: cwd.to_path_buf(),
                reason: err.to_string(),
            })?;

    if canonical_target.parent().is_none() {
        return Err(CleanRejection::FilesystemRoot);
    }
    if canonical_target == canonical_root {
        return Err(CleanRejection::ProjectRoot);
    }
    if canonical_target == canonical_cwd {
        return Err(CleanRejection::CurrentWorkingDirectory);
    }

    if !canonical_target.starts_with(&canonical_root) && !force {
        return Err(CleanRejection::OutsideProjectRootWithoutForce {
            resolved: canonical_target,
        });
    }

    Ok(Some(canonical_target))
}

/// Empty `target` (delete its contents but leave the directory itself).
///
/// Reads cwd from the environment. Tests should call
/// [`empty_out_dir_with_cwd`] to avoid mutating process-global state.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub fn empty_out_dir(target: &Path, project_root: &Path, force: bool) -> Result<()> {
    let cwd = std::env::current_dir().context("reading current working directory")?;
    empty_out_dir_with_cwd(target, project_root, &cwd, force)
}

/// Variant of [`empty_out_dir`] that takes an explicit cwd. Calls
/// [`assess_clean`] first; on rejection returns the rejection as
/// `anyhow::Error`. On `Ok(None)` (target missing) does nothing and returns.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub fn empty_out_dir_with_cwd(
    target: &Path,
    project_root: &Path,
    cwd: &Path,
    force: bool,
) -> Result<()> {
    let canonical = match assess_clean(target, project_root, cwd, force) {
        Ok(Some(path)) => path,
        Ok(None) => return Ok(()),
        Err(rej) => return Err(anyhow!(rej)),
    };

    let read_dir = std::fs::read_dir(&canonical)
        .with_context(|| format!("reading {} for clean", canonical.display()))?;
    for entry in read_dir {
        let entry = entry.with_context(|| format!("iterating {}", canonical.display()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .with_context(|| format!("statting {}", path.display()))?;
        if file_type.is_dir() && !file_type.is_symlink() {
            std::fs::remove_dir_all(&path)
                .with_context(|| format!("removing {}", path.display()))?;
        } else {
            std::fs::remove_file(&path).with_context(|| format!("removing {}", path.display()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn rejects_empty_path() {
        let project = TempDir::new().unwrap();
        let cwd = TempDir::new().unwrap();
        let result = assess_clean(Path::new(""), project.path(), cwd.path(), false);
        assert!(matches!(result, Err(CleanRejection::EmptyPath)));
    }

    #[test]
    fn rejects_filesystem_root() {
        let project = TempDir::new().unwrap();
        let cwd = TempDir::new().unwrap();
        let result = assess_clean(Path::new("/"), project.path(), cwd.path(), true);
        assert!(matches!(
            result,
            Err(CleanRejection::FilesystemRoot
                | CleanRejection::OutsideProjectRootWithoutForce { .. })
        ));
        // With force the rejection is FilesystemRoot — without force it's
        // OutsideProjectRootWithoutForce. Both block deletion; pick the more
        // specific filesystem-root rule when force=true.
    }

    #[test]
    fn rejects_project_root() {
        let project = TempDir::new().unwrap();
        let cwd = TempDir::new().unwrap();
        let result = assess_clean(project.path(), project.path(), cwd.path(), true);
        assert!(matches!(result, Err(CleanRejection::ProjectRoot)));
    }

    #[test]
    fn rejects_current_working_directory() {
        let project = TempDir::new().unwrap();
        let cwd = TempDir::new().unwrap();
        // cwd lives outside the project, but we mark force=true so the
        // outside-root rule does not preempt the cwd rule.
        let result = assess_clean(cwd.path(), project.path(), cwd.path(), true);
        assert!(matches!(
            result,
            Err(CleanRejection::CurrentWorkingDirectory)
        ));
    }

    #[test]
    fn rejects_outside_project_root_without_force() {
        let project = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        let cwd = TempDir::new().unwrap();
        fs::create_dir_all(outside.path().join("dist")).unwrap();
        let target = outside.path().join("dist");
        let result = assess_clean(&target, project.path(), cwd.path(), false);
        assert!(matches!(
            result,
            Err(CleanRejection::OutsideProjectRootWithoutForce { .. })
        ));
    }

    #[test]
    fn accepts_outside_project_root_with_force() {
        let project = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        let cwd = TempDir::new().unwrap();
        fs::create_dir_all(outside.path().join("dist")).unwrap();
        let target = outside.path().join("dist");
        let result = assess_clean(&target, project.path(), cwd.path(), true);
        assert!(matches!(result, Ok(Some(_))));
    }

    #[test]
    fn accepts_inside_project_root() {
        let project = TempDir::new().unwrap();
        let cwd = TempDir::new().unwrap();
        let target = project.path().join("dist");
        fs::create_dir_all(&target).unwrap();
        let result = assess_clean(&target, project.path(), cwd.path(), false);
        assert!(matches!(result, Ok(Some(_))));
    }

    #[test]
    fn missing_target_returns_none() {
        let project = TempDir::new().unwrap();
        let cwd = TempDir::new().unwrap();
        let target = project.path().join("never-built");
        let result = assess_clean(&target, project.path(), cwd.path(), false);
        assert!(matches!(result, Ok(None)));
    }

    /// GH #3110 — when `project_root` cannot be canonicalized (e.g. it
    /// no longer exists), the function must refuse to clean rather
    /// than silently fall back to the raw path and run safety
    /// comparisons against a non-canonical reference.
    #[test]
    fn refuses_clean_when_project_root_cannot_be_canonicalized() {
        let project_holder = TempDir::new().unwrap();
        let bogus_root = project_holder.path().join("does-not-exist");
        // `target` does need to exist for canonicalize to succeed on it
        // — the bug we are guarding only fires after `canonical_target`
        // is computed. Put the target inside the holder so it exists.
        let target = project_holder.path().join("dist");
        fs::create_dir_all(&target).unwrap();
        let cwd = TempDir::new().unwrap();

        let result = assess_clean(&target, &bogus_root, cwd.path(), true);
        assert!(
            matches!(
                result,
                Err(CleanRejection::CannotCanonicalizeProjectRoot { .. })
            ),
            "expected CannotCanonicalizeProjectRoot, got {result:?}"
        );
    }

    /// GH #3110 — same as above but for `cwd`.
    #[test]
    fn refuses_clean_when_cwd_cannot_be_canonicalized() {
        let project = TempDir::new().unwrap();
        let target = project.path().join("dist");
        fs::create_dir_all(&target).unwrap();
        let cwd_holder = TempDir::new().unwrap();
        let bogus_cwd = cwd_holder.path().join("does-not-exist");

        let result = assess_clean(&target, project.path(), &bogus_cwd, true);
        assert!(
            matches!(result, Err(CleanRejection::CannotCanonicalizeCwd { .. })),
            "expected CannotCanonicalizeCwd, got {result:?}"
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // GH #3538 — canonicalize io::Error context is preserved on rejection
    // ──────────────────────────────────────────────────────────────────

    /// When `project_root` canonicalize fails, the rejection must carry
    /// the underlying io::Error string in its `reason` field — not just a
    /// bare path. A user grepping logs for "Permission denied" or
    /// "No such file" must be able to land on this rejection.
    #[test]
    fn gh3538_project_root_canonicalize_failure_preserves_io_error_reason() {
        let project_holder = TempDir::new().unwrap();
        let bogus_root = project_holder.path().join("does-not-exist");
        let target = project_holder.path().join("dist");
        fs::create_dir_all(&target).unwrap();
        let cwd = TempDir::new().unwrap();

        let result = assess_clean(&target, &bogus_root, cwd.path(), true);
        match result {
            Err(CleanRejection::CannotCanonicalizeProjectRoot { reason, path }) => {
                assert_eq!(path, bogus_root, "rejection must name the offending path");
                assert!(
                    !reason.is_empty(),
                    "rejection must carry the underlying io::Error reason, got empty string"
                );
                // ENOENT-shape — the OS error string varies across platforms but
                // always mentions the file or its absence in some form.
                assert!(
                    reason.to_lowercase().contains("no such")
                        || reason.to_lowercase().contains("not found")
                        || reason.contains("os error 2"),
                    "expected ENOENT-shaped reason, got {reason:?}"
                );
            }
            other => panic!("expected CannotCanonicalizeProjectRoot, got {other:?}"),
        }
    }

    /// Same but for `cwd`.
    #[test]
    fn gh3538_cwd_canonicalize_failure_preserves_io_error_reason() {
        let project = TempDir::new().unwrap();
        let target = project.path().join("dist");
        fs::create_dir_all(&target).unwrap();
        let cwd_holder = TempDir::new().unwrap();
        let bogus_cwd = cwd_holder.path().join("does-not-exist");

        let result = assess_clean(&target, project.path(), &bogus_cwd, true);
        match result {
            Err(CleanRejection::CannotCanonicalizeCwd { reason, path }) => {
                assert_eq!(path, bogus_cwd);
                assert!(
                    !reason.is_empty(),
                    "cwd rejection must carry io::Error reason"
                );
            }
            other => panic!("expected CannotCanonicalizeCwd, got {other:?}"),
        }
    }

    /// When `target` canonicalize fails, the rejection variant must be
    /// `CannotCanonicalizeTarget` (not the previously misleading
    /// `SymlinkTraversal`), and must include the OS reason.
    ///
    /// We trigger the failure via chmod 000 on the target's parent —
    /// `canonicalize` then fails with EACCES instead of resolving the
    /// last component. (`target.exists()` returns true via the cached
    /// readable test path before we drop perms.)
    #[cfg(unix)]
    #[test]
    fn gh3538_target_canonicalize_failure_returns_cannot_canonicalize_target() {
        use std::os::unix::fs::PermissionsExt;

        let project = TempDir::new().unwrap();
        let parent = project.path().join("locked-parent");
        fs::create_dir_all(&parent).unwrap();
        let target = parent.join("dist");
        fs::create_dir_all(&target).unwrap();
        let cwd = TempDir::new().unwrap();

        // Drop perms on the parent so canonicalize of target fails EACCES.
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o000)).unwrap();

        // Skip cleanly when running as root (chmod has no effect).
        if target.exists() && fs::read_dir(&parent).is_ok() {
            fs::set_permissions(&parent, fs::Permissions::from_mode(0o755)).unwrap();
            return;
        }

        let result = assess_clean(&target, project.path(), cwd.path(), true);

        // Restore perms so tempdir cleanup can succeed.
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o755)).unwrap();

        match result {
            Err(CleanRejection::CannotCanonicalizeTarget { path, reason }) => {
                assert_eq!(path, target, "rejection must name the offending target");
                assert!(
                    !reason.is_empty(),
                    "target rejection must carry io::Error reason"
                );
            }
            // Some platforms surface target.exists()==false earlier and we
            // reach `Ok(None)` — pin that as an acceptable alternate outcome
            // since the bug we're guarding is specifically the
            // *misclassified-as-SymlinkTraversal* path.
            Ok(None) => { /* parent unreadable made exists() false; acceptable */ }
            other => panic!("expected CannotCanonicalizeTarget or Ok(None), got {other:?}"),
        }
    }

    /// Display message must include the OS reason, not just the path.
    /// This is the user-visible breadcrumb the PR is shipping.
    #[test]
    fn gh3538_display_includes_io_error_reason() {
        let rej = CleanRejection::CannotCanonicalizeTarget {
            path: PathBuf::from("/some/path"),
            reason: "Permission denied (os error 13)".to_string(),
        };
        let msg = format!("{rej}");
        assert!(
            msg.contains("/some/path"),
            "Display must include the path: {msg}"
        );
        assert!(
            msg.contains("Permission denied"),
            "Display must include the underlying io::Error reason: {msg}"
        );
        assert!(
            msg.contains("GH #3538"),
            "Display must carry the GH #3538 tag for log grep: {msg}"
        );
    }

    #[test]
    fn empty_out_dir_removes_stale_hashed_bundles() {
        let project = TempDir::new().unwrap();
        let dist = project.path().join("dist");
        fs::create_dir_all(&dist).unwrap();
        fs::write(dist.join("main.aaaaaaaa.js"), "stale").unwrap();
        fs::write(dist.join("main.aaaaaaaa.js.map"), "{}").unwrap();
        fs::write(dist.join("index.html"), "<html/>").unwrap();
        fs::create_dir_all(dist.join("assets")).unwrap();
        fs::write(dist.join("assets/logo.png"), "PNGDATA").unwrap();

        let cwd = TempDir::new().unwrap();
        empty_out_dir_with_cwd(&dist, project.path(), cwd.path(), false).unwrap();

        assert!(dist.exists(), "output dir itself preserved");
        assert!(!dist.join("main.aaaaaaaa.js").exists());
        assert!(!dist.join("main.aaaaaaaa.js.map").exists());
        assert!(!dist.join("index.html").exists());
        assert!(!dist.join("assets").exists());
    }
}
// CODEGEN-END
