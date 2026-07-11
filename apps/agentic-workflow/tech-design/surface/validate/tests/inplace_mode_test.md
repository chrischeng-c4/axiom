---
id: projects-score-tests-inplace-mode-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Standardization TDs support brownfield takeover, semantic coverage, traceability, and production readiness gates."
---

# Standardized apps/agentic-workflow/tests/cli/tests/inplace_mode_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/tests/cli/tests/inplace_mode_test.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=apps/agentic-workflow/tests/cli/tests/inplace_mode_test.rs -->
````rust
// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/validate/tests/inplace_mode_test.md#source
// CODEGEN-BEGIN
//! End-to-end smoke tests for `[agentic_workflow.workspace] mode = "in_place"`.
//!
//! In-place branch lifecycle tests:
//! verb conversion lets the aw binary run CRRR on the host repo's
//! branches instead of provisioning sibling worktree directories.
//!
//! These tests require the `score` binary; cargo wires `CARGO_BIN_EXE_aw`
//! automatically when the binary target is part of the same package.

use agentic_workflow::issues::LocalBackend;
use std::process::Command;

fn skip_unless_ready() -> Option<(std::path::PathBuf, String)> {
    let git = agentic_workflow::git::find_git_bin()?;
    let bin = std::env::var("CARGO_BIN_EXE_aw").ok().or_else(|| {
        let exe = std::env::current_exe().ok()?;
        let debug_dir = exe.parent()?.parent()?;
        let bin = debug_dir.join(format!("aw{}", std::env::consts::EXE_SUFFIX));
        bin.exists().then(|| bin.display().to_string())
    })?;
    Some((git, bin))
}

fn bootstrap_repo(git: &std::path::Path, root: &std::path::Path) {
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["init", "-b", "main"])
        .status()
        .expect("git init");
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["config", "user.email", "t@t"])
        .status()
        .unwrap();
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["config", "user.name", "t"])
        .status()
        .unwrap();
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["config", "commit.gpgsign", "false"])
        .status()
        .unwrap();
    std::fs::write(root.join("README.md"), "seed\n").unwrap();
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "seed"])
        .status()
        .unwrap();
}

