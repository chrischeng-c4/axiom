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

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the aw.toml inventory command authoritative"]
fn vat_host_process_gpu_visibility() {
    panic!(
        "AW EC placeholder for {}",
        "vat-host-process-gpu-visibility"
    );
}
// CODEGEN-END
