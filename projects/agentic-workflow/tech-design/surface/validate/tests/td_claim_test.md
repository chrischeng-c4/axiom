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

use clap::{CommandFactory, Parser};
use agentic_workflow::cli::Commands;

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

/// Phase write target is `td_reviewed` (CRRR bypass).
#[test]
fn test_td_claim_phase_target() {
    use agentic_workflow::issues::types::td_phase;
    // td claim writes phase td_reviewed.
    assert_eq!(td_phase::TD_REVIEWED, "td_reviewed");
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
    let Ok(score_bin) = std::env::var("CARGO_BIN_EXE_score") else {
        eprintln!("skipping: CARGO_BIN_EXE_score not set");
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

    // Write a TD spec on disk under a temporary location (outside .aw/).
    let spec_src = root.join("external-spec.md");
    std::fs::write(
        &spec_src,
        "---\nslug: e2e-claim-test\n---\n\n# external spec\n",
    )
    .unwrap();

    let slug = "e2e-claim-test";
    let status = Command::new(&score_bin)
        .arg("td")
        .arg("claim")
        .arg(slug)
        .arg("--from-path")
        .arg(&spec_src)
        .current_dir(root)
        .status()
        .expect("run aw td claim");
    assert!(status.success(), "td claim --from-path should succeed");

    // Stub MUST exist in the current checkout, with phase: td_reviewed.
    let wt_stub = root.join(".aw/issues/open").join(format!("{}.md", slug));
    assert!(
        wt_stub.exists(),
        "stub missing in current checkout: {}",
        wt_stub.display()
    );
    assert!(
        !root.join(".aw/worktrees").exists(),
        ".aw/worktrees/ must not be created by td claim"
    );
    let stub_body = std::fs::read_to_string(&wt_stub).unwrap();
    assert!(
        stub_body.contains("phase: td_reviewed"),
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