fn current_branch(git: &std::path::Path, root: &std::path::Path) -> String {
    let out = Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn branch_exists(git: &std::path::Path, root: &std::path::Path, branch: &str) -> bool {
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--verify", &format!("refs/heads/{}", branch)])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn git_status(git: &std::path::Path, root: &std::path::Path) -> String {
    let out = Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["status", "--porcelain"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn issue_path(root: &std::path::Path, slug: &str) -> std::path::PathBuf {
    LocalBackend::from_project_root(root)
        .issues_dir()
        .join("open")
        .join(format!("{slug}.md"))
}

fn write_issue_fixture(root: &std::path::Path, slug: &str, body: impl AsRef<str>) {
    let path = issue_path(root, slug);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body.as_ref()).unwrap();
}

fn read_issue_fixture(root: &std::path::Path, slug: &str) -> String {
    std::fs::read_to_string(issue_path(root, slug)).unwrap()
}

/// `td init` in InPlace mode should switch the host repo from `main` to branch
/// `td-<slug>` and NOT provision a `.aw/worktrees/td-<slug>/` dir.
#[test]
fn inplace_td_init_switches_branch_no_worktree_dir() {
    let Some((git, bin)) = skip_unless_ready() else {
        eprintln!("skipping: git or CARGO_BIN_EXE_aw missing");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    bootstrap_repo(&git, root);

    // Bootstrap .aw/ with InPlace mode enabled.
    std::fs::create_dir_all(root.join(".aw/tech-design")).unwrap();
    std::fs::write(
        root.join("aw.toml"),
        r#"
[agentic_workflow.workspace]
mode = "in_place"

[[projects]]
name = "agentic-workflow"
path = "."
"#,
    )
    .unwrap();

    // Open issue with state: open + a recognized project label so
    // `default_spec_path_for_issue_in_project` (#1403) can resolve it
    // through the aw.toml project row above.
    let slug = "demo-inplace";
    let issue_body = format!(
        "---\n\
         slug: {slug}\n\
         title: demo inplace flow\n\
         state: open\n\
         type: enhancement\n\
         labels: [\"crate:sdd\"]\n\
         review_count: 1\n\
         flagged_sections: [scope]\n\
         fill_retry_count: 1\n\
         ---\n\n# Body\n",
    );
    write_issue_fixture(root, slug, issue_body);
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "bootstrap"])
        .status()
        .unwrap();

    // Run `aw td create <slug>`.
    let out = Command::new(&bin)
        .arg("td")
        .arg("create")
        .arg(slug)
        .current_dir(root)
        .output()
        .expect("run aw td create");
    assert!(
        out.status.success(),
        "td create should succeed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    // Branch must exist.
    let branch = format!("td-{}", slug);
    assert!(
        branch_exists(&git, root, &branch),
        "branch '{}' should exist after td create",
        branch,
    );

    // Host repo must be ON the new branch (in-place steady state).
    assert_eq!(
        current_branch(&git, root),
        branch,
        "in-place mode should leave HEAD on td-<slug>",
    );

    // No worktree dir should be created.
    let worktree_dir = root.join(format!(".aw/worktrees/td-{}", slug));
    assert!(
        !worktree_dir.exists(),
        "InPlace mode must NOT provision {}",
        worktree_dir.display(),
    );

    let updated_issue = read_issue_fixture(root, slug);
    assert!(
        !updated_issue.contains("review_count:"),
        "td create should reset inherited issue review_count before TD review:\n{updated_issue}"
    );
    assert!(
        !updated_issue.contains("flagged_sections:"),
        "td create should reset inherited issue flagged_sections before TD review:\n{updated_issue}"
    );
    assert!(
        !updated_issue.contains("fill_retry_count:"),
        "td create should reset inherited issue fill_retry_count before TD review:\n{updated_issue}"
    );
}

/// `td create` should stay on a persistent project branch. Only `main` uses
/// the dedicated `td-<slug>` branch split.
#[test]
fn td_create_on_project_branch_stays_on_current_branch() {
    let Some((git, bin)) = skip_unless_ready() else {
        eprintln!("skipping: git or CARGO_BIN_EXE_aw missing");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    bootstrap_repo(&git, root);

    std::fs::create_dir_all(root.join(".aw/tech-design")).unwrap();
    std::fs::write(
        root.join("aw.toml"),
        r#"
[agentic_workflow.workspace]
mode = "in_place"

[[projects]]
name = "agentic-workflow"
path = "."
"#,
    )
    .unwrap();

    let slug = "demo-project-branch";
    let issue_body = format!(
        "---\n\
         slug: {slug}\n\
         title: demo project branch flow\n\
         state: open\n\
         type: enhancement\n\
         labels: [\"app:agentic-workflow\"]\n\
         review_count: 1\n\
         flagged_sections: [scope]\n\
         fill_retry_count: 1\n\
         ---\n\n# Body\n",
    );
    write_issue_fixture(root, slug, issue_body);
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "bootstrap"])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["switch", "-c", "project-score"])
        .status()
        .unwrap();

    let out = Command::new(&bin)
        .arg("td")
        .arg("create")
        .arg(slug)
        .current_dir(root)
        .output()
        .expect("run aw td create");
    assert!(
        out.status.success(),
        "td create should succeed on project branch:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    assert_eq!(current_branch(&git, root), "project-score");
    assert!(
        !branch_exists(&git, root, &format!("td-{}", slug)),
        "project branch mode should not create a td branch"
    );
    let updated_issue = read_issue_fixture(root, slug);
    assert!(
        updated_issue.contains("phase: td_inited"),
        "{updated_issue}"
    );
    assert!(
        updated_issue.contains("branch: project-score"),
        "{updated_issue}"
    );
    assert!(
        !updated_issue.contains("review_count:")
            && !updated_issue.contains("flagged_sections:")
            && !updated_issue.contains("fill_retry_count:"),
        "td create should clear inherited issue review state:\n{updated_issue}"
    );
}

#[test]
fn td_create_numeric_id_uses_tracker_id_branch_with_legacy_cache_file() {
    let Some((git, bin)) = skip_unless_ready() else {
        eprintln!("skipping: git or CARGO_BIN_EXE_aw missing");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    bootstrap_repo(&git, root);

    std::fs::create_dir_all(root.join(".aw/tech-design")).unwrap();
    std::fs::write(
        root.join("aw.toml"),
        "[[projects]]\nname = \"agentic-workflow\"\npath = \".\"\n",
    )
    .unwrap();

    let legacy_slug = "bug-slug-round-trip-broken-local-cache-slug-d";
    let issue_body = format!(
        "---\n\
         title: bug score slug round trip\n\
         state: open\n\
         type: bug\n\
         github_id: 1887\n\
         labels: [\"type:bug\", \"app:agentic-workflow\"]\n\
         ---\n\n# Body\n",
    );
    write_issue_fixture(root, legacy_slug, issue_body);
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "bootstrap legacy cache"])
        .status()
        .unwrap();

    let out = Command::new(&bin)
        .arg("td")
        .arg("create")
        .arg("1887")
        .current_dir(root)
        .output()
        .expect("run aw td create");
    assert!(
        out.status.success(),
        "td create should resolve numeric tracker id through legacy cache:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    assert!(
        branch_exists(&git, root, "td-1887"),
        "td create should provision the tracker-id branch"
    );
    assert_eq!(current_branch(&git, root), "td-1887");
    assert!(
        !issue_path(root, "1887").exists(),
        "td create should update the existing cache file instead of inventing a second one"
    );
    let updated = read_issue_fixture(root, legacy_slug);
    assert!(updated.contains("branch: td-1887"), "{updated}");
}

