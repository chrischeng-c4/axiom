// SPEC-MANAGED: projects/guard/tech-design/semantic/guard-ec-static-security-smoke.md#guard-vat-isolated-security-runner
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec guard-vat-isolated-security-runner
// @capability dynamic-security-evidence
// @claim vat-isolated-security-runner
// @contract vat-isolated-security-runner
// @category security
// @required_for_production true
// @command target/debug/guard scan projects/guard --compact --no-persist --vat-runner guard-security-smoke
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the manifest command authoritative"]
fn guard_vat_isolated_security_runner() {
    panic!("AW EC placeholder for guard-vat-isolated-security-runner");
}
// CODEGEN-END
