// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-competitor-performance-external-comparison
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-competitor-performance-external-comparison
// @capability competitor-performance
// @claim external-pg-and-opensearch-arena-comparison
// @contract competitor-performance-external-comparison
// @category efficiency
// @required_for_production true
// @command cd projects/lumen && ../../target/debug/vat run ec-efficiency-meter
// AW-EC-END

// Contract: The vat efficiency runner executes the Postgres/OpenSearch comparison path and resource attribution gate.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_competitor_performance_external_comparison() {
    let command = "cd projects/lumen && ../../target/debug/vat run ec-efficiency-meter";
    let id = "lumen-claim-competitor-performance-external-comparison";
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
