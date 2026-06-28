// SPEC-MANAGED: projects/meter/tech-design/logic/external-contracts.md#meter-benchmark-regression-api
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec meter-benchmark-regression-api
// @capability runtime-resource-attribution
// @claim benchmark-regression-api
// @contract benchmark-regression-api
// @category performance
// @required_for_production false
// @command cargo test -p meter benchmark::
// AW-EC-END

// Contract: benchmark tests pass
// Contract: adaptive benchmark and percentile contracts remain stable
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn meter_benchmark_regression_api() {
    let command = "cargo test -p meter benchmark::";
    let id = "meter-benchmark-regression-api";
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
