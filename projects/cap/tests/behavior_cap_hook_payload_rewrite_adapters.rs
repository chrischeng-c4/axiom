// SPEC-MANAGED: projects/cap/tech-design/logic/external-contracts.md#cap-hook-payload-rewrite-adapters
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec cap-hook-payload-rewrite-adapters
// @capability agent-hook-installation
// @claim hook-payload-rewrite-adapters
// @contract hook-payload-rewrite-adapters
// @category behavior
// @required_for_production true
// @command cargo test -p cap hook -- --nocapture
// AW-EC-END

// Contract: hook payload adapters rewrite Bash commands without making cap a hard dependency
// Contract: recursive cap invocations and empty commands remain fail-open
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn cap_hook_payload_rewrite_adapters() {
    let command = "cargo test -p cap hook -- --nocapture";
    let id = "cap-hook-payload-rewrite-adapters";
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
