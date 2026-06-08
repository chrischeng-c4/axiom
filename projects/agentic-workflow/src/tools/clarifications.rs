// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/clarifications/definition.md#source
// CODEGEN-BEGIN
use super::{get_required_array, get_required_string, ToolDefinition};
use crate::models::state::StatePhase;
use crate::services::pre_clarifications_service::{
    append_clarifications as service_append, AppendClarificationsInput, QuestionAnswer,
};
use crate::state::StateManager;
use crate::Result;
use chrono::Local;
use serde_json::{json, Value};
use std::path::Path;

/// @spec projects/agentic-workflow/tech-design/core/tools/clarifications/definition.md#source
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_create_clarifications".to_string(),
        description: "Create context_clarifications.md with structured Q&A from user".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "questions"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Change ID (lowercase, hyphens allowed)"
                },
                "questions": {
                    "type": "array",
                    "minItems": 1,
                    "description": "Array of Q&A pairs",
                    "items": {
                        "type": "object",
                        "required": ["topic", "question", "answer", "rationale"],
                        "properties": {
                            "topic": {
                                "type": "string",
                                "description": "Short topic label (e.g., 'Authentication Method')"
                            },
                            "question": {
                                "type": "string",
                                "description": "The question asked to the user"
                            },
                            "answer": {
                                "type": "string",
                                "description": "User's answer"
                            },
                            "rationale": {
                                "type": "string",
                                "description": "Why this choice was made"
                            }
                        }
                    }
                }
            }
        }),
    }
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/clarifications/execute.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/tools/clarifications/execute.md#source
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let questions = get_required_array(args, "questions")?;

    // Validate change_id format (security: prevent directory traversal)
    if !change_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!("Invalid change_id: must be lowercase alphanumeric with hyphens only");
    }

    // Create change directory if needed
    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    if !change_dir.exists() {
        std::fs::create_dir_all(&change_dir)?;
    }

    let clarifications_path = change_dir.join("context_clarifications.md");

    // Check for DAG in STATE.yaml (for multi-issue frontmatter)
    let dag = if change_dir.join("STATE.yaml").exists() {
        StateManager::load(&change_dir)
            .ok()
            .and_then(|sm| sm.state().dag.clone())
    } else {
        None
    };

    // Generate frontmatter (include issues list when DAG exists)
    let today = Local::now().format("%Y-%m-%d").to_string();
    let issues_line = if let Some(ref dag) = dag {
        let nums: Vec<String> = dag.issues.iter().map(|i| i.number.to_string()).collect();
        format!("\nissues: [{}]", nums.join(", "))
    } else {
        String::new()
    };
    let mut content = format!(
        "---\nchange: {}\ndate: {}{}\n---\n\n# Context Clarifications\n\n",
        change_id, today, issues_line
    );

    // Generate Q&A sections
    for (i, qa) in questions.iter().enumerate() {
        let topic = qa
            .get("topic")
            .and_then(|v| v.as_str())
            .unwrap_or("General");
        let question = qa.get("question").and_then(|v| v.as_str()).unwrap_or("");
        let answer = qa.get("answer").and_then(|v| v.as_str()).unwrap_or("");
        let rationale = qa.get("rationale").and_then(|v| v.as_str()).unwrap_or("");

        content.push_str(&format!("## Q{}: {}\n", i + 1, topic));
        content.push_str(&format!("- **Question**: {}\n", question));
        content.push_str(&format!("- **Answer**: {}\n", answer));
        content.push_str(&format!("- **Rationale**: {}\n", rationale));
        content.push('\n');
    }

    // Inject dependency graph section when DAG exists
    if let Some(ref dag) = dag {
        if !dag.issues.is_empty() {
            content.push_str("## Dependency Graph\n\n");
            content.push_str("| Order | Issue | Depends On |\n");
            content.push_str("|-------|-------|------------|\n");
            for (i, issue) in dag.issues.iter().enumerate() {
                let deps = if issue.blocked_by.is_empty() {
                    "\u{2014}".to_string()
                } else {
                    issue
                        .blocked_by
                        .iter()
                        .map(|d| format!("#{}", d))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                content.push_str(&format!(
                    "| {} | #{} \u{2014} {} | {} |\n",
                    i + 1,
                    issue.number,
                    issue.title,
                    deps
                ));
            }
            content.push('\n');

            // Mermaid diagram
            content.push_str("```mermaid\ngraph LR\n");
            for issue in &dag.issues {
                content.push_str(&format!(
                    "    {}[\"#{} {}\"]\n",
                    issue.number, issue.number, issue.title
                ));
            }
            for issue in &dag.issues {
                for dep in &issue.blocked_by {
                    content.push_str(&format!("    {} --> {}\n", dep, issue.number));
                }
            }
            content.push_str("```\n\n");
        }
    }

    std::fs::write(&clarifications_path, content)?;

    // Auto-update STATE.yaml phase
    super::workflow_common::update_phase(&change_dir, StatePhase::ChangeInited)?;

    let result = json!({
        "status": "ok",
        "change_id": change_id,
        "artifacts": [
            format!(".aw/changes/{}/context_clarifications.md", change_id),
            format!(".aw/changes/{}/STATE.yaml", change_id),
        ],
        "phase": "clarified",
        "questions_count": questions.len(),
        "next": "sdd_run_change"
    });
    Ok(result.to_string())
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/clarifications/post-clarifications.md#source
// CODEGEN-BEGIN
/// Write spec_clarifications.md (post-gap cross-reference artifact).
///
/// Called by `artifact_write` for `artifact="spec_clarifications"`.
/// Writes to `spec_clarifications.md` and sets the appropriate post_clarifications phase.
/// @spec projects/agentic-workflow/tech-design/core/tools/clarifications/post-clarifications.md#source
pub fn execute_post_clarifications(
    args: &Value,
    project_root: &Path,
    _action: &str,
) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let questions = args
        .get("questions")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let contradictions = args
        .get("contradictions")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if !change_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!("Invalid change_id: must be lowercase alphanumeric with hyphens only");
    }

    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    if !change_dir.exists() {
        std::fs::create_dir_all(&change_dir)?;
    }

    let file_path = change_dir.join("spec_clarifications.md");
    let today = Local::now().format("%Y-%m-%d").to_string();

    let mut content = format!(
        "---\nchange: {}\ndate: {}\n---\n\n# Spec Clarifications\n\n",
        change_id, today
    );

    // Questions section
    if !questions.is_empty() {
        content.push_str("## Questions\n\n");
        for (i, qa) in questions.iter().enumerate() {
            let topic = qa
                .get("topic")
                .and_then(|v| v.as_str())
                .unwrap_or("General");
            let question = qa.get("question").and_then(|v| v.as_str()).unwrap_or("");
            let answer = qa.get("answer").and_then(|v| v.as_str()).unwrap_or("");
            let rationale = qa.get("rationale").and_then(|v| v.as_str()).unwrap_or("");

            content.push_str(&format!("### Q{}: {}\n", i + 1, topic));
            content.push_str(&format!("- **Question**: {}\n", question));
            content.push_str(&format!("- **Answer**: {}\n", answer));
            content.push_str(&format!("- **Rationale**: {}\n\n", rationale));
        }
    }

    // Contradictions section
    if !contradictions.is_empty() {
        content.push_str("## Contradictions\n\n");
        for (i, c) in contradictions.iter().enumerate() {
            let source = c
                .get("source_artifact")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let original = c
                .get("original_claim")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let contradicting = c
                .get("contradicting_finding")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let resolution = c.get("resolution").and_then(|v| v.as_str()).unwrap_or("");

            content.push_str(&format!("### C{}: {} vs gathered context\n", i + 1, source));
            content.push_str(&format!("- **Original claim**: {}\n", original));
            content.push_str(&format!("- **Contradicting finding**: {}\n", contradicting));
            content.push_str(&format!("- **Resolution**: {}\n\n", resolution));
        }
    }

    if questions.is_empty() && contradictions.is_empty() {
        content.push_str("All original clarifications remain valid. No contradictions found.\n");
    }

    std::fs::write(&file_path, content)?;

    // Legacy PostClarifications phase removed; spec_clarifications artifact
    // still written but no phase update needed (handled by run_change flow)

    let phase_str = "spec_clarifications_written";

    let result = json!({
        "status": "ok",
        "change_id": change_id,
        "artifacts": [
            format!(".aw/changes/{}/spec_clarifications.md", change_id),
            format!(".aw/changes/{}/STATE.yaml", change_id),
        ],
        "phase": phase_str,
        "questions_count": questions.len(),
        "contradictions_count": contradictions.len(),
        "next": "sdd_run_change"
    });
    Ok(result.to_string())
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/clarifications/append.md#source
// CODEGEN-BEGIN
/// Append per-issue clarifications to context_clarifications.md (DAG mode).
///
/// Called internally by `artifact_write` when `issue` param is present.
/// @spec projects/agentic-workflow/tech-design/core/tools/clarifications/append.md#source
pub fn execute_append(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let questions = get_required_array(args, "questions")?;
    let issue_num = args.get("issue").and_then(|v| v.as_u64());

    // Convert JSON array to QuestionAnswer structs
    let qa_list: Vec<QuestionAnswer> = questions
        .iter()
        .map(|qa| QuestionAnswer {
            topic: qa
                .get("topic")
                .and_then(|v| v.as_str())
                .unwrap_or("General")
                .to_string(),
            question: qa
                .get("question")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            answer: qa
                .get("answer")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            rationale: qa
                .get("rationale")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
        .collect();

    let qa_count = qa_list.len();

    let input = AppendClarificationsInput {
        change_id: change_id.clone(),
        issue: issue_num,
        questions: qa_list,
    };

    let _service_result = service_append(input, project_root)?;

    let result = json!({
        "status": "ok",
        "change_id": change_id,
        "artifacts": [
            format!(".aw/changes/{}/context_clarifications.md", change_id),
            format!(".aw/changes/{}/STATE.yaml", change_id),
        ],
        "questions_count": qa_count,
        "next": "sdd_run_change"
    });
    Ok(result.to_string())
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/clarifications/tests.md#source
// CODEGEN-BEGIN
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn test_create_clarifications_valid() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        crate::test_util::write_minimal_issue(project_root, "add-oauth");

        let args = json!({
            "change_id": "add-oauth",
            "questions": [
                {
                    "topic": "Auth Method",
                    "question": "Which OAuth providers?",
                    "answer": "Google and GitHub",
                    "rationale": "Most common enterprise providers"
                }
            ]
        });

        let result = execute(&args, project_root).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["change_id"], "add-oauth");
        assert_eq!(parsed["phase"], "clarified");
        assert_eq!(parsed["questions_count"], 1);
        assert_eq!(parsed["next"], "sdd_run_change");
        assert!(parsed["artifacts"].as_array().unwrap().len() >= 1);

        let file_path = project_root.join(".aw/changes/add-oauth/context_clarifications.md");
        assert!(file_path.exists());

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("---"));
        assert!(content.contains("change: add-oauth"));
        assert!(content.contains("## Q1: Auth Method"));
        assert!(content.contains("**Question**: Which OAuth providers?"));
    }

    #[test]
    fn test_create_clarifications_invalid_change_id() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let args = json!({
            "change_id": "../etc/passwd",
            "questions": [{"topic": "test", "question": "test", "answer": "test", "rationale": "test"}]
        });

        let result = execute(&args, project_root);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_clarifications_multiple_questions() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        crate::test_util::write_minimal_issue(project_root, "multi-test");

        let args = json!({
            "change_id": "multi-test",
            "questions": [
                {
                    "topic": "First Topic",
                    "question": "First question?",
                    "answer": "First answer",
                    "rationale": "First rationale"
                },
                {
                    "topic": "Second Topic",
                    "question": "Second question?",
                    "answer": "Second answer",
                    "rationale": "Second rationale"
                }
            ]
        });

        let result = execute(&args, project_root).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["questions_count"], 2);

        let file_path = project_root.join(".aw/changes/multi-test/context_clarifications.md");
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("## Q1: First Topic"));
        assert!(content.contains("## Q2: Second Topic"));
    }

    #[test]
    fn test_append_clarifications_mcp_per_issue() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        crate::test_util::write_minimal_issue(project_root, "append-mcp-test");

        // First create initial clarifications
        let create_args = json!({
            "change_id": "append-mcp-test",
            "questions": [
                {
                    "topic": "Initial Topic",
                    "question": "Initial question?",
                    "answer": "Initial answer",
                    "rationale": "Initial rationale"
                }
            ]
        });
        execute(&create_args, project_root).unwrap();

        // Now append per-issue clarifications (DAG mode)
        let append_args = json!({
            "change_id": "append-mcp-test",
            "issue": 188,
            "questions": [
                {
                    "topic": "Auth Method",
                    "question": "Which auth?",
                    "answer": "JWT",
                    "rationale": "Standard"
                }
            ]
        });
        let result = execute_append(&append_args, project_root).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["change_id"], "append-mcp-test");
        assert_eq!(parsed["next"], "sdd_run_change");

        // Verify appended content in pre_clarifications.md (service_append writes here)
        let file_path = project_root.join(".aw/changes/append-mcp-test/pre_clarifications.md");
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("## Issue #188"));
        assert!(content.contains("### Q1: Auth Method"));
    }
}
// CODEGEN-END
