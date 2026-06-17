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

// Contract: guard can fold a vat-isolated runner into its report
// Contract: isolated evidence is visible without persisting guard state
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn guard_vat_isolated_security_runner() {
    let command = "target/debug/guard scan projects/guard --compact --no-persist --vat-runner guard-security-smoke";
    let id = "guard-vat-isolated-security-runner";
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
