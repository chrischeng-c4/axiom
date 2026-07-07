// SPEC-MANAGED: projects/arena/tech-design/logic/external-contracts.md#arena-vat-managed-comparison-runner
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec arena-vat-managed-comparison-runner
// @capability vat-runner-integration
// @claim vat-managed-comparison-runner
// @contract vat-managed-comparison-runner
// @category stability
// @required_for_production true
// @command cargo test -p arena
// AW-EC-END

// Contract: arena remains runnable as a vat runner
// Contract: arena stays protocol-agnostic and leaves environment setup to vat
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn arena_vat_managed_comparison_runner() {
    let command = "cargo test -p arena";
    let id = "arena-vat-managed-comparison-runner";
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
