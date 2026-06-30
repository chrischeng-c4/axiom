// SPEC-MANAGED: projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#vat-firestore-native-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-firestore-native-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim gcp-firebase-emulator-service-presets
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulators -- --nocapture --include-ignored
// AW-EC-END

// Contract: with gcloud + Java + the firestore component, vat starts the emulator, the runner sees FIRESTORE_EMULATOR_HOST, and vat state shows the service ready with the var in exported_env.
// Contract: the test skips gracefully when the native firestore emulator is unavailable.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_firestore_native_smoke() {
    let command = "cargo test -p vat --test vat_emulators -- --nocapture --include-ignored";
    let id = "vat-firestore-native-smoke";
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
