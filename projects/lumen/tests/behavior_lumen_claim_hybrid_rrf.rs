// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-hybrid-rrf
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-hybrid-rrf
// @capability hybrid-search
// @claim rrf-fusion-node-planner-integration
// @contract hybrid-rrf-planner
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test hybrid_rrf -- --nocapture
// AW-EC-END

// Contract: Lexical and semantic result lists are fused through RRF while preserving per-leg filters.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_hybrid_rrf() {
    let command = "cargo test -p lumen --test hybrid_rrf -- --nocapture";
    let id = "lumen-claim-hybrid-rrf";
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
