// SPEC-MANAGED: projects/vat/tech-design/interfaces/grpc/vat-network-sandbox-v2-transparent-grpc-routing-via-h2-mitm-reve.md#vat-grpc-mitm-routing-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-grpc-mitm-routing-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim vat-grpc-mitm-routing-smoke
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulator_grpc_mitm_routing -- --nocapture
// AW-EC-END

// Contract: a generated google.cloud.tasks.v2 gRPC client over TLS to cloudtasks.googleapis.com (trusting vat's CA), routed through the proxy, CreateQueue + CreateTask succeeds and the task is dispatched to a local sink.
// Contract: HTTP routing (#503) and unmatched-host forwarding still work (no regression).
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_grpc_mitm_routing_smoke() {
    let command = "cargo test -p vat --test vat_emulator_grpc_mitm_routing -- --nocapture";
    let id = "vat-grpc-mitm-routing-smoke";
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