/// Issue #939: `aw td create` must record the spec path it provisions/locates
/// for the issue in `Issue.implements`, so `cb.rs`'s tier-1
/// `Issue.implements` scope resolution (#854) has real data instead of
/// always falling through to tier-3 derived-path guessing. Uses an explicit
/// `--spec-path` so the recorded value is deterministic rather than
/// depending on `default_spec_path_for_issue_in_project`'s derivation.
#[test]
fn td_create_records_spec_path_in_issue_implements() {
    let Some((git, bin)) = skip_unless_ready() else {
        eprintln!("skipping: git or CARGO_BIN_EXE_aw missing");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    bootstrap_repo(&git, root);

    std::fs::create_dir_all(root.join(".aw/tech-design")).unwrap();
    std::fs::write(root.join("aw.toml"), "").unwrap();

    let slug = "demo-939-implements-test";
    let issue_body = format!(
        "---\n\
         slug: {slug}\n\
         title: demo 939 implements flow\n\
         state: open\n\
         type: enhancement\n\
         labels: [\"app:agentic-workflow\"]\n\
         ---\n\n# Body\n",
    );
    write_issue_fixture(root, slug, issue_body);
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "bootstrap"])
        .status()
        .unwrap();

    let spec_path = "custom/td-939-implements-test.md";
    let out = Command::new(&bin)
        .arg("td")
        .arg("create")
        .arg(slug)
        .arg("--spec-path")
        .arg(spec_path)
        .current_dir(root)
        .output()
        .expect("run aw td create");
    assert!(
        out.status.success(),
        "td create should succeed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    let updated_issue = read_issue_fixture(root, slug);
    assert!(
        updated_issue.contains("implements:") && updated_issue.contains(spec_path),
        "td create must record --spec-path in Issue.implements:\n{}",
        updated_issue
    );
}

/// Issue #939 idempotency: an issue that already carries the target spec
/// path in `Issue.implements` (e.g. adopted from a prior partial run) must
/// not gain a duplicate entry when `aw td create` provisions it. Pre-seeds
/// `implements` directly in the fixture rather than issuing two live CLI
/// calls, since a bare (non-`--apply`) repeat `aw td create` call requires a
/// fully clean tree that the first call's own uncommitted section payload
/// file would otherwise trip.
#[test]
fn td_create_does_not_duplicate_existing_implements_entry() {
    let Some((git, bin)) = skip_unless_ready() else {
        eprintln!("skipping: git or CARGO_BIN_EXE_aw missing");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    bootstrap_repo(&git, root);

    std::fs::create_dir_all(root.join(".aw/tech-design")).unwrap();
    std::fs::write(root.join("aw.toml"), "").unwrap();

    let slug = "demo-939-implements-idempotent-test";
    let spec_path = "custom/td-939-implements-idempotent-test.md";
    let issue_body = format!(
        "---\n\
         slug: {slug}\n\
         title: demo 939 implements idempotency\n\
         state: open\n\
         type: enhancement\n\
         labels: [\"app:agentic-workflow\"]\n\
         implements: [\"{spec_path}\"]\n\
         ---\n\n# Body\n",
    );
    write_issue_fixture(root, slug, issue_body);
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "bootstrap"])
        .status()
        .unwrap();

    let out = Command::new(&bin)
        .arg("td")
        .arg("create")
        .arg(slug)
        .arg("--spec-path")
        .arg(spec_path)
        .current_dir(root)
        .output()
        .expect("run aw td create");
    assert!(
        out.status.success(),
        "td create should succeed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    let updated_issue = read_issue_fixture(root, slug);
    // Scope the count to the YAML frontmatter only: the issue body also
    // carries a `<!-- score:workflow-state -->` comment block whose
    // `expected_command`/`dirty_paths` fields legitimately echo the same
    // `--spec-path` string, which would otherwise inflate a whole-file
    // substring count without any `implements` duplication at all.
    let frontmatter = updated_issue.splitn(3, "---").nth(1).unwrap_or("");
    assert_eq!(
        frontmatter.matches(spec_path).count(),
        1,
        "implements must not duplicate a spec path the issue already carried:\n{}",
        updated_issue
    );
}

