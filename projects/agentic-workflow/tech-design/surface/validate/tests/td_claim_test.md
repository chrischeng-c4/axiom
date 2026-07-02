---
id: projects-score-tests-td-claim-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Standardization TDs support brownfield takeover, semantic coverage, traceability, and production readiness gates."
---

# Standardized projects/agentic-workflow/tests/cli/tests/td_claim_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/tests/cli/tests/td_claim_test.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/tests/cli/tests/td_claim_test.rs -->
```rust
//! Integration tests for `aw td claim` (Phase 2 recovery).
//!
//! Tests for `aw td claim`.

use agentic_workflow::cli::Commands;
use clap::{CommandFactory, Parser};

#[derive(Parser)]
#[command(name = "aw")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// R1 smoke: `aw td claim` registers as a subcommand with the right args.
#[test]
fn test_td_claim_registered() {
    let cmd = Cli::command();
    let td = cmd.find_subcommand("td").expect("td namespace");
    let claim = td.find_subcommand("claim").expect("td claim subcommand");
    let positionals: Vec<String> = claim
        .get_positionals()
        .map(|p: &clap::Arg| p.get_id().as_str().to_string())
        .collect();
    assert!(positionals.iter().any(|p| p == "slug"));
}

/// R1b: --from-path flag is registered.
#[test]
fn test_td_claim_from_path_flag() {
    let cmd = Cli::command();
    let claim = cmd
        .find_subcommand("td")
        .and_then(|c| c.find_subcommand("claim"))
        .expect("td claim");
    claim
        .get_arguments()
        .find(|a: &&clap::Arg| a.get_id().as_str() == "from_path")
        .expect("--from-path registered");
}

/// R2: --force-rebase flag is registered as a boolean.
#[test]
fn test_td_claim_force_rebase_flag() {
    let cmd = Cli::command();
    let claim = cmd
        .find_subcommand("td")
        .and_then(|c| c.find_subcommand("claim"))
        .expect("td claim");
    claim
        .get_arguments()
        .find(|a: &&clap::Arg| a.get_id().as_str() == "force_rebase")
        .expect("--force-rebase registered");
}

/// Trailer constants are wired correctly for Td-Claim.
#[test]
fn test_td_claim_trailer_const() {
    use agentic_workflow::issues::types::lifecycle_trailer;
    assert_eq!(lifecycle_trailer::TD_CLAIM, "Td-Claim");
}

/// Phase write target is `td_created`: claim adopts an already-authored
/// spec into the worktree, which is semantically the post-create state.
/// `td_reviewed` has no outgoing transition in the linear lifecycle and
/// would permanently deadlock (issue #843).
#[test]
fn test_td_claim_phase_target() {
    use agentic_workflow::issues::types::td_phase;
    assert_eq!(td_phase::TD_CREATED, "td_created");
    // The linear lifecycle's next-command router must accept claim's
    // written phase and route it to the same verb claim's own dispatch
    // envelope names (`aw td gen`).
    assert_eq!(
        td_phase::next_phase_command("td_created"),
        Some("aw td gen")
    );
}

/// R6 e2e: B2 recovery happy path — `td claim --from-path <spec>` against
/// a fresh slug with no pre-existing issue. Verifies that stub creation
/// happens in the current checkout and that the Td-Claim lifecycle trailer
/// + phase advance both land without creating `.aw/worktrees/`.
///
/// @spec projects/agentic-workflow/tech-design/surface/specs/score-td-claim-stub-placement-fix.md#test-plan
#[test]
fn test_td_claim_e2e_phase_advance() {
    use std::process::Command;

    let Some(git) = agentic_workflow::git::find_git_bin() else {
        eprintln!("skipping: git binary not on PATH");
        return;
    };
    let Ok(aw_bin) = std::env::var("CARGO_BIN_EXE_aw") else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
        return;
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();

    // Initialize a clean git repo with one commit so branch activation works.
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["init", "-b", "main"])
        .status()
        .expect("git init");
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["config", "user.email", "test@test"])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["config", "user.name", "test"])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["config", "commit.gpgsign", "false"])
        .status()
        .unwrap();
    std::fs::write(root.join("README.md"), "seed\n").unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "seed"])
        .status()
        .unwrap();

    // Bootstrap minimal .aw/ layout.
    std::fs::create_dir_all(root.join(".aw/issues/open")).unwrap();
    std::fs::create_dir_all(root.join(".aw/tech-design")).unwrap();
    std::fs::write(root.join(".aw/config.toml"), "").unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "bootstrap .aw"])
        .status()
        .unwrap();

    // Write a TD spec on disk under a temporary location outside the repo
    // working tree entirely. `td claim` activates a `td-<slug>` branch when
    // launched from `main`, which requires a clean working tree; a spec file
    // written inside `root` (even outside `.aw/`) shows up as an untracked
    // change and trips that guard before claim ever runs.
    let spec_dir = tempfile::tempdir().expect("spec tempdir");
    let spec_src = spec_dir.path().join("external-spec.md");
    std::fs::write(
        &spec_src,
        "---\nslug: e2e-claim-test\n---\n\n# external spec\n",
    )
    .unwrap();

    let slug = "e2e-claim-test";
    let status = Command::new(&aw_bin)
        .arg("td")
        .arg("claim")
        .arg(slug)
        .arg("--from-path")
        .arg(&spec_src)
        .current_dir(root)
        .status()
        .expect("run aw td claim");
    assert!(status.success(), "td claim --from-path should succeed");

    // Stub MUST exist in the ephemeral issue working-copy store (issues live
    // under `/tmp/aw/...`, not `.aw/issues/`; see LocalBackend::from_project_root),
    // with phase: td_created (the linear lifecycle's post-create phase;
    // `aw td gen`'s guard requires exactly this value — see issue #843).
    use agentic_workflow::issues::backends::local::LocalBackend;
    let wt_stub = LocalBackend::from_project_root(root)
        .issues_dir()
        .join("open")
        .join(format!("{}.md", slug));
    assert!(
        wt_stub.exists(),
        "stub missing in issue store: {}",
        wt_stub.display()
    );
    assert!(
        !root.join(".aw/worktrees").exists(),
        ".aw/worktrees/ must not be created by td claim"
    );
    let stub_body = std::fs::read_to_string(&wt_stub).unwrap();
    assert!(
        stub_body.contains("phase: td_created"),
        "phase not advanced:\n{}",
        stub_body
    );

    // Current checkout git log must contain Lifecycle-Stage: Td-Claim trailer.
    let log = Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["log", "--format=%B"])
        .output()
        .expect("git log");
    let log_text = String::from_utf8_lossy(&log.stdout);
    assert!(
        log_text.contains("Lifecycle-Stage: Td-Claim"),
        "Td-Claim trailer missing from log:\n{}",
        log_text
    );
}

/// AC1/AC2: the exact command claim's dispatch envelope names (`aw td gen`)
/// succeeds against the phase claim just wrote, and the resulting phase after
/// gen is one with an outgoing transition (never a dead end) — no verb
/// sequence starting from claim reaches a phase with no successor.
///
/// @spec projects/agentic-workflow/tech-design/surface/specs/remove-td-cb-crrr-collapse-to-linear-lifecycle.md
#[test]
fn test_td_claim_then_gen_succeeds() {
    use std::process::Command;

    let Some(git) = agentic_workflow::git::find_git_bin() else {
        eprintln!("skipping: git binary not on PATH");
        return;
    };
    let Ok(aw_bin) = std::env::var("CARGO_BIN_EXE_aw") else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
        return;
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();

    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["init", "-b", "main"])
        .status()
        .expect("git init");
    for (k, v) in [
        ("user.email", "test@test"),
        ("user.name", "test"),
        ("commit.gpgsign", "false"),
    ] {
        Command::new(&git)
            .arg("-C")
            .arg(root)
            .args(["config", k, v])
            .status()
            .unwrap();
    }
    std::fs::write(root.join("README.md"), "seed\n").unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "seed"])
        .status()
        .unwrap();

    std::fs::create_dir_all(root.join(".aw/issues/open")).unwrap();
    std::fs::create_dir_all(root.join(".aw/tech-design")).unwrap();
    std::fs::write(root.join(".aw/config.toml"), "").unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "bootstrap .aw"])
        .status()
        .unwrap();

    let slug = "e2e-claim-gen-test";
    // Spec must live fully outside the repo working tree (see comment in
    // test_td_claim_e2e_phase_advance above) or the branch-activation clean
    // tree guard rejects claim before it ever writes a phase.
    let spec_dir = tempfile::tempdir().expect("spec tempdir");
    let spec_src = spec_dir.path().join("external-spec.md");
    std::fs::write(
        &spec_src,
        format!("---\nslug: {slug}\n---\n\n# external spec\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges: []\n```\n"),
    )
    .unwrap();

    let claim_status = Command::new(&aw_bin)
        .arg("td")
        .arg("claim")
        .arg(slug)
        .arg("--from-path")
        .arg(&spec_src)
        .current_dir(root)
        .status()
        .expect("run aw td claim");
    assert!(
        claim_status.success(),
        "td claim --from-path should succeed"
    );

    // The phase claim wrote must be exactly what `aw td gen`'s own guard
    // requires (run_gen_code checks phase == "td_created").
    use agentic_workflow::issues::backends::local::LocalBackend;
    let wt_stub = LocalBackend::from_project_root(root)
        .issues_dir()
        .join("open")
        .join(format!("{}.md", slug));
    let stub_body = std::fs::read_to_string(&wt_stub).unwrap();
    assert!(
        stub_body.contains("phase: td_created"),
        "claim must write td_created so gen's guard accepts it:\n{}",
        stub_body
    );

    // Running exactly the command named in claim's dispatch envelope
    // (`aw td gen`) must not hit the phase guard error.
    let gen_output = Command::new(&aw_bin)
        .arg("td")
        .arg("gen")
        .arg(slug)
        .arg("--spec-path")
        .arg("external-spec.md")
        .current_dir(root)
        .output()
        .expect("run aw td gen");
    let gen_stdout = String::from_utf8_lossy(&gen_output.stdout);
    let gen_stderr = String::from_utf8_lossy(&gen_output.stderr);
    assert!(
        !gen_stdout.contains("cannot gen-code: phase is"),
        "td gen rejected claim's phase (deadlock):\nstdout:\n{}\nstderr:\n{}",
        gen_stdout,
        gen_stderr
    );
}

