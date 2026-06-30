// SPEC-MANAGED: projects/vat/tech-design/interfaces/grpc/pool-the-grpc-mitm-reverse-proxy-h2c-upstream-connections.md#vat-grpc-pool-build
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-grpc-pool-build
// @capability agent-native-gpu-native-dev-containers
// @claim grpc-reverse-proxy-h2c-connection-pool
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo build -p vat --no-default-features
// AW-EC-END

// Contract: vat compiles with and without default features.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_grpc_pool_build() {
    let command = "cargo build -p vat --no-default-features";
    let id = "vat-grpc-pool-build";
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
