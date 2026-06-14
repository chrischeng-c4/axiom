// SPEC-MANAGED: projects/guard/tech-design/semantic/guard-ec-static-security-smoke.md#guard-static-scan-clean-report
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec guard-static-scan-clean-report
// @capability static-security-scan
// @claim json-report-envelope
// @contract guard-report-clean-static-scan
// @category security
// @required_for_production true
// @command target/debug/guard scan projects/guard --compact --no-persist
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the manifest command authoritative"]
fn guard_static_scan_clean_report() {
    panic!("AW EC placeholder for guard-static-scan-clean-report");
}
// CODEGEN-END
