// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-competitor-feature-search-breadth
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-competitor-feature-search-breadth
// @capability competitor-feature-parity
// @claim search-feature-breadth
// @contract competitor-feature-search-breadth
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test api_e2e --test vector_e2e --test hash_hamming --test collapse_nested -- --nocapture
// AW-EC-END

// Contract: The API, vector, hash, duplicate, and nested search surfaces execute correctly across the replacement feature set.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_competitor_feature_search_breadth() {
    let command =
        "cargo test -p lumen --test api_e2e --test vector_e2e --test hash_hamming --test collapse_nested -- --nocapture";
    let id = "lumen-claim-competitor-feature-search-breadth";
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
