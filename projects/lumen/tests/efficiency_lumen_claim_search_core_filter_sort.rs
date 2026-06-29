// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-search-core-filter-sort
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-search-core-filter-sort
// @capability search-core
// @claim filter-sort-early-termination
// @contract search-core-filter-sort
// @category efficiency
// @required_for_production true
// @command cargo test -p lumen --test perf_gate_vs_db -- --nocapture
// AW-EC-END

// Contract: Filter/sort early-termination behavior is covered by the ratcheted database comparison gate.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_search_core_filter_sort() {
    let command = "cargo test -p lumen --test perf_gate_vs_db -- --nocapture";
    let id = "lumen-claim-search-core-filter-sort";
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
