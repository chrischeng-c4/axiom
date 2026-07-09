// SPEC-MANAGED: apps/preview/external-contracts/behavior/router-target-ec.md#router-target-ec
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec router-target-ec
// @capability preview-external-contracts
// @claim router-target-ec
// @contract router-target-ec
// @category behavior
// @required_for_production true
// @command cargo test -p preview --test router_contract
// AW-EC-END

// Contract: Cookie target resolution maps only through a known RouteBinding.
// Contract: Header target selection can override cookie selection for API/mobile/manual clients.
// Contract: Unknown targets do not guess or synthesize namespaces.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn router_target_ec() {
    let command = "cargo test -p preview --test router_contract";
    let id = "router-target-ec";
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
