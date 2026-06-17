// SPEC-MANAGED: projects/guard/tech-design/semantic/guard-ec-static-security-smoke.md#guard-ec-security-evidence-command
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec guard-ec-security-evidence-command
// @capability security-ec-profile
// @claim ec-security-evidence-command
// @contract ec-security-evidence-command
// @category security
// @required_for_production true
// @command target/debug/guard scan projects/guard --profile security-lint --compact --no-persist --vat-runner guard-security-smoke --rig-scenario projects/guard/tests/rig/scenarios/security/guard_self_scan.toml --meter-target projects/guard --arena-command "target/debug/arena spec --compact"
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the manifest command authoritative"]
fn guard_ec_security_evidence_command() {
    panic!("AW EC placeholder for guard-ec-security-evidence-command");
}
// CODEGEN-END
