// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-docker-pull-json-fake
// @capability agent-native-gpu-native-dev-containers
// @claim headless-docker-command-shim
// @contract vat-docker-pull-json-fake
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_docker_shim docker_pull_json -- --nocapture
// AW-EC-END

// Contract: The deterministic contract accepts only direct pull with exact format/1..=1200-timeout selectors before one opaque image reference; it rejects empty/leading-dash/whitespace-control/URL-style `://`/leading Git-style `git@` remote forms while keeping ordinary OCI `@digest` opaque, retains raw unselected pull and docker image pull behavior, and maps only a selector-stripped request to public container image pull argv.
// Contract: It emits one bounded vat.docker.pull.v1/vat_json receipt only after normal client completion or child nonzero, marks the image not_owned_no_auto_cleanup with no registry management, safely inspects only a success, and emits no receipt on timeout/setup/capture/pipe failure.
// Contract: Current validation: cargo check passed; docker_shim lib 65/65; focused pull suite 5 plus 1 ignored (68 filtered). It does not claim Engine/API, registry auth lifecycle, provenance, digest, platform, freshness, image state, ownership, security, secret redaction, cancellation, download completion, or rollback.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_docker_pull_json_fake() {
    let command = "cargo test -p vat --test vat_docker_shim docker_pull_json -- --nocapture";
    let id = "vat-docker-pull-json-fake";
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
