// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3780.md#dev-server-cli-contract
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec dev-server-cli-contract
// @capability dev-server-hmr
// @claim dev-server-cli-contract
// @contract dev-server-cli-contract
// @category behavior
// @required_for_production true
// @command cargo test -p jet --lib cli::e2e_command_contract_tests -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn dev_server_cli_contract() {
    let command = "cargo test -p jet --lib cli::e2e_command_contract_tests -- --nocapture";
    let id = "dev-server-cli-contract";
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
