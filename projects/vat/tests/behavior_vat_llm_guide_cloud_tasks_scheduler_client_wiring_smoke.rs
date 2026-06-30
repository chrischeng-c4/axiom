// SPEC-MANAGED: projects/vat/tech-design/logic/vat-llm-guide-cloud-tasks-cloud-scheduler-emulator-usage.md#vat-llm-guide-cloud-tasks-scheduler-client-wiring-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-llm-guide-cloud-tasks-scheduler-client-wiring-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim built-in-rust-emulators-cloud-tasks-cloud-scheduler
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_toml_runner -- --nocapture
// AW-EC-END

// Contract: `vat llm` exits successfully and still mentions vat.toml runner mode, direct command mode, and state/diff/logs evidence commands.
// Contract: the guide mentions the cloud-tasks / cloud-scheduler client-wiring factory (REST transport + http endpoint + anonymous credentials) and the direct-REST alternative.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_llm_guide_cloud_tasks_scheduler_client_wiring_smoke() {
    let command = "cargo test -p vat --test vat_toml_runner -- --nocapture";
    let id = "vat-llm-guide-cloud-tasks-scheduler-client-wiring-smoke";
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
