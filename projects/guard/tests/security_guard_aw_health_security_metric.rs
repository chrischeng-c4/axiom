// SPEC-MANAGED: projects/guard/tech-design/semantic/guard-ec-static-security-smoke.md#guard-aw-health-security-metric
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec guard-aw-health-security-metric
// @capability security-ec-profile
// @claim aw-health-security-metric
// @contract aw-health-security-metric
// @category security
// @required_for_production true
// @command ./target/debug/aw ec check --project guard
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the manifest command authoritative"]
fn guard_aw_health_security_metric() {
    panic!("AW EC placeholder for guard-aw-health-security-metric");
}
// CODEGEN-END
