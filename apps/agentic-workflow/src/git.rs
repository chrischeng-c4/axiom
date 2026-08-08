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
pub fn commit_scoped_paths(project_root: &Path, paths: &[PathBuf], message: &str) -> Result<bool> {
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
pub fn commit_staged(project_root: &Path, message: &str, allow_empty: bool) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn init_git_repo(root: &Path) -> bool {
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
                let end_idx = (line_idx + 15).min(lines.len());
                let block = lines[line_idx..end_idx].join("\n");
                let has_git_add = block.contains("\"add\"") || block.contains("'add'");
                let has_git_commit = block.contains("\"commit\"") || block.contains("'commit'");
                if has_git_add || has_git_commit {
                    violations.push(format!(
                        "{}:{}: Direct Git process spawn with add/commit found: {}",
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
            "Found direct git add/commit calls in lifecycle modules:\n{}",
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

        stage_paths(repo, &[Path::new("test.txt")], false).unwrap();
        assert!(has_staged_changes(repo).unwrap());

        commit_staged(repo, "commit test", false).unwrap();
        assert!(!has_staged_changes(repo).unwrap());

        commit_staged(repo, "empty test", true).unwrap();

        let file2 = repo.join("scoped.txt");
        std::fs::write(&file2, "world\n").unwrap();
        stage_paths(repo, &[Path::new("scoped.txt")], true).unwrap();
        assert!(has_staged_changes_for_paths(repo, &[Path::new("scoped.txt")], true).unwrap());

        commit_only_paths(repo, &[Path::new("scoped.txt")], "scoped test", true).unwrap();
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
}

// CODEGEN-END
