// SPEC-MANAGED: projects/vat/tech-design/interfaces/rest/openapi-driven-mock-http-service.md#vat-openapi-standalone-and-proxy-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-openapi-standalone-and-proxy-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim openapi-driven-mock-http-service-spec-responses
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulator_openapi -- --nocapture
// AW-EC-END

// Contract: spawning vat emulator openapi --spec <tmp spec> and GETting a documented path returns the spec's example; an undocumented path returns 404.
// Contract: registering a spec for a host on the http-mock proxy answers a proxied HTTPS-MITM GET to that host from the spec (no stub, no upstream).
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_openapi_standalone_and_proxy_smoke() {
    let command = "cargo test -p vat --test vat_emulator_openapi -- --nocapture";
    let id = "vat-openapi-standalone-and-proxy-smoke";
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
