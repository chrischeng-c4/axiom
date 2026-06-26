// SPEC-MANAGED: projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md#vat-cloud-storage-roundtrip-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-cloud-storage-roundtrip-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim vat-cloud-storage-roundtrip-smoke
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulator_storage -- --nocapture
// AW-EC-END

// Contract: a media upload then download (alt=media) returns the same bytes; a multipart upload round-trips; list with a prefix finds the object; delete removes it (404 after).
// Contract: object names with slashes work; no gcloud / Java / Docker required; the emulator starts in well under a second.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_cloud_storage_roundtrip_smoke() {
    let command = "cargo test -p vat --test vat_emulator_storage -- --nocapture";
    let id = "vat-cloud-storage-roundtrip-smoke";
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
