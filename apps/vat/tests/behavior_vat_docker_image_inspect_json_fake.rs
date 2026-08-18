// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-docker-image-inspect-json-fake
// @capability agent-native-gpu-native-dev-containers
// @claim headless-docker-command-shim
// @contract vat-docker-image-inspect-json-fake
// @category behavior
// @required_for_production true
// @command RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim docker_image_inspect_json -- --nocapture
// AW-EC-END

// Contract: The deterministic fake contract accepts only direct docker image inspect JSON with one selector before one safe opaque IMAGE, strips the selector, invokes only container image inspect IMAGE, and byte-for-byte replays one validated Apple-native JSON document.
// Contract: Templates, --, extra references, and every other option fail before Apple Container; a valid native document with a nonzero child exit preserves that status, while malformed, oversized, or escaped-pipe capture suppresses raw stdout under five-second bounded isolated cleanup.
// Contract: Recorded validation: cargo check passed; canonical cargo test -p vat --lib docker_shim -- --nocapture passed 58/58; this focused integration passed 4/4 with 1 ignored. It does not claim Docker image-inspect schema/templates/Engine API, provenance, security, registry, build-completion, readiness, or secret redaction.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_docker_image_inspect_json_fake() {
    let command =
        "RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim docker_image_inspect_json -- --nocapture";
    let id = "vat-docker-image-inspect-json-fake";
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
