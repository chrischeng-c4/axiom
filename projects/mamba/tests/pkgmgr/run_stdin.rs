/// CLI integration tests for `mamba run -` (stdin sentinel).
///
/// REQ: R1 — reading source from stdin when argument is "-".
/// REQ: R2 — exit behaviour mirrors `mamba run <file.py>`.
/// REQ: R5 — CLI integration test exercising the stdin path.

use std::io::Write as _;
use std::process::{Command, Stdio};

/// Resolve the path to the mamba binary at test time.
fn mamba_bin() -> std::path::PathBuf {
    // CARGO_BIN_EXE_mamba is set by Cargo for integration tests when the
    // crate declares a [[bin]] with name "mamba".
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

// REQ: R1, R2, R5
#[test]
fn run_dash_reads_source_from_stdin_and_prints_output() {
    let bin = mamba_bin();
    let mut child = Command::new(&bin)
        .args(["run", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn mamba binary");

    // Write the script to stdin then close it (drop triggers EOF).
    {
        let stdin = child.stdin.as_mut().expect("stdin not captured");
        stdin
            .write_all(b"print(\"hi\")\n")
            .expect("failed to write to stdin");
    }

    let output = child.wait_with_output().expect("failed to wait for child");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "expected exit code 0, got {}\nstdout: {stdout}\nstderr: {stderr}",
        output.status
    );
    assert!(
        stdout.contains("hi"),
        "expected stdout to contain 'hi', got: {stdout:?}\nstderr: {stderr}"
    );
}

// REQ: R4 — existing file-path behaviour must be preserved (regression guard).
#[test]
fn run_file_path_still_works_after_stdin_change() {
    let dir = tempfile::tempdir().expect("tempdir");
    let script = dir.path().join("hello.py");
    std::fs::write(&script, "print(\"hello from file\")\n").expect("write script");

    let bin = mamba_bin();
    let output = Command::new(&bin)
        .args(["run", script.to_str().unwrap()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to run mamba binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "expected exit code 0 for file path run, got {}\nstdout: {stdout}\nstderr: {stderr}",
        output.status
    );
    assert!(
        stdout.contains("hello from file"),
        "expected stdout to contain 'hello from file', got: {stdout:?}\nstderr: {stderr}"
    );
}
