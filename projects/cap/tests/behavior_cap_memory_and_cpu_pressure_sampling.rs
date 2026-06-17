// SPEC-MANAGED: projects/cap/tech-design/logic/external-contracts.md#cap-memory-and-cpu-pressure-sampling
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec cap-memory-and-cpu-pressure-sampling
// @capability command-lease-throttling
// @claim memory-and-cpu-pressure-sampling
// @contract memory-and-cpu-pressure-sampling
// @category behavior
// @required_for_production true
// @command cargo test -p cap sampler -- --nocapture
// AW-EC-END

// Contract: sampler output is stable enough for daemon pressure decisions
// Contract: memory and CPU pressure readings remain available to lease admission
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn cap_memory_and_cpu_pressure_sampling() {
    let command = "cargo test -p cap sampler -- --nocapture";
    let id = "cap-memory-and-cpu-pressure-sampling";
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
