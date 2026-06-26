// SPEC-MANAGED: projects/vat/tech-design/interfaces/cli/migrate-upgrade-and-report-issue-to-the-shared-cli-std-crate.md#vat-cli-std-parity
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-cli-std-parity
// @capability agent-native-gpu-native-dev-containers
// @claim vat-cli-std-parity
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat cli_convention -- --nocapture
// AW-EC-END

// Contract: vat --help lists llm, upgrade, report-issue.
// Contract: vat report-issue --title X --dry-run prints the diagnostics body incl. the vat version and OS.
// Contract: vat upgrade --check exits cleanly.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_cli_std_parity() {
    let command = "cargo test -p vat cli_convention -- --nocapture";
    let id = "vat-cli-std-parity";
    let mut root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !root.join(".aw").is_dir() {
        assert!(
            root.pop(),
            "AW EC {id}: no .aw/ project root above {}",
            env!("CARGO_MANIFEST_DIR")
        );
    }
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(&root)
        .status()
        .unwrap_or_else(|e| panic!("AW EC {id}: failed to spawn `{command}`: {e}"));
    assert!(
        status.success(),
        "AW EC {id} FAILED (exit {:?}): {command}",
        status.code()
    );
}
// CODEGEN-END
