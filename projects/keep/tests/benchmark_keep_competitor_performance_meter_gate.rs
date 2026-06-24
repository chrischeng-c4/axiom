// SPEC-MANAGED: projects/keep/external-contracts/competitor-performance/efficiency/perf-gate.md#keep-competitor-performance-meter-gate
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec keep-competitor-performance-meter-gate
// @capability competitor-performance
// @claim vat-meter-runtime-gate
// @contract keep-meter-performance-report
// @category efficiency
// @required_for_production true
// @command cd projects/keep && ../../target/debug/vat run meter-efficiency
// AW-EC-END

// Contract: meter owns the pass/fail evidence for Keep's performance-relevant API and engine gate.
// Contract: The gate runs inside vat so report artifacts and transient state do not mutate the host checkout.
// Contract: Redis/Dragonfly comparison remains dogfood until external peer services are required by the EC.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn keep_competitor_performance_meter_gate() {
    let command = "cd projects/keep && ../../target/debug/vat run meter-efficiency";
    let id = "keep-competitor-performance-meter-gate";
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
