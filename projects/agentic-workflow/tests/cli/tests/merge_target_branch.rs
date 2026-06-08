// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/validate/tests/merge_target_branch.md#source
// CODEGEN-BEGIN
//! Regression tests for `resolve_merge_target` — covers the four resolution
//! cases from the TD spec test-plan (TP-A through TP-D) plus the error case.
//!
//! These are unit tests for the `merge_target` module rather than full CLI
//! integration tests (which would require building and installing the binary
//! and setting up a complete worktree), because the `resolve_merge_target`
//! function encapsulates all four precedence cases and can be exercised
//! directly via the library crate.
//!
//! Full end-to-end CLI integration tests (invoking the `score` binary) are
//! tracked as a follow-up — labelled `#[ignore]` below.

use std::process::Command;

// ── helpers ────────────────────────────────────────────────────────────────

fn init_git_repo(dir: &std::path::Path) {
    for args in &[
        vec!["init", "--initial-branch=main"],
        vec!["config", "user.email", "test@example.com"],
        vec!["config", "user.name", "Test"],
        vec!["commit", "--allow-empty", "-m", "init"],
    ] {
        Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .unwrap_or_else(|_| panic!("git {:?} failed", args));
    }
}

fn write_score_config(dir: &std::path::Path, default_branch: Option<&str>) {
    let score_dir = dir.join(".aw");
    std::fs::create_dir_all(&score_dir).unwrap();
    let content = match default_branch {
        Some(b) => format!(
            "[agentic_workflow.repo_platform]\ntype = \"github\"\nrepo = \"test/repo\"\ndefault_branch = \"{}\"\n",
            b
        ),
        None => "[agentic_workflow.repo_platform]\ntype = \"github\"\nrepo = \"test/repo\"\n".to_string(),
    };
    std::fs::write(score_dir.join("config.toml"), content).unwrap();
}

fn head_sha(dir: &std::path::Path) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn detach_head(dir: &std::path::Path) {
    let sha = head_sha(dir);
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["checkout", &sha])
        .output()
        .unwrap();
}

// ── TP-A: feature branch → merge should target feature branch ─────────────

/// REQ: TP-A
#[test]
fn case_a_feature_branch_detected() {
    let tmp = tempfile::tempdir().unwrap();
    init_git_repo(tmp.path());
    write_score_config(tmp.path(), Some("main"));

    // Create and switch to a feature branch
    Command::new("git")
        .arg("-C")
        .arg(tmp.path())
        .args(["checkout", "-b", "feature-abc"])
        .output()
        .unwrap();

    let result =
        agentic_workflow::cli::merge_target::resolve_merge_target(None, None, tmp.path()).unwrap();
    assert_eq!(
        result, "feature-abc",
        "merge target must be the current feature branch, not main"
    );
}

// ── TP-B: main branch → merge should target main ──────────────────────────

/// REQ: TP-B
#[test]
fn case_b_main_branch_detected() {
    let tmp = tempfile::tempdir().unwrap();
    init_git_repo(tmp.path());
    // stays on 'main'

    let result =
        agentic_workflow::cli::merge_target::resolve_merge_target(None, None, tmp.path()).unwrap();
    assert_eq!(result, "main");
}

// ── TP-C: --target-branch override wins regardless of current branch ───────

/// REQ: TP-C
#[test]
fn case_c_target_branch_override_wins() {
    let tmp = tempfile::tempdir().unwrap();
    init_git_repo(tmp.path());
    write_score_config(tmp.path(), Some("main"));

    // Currently on feature branch
    Command::new("git")
        .arg("-C")
        .arg(tmp.path())
        .args(["checkout", "-b", "feature-xyz"])
        .output()
        .unwrap();

    let result = agentic_workflow::cli::merge_target::resolve_merge_target(
        Some("release-1.0".to_string()),
        None,
        tmp.path(),
    )
    .unwrap();
    assert_eq!(
        result, "release-1.0",
        "--target-branch override must win over detected branch"
    );
}

// ── TP-D: detached HEAD + config default_branch → merge lands on config branch

/// REQ: TP-D
#[test]
fn case_d_detached_head_uses_config_default_branch() {
    let tmp = tempfile::tempdir().unwrap();
    init_git_repo(tmp.path());
    write_score_config(tmp.path(), Some("develop"));
    detach_head(tmp.path());

    let result =
        agentic_workflow::cli::merge_target::resolve_merge_target(None, None, tmp.path()).unwrap();
    assert_eq!(
        result, "develop",
        "detached HEAD must fall back to config default_branch"
    );
}

// ── error path: detached HEAD + no config → Err ───────────────────────────

/// REQ: TP-D (error variant)
#[test]
fn case_detached_head_no_config_returns_error() {
    let tmp = tempfile::tempdir().unwrap();
    init_git_repo(tmp.path());
    // No .aw/config.toml at all
    detach_head(tmp.path());

    let err = agentic_workflow::cli::merge_target::resolve_merge_target(None, None, tmp.path())
        .unwrap_err();
    assert!(
        err.to_string().contains("cannot determine merge target"),
        "error message must mention 'cannot determine merge target', got: {}",
        err
    );
}

// ── TP-E: stub — full CLI end-to-end (follow-up) ──────────────────────────

/// REQ: TP-E
/// Full `aw td merge` / `aw wi merge` end-to-end tests that spin up
/// a complete worktree and invoke the built binary. Skipped here because they
/// require a built binary and a fully-formed worktree scaffold. Tracked as
/// follow-up work.
#[test]
#[ignore = "requires built binary and full worktree setup; tracked as follow-up"]
fn case_e_full_cli_regression() {}

// CODEGEN-END
