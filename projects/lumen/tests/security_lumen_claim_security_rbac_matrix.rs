// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-security-rbac-matrix
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-security-rbac-matrix
// @capability security-hardening
// @claim role-based-authz-matrix-per-route
// @contract security-rbac-matrix
// @category security
// @required_for_production true
// @command cargo test -p lumen --test authz_matrix_e2e --test api_e2e -- --nocapture
// AW-EC-END

// Contract: Per-route RBAC enforces read/write/admin permissions and bounds result/page sizes.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_security_rbac_matrix() {
    let command = "cargo test -p lumen --test authz_matrix_e2e --test api_e2e -- --nocapture";
    let id = "lumen-claim-security-rbac-matrix";
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
