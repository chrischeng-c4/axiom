// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-search-core-planner
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-search-core-planner
// @capability search-core
// @claim query-planner-boolean-eval-roaring-postings
// @contract search-core-planner
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test planner_diff -- --nocapture
// AW-EC-END

// Contract: The planner keeps boolean evaluation and roaring-posting behavior aligned with brute-force expectations.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_search_core_planner() {
    let command = "cargo test -p lumen --test planner_diff -- --nocapture";
    let id = "lumen-claim-search-core-planner";
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
