// SPEC-MANAGED: projects/lumen/external-contracts/security/auth-bearer-rbac.md#lumen-security-auth-bearer-rbac
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-security-auth-bearer-rbac
// @capability security-auth
// @claim bearer-token-auth-lumen-auth
// @contract bearer-token-auth-lumen-auth
// @category security
// @required_for_production false
// @command cargo test -p lumen --test auth_e2e --test authz_matrix_e2e -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the manifest command authoritative"]
fn lumen_security_auth_bearer_rbac() {
    panic!("AW EC placeholder for lumen-security-auth-bearer-rbac");
}
// CODEGEN-END
