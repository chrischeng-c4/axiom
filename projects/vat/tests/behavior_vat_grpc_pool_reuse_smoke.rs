// SPEC-MANAGED: projects/vat/tech-design/interfaces/grpc/pool-the-grpc-mitm-reverse-proxy-h2c-upstream-connections.md#vat-grpc-pool-reuse-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-grpc-pool-reuse-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim vat-grpc-pool-reuse-smoke
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulator_grpc_mitm_routing -- --nocapture
// AW-EC-END

// Contract: two sequential gRPC calls through the MITM to the same emulator both succeed (reuse path), and the #509 single-call e2e still passes.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_grpc_pool_reuse_smoke() {
    let command = "cargo test -p vat --test vat_emulator_grpc_mitm_routing -- --nocapture";
    let id = "vat-grpc-pool-reuse-smoke";
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
