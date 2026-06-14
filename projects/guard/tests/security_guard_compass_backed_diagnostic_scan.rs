// SPEC-MANAGED: projects/guard/tech-design/semantic/guard-ec-static-security-smoke.md#guard-compass-backed-diagnostic-scan
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec guard-compass-backed-diagnostic-scan
// @capability static-security-scan
// @claim compass-backed-diagnostic-scan
// @contract compass-backed-diagnostic-scan
// @category security
// @required_for_production true
// @command CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard scan::tests::detects_javascript_eval_as_security_finding
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the manifest command authoritative"]
fn guard_compass_backed_diagnostic_scan() {
    panic!("AW EC placeholder for guard-compass-backed-diagnostic-scan");
}
// CODEGEN-END
