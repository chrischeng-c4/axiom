//! MVP CPython 3.12 identity gate (closes #2572).
//!
//! Parent: #2530 (performance gate suite).
//!
//! Locks the contract of `scripts/cpython_identity_check.py`. The
//! checker probes the comparison Python interpreter and validates
//! that its identity matches the required CPython major.minor
//! declared by `validation/profiles/performance.toml`'s
//! `runtime_identity.required_cpython`.
//!
//! Acceptance (issue #2572):
//!
//!     1. Running against Python 3.11 or 3.13 fails the default MVP
//!        perf gate.
//!     2. JSON summary includes CPython metadata
//!        (executable, version, implementation_name).
//!     3. Existing local debug flow can still opt out intentionally
//!        via `--local-debug-override` or
//!        `MAMBA_PERF_LOCAL_DEBUG_OVERRIDE=1`.

use std::path::PathBuf;
use std::process::Command;

use serde_json::{json, Value};

fn checker_script() -> PathBuf {
    crate::common::project_root()
        .join("scripts")
        .join("cpython_identity_check.py")
}

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir()
        .join(format!("mamba-cpython-identity-{tag}-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create tempdir");
    dir
}

fn write_identity(dir: &std::path::Path, name: &str, body: &Value) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, body.to_string()).unwrap();
    path
}

fn run_checker_env(args: &[&str], env: &[(&str, &str)]) -> (i32, String, String) {
    let mut cmd = Command::new("python3");
    cmd.arg(checker_script());
    cmd.args(args);
    cmd.current_dir(crate::common::project_root());
    // Clear the override env var unless the caller asked for it.
    cmd.env_remove("MAMBA_PERF_LOCAL_DEBUG_OVERRIDE");
    for (k, v) in env {
        cmd.env(k, v);
    }
    let output = cmd.output().expect("invoke cpython_identity_check.py");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn run_checker(args: &[&str]) -> (i32, String, String) {
    run_checker_env(args, &[])
}

fn run_checker_json(args: &[&str]) -> (i32, Value) {
    let mut full = vec!["--format", "json"];
    full.extend_from_slice(args);
    let (code, stdout, stderr) = run_checker(&full);
    let payload: Value =
        serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "checker JSON parse failed (code={code}): {e}\n--stdout--\n{stdout}\n--stderr--\n{stderr}"
            )
        });
    (code, payload)
}

// ─── Acceptance 1: Python 3.11 / 3.13 fails default gate ─────────

#[test]
fn python_311_identity_fails_default_gate() {
    let dir = unique_dir("py311");
    let body = json!({
        "executable": "/fake/python3.11",
        "version": "3.11.7 (main, Jan  1 2024)",
        "version_major_minor": "3.11",
        "implementation_name": "cpython",
    });
    let path = write_identity(&dir, "id.json", &body);
    let (code, payload) =
        run_checker_json(&["--identity-json", path.to_str().unwrap()]);
    assert_eq!(code, 1, "3.11 must fail default gate; payload={payload}");
    assert_eq!(payload["matches"], false);
    assert_eq!(payload["override_active"], false);
}

#[test]
fn python_313_identity_fails_default_gate() {
    let dir = unique_dir("py313");
    let body = json!({
        "executable": "/fake/python3.13",
        "version": "3.13.0 (main, Jan  1 2025)",
        "version_major_minor": "3.13",
        "implementation_name": "cpython",
    });
    let path = write_identity(&dir, "id.json", &body);
    let (code, payload) =
        run_checker_json(&["--identity-json", path.to_str().unwrap()]);
    assert_eq!(code, 1, "3.13 must fail default gate; payload={payload}");
    assert_eq!(payload["matches"], false);
}

