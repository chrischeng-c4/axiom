// SPEC-MANAGED: projects/rig/tech-design/logic/external-contracts.md#rig-open-loop-load-generator
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec rig-open-loop-load-generator
// @capability load-pins
// @claim open-loop-load-generator
// @contract open-loop-load-generator
// @category performance
// @required_for_production false
// @command cargo test -p rig
// AW-EC-END

// Contract: load generator tests pass
// Contract: open-loop load remains part of rig's public contract
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn rig_open_loop_load_generator() {
    let command = "cargo test -p rig";
    let id = "rig-open-loop-load-generator";
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
