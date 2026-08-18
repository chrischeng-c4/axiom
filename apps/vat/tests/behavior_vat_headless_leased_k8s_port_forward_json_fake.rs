// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-headless-leased-k8s-port-forward-json-fake
// @capability agent-native-gpu-native-dev-containers
// @claim headless-ephemeral-kubernetes-session
// @contract vat-headless-leased-k8s-port-forward-json-fake
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_k8s_ephemeral leased_session_port_forward_json -- --nocapture
// AW-EC-END

// Contract: The fake runtime proves text stays separate while JSON returns one vat.k8s.session.port-forward.v1 only after exact tunnel/group cleanup, preserving the host child exit and separately bounded stream snapshots without raw replay.
// Contract: VAT masks its own setup/API/tunnel/cleanup errors, preserves opaque credential-free child output in a successful result, refuses to open a tunnel when the lease crosses expiry after API proof, and cleans direct/outer children before readers join after partial setup failure.
// Contract: Focused deterministic filter passed 7/7. The independent-kubectl Service-forward E2E passed 1/1 (36 filtered) in 49.57s and includes the strict JSON tunnel only for one Service-only loopback session.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_headless_leased_k8s_port_forward_json_fake() {
    let command =
        "cargo test -p vat --test vat_k8s_ephemeral leased_session_port_forward_json -- --nocapture";
    let id = "vat-headless-leased-k8s-port-forward-json-fake";
    let mut root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !root.join(".aw").is_dir() {
        assert!(
            root.pop(),
            "AW EC {id}: no .aw/ project root above {}",
            env!("CARGO_MANIFEST_DIR")
        );
    }
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(&root)
        .output()
        .unwrap_or_else(|e| panic!("AW EC {id}: failed to spawn `{command}`: {e}"));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if output.status.success()
        && aw_ec_cargo_test_executed_count(command, &stdout, &stderr) == Some(0)
    {
        panic!("AW EC {id} FAILED: cargo test command passed but executed 0 tests: {command}\nstdout:\n{stdout}\nstderr:\n{stderr}");
    }
    assert!(
        output.status.success(),
        "AW EC {id} FAILED (exit {:?}): {command}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status.code()
    );
}

fn aw_ec_cargo_test_executed_count(command: &str, stdout: &str, stderr: &str) -> Option<usize> {
    if !command.contains("cargo test") {
        return None;
    }
    let mut total = 0usize;
    let mut saw_count = false;
    for line in stdout.lines().chain(stderr.lines()) {
        let Some(count) = aw_ec_parse_cargo_running_test_count(line) else {
            continue;
        };
        total = total.saturating_add(count);
        saw_count = true;
    }
    saw_count.then_some(total)
}

fn aw_ec_parse_cargo_running_test_count(line: &str) -> Option<usize> {
    let rest = line.trim().strip_prefix("running ")?;
    let number = rest
        .strip_suffix(" tests")
        .or_else(|| rest.strip_suffix(" test"))?;
    number.trim().parse().ok()
}
// CODEGEN-END
