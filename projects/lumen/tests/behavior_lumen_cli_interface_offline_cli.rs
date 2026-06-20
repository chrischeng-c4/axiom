// SPEC-MANAGED: projects/lumen/external-contracts/cli-interface/behavior/cli-interface.md#lumen-cli-interface-offline-cli
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-cli-interface-offline-cli
// @capability cli-interface
// @claim lumen-spec-schema-openapi-json-yaml-json-schema-offline
// @contract offline-cli-agent-onboarding
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test spec_cli -- --nocapture
// AW-EC-END

// Contract: lumen spec emits valid OpenAPI JSON, OpenAPI YAML, and JSON-schema output offline.
// Contract: lumen spec exposes query-shape, field, analyzer, and vector-metric catalogs.
// Contract: lumen llm outline, workflow, integration, quickstart, and recipes preserve the ingest-search-hydrate agent workflow and non-goals.
// Contract: lumen llm integration recommends the Postgres/AlloyDB boundary: database commit/outbox or CDC, external adapter-owned Pub/Sub retry/DLQ, HTTP writes into lumen, and no direct external publishing to lumen's NATS WAL.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_cli_interface_offline_cli() {
    let command = "cargo test -p lumen --test spec_cli -- --nocapture";
    let id = "lumen-cli-interface-offline-cli";
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
