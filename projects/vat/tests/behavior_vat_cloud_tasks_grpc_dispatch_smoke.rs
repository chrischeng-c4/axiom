// SPEC-MANAGED: projects/vat/tech-design/interfaces/grpc/cloud-tasks-and-cloud-scheduler-emulator-grpc-front-end-alongsid.md#vat-cloud-tasks-grpc-dispatch-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-cloud-tasks-grpc-dispatch-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim vat-cloud-tasks-grpc-dispatch-smoke
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulator_tasks_grpc -- --nocapture
// AW-EC-END

// Contract: a generated google.cloud.tasks.v2 gRPC client (insecure channel to the emulator host:port) CreateQueue + CreateTask, and the emulator POSTs the task body to a local sink.
// Contract: the REST surface on the same port still works.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_cloud_tasks_grpc_dispatch_smoke() {
    let command = "cargo test -p vat --test vat_emulator_tasks_grpc -- --nocapture";
    let id = "vat-cloud-tasks-grpc-dispatch-smoke";
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
