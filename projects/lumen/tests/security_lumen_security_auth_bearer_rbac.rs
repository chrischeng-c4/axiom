// SPEC-MANAGED: projects/lumen/tech-design/logic/external-contracts.md#lumen-security-auth-bearer-rbac
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-security-auth-bearer-rbac
// @capability security-auth
// @contract security-auth-bearer-rbac-enforced
// @category security
// @command cargo test -p lumen --test auth_e2e --test authz_matrix_e2e -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the manifest command authoritative"]
fn lumen_security_auth_bearer_rbac() {
    panic!("AW EC placeholder for lumen-security-auth-bearer-rbac");
}
// CODEGEN-END