/// In InPlace mode, repeated `enter_workspace_for_verb(provision_if_missing=false)`
/// calls (which is what the verb-side activate helper does) must bail loudly
/// if the branch was never provisioned. We exercise that via `aw td check`
/// (slug mode), which expects the workspace to already exist. Formerly
/// exercised via `aw td validate`, retired by #1277 and folded into `aw td
/// check` — both routed through the same `td_activate_inplace_if_present`
/// bail path.
#[test]
fn inplace_verb_bails_without_init() {
    let Some((git, bin)) = skip_unless_ready() else {
        eprintln!("skipping: git or CARGO_BIN_EXE_aw missing");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    bootstrap_repo(&git, root);

    std::fs::create_dir_all(root.join(".aw/tech-design")).unwrap();
    std::fs::write(
        root.join("aw.toml"),
        "[agentic_workflow.workspace]\nmode = \"in_place\"\n",
    )
    .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "bootstrap"])
        .status()
        .unwrap();

    // No `td init` ran; from `main`, `td check` (slug mode) should bail
    // because branch td-missing does not exist locally.
    let out = Command::new(&bin)
        .arg("td")
        .arg("check")
        .arg("missing")
        .current_dir(root)
        .output()
        .expect("run aw td check");
    assert!(
        !out.status.success(),
        "td check without init should fail:\nstdout={}",
        String::from_utf8_lossy(&out.stdout),
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    assert!(
        combined.contains("workspace not found") || combined.contains("does not exist"),
        "expected 'workspace not found' / 'does not exist'; got:\n{}",
        combined,
    );
}

#[test]
fn wi_validate_accepts_apply_dirty_issue_file_on_issue_branch() {
    let Some((git, bin)) = skip_unless_ready() else {
        eprintln!("skipping: git or CARGO_BIN_EXE_aw missing");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    bootstrap_repo(&git, root);

    std::fs::create_dir_all(root.join(".aw")).unwrap();
    std::fs::write(root.join("aw.toml"), "").unwrap();

    let slug = "demo";
    write_issue_fixture(
        root,
        slug,
        format!(
            "---\n\
             type: enhancement\n\
             title: demo\n\
             state: open\n\
             labels: [\"type:enhancement\", \"app:agentic-workflow\"]\n\
             phase: created\n\
             ---\n\n\
             ## Problem\n\n\
             Initial stub.\n"
        ),
    );
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "bootstrap issue"])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["switch", "-c", "issue-demo"])
        .status()
        .unwrap();

    write_issue_fixture(
        root,
        slug,
        format!(
            "---\n\
             type: enhancement\n\
             title: demo\n\
             state: open\n\
             labels: [\"type:enhancement\", \"app:agentic-workflow\"]\n\
             phase: created\n\
             ---\n\n\
             ## Problem\n\n\
             Filled body from apply.\n\n\
             ## Capability Alignment\n\n\
             Capability: Issue branch validation\n\
             Capability Gap: apply-produced issue body diffs were rejected before validation handoff\n\
             Progress Evidence: this fixture keeps issue state in the temp backend while the checkout stays clean\n\n\
             ## Requirements\n\n\
             - R1: Validate accepts the apply-produced issue body diff.\n\n\
             ## Scope\n\n\
             ### In Scope\n\
             - Validate temp-backed issue state without checkout-hosted issue files.\n\n\
             ### Out of Scope\n\
             - Allowing unrelated dirty files.\n\n\
             ## Acceptance Criteria\n\n\
             - AC1: wi validate accepts the matching temp issue working copy without dirtying the checkout.\n\n\
             ## Agent Estimate\n\n\
             agent_minutes: 30\n\
             confidence: medium\n\
             risk: low\n\
             human_attention: none\n\n\
             ## Reference Context\n\n\
             ### Related Specs\n\
             | Spec | Relevance |\n\
             |------|-----------|\n\
             | issue-cli-envelope.md | Owns apply/validate handoff. |\n\n\
             ### Spec Plan\n\
             | Spec ID | Action | Main Spec Ref |\n\
             |---------|--------|---------------|\n\
             | score-validate-apply-handoff | update | issue-cli-envelope.md |\n"
        ),
    );

    let out = Command::new(&bin)
        .arg("wi")
        .arg("validate")
        .arg(slug)
        .current_dir(root)
        .output()
        .expect("run aw wi validate");
    assert!(
        out.status.success(),
        "wi validate should accept apply-produced temp issue working copy:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Validation passed for 'demo'.") || stdout.contains("\"passed\":true"),
        "WI validate should use backend projection rather than issue-branch CRRR:\n{}",
        stdout,
    );
    assert!(
        git_status(&git, root).trim().is_empty(),
        "WI validate must keep checkout state clean when issue state lives in the temp backend",
    );
}

// CODEGEN-END
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/tests/cli/tests/inplace_mode_test.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Existing source claimed by `aw standardize managed run`. The code is
      wrapped in a tracked HANDWRITE block until deterministic generator
      coverage can replace it with CODEGEN.
```
