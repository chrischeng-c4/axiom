// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/standardize.md#demo-contract
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec demo-contract
// @capability unmapped
// @claim demo-contract
// @contract demo-contract
// @category behavior
// @required_for_production true
// @command cargo test -p demo demo_contract
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn demo_contract() {
    let command = "cargo test -p demo demo_contract";
    let id = "demo-contract";
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
