// SPEC-MANAGED: projects/cap/tech-design/logic/external-contracts.md#cap-command-lease-throttling
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec cap-command-lease-throttling
// @capability command-lease-throttling
// @claim lease-admission-and-process-supervision
// @contract command-lease-throttling
// @category behavior
// @required_for_production true
// @command cargo test -p cap throttle -- --nocapture && cargo test -p cap sampler -- --nocapture
// AW-EC-END

// Contract: cap leases pause, resume, and kill under configured pressure thresholds
// Contract: release outcomes preserve structured kill diagnostics
// Contract: sampler output is stable enough for daemon pressure decisions
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn cap_command_lease_throttling() {
    let command =
        "cargo test -p cap throttle -- --nocapture && cargo test -p cap sampler -- --nocapture";
    let id = "cap-command-lease-throttling";
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
