// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-cli-service-process-interface
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-cli-service-process-interface
// @capability cli-interface
// @claim service-process-interface
// @contract cli-service-process-interface
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test api_e2e -- --nocapture
// AW-EC-END

// Contract: The long-running service exposes health, readiness, version, metrics, indexing, and search routes through the binary-served API.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_cli_service_process_interface() {
    let command = "cargo test -p lumen --test api_e2e -- --nocapture";
    let id = "lumen-claim-cli-service-process-interface";
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
