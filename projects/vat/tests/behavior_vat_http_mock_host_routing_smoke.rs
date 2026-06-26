// SPEC-MANAGED: projects/vat/tech-design/logic/vat-network-sandbox-v1-transparent-http-service-routing-to-local.md#vat-http-mock-host-routing-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-http-mock-host-routing-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim vat-http-mock-host-routing-smoke
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulator_httpmock_routing -- --nocapture
// AW-EC-END

// Contract: with a route example.test -> http://127.0.0.1:<sink>, a request through the proxy to http://example.test/p (and https://example.test/p via CONNECT MITM) is answered by the local sink, not forwarded upstream.
// Contract: an unmatched host still forwards/records exactly as before (no regression).
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_http_mock_host_routing_smoke() {
    let command = "cargo test -p vat --test vat_emulator_httpmock_routing -- --nocapture";
    let id = "vat-http-mock-host-routing-smoke";
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
