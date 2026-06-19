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

// Contract: compass-backed scan tests detect security diagnostics
// Contract: guard preserves the static security engine integration
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn guard_compass_backed_diagnostic_scan() {
    let command = "CC=/usr/bin/cc PATH=\"$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin\" cargo test -p guard scan::tests::detects_javascript_eval_as_security_finding";
    let id = "guard-compass-backed-diagnostic-scan";
    let mut root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !root.join(".aw").is_dir() {
        assert!(
            root.pop(),
            "AW EC {id}: no .aw/ project root above {}",
            env!("CARGO_MANIFEST_DIR")
        );
    }
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(&root)
        .status()
        .unwrap_or_else(|e| panic!("AW EC {id}: failed to spawn `{command}`: {e}"));
    assert!(
        status.success(),
        "AW EC {id} FAILED (exit {:?}): {command}",
        status.code()
    );
}
// CODEGEN-END
