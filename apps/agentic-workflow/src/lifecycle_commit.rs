//! Unforgeable capability for mutating Git operations in `agentic-workflow`.
//!
//! All staging, commit, reset, and merge calls in production code must route
//! through this module, which holds exclusive ability to construct
//! `LifecycleCommitCapability`.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Artifact leaf enum declared per routed call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleLeaf {
    Wi,
    Ec,
    Td,
    Cb,
    Unrouted,
}

impl LifecycleLeaf {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Wi => "wi",
            Self::Ec => "ec",
            Self::Td => "td",
            Self::Cb => "cb",
            Self::Unrouted => "unrouted",
        }
    }
}

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
    leaf: LifecycleLeaf,
    project_root: &Path,
    paths: &[PathBuf],
    message: &str,
) -> Result<bool> {
    let cap = LifecycleCommitCapability::get();
    crate::git::commit_scoped_paths(&cap, project_root, paths, message)
        .with_context(|| format!("lifecycle leaf {}", leaf.as_str()))
}

/// Stage specified `paths` into git index.
pub fn stage_path_set<P: AsRef<Path>>(
    leaf: LifecycleLeaf,
    project_root: &Path,
    paths: &[P],
    literal_pathspecs: bool,
) -> Result<()> {
    let cap = LifecycleCommitCapability::get();
    crate::git::stage_paths(&cap, project_root, paths, literal_pathspecs)
        .with_context(|| format!("lifecycle leaf {}", leaf.as_str()))
}

/// Commit already staged changes with `message`. If `allow_empty` is true, pass `--allow-empty`.
pub fn commit_staged_changes(
    leaf: LifecycleLeaf,
    project_root: &Path,
    message: &str,
    allow_empty: bool,
) -> Result<()> {
    let cap = LifecycleCommitCapability::get();
    crate::git::commit_staged(&cap, project_root, message, allow_empty)
        .with_context(|| format!("lifecycle leaf {}", leaf.as_str()))
}

/// Create a path-scoped commit using `git commit --only -- <paths>`.
pub fn commit_only_path_set<P: AsRef<Path>>(
    leaf: LifecycleLeaf,
    project_root: &Path,
    paths: &[P],
    message: &str,
    literal_pathspecs: bool,
) -> Result<()> {
    let cap = LifecycleCommitCapability::get();
    crate::git::commit_only_paths(&cap, project_root, paths, message, literal_pathspecs)
        .with_context(|| format!("lifecycle leaf {}", leaf.as_str()))
}

/// Roll back the previous commit (`HEAD~1`).
pub fn rollback_last_commit(leaf: LifecycleLeaf, project_root: &Path) -> Result<()> {
    let cap = LifecycleCommitCapability::get();
    crate::git::git_reset(&cap, project_root, &["HEAD~1"])
        .with_context(|| format!("lifecycle leaf {}", leaf.as_str()))
}

