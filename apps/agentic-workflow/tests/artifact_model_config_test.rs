// HANDWRITE-BEGIN gap="missing-generator:python-artifact-model-selector" tracker="#2306" reason="Configuration fixtures verify the explicit artifact-model migration boundary without generating project configuration."
//! Project artifact-model configuration acceptance tests.
//!
//! @spec apps/agentic-workflow/tech-design/core/logic/python-artifact-model-selector.md#unit-test

use agentic_workflow::models::project::ProjectArtifactModel;
use agentic_workflow::services::project_registry::{
    load_project_config_rows, load_projects, resolve_project_config_row,
};
use std::fs;
use tempfile::TempDir;

fn root_with_config(content: &str) -> TempDir {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("aw.toml"), content).unwrap();
    temp
}

const PROJECT_WITH_WORKSPACE: &str = r#"
[[projects]]
name = "demo"
path = "projects/demo"
spec_model = "{artifact_model}"

[[projects.workspaces]]
paths = ["projects/demo/**"]
target = "python"
test_cmd = "python -m pytest"
"#;

#[test]
fn spec_model_config_parses_canonical_python_and_legacy_compatibility_values() {
    for (value, expected) in [
        ("legacy", ProjectArtifactModel::Legacy),
        ("python", ProjectArtifactModel::PythonV1),
    ] {
        let root = root_with_config(&PROJECT_WITH_WORKSPACE.replace("{artifact_model}", value));
        let project = load_projects(root.path()).unwrap().remove(0);
        let row = resolve_project_config_row(root.path(), "demo").unwrap();

        assert_eq!(project.artifact_model, Some(expected));
        assert_eq!(project.effective_artifact_model(), expected);
        assert_eq!(row.artifact_model, Some(expected));
        assert_eq!(row.effective_artifact_model(), expected);
    }
}

#[test]
fn artifact_model_config_defaults_unconfigured_projects_to_legacy() {
    let root = root_with_config(
        &PROJECT_WITH_WORKSPACE.replace("spec_model = \"{artifact_model}\"\n", ""),
    );
    let project = load_projects(root.path()).unwrap().remove(0);
    let row = load_project_config_rows(root.path()).unwrap().remove(0);

    assert_eq!(project.artifact_model, None);
    assert_eq!(
        project.effective_artifact_model(),
        ProjectArtifactModel::Legacy
    );
    assert_eq!(row.artifact_model, None);
    assert_eq!(row.effective_artifact_model(), ProjectArtifactModel::Legacy);
}

#[test]
fn artifact_model_config_rejects_unknown_values_with_accepted_options() {
    let root = root_with_config(&PROJECT_WITH_WORKSPACE.replace("{artifact_model}", "python-v2"));
    let error = load_projects(root.path()).unwrap_err();
    let message = format!("{error:#}");

    assert!(message.contains("python-v2"), "unexpected error: {message}");
    assert!(message.contains("legacy"), "unexpected error: {message}");
    assert!(message.contains("python"), "unexpected error: {message}");
}

#[test]
fn artifact_model_config_preserves_root_opt_in_when_local_overlay_omits_it() {
    let root = root_with_config(&PROJECT_WITH_WORKSPACE.replace("{artifact_model}", "python"));
    fs::create_dir_all(root.path().join("projects/demo")).unwrap();
    fs::write(
        root.path().join("projects/demo/aw.toml"),
        r#"
[project]
name = "demo"
path = "projects/demo"
"#,
    )
    .unwrap();

    let project = load_projects(root.path()).unwrap().remove(0);
    let row = resolve_project_config_row(root.path(), "demo").unwrap();

    assert_eq!(
        project.effective_artifact_model(),
        ProjectArtifactModel::PythonV1
    );
    assert_eq!(
        row.effective_artifact_model(),
        ProjectArtifactModel::PythonV1
    );
}

#[test]
fn legacy_artifact_model_python_v1_is_read_compatible() {
    let config = PROJECT_WITH_WORKSPACE
        .replace("spec_model", "artifact_model")
        .replace("{artifact_model}", "python-v1");
    let root = root_with_config(&config);
    assert_eq!(
        load_projects(root.path())
            .unwrap()
            .remove(0)
            .effective_artifact_model(),
        ProjectArtifactModel::PythonV1
    );
}
// HANDWRITE-END
