// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3781.md#task-runner-graph-cache
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec task-runner-graph-cache
// @capability workspace-task-runner
// @claim task-runner-graph-cache
// @contract task-runner-graph-cache
// @category behavior
// @required_for_production true
// @command cargo test -p jet --lib task_runner -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn task_runner_graph_cache() {
    let command = "cargo test -p jet --lib task_runner -- --nocapture";
    let id = "task-runner-graph-cache";
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
