---
id: projects-vat-tests-vat_cli_convention-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves the binary smoke tests for vat's mandatory CLI convention verbs."
---

# Standardized projects/vat/tests/vat_cli_convention.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/tests/vat_cli_convention.rs`, captured as a rust-source-unit (td_ast) item-tree
during vat standardization onto the codegen ladder.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `cli_convention_help_lists_all_three` | projects/vat/tests/vat_cli_convention.rs | function | private | 15 |  |
| `cli_convention_llm_flags` | projects/vat/tests/vat_cli_convention.rs | function | private | 36 |  |
| `cli_convention_issue_create_dry_run` | projects/vat/tests/vat_cli_convention.rs | function | private | 60 |  |
| `cli_convention_issue_help_lists_verbs` | projects/vat/tests/vat_cli_convention.rs | function | private | 83 |  |
| `cli_convention_upgrade_check_exits_cleanly` | projects/vat/tests/vat_cli_convention.rs | function | private | 99 |  |
| `vat` | projects/vat/tests/vat_cli_convention.rs | function | private | 10 | vat() -> &'static str |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Binary smoke test for the mandatory CLI convention: every CLI ships
//! `llm`, `upgrade`, and `issue` (CONTRIBUTING.md).
//!
//! @command cargo test -p vat cli_convention -- --nocapture

use std::process::Command;

/// Path to the freshly-built `vat` binary (cargo stamps this for integration tests).
fn vat() -> &'static str {
    env!("CARGO_BIN_EXE_vat")
}

#[test]
fn cli_convention_help_lists_all_three() {
    let out = Command::new(vat()).arg("--help").output().unwrap();
    assert!(out.status.success(), "vat --help should succeed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    for verb in ["llm", "upgrade", "report-issue"] {
        if verb == "report-issue" {
            assert!(
                !stdout.contains(verb),
                "vat --help should not expose deprecated `{verb}`:\n{stdout}"
            );
        } else {
            assert!(
                stdout
                    .lines()
                    .any(|line| line.trim_start().starts_with(verb)),
                "vat --help is missing the mandatory `{verb}` verb:\n{stdout}"
            );
        }
    }
    assert!(
        stdout
            .lines()
            .any(|line| line.trim_start().starts_with("issue")),
        "vat --help is missing the mandatory `issue` verb:\n{stdout}"
    );
}

#[test]
fn cli_convention_llm_flags() {
    let out = Command::new(vat())
        .args(["llm", "--topic", "outline", "--format", "json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "llm outline json should exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("\"project\"") && stdout.contains("\"topics\""),
        "llm --format json should print the cli-std JSON shape:\n{stdout}"
    );

    let guide = Command::new(vat())
        .args(["llm", "--topic", "guide"])
        .output()
        .unwrap();
    assert!(guide.status.success(), "llm --topic guide should exit 0");
    assert!(
        String::from_utf8_lossy(&guide.stdout).contains("vat LLM Guide"),
        "llm --topic guide should print the detailed vat guide"
    );
}

#[test]
fn cli_convention_issue_create_dry_run() {
    let out = Command::new(vat())
        .args(["issue", "create", "--title", "smoke test", "--dry-run"])
        .output()
        .unwrap();
    assert!(out.status.success(), "issue create --dry-run should exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("## Diagnostics"),
        "dry-run body should carry the diagnostics block:\n{stdout}"
    );
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "dry-run body should include the vat version:\n{stdout}"
    );
    assert!(
        stdout.contains(std::env::consts::OS),
        "dry-run body should include the OS:\n{stdout}"
    );
}

#[test]
fn cli_convention_issue_help_lists_verbs() {
    let out = Command::new(vat()).args(["issue", "--help"]).output().unwrap();
    assert!(out.status.success(), "issue --help should exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    for verb in ["search", "view", "create"] {
        assert!(
            stdout
                .lines()
                .any(|line| line.trim_start().starts_with(verb)),
            "vat issue --help is missing `{verb}`:\n{stdout}"
        );
    }
}

#[test]
fn cli_convention_upgrade_check_exits_cleanly() {
    // `--check` reaches the network; with connectivity it prints current/latest,
    // offline it errors cleanly. Either way it must exit with a code (never a
    // panic / signal) and never modify the running binary.
    let out = Command::new(vat())
        .args(["upgrade", "--check"])
        .output()
        .unwrap();
    assert!(
        out.status.code().is_some(),
        "upgrade --check should exit cleanly, got {:?}",
        out.status
    );
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/tests/vat_cli_convention.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/tests/vat_cli_convention.rs` captured during vat
      standardization.
```
