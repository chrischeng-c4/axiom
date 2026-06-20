// SPEC-MANAGED: projects/lumen/external-contracts/search/security/access-control.md#lumen-search-security-result-leak
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-search-security-result-leak
// @capability security-auth
// @claim score-confidentiality
// @contract search-security-result-leak
// @category security
// @required_for_production true
// @command cargo test -p lumen --test coverage_gaps_e2e search_security_result_leak_respects_collection_boundaries -- --nocapture
// AW-EC-END

// Contract: C3: relevance scores and hit existence do not leak documents across collection boundaries; RBAC denial coverage remains pinned by the authz matrix case.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_search_security_result_leak() {
    let command =
        "cargo test -p lumen --test coverage_gaps_e2e search_security_result_leak_respects_collection_boundaries -- --nocapture";
    let id = "lumen-search-security-result-leak";
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
