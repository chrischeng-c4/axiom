// SPEC-MANAGED: projects/guard/tech-design/semantic/guard-ec-static-security-smoke.md#guard-security-lint-policy
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec guard-security-lint-policy
// @capability security-policy-profile
// @claim security-lint-policy
// @contract security-lint-policy
// @category security
// @required_for_production true
// @command target/debug/guard scan projects/guard --profile security-lint --compact --no-persist
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the manifest command authoritative"]
fn guard_security_lint_policy() {
    panic!("AW EC placeholder for guard-security-lint-policy");
}
// CODEGEN-END
