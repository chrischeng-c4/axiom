//! #2297: executable normal-Python FastAPI/SQLite reference corpus.
use std::{path::PathBuf, process::Command};

#[test]
fn python_spec_http_db_fixture_runs_unit_and_black_box_ec_tests_on_cpython() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/python_spec_http_db");
    let environment = std::env::temp_dir().join("aw-python-spec-http-db-venv");
    let output = Command::new("uv")
        .args(["run", "--locked", "--group", "test", "pytest", "-q"])
        .current_dir(&root)
        .env("UV_PROJECT_ENVIRONMENT", environment)
        .output()
        .expect("run FastAPI reference fixture with CPython");
    assert!(
        output.status.success(),
        "fixture failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let pyproject = std::fs::read_to_string(root.join("pyproject.toml")).unwrap();
    assert!(pyproject.contains("fastapi"));
    assert!(pyproject.contains("pydantic"));
    assert!(pyproject.contains("sqlalchemy"));
    assert!(root.join("src/product_api/domain/product.py").is_file());
    assert!(root.join("src/product_api/interface/http.py").is_file());
    assert!(root.join("external-contracts/README.md").is_file());
}
