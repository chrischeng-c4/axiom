//! #2296: executable normal-Python Typer reference corpus.
use std::{path::PathBuf, process::Command};

#[test]
fn python_spec_typer_fixture_runs_unit_and_black_box_ec_tests_on_cpython() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/python_spec_typer");
    let output = Command::new("python3")
        .args(["-m", "pytest", "-q"])
        .current_dir(&root)
        .env("PYTHONPATH", root.join("src"))
        .output()
        .expect("run Typer reference fixture with CPython");
    assert!(
        output.status.success(),
        "fixture failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let pyproject = std::fs::read_to_string(root.join("pyproject.toml")).unwrap();
    assert!(pyproject.contains("typer"));
    assert!(root.join("tests/unit/test_create_task.py").is_file());
    assert!(root
        .join("external-contracts/tests/test_cli_contract.py")
        .is_file());

    let reference = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tech-design/core/logic/python-reference-typer-cli.md");
    let reference = std::fs::read_to_string(reference).unwrap();
    assert!(reference.contains("Observed reusable constructs"));
    assert!(reference.contains("Unsupported assumptions"));
}
