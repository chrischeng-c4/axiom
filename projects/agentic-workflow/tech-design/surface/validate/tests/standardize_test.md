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
````rust
//! Integration tests for `aw standardize`.

use agentic_workflow::cli::Commands;
use clap::{CommandFactory, Parser};
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

fn aw_bin() -> Option<String> {
    std::env::var("CARGO_BIN_EXE_aw").ok()
}

fn aw_bin_for_regression() -> Option<String> {
    aw_bin().or_else(|| {
        let mut path = std::env::current_exe().ok()?;
        path.pop();
        path.pop();
        path.push(if cfg!(windows) { "aw.exe" } else { "aw" });
        path.is_file().then(|| path.to_string_lossy().into_owned())
    })
}

fn write_traceability_fixture(root: &std::path::Path, with_capability_ref: bool) {
    write(
        root,
        ".aw/config.toml",
        r#"
[[projects]]
name = "demo"
path = "."
td_path = ".aw/tech-design/demo"
cap_path = "README.md"
label = "project:demo"

[[projects.workspaces]]
name = "demo"
paths = ["src/**"]
target = "python"
test_cmd = "true"
"#,
    );
    write(
        root,
        "README.md",
        r#"# demo

## Demo Capability

| Field | Value |
|---|---|
| ID | demo-capability |
| Root WI | - |
| Status | verified |
| Promise | Provide demo behavior. |
| Required Verification | smoke |
| Gate Inventory | - |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Demo closure | epic | - | implemented | verified | smoke | true |
"#,
    );
    write(
        root,
        "src/app.py",
        "# SPEC-MANAGED: .aw/tech-design/demo/app.md#changes\n# CODEGEN-BEGIN\ndef handle():\n    return 1\n# CODEGEN-END\n",
    );
    let refs = if with_capability_ref {
        r#"capability_refs:
  - id: demo-capability
    role: primary
    gap: demo-closure
    claim: demo-closure
    coverage: full
"#
    } else {
        ""
    };
    write(
        root,
        ".aw/tech-design/demo/app.md",
        &format!(
            r#"---
id: demo-td
{refs}---

# Demo TD

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: demo-logic
nodes: []
edges: []
---
flowchart TD
  A --> B
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: src/app.py
    action: modify
    section: logic
    impl_mode: hand-written
```
"#
        ),
    );
}

fn append_ec_case(root: &std::path::Path) {
    let path = root.join(".aw/tech-design/demo/app.md");
    let mut content = std::fs::read_to_string(&path).unwrap();
    content.push_str(
        r#"

## E2E
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: demo-ec-smoke
    capability_id: demo-capability
    contract_id: demo-closure
    category: behavior
    command: "true"
    assertions:
      - demo behavior remains available
```
"#,
    );
    std::fs::write(path, content).unwrap();
}

fn write_llms_semantic_td(root: &std::path::Path) {
    write(
        root,
        ".aw/tech-design/demo/semantic/demo-.md",
        r#"---
id: demo-agent-context
capability_refs:
  - id: demo-capability
    role: primary
    gap: demo-closure
    claim: demo-closure
    coverage: full
---

# Demo Agent Context

## Schema
<!-- type: schema lang: markdown -->

```markdown
llms.txt exposes TD-first project context for agents.
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: llms.txt
    action: modify
    section: schema
    impl_mode: codegen
```
"#,
    );
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
    let semantic = standardize.find_subcommand("semantic").expect("semantic");
    semantic.find_subcommand("report").expect("semantic report");
    semantic.find_subcommand("next").expect("semantic next");
    semantic.find_subcommand("run").expect("semantic run");
    let traceability = standardize
        .find_subcommand("traceability")
        .expect("traceability");
    traceability
        .find_subcommand("report")
        .expect("traceability report");
    traceability
        .find_subcommand("next")
        .expect("traceability next");
    traceability
        .find_subcommand("run")
        .expect("traceability run");
    assert!(standardize.find_subcommand("capability").is_none());
    assert!(standardize.find_subcommand("regenerable").is_none());
    // Legacy top-level aliases (`report`/`codegen`/`next`/`run`) have been
    // removed; only the canonical takeover subcommands remain.
    assert!(standardize.find_subcommand("report").is_none());
    assert!(standardize.find_subcommand("codegen").is_none());
    assert!(standardize.find_subcommand("next").is_none());
    assert!(standardize.find_subcommand("run").is_none());
}

#[test]
fn standardize_accepts_project_option_without_layer_subcommand() {
    let parsed = Cli::try_parse_from(["aw", "standardize", "--project", "cap"])
        .expect("standardize project option parses");
    let Commands::Standardize(args) = parsed.command else {
        panic!("expected standardize command");
    };
    assert_eq!(args.project.as_deref(), Some("cap"));
    assert!(args.command.is_none());
}

#[test]
fn standardize_project_shorthand_starts_at_capability_layer() {
    let Some(score) = aw_bin() else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
        return;
    };
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".aw/config.toml",
        r#"
[[projects]]
name = "cap"
path = "."
td_path = ".aw/tech-design/cap"
cap_path = "README.md"
label = "project:cap"

[[projects.workspaces]]
name = "cap"
paths = ["src/**"]
target = "rust"
test_cmd = "false"
"#,
    );
    write(tmp.path(), "README.md", "# cap\n\nNo capability map yet.\n");
    write(tmp.path(), "src/lib.rs", "pub fn cap() {}\n");

    let out = Command::new(score)
        .args(["standardize", "--project", "cap"])
        .current_dir(tmp.path())
        .output()
        .expect("run aw standardize project option");
    assert!(
        !out.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("standardize parent JSON");
    assert_eq!(json["schema_version"], "aw.cli.v1");
    assert_eq!(json["status"], "blocked");
    assert_eq!(json["layer"], "capability");
    assert_eq!(json["next"]["kind"], "blocked");
    assert!(json["next"].get("command").is_none());
    assert_eq!(json["completion"]["workflow_complete"], false);
    assert_eq!(json["completion"]["requires_hitl"], false);
    assert_eq!(json["health"]["test_gates"]["evaluated"], false);
}

