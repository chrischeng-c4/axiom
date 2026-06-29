// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-elastic-disk-tier
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-elastic-disk-tier
// @capability elastic-scale
// @claim ram-hot-disk-all-columnar-mmap-segment-tier-embedded-single-node-log
// @contract elastic-disk-tier
// @category efficiency
// @required_for_production true
// @command target/debug/meter test -- -p lumen --test disk_scale_proof -- --ignored
// AW-EC-END

// Contract: The disk-scale proof keeps the full corpus on disk-backed segments while bounded hot state remains in memory.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_elastic_disk_tier() {
    let command = "target/debug/meter test -- -p lumen --test disk_scale_proof -- --ignored";
    let id = "lumen-claim-elastic-disk-tier";
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
