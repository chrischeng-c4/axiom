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

/// Unforgeable capability token required by `crate::git` history probes.
///
/// Code outside `crate::lifecycle_commit` cannot construct this type because
/// its single field is private and no public constructor or `Default` / `Clone`
/// implementation exists.
pub struct LifecycleHistoryCapability {
    _private: (),
}

impl LifecycleHistoryCapability {
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

/// Unforgeable capability token required by `crate::git` working-tree probes.
///
/// Code outside `crate::lifecycle_commit` cannot construct this type because
/// its single field is private and no public constructor or `Default` / `Clone`
/// implementation exists.
pub struct LifecycleWorktreeCapability {
    _private: (),
}

impl LifecycleWorktreeCapability {
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

/// Run `git log` with args in `project_root`.
pub fn git_log(leaf: LifecycleLeaf, project_root: &Path, args: &[&str]) -> Result<String> {
    let cap = LifecycleHistoryCapability::get();
    crate::git::git_log(&cap, project_root, args)
        .with_context(|| format!("lifecycle leaf {}", leaf.as_str()))
}

/// Run `git rev-parse` with args in `project_root`.
pub fn git_rev_parse(leaf: LifecycleLeaf, project_root: &Path, args: &[&str]) -> Result<String> {
    let cap = LifecycleHistoryCapability::get();
    crate::git::git_rev_parse(&cap, project_root, args)
        .with_context(|| format!("lifecycle leaf {}", leaf.as_str()))
}

/// Run `git rev-list` with args in `project_root`.
pub fn git_rev_list(
    leaf: LifecycleLeaf,
    project_root: &Path,
    args: &[&str],
) -> Result<Vec<String>> {
    let cap = LifecycleHistoryCapability::get();
    crate::git::git_rev_list(&cap, project_root, args)
        .with_context(|| format!("lifecycle leaf {}", leaf.as_str()))
}

/// Run `git show` with args in `project_root`.
pub fn git_show(leaf: LifecycleLeaf, project_root: &Path, args: &[&str]) -> Result<String> {
    let cap = LifecycleHistoryCapability::get();
    crate::git::git_show(&cap, project_root, args)
        .with_context(|| format!("lifecycle leaf {}", leaf.as_str()))
}

/// Run `git cat-file blob <object>` in `project_root`.
pub fn git_cat_file_blob(
    leaf: LifecycleLeaf,
    project_root: &Path,
    object: &str,
) -> Result<Vec<u8>> {
    let cap = LifecycleHistoryCapability::get();
    crate::git::git_cat_file_blob(&cap, project_root, object)
        .with_context(|| format!("lifecycle leaf {}", leaf.as_str()))
}

/// Run `git diff --name-only` with args in `project_root`.
pub fn git_diff_name_only(
    leaf: LifecycleLeaf,
    project_root: &Path,
    args: &[&str],
) -> Result<Vec<String>> {
    let cap = LifecycleHistoryCapability::get();
    crate::git::git_diff_name_only(&cap, project_root, args)
        .with_context(|| format!("lifecycle leaf {}", leaf.as_str()))
}

/// Ensure no staged changes exist in `project_root`.
pub fn ensure_no_staged_changes(leaf: LifecycleLeaf, project_root: &Path) -> Result<()> {
    let cap = LifecycleWorktreeCapability::get();
    crate::git::ensure_no_staged_changes(&cap, project_root)
        .with_context(|| format!("lifecycle leaf {}", leaf.as_str()))
}

/// Return dirty paths under `scopes` in `project_root`.
pub fn dirty_paths(
    leaf: LifecycleLeaf,
    project_root: &Path,
    scopes: &[PathBuf],
    include_untracked: bool,
) -> Result<Vec<String>> {
    let cap = LifecycleWorktreeCapability::get();
    crate::git::dirty_paths(&cap, project_root, scopes, include_untracked)
        .with_context(|| format!("lifecycle leaf {}", leaf.as_str()))
}

/// Check if `project_root` has any staged changes.
pub fn has_staged_changes(leaf: LifecycleLeaf, project_root: &Path) -> Result<bool> {
    let cap = LifecycleWorktreeCapability::get();
    crate::git::has_staged_changes(&cap, project_root)
        .with_context(|| format!("lifecycle leaf {}", leaf.as_str()))
}

/// Check if specified `paths` in `project_root` have staged changes.
pub fn has_staged_changes_for_paths<P: AsRef<Path>>(
    leaf: LifecycleLeaf,
    project_root: &Path,
    paths: &[P],
    literal_pathspecs: bool,
) -> Result<bool> {
    let cap = LifecycleWorktreeCapability::get();
    crate::git::has_staged_changes_for_paths(&cap, project_root, paths, literal_pathspecs)
        .with_context(|| format!("lifecycle leaf {}", leaf.as_str()))
}

/// Run `git status` with args in `project_root` and return raw stdout bytes.
pub fn git_status<P: AsRef<Path>>(
    leaf: LifecycleLeaf,
    project_root: &Path,
    literal_pathspecs: bool,
    args: &[&str],
    pathspecs: &[P],
) -> Result<Vec<u8>> {
    let cap = LifecycleWorktreeCapability::get();
    crate::git::git_status(&cap, project_root, literal_pathspecs, args, pathspecs)
        .with_context(|| format!("lifecycle leaf {}", leaf.as_str()))
}

/// Return true when `project_root` is inside a git worktree.
pub fn is_git_repo(leaf: LifecycleLeaf, project_root: &Path) -> bool {
    let cap = LifecycleWorktreeCapability::get();
    crate::git::is_git_repo(&cap, project_root)
}

/// Run `git merge-base --is-ancestor` in `project_root`.
pub fn git_merge_base_is_ancestor(
    leaf: LifecycleLeaf,
    project_root: &Path,
    ancestor: &str,
    descendant: &str,
) -> Result<bool> {
    let cap = LifecycleHistoryCapability::get();
    crate::git::git_merge_base_is_ancestor(&cap, project_root, ancestor, descendant)
        .with_context(|| format!("lifecycle leaf {}", leaf.as_str()))
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

        let hcap = LifecycleHistoryCapability::for_test();

        // Verify exactly 1 new commit
        let rev_list = crate::git::git_rev_list(&hcap, repo, &["HEAD"]).unwrap();
        assert_eq!(
            rev_list.len(),
            2,
            "Expected exactly 2 commits (initial + 1 new)"
        );

        // Verify HEAD commit message is msg
        let head_msg = crate::git::git_log(&hcap, repo, &["-1", "--format=%B"]).unwrap();
        assert_eq!(
            head_msg.trim(),
            msg,
            "HEAD commit message should match passed message"
        );

        // Verify tree touches only the declared path (file1.txt)
        let diff_paths = crate::git::git_diff_name_only(&hcap, repo, &["HEAD~1", "HEAD"]).unwrap();
        assert_eq!(
            diff_paths,
            vec!["file1.txt"],
            "Commit tree should touch only file1.txt"
        );

        // Verify second file is still dirty and uncommitted
        let wcap = LifecycleWorktreeCapability::for_test();
        let dirty =
            crate::git::dirty_paths(&wcap, repo, &[std::path::PathBuf::from(".")], false).unwrap();
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

        for (rel_path, _, _) in leaf_allowlist {
            assert!(
                crate::git::lifecycle_commit_boundary::LIFECYCLE_LEAF_FILES.contains(&rel_path),
                "Routed leaf file {} absent from LIFECYCLE_LEAF_FILES constant",
                rel_path
            );
        }

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

    fn scan_source_for_direct_git_history_probes(rel_path: &str, content: &str) -> Vec<String> {
        let probes = [
            "git_log",
            "git_rev_parse",
            "git_rev_list",
            "git_show",
            "git_cat_file_blob",
            "git_diff_name_only",
            "git_merge_base_is_ancestor",
            "is_git_repo",
            "ensure_no_staged_changes",
            "dirty_paths",
            "has_staged_changes",
            "has_staged_changes_for_paths",
            "git_status",
        ];

        if rel_path == "git.rs" || rel_path == "lifecycle_commit.rs" {
            return Vec::new();
        }

        let mut violations = Vec::new();
        let mut in_cfg_test_depth: usize = 0;
        let mut pending_cfg_test = false;

        for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            let code_line = if let Some((code, _)) = line.split_once("//") {
                code
            } else {
                line
            };
            let code_trimmed = code_line.trim();

            if code_trimmed.starts_with("#[cfg(test)]") {
                pending_cfg_test = true;
            }

            if in_cfg_test_depth > 0 || pending_cfg_test {
                let open_braces = code_line.chars().filter(|&c| c == '{').count();
                let close_braces = code_line.chars().filter(|&c| c == '}').count();

                if pending_cfg_test {
                    if open_braces > 0 {
                        in_cfg_test_depth = open_braces.saturating_sub(close_braces);
                        pending_cfg_test = false;
                    } else if code_trimmed.ends_with(';') {
                        pending_cfg_test = false;
                    }
                } else {
                    in_cfg_test_depth =
                        (in_cfg_test_depth + open_braces).saturating_sub(close_braces);
                }
                continue;
            }

            for probe in probes {
                let match_fn = format!("crate::git::{probe}(");
                let match_fn2 = format!("git::{probe}(");
                let match_generic = format!("crate::git::{probe}<");
                let match_generic2 = format!("git::{probe}<");
                if code_line.contains(&match_fn)
                    || code_line.contains(&match_fn2)
                    || code_line.contains(&match_generic)
                    || code_line.contains(&match_generic2)
                {
                    violations.push(format!(
                        "{}:{}: Direct call to history probe {probe}: {}",
                        rel_path,
                        line_idx + 1,
                        trimmed
                    ));
                }
            }
        }

        violations
    }

