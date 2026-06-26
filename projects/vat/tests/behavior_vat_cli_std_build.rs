// SPEC-MANAGED: projects/vat/tech-design/interfaces/cli/migrate-upgrade-and-report-issue-to-the-shared-cli-std-crate.md#vat-cli-std-build
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-cli-std-build
// @capability agent-native-gpu-native-dev-containers
// @claim vat-cli-std-build
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo build -p vat --no-default-features
// AW-EC-END

// Contract: vat compiles with and without default features; the hand-rolled modules are gone.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_cli_std_build() {
    let command = "cargo build -p vat --no-default-features";
    let id = "vat-cli-std-build";
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
