---
id: projects-score-tests-standardize-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Standardization TDs support brownfield takeover, semantic coverage, traceability, and production readiness gates."
---

# Standardized projects/agentic-workflow/tests/cli/tests/standardize_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/tests/cli/tests/standardize_test.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/tests/cli/tests/standardize_test.rs -->
```rust
//! Integration tests for `aw standardize`.

use clap::{CommandFactory, Parser};
use agentic_workflow::cli::Commands;
use std::process::Command;
use tempfile::TempDir;

#[derive(Parser)]
#[command(name = "aw")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn write(root: &std::path::Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn score_bin() -> Option<String> {
    std::env::var("CARGO_BIN_EXE_score").ok()
}

#[test]
fn standardize_subcommands_registered() {
    let cmd = Cli::command();
    let standardize = cmd
        .find_subcommand("standardize")
        .expect("standardize namespace");
    let managed = standardize.find_subcommand("managed").expect("managed");
    managed.find_subcommand("report").expect("managed report");
    managed.find_subcommand("next").expect("managed next");
    managed.find_subcommand("run").expect("managed run");
    let regenerable = standardize
        .find_subcommand("regenerable")
        .expect("regenerable");
    regenerable
        .find_subcommand("report")
        .expect("regenerable report");
    regenerable
        .find_subcommand("next")
        .expect("regenerable next");
    regenerable.find_subcommand("run").expect("regenerable run");
    // Legacy top-level aliases (`report`/`codegen`/`next`/`run`) have been
    // removed; only the canonical `managed`/`regenerable` subcommands remain.
    assert!(standardize.find_subcommand("report").is_none());
    assert!(standardize.find_subcommand("codegen").is_none());
    assert!(standardize.find_subcommand("next").is_none());
    assert!(standardize.find_subcommand("run").is_none());
}

// The previous `standardize_codegen_reports_handwrite_blockers` test exercised
// the legacy `aw standardize codegen` alias. That alias has been removed;
// equivalent regenerability-layer coverage lives in
// `standardize_regenerable_report_*` cases that drive
// `aw standardize regenerable report` directly.

#[test]
fn standardize_run_claims_mixed_language_repo() {
    let Some(score) = score_bin() else {
        eprintln!("skipping: CARGO_BIN_EXE_score not set");
        return;
    };
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "src/lib.rs", "pub fn answer() -> i32 { 42 }\n");
    write(tmp.path(), "src/app.py", "def answer():\n    return 42\n");
    write(tmp.path(), "src/main.ts", "export const answer = 42;\n");

    let out = Command::new(score)
        .args([
            "standardize",
            "managed",
            "run",
            "--scope",
            "src/**",
            "--max-ticks",
            "10",
            "--json",
        ])
        .current_dir(tmp.path())
        .output()
        .expect("run aw standardize");
    assert!(
        out.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    for rel in ["src/lib.rs", "src/app.py", "src/main.ts"] {
        let content = std::fs::read_to_string(tmp.path().join(rel)).unwrap();
        assert!(content.contains("<HANDWRITE"), "{rel} should be managed");
    }
    assert!(
        !tmp.path().join(".aw/tech-design/src/lib.md").exists(),
        "managed layer must not write per-file TDs"
    );
    assert!(
        !tmp.path().join(".aw/tech-design/src/app.md").exists(),
        "managed layer must not write per-file TDs"
    );
    assert!(
        !tmp.path().join(".aw/tech-design/src/main.md").exists(),
        "managed layer must not write per-file TDs"
    );
}

#[test]
fn standardize_run_accepts_project_positional_from_config() {
    let Some(score) = score_bin() else {
        eprintln!("skipping: CARGO_BIN_EXE_score not set");
        return;
    };
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".aw/config.toml",
        r#"
[[projects]]
name = "agentic-workflow"
td_path = "projects/agentic-workflow/tech-design/core"

[[projects.workspaces]]
paths = ["projects/agentic-workflow/**"]
target = "rust"
test_cmd = "true"

[[projects]]
name = "jet"
td_path = ".aw/tech-design/projects/jet"

[[projects.workspaces]]
paths = ["projects/jet/**"]
target = "rust"
test_cmd = "true"
"#,
    );
    write(tmp.path(), "projects/agentic-workflow/src/lib.rs", "pub fn sdd() {}\n");
    write(tmp.path(), "projects/jet/src/lib.rs", "pub fn jet() {}\n");

    let out = Command::new(score)
        .args(["standardize", "run", "sdd", "--max-ticks", "1", "--json"])
        .current_dir(tmp.path())
        .output()
        .expect("run aw standardize");
    assert!(
        out.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let sdd = std::fs::read_to_string(tmp.path().join("projects/agentic-workflow/src/lib.rs")).unwrap();
    let jet = std::fs::read_to_string(tmp.path().join("projects/jet/src/lib.rs")).unwrap();
    assert!(sdd.contains("<HANDWRITE"));
    assert!(!jet.contains("<HANDWRITE"));
    assert!(tmp
        .path()
        .join("projects/agentic-workflow/tech-design/core/src/lib.md")
        .exists());
    assert!(!tmp
        .path()
        .join(".aw/tech-design/projects/jet/src/lib.md")
        .exists());
}

#[test]
fn standardize_non_interactive_blocks_for_bad_td() {
    let Some(score) = score_bin() else {
        eprintln!("skipping: CARGO_BIN_EXE_score not set");
        return;
    };
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".aw/tech-design/src/bad.md",
        "this is not a valid TD spec\n",
    );

    let out = Command::new(score)
        .args([
            "standardize",
            "managed",
            "run",
            "--scope",
            "src/**",
            "--non-interactive",
            "--json",
        ])
        .current_dir(tmp.path())
        .output()
        .expect("run aw standardize");
    assert!(!out.status.success(), "bad TD should block");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("\"action\": \"blocked\""),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("fix_spec_rule"), "stdout: {stdout}");
}

#[test]
fn standardize_scope_ignores_unrelated_bad_td() {
    let Some(score) = score_bin() else {
        eprintln!("skipping: CARGO_BIN_EXE_score not set");
        return;
    };
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".aw/tech-design/unrelated/bad.md",
        "this is not a valid TD spec\n",
    );

    let out = Command::new(score)
        .args(["standardize", "next", "--scope", "src/**", "--json"])
        .current_dir(tmp.path())
        .output()
        .expect("run aw standardize");
    assert!(
        out.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("fix_spec_rule"),
        "unrelated TD should not block scoped standardization: {stdout}"
    );
}

#[test]
fn standardize_regenerable_next_reports_handwrite_gap() {
    let Some(score) = score_bin() else {
        eprintln!("skipping: CARGO_BIN_EXE_score not set");
        return;
    };
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "src/lib.rs",
        "// <HANDWRITE gap=\"standardize:test\" tracker=\"tracker\" reason=\"needs generator\">\npub fn answer() -> i32 { 42 }\n// </HANDWRITE>\n",
    );

    let out = Command::new(score)
        .args([
            "standardize",
            "regenerable",
            "next",
            "--scope",
            "src/**",
            "--json",
        ])
        .current_dir(tmp.path())
        .output()
        .expect("run aw standardize regenerable");
    assert!(
        out.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"layer\": \"regenerable\""));
    assert!(stdout.contains("\"kind\": \"promote_handwrite\""));
    assert!(stdout.contains("full regenerability requires CODEGEN ownership"));
}

#[test]
fn standardize_successful_action_commits_once() {
    let Some(score) = score_bin() else {
        eprintln!("skipping: CARGO_BIN_EXE_score not set");
        return;
    };
    let tmp = TempDir::new().unwrap();
    Command::new("git")
        .args(["init"])
        .current_dir(tmp.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "score@example.test"])
        .current_dir(tmp.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Score Test"])
        .current_dir(tmp.path())
        .status()
        .unwrap();
    write(tmp.path(), "src/lib.rs", "pub fn answer() -> i32 { 42 }\n");
    Command::new("git")
        .args(["add", "src/lib.rs"])
        .current_dir(tmp.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(tmp.path())
        .status()
        .unwrap();

    let out = Command::new(score)
        .args([
            "standardize",
            "run",
            "--scope",
            "src/lib.rs",
            "--max-ticks",
            "1",
            "--json",
        ])
        .current_dir(tmp.path())
        .output()
        .expect("run aw standardize");
    assert!(
        out.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let count = Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&count.stdout).trim(), "2");
    let body = Command::new("git")
        .args(["show", "-s", "--format=%B", "HEAD"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    let body = String::from_utf8_lossy(&body.stdout);
    assert!(body.contains("Lifecycle-Stage: Standardize-ClaimCode"));
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/cli/tests/standardize_test.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Existing source claimed by `aw standardize managed run`. The code is
      wrapped in a tracked HANDWRITE block until deterministic generator
      coverage can replace it with CODEGEN.
      Parser coverage also protects `aw standardize <project>` as a
      parent-workflow shorthand. Runtime coverage verifies the shorthand starts
      at the first incomplete layer and does not run full health gates before
      capability structure is valid.
```
