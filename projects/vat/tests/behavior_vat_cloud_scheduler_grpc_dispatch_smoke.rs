// SPEC-MANAGED: projects/vat/tech-design/interfaces/grpc/cloud-tasks-and-cloud-scheduler-emulator-grpc-front-end-alongsid.md#vat-cloud-scheduler-grpc-dispatch-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-cloud-scheduler-grpc-dispatch-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim dual-protocol-emulators-cloud-tasks-scheduler-grpc-alongside-rest
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulator_scheduler_grpc -- --nocapture
// AW-EC-END

// Contract: a generated google.cloud.scheduler.v1 gRPC client CreateJob + RunJob, and the emulator POSTs the job httpTarget to a local sink.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_cloud_scheduler_grpc_dispatch_smoke() {
    let command = "cargo test -p vat --test vat_emulator_scheduler_grpc -- --nocapture";
    let id = "vat-cloud-scheduler-grpc-dispatch-smoke";
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
