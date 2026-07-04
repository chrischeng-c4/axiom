// SPEC-MANAGED: projects/preview/external-contracts/behavior/render-contract-ec.md#render-contract-ec
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec render-contract-ec
// @capability preview-external-contracts
// @claim render-contract-ec
// @contract render-contract-ec
// @category behavior
// @required_for_production true
// @command cargo test -p preview --test render_contract
// AW-EC-END

// Contract: Render contract tests cover generated file names, base workload clone plan, namespace naming, route binding, and cleanup protected namespace output.
// Contract: The render EC remains runnable without a live cluster.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn render_contract_ec() {
    let command = "cargo test -p preview --test render_contract";
    let id = "render-contract-ec";
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
