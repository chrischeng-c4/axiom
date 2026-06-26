// SPEC-MANAGED: projects/vat/tech-design/logic/vat-network-sandbox-full-hermetic-http-mock-no-forward-mode-bloc.md#vat-hermetic-no-forward-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-hermetic-no-forward-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim vat-hermetic-no-forward-smoke
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulator_httpmock_hermetic -- --nocapture
// AW-EC-END

// Contract: a proxy started with --no-forward returns 502 hermetic for an unmatched host (no upstream reached) while a registered stub on the same proxy returns 200; an unmatched request on a default (forwarding) proxy still forwards.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_hermetic_no_forward_smoke() {
    let command = "cargo test -p vat --test vat_emulator_httpmock_hermetic -- --nocapture";
    let id = "vat-hermetic-no-forward-smoke";
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
