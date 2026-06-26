// SPEC-MANAGED: projects/vat/tech-design/logic/external-contracts.md#vat-copy-on-write-lifecycle
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-copy-on-write-lifecycle
// @capability agent-native-gpu-native-dev-containers
// @claim copy-on-write-fork-and-snapshot-lifecycle
// @contract copy-on-write-fork-and-snapshot-lifecycle
// @category behavior
// @required_for_production true
// @command rg -n -e copy-on-write -e fork -e snapshot -e clonefile -e APFS projects/vat/README.md
// AW-EC-END

// Contract: README preserves copy-on-write lifecycle language
// Contract: fork, snapshot, clonefile, and APFS remain visible contract terms
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_copy_on_write_lifecycle() {
    let command =
        "rg -n -e copy-on-write -e fork -e snapshot -e clonefile -e APFS projects/vat/README.md";
    let id = "vat-copy-on-write-lifecycle";
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
