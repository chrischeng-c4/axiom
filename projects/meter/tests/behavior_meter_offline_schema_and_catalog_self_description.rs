// SPEC-MANAGED: projects/meter/tech-design/logic/external-contracts.md#meter-offline-schema-and-catalog-self-description
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec meter-offline-schema-and-catalog-self-description
// @capability agent-use-first-cli
// @claim offline-schema-and-catalog-self-description
// @contract offline-schema-and-catalog-self-description
// @category behavior
// @required_for_production true
// @command cargo run -p meter-cli --bin meter -- spec --catalog --compact
// AW-EC-END

// Contract: meter spec emits the finding catalog without target setup
// Contract: offline self-description remains available for agents
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn meter_offline_schema_and_catalog_self_description() {
    let command = "cargo run -p meter-cli --bin meter -- spec --catalog --compact";
    let id = "meter-offline-schema-and-catalog-self-description";
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
