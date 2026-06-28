// SPEC-MANAGED: projects/meter/tech-design/logic/external-contracts.md#meter-json-default-report-envelope-and-findings
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec meter-json-default-report-envelope-and-findings
// @capability agent-use-first-cli
// @claim json-default-report-envelope-and-findings
// @contract json-default-report-envelope-and-findings
// @category behavior
// @required_for_production true
// @command cargo test -p meter report::
// AW-EC-END

// Contract: report envelope and finding model tests pass
// Contract: default agent-facing report shape remains deterministic
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn meter_json_default_report_envelope_and_findings() {
    let command = "cargo test -p meter report::";
    let id = "meter-json-default-report-envelope-and-findings";
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
