// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-security-bearer-auth
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-security-bearer-auth
// @capability security-hardening
// @claim bearer-token-auth-lumen-auth
// @contract security-bearer-auth
// @category security
// @required_for_production true
// @command cargo test -p lumen --test auth_e2e --test authz_matrix_e2e -- --nocapture
// AW-EC-END

// Contract: Bearer-token auth rejects invalid callers and accepts valid tokens under LUMEN_AUTH=required.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_security_bearer_auth() {
    let command = "cargo test -p lumen --test auth_e2e --test authz_matrix_e2e -- --nocapture";
    let id = "lumen-claim-security-bearer-auth";
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
