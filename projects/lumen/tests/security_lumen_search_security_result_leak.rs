// SPEC-MANAGED: projects/lumen/external-contracts/search/security/access-control.md#lumen-search-security-result-leak
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-search-security-result-leak
// @capability search
// @claim score-confidentiality
// @contract search-security-result-leak
// @category security
// @required_for_production false
// @command cargo test -p lumen --test security_lumen_search_security_result_leak -- --ignored
// AW-EC-END

// Contract: GAP (C3): relevance scores and hit existence do not leak documents across collection / RBAC boundaries. Confidentiality contract + test not yet written.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_search_security_result_leak() {
    let command = "cargo test -p lumen --test security_lumen_search_security_result_leak -- --ignored";
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
