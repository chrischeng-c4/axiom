// SPEC-MANAGED: projects/lumen/external-contracts/agentic-integration/behavior/agentic-integration.md#lumen-agentic-integration-query-catalog
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-agentic-integration-query-catalog
// @capability agentic-integration
// @claim query-shape-cookbook-field-analyzer-catalog
// @contract query-shape-cookbook-field-analyzer-catalog
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test spec_cli -- --nocapture
// AW-EC-END

// Contract: lumen spec exposes query-shape, field, analyzer, and vector-metric catalogs.
// Contract: agent-facing query catalog output remains deterministic and offline.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_agentic_integration_query_catalog() {
    let command = "cargo test -p lumen --test spec_cli -- --nocapture";
    let id = "lumen-agentic-integration-query-catalog";
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
