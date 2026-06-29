// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-duplicates-group-by
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-duplicates-group-by
// @capability duplicate-nested-search
// @claim duplicates-group-by
// @contract duplicates-group-by
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test api_e2e duplicates_finds_groups -- --exact --nocapture
// AW-EC-END

// Contract: Duplicate detection returns groups of external IDs sharing a field value.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_duplicates_group_by() {
    let command =
        "cargo test -p lumen --test api_e2e duplicates_finds_groups -- --exact --nocapture";
    let id = "lumen-claim-duplicates-group-by";
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
