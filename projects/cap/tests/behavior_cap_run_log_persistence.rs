// SPEC-MANAGED: projects/cap/tech-design/logic/external-contracts.md#cap-run-log-persistence
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec cap-run-log-persistence
// @capability config-logging-and-reap-policy
// @claim run-log-persistence
// @contract run-log-persistence
// @category behavior
// @required_for_production true
// @command cargo test -p cap eventlog -- --nocapture
// AW-EC-END

// Contract: JSONL event log persistence roundtrips run records
// Contract: run evidence remains durable enough for agent diagnosis
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn cap_run_log_persistence() {
    let command = "cargo test -p cap eventlog -- --nocapture";
    let id = "cap-run-log-persistence";
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
