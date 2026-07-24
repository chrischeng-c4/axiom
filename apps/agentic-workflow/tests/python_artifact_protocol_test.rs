// HANDWRITE-BEGIN gap="missing-generator:e2e-test:python-artifact-project-protocol" tracker="#2290" reason="The fixture suite runs real CPython project entrypoints and exercises malformed, stale, failure, and timeout boundaries."
//! CPython project protocol acceptance tests.
//!
//! @spec apps/agentic-workflow/tech-design/core/logic/python-artifact-project-protocol.md#unit-test

use agentic_workflow::services::python_artifact::{
    discover_python_artifact_project, run_python_artifact_project, PythonArtifactRunOptions,
    PythonArtifactStatus,
};
use std::{fs, path::Path, time::Duration};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

fn fixture_root(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/python_artifact_protocol")
        .join(name)
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let destination_path = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &destination_path);
        } else {
            fs::copy(entry.path(), destination_path).unwrap();
        }
    }
}

fn copied_fixture(name: &str) -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    copy_tree(&fixture_root(name), temp.path());
    temp
}

#[test]
fn python_artifact_protocol_discovers_and_runs_a_cpython_project() {
    let fixture = copied_fixture("success");
    let project = discover_python_artifact_project(fixture.path()).unwrap();
    let initial_source_digest = project.source_digest().to_string();
    let initial_dependency_digest = project.dependency_lock_digest().to_string();

    fs::create_dir_all(fixture.path().join("src/__pycache__")).unwrap();
    fs::write(
        fixture
            .path()
            .join("src/__pycache__/runner.cpython-313.pyc"),
        "cache bytes",
    )
    .unwrap();
    fs::create_dir_all(fixture.path().join(".venv/lib/python3.13/site-packages")).unwrap();
    fs::write(
        fixture
            .path()
            .join(".venv/lib/python3.13/site-packages/ignored.py"),
        "ignored = True\n",
    )
    .unwrap();

    let rediscovered = discover_python_artifact_project(fixture.path()).unwrap();
    assert_eq!(rediscovered.source_digest(), initial_source_digest);
    assert_eq!(
        rediscovered.dependency_lock_digest(),
        initial_dependency_digest
    );

    let result =
        run_python_artifact_project(&rediscovered, "check", &PythonArtifactRunOptions::default())
            .unwrap();
    assert_eq!(result.status, PythonArtifactStatus::Passed);
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.evidence_paths.len(), 1);
    assert!(result.evidence_paths[0].ends_with("evidence/result.json"));
}

#[test]
fn python_artifact_protocol_accepts_a_structured_terminal_failure() {
    let fixture = copied_fixture("structured_failure");
    let project = discover_python_artifact_project(fixture.path()).unwrap();
    let result =
        run_python_artifact_project(&project, "check", &PythonArtifactRunOptions::default())
            .unwrap();

    assert_eq!(result.status, PythonArtifactStatus::Failed);
    assert_eq!(result.exit_code, 1);
    assert_eq!(result.evidence_paths.len(), 1);
}

#[test]
fn python_artifact_protocol_rejects_malformed_stdout() {
    let fixture = copied_fixture("malformed_output");
    let project = discover_python_artifact_project(fixture.path()).unwrap();
    let error =
        run_python_artifact_project(&project, "check", &PythonArtifactRunOptions::default())
            .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("malformed Python artifact result envelope"),
        "unexpected error: {error:#}"
    );
}

#[test]
fn python_artifact_protocol_rejects_stale_source_digest() {
    let fixture = copied_fixture("stale_digest");
    let project = discover_python_artifact_project(fixture.path()).unwrap();
    let error =
        run_python_artifact_project(&project, "check", &PythonArtifactRunOptions::default())
            .unwrap_err();

    assert!(
        error.to_string().contains("source digest mismatch"),
        "unexpected error: {error:#}"
    );
}

#[test]
fn python_artifact_protocol_rejects_empty_evidence() {
    let fixture = copied_fixture("success");
    let project = discover_python_artifact_project(fixture.path()).unwrap();
    fs::write(fixture.path().join("evidence/result.json"), "").unwrap();
    let error =
        run_python_artifact_project(&project, "check", &PythonArtifactRunOptions::default())
            .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("result evidence must be non-empty"),
        "unexpected error: {error:#}"
    );
}

#[test]
fn python_artifact_protocol_terminates_a_timed_out_runner() {
    let fixture = copied_fixture("timeout");
    let project = discover_python_artifact_project(fixture.path()).unwrap();
    let options = PythonArtifactRunOptions {
        timeout: Duration::from_millis(200),
        ..PythonArtifactRunOptions::default()
    };
    let error = run_python_artifact_project(&project, "check", &options).unwrap_err();

    assert!(
        error.to_string().contains("timed out"),
        "unexpected error: {error:#}"
    );
}

#[cfg(unix)]
#[test]
fn python_artifact_protocol_rejects_a_non_cpython_interpreter() {
    let fixture = copied_fixture("success");
    let project = discover_python_artifact_project(fixture.path()).unwrap();
    let fake_python = fixture.path().join("fake-python");
    fs::write(&fake_python, "#!/bin/sh\nprintf 'PyPy\\n'\n").unwrap();
    let mut permissions = fs::metadata(&fake_python).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_python, permissions).unwrap();

    let options = PythonArtifactRunOptions {
        python_executable: fake_python,
        ..PythonArtifactRunOptions::default()
    };
    let error = run_python_artifact_project(&project, "check", &options).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("requires CPython, observed `PyPy`"),
        "unexpected error: {error:#}"
    );
}

#[cfg(unix)]
#[test]
fn python_artifact_protocol_rejects_configured_symlink_components() {
    let fixture = copied_fixture("success");
    let source = fixture.path().join("src");
    let real_source = fixture.path().join("real-src");
    fs::rename(&source, &real_source).unwrap();
    std::os::unix::fs::symlink("real-src", &source).unwrap();

    let error = discover_python_artifact_project(fixture.path()).unwrap_err();
    assert!(
        error.to_string().contains("may not traverse a symlink"),
        "unexpected error: {error:#}"
    );
}
// HANDWRITE-END
