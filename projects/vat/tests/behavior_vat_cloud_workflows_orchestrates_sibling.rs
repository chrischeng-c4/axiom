// SPEC-MANAGED: projects/vat/tech-design/logic/built-in-cloud-workflows-emulator.md#vat-cloud-workflows-orchestrates-sibling
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-cloud-workflows-orchestrates-sibling
// @capability agent-native-gpu-native-dev-containers
// @claim vat-cloud-workflows-orchestrates-sibling
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulator_workflows -- --nocapture --include-ignored
// AW-EC-END

// Contract: a vat.toml with preset = cloud-workflows alongside another emulator preset runs a workflow whose http step targets that sibling emulator's exported host, end to end.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_cloud_workflows_orchestrates_sibling() {
    let command =
        "cargo test -p vat --test vat_emulator_workflows -- --nocapture --include-ignored";
    let id = "vat-cloud-workflows-orchestrates-sibling";
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
