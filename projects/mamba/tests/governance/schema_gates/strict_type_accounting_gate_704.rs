//! Executable metadata gate for #704: strict-type accounting must stay
//! machine-readable and wired into replacement readiness.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn project_root() -> PathBuf {
    crate::common::project_root()
}

fn mamba_root() -> PathBuf {
    project_root()
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
fn strict_type_tools_are_python_parseable() {
    let root = mamba_root();
    py_compile(&[
        root.join("tests/harness/cpython/tools/strict_type_accounting.py"),
        root.join("tests/harness/cpython/tools/replacement_readiness.py"),
        root.join("tests/harness/cpython/tools/fixture_lint.py"),
        root.join("tests/harness/cpython/tools/type_enforce_matrix.py"),
    ]);
}

#[test]
fn fixture_lint_supports_type_facet_filter() {
    let output = Command::new("python3.12")
        .arg("tests/harness/cpython/tools/fixture_lint.py")
        .args(["--bucket", "type", "--show", "1"])
        .current_dir(mamba_root())
        .output()
        .expect("run fixture_lint type facet");
    assert!(
        output.status.success(),
        "fixture_lint --bucket type failed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("recorded="),
        "fixture_lint output should include fixture counts: {stdout}"
    );
}

#[test]
fn replacement_readiness_uses_strict_type_accounting_tool() {
    let text = fs::read_to_string(
        mamba_root().join("tests/harness/cpython/tools/replacement_readiness.py"),
    )
    .expect("read replacement_readiness.py");
    assert!(text.contains("STRICT_TYPE_ACCOUNTING"));
    assert!(text.contains("strict_type_dimension"));
    assert!(text.contains("type_enforced"));
    assert!(
        !text.contains(
            "strict-type denominator and verified divergence accounting are not yet integrated"
        ),
        "strict-type readiness must not regress to a blocked placeholder"
    );
}

#[test]
fn declared_type_divergences_have_machine_owner_refs() {
    let path = mamba_root().join("tests/harness/cpython/config/type_divergences.txt");
    let text = fs::read_to_string(path).expect("read type_divergences.txt");
    let mut current_owner = false;
    let mut entries = 0usize;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') {
            if line.contains("owner:") && line.contains('#') {
                current_owner = true;
            }
            continue;
        }
        entries += 1;
        assert!(
            current_owner,
            "type divergence entry lacks preceding '# owner: #<issue>' line: {line}"
        );
        assert!(
            line.starts_with("projects/mamba/tests/cpython/"),
            "type divergence must use repo-relative fixture path: {line}"
        );
        current_owner = false;
    }
    assert!(
        entries > 0,
        "expected at least one declared type divergence"
    );
}

#[test]
fn generated_typeshed_denominator_header_is_present() {
    let text = fs::read_to_string(mamba_root().join("src/types/stdlib_sigs_generated.rs"))
        .expect("read generated stdlib sig table");
    assert!(text.contains("rows:"));
    assert!(text.contains("enforceable (scalar):"));
    assert!(text.contains("unknown-skipped:"));
}
