// SPEC-MANAGED: projects/vat/tech-design/logic/built-in-http-mock-record-replay-proxy.md#vat-http-mock-stub-and-mitm-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-http-mock-stub-and-mitm-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim vat-http-mock-stub-and-mitm-smoke
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulator_httpmock -- --nocapture
// AW-EC-END

// Contract: a stubbed GET http://api.test/v1/x through the proxy returns the stub body with no network.
// Contract: a client trusting the minted CA and using the proxy GETs a stubbed https://api.test/v1/y via CONNECT+MITM and receives the stub (no real upstream).
// Contract: with no stub, a first GET to a local upstream records+forwards and a second GET replays from the cassette with the upstream down.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_http_mock_stub_and_mitm_smoke() {
    let command = "cargo test -p vat --test vat_emulator_httpmock -- --nocapture";
    let id = "vat-http-mock-stub-and-mitm-smoke";
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
