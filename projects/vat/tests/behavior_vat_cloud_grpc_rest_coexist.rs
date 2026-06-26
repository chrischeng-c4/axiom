// SPEC-MANAGED: projects/vat/tech-design/interfaces/grpc/cloud-tasks-and-cloud-scheduler-emulator-grpc-front-end-alongsid.md#vat-cloud-grpc-rest-coexist
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-cloud-grpc-rest-coexist
// @capability agent-native-gpu-native-dev-containers
// @claim vat-cloud-grpc-rest-coexist
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulator_tasks --test vat_emulator_scheduler -- --nocapture
// AW-EC-END

// Contract: the existing REST e2e tests pass unchanged (REST + gRPC coexist on one port).
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_cloud_grpc_rest_coexist() {
    let command =
        "cargo test -p vat --test vat_emulator_tasks --test vat_emulator_scheduler -- --nocapture";
    let id = "vat-cloud-grpc-rest-coexist";
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