/// Recursively find a file named `name` under `dir` (skipping `.git`). Used
/// to locate wherever `td claim`'s dest-path derivation copied the claimed
/// spec, without hard-coding that derivation logic in the test itself.
fn find_file_named(dir: &std::path::Path, name: &str) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|s| s.to_str()) == Some(".git") {
                continue;
            }
            if let Some(found) = find_file_named(&path, name) {
                return Some(found);
            }
        } else if path.file_name().and_then(|s| s.to_str()) == Some(name) {
            return Some(path);
        }
    }
    None
}

/// Issue #939: `aw td claim --from-path` copies an already-authored spec
/// into the worktree — it must also record that copied spec's
/// repo-relative path in the issue's `implements`, the same way `aw td
/// create` does (see `inplace_mode_test.rs`), so `cb.rs`'s tier-1
/// `Issue.implements` scope resolution (#854) has real data to resolve a
/// claimed spec from instead of always falling through to tier-3
/// derived-path guessing.
#[test]
fn test_td_claim_records_implements() {
    use std::process::Command;

    let Some(git) = agentic_workflow::git::find_git_bin() else {
        eprintln!("skipping: git binary not on PATH");
        return;
    };
    let Ok(aw_bin) = std::env::var("CARGO_BIN_EXE_aw") else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
        return;
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();

    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["init", "-b", "main"])
        .status()
        .expect("git init");
    for (k, v) in [
        ("user.email", "test@test"),
        ("user.name", "test"),
        ("commit.gpgsign", "false"),
    ] {
        Command::new(&git)
            .arg("-C")
            .arg(root)
            .args(["config", k, v])
            .status()
            .unwrap();
    }
    std::fs::write(root.join("README.md"), "seed\n").unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "seed"])
        .status()
        .unwrap();

    std::fs::create_dir_all(root.join(".aw/issues/open")).unwrap();
    std::fs::create_dir_all(root.join(".aw/tech-design")).unwrap();
    std::fs::write(root.join(".aw/config.toml"), "").unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "bootstrap .aw"])
        .status()
        .unwrap();

    let spec_dir = tempfile::tempdir().expect("spec tempdir");
    let spec_src = spec_dir.path().join("implements-claim-spec.md");
    std::fs::write(
        &spec_src,
        "---\nslug: e2e-claim-implements-test\n---\n\n# external spec\n",
    )
    .unwrap();

    let slug = "e2e-claim-implements-test";
    let status = Command::new(&aw_bin)
        .arg("td")
        .arg("claim")
        .arg(slug)
        .arg("--from-path")
        .arg(&spec_src)
        .current_dir(root)
        .status()
        .expect("run aw td claim");
    assert!(status.success(), "td claim --from-path should succeed");

    let copied = find_file_named(root, "implements-claim-spec.md")
        .expect("claim must copy the --from-path spec somewhere into the worktree");
    let dest_rel = copied
        .strip_prefix(root)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/");

    use agentic_workflow::issues::backends::local::LocalBackend;
    let wt_stub = LocalBackend::from_project_root(root)
        .issues_dir()
        .join("open")
        .join(format!("{}.md", slug));
    let stub_body = std::fs::read_to_string(&wt_stub).unwrap();
    assert!(
        stub_body.contains("implements:") && stub_body.contains(dest_rel.as_str()),
        "claim must record the copied spec path '{}' in Issue.implements:\n{}",
        dest_rel,
        stub_body
    );
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/cli/tests/td_claim_test.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Existing source claimed by `aw standardize managed run`. The code is
      wrapped in a tracked HANDWRITE block until deterministic generator
      coverage can replace it with CODEGEN.
```
