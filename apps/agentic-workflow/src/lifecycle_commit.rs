//! Unforgeable capability for mutating Git operations in `agentic-workflow`.
//!
//! All staging, commit, reset, and merge calls in production code must route
//! through this module, which holds exclusive ability to construct
//! `LifecycleCommitCapability`.

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Unforgeable capability token required by `crate::git` mutating primitives.
///
/// Code outside `crate::lifecycle_commit` cannot construct this type because
/// its single field is private and no public constructor or `Default` / `Clone`
/// implementation exists.
pub struct LifecycleCommitCapability {
    _private: (),
}

impl LifecycleCommitCapability {
    /// Internal constructor accessible only within `crate::lifecycle_commit`.
    fn get() -> Self {
        Self { _private: () }
    }

    /// Test-only constructor available when compiled under `#[cfg(test)]`.
    #[cfg(test)]
    pub(crate) fn for_test() -> Self {
        Self { _private: () }
    }
}

/// Stage exactly `paths`, create `message` as a lifecycle commit, and no-op
/// when those paths have no staged diff.
pub fn commit_scoped_path_set(
    project_root: &Path,
    paths: &[PathBuf],
    message: &str,
) -> Result<bool> {
    let cap = LifecycleCommitCapability::get();
    crate::git::commit_scoped_paths(&cap, project_root, paths, message)
}

/// Stage specified `paths` into git index.
pub fn stage_path_set<P: AsRef<Path>>(
    project_root: &Path,
    paths: &[P],
    literal_pathspecs: bool,
) -> Result<()> {
    let cap = LifecycleCommitCapability::get();
    crate::git::stage_paths(&cap, project_root, paths, literal_pathspecs)
}

/// Commit already staged changes with `message`. If `allow_empty` is true, pass `--allow-empty`.
pub fn commit_staged_changes(project_root: &Path, message: &str, allow_empty: bool) -> Result<()> {
    let cap = LifecycleCommitCapability::get();
    crate::git::commit_staged(&cap, project_root, message, allow_empty)
}

/// Create a path-scoped commit using `git commit --only -- <paths>`.
pub fn commit_only_path_set<P: AsRef<Path>>(
    project_root: &Path,
    paths: &[P],
    message: &str,
    literal_pathspecs: bool,
) -> Result<()> {
    let cap = LifecycleCommitCapability::get();
    crate::git::commit_only_paths(&cap, project_root, paths, message, literal_pathspecs)
}

/// Roll back the previous commit (`HEAD~1`).
pub fn rollback_last_commit(project_root: &Path) -> Result<()> {
    let cap = LifecycleCommitCapability::get();
    crate::git::git_reset(&cap, project_root, &["HEAD~1"])
}

