// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/init_change_service_preamble_source.md#source
// CODEGEN-BEGIN
//! Init change service — business logic for creating new changes.
//!
//! Extracted from `mcp/tools/init_change.rs`.

use crate::state::StateManager;
use crate::Result;
use std::path::Path;
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/init_change_service.md#schema
// CODEGEN-BEGIN
/// Input for creating a new change.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/init_change_service.md#schema
pub struct CreateChangeInput {
    /// Change identifier slug.
    pub change_id: String,
    /// Raw user-supplied description.
    pub description: String,
    /// Optional list of issue references to attach.
    pub issue_refs: Option<Vec<String>>,
    /// Optional git-workflow strategy name.
    pub git_workflow: Option<String>,
}

/// Result of creating a new change.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/init_change_service.md#schema
pub struct CreateChangeResult {
    /// Echoed change identifier.
    pub change_id: String,
    /// Relative paths of artifacts written by create_change.
    pub artifacts_written: Vec<String>,
    /// True if issue_refs was non-empty and issues were fetched.
    pub has_issues: bool,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/init_change_service_runtime_source.md#source
// CODEGEN-BEGIN
/// Create a new change directory with STATE.yaml.
///
/// Steps:
/// 1. Create change directory
/// 2. Optionally fetch issues (builds DAG in STATE.yaml)
/// 3. Write user_input.md with raw description
/// 4. Initialize STATE.yaml with git_workflow field
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/init_change_service_runtime_source.md#source
pub fn create_change(input: CreateChangeInput, project_root: &Path) -> Result<CreateChangeResult> {
    let change_dir = project_root.join(".aw/changes").join(&input.change_id);

    // 1. Create change directory
    std::fs::create_dir_all(&change_dir)?;

    // 2. If issue_refs present, fetch issues
    let has_issues = if let Some(ref refs) = input.issue_refs {
        if !refs.is_empty() {
            let fetch_args = serde_json::json!({
                "change_id": input.change_id,
                "issue_refs": refs,
            });
            crate::tools::fetch_issues::execute(&fetch_args, project_root)?;
            true
        } else {
            false
        }
    } else {
        false
    };

    // 3. Write user_input.md
    std::fs::write(change_dir.join("user_input.md"), &input.description)?;

    // 4. Load STATE.yaml and set git_workflow
    let mut sm = StateManager::load(&change_dir)?;
    if let Some(ref wf) = input.git_workflow {
        sm.state_mut().git_workflow = Some(wf.clone());
    }
    sm.save()?;

    Ok(CreateChangeResult {
        change_id: input.change_id,
        artifacts_written: vec!["user_input.md".to_string(), "STATE.yaml".to_string()],
        has_issues,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_project() -> TempDir {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".aw/changes")).unwrap();
        tmp
    }

    #[test]
    fn test_create_change_basic() {
        let tmp = setup_project();
        crate::test_util::write_minimal_issue(tmp.path(), "test-change");
        let input = CreateChangeInput {
            change_id: "test-change".to_string(),
            description: "Add new feature".to_string(),
            issue_refs: None,
            git_workflow: Some("new_branch".to_string()),
        };
        let result = create_change(input, tmp.path()).unwrap();
        assert_eq!(result.change_id, "test-change");
        assert!(!result.has_issues);

        let change_dir = tmp.path().join(".aw/changes/test-change");
        // STATE.yaml is deprecated — state is now stored in issue frontmatter
        assert!(change_dir.join("user_input.md").exists());
        assert_eq!(
            std::fs::read_to_string(change_dir.join("user_input.md")).unwrap(),
            "Add new feature"
        );

        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(sm.state().git_workflow.as_deref(), Some("new_branch"));
    }

    #[test]
    fn test_create_change_without_git_workflow() {
        let tmp = setup_project();
        crate::test_util::write_minimal_issue(tmp.path(), "simple-change");
        let input = CreateChangeInput {
            change_id: "simple-change".to_string(),
            description: "Simple change".to_string(),
            issue_refs: None,
            git_workflow: None,
        };
        let result = create_change(input, tmp.path()).unwrap();
        assert_eq!(result.change_id, "simple-change");

        let change_dir = tmp.path().join(".aw/changes/simple-change");
        let sm = StateManager::load(&change_dir).unwrap();
        assert!(sm.state().git_workflow.is_none());
    }
}
// CODEGEN-END
