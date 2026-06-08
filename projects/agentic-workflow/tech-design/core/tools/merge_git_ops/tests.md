---
id: sdd-tools-merge-git-ops-tests
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools merge git ops tests

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/merge_git_ops.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `GitOpsResult` | projects/agentic-workflow/src/tools/merge_git_ops.rs | struct | pub | 29 |  |
| `find_git_binary` | projects/agentic-workflow/src/tools/merge_git_ops.rs | function | pub | 442 | find_git_binary() -> Option<PathBuf> |
| `post_archive_git_ops` | projects/agentic-workflow/src/tools/merge_git_ops.rs | function | pub | 81 | post_archive_git_ops(     project_root: &Path,     change_id: &str,     archive_path: &Path,     repo_platform: Option<&RepoPlatformConfig>,     merged_specs: &[Value], ) -> Result<GitOpsResult> |
| `resolve_worktree_dir` | projects/agentic-workflow/src/tools/merge_git_ops.rs | function | pub | 278 | resolve_worktree_dir(project_root: &Path, change_id: &str) -> Option<PathBuf> |
## Source
<!-- type: source lang: rust -->

````rust
// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn test_build_commit_message_with_summary() {
        let msg = build_commit_message("1136", Some("feat(sdd): platform config restructure"));
        assert_eq!(
            msg,
            "chore(sdd): merge 1136 — feat(sdd): platform config restructure"
        );
    }

    #[test]
    fn test_build_commit_message_without_summary() {
        let msg = build_commit_message("42", None);
        assert_eq!(msg, "chore(sdd): merge 42");
    }

    #[test]
    fn test_build_commit_message_truncates_at_72_chars() {
        let long_summary = "a".repeat(80);
        let msg = build_commit_message("100", Some(&long_summary));
        let expected = format!("chore(sdd): merge 100 — {}", "a".repeat(72));
        assert_eq!(msg, expected);
    }

    #[test]
    fn test_read_user_input_summary_with_content() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("user_input.md"),
            "My change description\nMore detail",
        )
        .unwrap();
        let result = read_user_input_summary(tmp.path());
        assert_eq!(result, Some("My change description".to_string()));
    }

    #[test]
    fn test_read_user_input_summary_missing_file() {
        let tmp = TempDir::new().unwrap();
        let result = read_user_input_summary(tmp.path());
        assert_eq!(result, None);
    }

    #[test]
    fn test_read_user_input_summary_empty_file() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("user_input.md"), "").unwrap();
        let result = read_user_input_summary(tmp.path());
        assert_eq!(result, None);
    }

    #[test]
    fn test_read_user_input_summary_blank_lines_then_content() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("user_input.md"), "\n\n  \nActual content\n").unwrap();
        let result = read_user_input_summary(tmp.path());
        assert_eq!(result, Some("Actual content".to_string()));
    }

    #[test]
    fn test_extract_commit_sha_from_typical_output() {
        let output = "[main abc1234f] chore(sdd): merge 42";
        let sha = extract_commit_sha(output, Path::new("/nonexistent"), Path::new("/nonexistent"));
        assert!(sha.is_some());
        let sha_val = sha.unwrap();
        assert!(sha_val.len() >= 7);
        assert!(sha_val.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_extract_commit_sha_from_detached_head() {
        let output = "[detached HEAD a1b2c3d] chore(sdd): merge 99";
        let sha = extract_commit_sha(output, Path::new("/nonexistent"), Path::new("/nonexistent"));
        assert!(sha.is_some());
        assert_eq!(sha.unwrap(), "a1b2c3d");
    }

    #[test]
    fn test_build_pr_body_with_specs() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("user_input.md"), "Add new feature").unwrap();

        let specs = vec![
            json!({"spec_id": "auth-flow", "target": ".aw/tech-design/sdd/logic/auth-flow.md"}),
            json!({"spec_id": "config", "target": ".aw/tech-design/sdd/config/config.md"}),
        ];
        let body = build_pr_body("1136", tmp.path(), &specs);
        assert!(body.contains("SDD Merge: 1136"));
        assert!(body.contains("Add new feature"));
        assert!(body.contains(".aw/tech-design/sdd/logic/auth-flow.md"));
        assert!(body.contains(".aw/tech-design/sdd/config/config.md"));
        assert!(body.contains("Merged Specs"));
    }

    #[test]
    fn test_build_pr_body_no_specs_no_input() {
        let tmp = TempDir::new().unwrap();
        let body = build_pr_body("42", tmp.path(), &[]);
        assert!(body.contains("SDD Merge: 42"));
        assert!(!body.contains("Merged Specs"));
        assert!(!body.contains("Description"));
    }

    #[test]
    fn test_repo_platform_config_deserialization() {
        use crate::models::change::RepoPlatformConfig;

        let toml_content = r#"
type = "github"
repo = "owner/repo"
default_branch = "main"
auto_commit = true
auto_pr = false
"#;
        let rp: RepoPlatformConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(rp.type_, "github");
        assert_eq!(rp.repo, "owner/repo");
        assert_eq!(rp.default_branch, "main");
        assert!(rp.auto_commit);
        assert!(!rp.auto_pr);
    }

    #[test]
    fn test_repo_platform_config_absent_is_none() {
        use crate::models::SddConfig;

        let toml_content = "";
        let config: SddConfig = toml::from_str(toml_content).unwrap();
        assert!(
            config.repo_platform.is_none(),
            "absent repo_platform must be None"
        );
    }

    #[test]
    fn test_repo_platform_defaults() {
        use crate::models::change::RepoPlatformConfig;

        let toml_content = r#"
type = "github"
repo = "owner/repo"
"#;
        let rp: RepoPlatformConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(
            rp.default_branch, "main",
            "default_branch must default to 'main'"
        );
        assert!(!rp.auto_commit, "auto_commit must default to false");
        assert!(!rp.auto_pr, "auto_pr must default to false");
    }

    #[test]
    fn test_tech_design_platform_config_deserialization() {
        use crate::models::change::TechDesignPlatformConfig;

        let toml_content = r#"
type = "local"
path = ".aw/tech-design"
"#;
        let sp: TechDesignPlatformConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(sp.type_, "local");
        assert_eq!(sp.path, ".aw/tech-design");
    }

    #[test]
    fn test_sdd_config_load_with_repo_platform() {
        use crate::models::SddConfig;

        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join(".aw");
        std::fs::create_dir_all(&config_dir).unwrap();
        let toml_content = r#"
[agentic_workflow.repo_platform]
type = "github"
repo = "owner/repo"
auto_commit = true

[agentic_workflow.tech_design_platform]
type = "local"
path = ".aw/tech-design"
"#;
        std::fs::write(config_dir.join("config.toml"), toml_content).unwrap();

        let config = SddConfig::load(tmp.path()).unwrap();
        let rp = config
            .repo_platform
            .expect("repo_platform must be loaded from [agentic_workflow.repo_platform]");
        assert_eq!(rp.type_, "github");
        assert_eq!(rp.repo, "owner/repo");
        assert!(rp.auto_commit);

        let sp = config
            .tech_design_platform
            .expect("tech_design_platform must be loaded from [agentic_workflow.tech_design_platform]");
        assert_eq!(sp.type_, "local");
        assert_eq!(sp.path, ".aw/tech-design");
    }

    #[test]
    fn test_sdd_config_load_without_repo_platform() {
        use crate::models::SddConfig;

        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join(".aw");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(config_dir.join("config.toml"), "").unwrap();

        let config = SddConfig::load(tmp.path()).unwrap();
        assert!(
            config.repo_platform.is_none(),
            "absent [agentic_workflow.repo_platform] must leave None"
        );
        assert!(
            config.tech_design_platform.is_none(),
            "absent [agentic_workflow.tech_design_platform] must leave None"
        );
    }

    // ─── Worktree Merge Sequence Tests ───────────────────────────────────

    /// Initialize a bare git repo with an initial commit. Returns true if git
    /// is available and the repo was initialized.
    fn init_repo(dir: &Path) -> bool {
        let Some(git) = find_git_binary() else {
            return false;
        };
        let ok = std::process::Command::new(&git)
            .args(["init", "-b", "main"])
            .current_dir(dir)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !ok {
            return false;
        }
        for (k, v) in [("user.email", "test@example.com"), ("user.name", "Test")] {
            let _ = std::process::Command::new(&git)
                .args(["config", k, v])
                .current_dir(dir)
                .output();
        }
        std::fs::write(dir.join("README.md"), "init\n").unwrap();
        let _ = std::process::Command::new(&git)
            .args(["add", "README.md"])
            .current_dir(dir)
            .output();
        let _ = std::process::Command::new(&git)
            .args(["commit", "-m", "init"])
            .current_dir(dir)
            .output();
        true
    }

    // REQ: worktree-per-change — resolve_worktree_dir detects worktree presence
    #[test]
    fn test_resolve_worktree_dir_none_when_missing() {
        let tmp = TempDir::new().unwrap();
        assert!(resolve_worktree_dir(tmp.path(), "nothing-here").is_none());
    }

    // REQ: worktree-per-change — resolve_worktree_dir returns Some when dir exists
    #[test]
    fn test_resolve_worktree_dir_some_when_present() {
        let tmp = TempDir::new().unwrap();
        let wt = tmp.path().join(".aw/worktrees/my-change");
        std::fs::create_dir_all(&wt).unwrap();
        assert_eq!(resolve_worktree_dir(tmp.path(), "my-change"), Some(wt));
    }

    /// End-to-end test of the 5-step merge sequence: create worktree,
    /// make a commit on the branch, invoke post_archive_git_ops, verify
    /// the branch is merged into main and the worktree is cleaned up.
    // REQ: worktree-per-change — full 5-step sequence happy path
    #[test]
    fn test_post_archive_merges_worktree_and_cleans_up() {
        let tmp = TempDir::new().unwrap();
        if !init_repo(tmp.path()) {
            return;
        }

        let slug = "enhancement-full-merge";
        let Some(git) = find_git_binary() else { return };

        // Create the worktree and a commit on the new branch so there's
        // something for step 3 to merge.
        let wt_rel = format!(".aw/worktrees/{}", slug);
        std::fs::create_dir_all(tmp.path().join(".aw/worktrees")).unwrap();
        let add_out = std::process::Command::new(&git)
            .args(["worktree", "add", "-b", &format!("cclab/{}", slug), &wt_rel])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        assert!(
            add_out.status.success(),
            "git worktree add failed: {}",
            String::from_utf8_lossy(&add_out.stderr)
        );

        // Write a file under cclab/ in the worktree so the status check picks it up.
        let wt_abs = tmp.path().join(&wt_rel);
        std::fs::create_dir_all(wt_abs.join("cclab")).unwrap();
        std::fs::write(wt_abs.join("cclab/change-file.txt"), "worktree content\n").unwrap();

        // Prepare a stub archive path with user_input.md so commit msg builds.
        let archive_abs = tmp
            .path()
            .join(".aw/archive/20260101-enhancement-full-merge");
        std::fs::create_dir_all(&archive_abs).unwrap();
        std::fs::write(
            archive_abs.join("user_input.md"),
            "Add worktree isolation\n",
        )
        .unwrap();

        let config = RepoPlatformConfig {
            type_: "github".to_string(),
            repo: "test/repo".to_string(),
            host: None,
            default_branch: "main".to_string(),
            auto_commit: true,
            auto_pr: false,
        };

        let result = post_archive_git_ops(tmp.path(), slug, &archive_abs, Some(&config), &[]);
        assert!(
            result.is_ok(),
            "post_archive_git_ops failed: {:?}",
            result.err()
        );
        let ops = result.unwrap();
        assert!(ops.git_commit_sha.is_some(), "expected a commit SHA");

        // Verify the worktree directory is gone
        assert!(
            !wt_abs.exists(),
            "worktree directory should have been removed"
        );

        // Verify the branch was deleted
        let branch_check = std::process::Command::new(&git)
            .args([
                "show-ref",
                "--verify",
                "--quiet",
                &format!("refs/heads/cclab/{}", slug),
            ])
            .current_dir(tmp.path())
            .status()
            .unwrap();
        assert!(
            !branch_check.success(),
            "branch cclab/{} should have been deleted",
            slug
        );

        // Verify we're on main and the change file landed here
        let branch = std::process::Command::new(&git)
            .args(["branch", "--show-current"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&branch.stdout).trim(),
            "main",
            "should be on main branch after merge"
        );
        assert!(
            tmp.path().join("cclab/change-file.txt").exists(),
            "change file should be present on main after merge"
        );
    }

    /// Simulate a merge conflict: create a worktree, commit file X with content
    /// A on the branch, and commit the same file with content B on main.
    /// `post_archive_git_ops` must return an Err with a message mentioning
    /// conflict and manual resolution.
    // REQ: worktree-per-change — step 3 fails fast on conflict
    #[test]
    fn test_post_archive_fails_fast_on_merge_conflict() {
        let tmp = TempDir::new().unwrap();
        if !init_repo(tmp.path()) {
            return;
        }

        let slug = "enhancement-conflict";
        let Some(git) = find_git_binary() else { return };

        // Create a file on main first so both branches diverge from a common base
        std::fs::create_dir_all(tmp.path().join("cclab")).unwrap();
        std::fs::write(tmp.path().join("cclab/contested.txt"), "base\n").unwrap();
        std::process::Command::new(&git)
            .args(["add", "cclab/contested.txt"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        std::process::Command::new(&git)
            .args(["commit", "-m", "base"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        // Now create the worktree branch from current main
        let wt_rel = format!(".aw/worktrees/{}", slug);
        std::fs::create_dir_all(tmp.path().join(".aw/worktrees")).unwrap();
        std::process::Command::new(&git)
            .args(["worktree", "add", "-b", &format!("cclab/{}", slug), &wt_rel])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        // Modify contested.txt on main (uncommitted won't diverge — need a commit)
        std::fs::write(tmp.path().join("cclab/contested.txt"), "main version\n").unwrap();
        std::process::Command::new(&git)
            .args(["add", "cclab/contested.txt"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        std::process::Command::new(&git)
            .args(["commit", "-m", "main change"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        // Modify contested.txt inside the worktree (different content)
        let wt_abs = tmp.path().join(&wt_rel);
        std::fs::write(wt_abs.join("cclab/contested.txt"), "branch version\n").unwrap();

        // Archive stub
        let archive_abs = tmp
            .path()
            .join(".aw/archive/20260101-enhancement-conflict");
        std::fs::create_dir_all(&archive_abs).unwrap();
        std::fs::write(archive_abs.join("user_input.md"), "conflict test\n").unwrap();

        let config = RepoPlatformConfig {
            type_: "github".to_string(),
            repo: "test/repo".to_string(),
            host: None,
            default_branch: "main".to_string(),
            auto_commit: true,
            auto_pr: false,
        };

        let result = post_archive_git_ops(tmp.path(), slug, &archive_abs, Some(&config), &[]);
        assert!(
            result.is_err(),
            "expected hard error on merge conflict, got {:?}",
            result.ok().map(|r| r.git_warning)
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.to_lowercase().contains("conflict"),
            "error must mention conflict: {}",
            err_msg
        );
        assert!(
            err_msg.contains("Resolve manually") || err_msg.contains("resolve"),
            "error must suggest manual resolution: {}",
            err_msg
        );
    }

    /// When no worktree exists, post_archive_git_ops falls back to the legacy
    /// flow (steps 2 + 5 only) and does not error. The archive file under
    /// `.aw/archive/` is treated as a dirty path and committed.
    ///
    /// REQ: bug-create-change-merge-archive-moves-not-committed-sp
    /// Before the fix, the status pathspec was `-- cclab/` only, so the
    /// archive file was NOT seen as dirty and no commit was made
    /// (git_commit_sha was None). After the fix, the pathspec is
    /// `-- cclab/ .aw/`, so the archive file IS staged and committed.
    // REQ: worktree-per-change — backward compat legacy in-place changes
    #[test]
    fn test_post_archive_legacy_no_worktree() {
        let tmp = TempDir::new().unwrap();
        if !init_repo(tmp.path()) {
            return;
        }

        // Legacy flow: no worktree, one archive file under .aw/archive/.
        let archive_abs = tmp.path().join(".aw/archive/20260101-legacy");
        std::fs::create_dir_all(&archive_abs).unwrap();
        std::fs::write(archive_abs.join("user_input.md"), "legacy change\n").unwrap();

        let config = RepoPlatformConfig {
            type_: "github".to_string(),
            repo: "test/repo".to_string(),
            host: None,
            default_branch: "main".to_string(),
            auto_commit: true,
            auto_pr: false,
        };

        let result = post_archive_git_ops(
            tmp.path(),
            "legacy-change",
            &archive_abs,
            Some(&config),
            &[],
        );
        assert!(
            result.is_ok(),
            "legacy path should not error: {:?}",
            result.err()
        );
        let ops = result.unwrap();
        // The dirty .aw/ path is committed — SHA should be populated.
        assert!(
            ops.git_commit_sha.is_some(),
            "expected auto-commit of .aw/ archive path; got None"
        );
        assert!(
            ops.git_warning.is_none()
                || !ops
                    .git_warning
                    .as_deref()
                    .unwrap_or("")
                    .contains("conflict")
        );
    }

    /// Fix 1: After step 3 merge, git_commit_sha must be the merge commit SHA
    /// on project_root (not the worktree branch commit), and must not be null.
    // @spec projects/agentic-workflow/tech-design/core/logic/merge-gaps-fix.md#R1
    #[test]
    fn test_commit_sha_is_merge_commit_on_default_branch() {
        let tmp = TempDir::new().unwrap();
        if !init_repo(tmp.path()) {
            return;
        }

        let slug = "fix1-sha-capture";
        let Some(git) = find_git_binary() else { return };

        // Create a worktree branch and add a file to it.
        let wt_rel = format!(".aw/worktrees/{}", slug);
        std::fs::create_dir_all(tmp.path().join(".aw/worktrees")).unwrap();
        let add_wt = std::process::Command::new(&git)
            .args(["worktree", "add", "-b", &format!("cclab/{}", slug), &wt_rel])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        assert!(
            add_wt.status.success(),
            "git worktree add failed: {}",
            String::from_utf8_lossy(&add_wt.stderr)
        );

        let wt_abs = tmp.path().join(&wt_rel);
        std::fs::create_dir_all(wt_abs.join("cclab")).unwrap();
        std::fs::write(wt_abs.join("cclab/test-file.txt"), "sha test\n").unwrap();

        let archive_abs = tmp.path().join(".aw/archive/20260101-fix1-sha");
        std::fs::create_dir_all(&archive_abs).unwrap();
        std::fs::write(archive_abs.join("user_input.md"), "SHA capture fix\n").unwrap();

        let config = RepoPlatformConfig {
            type_: "github".to_string(),
            repo: "test/repo".to_string(),
            host: None,
            default_branch: "main".to_string(),
            auto_commit: true,
            auto_pr: false,
        };

        let result = post_archive_git_ops(tmp.path(), slug, &archive_abs, Some(&config), &[]);
        assert!(
            result.is_ok(),
            "post_archive_git_ops failed: {:?}",
            result.err()
        );
        let ops = result.unwrap();

        // Must have a SHA — the merge commit on main.
        assert!(
            ops.git_commit_sha.is_some(),
            "git_commit_sha must not be null after merge"
        );
        let sha = ops.git_commit_sha.unwrap();
        assert!(sha.len() >= 7, "SHA must be at least 7 chars, got: {}", sha);
        assert!(
            sha.chars().all(|c| c.is_ascii_hexdigit()),
            "SHA must be hex, got: {}",
            sha
        );

        // Verify the SHA matches HEAD on project_root (main branch after merge).
        let head_output = std::process::Command::new(&git)
            .args(["rev-parse", "HEAD"])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        let head_sha = String::from_utf8_lossy(&head_output.stdout)
            .trim()
            .to_string();
        assert!(
            head_sha.starts_with(&sha)
                || sha.starts_with(&head_sha[..sha.len().min(head_sha.len())]),
            "reported SHA {} must match or be prefix of HEAD SHA {}",
            sha,
            head_sha
        );
    }

    /// Fix 2: auto_pr=true in legacy in-place flow (no worktree) must NOT attempt PR creation.
    // @spec projects/agentic-workflow/tech-design/core/logic/merge-gaps-fix.md#R2
    #[test]
    fn test_auto_pr_skipped_in_legacy_flow() {
        let tmp = TempDir::new().unwrap();
        if !init_repo(tmp.path()) {
            return;
        }

        // No worktree created — legacy in-place flow.
        let archive_abs = tmp.path().join(".aw/archive/20260101-legacy-pr");
        std::fs::create_dir_all(&archive_abs).unwrap();
        std::fs::write(archive_abs.join("user_input.md"), "legacy auto-pr test\n").unwrap();

        let config = RepoPlatformConfig {
            type_: "github".to_string(),
            repo: "test/repo".to_string(),
            host: None,
            default_branch: "main".to_string(),
            auto_commit: true,
            auto_pr: true, // enabled, but no worktree → must be skipped
        };

        let result = post_archive_git_ops(
            tmp.path(),
            "legacy-pr-change",
            &archive_abs,
            Some(&config),
            &[],
        );
        assert!(
            result.is_ok(),
            "should not error in legacy flow: {:?}",
            result.err()
        );
        let ops = result.unwrap();
        // PR URL must be None (no worktree → PR skipped).
        assert!(
            ops.pr_url.is_none(),
            "pr_url must be None in legacy flow when no worktree exists; got: {:?}",
            ops.pr_url
        );
    }

    /// Fix 2: auto_pr=false with a worktree must produce no PR URL and no PR warning.
    // @spec projects/agentic-workflow/tech-design/core/logic/merge-gaps-fix.md#R2
    #[test]
    fn test_auto_pr_disabled_produces_no_pr_url() {
        let tmp = TempDir::new().unwrap();
        if !init_repo(tmp.path()) {
            return;
        }

        let slug = "fix2-no-pr";
        let Some(git) = find_git_binary() else { return };

        let wt_rel = format!(".aw/worktrees/{}", slug);
        std::fs::create_dir_all(tmp.path().join(".aw/worktrees")).unwrap();
        std::process::Command::new(&git)
            .args(["worktree", "add", "-b", &format!("cclab/{}", slug), &wt_rel])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        let wt_abs = tmp.path().join(&wt_rel);
        std::fs::create_dir_all(wt_abs.join("cclab")).unwrap();
        std::fs::write(wt_abs.join("cclab/test.txt"), "content\n").unwrap();

        let archive_abs = tmp.path().join(".aw/archive/20260101-fix2-no-pr");
        std::fs::create_dir_all(&archive_abs).unwrap();
        std::fs::write(archive_abs.join("user_input.md"), "auto-pr disabled test\n").unwrap();

        let config = RepoPlatformConfig {
            type_: "github".to_string(),
            repo: "test/repo".to_string(),
            host: None,
            default_branch: "main".to_string(),
            auto_commit: true,
            auto_pr: false,
        };

        let result = post_archive_git_ops(tmp.path(), slug, &archive_abs, Some(&config), &[]);
        assert!(result.is_ok(), "should not error: {:?}", result.err());
        let ops = result.unwrap();
        assert!(
            ops.pr_url.is_none(),
            "pr_url must be None when auto_pr=false"
        );
    }

    /// Fix 3: capture_head_sha returns the current HEAD SHA for a given dir.
    // @spec projects/agentic-workflow/tech-design/core/logic/merge-gaps-fix.md#R1
    #[test]
    fn test_capture_head_sha_returns_valid_sha() {
        let tmp = TempDir::new().unwrap();
        if !init_repo(tmp.path()) {
            return;
        }
        let Some(git) = find_git_binary() else { return };
        let sha = capture_head_sha(&git, tmp.path());
        assert!(
            sha.is_some(),
            "capture_head_sha must return Some for a valid repo"
        );
        let sha_str = sha.unwrap();
        assert_eq!(
            sha_str.len(),
            40,
            "SHA must be 40 hex chars, got len={}",
            sha_str.len()
        );
        assert!(
            sha_str.chars().all(|c| c.is_ascii_hexdigit()),
            "SHA must be hex, got: {}",
            sha_str
        );
    }
}

````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/merge_git_ops.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "tests"
      - "<module-trailer>"
    description: "Regression tests for post-archive git operations, config parsing, worktree merge behavior, and PR gating."
```
