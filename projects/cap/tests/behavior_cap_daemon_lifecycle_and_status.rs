// SPEC-MANAGED: projects/cap/tech-design/logic/external-contracts.md#cap-daemon-lifecycle-and-status
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec cap-daemon-lifecycle-and-status
// @capability daemon-lifecycle-and-status
// @claim daemon-process-lifecycle
// @contract daemon-lifecycle-and-status
// @category behavior
// @required_for_production true
// @command cargo test -p cap daemon -- --nocapture && cargo test -p cap cli -- --nocapture
// AW-EC-END

// Contract: daemon lifecycle and liveness surfaces remain callable
// Contract: status and wait CLI surfaces use the daemon contract without blocking agent commands
// Contract: command leases stay isolated by process group
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn cap_daemon_lifecycle_and_status() {
    let command = "cargo test -p cap daemon -- --nocapture && cargo test -p cap cli -- --nocapture";
    let id = "cap-daemon-lifecycle-and-status";
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