#[test]
fn standardize_project_health_gate_verifies_ec_cases() {
    let Some(score) = aw_bin_for_regression() else {
        eprintln!("skipping: aw test binary not available");
        return;
    };
    let tmp = TempDir::new().unwrap();
    write_traceability_fixture(tmp.path(), true);
    append_ec_case(tmp.path());
    for args in [
        vec!["init"],
        vec!["config", "user.email", "aw@example.test"],
        vec!["config", "user.name", "AW Test"],
        vec!["add", "."],
        vec!["commit", "-m", "init"],
    ] {
        let status = Command::new("git")
            .args(args)
            .current_dir(tmp.path())
            .status()
            .expect("run git setup");
        assert!(status.success(), "git setup failed");
    }

    let ec_gen = Command::new(&score)
        .args(["ec", "gen", "--project", "demo"])
        .current_dir(tmp.path())
        .output()
        .expect("run aw ec gen");
    assert!(
        ec_gen.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&ec_gen.stdout),
        String::from_utf8_lossy(&ec_gen.stderr)
    );
    for args in [vec!["add", "."], vec!["commit", "-m", "ec gen"]] {
        let status = Command::new("git")
            .args(args)
            .current_dir(tmp.path())
            .status()
            .expect("commit generated EC files");
        assert!(status.success(), "git EC commit failed");
    }
    let managed = Command::new(&score)
        .args([
            "standardize",
            "managed",
            "run",
            "--project",
            "demo",
            "--non-interactive",
            "--max-ticks",
            "1",
            "--json",
        ])
        .current_dir(tmp.path())
        .output()
        .expect("run aw standardize managed");
    assert!(
        managed.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&managed.stdout),
        String::from_utf8_lossy(&managed.stderr)
    );
    write_llms_semantic_td(tmp.path());

    let out = Command::new(score)
        .args(["standardize", "--project", "demo"])
        .current_dir(tmp.path())
        .output()
        .expect("run aw standardize health gate");
    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("standardize health JSON");

    assert_eq!(
        json["layer"],
        "health",
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(json["health"]["ec_status"], "passed");
    assert_eq!(json["health"]["ec_verify_evaluated"].as_bool(), Some(true));
    assert_eq!(json["health"]["ec"]["command_count"].as_u64(), Some(1));
    assert_eq!(json["health"]["ec"]["passed_count"].as_u64(), Some(1));
}

