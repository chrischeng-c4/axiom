//! Executable metadata gate for #713: concurrency/free-threaded readiness must
//! stay strict, machine-readable, and wired into replacement readiness.

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
fn concurrency_readiness_tools_are_python_parseable() {
    let root = mamba_root();
    py_compile(&[
        root.join("tests/harness/cpython/tools/concurrency_readiness.py"),
        root.join("tests/harness/cpython/tools/replacement_readiness.py"),
    ]);
}

#[test]
fn concurrency_readiness_emits_required_state_vocabulary() {
    let output = Command::new("python3.12")
        .arg("tests/harness/cpython/tools/concurrency_readiness.py")
        .args(["--json", "--show", "1"])
        .current_dir(mamba_root())
        .output()
        .expect("run concurrency_readiness");
    assert!(
        output.status.code() == Some(0) || output.status.code() == Some(70),
        "concurrency_readiness should be green or red, not crash\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("concurrency readiness JSON");
    assert_eq!(payload["owner_issue"], "#713");
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
    assert!(
        counts
            .get("fixtures")
            .and_then(|value| value.as_u64())
            .unwrap_or(0)
            > 0,
        "#713 readiness must have a nonzero denominator"
    );

    for scope in [
        "thread_primitives",
        "async_context",
        "process_parallelism",
        "synchronization_queues",
        "signal_interaction",
    ] {
        assert!(
            payload["by_scope"].get(scope).is_some(),
            "missing concurrency readiness scope {scope}"
        );
    }
    for semantic_class in [
        "thread_lifecycle",
        "synchronization_locks",
        "queue_semantics",
        "executor_future",
        "multiprocessing_processes",
        "asyncio_tasks_event_loop",
        "contextvars_propagation",
        "signal_thread_interaction",
        "subprocess_process_io",
        "race_deadlock_determinism",
        "free_threaded_no_gil_contract",
    ] {
        assert!(
            payload["by_semantic_class"].get(semantic_class).is_some(),
            "missing concurrency semantic class {semantic_class}"
        );
    }
    for lib in [
        "threading",
        "_thread",
        "asyncio",
        "contextvars",
        "multiprocessing",
        "subprocess",
        "concurrent_futures",
        "queue",
        "signal",
    ] {
        assert!(
            payload["by_lib"].get(lib).is_some(),
            "missing concurrency target lib {lib}"
        );
    }

    let commands = payload["evidence_commands"]
        .as_array()
        .expect("evidence_commands array");
    assert!(
        commands
            .iter()
            .any(|cmd| cmd.as_str().unwrap_or("").contains("threading")),
        "#713 evidence should include a real threading fixture path"
    );
}

#[test]
fn replacement_readiness_uses_concurrency_readiness_tool() {
    let text = fs::read_to_string(
        mamba_root().join("tests/harness/cpython/tools/replacement_readiness.py"),
    )
    .expect("read replacement_readiness.py");
    assert!(text.contains("CONCURRENCY_READINESS"));
    assert!(text.contains("concurrency_dimension"));
    assert!(
        !text.contains(
            "concurrency and free-threaded semantics still need strict replacement gates"
        ),
        "concurrency readiness must not regress to a blocked placeholder"
    );
}
