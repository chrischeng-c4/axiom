// SPEC-MANAGED: projects/vat/tech-design/logic/built-in-http-mock-record-replay-proxy.md#vat-http-mock-preset-run-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-http-mock-preset-run-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim built-in-http-mock-record-replay-proxy-https-mitm
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulator_httpmock -- --nocapture --include-ignored
// AW-EC-END

// Contract: a preset = http-mock vat.toml run exports HTTP(S)_PROXY + NO_PROXY + the CA-trust vars; the runner curls a stubbed https URL with no code change and gets the stub.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_http_mock_preset_run_smoke() {
    let command = "cargo test -p vat --test vat_emulator_httpmock -- --nocapture --include-ignored";
    let id = "vat-http-mock-preset-run-smoke";
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
