// SPEC-MANAGED: apps/preview/external-contracts/behavior/cookie-header-route-binding-contract.md#cookie-header-route-binding-contract
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec cookie-header-route-binding-contract
// @capability gke-uat-preview-environment-rendering
// @claim cookie-header-route-binding-contract
// @contract cookie-header-route-binding-contract
// @category behavior
// @required_for_production true
// @command cargo test -p preview route_binding_uses_target_not_namespace_cookie
// AW-EC-END

// Contract: Route binding keeps the public target `mr-<id>` separate from namespace `uat-mr-<id>`.
// Contract: Browser cookie selection uses `uat_target`, not a raw namespace value.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn cookie_header_route_binding_contract() {
    let command = "cargo test -p preview route_binding_uses_target_not_namespace_cookie";
    let id = "cookie-header-route-binding-contract";
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
