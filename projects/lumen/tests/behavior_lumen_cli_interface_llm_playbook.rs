// SPEC-MANAGED: projects/lumen/external-contracts/cli-interface/behavior/cli-interface.md#lumen-cli-interface-llm-playbook
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-cli-interface-llm-playbook
// @capability cli-interface
// @claim lumen-llm-agent-topics-outline-workflow-integration-quickstart-recipes
// @contract lumen-llm-agent-topics-outline-workflow-integration-quickstart-recipes
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test spec_cli -- --nocapture
// AW-EC-END

// Contract: lumen llm outline, workflow, integration, quickstart, and recipes preserve the agent-facing topic set.
// Contract: lumen llm integration preserves the provider-neutral Postgres/AlloyDB adapter guidance and keeps Pub/Sub-specific ownership outside lumen core.
// Contract: agent-facing playbook output remains deterministic and offline.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_cli_interface_llm_playbook() {
    let command = "cargo test -p lumen --test spec_cli -- --nocapture";
    let id = "lumen-cli-interface-llm-playbook";
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
