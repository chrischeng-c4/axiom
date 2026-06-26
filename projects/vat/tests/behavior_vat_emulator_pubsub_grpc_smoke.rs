// SPEC-MANAGED: projects/vat/tech-design/logic/built-in-rust-emulators-pub-sub-firebase-auth.md#vat-emulator-pubsub-grpc-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-emulator-pubsub-grpc-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim vat-emulator-pubsub-grpc-smoke
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulator_pubsub -- --nocapture
// AW-EC-END

// Contract: a tonic client generated from the same proto can CreateTopic -> CreateSubscription -> Publish -> Pull -> Acknowledge against `vat emulator pubsub`.
// Contract: no gcloud / Java required.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_emulator_pubsub_grpc_smoke() {
    let command = "cargo test -p vat --test vat_emulator_pubsub -- --nocapture";
    let id = "vat-emulator-pubsub-grpc-smoke";
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
