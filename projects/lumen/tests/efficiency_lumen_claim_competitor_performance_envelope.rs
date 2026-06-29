// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-competitor-performance-envelope
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-competitor-performance-envelope
// @capability competitor-performance
// @claim perf-gate-envelope-absolute-latency-throughput-floors
// @contract competitor-performance-envelope
// @category efficiency
// @required_for_production true
// @command cargo test -p lumen --test perf_gate -- --nocapture
// AW-EC-END

// Contract: Absolute latency and throughput floors stay within the ratcheted perf gate envelope.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_competitor_performance_envelope() {
    let command = "cargo test -p lumen --test perf_gate -- --nocapture";
    let id = "lumen-claim-competitor-performance-envelope";
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
