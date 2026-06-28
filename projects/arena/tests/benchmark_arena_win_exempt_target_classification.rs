// SPEC-MANAGED: projects/arena/tech-design/logic/external-contracts.md#arena-win-exempt-target-classification
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec arena-win-exempt-target-classification
// @capability ratio-ratchet-gates
// @claim win-exempt-target-classification
// @contract win-exempt-target-classification
// @category benchmark
// @required_for_production false
// @command cargo test -p arena
// AW-EC-END

// Contract: win, exempt, and target classification tests pass
// Contract: floor-dominated cells remain report-only when classified exempt
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn arena_win_exempt_target_classification() {
    let command = "cargo test -p arena";
    let id = "arena-win-exempt-target-classification";
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