    #[test]
    fn test_no_direct_git_history_probes_outside_lifecycle_commit() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| "apps/agentic-workflow".to_string());
        let src_dir = Path::new(&manifest_dir).join("src");

        let mut violations = Vec::new();
        let mut routed_history_calls = 0;
        let mut routed_worktree_calls = 0;

        fn walk_dir(
            dir: &Path,
            src_dir: &Path,
            violations: &mut Vec<String>,
            routed_history_calls: &mut usize,
            routed_worktree_calls: &mut usize,
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
                        violations,
                        routed_history_calls,
                        routed_worktree_calls,
                    );
                } else if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                    let rel_path = path
                        .strip_prefix(src_dir)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .replace('\\', "/");
                    let content = std::fs::read_to_string(&path)
                        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

                    let file_violations =
                        scan_source_for_direct_git_history_probes(&rel_path, &content);
                    violations.extend(file_violations);

                    if rel_path != "git.rs" && rel_path != "lifecycle_commit.rs" {
                        let history_probes = [
                            "git_log",
                            "git_rev_parse",
                            "git_rev_list",
                            "git_show",
                            "git_cat_file_blob",
                            "git_diff_name_only",
                        ];
                        for probe in history_probes {
                            let routed_pattern = format!("crate::lifecycle_commit::{probe}");
                            *routed_history_calls += content.matches(&routed_pattern).count();
                        }
                        let worktree_probes = [
                            "ensure_no_staged_changes",
                            "dirty_paths",
                            "has_staged_changes",
                            "has_staged_changes_for_paths",
                            "git_status",
                        ];
                        for probe in worktree_probes {
                            let routed_pattern = format!("crate::lifecycle_commit::{probe}");
                            *routed_worktree_calls += content.matches(&routed_pattern).count();
                        }
                    }
                }
            }
        }

        walk_dir(
            &src_dir,
            &src_dir,
            &mut violations,
            &mut routed_history_calls,
            &mut routed_worktree_calls,
        );

        assert!(
            violations.is_empty(),
            "Found direct calls to git history probes outside lifecycle_commit.rs / git.rs:\n{}",
            violations.join("\n")
        );

        assert!(
            routed_history_calls > 0,
            "Expected non-zero routed history calls in production code, found {routed_history_calls}"
        );

        assert!(
            routed_worktree_calls > 0,
            "Expected non-zero routed worktree calls in production code, found {routed_worktree_calls}"
        );
    }

    #[test]
    fn test_routed_history_probe_diff_name_only() {
        let temp = tempfile::TempDir::new().unwrap();
        let repo = temp.path();

        let git_bin = crate::git::find_git_bin()
            .expect("git binary must be available for history probe test");

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

        let file1 = repo.join("test_file.txt");
        std::fs::write(&file1, "initial content\n").unwrap();

        for args in [vec!["add", "."], vec!["commit", "-m", "first commit", "-q"]] {
            let out = std::process::Command::new(&git_bin)
                .args(&args)
                .current_dir(repo)
                .output()
                .expect("git initial commit failed");
            assert!(out.status.success(), "git {:?} failed", args);
        }

        std::fs::write(&file1, "modified content\n").unwrap();

        for args in [
            vec!["add", "."],
            vec!["commit", "-m", "second commit", "-q"],
        ] {
            let out = std::process::Command::new(&git_bin)
                .args(&args)
                .current_dir(repo)
                .output()
                .expect("git second commit failed");
            assert!(out.status.success(), "git {:?} failed", args);
        }

        let diff_paths = git_diff_name_only(LifecycleLeaf::Cb, repo, &["HEAD~1", "HEAD"])
            .expect("routed git_diff_name_only probe should succeed");

        assert_eq!(
            diff_paths,
            vec!["test_file.txt"],
            "Routed diff probe should return exactly the changed path from git diff output"
        );
    }

    #[test]
    fn test_routed_history_probes_leaf_error_context() {
        let temp = tempfile::TempDir::new().unwrap();
        let non_repo = temp.path();

        let err = git_log(LifecycleLeaf::Wi, non_repo, &["HEAD"])
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("lifecycle leaf wi"),
            "error message should contain leaf context: {err}"
        );
    }

    #[test]
    fn test_history_probe_negative_control_production_call() {
        let fixture = r#"
pub fn prod_fn() {
    let _ = crate::git::git_log(project_root, &["HEAD"]);
}
"#;
        let violations = scan_source_for_direct_git_history_probes("src/cli/offender.rs", fixture);
        assert_eq!(
            violations.len(),
            1,
            "expected 1 violation for direct git_log call in production"
        );
        assert!(
            violations[0].contains("src/cli/offender.rs:3:"),
            "got: {}",
            violations[0]
        );
    }

    #[test]
    fn test_history_probe_negative_control_cfg_test_call() {
        let fixture = r#"
#[cfg(test)]
mod tests {
    fn test_fn() {
        let _ = crate::git::git_rev_parse(project_root, &["HEAD"]);
    }
}
"#;
        let violations = scan_source_for_direct_git_history_probes("src/cli/test_file.rs", fixture);
        assert!(
            violations.is_empty(),
            "expected 0 violations for git_rev_parse in #[cfg(test)], got: {violations:?}"
        );
    }

    #[test]
    fn test_worktree_probe_negative_control_production_call() {
        let fixture = r#"
pub fn prod_fn() {
    let _ = crate::git::git_status(project_root, &scopes);
}
"#;
        let violations = scan_source_for_direct_git_history_probes("src/cli/offender.rs", fixture);
        assert_eq!(
            violations.len(),
            1,
            "expected 1 violation for direct git_status call in production"
        );
        assert!(
            violations[0].contains("src/cli/offender.rs:3:"),
            "got: {}",
            violations[0]
        );
    }

    #[test]
    fn test_worktree_probe_negative_control_cfg_test_call() {
        let fixture = r#"
#[cfg(test)]
mod tests {
    fn test_mod_fn() {
        let _ = crate::git::dirty_paths(cap, project_root, &scopes, true);
    }
}

#[cfg(test)]
fn test_item_fn() {
    let _ = crate::git::dirty_paths(cap, project_root, &scopes, true);
}
"#;
        let violations = scan_source_for_direct_git_history_probes("src/cli/test_file.rs", fixture);
        assert!(
            violations.is_empty(),
            "expected 0 violations for dirty_paths in #[cfg(test)] mod and item-level fn, got: {violations:?}"
        );
    }

    #[test]
    fn test_direct_git_history_probes_negative_control_all_eleven_production_calls() {
        let expected_probes = [
            "git_log",
            "git_rev_parse",
            "git_rev_list",
            "git_show",
            "git_cat_file_blob",
            "git_diff_name_only",
            "ensure_no_staged_changes",
            "dirty_paths",
            "has_staged_changes",
            "has_staged_changes_for_paths",
            "git_status",
        ];
        let fixture = r#"
pub fn prod_fn() {
    let _ = crate::git::git_log(project_root, &["HEAD"]);
    let _ = crate::git::git_rev_parse(project_root, &["HEAD"]);
    let _ = crate::git::git_rev_list(project_root, &["HEAD"]);
    let _ = crate::git::git_show(project_root, &["HEAD"]);
    let _ = crate::git::git_cat_file_blob(project_root, "oid");
    let _ = crate::git::git_diff_name_only(project_root, &["HEAD~1", "HEAD"]);
    let _ = crate::git::ensure_no_staged_changes(project_root);
    let _ = crate::git::dirty_paths(project_root, &scopes, true);
    let _ = crate::git::has_staged_changes(project_root);
    let _ = crate::git::has_staged_changes_for_paths(project_root, &paths, true);
    let _ = crate::git::git_status(project_root, true, &args, &pathspecs);
}
"#;
        let violations = scan_source_for_direct_git_history_probes("src/cli/offender.rs", fixture);
        assert_eq!(
            violations.len(),
            11,
            "expected 11 violations for direct calls to all 11 probe names in production, got: {violations:?}"
        );
        let violations_text = violations.join("\n");
        for probe in expected_probes {
            assert!(
                violations_text.contains(probe),
                "violations output should contain probe name {probe}, got:\n{violations_text}"
            );
        }
    }

    #[test]
    fn test_direct_git_history_probes_negative_control_all_eleven_cfg_test_calls() {
        let expected_probes = [
            "git_log",
            "git_rev_parse",
            "git_rev_list",
            "git_show",
            "git_cat_file_blob",
            "git_diff_name_only",
            "git_merge_base_is_ancestor",
            "is_git_repo",
            "ensure_no_staged_changes",
            "dirty_paths",
            "has_staged_changes",
            "has_staged_changes_for_paths",
            "git_status",
        ];
        let fixture = r#"
#[cfg(test)]
mod tests {
    fn test_mod_fn() {
        let _ = crate::git::git_log(project_root, &["HEAD"]);
        let _ = crate::git::git_rev_parse(project_root, &["HEAD"]);
        let _ = crate::git::git_rev_list(project_root, &["HEAD"]);
        let _ = crate::git::git_show(project_root, &["HEAD"]);
        let _ = crate::git::git_cat_file_blob(project_root, "oid");
        let _ = crate::git::git_diff_name_only(project_root, &["HEAD~1", "HEAD"]);
        let _ = crate::git::git_merge_base_is_ancestor(project_root, "feature", "HEAD");
        let _ = crate::git::is_git_repo(project_root);
        let _ = crate::git::ensure_no_staged_changes(project_root);
        let _ = crate::git::dirty_paths(project_root, &scopes, true);
        let _ = crate::git::has_staged_changes(project_root);
        let _ = crate::git::has_staged_changes_for_paths(project_root, &paths, true);
        let _ = crate::git::git_status(project_root, true, &args, &pathspecs);
    }
}

#[cfg(test)]
fn test_item_fn() {
    let _ = crate::git::git_log(project_root, &["HEAD"]);
    let _ = crate::git::git_rev_parse(project_root, &["HEAD"]);
    let _ = crate::git::git_rev_list(project_root, &["HEAD"]);
    let _ = crate::git::git_show(project_root, &["HEAD"]);
    let _ = crate::git::git_cat_file_blob(project_root, "oid");
    let _ = crate::git::git_diff_name_only(project_root, &["HEAD~1", "HEAD"]);
    let _ = crate::git::git_merge_base_is_ancestor(project_root, "feature", "HEAD");
    let _ = crate::git::is_git_repo(project_root);
    let _ = crate::git::ensure_no_staged_changes(project_root);
    let _ = crate::git::dirty_paths(project_root, &scopes, true);
    let _ = crate::git::has_staged_changes(project_root);
    let _ = crate::git::has_staged_changes_for_paths(project_root, &paths, true);
    let _ = crate::git::git_status(project_root, true, &args, &pathspecs);
}
"#;
        let violations = scan_source_for_direct_git_history_probes("src/cli/test_file.rs", fixture);
        assert!(
            violations.is_empty(),
            "expected 0 violations for all 13 probes in #[cfg(test)] mod and item-level fn, got: {violations:?}"
        );
        assert_eq!(expected_probes.len(), 13);
    }

    fn find_pub_fn_param_list(content: &str, probe: &str) -> Option<String> {
        let mut search_idx = 0;
        while let Some(pos) = content[search_idx..].find("pub fn") {
            let absolute_pos = search_idx + pos;
            search_idx = absolute_pos + 6;

            let rest = &content[search_idx..];
            let trimmed_rest = rest.trim_start();
            if !trimmed_rest.starts_with(probe) {
                continue;
            }

            let after_probe = &trimmed_rest[probe.len()..];
            let after_probe_trimmed = after_probe.trim_start();

            if !after_probe_trimmed.starts_with('(') && !after_probe_trimmed.starts_with('<') {
                continue;
            }

            let mut chars_indices = after_probe_trimmed.char_indices();
            let mut paren_start_pos = None;
            let mut angle_depth = 0;

            while let Some((idx, ch)) = chars_indices.next() {
                match ch {
                    '<' => angle_depth += 1,
                    '>' => {
                        if angle_depth > 0 {
                            angle_depth -= 1;
                        }
                    }
                    '(' if angle_depth == 0 => {
                        paren_start_pos = Some(idx);
                        break;
                    }
                    _ => {}
                }
            }

            let Some(start_idx) = paren_start_pos else {
                continue;
            };

            let mut paren_depth = 0;
            let mut end_idx = None;

            for (idx, ch) in after_probe_trimmed[start_idx..].char_indices() {
                match ch {
                    '(' => paren_depth += 1,
                    ')' => {
                        paren_depth -= 1;
                        if paren_depth == 0 {
                            end_idx = Some(start_idx + idx);
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if let Some(end_pos) = end_idx {
                return Some(after_probe_trimmed[start_idx + 1..end_pos].to_string());
            }
        }

        None
    }

    fn param_list_has_capability(param_list: &str, target_type: &str) -> bool {
        let clean_param_list: String = param_list
            .lines()
            .map(|line| {
                if let Some((code, _)) = line.split_once("//") {
                    code
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let mut current_param = String::new();
        let mut p_depth = 0;
        let mut a_depth = 0;
        let mut params = Vec::new();

        for ch in clean_param_list.chars() {
            match ch {
                '(' => {
                    p_depth += 1;
                    current_param.push(ch);
                }
                ')' => {
                    if p_depth > 0 {
                        p_depth -= 1;
                    }
                    current_param.push(ch);
                }
                '<' => {
                    a_depth += 1;
                    current_param.push(ch);
                }
                '>' => {
                    if a_depth > 0 {
                        a_depth -= 1;
                    }
                    current_param.push(ch);
                }
                ',' if p_depth == 0 && a_depth == 0 => {
                    params.push(std::mem::take(&mut current_param));
                }
                _ => {
                    current_param.push(ch);
                }
            }
        }
        if !current_param.trim().is_empty() {
            params.push(current_param);
        }

        for param in params {
            if let Some((_pat, type_part)) = param.split_once(':') {
                let normalized_type: String = type_part.split_whitespace().collect();
                if normalized_type == target_type {
                    return true;
                }
            }
        }

        false
    }

    fn mask_raw_strings_and_comments(content: &str) -> String {
        let bytes = content.as_bytes();
        let len = bytes.len();
        let mut masked = vec![b' '; len];
        let mut i = 0;

        while i < len {
            // Line comments //...
            if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'/' {
                while i < len && bytes[i] != b'\n' {
                    masked[i] = b' ';
                    i += 1;
                }
                if i < len && bytes[i] == b'\n' {
                    masked[i] = b'\n';
                    i += 1;
                }
                continue;
            }

            // Block comments /*...*/
            if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'*' {
                masked[i] = b' ';
                masked[i + 1] = b' ';
                i += 2;
                while i < len {
                    if i + 1 < len && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                        masked[i] = b' ';
                        masked[i + 1] = b' ';
                        i += 2;
                        break;
                    }
                    if bytes[i] == b'\n' {
                        masked[i] = b'\n';
                    } else {
                        masked[i] = b' ';
                    }
                    i += 1;
                }
                continue;
            }

            // Raw string literals r#"..."# or r"..." or r##"..."##
            if bytes[i] == b'r' {
                let mut hash_count = 0;
                let mut j = i + 1;
                while j < len && bytes[j] == b'#' {
                    hash_count += 1;
                    j += 1;
                }
                if j < len && bytes[j] == b'"' {
                    for k in i..=j {
                        masked[k] = b' ';
                    }
                    i = j + 1;
                    while i < len {
                        if bytes[i] == b'"' {
                            let mut match_hashes = true;
                            if i + hash_count < len {
                                for h in 1..=hash_count {
                                    if bytes[i + h] != b'#' {
                                        match_hashes = false;
                                        break;
                                    }
                                }
                            } else {
                                match_hashes = false;
                            }
                            if match_hashes {
                                for k in 0..=hash_count {
                                    masked[i + k] = b' ';
                                }
                                i += hash_count + 1;
                                break;
                            }
                        }
                        if bytes[i] == b'\n' {
                            masked[i] = b'\n';
                        } else {
                            masked[i] = b' ';
                        }
                        i += 1;
                    }
                    continue;
                }
            }

            // Normal string literals "..."
            if bytes[i] == b'"' {
                masked[i] = b' ';
                i += 1;
                while i < len {
                    if bytes[i] == b'\\' {
                        masked[i] = b' ';
                        if i + 1 < len {
                            if bytes[i + 1] == b'\n' {
                                masked[i + 1] = b'\n';
                            } else {
                                masked[i + 1] = b' ';
                            }
                            i += 2;
                            continue;
                        }
                    }
                    if bytes[i] == b'"' {
                        masked[i] = b' ';
                        i += 1;
                        break;
                    }
                    if bytes[i] == b'\n' {
                        masked[i] = b'\n';
                    } else {
                        masked[i] = b' ';
                    }
                    i += 1;
                }
                continue;
            }

            masked[i] = bytes[i];
            i += 1;
        }

        String::from_utf8(masked).unwrap()
    }

    struct PubFnDecl {
        name: String,
        param_list: String,
        body: String,
    }

    fn find_all_pub_fns(masked_content: &str) -> Vec<PubFnDecl> {
        let mut decls = Vec::new();
        let mut search_idx = 0;
        while let Some(pos) = masked_content[search_idx..].find("pub fn") {
            let abs_pos = search_idx + pos;
            search_idx = abs_pos + 6;

            let rest = &masked_content[search_idx..];
            let trimmed_rest = rest.trim_start();

            let name_end = trimmed_rest
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(trimmed_rest.len());
            if name_end == 0 {
                continue;
            }
            let fn_name = trimmed_rest[..name_end].to_string();

            let after_name = &trimmed_rest[name_end..];
            let after_name_trimmed = after_name.trim_start();

            if !after_name_trimmed.starts_with('(') && !after_name_trimmed.starts_with('<') {
                continue;
            }

            let mut angle_depth = 0;
            let mut paren_start_pos = None;
            for (idx, ch) in after_name_trimmed.char_indices() {
                match ch {
                    '<' => angle_depth += 1,
                    '>' => {
                        if angle_depth > 0 {
                            angle_depth -= 1;
                        }
                    }
                    '(' if angle_depth == 0 => {
                        paren_start_pos = Some(idx);
                        break;
                    }
                    _ => {}
                }
            }

            let Some(start_idx) = paren_start_pos else {
                continue;
            };

            let mut paren_depth = 0;
            let mut end_idx = None;
            for (idx, ch) in after_name_trimmed[start_idx..].char_indices() {
                match ch {
                    '(' => paren_depth += 1,
                    ')' => {
                        paren_depth -= 1;
                        if paren_depth == 0 {
                            end_idx = Some(start_idx + idx);
                            break;
                        }
                    }
                    _ => {}
                }
            }

            let Some(end_pos) = end_idx else {
                continue;
            };
            let param_list = after_name_trimmed[start_idx + 1..end_pos].to_string();

            let after_params = &after_name_trimmed[end_pos + 1..];
            let mut body = String::new();
            if let Some(brace_start) = after_params.find('{') {
                let mut brace_depth = 0;
                let mut body_end = None;
                for (idx, ch) in after_params[brace_start..].char_indices() {
                    match ch {
                        '{' => brace_depth += 1,
                        '}' => {
                            brace_depth -= 1;
                            if brace_depth == 0 {
                                body_end = Some(brace_start + idx);
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                if let Some(b_end) = body_end {
                    body = after_params[brace_start + 1..b_end].to_string();
                }
            }

            decls.push(PubFnDecl {
                name: fn_name,
                param_list,
                body,
            });
        }
        decls
    }

    fn scan_source_for_history_probe_capabilities(content: &str) -> Vec<String> {
        let mutating_primitives = [
            (
                "commit_scoped_paths",
                "&crate::lifecycle_commit::LifecycleCommitCapability",
            ),
            (
                "stage_paths",
                "&crate::lifecycle_commit::LifecycleCommitCapability",
            ),
            (
                "commit_staged",
                "&crate::lifecycle_commit::LifecycleCommitCapability",
            ),
            (
                "commit_only_paths",
                "&crate::lifecycle_commit::LifecycleCommitCapability",
            ),
            (
                "git_reset",
                "&crate::lifecycle_commit::LifecycleCommitCapability",
            ),
            (
                "git_merge",
                "&crate::lifecycle_commit::LifecycleCommitCapability",
            ),
        ];

        let history_probes = [
            (
                "git_log",
                "&crate::lifecycle_commit::LifecycleHistoryCapability",
            ),
            (
                "git_rev_parse",
                "&crate::lifecycle_commit::LifecycleHistoryCapability",
            ),
            (
                "git_rev_list",
                "&crate::lifecycle_commit::LifecycleHistoryCapability",
            ),
            (
                "git_show",
                "&crate::lifecycle_commit::LifecycleHistoryCapability",
            ),
            (
                "git_cat_file_blob",
                "&crate::lifecycle_commit::LifecycleHistoryCapability",
            ),
            (
                "git_diff_name_only",
                "&crate::lifecycle_commit::LifecycleHistoryCapability",
            ),
            (
                "git_merge_base_is_ancestor",
                "&crate::lifecycle_commit::LifecycleHistoryCapability",
            ),
        ];

        let worktree_probes = [
            (
                "is_git_repo",
                "&crate::lifecycle_commit::LifecycleWorktreeCapability",
            ),
            (
                "ensure_no_staged_changes",
                "&crate::lifecycle_commit::LifecycleWorktreeCapability",
            ),
            (
                "dirty_paths",
                "&crate::lifecycle_commit::LifecycleWorktreeCapability",
            ),
            (
                "has_staged_changes",
                "&crate::lifecycle_commit::LifecycleWorktreeCapability",
            ),
            (
                "has_staged_changes_for_paths",
                "&crate::lifecycle_commit::LifecycleWorktreeCapability",
            ),
            (
                "git_status",
                "&crate::lifecycle_commit::LifecycleWorktreeCapability",
            ),
        ];

        let exemptions = ["find_git_bin", "find_rustfmt_bin"];

        let masked = mask_raw_strings_and_comments(content);
        let decls = find_all_pub_fns(&masked);

        let mut violations = Vec::new();

        // Direction 1: Every production pub fn found in git.rs must be validly registered and declared.
        for decl in &decls {
            if let Some((_, target_type)) = mutating_primitives
                .iter()
                .find(|(name, _)| *name == decl.name)
            {
                if !param_list_has_capability(&decl.param_list, target_type) {
                    violations.push(format!(
                        "Probe {} in git.rs does not declare a parameter of type {}",
                        decl.name, target_type
                    ));
                }
            } else if let Some((_, target_type)) =
                history_probes.iter().find(|(name, _)| *name == decl.name)
            {
                if !param_list_has_capability(&decl.param_list, target_type) {
                    violations.push(format!(
                        "Probe {} in git.rs does not declare a parameter of type {}",
                        decl.name, target_type
                    ));
                }
            } else if let Some((_, target_type)) =
                worktree_probes.iter().find(|(name, _)| *name == decl.name)
            {
                if !param_list_has_capability(&decl.param_list, target_type) {
                    violations.push(format!(
                        "Probe {} in git.rs does not declare a parameter of type {}",
                        decl.name, target_type
                    ));
                }
            } else if exemptions.contains(&decl.name.as_str()) {
                if param_list_has_capability(&decl.param_list, "LifecycleCommitCapability")
                    || param_list_has_capability(&decl.param_list, "LifecycleHistoryCapability")
                    || param_list_has_capability(&decl.param_list, "LifecycleWorktreeCapability")
                {
                    violations.push(format!(
                        "Exempt function {} in git.rs should not declare a capability parameter",
                        decl.name
                    ));
                }
                if decl.body.contains("Command::new") {
                    violations.push(format!(
                        "Exempt function {} in git.rs spawns a child process via Command::new",
                        decl.name
                    ));
                }
            } else {
                violations.push(format!(
                    "Probe {} in git.rs declaration not found on registered probe or exemption lists",
                    decl.name
                ));
            }
        }

        // Direction 2: Every registered probe/exemption must exist as a pub fn in git.rs.
        for (probe, _) in history_probes
            .iter()
            .chain(worktree_probes.iter())
            .chain(mutating_primitives.iter())
        {
            if !decls.iter().any(|d| d.name == *probe) {
                violations.push(format!(
                    "Probe {probe} declaration not found as pub fn in git.rs"
                ));
            }
        }
        for name in &exemptions {
            if !decls.iter().any(|d| d.name == *name) {
                violations.push(format!(
                    "Exempt function {name} declaration not found as pub fn in git.rs"
                ));
            }
        }

        violations
    }

    #[test]
    fn test_history_probes_declare_capability_parameter() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| "apps/agentic-workflow".to_string());
        let git_rs_path = Path::new(&manifest_dir).join("src").join("git.rs");
        let content = std::fs::read_to_string(&git_rs_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", git_rs_path.display(), e));

        let violations = scan_source_for_history_probe_capabilities(&content);
        assert!(
            violations.is_empty(),
            "Probe capability parameter violations in git.rs:\n{}",
            violations.join("\n")
        );
    }

    #[test]
    fn test_history_probe_capability_scanner_negative_control_missing_parameter() {
        let fixture = r#"
pub fn find_git_bin() -> Option<PathBuf> { None }
pub fn find_rustfmt_bin() -> Option<PathBuf> { None }
pub fn is_git_repo(_cap: &crate::lifecycle_commit::LifecycleWorktreeCapability, project_root: &Path) -> bool { true }
pub fn ensure_no_staged_changes(_cap: &crate::lifecycle_commit::LifecycleWorktreeCapability, project_root: &Path) -> Result<()> { Ok(()) }
pub fn commit_scoped_paths(_cap: &crate::lifecycle_commit::LifecycleCommitCapability, project_root: &Path, paths: &[PathBuf], message: &str) -> Result<bool> { Ok(true) }
pub fn dirty_paths(_cap: &crate::lifecycle_commit::LifecycleWorktreeCapability, project_root: &Path, scopes: &[PathBuf], include_untracked: bool) -> Result<Vec<String>> { Ok(Vec::new()) }
pub fn stage_paths<P: AsRef<Path>>(_cap: &crate::lifecycle_commit::LifecycleCommitCapability, project_root: &Path, paths: &[P], literal_pathspecs: bool) -> Result<()> { Ok(()) }
pub fn has_staged_changes(_cap: &crate::lifecycle_commit::LifecycleWorktreeCapability, project_root: &Path) -> Result<bool> { Ok(true) }
pub fn has_staged_changes_for_paths<P: AsRef<Path>>(_cap: &crate::lifecycle_commit::LifecycleWorktreeCapability, project_root: &Path, paths: &[P], literal_pathspecs: bool) -> Result<bool> { Ok(true) }
pub fn commit_staged(_cap: &crate::lifecycle_commit::LifecycleCommitCapability, project_root: &Path, message: &str, allow_empty: bool) -> Result<()> { Ok(()) }
pub fn commit_only_paths<P: AsRef<Path>>(_cap: &crate::lifecycle_commit::LifecycleCommitCapability, project_root: &Path, paths: &[P], message: &str, literal_pathspecs: bool) -> Result<()> { Ok(()) }
pub fn git_log(
    project_root: &Path,
    args: &[&str],
) -> Result<String> {
    Ok(String::new())
}
pub fn git_rev_parse(
    _cap: &crate::lifecycle_commit::LifecycleHistoryCapability,
    project_root: &Path,
) -> Result<String> {
    Ok(String::new())
}
pub fn git_rev_list(
    _cap: &crate::lifecycle_commit::LifecycleHistoryCapability,
) -> Result<Vec<String>> {
    Ok(Vec::new())
}
pub fn git_show(
    _cap: &crate::lifecycle_commit::LifecycleHistoryCapability,
) -> Result<String> {
    Ok(String::new())
}
pub fn git_cat_file_blob(
    _cap: &crate::lifecycle_commit::LifecycleHistoryCapability,
) -> Result<Vec<u8>> {
    Ok(Vec::new())
}
pub fn git_diff_name_only(
    _cap: &crate::lifecycle_commit::LifecycleHistoryCapability,
) -> Result<Vec<String>> {
    Ok(Vec::new())
}
pub fn git_merge_base_is_ancestor(
    _cap: &crate::lifecycle_commit::LifecycleHistoryCapability,
    project_root: &Path,
    ancestor: &str,
    descendant: &str,
) -> Result<bool> { Ok(true) }
pub fn git_reset(_cap: &crate::lifecycle_commit::LifecycleCommitCapability, project_root: &Path, args: &[&str]) -> Result<()> { Ok(()) }
pub fn git_merge(_cap: &crate::lifecycle_commit::LifecycleCommitCapability, project_root: &Path, args: &[&str]) -> Result<()> { Ok(()) }
pub fn git_status<P: AsRef<Path>>(_cap: &crate::lifecycle_commit::LifecycleWorktreeCapability, project_root: &Path, literal_pathspecs: bool, args: &[&str], pathspecs: &[P]) -> Result<Vec<u8>> { Ok(Vec::new()) }
"#;
        let violations = scan_source_for_history_probe_capabilities(fixture);
        assert_eq!(
            violations.len(),
            1,
            "expected 1 violation when git_log capability parameter is missing"
        );
        assert!(
            violations[0].contains("git_log"),
            "violation should name the probe git_log, got: {}",
            violations[0]
        );
    }

    #[test]
    fn test_history_probe_capability_scanner_negative_control_all_thirteen_missing_parameters() {
        let expected_probes = [
            "git_log",
            "git_rev_parse",
            "git_rev_list",
            "git_show",
            "git_cat_file_blob",
            "git_diff_name_only",
            "git_merge_base_is_ancestor",
            "is_git_repo",
            "ensure_no_staged_changes",
            "dirty_paths",
            "has_staged_changes",
            "has_staged_changes_for_paths",
            "git_status",
        ];
        let fixture = r#"
pub fn find_git_bin() -> Option<PathBuf> { None }
pub fn find_rustfmt_bin() -> Option<PathBuf> { None }
pub fn commit_scoped_paths(_cap: &crate::lifecycle_commit::LifecycleCommitCapability, project_root: &Path, paths: &[PathBuf], message: &str) -> Result<bool> { Ok(true) }
pub fn stage_paths<P: AsRef<Path>>(_cap: &crate::lifecycle_commit::LifecycleCommitCapability, project_root: &Path, paths: &[P], literal_pathspecs: bool) -> Result<()> { Ok(()) }
pub fn commit_staged(_cap: &crate::lifecycle_commit::LifecycleCommitCapability, project_root: &Path, message: &str, allow_empty: bool) -> Result<()> { Ok(()) }
pub fn commit_only_paths<P: AsRef<Path>>(_cap: &crate::lifecycle_commit::LifecycleCommitCapability, project_root: &Path, paths: &[P], message: &str, literal_pathspecs: bool) -> Result<()> { Ok(()) }
pub fn git_reset(_cap: &crate::lifecycle_commit::LifecycleCommitCapability, project_root: &Path, args: &[&str]) -> Result<()> { Ok(()) }
pub fn git_merge(_cap: &crate::lifecycle_commit::LifecycleCommitCapability, project_root: &Path, args: &[&str]) -> Result<()> { Ok(()) }
pub fn is_git_repo(project_root: &Path) -> bool { true }
pub fn git_log(
    project_root: &Path,
    args: &[&str],
) -> Result<String> {
    Ok(String::new())
}
pub fn git_rev_parse(
    project_root: &Path,
) -> Result<String> {
    Ok(String::new())
}
pub fn git_rev_list(
    project_root: &Path,
) -> Result<Vec<String>> {
    Ok(Vec::new())
}
pub fn git_show(
    project_root: &Path,
) -> Result<String> {
    Ok(String::new())
}
pub fn git_cat_file_blob(
    project_root: &Path,
) -> Result<Vec<u8>> {
    Ok(Vec::new())
}
pub fn git_diff_name_only(
    project_root: &Path,
) -> Result<Vec<String>> {
    Ok(Vec::new())
}
pub fn git_merge_base_is_ancestor(
    project_root: &Path,
    ancestor: &str,
    descendant: &str,
) -> Result<bool> { Ok(true) }
pub fn ensure_no_staged_changes(
    project_root: &Path,
) -> Result<()> { Ok(()) }
pub fn dirty_paths(project_root: &Path, scopes: &[PathBuf], include_untracked: bool) -> Result<Vec<String>> { Ok(Vec::new()) }
pub fn has_staged_changes(project_root: &Path) -> Result<bool> { Ok(true) }
pub fn has_staged_changes_for_paths<P: AsRef<Path>>(project_root: &Path, paths: &[P], literal_pathspecs: bool) -> Result<bool> { Ok(true) }
pub fn git_status<P: AsRef<Path>>(project_root: &Path, literal_pathspecs: bool, args: &[&str], pathspecs: &[P]) -> Result<Vec<u8>> { Ok(Vec::new()) }
"#;
        let violations = scan_source_for_history_probe_capabilities(fixture);
        assert_eq!(
            violations.len(),
            13,
            "expected 13 violations when all 13 probe capability parameters are missing, got: {violations:?}"
        );
        let violations_text = violations.join("\n");
        for probe in expected_probes {
            assert!(
                violations_text.contains(probe),
                "violations output should contain probe name {probe}, got:\n{violations_text}"
            );
        }
    }

    #[test]
    fn test_history_probe_capability_scanner_reflow_and_rename() {
        let fixture = r#"
pub fn find_git_bin() -> Option<PathBuf> { None }
pub fn find_rustfmt_bin() -> Option<PathBuf> { None }
pub fn is_git_repo(_cap: &crate::lifecycle_commit::LifecycleWorktreeCapability, project_root: &Path) -> bool { true }
pub fn ensure_no_staged_changes(_cap: &crate::lifecycle_commit::LifecycleWorktreeCapability, project_root: &Path) -> Result<()> { Ok(()) }
pub fn commit_scoped_paths(_cap: &crate::lifecycle_commit::LifecycleCommitCapability, project_root: &Path, paths: &[PathBuf], message: &str) -> Result<bool> { Ok(true) }
pub fn dirty_paths(_cap: &crate::lifecycle_commit::LifecycleWorktreeCapability, project_root: &Path, scopes: &[PathBuf], include_untracked: bool) -> Result<Vec<String>> { Ok(Vec::new()) }
pub fn stage_paths<P: AsRef<Path>>(_cap: &crate::lifecycle_commit::LifecycleCommitCapability, project_root: &Path, paths: &[P], literal_pathspecs: bool) -> Result<()> { Ok(()) }
pub fn has_staged_changes(_cap: &crate::lifecycle_commit::LifecycleWorktreeCapability, project_root: &Path) -> Result<bool> { Ok(true) }
pub fn has_staged_changes_for_paths<P: AsRef<Path>>(_cap: &crate::lifecycle_commit::LifecycleWorktreeCapability, project_root: &Path, paths: &[P], literal_pathspecs: bool) -> Result<bool> { Ok(true) }
pub fn commit_staged(_cap: &crate::lifecycle_commit::LifecycleCommitCapability, project_root: &Path, message: &str, allow_empty: bool) -> Result<()> { Ok(()) }
pub fn commit_only_paths<P: AsRef<Path>>(_cap: &crate::lifecycle_commit::LifecycleCommitCapability, project_root: &Path, paths: &[P], message: &str, literal_pathspecs: bool) -> Result<()> { Ok(()) }
pub fn git_log(
    custom_cap_name:
        &crate::lifecycle_commit::LifecycleHistoryCapability,
    project_root: &Path,
) -> Result<String> {
    Ok(String::new())
}
pub fn git_rev_parse(_cap: &crate::lifecycle_commit::LifecycleHistoryCapability) -> Result<String> {
    Ok(String::new())
}
pub fn git_rev_list(_cap: &crate::lifecycle_commit::LifecycleHistoryCapability) -> Result<Vec<String>> {
    Ok(Vec::new())
}
pub fn git_show(_cap: &crate::lifecycle_commit::LifecycleHistoryCapability) -> Result<String> {
    Ok(String::new())
}
pub fn git_cat_file_blob(_cap: &crate::lifecycle_commit::LifecycleHistoryCapability) -> Result<Vec<u8>> {
    Ok(Vec::new())
}
pub fn git_diff_name_only(_cap: &crate::lifecycle_commit::LifecycleHistoryCapability) -> Result<Vec<String>> {
    Ok(Vec::new())
}
pub fn git_merge_base_is_ancestor(_cap: &crate::lifecycle_commit::LifecycleHistoryCapability, project_root: &Path, ancestor: &str, descendant: &str) -> Result<bool> { Ok(true) }
pub fn git_reset(_cap: &crate::lifecycle_commit::LifecycleCommitCapability, project_root: &Path, args: &[&str]) -> Result<()> { Ok(()) }
pub fn git_merge(_cap: &crate::lifecycle_commit::LifecycleCommitCapability, project_root: &Path, args: &[&str]) -> Result<()> { Ok(()) }
pub fn git_status<P: AsRef<Path>>(_cap: &crate::lifecycle_commit::LifecycleWorktreeCapability, project_root: &Path, literal_pathspecs: bool, args: &[&str], pathspecs: &[P]) -> Result<Vec<u8>> { Ok(Vec::new()) }
"#;
        let violations = scan_source_for_history_probe_capabilities(fixture);
        assert!(
            violations.is_empty(),
            "expected 0 violations for renamed parameter or reflowed signature, got: {violations:?}"
        );
    }

    #[test]
    fn test_routed_worktree_probe_has_staged_changes() {
        let temp = tempfile::TempDir::new().unwrap();
        let repo = temp.path();

        let git_bin = crate::git::find_git_bin()
            .expect("git binary must be available for worktree probe test");

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

        let file1 = repo.join("test_file.txt");
        std::fs::write(&file1, "staged content\n").unwrap();

        let out = std::process::Command::new(&git_bin)
            .args(["add", "."])
            .current_dir(repo)
            .output()
            .expect("git add failed");
        assert!(out.status.success());

        let staged = has_staged_changes(LifecycleLeaf::Cb, repo)
            .expect("routed has_staged_changes probe should succeed");
        assert!(
            staged,
            "routed has_staged_changes should return true when changes are staged"
        );

        let out = std::process::Command::new(&git_bin)
            .args(["reset"])
            .current_dir(repo)
            .output()
            .expect("git reset failed");
        assert!(out.status.success());

        let staged_after_reset = has_staged_changes(LifecycleLeaf::Cb, repo)
            .expect("routed has_staged_changes probe should succeed");
        assert!(
            !staged_after_reset,
            "routed has_staged_changes should return false after git reset"
        );
    }

    #[test]
    fn test_routed_worktree_probes_leaf_error_context() {
        let temp = tempfile::TempDir::new().unwrap();
        let non_repo = temp.path();

        let err = ensure_no_staged_changes(LifecycleLeaf::Wi, non_repo)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("lifecycle leaf wi"),
            "error message should contain leaf context: {err}"
        );
    }
}
