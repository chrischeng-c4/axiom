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

// Contract: security-lint profile runs on the guard source tree
// Contract: security-impacting lint remains part of the guard policy profile
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn guard_security_lint_policy() {
    let command =
        "target/debug/guard scan projects/guard --profile security-lint --compact --no-persist";
    let id = "guard-security-lint-policy";
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
