// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-lexical-bm25
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-lexical-bm25
// @capability lexical-search
// @claim bm25-ranking-and-analyzers
// @contract lexical-bm25-ranking-analyzers
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test perf_gate_vs_db -- --nocapture
// AW-EC-END

// Contract: BM25 ranking and analyzer behavior pass the ratcheted performance/conformance comparison.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_lexical_bm25() {
    let command = "cargo test -p lumen --test perf_gate_vs_db -- --nocapture";
    let id = "lumen-claim-lexical-bm25";
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
