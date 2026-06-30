// SPEC-MANAGED: projects/vat/tech-design/logic/built-in-rust-emulators-pub-sub-firebase-auth.md#vat-builtin-preset-run-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-builtin-preset-run-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim built-in-rust-emulators-pub-sub-grpc-firebase-auth-rest
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulator_pubsub -- --nocapture --include-ignored
// AW-EC-END

// Contract: a `preset = "firebase-auth"` or `preset = "pubsub"` vat.toml run exports FIREBASE_AUTH_EMULATOR_HOST / PUBSUB_EMULATOR_HOST and the runner reaches the emulator; nothing remains after teardown.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_builtin_preset_run_smoke() {
    let command = "cargo test -p vat --test vat_emulator_pubsub -- --nocapture --include-ignored";
    let id = "vat-builtin-preset-run-smoke";
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
