// SPEC-MANAGED: projects/cap/tech-design/logic/external-contracts.md#cap-agent-hook-installation
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec cap-agent-hook-installation
// @capability agent-hook-installation
// @claim claude-and-codex-hook-installation
// @contract agent-hook-installation
// @category behavior
// @required_for_production true
// @command cargo test -p cap hook_install -- --nocapture && cargo test -p cap hook -- --nocapture
// AW-EC-END

// Contract: cap init hook installation is idempotent for Claude Code and Codex CLI
// Contract: unrelated user hooks are preserved
// Contract: hook payload adapters rewrite Bash commands without making cap a hard dependency
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn cap_agent_hook_installation() {
    let command = "cargo test -p cap hook_install -- --nocapture && cargo test -p cap hook -- --nocapture";
    let id = "cap-agent-hook-installation";
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
