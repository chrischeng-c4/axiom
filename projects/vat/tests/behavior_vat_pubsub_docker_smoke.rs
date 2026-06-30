// SPEC-MANAGED: projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#vat-pubsub-docker-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-pubsub-docker-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim gcp-firebase-emulator-service-presets
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulators -- --nocapture --include-ignored
// AW-EC-END

// Contract: without the pubsub gcloud component, runtime=auto resolves to docker, exports PUBSUB_EMULATOR_HOST, and removes the container at teardown.
// Contract: the test skips gracefully when docker is unavailable.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_pubsub_docker_smoke() {
    let command = "cargo test -p vat --test vat_emulators -- --nocapture --include-ignored";
    let id = "vat-pubsub-docker-smoke";
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
