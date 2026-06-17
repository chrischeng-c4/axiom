// SPEC-MANAGED: projects/guard/tech-design/semantic/guard-ec-static-security-smoke.md#guard-baseline-static-policy
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec guard-baseline-static-policy
// @capability security-policy-profile
// @claim baseline-static-policy
// @contract baseline-static-policy
// @category security
// @required_for_production true
// @command CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard detects_javascript_eval_as_security_finding
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the manifest command authoritative"]
fn guard_baseline_static_policy() {
    panic!("AW EC placeholder for guard-baseline-static-policy");
}
// CODEGEN-END