// The previous `standardize_codegen_reports_handwrite_blockers` test exercised
// the legacy `aw standardize codegen` alias. That alias has been removed.

#[test]
fn standardize_traceability_report_next_and_run_surface_blockers() {
    let Some(score) = aw_bin() else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
        return;
    };
    let tmp = TempDir::new().unwrap();
    write_traceability_fixture(tmp.path(), false);

    let report = Command::new(&score)
        .args([
            "standardize",
            "traceability",
            "report",
            "--project",
            "demo",
            "--json",
        ])
        .current_dir(tmp.path())
        .output()
        .expect("run aw standardize traceability report");
    assert!(
        report.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&report.stdout),
        String::from_utf8_lossy(&report.stderr)
    );
    let report_json: serde_json::Value =
        serde_json::from_slice(&report.stdout).expect("traceability report JSON");
    assert_eq!(report_json["project"], "demo");
    assert!(report_json.get("command_traceability").is_some());
    assert!(report_json["blocker_count"].as_u64().unwrap() > 0);

    let next = Command::new(&score)
        .args([
            "standardize",
            "traceability",
            "next",
            "--project",
            "demo",
            "--json",
        ])
        .current_dir(tmp.path())
        .output()
        .expect("run aw standardize traceability next");
    assert!(next.status.success());
    let next_json: serde_json::Value =
        serde_json::from_slice(&next.stdout).expect("traceability next JSON");
    assert_eq!(next_json["next_action"]["kind"], "blocked");
    assert_eq!(
        next_json["next_action"]["id"],
        "source_block_td_no_capability_ref"
    );
    assert_eq!(
        next_json["mainthread_task"]["decision_required"],
        "attach_source_or_cb_edge_to_capability_td"
    );
    assert!(next_json["agent_prompt"]
        .as_str()
        .unwrap()
        .contains("Resolve exactly one AW traceability blocker"));

    let run = Command::new(&score)
        .args([
            "standardize",
            "traceability",
            "run",
            "--project",
            "demo",
            "--non-interactive",
            "--json",
        ])
        .current_dir(tmp.path())
        .output()
        .expect("run aw standardize traceability run");
    assert!(!run.status.success());
    let run_json: serde_json::Value =
        serde_json::from_slice(&run.stdout).expect("traceability run JSON");
    assert_eq!(run_json["action"], "blocked");
    assert_eq!(run_json["layer"], "traceability");
    assert_eq!(
        run_json["invoke"]["command"],
        "aw standardize traceability next --project demo --json"
    );
    assert!(run_json["mainthread_task"]["success_criteria"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item.as_str().unwrap().contains("source/CB block resolves")));
}

#[test]
fn standardize_run_claims_mixed_language_repo() {
    let Some(score) = aw_bin() else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
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
fn standardize_run_accepts_project_option_from_config() {
    let Some(score) = aw_bin() else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
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
    write(
        tmp.path(),
        "projects/agentic-workflow/src/lib.rs",
        "pub fn sdd() {}\n",
    );
    write(tmp.path(), "projects/jet/src/lib.rs", "pub fn jet() {}\n");

    let out = Command::new(score)
        .args([
            "standardize",
            "managed",
            "run",
            "--project",
            "agentic-workflow",
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

    let sdd =
        std::fs::read_to_string(tmp.path().join("projects/agentic-workflow/src/lib.rs")).unwrap();
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
    let Some(score) = aw_bin() else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
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
    let Some(score) = aw_bin() else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
        return;
    };
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".aw/tech-design/unrelated/bad.md",
        "this is not a valid TD spec\n",
    );

    let out = Command::new(score)
        .args([
            "standardize",
            "managed",
            "next",
            "--scope",
            "src/**",
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
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("fix_spec_rule"),
        "unrelated TD should not block scoped standardization: {stdout}"
    );
}

#[test]
fn standardize_successful_action_commits_once() {
    let Some(score) = aw_bin() else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
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
            "managed",
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
