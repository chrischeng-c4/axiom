// SPEC-MANAGED: projects/rig/tech-design/logic/external-contracts.md#rig-floor-ratchet-pin-gates
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec rig-floor-ratchet-pin-gates
// @capability load-pins
// @claim floor-and-ratchet-pin-gates
// @contract floor-and-ratchet-pin-gates
// @category performance
// @required_for_production false
// @command cargo test -p rig
// AW-EC-END

// Contract: pin gate tests pass
// Contract: floor and ratchet baseline semantics remain covered
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn rig_floor_ratchet_pin_gates() {
    let command = "cargo test -p rig";
    let id = "rig-floor-ratchet-pin-gates";
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
