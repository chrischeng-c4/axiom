// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-backup-periodic-snapshotter
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-backup-periodic-snapshotter
// @capability backup-restore
// @claim periodic-snapshotter-serve
// @contract backup-periodic-snapshotter
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test backup_restore_e2e -- --nocapture
// AW-EC-END

// Contract: The serving process snapshot loop and restore path remain covered by the backup/restore e2e gate.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_backup_periodic_snapshotter() {
    let command = "cargo test -p lumen --test backup_restore_e2e -- --nocapture";
    let id = "lumen-claim-backup-periodic-snapshotter";
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
