// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/post_clarifications_service_preamble_source.md#source
// CODEGEN-BEGIN
//! Post-clarifications service — business logic for spec_clarifications.md.
//!
//! Extracted from `mcp/tools/clarifications.rs` (post_clarifications part).

use crate::Result;
use chrono::Local;
use std::path::Path;
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/post_clarifications_service.md#schema
// CODEGEN-BEGIN
/// A contradiction surfaced while cross-referencing artifacts.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/post_clarifications_service.md#schema
pub struct Contradiction {
    /// Path / identifier of the source artifact.
    pub source_artifact: String,
    /// Original claim from the source artifact.
    pub original_claim: String,
    /// Finding that contradicts the original claim.
    pub contradicting_finding: String,
    /// Resolution chosen to reconcile the contradiction.
    pub resolution: String,
}

/// Input for creating post-clarifications artifact.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/post_clarifications_service.md#schema
pub struct CreatePostClarificationsInput {
    /// Change identifier slug.
    pub change_id: String,
    /// Question-answer rows to include.
    pub questions: Vec<PostQuestion>,
    /// Contradiction rows to include.
    pub contradictions: Vec<Contradiction>,
}

/// Result of creating the post-clarifications artifact.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/post_clarifications_service.md#schema
pub struct PostClarificationsResult {
    /// Relative paths of artifacts written.
    pub artifacts_written: Vec<String>,
    /// Number of question rows written.
    pub questions_count: usize,
    /// Number of contradiction rows written.
    pub contradictions_count: usize,
}

/// A question-answer pair captured during post-gap clarification.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/post_clarifications_service.md#schema
pub struct PostQuestion {
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

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/post_clarifications_service_runtime_source.md#source
// CODEGEN-BEGIN
/// Create spec_clarifications.md (post-gap cross-reference artifact).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/post_clarifications_service_runtime_source.md#source
pub fn create_post_clarifications(
    input: CreatePostClarificationsInput,
    project_root: &Path,
) -> Result<PostClarificationsResult> {
    let change_dir = project_root.join(".aw/changes").join(&input.change_id);
    if !change_dir.exists() {
        std::fs::create_dir_all(&change_dir)?;
    }

    let file_path = change_dir.join("spec_clarifications.md");
    let today = Local::now().format("%Y-%m-%d").to_string();

    let mut content = format!(
        "---\nchange: {}\ndate: {}\n---\n\n# Spec Clarifications\n\n",
        input.change_id, today
    );

    // Questions section
    if !input.questions.is_empty() {
        content.push_str("## Questions\n\n");
        for (i, qa) in input.questions.iter().enumerate() {
            content.push_str(&format!("### Q{}: {}\n", i + 1, qa.topic));
            content.push_str(&format!("- **Question**: {}\n", qa.question));
            content.push_str(&format!("- **Answer**: {}\n", qa.answer));
            content.push_str(&format!("- **Rationale**: {}\n\n", qa.rationale));
        }
    }

    // Contradictions section
    if !input.contradictions.is_empty() {
        content.push_str("## Contradictions\n\n");
        for (i, c) in input.contradictions.iter().enumerate() {
            content.push_str(&format!(
                "### C{}: {} vs gathered context\n",
                i + 1,
                c.source_artifact
            ));
            content.push_str(&format!("- **Original claim**: {}\n", c.original_claim));
            content.push_str(&format!(
                "- **Contradicting finding**: {}\n",
                c.contradicting_finding
            ));
            content.push_str(&format!("- **Resolution**: {}\n\n", c.resolution));
        }
    }

    if input.questions.is_empty() && input.contradictions.is_empty() {
        content.push_str("All original clarifications remain valid. No contradictions found.\n");
    }

    std::fs::write(&file_path, content)?;

    let q_count = input.questions.len();
    let c_count = input.contradictions.len();

    Ok(PostClarificationsResult {
        artifacts_written: vec![format!(
            ".aw/changes/{}/spec_clarifications.md",
            input.change_id
        )],
        questions_count: q_count,
        contradictions_count: c_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_post_clarifications() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/test-post");
        std::fs::create_dir_all(&change_dir).unwrap();

        let input = CreatePostClarificationsInput {
            change_id: "test-post".to_string(),
            questions: vec![PostQuestion {
                topic: "Auth".to_string(),
                question: "Which method?".to_string(),
                answer: "JWT".to_string(),
                rationale: "Standard".to_string(),
            }],
            contradictions: vec![],
        };

        let result = create_post_clarifications(input, tmp.path()).unwrap();
        assert_eq!(result.questions_count, 1);
        assert_eq!(result.contradictions_count, 0);

        let content = std::fs::read_to_string(change_dir.join("spec_clarifications.md")).unwrap();
        assert!(content.contains("# Spec Clarifications"));
        assert!(content.contains("### Q1: Auth"));
    }

    #[test]
    fn test_create_post_clarifications_empty() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/test-empty");
        std::fs::create_dir_all(&change_dir).unwrap();

        let input = CreatePostClarificationsInput {
            change_id: "test-empty".to_string(),
            questions: vec![],
            contradictions: vec![],
        };

        let result = create_post_clarifications(input, tmp.path()).unwrap();
        assert_eq!(result.questions_count, 0);

        let content = std::fs::read_to_string(change_dir.join("spec_clarifications.md")).unwrap();
        assert!(content.contains("No contradictions found"));
    }
}
// CODEGEN-END
