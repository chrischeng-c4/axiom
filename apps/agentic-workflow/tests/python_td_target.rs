use agentic_workflow::services::{
    python_td::compile_python_td_project, python_td_rust_target::emit_python_td_rust_target,
    python_td_target::emit_python_td_target,
    python_td_typescript_target::emit_python_td_typescript_target,
};
use std::{fs, path::PathBuf, process::Command};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn todo_td() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap()
        .join("examples/todo-app/td")
}

fn snapshot(root: &std::path::Path) -> Vec<(String, Vec<u8>)> {
    let mut files = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| {
            (
                entry
                    .path()
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                fs::read(entry.path()).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

fn seed_existing_project(root: &std::path::Path, manifest: &str) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join(manifest), b"existing manifest\n").unwrap();
    fs::write(root.join("src/existing.txt"), b"existing source\n").unwrap();
}

#[test]
fn python_td_target_generates_deterministic_packages_and_native_tests() {
    for fixture_name in ["python_spec_typer", "python_spec_http_db"] {
        let ir = compile_python_td_project(&fixture(fixture_name)).unwrap();
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        let first_target = emit_python_td_target(&ir, first.path()).unwrap();
        let second_target = emit_python_td_target(&ir, second.path()).unwrap();

        assert_eq!(
            first_target, second_target,
            "{fixture_name} target manifest drifted"
        );
        assert_eq!(
            snapshot(first.path()),
            snapshot(second.path()),
            "{fixture_name} cold output drifted"
        );
        assert!(first
            .path()
            .join("tests/unit/test_generated_inventory.py")
            .is_file());
        assert!(first.path().join("pyproject.toml").is_file());
        assert!(!first.path().join("external-contracts").exists());
        let pyproject = fs::read_to_string(first.path().join("pyproject.toml")).unwrap();
        assert!(pyproject.contains("build-backend = \"setuptools.build_meta\""));
        assert!(pyproject.contains("where = [\"src\"]"));

        let result = Command::new("python3")
            .args(["-m", "unittest", "discover", "-s", "tests/unit"])
            .current_dir(first.path())
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{fixture_name} generated tests failed:\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

#[test]
fn native_targets_reject_existing_projects_before_any_write() {
    let ir = compile_python_td_project(&fixture("python_spec_typer")).unwrap();

    let python = tempfile::tempdir().unwrap();
    seed_existing_project(python.path(), "pyproject.toml");
    let python_before = snapshot(python.path());
    let python_error = emit_python_td_target(&ir, python.path()).unwrap_err();
    assert!(python_error
        .to_string()
        .contains("non-empty existing project"));
    assert_eq!(snapshot(python.path()), python_before);

    let rust = tempfile::tempdir().unwrap();
    seed_existing_project(rust.path(), "Cargo.toml");
    let rust_before = snapshot(rust.path());
    let rust_error = emit_python_td_rust_target(&ir, rust.path()).unwrap_err();
    assert!(rust_error
        .to_string()
        .contains("non-empty existing project"));
    assert_eq!(snapshot(rust.path()), rust_before);

    let typescript = tempfile::tempdir().unwrap();
    seed_existing_project(typescript.path(), "package.json");
    let typescript_before = snapshot(typescript.path());
    let typescript_error = emit_python_td_typescript_target(&ir, typescript.path()).unwrap_err();
    assert!(typescript_error
        .to_string()
        .contains("non-empty existing project"));
    assert_eq!(snapshot(typescript.path()), typescript_before);
}

#[test]
fn cb_gen_python_target_routes_to_native_emitter() {
    let output = tempfile::tempdir().unwrap();
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap()
        .to_path_buf();
    let result = Command::new(env!("CARGO_BIN_EXE_aw"))
        .args([
            "cb",
            "gen",
            "--target",
            "python",
            "--source-root",
            fixture("python_spec_typer").to_str().unwrap(),
            "--output-dir",
            output.path().to_str().unwrap(),
        ])
        .current_dir(workspace_root)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "aw cb gen --target python failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.path().join("src/task_cli/__init__.py").is_file());
    assert!(output
        .path()
        .join("tests/unit/test_generated_inventory.py")
        .is_file());
}

#[test]
fn python_ir_rust_target_is_compiling_cold_and_fail_closed() {
    let ir = compile_python_td_project(&fixture("python_spec_typer")).unwrap();
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let first_target = emit_python_td_rust_target(&ir, first.path()).unwrap();
    let second_target = emit_python_td_rust_target(&ir, second.path()).unwrap();
    assert_eq!(first_target, second_target);
    assert_eq!(snapshot(first.path()), snapshot(second.path()));
    let check = Command::new("cargo")
        .args(["test", "--quiet"])
        .current_dir(first.path())
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let bad = tempfile::tempdir().unwrap();
    let mut unsupported = ir.clone();
    unsupported
        .modules
        .iter_mut()
        .find(|module| module.path.starts_with("src/"))
        .unwrap()
        .declarations[0]
        .name = "not-valid".to_string();
    assert!(emit_python_td_rust_target(&unsupported, bad.path()).is_err());
    assert!(
        !bad.path().join("Cargo.toml").exists(),
        "unsupported IR must not partially apply"
    );
}

#[test]
fn cb_gen_rust_target_routes_to_native_emitter() {
    let output = tempfile::tempdir().unwrap();
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap()
        .to_path_buf();
    let result = Command::new(env!("CARGO_BIN_EXE_aw"))
        .args([
            "cb",
            "gen",
            "--target",
            "rust",
            "--source-root",
            fixture("python_spec_typer").to_str().unwrap(),
            "--output-dir",
            output.path().to_str().unwrap(),
        ])
        .current_dir(workspace_root)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.path().join("Cargo.toml").is_file());
    assert!(output.path().join("tests/generated_inventory.rs").is_file());
}

#[test]
fn cb_gen_rust_target_refuses_existing_project_without_partial_output() {
    let output = tempfile::tempdir().unwrap();
    seed_existing_project(output.path(), "Cargo.toml");
    let before = snapshot(output.path());
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap()
        .to_path_buf();
    let result = Command::new(env!("CARGO_BIN_EXE_aw"))
        .args([
            "cb",
            "gen",
            "--target",
            "rust",
            "--source-root",
            fixture("python_spec_typer").to_str().unwrap(),
            "--output-dir",
            output.path().to_str().unwrap(),
        ])
        .current_dir(workspace_root)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("non-empty existing project"),
        "stderr={}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(snapshot(output.path()), before);
}

#[test]
fn todo_python_td_generates_a_compiling_rust_target_through_the_cli() {
    let output = tempfile::tempdir().unwrap();
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap()
        .to_path_buf();
    let result = Command::new(env!("CARGO_BIN_EXE_aw"))
        .args([
            "cb",
            "gen",
            "--target",
            "rust",
            "--source-root",
            todo_td().to_str().unwrap(),
            "--output-dir",
            output.path().to_str().unwrap(),
        ])
        .current_dir(&workspace_root)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "Todo Python TD Rust generation failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.path().join("src/domain/todo.rs").is_file());
    assert!(output.path().join("src/interface/todo_ui.rs").is_file());

    let test = Command::new("cargo")
        .args(["test", "--quiet"])
        .current_dir(output.path())
        .output()
        .unwrap();
    assert!(
        test.status.success(),
        "generated Todo Rust target tests failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
}

#[test]
fn python_ir_typescript_target_is_tested_cold_and_fail_closed() {
    let ir = compile_python_td_project(&fixture("python_spec_typer")).unwrap();
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let first_target = emit_python_td_typescript_target(&ir, first.path()).unwrap();
    let second_target = emit_python_td_typescript_target(&ir, second.path()).unwrap();
    assert_eq!(first_target, second_target);
    assert_eq!(snapshot(first.path()), snapshot(second.path()));
    assert!(first.path().join("package.json").is_file());
    assert!(first.path().join("tsconfig.json").is_file());
    let test = Command::new("node")
        .args(["--test", "tests/generated_inventory.test.mjs"])
        .current_dir(first.path())
        .output()
        .unwrap();
    assert!(
        test.status.success(),
        "generated TypeScript tests failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );

    let bad = tempfile::tempdir().unwrap();
    let mut unsupported = ir.clone();
    unsupported
        .modules
        .iter_mut()
        .find(|module| module.path.starts_with("src/"))
        .unwrap()
        .declarations[0]
        .name = "not-valid".to_string();
    assert!(emit_python_td_typescript_target(&unsupported, bad.path()).is_err());
    assert!(
        !bad.path().join("package.json").exists(),
        "unsupported IR must not partially apply"
    );
}

#[test]
fn cb_gen_typescript_target_routes_to_native_emitter() {
    let output = tempfile::tempdir().unwrap();
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap()
        .to_path_buf();
    let result = Command::new(env!("CARGO_BIN_EXE_aw"))
        .args([
            "cb",
            "gen",
            "--target",
            "typescript",
            "--source-root",
            fixture("python_spec_typer").to_str().unwrap(),
            "--output-dir",
            output.path().to_str().unwrap(),
        ])
        .current_dir(workspace_root)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "aw cb gen --target typescript failed:\nstderr={}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.path().join("src/index.ts").is_file());
    assert!(output
        .path()
        .join("tests/generated_inventory.test.mjs")
        .is_file());
}
