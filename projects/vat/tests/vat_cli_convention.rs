// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-tests-vat_cli_convention-rs.md#rust-source-unit
// CODEGEN-BEGIN
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
    let outline = Command::new(vat())
        .args(["llm", "--topic", "outline", "--format", "json"])
        .output()
        .unwrap();
    assert!(outline.status.success(), "llm outline json should exit 0");
    let stdout = String::from_utf8_lossy(&outline.stdout);
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
    let out = Command::new(vat())
        .args(["issue", "--help"])
        .output()
        .unwrap();
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
// CODEGEN-END
