// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3781.md#nx-graph-parity
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec nx-graph-parity
// @capability workspace-task-runner
// @claim nx-graph-parity
// @contract nx-graph-parity
// @category behavior
// @required_for_production true
// @command cargo test -p jet --lib pkg_manager::nx -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn nx_graph_parity() {
    let command = "cargo test -p jet --lib pkg_manager::nx -- --nocapture";
    let id = "nx-graph-parity";
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
