// SPEC-MANAGED: projects/arena/tech-design/logic/external-contracts.md#arena-report-envelope
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec arena-report-envelope
// @capability n-target-comparison-runner
// @claim arena-report-envelope
// @contract arena-report-envelope
// @category behavior
// @required_for_production true
// @command cargo test -p arena
// AW-EC-END

// Contract: arena report tests pass
// Contract: arena.report/1 remains the single JSON report contract
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn arena_report_envelope() {
    let command = "cargo test -p arena";
    let id = "arena-report-envelope";
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
