// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-docker-exec-json-fake
// @capability agent-native-gpu-native-dev-containers
// @claim headless-docker-command-shim
// @contract vat-docker-exec-json-fake
// @category behavior
// @required_for_production true
// @command RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim docker_exec_json -- --nocapture
// AW-EC-END

// Contract: The deterministic fake contract accepts only direct/container exec JSON with one format and one 1..=1200 timeout before a safe id, a literal Docker-facing delimiter, and a nonempty raw command; raw/unformatted exec remains generic.
// Contract: VAT removes the Docker-only delimiter and normalizes to Apple `container exec CONTAINER COMMAND [ARG...]`, then emits one `vat.docker.exec.v1` / `vat_json` wrapper with separate serialized-64-KiB-capped suffixes, `timeout_scope=host-container-client-observation`, and a safe inspect next.
// Contract: Ordinary child failure preserves wrapper+exit; timeout or setup/capture failure emits no partial wrapper. Deterministic validation passed: docker_shim library 54/54 and focused docker_exec_json 4/4. The host timeout does not claim guest command termination; no Docker Engine parity, generic runtime, Compose, or Kubernetes claim follows.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_docker_exec_json_fake() {
    let command =
        "RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim docker_exec_json -- --nocapture";
    let id = "vat-docker-exec-json-fake";
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
