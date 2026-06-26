// SPEC-MANAGED: projects/vat/tech-design/interfaces/rest/openapi-driven-mock-http-service.md#vat-openapi-preset-run-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-openapi-preset-run-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim vat-openapi-preset-run-smoke
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat openapi_preset_serves_spec -- --nocapture --ignored
// AW-EC-END

// Contract: a preset = openapi vat.toml run exports OPENAPI_MOCK_HOST and the runner curls a documented operation, getting the spec-derived response with no app code change.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_openapi_preset_run_smoke() {
    let command = "cargo test -p vat openapi_preset_serves_spec -- --nocapture --ignored";
    let id = "vat-openapi-preset-run-smoke";
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
