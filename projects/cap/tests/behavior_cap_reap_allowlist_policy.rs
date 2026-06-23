// SPEC-MANAGED: projects/cap/tech-design/logic/external-contracts.md#cap-reap-allowlist-policy
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec cap-reap-allowlist-policy
// @capability config-logging-and-reap-policy
// @claim reap-allowlist-policy
// @contract reap-allowlist-policy
// @category behavior
// @required_for_production true
// @command cargo test -p cap reap -- --nocapture
// AW-EC-END

// Contract: reap allowlist policy stays bounded
// Contract: active cap leases are excluded from reap actions
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn cap_reap_allowlist_policy() {
    let command = "cargo test -p cap reap -- --nocapture";
    let id = "cap-reap-allowlist-policy";
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
