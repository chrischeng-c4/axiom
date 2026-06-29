//! Executable metadata gate for #710: platform/OS/process/network/TLS
//! readiness must stay machine-readable and wired into replacement readiness.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn mamba_root() -> PathBuf {
    crate::common::project_root()
}

fn py_compile(paths: &[PathBuf]) {
    let output = Command::new("python3.12")
        .arg("-m")
        .arg("py_compile")
        .args(paths)
        .current_dir(mamba_root())
        .output()
        .expect("run py_compile");
    assert!(
        output.status.success(),
        "py_compile failed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn platform_readiness_tools_are_python_parseable() {
    let root = mamba_root();
    py_compile(&[
        root.join("tests/harness/cpython/tools/platform_readiness.py"),
        root.join("tests/harness/cpython/tools/replacement_readiness.py"),
    ]);
}

#[test]
fn platform_readiness_emits_required_state_vocabulary() {
    let output = Command::new("python3.12")
        .arg("tests/harness/cpython/tools/platform_readiness.py")
        .args(["--json", "--show", "1"])
        .current_dir(mamba_root())
        .output()
        .expect("run platform_readiness");
    assert!(
        output.status.code() == Some(0) || output.status.code() == Some(70),
        "platform_readiness should be green or red, not crash\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("platform readiness JSON");
    assert_eq!(payload["owner_issue"], "#710");
    assert_eq!(payload["schema_version"], 1);

    let counts = payload["counts"].as_object().expect("counts object");
    for key in [
        "pass_candidate",
        "runtime_failure_debt",
        "sandbox_denied",
        "unsupported_platform",
        "runtime_ok",
        "runtime_fail",
        "runtime_timeout",
        "runtime_crash",
        "unmeasured",
    ] {
        assert!(counts.contains_key(key), "missing count key {key}");
    }
    for scope in [
        "filesystem_environment",
        "process_signal",
        "network_io",
        "tls",
    ] {
        assert!(
            payload["by_scope"].get(scope).is_some(),
            "missing platform readiness scope {scope}"
        );
    }
}

#[test]
fn replacement_readiness_uses_platform_readiness_tool() {
    let text = fs::read_to_string(
        mamba_root().join("tests/harness/cpython/tools/replacement_readiness.py"),
    )
    .expect("read replacement_readiness.py");
    assert!(text.contains("PLATFORM_READINESS"));
    assert!(text.contains("platform_dimension"));
    assert!(text.contains("\"--json\""));
    assert!(
        !text.contains("platform/OS/process/network/TLS coverage is not yet proven as a replacement gate"),
        "platform readiness must not regress to a blocked placeholder"
    );
}
