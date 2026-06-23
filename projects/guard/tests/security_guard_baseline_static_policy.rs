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

// Contract: baseline static policy maps compass diagnostics into guard findings
// Contract: policy severity normalization remains covered by guard tests
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn guard_baseline_static_policy() {
    let command = "CC=/usr/bin/cc PATH=\"$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin\" cargo test -p guard detects_javascript_eval_as_security_finding";
    let id = "guard-baseline-static-policy";
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
