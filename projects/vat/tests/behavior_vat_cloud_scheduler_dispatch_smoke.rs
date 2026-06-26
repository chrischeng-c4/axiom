// SPEC-MANAGED: projects/vat/tech-design/logic/built-in-cloud-tasks-cloud-scheduler-emulators.md#vat-cloud-scheduler-dispatch-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-cloud-scheduler-dispatch-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim vat-cloud-scheduler-dispatch-smoke
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulator_scheduler -- --nocapture
// AW-EC-END

// Contract: creating an httpTarget job and calling jobs/{j}:run results in the emulator POSTing to the sink.
// Contract: no gcloud / Java required.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_cloud_scheduler_dispatch_smoke() {
    let command = "cargo test -p vat --test vat_emulator_scheduler -- --nocapture";
    let id = "vat-cloud-scheduler-dispatch-smoke";
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
