// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-nested-collapse
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-nested-collapse
// @capability duplicate-nested-search
// @claim nested-group-has-child-collapse
// @contract nested-group-has-child-collapse
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test collapse_nested -- --nocapture
// AW-EC-END

// Contract: Nested has_child/group/collapse behavior passes the data-table search tests.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_nested_collapse() {
    let command = "cargo test -p lumen --test collapse_nested -- --nocapture";
    let id = "lumen-claim-nested-collapse";
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
