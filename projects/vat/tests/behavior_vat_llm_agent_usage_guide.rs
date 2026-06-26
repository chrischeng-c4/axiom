// SPEC-MANAGED: projects/vat/tech-design/logic/llm-agent-usage-guide.md#vat-llm-agent-usage-guide
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-llm-agent-usage-guide
// @capability agent-native-gpu-native-dev-containers
// @claim agent-legible-state-and-diff-surface
// @contract agent-legible-state-and-diff-surface
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_toml_runner -- --nocapture
// AW-EC-END

// Contract: `vat llm` exits successfully.
// Contract: The guide mentions vat.toml runner mode and direct command mode.
// Contract: The guide mentions state, diff, and logs evidence commands.
// Contract: The guide preserves non-Docker and non-daemon boundaries.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_llm_agent_usage_guide() {
    let command = "cargo test -p vat --test vat_toml_runner -- --nocapture";
    let id = "vat-llm-agent-usage-guide";
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
