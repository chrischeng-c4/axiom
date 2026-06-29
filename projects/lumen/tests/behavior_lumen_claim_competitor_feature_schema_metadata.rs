// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-competitor-feature-schema-metadata
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-competitor-feature-schema-metadata
// @capability competitor-feature-parity
// @claim schema-and-metadata-breadth
// @contract competitor-feature-schema-metadata
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test drop_field_e2e --test reindex_stream_e2e --test stats_metadata_e2e -- --nocapture
// AW-EC-END

// Contract: Schema lifecycle, reindex/replay, and stats/metadata behavior pass the production conformance tests.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_competitor_feature_schema_metadata() {
    let command =
        "cargo test -p lumen --test drop_field_e2e --test reindex_stream_e2e --test stats_metadata_e2e -- --nocapture";
    let id = "lumen-claim-competitor-feature-schema-metadata";
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
