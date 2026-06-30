// SPEC-MANAGED: projects/vat/tech-design/logic/gcp-firebase-emulator-service-presets.md#vat-firebase-bundle-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-firebase-bundle-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim gcp-firebase-emulator-service-presets
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulators -- --nocapture --include-ignored
// AW-EC-END

// Contract: a firebase preset with a firebase.json starts the suite and exports the configured *_EMULATOR_HOST vars.
// Contract: the test skips gracefully when firebase-tools and docker are both unavailable.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_firebase_bundle_smoke() {
    let command = "cargo test -p vat --test vat_emulators -- --nocapture --include-ignored";
    let id = "vat-firebase-bundle-smoke";
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
