// SPEC-MANAGED: projects/guard/tech-design/semantic/guard-ec-static-security-smoke.md#guard-cli-module-registration
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec guard-cli-module-registration
// @capability security-policy-profile
// @claim cli-module-registration
// @contract cli-module-registration
// @category behavior
// @required_for_production true
// @command CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard-cli registered_in_slice
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the manifest command authoritative"]
fn guard_cli_module_registration() {
    panic!("AW EC placeholder for guard-cli-module-registration");
}
// CODEGEN-END
