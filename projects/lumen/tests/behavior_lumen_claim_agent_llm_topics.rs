// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-agent-llm-topics
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-agent-llm-topics
// @capability agent-offline-integration
// @claim lumen-llm-agent-topics-outline-workflow-integration-quickstart-recipes
// @contract agent-llm-topics
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test spec_cli -- --nocapture
// AW-EC-END

// Contract: The offline LLM outline, workflow, integration, quickstart, and recipe topics remain available.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_agent_llm_topics() {
    let command = "cargo test -p lumen --test spec_cli -- --nocapture";
    let id = "lumen-claim-agent-llm-topics";
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
