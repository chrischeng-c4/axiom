// SPEC-MANAGED: projects/vat/tech-design/logic/built-in-rust-emulators-pub-sub-firebase-auth.md#vat-emulator-auth-rest-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-emulator-auth-rest-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim built-in-rust-emulators-pub-sub-grpc-firebase-auth-rest
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulator_auth -- --nocapture
// AW-EC-END

// Contract: spawning `vat emulator firebase-auth` and driving signUp -> signInWithPassword -> lookup over HTTP returns a JWT idToken and the created user.
// Contract: no Java / firebase-tools required; the emulator starts in well under a second.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_emulator_auth_rest_smoke() {
    let command = "cargo test -p vat --test vat_emulator_auth -- --nocapture";
    let id = "vat-emulator-auth-rest-smoke";
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
