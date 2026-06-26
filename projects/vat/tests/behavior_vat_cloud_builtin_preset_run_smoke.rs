// SPEC-MANAGED: projects/vat/tech-design/logic/built-in-cloud-tasks-cloud-scheduler-emulators.md#vat-cloud-builtin-preset-run-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-cloud-builtin-preset-run-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim vat-cloud-builtin-preset-run-smoke
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat cloud_builtin_preset_exports_host -- --nocapture --ignored
// AW-EC-END

// Contract: a `preset = "cloud-tasks"` / `preset = "cloud-scheduler"` vat.toml run exports CLOUD_TASKS_EMULATOR_HOST / CLOUD_SCHEDULER_EMULATOR_HOST and the runner reaches the emulator; nothing remains after teardown.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_cloud_builtin_preset_run_smoke() {
    let command = "cargo test -p vat cloud_builtin_preset_exports_host -- --nocapture --ignored";
    let id = "vat-cloud-builtin-preset-run-smoke";
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
