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

// Contract: command exits zero
// Contract: stdout is a guard.report/1 JSON envelope
// Contract: summary.security_findings is zero for the guard source tree
// Contract: integrations.static_engine is compass
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn guard_static_scan_clean_report() {
    let command = "target/debug/guard scan projects/guard --compact --no-persist";
    let id = "guard-static-scan-clean-report";
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
