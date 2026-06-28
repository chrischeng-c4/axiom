// SPEC-MANAGED: projects/meter/tech-design/logic/external-contracts.md#meter-embedded-profiler-api
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec meter-embedded-profiler-api
// @capability runtime-resource-attribution
// @claim embedded-profiler-api
// @contract embedded-profiler-api
// @category performance
// @required_for_production false
// @command cargo test -p meter performance::profiler
// AW-EC-END

// Contract: embedded profiler tests pass
// Contract: RSS snapshot, phase breakdown, and profile result contracts remain available
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn meter_embedded_profiler_api() {
    let command = "cargo test -p meter performance::profiler";
    let id = "meter-embedded-profiler-api";
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
