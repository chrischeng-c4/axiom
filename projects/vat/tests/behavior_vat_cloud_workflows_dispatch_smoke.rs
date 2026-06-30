// SPEC-MANAGED: projects/vat/tech-design/logic/built-in-cloud-workflows-emulator.md#vat-cloud-workflows-dispatch-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-cloud-workflows-dispatch-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim built-in-rust-emulator-cloud-workflows-subset-interpreter
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulator_workflows -- --nocapture
// AW-EC-END

// Contract: createWorkflow + createExecution for a workflow that assigns, call: http.post to a local sink, and returns yields a SUCCEEDED execution with the expected result and the sink receives the call.
// Contract: a try block calling a dead URL falls through to except (no panic); a named subworkflow call returns its value.
// Contract: no gcloud / Java / Docker required; the emulator starts in well under a second.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_cloud_workflows_dispatch_smoke() {
    let command = "cargo test -p vat --test vat_emulator_workflows -- --nocapture";
    let id = "vat-cloud-workflows-dispatch-smoke";
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
