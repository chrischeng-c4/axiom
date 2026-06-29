// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-cli-deployment-operator-command-surface
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-cli-deployment-operator-command-surface
// @capability cli-interface
// @claim deployment-operator-command-surface
// @contract cli-deployment-operator-command-surface
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --features operator --test operator_render -- --nocapture
// AW-EC-END

// Contract: The operator-facing command surface renders CRD and serving objects used by the deployment path.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_cli_deployment_operator_command_surface() {
    let command = "cargo test -p lumen --features operator --test operator_render -- --nocapture";
    let id = "lumen-claim-cli-deployment-operator-command-surface";
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
