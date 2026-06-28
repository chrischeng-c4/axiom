// SPEC-MANAGED: projects/lumen/external-contracts/cli-interface/behavior/cli-interface.md#lumen-cli-interface-generated-clients
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-cli-interface-generated-clients
// @capability cli-interface
// @claim lumen-spec-schema-openapi-json-yaml-json-schema-offline
// @contract spec-gen-generated-clients-public-api-journey
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test behavior_lumen_cli_interface_generated_clients -- --ignored --nocapture
// AW-EC-END

// Contract: lumen spec gen emits Python, TypeScript, and Rust clients from the offline OpenAPI document.
// Contract: generated clients drive health, readiness, version, collection creation, indexing, search, duplicates, stats, and forced drop against a live h2c Lumen service.
// Contract: the generated Python client validates recursive pydantic QueryNode union shapes while using the bundled h2c runtime.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_cli_interface_generated_clients() {
    let command =
        "cargo test -p lumen --test behavior_lumen_cli_interface_generated_clients -- --ignored --nocapture";
    let id = "lumen-cli-interface-generated-clients";
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