/// Merge a branch using `--no-ff` with `message`.
pub fn merge_branch_no_ff(project_root: &Path, branch: &str, message: &str) -> Result<()> {
    let cap = LifecycleCommitCapability::get();
    crate::git::git_merge(&cap, project_root, &["--no-ff", "-m", message, branch])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_capability_required_for_git_primitives() {
        let cap = LifecycleCommitCapability::for_test();
        let temp = tempfile::TempDir::new().unwrap();
        let repo = temp.path();

        let git_bin = crate::git::find_git_bin();
        if git_bin.is_none() {
            return;
        }
        let init = std::process::Command::new(git_bin.unwrap())
            .args(["init", "-q"])
            .current_dir(repo)
            .output()
            .unwrap();
        if !init.status.success() {
            return;
        }

        assert!(crate::git::stage_paths::<&str>(&cap, repo, &[], false).is_ok());
    }

    #[test]
    fn test_routed_call_site_counts() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| "apps/agentic-workflow".to_string());
        let src_dir = Path::new(&manifest_dir).join("src");

        let allowlist = [
            ("cli/cb.rs", 8),
            ("cli/cb_fill.rs", 6),
            ("cli/td.rs", 3),
            ("cli/td_lock.rs", 2),
            ("cli/run.rs", 1),
            ("cli/td_migrate.rs", 1),
        ];

        let operations = [
            "commit_scoped_path_set",
            "stage_path_set",
            "commit_staged_changes",
            "commit_only_path_set",
            "rollback_last_commit",
            "merge_branch_no_ff",
        ];

        let mut violations = Vec::new();
        let mut total_found = 0;
        let mut visited_allowlist_files = std::collections::HashSet::new();

        fn walk_dir(
            dir: &Path,
            src_dir: &Path,
            allowlist: &[(&str, usize)],
            operations: &[&str],
            violations: &mut Vec<String>,
            total_found: &mut usize,
            visited_allowlist_files: &mut std::collections::HashSet<String>,
        ) {
            let mut entries: Vec<_> = std::fs::read_dir(dir)
                .unwrap()
                .filter_map(|e| e.ok())
                .collect();
            entries.sort_by_key(|e| e.path());

            for entry in entries {
                let path = entry.path();
                if path.is_dir() {
                    walk_dir(
                        &path,
                        src_dir,
                        allowlist,
                        operations,
                        violations,
                        total_found,
                        visited_allowlist_files,
                    );
                } else if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                    if path == src_dir.join("lifecycle_commit.rs") {
                        continue;
                    }
                    let rel_path = path
                        .strip_prefix(src_dir)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .replace('\\', "/");

                    let content = std::fs::read_to_string(&path)
                        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

                    let expected_count = allowlist
                        .iter()
                        .find(|(m, _)| *m == rel_path)
                        .map(|(_, c)| *c);

                    if expected_count.is_some() {
                        visited_allowlist_files.insert(rel_path.clone());
                    }

                    let mut file_count = 0;
                    let mut line_violations = Vec::new();

                    for (line_idx, line) in content.lines().enumerate() {
                        let code_line = if let Some((code, _)) = line.split_once("//") {
                            code
                        } else {
                            line
                        };
                        if code_line.contains("crate::lifecycle_commit::") {
                            for op in operations {
                                if code_line.contains(&format!("crate::lifecycle_commit::{op}")) {
                                    file_count += 1;
                                    if expected_count.is_none() {
                                        line_violations.push(format!(
                                            "{}:{}: Call to crate::lifecycle_commit::{} outside allowlist: {}",
                                            rel_path,
                                            line_idx + 1,
                                            op,
                                            line.trim()
                                        ));
                                    }
                                }
                            }
                        }
                    }

                    *total_found += file_count;

                    if let Some(exp) = expected_count {
                        if file_count != exp {
                            violations.push(format!(
                                "Module {} expected {} routed call sites, found {}",
                                rel_path, exp, file_count
                            ));
                        }
                    } else if file_count > 0 {
                        violations.extend(line_violations);
                    }
                }
            }
        }

        walk_dir(
            &src_dir,
            &src_dir,
            &allowlist,
            &operations,
            &mut violations,
            &mut total_found,
            &mut visited_allowlist_files,
        );

        for (rel_path, expected_count) in allowlist {
            if !visited_allowlist_files.contains(rel_path) {
                violations.push(format!(
                    "Module {} expected {} routed call sites, found 0",
                    rel_path, expected_count
                ));
            }
        }

        assert!(
            violations.is_empty(),
            "Routed call census violations:\n{}",
            violations.join("\n")
        );

        assert_eq!(total_found, 21, "Expected exactly 21 total routed sites");
    }

    #[test]
    fn test_no_direct_git_mutating_primitives_outside_lifecycle_commit() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| "apps/agentic-workflow".to_string());
        let src_dir = Path::new(&manifest_dir).join("src");

        let primitives = [
            "commit_scoped_paths",
            "stage_paths",
            "commit_staged",
            "commit_only_paths",
            "git_reset",
            "git_merge",
        ];

        let mut violations = Vec::new();

        fn walk_dir(dir: &Path, primitives: &[&str], violations: &mut Vec<String>, src_dir: &Path) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    walk_dir(&path, primitives, violations, src_dir);
                } else if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                    if path == src_dir.join("lifecycle_commit.rs") {
                        continue;
                    }
                    let content = std::fs::read_to_string(&path).unwrap();
                    let rel_path = path.to_string_lossy();
                    for (line_idx, line) in content.lines().enumerate() {
                        let code_line = if let Some((code, _)) = line.split_once("//") {
                            code
                        } else {
                            line
                        };
                        for prim in primitives {
                            let match_fn = format!("crate::git::{prim}(");
                            let match_fn2 = format!("git::{prim}(");
                            let match_generic = format!("crate::git::{prim}<");
                            let match_generic2 = format!("git::{prim}<");
                            if code_line.contains(&match_fn)
                                || code_line.contains(&match_fn2)
                                || code_line.contains(&match_generic)
                                || code_line.contains(&match_generic2)
                            {
                                violations.push(format!(
                                    "{}:{}: Direct call to primitive {prim}: {}",
                                    rel_path,
                                    line_idx + 1,
                                    line.trim()
                                ));
                            }
                        }
                    }
                }
            }
        }

        walk_dir(&src_dir, &primitives, &mut violations, &src_dir);

        assert!(
            violations.is_empty(),
            "Found direct calls to git primitives outside lifecycle_commit.rs:\n{}",
            violations.join("\n")
        );
    }
}
