use agentic_workflow::services::python_td::compile_python_td_project;
use std::fs;

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn python_td_compiler_reference_projects_compile_without_execution() {
    for name in ["python_spec_typer", "python_spec_http_db"] {
        let ir = compile_python_td_project(&fixture(name)).unwrap();
        assert_eq!(ir.schema_version, "aw.python-td-ir.v1");
        assert!(!ir.modules.is_empty());
        assert!(ir.semantic_digest.starts_with("sha256:"));
    }

    let temporary = tempfile::tempdir().unwrap();
    fs::create_dir_all(temporary.path().join("src/demo/domain")).unwrap();
    fs::write(
        temporary.path().join("src/demo/domain/model.py"),
        "raise RuntimeError('must never execute while parsing')\n\nclass Model:\n    pass\n",
    )
    .unwrap();
    assert!(compile_python_td_project(temporary.path()).is_ok());
}

#[test]
fn python_td_compiler_digest_ignores_formatting_only_edits() {
    let temporary = tempfile::tempdir().unwrap();
    fs::create_dir_all(temporary.path().join("src/demo/application")).unwrap();
    let source = temporary.path().join("src/demo/application/create.py");
    fs::write(
        &source,
        "def create(title: str) -> str:\n    return title\n",
    )
    .unwrap();
    let before = compile_python_td_project(temporary.path()).unwrap();

    fs::write(
        &source,
        "\n\ndef create( title : str ) -> str :\n\n    return title\n",
    )
    .unwrap();
    let after = compile_python_td_project(temporary.path()).unwrap();

    assert_eq!(before.semantic_digest, after.semantic_digest);
    assert_eq!(
        before.modules[0].declarations[0].id,
        after.modules[0].declarations[0].id
    );
}

#[test]
fn python_td_compiler_keeps_explicit_artifact_identity_across_a_projection_move() {
    let temporary = tempfile::tempdir().unwrap();
    let first = temporary.path().join("src/demo/domain/issue_invoice.py");
    fs::create_dir_all(first.parent().unwrap()).unwrap();
    fs::write(
        &first,
        "__aw_artifact_id__ = \"artifact:billing/issue-invoice\"\n\ndef issue_invoice() -> None:\n    pass\n",
    )
    .unwrap();
    let before = compile_python_td_project(temporary.path()).unwrap();

    let second = temporary.path().join("src/demo/domain/renamed_invoice.py");
    fs::rename(&first, &second).unwrap();
    let after = compile_python_td_project(temporary.path()).unwrap();

    assert_eq!(before.modules[0].id, "artifact:billing/issue-invoice");
    assert_eq!(
        before.modules[0].artifact_id.as_deref(),
        Some("artifact:billing/issue-invoice")
    );
    assert_eq!(before.modules[0].id, after.modules[0].id);
    assert_eq!(
        before.modules[0].declarations[0].id,
        after.modules[0].declarations[0].id
    );
    assert_eq!(before.semantic_digest, after.semantic_digest);
    assert_ne!(before.modules[0].path, after.modules[0].path);
}

#[test]
fn python_td_compiler_rejects_invalid_explicit_artifact_identity() {
    let temporary = tempfile::tempdir().unwrap();
    let source = temporary.path().join("src/demo/domain/model.py");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(
        &source,
        "__aw_artifact_id__ = \"src/demo/domain/model.py\"\n",
    )
    .unwrap();

    let error = compile_python_td_project(temporary.path())
        .unwrap_err()
        .to_string();
    assert!(error.contains("[invalid-artifact-id]"));
    assert!(error.contains("artifact:<context>/<name>"));
}

#[test]
fn python_td_compiler_reports_source_spans_for_rejected_semantics() {
    let temporary = tempfile::tempdir().unwrap();
    fs::create_dir_all(temporary.path().join("src/demo/application")).unwrap();
    let source = temporary.path().join("src/demo/application/create.py");
    fs::write(&source, "factory = lambda value: value\n").unwrap();

    let error = compile_python_td_project(temporary.path())
        .unwrap_err()
        .to_string();
    assert!(error.contains("[unsupported-syntax]"));
    assert!(error.contains("create.py:1:11"));
    assert!(error.contains("replace lambda"));
}

#[test]
fn python_td_compiler_rejects_unresolved_local_imports() {
    let temporary = tempfile::tempdir().unwrap();
    fs::create_dir_all(temporary.path().join("src/demo/application")).unwrap();
    fs::write(
        temporary.path().join("src/demo/application/create.py"),
        "from demo.missing import value\n",
    )
    .unwrap();

    let error = compile_python_td_project(temporary.path())
        .unwrap_err()
        .to_string();
    assert!(error.contains("[unresolved-local-import]"));
    assert!(error.contains("create.py:1:1"));
    assert!(error.contains("demo.missing"));
}

#[test]
fn python_td_compiler_routes_python_projects_through_td_ast_and_check() {
    let root = fixture("python_spec_http_db");
    let ast = std::process::Command::new(env!("CARGO_BIN_EXE_aw"))
        .args(["td", "ast", root.to_str().unwrap(), "--pretty"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(
        ast.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&ast.stderr)
    );
    let ir: serde_json::Value = serde_json::from_slice(&ast.stdout).unwrap();
    assert_eq!(ir["schema_version"], "aw.python-td-ir.v1");

    let check = std::process::Command::new(env!("CARGO_BIN_EXE_aw"))
        .args(["td", "check", root.to_str().unwrap(), "--json"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&check.stderr)
    );
    let checked: serde_json::Value = serde_json::from_slice(&check.stdout).unwrap();
    assert_eq!(checked["semantic_digest"], ir["semantic_digest"]);
}

#[test]
fn python_td_check_fails_closed_when_configured_project_has_no_python_sources() {
    let root = tempfile::tempdir().unwrap();
    let project = root.path().join("projects/demo");
    let tech_design = project.join("tech-design");
    fs::create_dir_all(&tech_design).unwrap();
    fs::write(
        root.path().join("aw.toml"),
        r#"
[[projects]]
name = "demo"
path = "projects/demo"
"#,
    )
    .unwrap();
    fs::write(
        tech_design.join("legacy.md"),
        "# Legacy TD\n\nThis must not make a Python TD check pass.\n",
    )
    .unwrap();

    let check = std::process::Command::new(env!("CARGO_BIN_EXE_aw"))
        .args([
            "td",
            "check",
            tech_design.to_str().unwrap(),
            "--project",
            "demo",
        ])
        .current_dir(root.path())
        .output()
        .unwrap();

    assert!(
        !check.status.success(),
        "stdout={}",
        String::from_utf8_lossy(&check.stdout)
    );
    let stderr = String::from_utf8_lossy(&check.stderr);
    assert!(
        stderr.contains("Python TD compiler found no .py files"),
        "stderr={stderr}"
    );
}