#[test]
fn pypy_312_identity_fails_default_gate_even_with_matching_major_minor() {
    let dir = unique_dir("pypy");
    let body = json!({
        "executable": "/fake/pypy3.12",
        "version": "3.12.4 [PyPy 7.3.x]",
        "version_major_minor": "3.12",
        "implementation_name": "pypy",
    });
    let path = write_identity(&dir, "id.json", &body);
    let (code, payload) =
        run_checker_json(&["--identity-json", path.to_str().unwrap()]);
    assert_eq!(
        code, 1,
        "non-CPython implementation must fail even on 3.12; payload={payload}"
    );
    assert_eq!(payload["matches"], false);
}

#[test]
fn cpython_312_identity_passes_default_gate() {
    let dir = unique_dir("cpython312");
    let body = json!({
        "executable": "/fake/python3.12",
        "version": "3.12.4 (main, Jul 19 2024)",
        "version_major_minor": "3.12",
        "implementation_name": "cpython",
    });
    let path = write_identity(&dir, "id.json", &body);
    let (code, payload) =
        run_checker_json(&["--identity-json", path.to_str().unwrap()]);
    assert_eq!(code, 0, "CPython 3.12 must pass; payload={payload}");
    assert_eq!(payload["matches"], true);
    assert_eq!(payload["override_active"], false);
}

// ─── Acceptance 2: JSON summary includes CPython metadata ────────

#[test]
fn json_summary_includes_executable_version_and_implementation() {
    let dir = unique_dir("json-shape");
    let body = json!({
        "executable": "/abs/python3",
        "version": "3.12.4 (main, ...)",
        "version_major_minor": "3.12",
        "implementation_name": "cpython",
    });
    let path = write_identity(&dir, "id.json", &body);
    let (_code, payload) =
        run_checker_json(&["--identity-json", path.to_str().unwrap()]);
    assert_eq!(payload["executable"], "/abs/python3");
    assert_eq!(payload["version_major_minor"], "3.12");
    assert_eq!(payload["implementation_name"], "cpython");
    assert_eq!(payload["required_cpython"], "3.12");
    assert_eq!(payload["required_implementation"], "cpython");
    assert!(payload.get("matches").is_some());
    assert!(payload.get("override_active").is_some());
    assert!(payload.get("exit_code").is_some());
}

#[test]
fn json_summary_field_set_is_locked() {
    let dir = unique_dir("locked-fields");
    let body = json!({
        "executable": "/x", "version": "3.12.0", "version_major_minor": "3.12",
        "implementation_name": "cpython",
    });
    let path = write_identity(&dir, "id.json", &body);
    let (_, payload) = run_checker_json(&["--identity-json", path.to_str().unwrap()]);
    for k in &[
        "executable",
        "version",
        "version_major_minor",
        "implementation_name",
        "required_cpython",
        "required_implementation",
        "matches",
        "override_active",
        "exit_code",
        "schema_version",
    ] {
        assert!(
            payload.get(k).is_some(),
            "JSON summary missing locked field {k}"
        );
    }
}

// ─── Acceptance 3: local debug override ──────────────────────────

#[test]
fn local_debug_override_flag_turns_mismatch_into_pass() {
    let dir = unique_dir("override-flag");
    let body = json!({
        "executable": "/fake/python3.13",
        "version": "3.13.0",
        "version_major_minor": "3.13",
        "implementation_name": "cpython",
    });
    let path = write_identity(&dir, "id.json", &body);
    let (code, payload) = run_checker_json(&[
        "--identity-json",
        path.to_str().unwrap(),
        "--local-debug-override",
    ]);
    assert_eq!(code, 0, "override must convert mismatch to pass");
    assert_eq!(payload["matches"], false);
    assert_eq!(payload["override_active"], true);
}

#[test]
fn local_debug_override_env_var_turns_mismatch_into_pass() {
    let dir = unique_dir("override-env");
    let body = json!({
        "executable": "/fake/python3.11",
        "version": "3.11.7",
        "version_major_minor": "3.11",
        "implementation_name": "cpython",
    });
    let path = write_identity(&dir, "id.json", &body);
    let (code, stdout, _stderr) = run_checker_env(
        &[
            "--identity-json",
            path.to_str().unwrap(),
            "--format",
            "json",
        ],
        &[("MAMBA_PERF_LOCAL_DEBUG_OVERRIDE", "1")],
    );
    let payload: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(code, 0, "env override must convert mismatch to pass");
    assert_eq!(payload["override_active"], true);
}

