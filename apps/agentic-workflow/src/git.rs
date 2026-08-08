// SPEC-MANAGED: apps/agentic-workflow/tech-design/core/logic/git.md#source
// CODEGEN-BEGIN
//! Neutral home for `find_git_bin` — formerly under `crate::worktree`.
//!
//! @spec apps/agentic-workflow/tech-design/core/worktree-retirement.md#schema (R5)
//!
//! `worktree.rs` was gutted in Phase C. Callers that still need to
//! locate the `git` binary (in-place CRRR helpers, the one-shot
//! `score migrate-worktrees` verb) import this module instead.

use anyhow::{Context, Result};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Locate the `git` binary on `PATH`. Returns `None` if `which git`
/// fails or returns an empty string.
/// @spec apps/agentic-workflow/tech-design/core/logic/git.md#source
pub fn find_git_bin() -> Option<PathBuf> {
    find_bin_on_path("git", std::env::var_os("PATH")).or_else(find_default_git_bin)
}

pub fn find_rustfmt_bin() -> Option<PathBuf> {
    find_bin_on_path("rustfmt", std::env::var_os("PATH")).or_else(find_default_rustfmt_bin)
}

fn find_default_git_bin() -> Option<PathBuf> {
    [
        "/opt/homebrew/bin/git",
        "/usr/local/bin/git",
        "/usr/bin/git",
    ]
    .iter()
    .map(PathBuf::from)
    .find(|path| path.is_file())
}

fn find_default_rustfmt_bin() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(cargo_home) = std::env::var_os("CARGO_HOME") {
        candidates.push(PathBuf::from(cargo_home).join("bin/rustfmt"));
    }
    if let Some(rustup_home) = std::env::var_os("RUSTUP_HOME") {
        let root = PathBuf::from(rustup_home).join("toolchains");
        candidates.push(root.join("stable-aarch64-apple-darwin/bin/rustfmt"));
        candidates.push(root.join("stable-x86_64-apple-darwin/bin/rustfmt"));
    }
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        candidates.push(home.join(".cargo/bin/rustfmt"));
        candidates.push(home.join(".rustup/toolchains/stable-aarch64-apple-darwin/bin/rustfmt"));
        candidates.push(home.join(".rustup/toolchains/stable-x86_64-apple-darwin/bin/rustfmt"));
    }
    candidates.extend([
        PathBuf::from("/opt/homebrew/bin/rustfmt"),
        PathBuf::from("/usr/local/bin/rustfmt"),
    ]);
    candidates.into_iter().find(|path| path.is_file())
}

fn find_bin_on_path(binary: &str, path_env: Option<impl AsRef<OsStr>>) -> Option<PathBuf> {
    let path_env = path_env?;
    std::env::split_paths(path_env.as_ref())
        .map(|dir| dir.join(binary))
        .find(|path| path.is_file())
}

