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
}

impl LifecycleLeaf {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Wi => "wi",
            Self::Ec => "ec",
            Self::Td => "td",
            Self::Cb => "cb",
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

/// Stage exactly `paths` for project persistence, create `message`, and no-op
/// when those paths have no staged diff.
pub fn commit_project_persistence(
    project_root: &Path,
    paths: &[PathBuf],
    message: &str,
) -> Result<bool> {
    let cap = LifecycleCommitCapability::get();
    crate::git::commit_scoped_paths(&cap, project_root, paths, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_lifecycle_leaf_as_str() {
        let variants = [
            LifecycleLeaf::Wi,
            LifecycleLeaf::Ec,
            LifecycleLeaf::Td,
            LifecycleLeaf::Cb,
        ];
        let strings: Vec<&'static str> = variants
            .iter()
            .map(|leaf| match leaf {
                LifecycleLeaf::Wi => leaf.as_str(),
                LifecycleLeaf::Ec => leaf.as_str(),
                LifecycleLeaf::Td => leaf.as_str(),
                LifecycleLeaf::Cb => leaf.as_str(),
            })
            .collect();
        assert_eq!(strings, ["wi", "ec", "td", "cb"]);

        let set: std::collections::HashSet<_> = strings.iter().copied().collect();
        assert_eq!(
            set.len(),
            4,
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
    fn test_commit_project_persistence_scoped_path() {
        let temp = tempfile::TempDir::new().unwrap();
        let repo = temp.path();

        let git_bin = crate::git::find_git_bin()
            .expect("git binary must be available for project persistence test");

        for args in [
            vec!["init", "-q", "-b", "main"],
            vec!["config", "user.email", "test@example.com"],
            vec!["config", "user.name", "Test"],
        ] {
            let out = std::process::Command::new(&git_bin)
                .args(&args)
                .current_dir(repo)
                .output()
                .expect("git setup command failed");
            assert!(out.status.success(), "git {:?} failed", args);
        }

        let file1 = repo.join("file1.txt");
        let file2 = repo.join("file2.txt");

        std::fs::write(&file1, "initial 1\n").unwrap();
        std::fs::write(&file2, "initial 2\n").unwrap();

        for args in [
            vec!["add", "."],
            vec!["commit", "-m", "initial commit", "-q"],
        ] {
            let out = std::process::Command::new(&git_bin)
                .args(&args)
                .current_dir(repo)
                .output()
                .expect("git initial commit command failed");
            assert!(out.status.success(), "git {:?} failed", args);
        }

        // Make both files dirty
        std::fs::write(&file1, "modified 1\n").unwrap();
        std::fs::write(&file2, "modified 2\n").unwrap();

        let msg = "test project persistence commit message";
        let committed = commit_project_persistence(repo, &[file1.clone()], msg)
            .expect("commit_project_persistence should succeed");

        assert!(
            committed,
            "commit_project_persistence should return true for modified path"
        );

        // Verify exactly 1 new commit
        let rev_list = crate::git::git_rev_list(repo, &["HEAD"]).unwrap();
        assert_eq!(
            rev_list.len(),
            2,
            "Expected exactly 2 commits (initial + 1 new)"
        );

        // Verify HEAD commit message is msg
        let head_msg = crate::git::git_log(repo, &["-1", "--format=%B"]).unwrap();
        assert_eq!(
            head_msg.trim(),
            msg,
            "HEAD commit message should match passed message"
        );

        // Verify tree touches only the declared path (file1.txt)
        let diff_paths = crate::git::git_diff_name_only(repo, &["HEAD~1", "HEAD"]).unwrap();
        assert_eq!(
            diff_paths,
            vec!["file1.txt"],
            "Commit tree should touch only file1.txt"
        );

        // Verify second file is still dirty and uncommitted
        let dirty = crate::git::dirty_paths(repo, &[std::path::PathBuf::from(".")], false).unwrap();
        assert_eq!(
            dirty,
            vec!["file2.txt"],
            "file2.txt should remain dirty and uncommitted"
        );
    }

    #[test]
    fn test_routed_call_site_counts() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| "apps/agentic-workflow".to_string());
        let src_dir = Path::new(&manifest_dir).join("src");

        let leaf_allowlist = [
            ("cli/cb.rs", LifecycleLeaf::Cb, 8),
            ("cli/cb_fill.rs", LifecycleLeaf::Cb, 6),
            ("cli/td.rs", LifecycleLeaf::Td, 3),
            ("cli/td_lock.rs", LifecycleLeaf::Td, 2),
            ("cli/td_migrate.rs", LifecycleLeaf::Td, 1),
        ];

        let leaf_operations = [
            "commit_scoped_path_set",
            "stage_path_set",
            "commit_staged_changes",
            "commit_only_path_set",
            "rollback_last_commit",
            "merge_branch_no_ff",
        ];

        let non_lifecycle_route = "commit_project_persistence";

        let mut violations = Vec::new();
        let mut leaf_total_found = 0;
        let mut visited_allowlist_files = std::collections::HashSet::new();
        let mut unrouted_token_count = 0;
        let mut cb_token_count = 0;
        let mut non_lifecycle_calls = Vec::new();

        fn walk_dir(
            dir: &Path,
            src_dir: &Path,
            leaf_allowlist: &[(&str, LifecycleLeaf, usize)],
            leaf_operations: &[&str],
            non_lifecycle_route: &str,
            violations: &mut Vec<String>,
            leaf_total_found: &mut usize,
            visited_allowlist_files: &mut std::collections::HashSet<String>,
            unrouted_token_count: &mut usize,
            cb_token_count: &mut usize,
            non_lifecycle_calls: &mut Vec<String>,
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
                        leaf_allowlist,
                        leaf_operations,
                        non_lifecycle_route,
                        violations,
                        leaf_total_found,
                        visited_allowlist_files,
                        unrouted_token_count,
                        cb_token_count,
                        non_lifecycle_calls,
                    );
                } else if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                    let content = std::fs::read_to_string(&path)
                        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

                    // The census matches the qualified path only (LifecycleLeaf followed by :: and Unrouted);
                    // bare word occurrences of Unrouted in prose are out of scope on purpose,
                    // and `concat!` is what stops the assertion from counting itself.
                    let target_unrouted1 = concat!("LifecycleLeaf", "::", "Unrouted");
                    let target_unrouted2 = concat!("LifecycleLeaf", " :: ", "Unrouted");
                    let target_cb1 = concat!("LifecycleLeaf", "::", "Cb");
                    let target_cb2 = concat!("LifecycleLeaf", " :: ", "Cb");

                    let unrouted_count = content.matches(target_unrouted1).count()
                        + content.matches(target_unrouted2).count();
                    let cb_count =
                        content.matches(target_cb1).count() + content.matches(target_cb2).count();
                    *unrouted_token_count += unrouted_count;
                    *cb_token_count += cb_count;

                    if path == src_dir.join("lifecycle_commit.rs") {
                        let fn_decl = format!("fn {non_lifecycle_route}");
                        if let Some(pos) = content.find(&fn_decl) {
                            let after_fn = &content[pos..];
                            if let Some(open_paren) = after_fn.find('(') {
                                let after_open = &after_fn[open_paren + 1..];
                                if let Some(close_paren) = after_open.find(')') {
                                    let param_list = &after_open[..close_paren];
                                    if param_list.contains("LifecycleLeaf") {
                                        violations.push(format!(
                                            "Non-lifecycle route {} declaration contains LifecycleLeaf in parameter list: ({})",
                                            non_lifecycle_route,
                                            param_list.trim()
                                        ));
                                    }
                                }
                            }
                        } else {
                            violations.push(format!(
                                "Declaration of non-lifecycle route {} not found in lifecycle_commit.rs",
                                non_lifecycle_route
                            ));
                        }
                        continue;
                    }
                    let rel_path = path
                        .strip_prefix(src_dir)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .replace('\\', "/");

                    let expected_entry = leaf_allowlist.iter().find(|(m, _, _)| *m == rel_path);

                    if expected_entry.is_some() {
                        visited_allowlist_files.insert(rel_path.clone());
                    }

                    let lines: Vec<&str> = content.lines().collect();
                    let mut file_leaf_count = 0;

                    for (line_idx, line) in lines.iter().enumerate() {
                        let code_line = if let Some((code, _)) = line.split_once("//") {
                            code
                        } else {
                            line
                        };

                        let non_lifecycle_match =
                            format!("lifecycle_commit::{non_lifecycle_route}");
                        if code_line.contains(&non_lifecycle_match) {
                            let call_ref =
                                format!("{}:{}: {}", rel_path, line_idx + 1, line.trim());
                            non_lifecycle_calls.push(call_ref);
                            if rel_path != "cli/run.rs" {
                                violations.push(format!(
                                    "{}:{}: Non-lifecycle route {} called outside allowed file cli/run.rs: {}",
                                    rel_path,
                                    line_idx + 1,
                                    non_lifecycle_route,
                                    line.trim()
                                ));
                            }
                        }

                        if code_line.contains("crate::lifecycle_commit::") {
                            for op in leaf_operations {
                                if code_line.contains(&format!("crate::lifecycle_commit::{op}")) {
                                    file_leaf_count += 1;

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

                    *leaf_total_found += file_leaf_count;

                    if let Some((_, _, exp)) = expected_entry {
                        if file_leaf_count != *exp {
                            violations.push(format!(
                                "Module {} expected {} routed call sites, found {}",
                                rel_path, exp, file_leaf_count
                            ));
                        }
                    }
                }
            }
        }

        walk_dir(
            &src_dir,
            &src_dir,
            &leaf_allowlist,
            &leaf_operations,
            non_lifecycle_route,
            &mut violations,
            &mut leaf_total_found,
            &mut visited_allowlist_files,
            &mut unrouted_token_count,
            &mut cb_token_count,
            &mut non_lifecycle_calls,
        );

        for (rel_path, _, expected_count) in leaf_allowlist {
            if !visited_allowlist_files.contains(rel_path) {
                violations.push(format!(
                    "Module {} expected {} routed call sites, found 0",
                    rel_path, expected_count
                ));
            }
        }

        if non_lifecycle_calls.len() != 1 {
            let calls_str = if non_lifecycle_calls.is_empty() {
                "none".to_string()
            } else {
                non_lifecycle_calls.join(", ")
            };
            violations.push(format!(
                "Expected exactly 1 call site of {} in crate source tree, found {}: [{}]",
                non_lifecycle_route,
                non_lifecycle_calls.len(),
                calls_str
            ));
        }

        assert_eq!(
            unrouted_token_count,
            0,
            "{}",
            concat!(
                "Expected 0 occurrences of LifecycleLeaf",
                "::",
                "Unrouted token in src/"
            )
        );
        assert!(
            cb_token_count > 0,
            "{}",
            concat!(
                "Expected non-zero occurrences of LifecycleLeaf",
                "::",
                "Cb token in src/"
            )
        );

        assert!(
            violations.is_empty(),
            "Routed call census violations:\n{}",
            violations.join("\n")
        );

        assert_eq!(
            leaf_total_found, 20,
            "Expected exactly 20 total leaf-taking routed sites"
        );
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
