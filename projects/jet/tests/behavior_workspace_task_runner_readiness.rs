// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3781.md#workspace-task-runner-readiness
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec workspace-task-runner-readiness
// @capability workspace-task-runner
// @claim workspace-task-runner-readiness
// @contract workspace-task-runner-readiness
// @category behavior
// @required_for_production true
// @command cargo test -p jet --lib task_runner -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn workspace_task_runner_readiness() {
    let command = "cargo test -p jet --lib task_runner -- --nocapture";
    let id = "workspace-task-runner-readiness";
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
