// SPEC-MANAGED: projects/lumen/external-contracts/security-hardening/security/auth-bearer-rbac.md#lumen-security-hardening-auth-bearer-rbac
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-security-hardening-auth-bearer-rbac
// @capability security-hardening
// @claim bearer-token-auth-lumen-auth
// @contract bearer-token-auth-lumen-auth
// @category security
// @required_for_production false
// @command cargo test -p lumen --test auth_e2e --test authz_matrix_e2e -- --nocapture
// AW-EC-END

// Contract: Bearer-token auth rejects missing and invalid tokens when LUMEN_AUTH=required; accepts valid tokens.
// Contract: Per-route RBAC authz matrix enforces each token's role permissions on every API route (read vs write vs admin).
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_security_hardening_auth_bearer_rbac() {
    let command = "cargo test -p lumen --test auth_e2e --test authz_matrix_e2e -- --nocapture";
    let id = "lumen-security-hardening-auth-bearer-rbac";
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
