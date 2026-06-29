//! Executable metadata gate for #712: debugger/introspection/profiling/tracing
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
fn debugger_readiness_tools_are_python_parseable() {
    let root = mamba_root();
    py_compile(&[
        root.join("tests/harness/cpython/tools/debugger_readiness.py"),
        root.join("tests/harness/cpython/tools/replacement_readiness.py"),
    ]);
}

#[test]
fn debugger_readiness_emits_required_state_vocabulary() {
    let output = Command::new("python3.12")
        .arg("tests/harness/cpython/tools/debugger_readiness.py")
        .args(["--json", "--show", "1"])
        .current_dir(mamba_root())
        .output()
        .expect("run debugger_readiness");
    assert!(
        output.status.code() == Some(0) || output.status.code() == Some(70),
        "debugger_readiness should be green or red, not crash\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("debugger readiness JSON");
    assert_eq!(payload["owner_issue"], "#712");
    assert_eq!(payload["schema_version"], 1);

    let counts = payload["counts"].as_object().expect("counts object");
    for key in [
        "fixtures",
        "scopes",
        "target_libs",
        "missing_target_libs",
        "semantic_classes",
        "missing_semantic_classes",
        "parse_errors",
        "pass_candidate",
        "promotion_pending",
        "runtime_failure_debt",
        "sandbox_denied",
        "unsupported_platform",
        "metadata_error",
        "runtime_ok",
        "runtime_fail",
        "runtime_timeout",
        "runtime_crash",
        "unmeasured",
        "unowned_gap_count",
        "perf_pins",
        "malformed_perf_pins",
    ] {
        assert!(counts.contains_key(key), "missing count key {key}");
    }

    for scope in [
        "frames_tracebacks",
        "inspect_disassembly",
        "debugger_hooks",
        "profiling_tracing",
        "memory_diagnostics",
    ] {
        assert!(
            payload["by_scope"].get(scope).is_some(),
            "missing debugger readiness scope {scope}"
        );
    }
    for semantic_class in [
        "frame_objects",
        "traceback_formatting",
        "inspect_signature",
        "disassembly_bytecode",
        "debugger_breakpoints",
        "trace_hooks",
        "profile_stats",
        "monitoring_events",
        "tracemalloc_snapshots",
        "warnings_capture",
    ] {
        assert!(
            payload["by_semantic_class"].get(semantic_class).is_some(),
            "missing debugger semantic class {semantic_class}"
        );
    }
    for lib in [
        "bdb",
        "pdb",
        "sys_settrace",
        "sys_setprofile",
        "traceback",
        "sys",
        "inspect",
        "tracemalloc",
        "monitoring",
        "threading_trace_profile",
    ] {
        assert!(
            payload["by_lib"].get(lib).is_some(),
            "missing debugger target lib {lib}"
        );
    }

    let commands = payload["evidence_commands"]
        .as_array()
        .expect("evidence_commands array");
    assert!(
        commands
            .iter()
            .any(|cmd| cmd.as_str().unwrap_or("").contains("sys_settrace")),
        "#712 evidence should include a real trace-hook fixture path"
    );
}

#[test]
fn replacement_readiness_uses_debugger_readiness_tool() {
    let text = fs::read_to_string(
        mamba_root().join("tests/harness/cpython/tools/replacement_readiness.py"),
    )
    .expect("read replacement_readiness.py");
    assert!(text.contains("DEBUGGER_READINESS"));
    assert!(text.contains("debugger_dimension"));
    assert!(
        !text.contains(
            "debugger/introspection/profiling/tracing surfaces are not yet replacement-ready"
        ),
        "debugger readiness must not regress to a blocked placeholder"
    );
}
