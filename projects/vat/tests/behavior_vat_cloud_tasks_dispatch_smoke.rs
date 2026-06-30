// SPEC-MANAGED: projects/vat/tech-design/logic/built-in-cloud-tasks-cloud-scheduler-emulators.md#vat-cloud-tasks-dispatch-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-cloud-tasks-dispatch-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim built-in-rust-emulators-cloud-tasks-cloud-scheduler
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulator_tasks -- --nocapture
// AW-EC-END

// Contract: spawning `vat emulator cloud-tasks`, creating a queue and a task targeting a local sink, results in the emulator POSTing the task body to the sink.
// Contract: no gcloud / Java required; the emulator starts in well under a second.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_cloud_tasks_dispatch_smoke() {
    let command = "cargo test -p vat --test vat_emulator_tasks -- --nocapture";
    let id = "vat-cloud-tasks-dispatch-smoke";
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
