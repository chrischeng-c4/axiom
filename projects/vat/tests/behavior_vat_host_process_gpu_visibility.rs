// SPEC-MANAGED: projects/vat/tech-design/logic/external-contracts.md#vat-host-process-gpu-visibility
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-host-process-gpu-visibility
// @capability agent-native-gpu-native-dev-containers
// @claim host-process-execution-and-gpu-visibility
// @contract host-process-execution-and-gpu-visibility
// @category behavior
// @required_for_production true
// @command rg -n -e 'Apple GPU' -e Metal -e MPS -e MLX -e tensorflow-metal projects/vat/README.md projects/vat/src/gpu.rs
// AW-EC-END

// Contract: README/source names Apple GPU access as a host-process property
// Contract: Metal, MPS, MLX, and tensorflow-metal are present in the GPU contract
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_host_process_gpu_visibility() {
    let command =
        "rg -n -e 'Apple GPU' -e Metal -e MPS -e MLX -e tensorflow-metal projects/vat/README.md projects/vat/src/gpu.rs";
    let id = "vat-host-process-gpu-visibility";
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
