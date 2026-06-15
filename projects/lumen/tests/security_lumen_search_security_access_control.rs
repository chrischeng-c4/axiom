// SPEC-MANAGED: projects/lumen/external-contracts/search/security/access-control.md#lumen-search-security-access-control
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-search-security-access-control
// @capability search
// @claim per-route-rbac-result-filtering
// @contract search-security-rbac-and-limit
// @category security
// @required_for_production true
// @command cargo test -p lumen --test authz_matrix_e2e --test api_e2e -- --nocapture
// AW-EC-END

// Contract: FILTERING: search over a collection the token cannot read returns 403; results never leak rows outside the caller's RBAC scope.
// Contract: PAGINATION: bulk/index requests over MAX_INDEX_ITEMS return 413; result pages are bounded (cursor), not unbounded.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_search_security_access_control() {
    let command = "cargo test -p lumen --test authz_matrix_e2e --test api_e2e -- --nocapture";
    let id = "lumen-search-security-access-control";
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
