// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-doctor-selected-plan-docker-probe
// @capability agent-native-gpu-native-dev-containers
// @claim microvm-sandbox-backend-for-vat-run
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_toml_runner -- --nocapture
// AW-EC-END

// Contract: A fake explicit MicroVm image/preset plan with an unselected Docker service invokes exactly one read-only container system status probe per invocation plus bounded read-only builder advisory probes, never Docker even when Docker is on PATH, maps services.docker_services to not_probed, and emits docker.daemon_probe.state=skipped with the selected-plan reason as provenance rather than unavailable evidence; daemon=false has no unavailable meaning because no Docker command ran.
// Contract: The builder advisory records shared_unknown ownership, automatic_cleanup=false, separate configuration/observed stats/global disk, and nonfatal unknown/timeout/probe errors without lifecycle mutation or a change to doctor runtime success; a real state is reported only when supported by the installed CLI.
// Contract: An unsupported selected MicroVm preset without a declared OCI route fails closed without Docker fallback. A selected Docker runtime, Auto image, eligible Auto preset fallback, or cluster retains normal Docker probing; a selected cluster requires Docker. Doctor never autostarts Apple Container or changes runtime through fallback.
// Contract: Recorded implementation validation: `cargo test -p vat --test vat_toml_runner -- --nocapture` passed 26/26; `cargo test -p vat --lib sandbox::microvm::tests -- --nocapture` passed 7/7.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_doctor_selected_plan_docker_probe() {
    let command = "cargo test -p vat --test vat_toml_runner -- --nocapture";
    let id = "vat-doctor-selected-plan-docker-probe";
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
