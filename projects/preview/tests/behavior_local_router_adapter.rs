// SPEC-MANAGED: projects/preview/external-contracts/behavior/local-router-adapter.md#local-router-adapter
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec local-router-adapter
// @capability preview-external-contracts
// @claim local-router-adapter
// @contract local-router-adapter
// @category behavior
// @required_for_production true
// @command cargo test -p preview --test local_cicd_contract local_router_resolve_proves_base_preview_and_fail_closed
// AW-EC-END

// Contract: `preview router resolve --dir <rendered-dir>` routes requests without target header/cookie to the base route.
// Contract: `preview router resolve` routes valid X-UAT-Target values to the preview namespace and service.
// Contract: `preview router resolve` lets X-UAT-Target override uat_target cookie.
// Contract: Invalid targets return a not-found decision and never silently fallback to base.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn local_router_adapter() {
    let command =
        "cargo test -p preview --test local_cicd_contract local_router_resolve_proves_base_preview_and_fail_closed";
    let id = "local-router-adapter";
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
