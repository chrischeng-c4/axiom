// SPEC-MANAGED: projects/cap/tech-design/logic/external-contracts.md#cap-config-logging-and-reap-policy
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec cap-config-logging-and-reap-policy
// @capability config-logging-and-reap-policy
// @claim configuration-defaults-and-compatibility
// @contract config-logging-and-reap-policy
// @category behavior
// @required_for_production true
// @command cargo test -p cap config -- --nocapture && cargo test -p cap eventlog -- --nocapture && cargo test -p cap reap -- --nocapture
// AW-EC-END

// Contract: configuration defaults and compatibility rules stay stable
// Contract: JSONL event log persistence roundtrips run records
// Contract: reap allowlist policy stays bounded and excludes active cap leases
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn cap_config_logging_and_reap_policy() {
    let command = "cargo test -p cap config -- --nocapture && cargo test -p cap eventlog -- --nocapture && cargo test -p cap reap -- --nocapture";
    let id = "cap-config-logging-and-reap-policy";
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