/// Return true when `project_root` is inside a git worktree.
/// @spec apps/agentic-workflow/tech-design/core/logic/git.md#source
pub fn is_git_repo(project_root: &Path) -> bool {
    let Some(git_bin) = find_git_bin() else {
        return false;
    };
    std::process::Command::new(git_bin)
        .arg("-C")
        .arg(project_root)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// Refuse to mix lifecycle commits with already-staged user changes.
/// @spec apps/agentic-workflow/tech-design/core/logic/git.md#source
pub fn ensure_no_staged_changes(project_root: &Path) -> Result<()> {
    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let output = std::process::Command::new(git_bin)
        .arg("-C")
        .arg(project_root)
        .args(["diff", "--cached", "--name-only"])
        .output()
        .context("git diff --cached failed")?;
    if !output.status.success() {
        anyhow::bail!(
            "git diff --cached failed in {}: {}",
            project_root.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let staged = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !staged.is_empty() {
        anyhow::bail!(
            "refusing to commit lifecycle changes with pre-existing staged paths:\n{}\n\
             Unstage only those paths with `git restore --staged -- <path>` and re-run; unrelated unstaged or untracked work is left untouched.",
            staged,
        );
    }
    Ok(())
}

/// Stage exactly `paths`, create `message` as a lifecycle commit, and no-op
/// when those paths have no staged diff.
/// @spec apps/agentic-workflow/tech-design/core/logic/git.md#source
pub fn commit_scoped_paths(
    _cap: &crate::lifecycle_commit::LifecycleCommitCapability,
    project_root: &Path,
    paths: &[PathBuf],
    message: &str,
) -> Result<bool> {
    if paths.is_empty() || !is_git_repo(project_root) {
        return Ok(false);
    }
    ensure_no_staged_changes(project_root)?;
    let rel_paths = repo_relative_paths(project_root, paths)?;
    if rel_paths.is_empty() {
        return Ok(false);
    }

    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let mut add = std::process::Command::new(&git_bin);
    add.arg("-C").arg(project_root).arg("add").arg("--");
    for path in &rel_paths {
        add.arg(path);
    }
    let add_out = add.output().context("git add failed")?;
    if !add_out.status.success() {
        anyhow::bail!(
            "git add failed: {}",
            String::from_utf8_lossy(&add_out.stderr).trim()
        );
    }

    let mut diff = std::process::Command::new(&git_bin);
    diff.arg("-C")
        .arg(project_root)
        .args(["diff", "--cached", "--quiet", "--"]);
    for path in &rel_paths {
        diff.arg(path);
    }
    let staged = diff.status().context("git diff --cached failed")?;
    if staged.success() {
        return Ok(false);
    }

    let mut commit = std::process::Command::new(&git_bin);
    commit
        .arg("-C")
        .arg(project_root)
        .args(["commit", "-m"])
        .arg(message)
        .arg("--");
    for path in &rel_paths {
        commit.arg(path);
    }
    let commit_out = commit.output().context("git commit failed")?;
    if !commit_out.status.success() {
        anyhow::bail!(
            "git commit failed: {}",
            String::from_utf8_lossy(&commit_out.stderr).trim()
        );
    }
    Ok(true)
}

/// Return dirty paths under the supplied repo-relative or absolute scopes.
/// @spec apps/agentic-workflow/tech-design/core/logic/git.md#source
pub fn dirty_paths(
    project_root: &Path,
    scopes: &[PathBuf],
    include_untracked: bool,
) -> Result<Vec<String>> {
    if scopes.is_empty() || !is_git_repo(project_root) {
        return Ok(Vec::new());
    }
    let rel_scopes = repo_relative_paths(project_root, scopes)?;
    if rel_scopes.is_empty() {
        return Ok(Vec::new());
    }
    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let mut status = std::process::Command::new(git_bin);
    status
        .arg("-C")
        .arg(project_root)
        .args(["status", "--porcelain=v1"]);
    if include_untracked {
        status.arg("--untracked-files=all");
    } else {
        status.arg("--untracked-files=no");
    }
    status.arg("--");
    for scope in &rel_scopes {
        status.arg(scope);
    }
    let out = status.output().context("git status failed")?;
    if !out.status.success() {
        anyhow::bail!(
            "git status failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    let mut paths = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| line.get(3..))
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    Ok(paths)
}

/// Stage specified `paths` into git index.
pub fn stage_paths<P: AsRef<Path>>(
    _cap: &crate::lifecycle_commit::LifecycleCommitCapability,
    project_root: &Path,
    paths: &[P],
    literal_pathspecs: bool,
) -> Result<()> {
    if paths.is_empty() {
        return Ok(());
    }
    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let mut command = std::process::Command::new(&git_bin);
    if literal_pathspecs {
        command.arg("--literal-pathspecs");
    }
    command.arg("-C").arg(project_root).arg("add").arg("--");
    for path in paths {
        command.arg(path.as_ref());
    }
    let add_out = command.output().context("git add failed")?;
    if !add_out.status.success() {
        anyhow::bail!(
            "git add failed: {}",
            String::from_utf8_lossy(&add_out.stderr).trim()
        );
    }
    Ok(())
}

fn git_diff_has_changes(status: std::process::ExitStatus, operation: &str) -> Result<bool> {
    match status.code() {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        Some(code) => anyhow::bail!("{operation} failed with exit code {code}"),
        None => anyhow::bail!("{operation} terminated without an exit code"),
    }
}

/// Check if `project_root` has any staged changes.
pub fn has_staged_changes(project_root: &Path) -> Result<bool> {
    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let status = std::process::Command::new(git_bin)
        .arg("-C")
        .arg(project_root)
        .args(["diff", "--cached", "--quiet"])
        .status()
        .context("git diff --cached failed")?;
    git_diff_has_changes(status, "git diff --cached")
}

/// Check if specified `paths` in `project_root` have staged changes.
pub fn has_staged_changes_for_paths<P: AsRef<Path>>(
    project_root: &Path,
    paths: &[P],
    literal_pathspecs: bool,
) -> Result<bool> {
    if paths.is_empty() {
        return Ok(false);
    }
    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let mut command = std::process::Command::new(&git_bin);
    if literal_pathspecs {
        command.arg("--literal-pathspecs");
    }
    command
        .arg("-C")
        .arg(project_root)
        .args(["diff", "--cached", "--quiet", "--"]);
    for path in paths {
        command.arg(path.as_ref());
    }
    let status = command.status().context("git diff --cached failed")?;
    git_diff_has_changes(status, "git diff --cached")
}

/// Commit already staged changes with `message`. If `allow_empty` is true, pass `--allow-empty`.
pub fn commit_staged(
    _cap: &crate::lifecycle_commit::LifecycleCommitCapability,
    project_root: &Path,
    message: &str,
    allow_empty: bool,
) -> Result<()> {
    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let mut command = std::process::Command::new(&git_bin);
    command.arg("-C").arg(project_root).arg("commit");
    if allow_empty {
        command.arg("--allow-empty");
    }
    command.args(["-m", message]);
    let output = command.output().context("git commit failed")?;
    if !output.status.success() {
        anyhow::bail!(
            "git commit failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

/// Create a path-scoped commit using `git commit --only -- <paths>`.
pub fn commit_only_paths<P: AsRef<Path>>(
    _cap: &crate::lifecycle_commit::LifecycleCommitCapability,
    project_root: &Path,
    paths: &[P],
    message: &str,
    literal_pathspecs: bool,
) -> Result<()> {
    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let mut command = std::process::Command::new(&git_bin);
    if literal_pathspecs {
        command.arg("--literal-pathspecs");
    }
    command
        .arg("-C")
        .arg(project_root)
        .args(["commit", "--only", "-m", message, "--"]);
    for path in paths {
        command.arg(path.as_ref());
    }
    let output = command.output().context("git commit failed")?;
    if !output.status.success() {
        anyhow::bail!(
            "git commit failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

fn repo_relative_paths(project_root: &Path, paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut rel_paths = Vec::new();
    for path in paths {
        let rel = if path.is_absolute() {
            path.strip_prefix(project_root).with_context(|| {
                format!(
                    "{} is outside git worktree {}",
                    path.display(),
                    project_root.display()
                )
            })?
        } else {
            path.as_path()
        };
        if rel.as_os_str().is_empty() {
            continue;
        }
        rel_paths.push(rel.to_path_buf());
    }
    rel_paths.sort();
    rel_paths.dedup();
    Ok(rel_paths)
}

fn format_git_error(op: &str, status: std::process::ExitStatus, stderr: &[u8]) -> anyhow::Error {
    let stderr_str = String::from_utf8_lossy(stderr);
    let trimmed = stderr_str.trim();
    if let Some(code) = status.code() {
        anyhow::anyhow!("{op} failed with exit code {code}: {trimmed}")
    } else {
        anyhow::anyhow!("{op} terminated without an exit code: {trimmed}")
    }
}

/// Run `git log` with args in `project_root`.
pub fn git_log(
    _cap: &crate::lifecycle_commit::LifecycleHistoryCapability,
    project_root: &Path,
    args: &[&str],
) -> Result<String> {
    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let output = std::process::Command::new(git_bin)
        .arg("-C")
        .arg(project_root)
        .arg("log")
        .args(args)
        .output()
        .context("git log failed")?;
    if !output.status.success() {
        return Err(format_git_error("git log", output.status, &output.stderr));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Run `git rev-parse` with args in `project_root`.
pub fn git_rev_parse(
    _cap: &crate::lifecycle_commit::LifecycleHistoryCapability,
    project_root: &Path,
    args: &[&str],
) -> Result<String> {
    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let output = std::process::Command::new(git_bin)
        .arg("-C")
        .arg(project_root)
        .arg("rev-parse")
        .args(args)
        .output()
        .context("git rev-parse failed")?;
    if !output.status.success() {
        return Err(format_git_error(
            "git rev-parse",
            output.status,
            &output.stderr,
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Run `git rev-list` with args in `project_root`.
pub fn git_rev_list(
    _cap: &crate::lifecycle_commit::LifecycleHistoryCapability,
    project_root: &Path,
    args: &[&str],
) -> Result<Vec<String>> {
    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let output = std::process::Command::new(git_bin)
        .arg("-C")
        .arg(project_root)
        .arg("rev-list")
        .args(args)
        .output()
        .context("git rev-list failed")?;
    if !output.status.success() {
        return Err(format_git_error(
            "git rev-list",
            output.status,
            &output.stderr,
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

/// Run `git merge-base --is-ancestor` in `project_root`.
pub fn git_merge_base_is_ancestor(
    project_root: &Path,
    ancestor: &str,
    descendant: &str,
) -> Result<bool> {
    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let output = std::process::Command::new(git_bin)
        .arg("-C")
        .arg(project_root)
        .args(["merge-base", "--is-ancestor", ancestor, descendant])
        .output()
        .context("git merge-base --is-ancestor failed")?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        Some(_) => Err(format_git_error(
            "git merge-base --is-ancestor",
            output.status,
            &output.stderr,
        )),
        None => anyhow::bail!("git merge-base --is-ancestor terminated without an exit code"),
    }
}

/// Run `git cat-file blob <object>` in `project_root`.
pub fn git_cat_file_blob(
    _cap: &crate::lifecycle_commit::LifecycleHistoryCapability,
    project_root: &Path,
    object: &str,
) -> Result<Vec<u8>> {
    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let output = std::process::Command::new(git_bin)
        .arg("-C")
        .arg(project_root)
        .args(["cat-file", "blob", object])
        .output()
        .context("git cat-file failed")?;
    if !output.status.success() {
        return Err(format_git_error(
            "git cat-file",
            output.status,
            &output.stderr,
        ));
    }
    Ok(output.stdout)
}

/// Run `git show` with args in `project_root`.
pub fn git_show(
    _cap: &crate::lifecycle_commit::LifecycleHistoryCapability,
    project_root: &Path,
    args: &[&str],
) -> Result<String> {
    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let output = std::process::Command::new(git_bin)
        .arg("-C")
        .arg(project_root)
        .arg("show")
        .args(args)
        .output()
        .context("git show failed")?;
    if !output.status.success() {
        return Err(format_git_error("git show", output.status, &output.stderr));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Run `git reset` with args in `project_root`.
pub fn git_reset(
    _cap: &crate::lifecycle_commit::LifecycleCommitCapability,
    project_root: &Path,
    args: &[&str],
) -> Result<()> {
    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let output = std::process::Command::new(git_bin)
        .arg("-C")
        .arg(project_root)
        .arg("reset")
        .args(args)
        .output()
        .context("git reset failed")?;
    if !output.status.success() {
        return Err(format_git_error("git reset", output.status, &output.stderr));
    }
    Ok(())
}

/// Run `git merge` with args in `project_root`.
pub fn git_merge(
    _cap: &crate::lifecycle_commit::LifecycleCommitCapability,
    project_root: &Path,
    args: &[&str],
) -> Result<()> {
    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let output = std::process::Command::new(git_bin)
        .arg("-C")
        .arg(project_root)
        .arg("merge")
        .args(args)
        .output()
        .context("git merge failed")?;
    if !output.status.success() {
        return Err(format_git_error("git merge", output.status, &output.stderr));
    }
    Ok(())
}

/// Run `git diff --name-only` with args in `project_root`.
pub fn git_diff_name_only(
    _cap: &crate::lifecycle_commit::LifecycleHistoryCapability,
    project_root: &Path,
    args: &[&str],
) -> Result<Vec<String>> {
    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let output = std::process::Command::new(git_bin)
        .arg("-C")
        .arg(project_root)
        .arg("diff")
        .arg("--name-only")
        .args(args)
        .output()
        .context("git diff --name-only failed")?;
    if !output.status.success() {
        return Err(format_git_error(
            "git diff --name-only",
            output.status,
            &output.stderr,
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

/// Run `git status` with args in `project_root` and return raw stdout bytes.
pub fn git_status<P: AsRef<Path>>(
    project_root: &Path,
    literal_pathspecs: bool,
    args: &[&str],
    pathspecs: &[P],
) -> Result<Vec<u8>> {
    let git_bin = find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    let mut command = std::process::Command::new(git_bin);
    if literal_pathspecs {
        command.arg("--literal-pathspecs");
    }
    command.arg("-C").arg(project_root).arg("status").args(args);
    if !pathspecs.is_empty() {
        command.arg("--");
        for path in pathspecs {
            command.arg(path.as_ref());
        }
    }
    let output = command.output().context("git status failed")?;
    if !output.status.success() {
        return Err(format_git_error(
            "git status",
            output.status,
            &output.stderr,
        ));
    }
    Ok(output.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    pub(super) fn init_git_repo(root: &Path) -> bool {
        let Some(git) = find_git_bin() else {
            return false;
        };
        std::process::Command::new(git)
            .args(["init", "-q"])
            .current_dir(root)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[test]
    fn find_bin_on_path_scans_path_without_shelling_out_to_which() {
        let dir = TempDir::new().unwrap();
        let git_path = dir.path().join("git");
        std::fs::write(&git_path, "").unwrap();
        let path_env = std::env::join_paths([dir.path()]).unwrap();

        assert_eq!(find_bin_on_path("git", Some(path_env)), Some(git_path));
    }

    #[test]
    fn staged_guard_allows_unstaged_work_and_names_staged_paths() {
        let dir = TempDir::new().unwrap();
        if !init_git_repo(dir.path()) {
            return;
        }
        let review = dir.path().join("ec-review.json");
        std::fs::write(&review, "{}\n").unwrap();
        ensure_no_staged_changes(dir.path()).unwrap();

        let git = find_git_bin().unwrap();
        let staged = std::process::Command::new(git)
            .args(["add", "ec-review.json"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert!(staged.status.success());
        let error = ensure_no_staged_changes(dir.path())
            .unwrap_err()
            .to_string();
        assert!(error.contains("ec-review.json"), "{error}");
        assert!(error.contains("git restore --staged"), "{error}");
    }
}

#[cfg(test)]
mod lifecycle_commit_boundary {
    use super::*;
    use tempfile::TempDir;

    fn count_line_braces(line: &str) -> (usize, usize) {
        let mut opens = 0;
        let mut closes = 0;
        let mut in_str = false;
        let mut escaped = false;
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if in_str {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_str = false;
                }
            } else {
                if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
                    break;
                }
                if c == '"' {
                    in_str = true;
                } else if c == '{' {
                    opens += 1;
                } else if c == '}' {
                    closes += 1;
                }
            }
            i += 1;
        }
        (opens, closes)
    }

    fn strip_line_comment(line: &str) -> &str {
        let mut in_str = false;
        let mut escaped = false;
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if in_str {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_str = false;
                }
            } else {
                if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
                    return &line[..i];
                }
                if c == '"' {
                    in_str = true;
                }
            }
            i += 1;
        }
        line
    }

    fn is_git_command(line_idx: usize, lines: &[&str]) -> bool {
        let line = lines[line_idx];
        let end_idx = (line_idx + 5).min(lines.len());
        let block = lines[line_idx..end_idx].join("\n");
        let cmd_start = match block.find("Command::new") {
            Some(idx) => idx + "Command::new".len(),
            None => match block.find("Command :: new") {
                Some(idx) => idx + "Command :: new".len(),
                None => return false,
            },
        };
        let after_cmd = &block[cmd_start..];
        let arg_content = match after_cmd.find('(') {
            Some(open) => {
                let inner = &after_cmd[open + 1..];
                match inner.find(')') {
                    Some(close) => inner[..close].trim(),
                    None => inner.trim(),
                }
            }
            None => "",
        };

        let cleaned_arg = arg_content
            .trim_start_matches('&')
            .trim_start_matches("mut ")
            .trim();

        if cleaned_arg.to_lowercase().contains("git") {
            return true;
        }

        if arg_content.contains("find_git_bin") {
            return true;
        }

        let binding = cleaned_arg.split('.').next().unwrap_or(cleaned_arg).trim();

        if !binding.is_empty() && binding.chars().all(|c| c.is_alphanumeric() || c == '_') {
            let start_back = if line_idx > 50 { line_idx - 50 } else { 0 };
            for prev_idx in (start_back..line_idx).rev() {
                let prev_line = lines[prev_idx];
                if prev_line.contains(binding) && prev_line.contains("find_git_bin") {
                    return true;
                }
                if prev_line.trim_start().starts_with("fn ")
                    || prev_line.trim_start().starts_with("pub fn")
                    || prev_line.trim_start().starts_with("pub(crate) fn")
                {
                    break;
                }
            }
        }

        false
    }

    fn scan_source_for_git_add_or_commit(rel_file: &str, content: &str) -> Vec<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut violations = Vec::new();
        let mut test_depth = 0;
        let mut pending_cfg_test = false;

        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = line_idx + 1;
            let code_line = strip_line_comment(line);
            let trimmed_code = code_line.trim();

            if trimmed_code.contains("#[cfg(test)]") || trimmed_code.contains("#[cfg(any(test,") {
                pending_cfg_test = true;
            }

            let (opens, closes) = count_line_braces(line);

            if pending_cfg_test && opens > 0 {
                test_depth += opens;
                pending_cfg_test = false;
            } else if test_depth > 0 {
                test_depth += opens;
            }

            let is_in_test = test_depth > 0 || pending_cfg_test;

            if test_depth > 0 {
                if closes >= test_depth {
                    test_depth = 0;
                } else {
                    test_depth -= closes;
                }
            }

            if is_in_test {
                continue;
            }

            if line.contains("Command::new") || line.contains("Command :: new") {
                if is_git_command(line_idx, &lines) {
                    violations.push(format!(
                        "{}:{}: Direct Git process spawn found: {}",
                        rel_file,
                        line_num,
                        line.trim()
                    ));
                }
            }
        }

        violations
    }

    #[test]
    fn no_direct_git_add_or_commit_in_lifecycle_modules() {
        let lifecycle_files = [
            "src/cli/issues.rs",
            "src/cli/ec.rs",
            "src/cli/td.rs",
            "src/cli/cb.rs",
            "src/cli/cb_fill.rs",
            "src/cli/td_lock.rs",
            "src/cli/change_lifecycle.rs",
        ];

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| "apps/agentic-workflow".to_string());
        let mut violations = Vec::new();

        for rel_file in lifecycle_files {
            let full_path = std::path::Path::new(&manifest_dir).join(rel_file);
            let content = match std::fs::read_to_string(&full_path) {
                Ok(c) => c,
                Err(e) => {
                    violations.push(format!("{}: failed to read: {}", rel_file, e));
                    continue;
                }
            };

            violations.extend(scan_source_for_git_add_or_commit(rel_file, &content));
        }

        assert!(
            violations.is_empty(),
            "Found direct git calls in lifecycle modules:\n{}",
            violations.join("\n")
        );
    }

    #[test]
    fn scan_reports_git_commit_in_production_fn_following_cfg_test_doc_comment() {
        let fixture = r#"
/// This was previously `#[cfg(test)]`-only while production carried its own
pub fn prod_fn() {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("commit");
}

#[cfg(test)]
fn test_fn() {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("commit");
}
"#;

        let violations = scan_source_for_git_add_or_commit("fixture.rs", fixture);
        assert_eq!(
            violations.len(),
            1,
            "expected exactly 1 violation for production function, got: {violations:?}"
        );
        assert!(
            violations[0].contains("fixture.rs:4:"),
            "expected violation at line 4, got: {}",
            violations[0]
        );
    }

    #[test]
    fn scan_reports_git_log_in_production_fn() {
        let fixture = r#"
pub fn prod_fn() {
    let output = std::process::Command::new("git")
        .args(["log", "--format=%B"])
        .output();
}

#[cfg(test)]
fn test_fn() {
    let output = std::process::Command::new("git")
        .args(["log", "--format=%B"])
        .output();
}
"#;
        let violations =
            scan_source_for_git_add_or_commit("apps/agentic-workflow/src/cli/cb.rs", fixture);
        assert_eq!(
            violations.len(),
            1,
            "expected 1 violation, got: {violations:?}"
        );
        assert!(
            violations[0].contains("apps/agentic-workflow/src/cli/cb.rs:3:"),
            "got: {}",
            violations[0]
        );
    }

    #[test]
    fn scan_reports_git_rev_parse_in_production_fn() {
        let fixture = r#"
pub fn prod_fn() {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output();
}
"#;
        let violations = scan_source_for_git_add_or_commit("src/cli/cb.rs", fixture);
        assert_eq!(
            violations.len(),
            1,
            "expected 1 violation, got: {violations:?}"
        );
        assert!(
            violations[0].contains("src/cli/cb.rs:3:"),
            "got: {}",
            violations[0]
        );
    }

    #[test]
    fn scan_reports_git_rev_list_in_production_fn() {
        let fixture = r#"
pub fn prod_fn() {
    let output = std::process::Command::new("git")
        .args(["rev-list", "HEAD"])
        .output();
}
"#;
        let violations = scan_source_for_git_add_or_commit("src/cli/cb.rs", fixture);
        assert_eq!(
            violations.len(),
            1,
            "expected 1 violation, got: {violations:?}"
        );
        assert!(
            violations[0].contains("src/cli/cb.rs:3:"),
            "got: {}",
            violations[0]
        );
    }

    #[test]
    fn scan_reports_git_add_in_production_fn() {
        let fixture = r#"
pub fn prod_fn() {
    let output = std::process::Command::new("git")
        .args(["add", "sample.txt"])
        .output();
}
"#;
        let violations = scan_source_for_git_add_or_commit("src/cli/cb.rs", fixture);
        assert_eq!(
            violations.len(),
            1,
            "expected 1 violation, got: {violations:?}"
        );
        assert!(
            violations[0].contains("src/cli/cb.rs:3:"),
            "got: {}",
            violations[0]
        );
    }

    #[test]
    fn scan_reports_git_merge_base_in_production_fn() {
        let fixture = r#"
pub fn prod_fn() {
    let output = std::process::Command::new("git")
        .args(["merge-base", "--is-ancestor", "feature", "HEAD"])
        .output();
}
"#;
        let violations = scan_source_for_git_add_or_commit("src/cli/cb.rs", fixture);
        assert_eq!(
            violations.len(),
            1,
            "expected 1 violation, got: {violations:?}"
        );
        assert!(
            violations[0].contains("src/cli/cb.rs:3:"),
            "got: {}",
            violations[0]
        );
    }

    #[test]
    fn scan_reports_git_cat_file_in_production_fn() {
        let fixture = r#"
pub fn prod_fn() {
    let output = std::process::Command::new("git")
        .args(["cat-file", "blob", "HEAD:sample.txt"])
        .output();
}
"#;
        let violations = scan_source_for_git_add_or_commit("src/cli/cb.rs", fixture);
        assert_eq!(
            violations.len(),
            1,
            "expected 1 violation, got: {violations:?}"
        );
        assert!(
            violations[0].contains("src/cli/cb.rs:3:"),
            "got: {}",
            violations[0]
        );
    }

    #[test]
    fn scan_reports_git_show_in_production_fn() {
        let fixture = r#"
pub fn prod_fn() {
    let output = std::process::Command::new("git")
        .args(["show", "HEAD:sample.txt"])
        .output();
}
"#;
        let violations = scan_source_for_git_add_or_commit("src/cli/cb.rs", fixture);
        assert_eq!(
            violations.len(),
            1,
            "expected 1 violation, got: {violations:?}"
        );
        assert!(
            violations[0].contains("src/cli/cb.rs:3:"),
            "got: {}",
            violations[0]
        );
    }

    #[test]
    fn test_lifecycle_commit_boundary_primitives() {
        let dir = TempDir::new().unwrap();
        let repo = dir.path();

        let git = find_git_bin().unwrap();
        let init = std::process::Command::new(&git)
            .args(["init", "-q"])
            .current_dir(repo)
            .output()
            .unwrap();
        assert!(init.status.success());

        let file1 = repo.join("test.txt");
        std::fs::write(&file1, "hello\n").unwrap();

        let cap = crate::lifecycle_commit::LifecycleCommitCapability::for_test();
        stage_paths(&cap, repo, &[Path::new("test.txt")], false).unwrap();
        assert!(has_staged_changes(repo).unwrap());

        commit_staged(&cap, repo, "commit test", false).unwrap();
        assert!(!has_staged_changes(repo).unwrap());

        commit_staged(&cap, repo, "empty test", true).unwrap();

        let file2 = repo.join("scoped.txt");
        std::fs::write(&file2, "world\n").unwrap();
        stage_paths(&cap, repo, &[Path::new("scoped.txt")], true).unwrap();
        assert!(has_staged_changes_for_paths(repo, &[Path::new("scoped.txt")], true).unwrap());

        commit_only_paths(&cap, repo, &[Path::new("scoped.txt")], "scoped test", true).unwrap();
        assert!(!has_staged_changes_for_paths(repo, &[Path::new("scoped.txt")], true).unwrap());
    }

    #[test]
    fn test_has_staged_changes_exit_code_classification() {
        let dir = TempDir::new().unwrap();
        let non_repo = dir.path();

        let err1 = has_staged_changes(non_repo).unwrap_err().to_string();
        assert!(
            err1.contains("git diff --cached failed with exit code"),
            "expected non-0/1 exit code error text, got: {err1}"
        );

        let err2 = has_staged_changes_for_paths(non_repo, &[Path::new("test.txt")], true)
            .unwrap_err()
            .to_string();
        assert!(
            err2.contains("git diff --cached failed with exit code"),
            "expected non-0/1 exit code error text, got: {err2}"
        );
    }

    #[test]
    fn test_history_primitives_fidelity_and_absence() {
        let dir = TempDir::new().unwrap();
        let repo = dir.path();
        let git_bin = find_git_bin().unwrap();

        let init = std::process::Command::new(&git_bin)
            .args(["init", "-q"])
            .current_dir(repo)
            .output()
            .unwrap();
        assert!(init.status.success());

        // Configure git identity for commits
        let _ = std::process::Command::new(&git_bin)
            .args(["config", "user.name", "Test"])
            .current_dir(repo)
            .output();
        let _ = std::process::Command::new(&git_bin)
            .args(["config", "user.email", "test@example.com"])
            .current_dir(repo)
            .output();

        let f = repo.join("sample.txt");
        std::fs::write(&f, "content v1\n").unwrap();

        let add = std::process::Command::new(&git_bin)
            .args(["add", "sample.txt"])
            .current_dir(repo)
            .output()
            .unwrap();
        assert!(add.status.success());

        let commit1 = std::process::Command::new(&git_bin)
            .args(["commit", "-m", "initial commit"])
            .current_dir(repo)
            .output()
            .unwrap();
        assert!(commit1.status.success());

        let branch_cmd = std::process::Command::new(&git_bin)
            .args(["branch", "feature"])
            .current_dir(repo)
            .output()
            .unwrap();
        assert!(branch_cmd.status.success());

        std::fs::write(&f, "content v2\n").unwrap();
        let add2 = std::process::Command::new(&git_bin)
            .args(["add", "sample.txt"])
            .current_dir(repo)
            .output()
            .unwrap();
        assert!(add2.status.success());
        let commit2 = std::process::Command::new(&git_bin)
            .args(["commit", "-m", "second commit"])
            .current_dir(repo)
            .output()
            .unwrap();
        assert!(commit2.status.success());

        std::fs::write(&f, "content v3\n").unwrap();
        let add3 = std::process::Command::new(&git_bin)
            .args(["add", "sample.txt"])
            .current_dir(repo)
            .output()
            .unwrap();
        assert!(add3.status.success());
        let commit3 = std::process::Command::new(&git_bin)
            .args(["commit", "-m", "third commit"])
            .current_dir(repo)
            .output()
            .unwrap();
        assert!(commit3.status.success());

        let hcap = crate::lifecycle_commit::LifecycleHistoryCapability::for_test();

        // 1. git_log - present & absent
        let log_p = git_log(&hcap, repo, &["-1", "--format=%B"]).unwrap();
        let indep_log_p = std::process::Command::new(&git_bin)
            .args(["log", "-1", "--format=%B"])
            .current_dir(repo)
            .output()
            .unwrap();
        assert_eq!(
            log_p.trim(),
            String::from_utf8_lossy(&indep_log_p.stdout).trim()
        );

        let log_a = git_log(&hcap, repo, &["--grep", "nonexistent_grep_string_999"]).unwrap();
        let indep_log_a = std::process::Command::new(&git_bin)
            .args(["log", "--grep", "nonexistent_grep_string_999"])
            .current_dir(repo)
            .output()
            .unwrap();
        assert_eq!(
            log_a.trim(),
            String::from_utf8_lossy(&indep_log_a.stdout).trim()
        );

        // 2. git_rev_parse - present & absent
        let rev_p = git_rev_parse(&hcap, repo, &["HEAD"]).unwrap();
        let indep_rev_p = std::process::Command::new(&git_bin)
            .args(["rev-parse", "HEAD"])
            .current_dir(repo)
            .output()
            .unwrap();
        assert_eq!(rev_p, String::from_utf8_lossy(&indep_rev_p.stdout).trim());

        assert!(git_rev_parse(&hcap, repo, &["--verify", "-q", "refs/heads/nonexistent"]).is_err());

        // 3. git_rev_list - present & absent
        let list_p = git_rev_list(&hcap, repo, &["HEAD"]).unwrap();
        let indep_list_p = std::process::Command::new(&git_bin)
            .args(["rev-list", "HEAD"])
            .current_dir(repo)
            .output()
            .unwrap();
        let expected_list: Vec<String> = String::from_utf8_lossy(&indep_list_p.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        assert_eq!(
            expected_list.len(),
            3,
            "expected 3 commits in independent rev-list output"
        );
        assert_eq!(
            list_p.len(),
            3,
            "expected 3 commits in git_rev_list output, got {}",
            list_p.len()
        );
        assert_eq!(list_p, expected_list);

        let c3_sha = std::process::Command::new(&git_bin)
            .args(["rev-parse", "HEAD"])
            .current_dir(repo)
            .output()
            .unwrap();
        let c3_str = String::from_utf8_lossy(&c3_sha.stdout).trim().to_string();

        let c2_sha = std::process::Command::new(&git_bin)
            .args(["rev-parse", "HEAD~1"])
            .current_dir(repo)
            .output()
            .unwrap();
        let c2_str = String::from_utf8_lossy(&c2_sha.stdout).trim().to_string();

        let c1_sha = std::process::Command::new(&git_bin)
            .args(["rev-parse", "HEAD~2"])
            .current_dir(repo)
            .output()
            .unwrap();
        let c1_str = String::from_utf8_lossy(&c1_sha.stdout).trim().to_string();

        assert_eq!(list_p, vec![c3_str, c2_str, c1_str]);

        assert!(git_rev_list(&hcap, repo, &["nonexistent_ref_xyz"]).is_err());

        // 4. git_merge_base_is_ancestor - present & absent
        // feature is at initial commit, HEAD is at third commit
        // feature is ancestor of HEAD
        let is_anc = git_merge_base_is_ancestor(repo, "feature", "HEAD").unwrap();
        let indep_anc = std::process::Command::new(&git_bin)
            .args(["merge-base", "--is-ancestor", "feature", "HEAD"])
            .current_dir(repo)
            .status()
            .unwrap()
            .success();
        assert_eq!(is_anc, indep_anc);
        assert!(is_anc);

        // HEAD is NOT ancestor of feature
        let not_anc = git_merge_base_is_ancestor(repo, "HEAD", "feature").unwrap();
        let indep_not_anc = std::process::Command::new(&git_bin)
            .args(["merge-base", "--is-ancestor", "HEAD", "feature"])
            .current_dir(repo)
            .status()
            .unwrap()
            .success();
        assert_eq!(not_anc, indep_not_anc);
        assert!(!not_anc);

        // 5. git_cat_file_blob - present & absent
        let cat_p = git_cat_file_blob(&hcap, repo, "HEAD:sample.txt").unwrap();
        let indep_cat_p = std::process::Command::new(&git_bin)
            .args(["cat-file", "blob", "HEAD:sample.txt"])
            .current_dir(repo)
            .output()
            .unwrap();
        assert_eq!(cat_p, indep_cat_p.stdout);
        assert_eq!(String::from_utf8_lossy(&cat_p), "content v3\n");

        assert!(git_cat_file_blob(&hcap, repo, "HEAD:nonexistent.txt").is_err());

        // 6. git_show - present & absent
        let show_p = git_show(&hcap, repo, &["HEAD:sample.txt"]).unwrap();
        let indep_show_p = std::process::Command::new(&git_bin)
            .args(["show", "HEAD:sample.txt"])
            .current_dir(repo)
            .output()
            .unwrap();
        assert_eq!(show_p, String::from_utf8_lossy(&indep_show_p.stdout));

        assert!(git_show(&hcap, repo, &["HEAD:nonexistent.txt"]).is_err());
    }

    #[test]
    fn test_history_primitives_non_repo_failure() {
        let dir = TempDir::new().unwrap();
        let non_repo = dir.path();
        let hcap = crate::lifecycle_commit::LifecycleHistoryCapability::for_test();

        let err_log = git_log(&hcap, non_repo, &["HEAD"]).unwrap_err().to_string();
        assert!(
            err_log.contains("exit code 128"),
            "expected exit code 128 error, got: {err_log}"
        );

        let err_rp = git_rev_parse(&hcap, non_repo, &["HEAD"])
            .unwrap_err()
            .to_string();
        assert!(
            err_rp.contains("exit code 128"),
            "expected exit code 128 error, got: {err_rp}"
        );

        let err_rl = git_rev_list(&hcap, non_repo, &["HEAD"])
            .unwrap_err()
            .to_string();
        assert!(
            err_rl.contains("exit code 128"),
            "expected exit code 128 error, got: {err_rl}"
        );

        let err_mb = git_merge_base_is_ancestor(non_repo, "HEAD", "HEAD")
            .unwrap_err()
            .to_string();
        assert!(
            err_mb.contains("exit code 128"),
            "expected exit code 128 error, got: {err_mb}"
        );

        let err_cf = git_cat_file_blob(&hcap, non_repo, "HEAD:file")
            .unwrap_err()
            .to_string();
        assert!(
            err_cf.contains("exit code 128"),
            "expected exit code 128 error, got: {err_cf}"
        );

        let err_sh = git_show(&hcap, non_repo, &["HEAD"])
            .unwrap_err()
            .to_string();
        assert!(
            err_sh.contains("exit code 128"),
            "expected exit code 128 error, got: {err_sh}"
        );
    }

    #[test]
    fn measurement_1_cb_status_and_td_diff_reported() {
        let fixture_cb = r#"
pub fn prod_cb() {
    let git = crate::git::find_git_bin().unwrap();
    let _ = std::process::Command::new(git).args(["status"]).output();
}
"#;
        let v_cb = scan_source_for_git_add_or_commit("src/cli/cb.rs", fixture_cb);
        assert_eq!(v_cb.len(), 1, "expected 1 violation for status in cb.rs");
        assert!(v_cb[0].contains("src/cli/cb.rs:4:"), "got: {}", v_cb[0]);

        let fixture_td = r#"
pub fn prod_td() {
    let git = crate::git::find_git_bin().unwrap();
    let _ = std::process::Command::new(git).args(["diff"]).output();
}
"#;
        let v_td = scan_source_for_git_add_or_commit("src/cli/td.rs", fixture_td);
        assert_eq!(v_td.len(), 1, "expected 1 violation for diff in td.rs");
        assert!(v_td[0].contains("src/cli/td.rs:4:"), "got: {}", v_td[0]);
    }

    #[test]
    fn measurement_2_git_spawn_with_reset_reported() {
        let fixture = r#"
pub fn prod_fn() {
    let git = crate::git::find_git_bin().unwrap();
    let _ = std::process::Command::new(git).args(["reset", "HEAD~1"]).output();
}
"#;
        let v = scan_source_for_git_add_or_commit("src/cli/cb.rs", fixture);
        assert_eq!(v.len(), 1, "expected 1 violation for reset");
        assert!(v[0].contains("src/cli/cb.rs:4:"), "got: {}", v[0]);
    }

    #[test]
    fn measurement_3_git_spawn_with_merge_reported() {
        let fixture = r#"
pub fn prod_fn() {
    let git = crate::git::find_git_bin().unwrap();
    let _ = std::process::Command::new(git).args(["merge", "--no-ff", "branch"]).output();
}
"#;
        let v = scan_source_for_git_add_or_commit("src/cli/cb.rs", fixture);
        assert_eq!(v.len(), 1, "expected 1 violation for merge");
        assert!(v[0].contains("src/cli/cb.rs:4:"), "got: {}", v[0]);
    }

    #[test]
    fn measurement_4_git_spawn_with_stash_reported() {
        let fixture = r#"
pub fn prod_fn() {
    let git = crate::git::find_git_bin().unwrap();
    let _ = std::process::Command::new(git).args(["stash"]).output();
}
"#;
        let v = scan_source_for_git_add_or_commit("src/cli/cb.rs", fixture);
        assert_eq!(v.len(), 1, "expected 1 violation for stash");
        assert!(v[0].contains("src/cli/cb.rs:4:"), "got: {}", v[0]);
    }

    #[test]
    fn measurement_5_git_spawn_with_non_git_binding_name_and_no_subcommand_literal_reported() {
        let fixture = r#"
pub fn prod_fn() {
    let bin = crate::git::find_git_bin().unwrap();
    let _ = std::process::Command::new(bin).output();
}
"#;
        let v = scan_source_for_git_add_or_commit("src/cli/cb.rs", fixture);
        assert_eq!(v.len(), 1, "expected 1 violation for non-git binding name");
        assert!(v[0].contains("src/cli/cb.rs:4:"), "got: {}", v[0]);
    }

    #[test]
    fn measurement_6_non_git_process_spawns_not_reported() {
        let fixture = r#"
pub fn prod_fn() {
    let _ = std::process::Command::new("sh").arg("-c").arg("echo hi").output();
    let rustfmt = crate::git::find_rustfmt_bin().unwrap();
    let _ = std::process::Command::new(&rustfmt).arg("--edition").output();
    let tool_path = crate::git::find_cargo_bin().unwrap();
    let _ = std::process::Command::new(&tool_path).arg("check").output();
}
"#;
        let v = scan_source_for_git_add_or_commit("src/cli/cb.rs", fixture);
        assert!(
            v.is_empty(),
            "expected 0 violations for non-git spawns, got: {v:?}"
        );
    }

    #[test]
    fn measurement_7_test_module_git_reset_not_reported() {
        let fixture = r#"
#[cfg(test)]
mod tests {
    fn test_reset() {
        let git = crate::git::find_git_bin().unwrap();
        let _ = std::process::Command::new(git).args(["reset", "HEAD~1"]).output();
    }
}
"#;
        let v = scan_source_for_git_add_or_commit("src/cli/cb.rs", fixture);
        assert!(
            v.is_empty(),
            "expected 0 violations in test module, got: {v:?}"
        );
    }

    #[test]
    fn measurement_8_status_porcelain_z_preserves_non_utf8_path_bytes() {
        let temp = TempDir::new().unwrap();
        if !super::tests::init_git_repo(temp.path()) {
            return;
        }
        let git_bin = find_git_bin().unwrap();

        let mut hash_child = std::process::Command::new(&git_bin)
            .arg("-C")
            .arg(temp.path())
            .args(["hash-object", "-w", "--stdin"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        {
            use std::io::Write;
            let stdin = hash_child.stdin.as_mut().unwrap();
            stdin.write_all(b"test content").unwrap();
        }
        let hash_out = hash_child.wait_with_output().unwrap();
        let blob_sha = String::from_utf8_lossy(&hash_out.stdout).trim().to_string();

        #[cfg(unix)]
        use std::os::unix::ffi::OsStrExt;

        #[cfg(unix)]
        let non_utf8_name = std::ffi::OsStr::from_bytes(b"invalid_\xff_path.txt");
        #[cfg(not(unix))]
        let non_utf8_name = std::ffi::OsStr::new("invalid_path.txt");

        let mut update_cmd = std::process::Command::new(&git_bin);
        update_cmd
            .arg("-C")
            .arg(temp.path())
            .args(["update-index", "--add", "--cacheinfo", "100644", &blob_sha])
            .arg(non_utf8_name);
        let update_out = update_cmd.output().unwrap();
        assert!(
            update_out.status.success(),
            "git update-index failed: {}",
            String::from_utf8_lossy(&update_out.stderr)
        );

        let routed_bytes = git_status(
            temp.path(),
            false,
            &["--porcelain=v1", "-z", "--untracked-files=all"],
            &[] as &[&Path],
        )
        .unwrap();

        let direct_output = std::process::Command::new(&git_bin)
            .arg("-C")
            .arg(temp.path())
            .args(["status", "--porcelain=v1", "-z", "--untracked-files=all"])
            .output()
            .unwrap();

        assert_eq!(
            routed_bytes, direct_output.stdout,
            "routed status bytes must match direct Git stdout byte-for-byte"
        );
        #[cfg(unix)]
        assert!(
            routed_bytes.windows(14).any(|w| w == b"invalid_\xff_path"),
            "non-UTF-8 path bytes must be preserved in stdout"
        );
    }

    #[test]
    fn measurement_defect_1_git_status_pathspec_non_utf8() {
        let temp = TempDir::new().unwrap();
        if !super::tests::init_git_repo(temp.path()) {
            return;
        }
        let git_bin = find_git_bin().unwrap();

        let mut hash_child = std::process::Command::new(&git_bin)
            .arg("-C")
            .arg(temp.path())
            .args(["hash-object", "-w", "--stdin"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        {
            use std::io::Write;
            let stdin = hash_child.stdin.as_mut().unwrap();
            stdin.write_all(b"test content").unwrap();
        }
        let hash_out = hash_child.wait_with_output().unwrap();
        let blob_sha = String::from_utf8_lossy(&hash_out.stdout).trim().to_string();

        #[cfg(unix)]
        use std::os::unix::ffi::OsStrExt;

        #[cfg(unix)]
        let non_utf8_name = std::ffi::OsStr::from_bytes(b"invalid_\xff_path.txt");
        #[cfg(not(unix))]
        let non_utf8_name = std::ffi::OsStr::new("invalid_path.txt");

        let non_utf8_path = std::path::Path::new(non_utf8_name);

        let mut update_cmd = std::process::Command::new(&git_bin);
        update_cmd
            .arg("-C")
            .arg(temp.path())
            .args(["update-index", "--add", "--cacheinfo", "100644", &blob_sha])
            .arg(non_utf8_name);
        let update_out = update_cmd.output().unwrap();
        assert!(
            update_out.status.success(),
            "git update-index failed: {}",
            String::from_utf8_lossy(&update_out.stderr)
        );

        let routed_bytes = git_status(
            temp.path(),
            true,
            &["--porcelain=v1", "-z", "--untracked-files=all"],
            &[non_utf8_path],
        )
        .unwrap();

        let direct_output = std::process::Command::new(&git_bin)
            .arg("--literal-pathspecs")
            .arg("-C")
            .arg(temp.path())
            .args([
                "status",
                "--porcelain=v1",
                "-z",
                "--untracked-files=all",
                "--",
            ])
            .arg(non_utf8_name)
            .output()
            .unwrap();

        assert_eq!(
            routed_bytes, direct_output.stdout,
            "routed status bytes must match direct Git stdout byte-for-byte"
        );
        assert!(
            !routed_bytes.is_empty(),
            "routed status stdout must not be empty"
        );
        assert!(
            !direct_output.stdout.is_empty(),
            "direct git status stdout must not be empty"
        );
    }

    #[test]
    fn measurement_9_routed_call_sites_on_git_failure() {
        let non_repo = TempDir::new().unwrap();

        assert!(
            crate::cli::td::checkout_has_only_exact_untracked_path(non_repo.path(), "file.txt")
                .is_err(),
            "td.rs:544 must return Err on git failure"
        );

        assert!(
            crate::cli::cb::committed_paths_since_td_init(non_repo.path(), "slug").is_err(),
            "cb.rs:7405 must return Err on git failure"
        );

        assert!(
            crate::cli::cb::committed_paths_after_td_python_source(non_repo.path(), "slug")
                .is_err(),
            "cb.rs:7445 must return Err on git failure"
        );

        assert!(
            git_status(
                non_repo.path(),
                true,
                &["--porcelain=v1", "-z", "--untracked-files=all",],
                &[Path::new("lock")],
            )
            .is_err(),
            "td_lock.rs:376 primitive call must return Err on git failure"
        );

        assert!(
            crate::cli::td::discover_worktree_spec(non_repo.path()).is_none(),
            "td.rs:847 must return None on git failure"
        );

        assert!(
            crate::cli::cb::dirty_touched_scope_gate_message(
                non_repo.path(),
                "slug",
                &["path.rs".to_string()]
            )
            .is_none(),
            "cb.rs:6870 must return None on git failure"
        );

        assert!(
            crate::cli::cb_fill::branch_changed_files(non_repo.path(), "main").is_empty(),
            "cb_fill.rs:1363 must return empty set on git failure"
        );
    }
}

// CODEGEN-END
