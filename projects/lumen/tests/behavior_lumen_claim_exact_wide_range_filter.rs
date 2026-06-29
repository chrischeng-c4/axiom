// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-exact-wide-range-filter
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-exact-wide-range-filter
// @capability exact-filter-search
// @claim wide-range-filter-index-on-disk-sorted-value-range
// @contract exact-wide-range-filter
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test perf_gate_vs_db -- --nocapture
// AW-EC-END

// Contract: Wide range filters over sorted disk-backed values pass the exact/filter gate.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_exact_wide_range_filter() {
    let command = "cargo test -p lumen --test perf_gate_vs_db -- --nocapture";
    let id = "lumen-claim-exact-wide-range-filter";
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