/// Merge a branch using `--no-ff` with `message`.
pub fn merge_branch_no_ff(
    leaf: LifecycleLeaf,
    project_root: &Path,
    branch: &str,
    message: &str,
) -> Result<()> {
    let cap = LifecycleCommitCapability::get();
    crate::git::git_merge(&cap, project_root, &["--no-ff", "-m", message, branch])
        .with_context(|| format!("lifecycle leaf {}", leaf.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_lifecycle_leaf_as_str() {
        assert_eq!(LifecycleLeaf::Wi.as_str(), "wi");
        assert_eq!(LifecycleLeaf::Ec.as_str(), "ec");
        assert_eq!(LifecycleLeaf::Td.as_str(), "td");
        assert_eq!(LifecycleLeaf::Cb.as_str(), "cb");
        assert_eq!(LifecycleLeaf::Unrouted.as_str(), "unrouted");

        let strings = [
            LifecycleLeaf::Wi.as_str(),
            LifecycleLeaf::Ec.as_str(),
            LifecycleLeaf::Td.as_str(),
            LifecycleLeaf::Cb.as_str(),
            LifecycleLeaf::Unrouted.as_str(),
        ];
        let set: std::collections::HashSet<_> = strings.iter().copied().collect();
        assert_eq!(
            set.len(),
            5,
            "LifecycleLeaf as_str values must be pairwise distinct"
        );
    }

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
            ("cli/cb.rs", LifecycleLeaf::Cb, 8),
            ("cli/cb_fill.rs", LifecycleLeaf::Cb, 6),
            ("cli/td.rs", LifecycleLeaf::Td, 3),
            ("cli/td_lock.rs", LifecycleLeaf::Td, 2),
            ("cli/run.rs", LifecycleLeaf::Unrouted, 1),
            ("cli/td_migrate.rs", LifecycleLeaf::Td, 1),
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
        let mut unrouted_sites = Vec::new();

        fn walk_dir(
            dir: &Path,
            src_dir: &Path,
            allowlist: &[(&str, LifecycleLeaf, usize)],
            operations: &[&str],
            violations: &mut Vec<String>,
            total_found: &mut usize,
            visited_allowlist_files: &mut std::collections::HashSet<String>,
            unrouted_sites: &mut Vec<String>,
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
                        unrouted_sites,
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

                    let expected_entry = allowlist.iter().find(|(m, _, _)| *m == rel_path);

                    if expected_entry.is_some() {
                        visited_allowlist_files.insert(rel_path.clone());
                    }

                    let lines: Vec<&str> = content.lines().collect();
                    let mut file_count = 0;

                    for (line_idx, line) in lines.iter().enumerate() {
                        let code_line = if let Some((code, _)) = line.split_once("//") {
                            code
                        } else {
                            line
                        };
                        if code_line.contains("crate::lifecycle_commit::") {
                            for op in operations {
                                if code_line.contains(&format!("crate::lifecycle_commit::{op}")) {
                                    file_count += 1;

                                    let mut span = String::new();
                                    let mut found_terminator = false;

                                    let op_match = format!("crate::lifecycle_commit::{op}");
                                    let call_start_pos = code_line.find(&op_match).unwrap();
                                    let after_op = &code_line[call_start_pos + op_match.len()..];

                                    if after_op.contains(')') {
                                        span.push_str(code_line);
                                        found_terminator = true;
                                    } else {
                                        span.push_str(code_line);
                                        span.push('\n');
                                        for k in (line_idx + 1)..lines.len().min(line_idx + 20) {
                                            let raw_l = lines[k];
                                            let code_l =
                                                if let Some((code, _)) = raw_l.split_once("//") {
                                                    code
                                                } else {
                                                    raw_l
                                                };
                                            span.push_str(code_l);
                                            span.push('\n');
                                            if code_l.contains(')') {
                                                found_terminator = true;
                                                break;
                                            }
                                        }
                                    }

                                    if !found_terminator {
                                        violations.push(format!(
                                            "{}:{}: Call to crate::lifecycle_commit::{} span missing closing terminator ');'",
                                            rel_path,
                                            line_idx + 1,
                                            op
                                        ));
                                        continue;
                                    }

                                    let leaf_variants = [
                                        (LifecycleLeaf::Wi, "LifecycleLeaf::Wi"),
                                        (LifecycleLeaf::Ec, "LifecycleLeaf::Ec"),
                                        (LifecycleLeaf::Td, "LifecycleLeaf::Td"),
                                        (LifecycleLeaf::Cb, "LifecycleLeaf::Cb"),
                                        (LifecycleLeaf::Unrouted, "LifecycleLeaf::Unrouted"),
                                    ];

                                    let mut found_leaves = Vec::new();
                                    for (variant, pat) in leaf_variants {
                                        let count = span.matches(pat).count();
                                        for _ in 0..count {
                                            found_leaves.push(variant);
                                        }
                                    }

                                    let found_leaf = match found_leaves.len() {
                                        0 => {
                                            violations.push(format!(
                                                "{}:{}: Call to crate::lifecycle_commit::{} carries no literal LifecycleLeaf variant inside its call span",
                                                rel_path,
                                                line_idx + 1,
                                                op
                                            ));
                                            None
                                        }
                                        1 => Some(found_leaves[0]),
                                        _ => {
                                            violations.push(format!(
                                                "{}:{}: Call to crate::lifecycle_commit::{} carries multiple literal LifecycleLeaf variants inside its call span",
                                                rel_path,
                                                line_idx + 1,
                                                op
                                            ));
                                            None
                                        }
                                    };

                                    if let Some(found_leaf) = found_leaf {
                                        if found_leaf == LifecycleLeaf::Unrouted {
                                            unrouted_sites.push(format!(
                                                "{}:{}",
                                                rel_path,
                                                line_idx + 1
                                            ));
                                        }

                                        if let Some((_, expected_leaf, _)) = expected_entry {
                                            if found_leaf != *expected_leaf {
                                                violations.push(format!(
                                                    "{}:{}: Declared leaf LifecycleLeaf::{:?} does not match expected LifecycleLeaf::{:?} for module {}",
                                                    rel_path,
                                                    line_idx + 1,
                                                    found_leaf,
                                                    expected_leaf,
                                                    rel_path
                                                ));
                                            }
                                        } else {
                                            violations.push(format!(
                                                "{}:{}: Call to crate::lifecycle_commit::{} outside allowlist: {}",
                                                rel_path,
                                                line_idx + 1,
                                                op,
                                                line.trim()
                                            ));
                                        }
                                    } else if expected_entry.is_none() {
                                        violations.push(format!(
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

                    if let Some((_, _, exp)) = expected_entry {
                        if file_count != *exp {
                            violations.push(format!(
                                "Module {} expected {} routed call sites, found {}",
                                rel_path, exp, file_count
                            ));
                        }
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
            &mut unrouted_sites,
        );

        for (rel_path, _, expected_count) in allowlist {
            if !visited_allowlist_files.contains(rel_path) {
                violations.push(format!(
                    "Module {} expected {} routed call sites, found 0",
                    rel_path, expected_count
                ));
            }
        }

        if unrouted_sites.len() != 1 {
            let unrouted_list = if unrouted_sites.is_empty() {
                "none".to_string()
            } else {
                unrouted_sites.join(", ")
            };
            violations.push(format!(
                "Expected exactly 1 LifecycleLeaf::Unrouted call site in crate source tree (count may only decrease), found {}: [{}]",
                unrouted_sites.len(),
                unrouted_list
            ));
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
