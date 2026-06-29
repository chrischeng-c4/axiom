// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-schema-stats-metadata
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-schema-stats-metadata
// @capability schema-ops-lifecycle
// @claim stats-metadata
// @contract schema-stats-metadata
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test stats_metadata_e2e -- --nocapture
// AW-EC-END

// Contract: Stats and per-field metadata match indexed data and byte attribution.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_schema_stats_metadata() {
    let command = "cargo test -p lumen --test stats_metadata_e2e -- --nocapture";
    let id = "lumen-claim-schema-stats-metadata";
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
