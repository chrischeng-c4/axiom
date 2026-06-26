// SPEC-MANAGED: projects/arena/tech-design/logic/external-contracts.md#arena-sequential-target-fanout-measurement
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec arena-sequential-target-fanout-measurement
// @capability n-target-comparison-runner
// @claim sequential-target-fanout-and-measurement
// @contract sequential-target-fanout-and-measurement
// @category benchmark
// @required_for_production false
// @command cargo test -p arena
// AW-EC-END

// Contract: arena pipeline tests pass
// Contract: target fanout and measurement remain covered by the local suite
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn arena_sequential_target_fanout_measurement() {
    let command = "cargo test -p arena";
    let id = "arena-sequential-target-fanout-measurement";
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
