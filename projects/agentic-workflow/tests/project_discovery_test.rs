// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/project_discovery_test.md#tests
// CODEGEN-BEGIN

//! External integration tests for project_discovery.
//!
//! Tests T19 (Rule E package name) and T20 (relative test_cmd path) from the
//! spec test plan.
//!
//! REQ: REQ-007, REQ-008

use std::fs;
use tempfile::TempDir;

use agentic_workflow::models::tech_stack::Language;
use agentic_workflow::services::project_discovery::discover_projects;

#[test]
fn rule_e_package_name() {
    // Directory name = "cli", but [package].name in Cargo.toml = "agentic-workflow"
    let tmp = TempDir::new().unwrap();
    let proj = tmp.path().join("projects").join("agentic-workflow");
    let cli = proj.join("cli");
    fs::create_dir_all(&cli).unwrap();

    // The key: directory is named "cli" but package name is "agentic-workflow"
    fs::write(
        cli.join("Cargo.toml"),
        "[package]\nname = \"agentic-workflow\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    let projects = discover_projects(tmp.path()).unwrap();

    // Should discover "agentic-workflow" project under projects/
    assert_eq!(
        projects.len(),
        1,
        "expected 1 project; got {:?}",
        projects.iter().map(|p| &p.name).collect::<Vec<_>>()
    );
    let ws = &projects[0].workspaces[0];

    // R8: workspace name must come from [package].name, not directory name "cli"
    assert_eq!(
        ws.name.as_deref(),
        Some("agentic-workflow"),
        "Rule E must derive workspace name from [package].name in nested Cargo.toml; \
         directory name was 'cli' but [package].name is 'agentic-workflow'"
    );

    // R8: test_cmd must use [package].name
    assert_eq!(
        ws.test_cmd.as_deref(),
        Some("cargo test -p agentic-workflow"),
        "Rule E test_cmd must use [package].name; got: {:?}",
        ws.test_cmd
    );

    assert_eq!(ws.target, Language::Rust);
}

#[test]
fn test_cmd_relative_path_python() {
    // Layout: <tmp>/projects/conductor/ with pyproject.toml + uv.lock at the project root.
    // This is the pattern where the project itself is a Python project (Rule C fires on the
    // project directory, not a subdirectory).
    let tmp = TempDir::new().unwrap();
    let conductor = tmp.path().join("projects").join("conductor");
    fs::create_dir_all(&conductor).unwrap();
    fs::write(
        conductor.join("pyproject.toml"),
        "[project]\nname = \"conductor\"\n",
    )
    .unwrap();
    fs::write(conductor.join("uv.lock"), "# uv lockfile\n").unwrap();

    let projects = discover_projects(tmp.path()).unwrap();
    assert_eq!(projects.len(), 1, "expected 1 project (conductor)");

    let ws = &projects[0].workspaces[0];
    assert_eq!(ws.target, Language::Python);

    let cmd = ws
        .test_cmd
        .as_deref()
        .expect("Rule C with uv.lock must produce test_cmd");

    // R7: must NOT contain absolute path
    let abs_str = tmp.path().to_string_lossy();
    assert!(
        !cmd.contains(abs_str.as_ref()),
        "test_cmd must not contain absolute filesystem path; got: {cmd}"
    );

    // R7: must use project-relative path (projects/conductor, not /tmp/xyz/projects/conductor)
    assert!(
        cmd.starts_with("cd projects/conductor"),
        "test_cmd must use project-relative path 'projects/conductor'; got: {cmd}"
    );
    assert!(
        cmd.contains("uv run pytest"),
        "test_cmd must contain 'uv run pytest'; got: {cmd}"
    );
}

#[test]
fn test_cmd_relative_path_typescript() {
    // Layout: <tmp>/projects/ui/ with package.json + vitest at the project root (Rule D).
    let tmp = TempDir::new().unwrap();
    let ui = tmp.path().join("projects").join("ui");
    fs::create_dir_all(&ui).unwrap();
    fs::write(
        ui.join("package.json"),
        r#"{"name":"ui","devDependencies":{"vitest":"^1.0.0","typescript":"^5.0.0"}}"#,
    )
    .unwrap();

    let projects = discover_projects(tmp.path()).unwrap();
    assert_eq!(projects.len(), 1, "expected 1 project (ui)");

    let ws = &projects[0].workspaces[0];

    let cmd = ws
        .test_cmd
        .as_deref()
        .expect("Rule D with vitest must produce test_cmd");

    // R7: must NOT contain absolute path
    let abs_str = tmp.path().to_string_lossy();
    assert!(
        !cmd.contains(abs_str.as_ref()),
        "test_cmd must not contain absolute filesystem path; got: {cmd}"
    );

    // R7: must use project-relative path (projects/ui)
    assert!(
        cmd.starts_with("cd projects/ui"),
        "test_cmd must use project-relative path 'projects/ui'; got: {cmd}"
    );
    assert!(
        cmd.contains("npx vitest run"),
        "test_cmd must contain 'npx vitest run'; got: {cmd}"
    );
}

#[test]
fn rule_a_be_test_cmd_is_relative() {
    // be/fe split: both test_cmds must use relative paths
    let tmp = TempDir::new().unwrap();
    let proj = tmp.path().join("projects").join("myapp");
    let be = proj.join("be");
    let fe = proj.join("fe");
    fs::create_dir_all(&be).unwrap();
    fs::create_dir_all(&fe).unwrap();
    fs::write(
        be.join("pyproject.toml"),
        "[project]\nname = \"myapp-be\"\n",
    )
    .unwrap();
    fs::write(be.join("uv.lock"), "# lockfile\n").unwrap();
    fs::write(
        fe.join("package.json"),
        r#"{"name":"myapp-fe","devDependencies":{"vitest":"^2.0.0"}}"#,
    )
    .unwrap();

    let projects = discover_projects(tmp.path()).unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].workspaces.len(), 2);

    let abs_str = tmp.path().to_string_lossy();
    for ws in &projects[0].workspaces {
        if let Some(ref cmd) = ws.test_cmd {
            assert!(
                !cmd.contains(abs_str.as_ref()),
                "workspace '{}' test_cmd must not contain absolute path; got: {cmd}",
                ws.name.as_deref().unwrap_or("?")
            );
        }
    }
}

#[test]
fn rule_e_fallback_to_dir_basename_when_no_package_name() {
    let tmp = TempDir::new().unwrap();
    let proj = tmp.path().join("crates").join("my-proj");
    let sub = proj.join("engine");
    fs::create_dir_all(&sub).unwrap();

    // Cargo.toml missing [package] entirely — fallback to dir name
    fs::write(
        sub.join("Cargo.toml"),
        "# malformed toml without package table\n",
    )
    .unwrap();

    let projects = discover_projects(tmp.path()).unwrap();
    assert_eq!(projects.len(), 1);
    let ws = &projects[0].workspaces[0];

    // Falls back to directory basename "engine"
    assert_eq!(
        ws.name.as_deref(),
        Some("engine"),
        "Rule E must fall back to directory basename when [package].name is absent"
    );
    assert_eq!(
        ws.test_cmd.as_deref(),
        Some("cargo test -p engine"),
        "Rule E fallback test_cmd must use directory basename"
    );
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests.md#schema
// CODEGEN-BEGIN
// SPEC-REF: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests.md#schema
// TODO: Existing source behavior is covered by this feature/domain semantic TD.

// CODEGEN-END
