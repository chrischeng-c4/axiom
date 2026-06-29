// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-long-running-log-fanout
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-long-running-log-fanout
// @capability long-running-stability
// @claim log-fan-out-rebuild-from-log
// @contract long-running-log-fanout
// @category stability
// @required_for_production true
// @command cargo test -p lumen --test wal_relay --test wal_nats_e2e -- --nocapture
// AW-EC-END

// Contract: A late or second node can replay the published write stream and converge with live writes.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_long_running_log_fanout() {
    let command = "cargo test -p lumen --test wal_relay --test wal_nats_e2e -- --nocapture";
    let id = "lumen-claim-long-running-log-fanout";
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
