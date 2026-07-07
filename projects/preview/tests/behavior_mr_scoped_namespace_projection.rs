// SPEC-MANAGED: projects/preview/external-contracts/behavior/mr-scoped-namespace-projection.md#mr-scoped-namespace-projection
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec mr-scoped-namespace-projection
// @capability gke-uat-preview-environment-rendering
// @claim mr-scoped-namespace-projection
// @contract mr-scoped-namespace-projection
// @category behavior
// @required_for_production true
// @command cargo test -p preview render_creates_gke_contract_files
// AW-EC-END

// Contract: `preview render` emits spec, workload clone plan, namespace, service account, quota, limits, RBAC, deployment, service, route-binding, MR comment, and cleanup-plan files.
// Contract: The rendered namespace is named `uat-mr-<id>`, carries preview labels, and records the base namespace/source workload.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn mr_scoped_namespace_projection() {
    let command = "cargo test -p preview render_creates_gke_contract_files";
    let id = "mr-scoped-namespace-projection";
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
