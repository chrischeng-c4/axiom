---
id: projects-score-tests-cb-claim-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Standardization TDs support brownfield takeover, semantic coverage, traceability, and production readiness gates."
---

# Standardized projects/agentic-workflow/tests/cli/tests/cb_claim_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/tests/cli/tests/cb_claim_test.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/tests/cli/tests/cb_claim_test.rs -->
```rust
// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/validate/tests/cb_claim_test.md#source
// CODEGEN-BEGIN
//! Integration tests for `aw td code-claim` (Phase 2 recovery).
//!
//! Tests for `aw td code-claim`.

use agentic_workflow::cli::Commands;
use clap::{CommandFactory, Parser};

#[derive(Parser)]
#[command(name = "aw")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// R3 smoke: `aw td code-claim` registers with the right positional arg.
#[test]
fn test_cb_claim_registered() {
    let cmd = Cli::command();
    let claim = cmd
        .find_subcommand("td")
        .and_then(|c| c.find_subcommand("code-claim"))
        .expect("td code-claim subcommand");
    let positionals: Vec<String> = claim
        .get_positionals()
        .map(|p: &clap::Arg| p.get_id().as_str().to_string())
        .collect();
    assert!(positionals.iter().any(|p| p == "code_path"));
}

/// R4: --init flag registered.
#[test]
fn test_cb_claim_init_flag() {
    let cmd = Cli::command();
    let claim = cmd
        .find_subcommand("td")
        .and_then(|c| c.find_subcommand("code-claim"))
        .expect("td code-claim");
    claim
        .get_arguments()
        .find(|a: &&clap::Arg| a.get_id().as_str() == "init")
        .expect("--init registered");
}

/// --issue-stub flag registered.
#[test]
fn test_cb_claim_issue_stub_flag() {
    let cmd = Cli::command();
    let claim = cmd
        .find_subcommand("td")
        .and_then(|c| c.find_subcommand("code-claim"))
        .expect("td code-claim");
    claim
        .get_arguments()
        .find(|a: &&clap::Arg| a.get_id().as_str() == "issue_stub")
        .expect("--issue-stub registered");
}

/// Cb-Claim trailer constant is exposed for downstream readers.
#[test]
fn test_cb_claim_trailer_const() {
    use agentic_workflow::issues::types::lifecycle_trailer;
    assert_eq!(lifecycle_trailer::CB_CLAIM, "Cb-Claim");
}

/// R3 e2e: full fillback + write + trailer flow. Marked #[ignore]
/// because the fillback pipeline requires tree-sitter parsing on a real
/// codebase plus filesystem writes.
#[test]
#[ignore = "requires real codebase + fillback infrastructure; run manually with --ignored"]
fn test_cb_claim_fillback_invoked_e2e() {
    // Reserved for end-to-end: feed a small fixture into `aw td code-claim`,
    // assert .aw/tech-design/<group>/<derived>.md exists and contains
    // YAML frontmatter; assert the result envelope action == "done".
}

/// R3: `--non-interactive` flag is registered as a boolean.
///
/// @spec projects/agentic-workflow/tech-design/surface/specs/score-recovery-verbs-non-interactive.md#test-plan
#[test]
fn test_cb_claim_non_interactive_flag_registered() {
    let cmd = Cli::command();
    let claim = cmd
        .find_subcommand("td")
        .and_then(|c| c.find_subcommand("code-claim"))
        .expect("td code-claim");
    claim
        .get_arguments()
        .find(|a: &&clap::Arg| a.get_id().as_str() == "non_interactive")
        .expect("--non-interactive registered");
}

/// R6 e2e: `aw td code-claim --non-interactive --init <crate-path>` against
/// a synthesised tempdir crate. Verifies (a) exit 0; (b) command does not
/// hang on stdin; (c) at least one spec file is written under
/// `.aw/tech-design/`. Wraps with a 30-second timeout enforced via the
/// child process — if the interactive path is reached, dialoguer would
/// block on the closed stdin and the timeout fires.
///
/// @spec projects/agentic-workflow/tech-design/surface/specs/score-recovery-verbs-non-interactive.md#test-plan
#[test]
fn test_cb_claim_non_interactive_writes_spec() {
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    let Ok(aw_bin) = std::env::var("CARGO_BIN_EXE_aw") else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
        return;
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();

    // Synthesise a minimal crate with one public type.
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("src/lib.rs"),
        "pub struct Foo {\n    pub name: String,\n}\n",
    )
    .unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"foo\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
    )
    .unwrap();

    let mut child = Command::new(&aw_bin)
        .arg("td")
        .arg("code-claim")
        .arg(".")
        .arg("--non-interactive")
        .arg("--init")
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn aw td code-claim");

    let deadline = Instant::now() + Duration::from_secs(30);
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break s,
            Ok(None) => {
                if Instant::now() > deadline {
                    let _ = child.kill();
                    panic!("aw td code-claim --non-interactive hung past 30s — interactive prompt still blocking");
                }
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(e) => panic!("try_wait failed: {}", e),
        }
    };

    assert!(
        status.success(),
        "aw td code-claim --non-interactive exit code {:?}",
        status.code()
    );

    // At least one spec file should land under .aw/tech-design/.
    let td_dir = root.join(".aw/tech-design");
    assert!(td_dir.exists(), "tech_design dir missing");
    let spec_count = count_md_recursive(&td_dir);
    assert!(
        spec_count > 0,
        "no spec files written under {}",
        td_dir.display()
    );
}

fn count_md_recursive(dir: &std::path::Path) -> usize {
    let mut n = 0;
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            n += count_md_recursive(&p);
        } else if p.extension().map(|e| e == "md").unwrap_or(false) {
            n += 1;
        }
    }
    n
}

// CODEGEN-END
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/cli/tests/cb_claim_test.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Existing source claimed by `aw standardize managed run`. The code is
      wrapped in a tracked HANDWRITE block until deterministic generator
      coverage can replace it with CODEGEN.
```
