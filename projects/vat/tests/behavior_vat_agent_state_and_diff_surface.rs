// SPEC-MANAGED: projects/vat/tech-design/logic/external-contracts.md#vat-agent-state-and-diff-surface
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-agent-state-and-diff-surface
// @capability agent-native-gpu-native-dev-containers
// @claim agent-legible-state-and-diff-surface
// @contract agent-legible-state-and-diff-surface
// @category behavior
// @required_for_production true
// @command rg -n -e 'vat state' -e 'vat diff' -e '--json' -e structured projects/vat/README.md
// AW-EC-END

// Contract: README exposes vat state and vat diff
// Contract: structured JSON output remains part of the agent-facing contract
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_agent_state_and_diff_surface() {
    let command =
        "rg -n -e 'vat state' -e 'vat diff' -e '--json' -e structured projects/vat/README.md";
    let id = "vat-agent-state-and-diff-surface";
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
