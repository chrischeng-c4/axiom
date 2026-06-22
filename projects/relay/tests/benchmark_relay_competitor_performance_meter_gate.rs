// SPEC-MANAGED: projects/relay/external-contracts/competitor-performance/efficiency/perf-gate.md#relay-competitor-performance-meter-gate
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec relay-competitor-performance-meter-gate
// @capability competitor-performance
// @claim normalized-win-ratchet-decision-model
// @contract relay-meter-throughput-ratchet
// @category efficiency
// @required_for_production true
// @command cd projects/relay && ../../target/debug/vat run meter-perf
// AW-EC-END

// Contract: The normalized perf-gate ratchet fails on regression and must-beat loss.
// Contract: The small-scale append, broadcast, and work-queue lease/ack workloads complete correctly.
// Contract: The gate is executed by meter inside a vat workspace, not by a legacy arena-only dispatch path.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn relay_competitor_performance_meter_gate() {
    let command = "cd projects/relay && ../../target/debug/vat run meter-perf";
    let id = "relay-competitor-performance-meter-gate";
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
