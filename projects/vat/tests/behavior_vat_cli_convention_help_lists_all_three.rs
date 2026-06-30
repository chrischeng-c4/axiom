// SPEC-MANAGED: projects/vat/tech-design/interfaces/cli/vat-upgrade-and-report-issue-subcommands-for-the-mandatory-cli-c.md#vat-cli-convention-help-lists-all-three
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-cli-convention-help-lists-all-three
// @capability agent-native-gpu-native-dev-containers
// @claim vat-cli-convention-help-lists-all-three
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_cli_convention -- --nocapture
// AW-EC-END

// Contract: `vat --help` output contains `llm`, `upgrade`, and `issue`.
// Contract: `vat upgrade --check` exits 0 and reports current vs latest without writing the binary (network-permitting; offline it errors cleanly, never panics).
// Contract: `vat issue create --title X --dry-run` prints a body containing the vat version and OS/arch and submits nothing.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_cli_convention_help_lists_all_three() {
    let command = "cargo test -p vat --test vat_cli_convention -- --nocapture";
    let id = "vat-cli-convention-help-lists-all-three";
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
