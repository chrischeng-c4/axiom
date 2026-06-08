// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/pre_clarifications_service_preamble_source.md#source
// CODEGEN-BEGIN
//! Clarifications service - Business logic for creating pre_clarifications.md
//!
//! Provides structured Q&A capture from user interactions during planning.

use crate::models::state::StatePhase;
use crate::state::StateManager;
use crate::Result;
use chrono::Local;
use std::path::Path;
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/pre_clarifications_service.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Input for appending per-issue pre-clarifications to an existing artifact.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/pre_clarifications_service.md#schema
#[derive(Debug, Serialize, Deserialize)]
pub struct AppendClarificationsInput {
    /// Change identifier slug.
    pub change_id: String,
    /// Optional issue number to anchor the appended Q&A section.
    pub issue: Option<u64>,
    /// Question-answer rows to append.
    pub questions: Vec<QuestionAnswer>,
}

/// Input for creating pre-clarifications artifact.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/pre_clarifications_service.md#schema
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateClarificationsInput {
    /// Change identifier slug.
    pub change_id: String,
    /// Question-answer rows to include in the new artifact.
    pub questions: Vec<QuestionAnswer>,
}

/// A question-answer pair captured during pre-gap clarification.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/pre_clarifications_service.md#schema
#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionAnswer {
    /// Topic / area the question addresses.
    pub topic: String,
    /// Question text.
    pub question: String,
    /// Answer text.
    pub answer: String,
    /// Reasoning behind the answer.
    pub rationale: String,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/pre_clarifications_service_runtime_source.md#source
// CODEGEN-BEGIN
/// Create pre_clarifications.md file with structured Q&A
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/pre_clarifications_service_runtime_source.md#source
pub fn create_clarifications(
    input: CreateClarificationsInput,
    project_root: &Path,
) -> Result<String> {
    let change_id = &input.change_id;

    // Validate change_id format (security: prevent directory traversal)
    if !change_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!("Invalid change_id: must be lowercase alphanumeric with hyphens only");
    }

    // Create change directory if needed
    let change_dir = project_root.join(".aw/changes").join(change_id);
    if !change_dir.exists() {
        std::fs::create_dir_all(&change_dir)?;
    }

    let clarifications_path = change_dir.join("pre_clarifications.md");

    // Generate frontmatter
    let today = Local::now().format("%Y-%m-%d").to_string();
    let mut content = format!(
        "---\nchange: {}\ndate: {}\n---\n\n# Context Clarifications\n\n",
        change_id, today
    );

    // Generate Q&A sections
    for (i, qa) in input.questions.iter().enumerate() {
        content.push_str(&format!("## Q{}: {}\n", i + 1, qa.topic));
        content.push_str(&format!("- **Question**: {}\n", qa.question));
        content.push_str(&format!("- **Answer**: {}\n", qa.answer));
        content.push_str(&format!("- **Rationale**: {}\n", qa.rationale));
        content.push('\n');
    }

    std::fs::write(&clarifications_path, content)?;

    // Initialize STATE.yaml if it doesn't exist
    let state_path = change_dir.join("STATE.yaml");
    if !state_path.exists() {
        let mut state_manager = StateManager::load(&change_dir)?;
        state_manager.set_phase(StatePhase::ChangeInited)?;
        state_manager.set_last_action("clarifications created");
        state_manager.save()?;
    }

    Ok(format!(
        "✓ Clarifications written: .aw/changes/{}/pre_clarifications.md\n  Questions: {}",
        change_id,
        input.questions.len()
    ))
}

