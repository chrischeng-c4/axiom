// SPEC-MANAGED: projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md#vat-cloud-storage-preset-run-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-cloud-storage-preset-run-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim vat-cloud-storage-preset-run-smoke
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat cloud_storage_preset_exports_host -- --nocapture --ignored
// AW-EC-END

// Contract: a preset = cloud-storage vat.toml run exports STORAGE_EMULATOR_HOST and the runner uploads then downloads an object byte-identical; nothing remains after teardown.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_cloud_storage_preset_run_smoke() {
    let command = "cargo test -p vat cloud_storage_preset_exports_host -- --nocapture --ignored";
    let id = "vat-cloud-storage-preset-run-smoke";
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
