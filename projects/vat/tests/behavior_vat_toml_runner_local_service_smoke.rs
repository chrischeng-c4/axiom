// SPEC-MANAGED: projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#vat-toml-runner-local-service-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-toml-runner-local-service-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim local-agent-test-runner-protocol
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_toml_runner -- --nocapture
// AW-EC-END

// Contract: vat run <runner-id> starts a local readiness service, runs the runner, captures logs, records artifacts, and returns JSON evidence.
// Contract: failed runner evidence remains inspectable.
// Contract: direct vat run -- <cmd> compatibility is preserved.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_toml_runner_local_service_smoke() {
    let command = "cargo test -p vat --test vat_toml_runner -- --nocapture";
    let id = "vat-toml-runner-local-service-smoke";
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
