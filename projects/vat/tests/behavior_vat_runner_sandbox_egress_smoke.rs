// SPEC-MANAGED: projects/vat/tech-design/logic/apply-the-sandbox-seatbelt-isolation-egress-to-runner-mode-comma.md#vat-runner-sandbox-egress-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-runner-sandbox-egress-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim vat-runner-sandbox-egress-smoke
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_runner_sandbox -- --nocapture
// AW-EC-END

// Contract: a `vat run <runner>` with isolation=seatbelt + egress=localhost-only runs the runner under sandbox-exec; the runner can reach a localhost listener but a connect to a non-loopback host is denied; an emulator service still reaches the network. Skips off-macOS / no sandbox-exec.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_runner_sandbox_egress_smoke() {
    let command = "cargo test -p vat --test vat_runner_sandbox -- --nocapture";
    let id = "vat-runner-sandbox-egress-smoke";
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
