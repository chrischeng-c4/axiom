// SPEC-MANAGED: projects/lumen/external-contracts/security-hardening/security/access-control.md#lumen-security-hardening-access-control
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-security-hardening-access-control
// @capability security-hardening
// @claim role-based-authz-matrix-per-route
// @contract search-security-rbac-and-limit
// @category security
// @required_for_production true
// @command cargo test -p lumen --test authz_matrix_e2e --test api_e2e -- --nocapture
// AW-EC-END

// Contract: FILTERING: search over a collection the token cannot read returns 403; results never leak rows outside the caller's RBAC scope.
// Contract: PAGINATION: bulk/index requests over MAX_INDEX_ITEMS return 413; result pages are bounded (cursor), not unbounded.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_security_hardening_access_control() {
    let command = "cargo test -p lumen --test authz_matrix_e2e --test api_e2e -- --nocapture";
    let id = "lumen-security-hardening-access-control";
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