#[test]
fn text_output_warns_when_override_is_active() {
    let dir = unique_dir("override-warning");
    let body = json!({
        "executable": "/fake/python3.11",
        "version": "3.11.7",
        "version_major_minor": "3.11",
        "implementation_name": "cpython",
    });
    let path = write_identity(&dir, "id.json", &body);
    let (code, _stdout, stderr) = run_checker(&[
        "--identity-json",
        path.to_str().unwrap(),
        "--local-debug-override",
        "--format",
        "text",
    ]);
    assert_eq!(code, 0, "override path exits 0");
    assert!(
        stderr.contains("MISMATCH") && stderr.contains("override active"),
        "text output must surface that override is masking a real mismatch; got {stderr}"
    );
    assert!(
        stderr.contains("release runs must NOT"),
        "text output must warn that override is local-debug only; got {stderr}"
    );
}

// ─── Identity JSON loader edge cases ─────────────────────────────

#[test]
fn checker_exits_101_when_identity_json_missing() {
    let (code, _stdout, stderr) =
        run_checker(&["--identity-json", "/tmp/cpython-id-does-not-exist.json"]);
    assert_eq!(code, 101);
    assert!(stderr.contains("identity JSON missing"));
}

#[test]
fn checker_exits_101_when_identity_json_invalid() {
    let dir = unique_dir("bad-json");
    let path = dir.join("id.json");
    std::fs::write(&path, "{ not valid json").unwrap();
    let (code, _stdout, stderr) =
        run_checker(&["--identity-json", path.to_str().unwrap()]);
    assert_eq!(code, 101);
    assert!(
        stderr.contains("invalid"),
        "stderr must mention invalid JSON; got {stderr}"
    );
}

#[test]
fn version_major_minor_is_extracted_when_omitted_from_identity_json() {
    let dir = unique_dir("derive-mm");
    let body = json!({
        "executable": "/x",
        "version": "3.12.4 (main, Jul 19 2024)",
        // version_major_minor intentionally omitted — checker derives it.
        "implementation_name": "cpython",
    });
    let path = write_identity(&dir, "id.json", &body);
    let (code, payload) =
        run_checker_json(&["--identity-json", path.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert_eq!(payload["version_major_minor"], "3.12");
}

// ─── Probe path against the real python3 on PATH ─────────────────

#[test]
fn probe_against_real_python3_emits_some_identity() {
    // We can't assert the version (CI runners differ); we only
    // assert the JSON shape and that probing succeeds.
    let (_code, payload) = run_checker_json(&[]);
    assert!(payload["executable"].as_str().map_or(false, |s| !s.is_empty()));
    assert!(payload["version"].as_str().map_or(false, |s| !s.is_empty()));
    assert!(payload["implementation_name"]
        .as_str()
        .map_or(false, |s| !s.is_empty()));
}

#[test]
fn checker_exits_101_when_python_binary_does_not_exist() {
    let (code, _stdout, stderr) = run_checker(&["--python", "/no/such/python-binary-3.99"]);
    assert_eq!(code, 101);
    assert!(
        stderr.contains("failed to probe") || stderr.contains("/no/such"),
        "stderr must explain the probe failure; got {stderr}"
    );
}

// ─── CLI surface ─────────────────────────────────────────────────

#[test]
fn checker_help_documents_python_identity_and_override_flags() {
    let (code, stdout, _stderr) = run_checker(&["--help"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("--python"));
    assert!(stdout.contains("--identity-json"));
    assert!(stdout.contains("--local-debug-override"));
    assert!(stdout.contains("--format"));
}
