// SPEC-MANAGED: projects/vat/tech-design/logic/built-in-http-mock-record-replay-proxy.md#vat-http-mock-lean-build
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-http-mock-lean-build
// @capability agent-native-gpu-native-dev-containers
// @claim built-in-http-mock-record-replay-proxy-https-mitm
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo build -p vat --no-default-features
// AW-EC-END

// Contract: vat compiles without the emulator feature; the http-mock emulator verb then errors cleanly, never a panic.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_http_mock_lean_build() {
    let command = "cargo build -p vat --no-default-features";
    let id = "vat-http-mock-lean-build";
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
