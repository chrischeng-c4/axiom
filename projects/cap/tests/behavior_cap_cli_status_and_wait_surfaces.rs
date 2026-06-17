// SPEC-MANAGED: projects/cap/tech-design/logic/external-contracts.md#cap-cli-status-and-wait-surfaces
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec cap-cli-status-and-wait-surfaces
// @capability daemon-lifecycle-and-status
// @claim cli-status-and-wait-surfaces
// @contract cli-status-and-wait-surfaces
// @category behavior
// @required_for_production true
// @command cargo test -p cap cli -- --nocapture
// AW-EC-END

// Contract: status and wait CLI surfaces use the daemon contract without blocking agent commands
// Contract: command leases stay isolated by process group
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn cap_cli_status_and_wait_surfaces() {
    let command = "cargo test -p cap cli -- --nocapture";
    let id = "cap-cli-status-and-wait-surfaces";
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