/// Append per-issue clarifications to existing pre_clarifications.md (DAG mode).
///
/// - If pre_clarifications.md doesn't exist, creates a new file
/// - If it exists, appends new Q&A section under an issue header
/// - Updates STATE.yaml phase to 'clarified'
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/pre_clarifications_service_runtime_source.md#source
pub fn append_clarifications(
    input: AppendClarificationsInput,
    project_root: &Path,
) -> Result<String> {
    let change_id = &input.change_id;

    // Validate change_id format (security: prevent directory traversal)
    if !change_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!("Invalid change_id: must be lowercase alphanumeric with hyphens only");
    }

    let change_dir = project_root.join(".aw/changes").join(change_id);
    if !change_dir.exists() {
        anyhow::bail!("Change directory not found: {}", change_id);
    }

    let clarifications_path = change_dir.join("pre_clarifications.md");

    // Build section header based on issue number
    let section_header = match input.issue {
        Some(num) => format!("## Issue #{}", num),
        None => "## Additional Clarifications".to_string(),
    };

    // Build the new content to append
    let mut new_content = String::new();
    new_content.push_str("\n\n");
    new_content.push_str(&format!("{}\n\n", section_header));

    // Generate Q&A sections
    for (i, qa) in input.questions.iter().enumerate() {
        new_content.push_str(&format!("### Q{}: {}\n", i + 1, qa.topic));
        new_content.push_str(&format!("- **Question**: {}\n", qa.question));
        new_content.push_str(&format!("- **Answer**: {}\n", qa.answer));
        new_content.push_str(&format!("- **Rationale**: {}\n", qa.rationale));
        new_content.push('\n');
    }

    if clarifications_path.exists() {
        // Append to existing file
        let existing_content = std::fs::read_to_string(&clarifications_path)?;
        let updated_content = format!("{}{}", existing_content.trim_end(), new_content);
        std::fs::write(&clarifications_path, updated_content)?;
    } else {
        // Create new file with frontmatter
        let today = Local::now().format("%Y-%m-%d").to_string();
        let content = format!(
            "---\nchange: {}\ndate: {}\n---\n\n# Context Clarifications\n{}",
            change_id, today, new_content
        );
        std::fs::write(&clarifications_path, content)?;
    }

    // Update STATE.yaml phase to 'clarified'
    let mut state_manager = StateManager::load(&change_dir)?;
    state_manager.set_phase(StatePhase::ChangeInited)?;
    state_manager.set_last_action(&format!(
        "per-issue clarifications appended{}",
        input
            .issue
            .map(|n| format!(" (issue #{})", n))
            .unwrap_or_default()
    ));
    state_manager.save()?;

    Ok(format!(
        "✓ Clarifications appended: .aw/changes/{}/pre_clarifications.md\n  Issue: {:?}\n  Questions: {}\n  State: clarified",
        change_id,
        input.issue,
        input.questions.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::state::StatePhase;
    use crate::state::StateManager;
    use tempfile::TempDir;

    #[test]
    fn test_create_clarifications_valid() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        crate::test_util::write_minimal_issue(project_root, "add-oauth");

        let input = CreateClarificationsInput {
            change_id: "add-oauth".to_string(),
            questions: vec![QuestionAnswer {
                topic: "Auth Method".to_string(),
                question: "Which OAuth providers?".to_string(),
                answer: "Google and GitHub".to_string(),
                rationale: "Most common enterprise providers".to_string(),
            }],
        };

        let result = create_clarifications(input, project_root).unwrap();
        assert!(result.contains("✓ Clarifications written"));

        let file_path = project_root.join(".aw/changes/add-oauth/pre_clarifications.md");
        assert!(file_path.exists());

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("---"));
        assert!(content.contains("change: add-oauth"));
        assert!(content.contains("## Q1: Auth Method"));
        assert!(content.contains("**Question**: Which OAuth providers?"));

        // Verify state was initialized (STATE.yaml is deprecated; check via StateManager)
        let change_dir = project_root.join(".aw/changes/add-oauth");
        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(*sm.phase(), StatePhase::ChangeInited);
    }

    #[test]
    fn test_create_clarifications_invalid_change_id() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let input = CreateClarificationsInput {
            change_id: "../etc/passwd".to_string(),
            questions: vec![QuestionAnswer {
                topic: "test".to_string(),
                question: "test".to_string(),
                answer: "test".to_string(),
                rationale: "test".to_string(),
            }],
        };

        let result = create_clarifications(input, project_root);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_clarifications_multiple_questions() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        crate::test_util::write_minimal_issue(project_root, "multi-test");

        let input = CreateClarificationsInput {
            change_id: "multi-test".to_string(),
            questions: vec![
                QuestionAnswer {
                    topic: "First Topic".to_string(),
                    question: "First question?".to_string(),
                    answer: "First answer".to_string(),
                    rationale: "First rationale".to_string(),
                },
                QuestionAnswer {
                    topic: "Second Topic".to_string(),
                    question: "Second question?".to_string(),
                    answer: "Second answer".to_string(),
                    rationale: "Second rationale".to_string(),
                },
            ],
        };

        let result = create_clarifications(input, project_root).unwrap();
        assert!(result.contains("Questions: 2"));

        let file_path = project_root.join(".aw/changes/multi-test/pre_clarifications.md");
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("## Q1: First Topic"));
        assert!(content.contains("## Q2: Second Topic"));
    }

    #[test]
    fn test_append_clarifications_per_issue() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        crate::test_util::write_minimal_issue(project_root, "append-test");

        // First, create initial clarifications
        let initial_input = CreateClarificationsInput {
            change_id: "append-test".to_string(),
            questions: vec![QuestionAnswer {
                topic: "Initial Topic".to_string(),
                question: "Initial question?".to_string(),
                answer: "Initial answer".to_string(),
                rationale: "Initial rationale".to_string(),
            }],
        };
        create_clarifications(initial_input, project_root).unwrap();

        // Append per-issue clarifications (DAG mode)
        let append_input = AppendClarificationsInput {
            change_id: "append-test".to_string(),
            issue: Some(188),
            questions: vec![QuestionAnswer {
                topic: "Auth Method".to_string(),
                question: "Which auth?".to_string(),
                answer: "JWT".to_string(),
                rationale: "Standard".to_string(),
            }],
        };
        let result = append_clarifications(append_input, project_root).unwrap();
        assert!(result.contains("✓ Clarifications appended"));
        assert!(result.contains("State: clarified"));

        // Verify file contents
        let file_path = project_root.join(".aw/changes/append-test/pre_clarifications.md");
        let content = std::fs::read_to_string(&file_path).unwrap();

        // Should have both initial and per-issue content
        assert!(content.contains("## Q1: Initial Topic"));
        assert!(content.contains("## Issue #188"));
        assert!(content.contains("### Q1: Auth Method"));
    }

    #[test]
    fn test_append_clarifications_creates_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory without pre_clarifications.md
        let change_dir = project_root.join(".aw/changes/new-append");
        std::fs::create_dir_all(&change_dir).unwrap();

        // Create backing issue so StateManager can load and save
        crate::test_util::write_minimal_issue(project_root, "new-append");

        let input = AppendClarificationsInput {
            change_id: "new-append".to_string(),
            issue: Some(42),
            questions: vec![QuestionAnswer {
                topic: "New Topic".to_string(),
                question: "New question?".to_string(),
                answer: "New answer".to_string(),
                rationale: "New rationale".to_string(),
            }],
        };

        let result = append_clarifications(input, project_root).unwrap();
        assert!(result.contains("✓ Clarifications appended"));

        // Verify file was created with frontmatter
        let file_path = project_root.join(".aw/changes/new-append/pre_clarifications.md");
        assert!(file_path.exists());
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("---"));
        assert!(content.contains("change: new-append"));
        assert!(content.contains("## Issue #42"));
    }

    #[test]
    fn test_append_clarifications_without_issue() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        crate::test_util::write_minimal_issue(project_root, "no-issue-test");

        // Create initial clarifications
        let initial_input = CreateClarificationsInput {
            change_id: "no-issue-test".to_string(),
            questions: vec![QuestionAnswer {
                topic: "Init".to_string(),
                question: "Q?".to_string(),
                answer: "A".to_string(),
                rationale: "R".to_string(),
            }],
        };
        create_clarifications(initial_input, project_root).unwrap();

        // Append without issue number
        let append_input = AppendClarificationsInput {
            change_id: "no-issue-test".to_string(),
            issue: None,
            questions: vec![QuestionAnswer {
                topic: "Extra".to_string(),
                question: "Q2?".to_string(),
                answer: "A2".to_string(),
                rationale: "R2".to_string(),
            }],
        };
        append_clarifications(append_input, project_root).unwrap();

        let file_path = project_root.join(".aw/changes/no-issue-test/pre_clarifications.md");
        let content = std::fs::read_to_string(&file_path).unwrap();

        assert!(content.contains("## Additional Clarifications"));
        assert!(content.contains("### Q1: Extra"));
    }
}
// CODEGEN-END
