// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_pre_clarifications/definition.md#source
// CODEGEN-BEGIN
use super::{get_required_array, get_required_string, ToolDefinition};
use crate::models::state::StatePhase;
use crate::models::WorkflowArtifact;
use crate::services::pre_clarifications_service::{
    append_clarifications as service_append, AppendClarificationsInput, QuestionAnswer,
};
use crate::state::StateManager;
use crate::tools::workflow_common;
use crate::Result;
use chrono::Local;
use serde_json::{json, Value};
use std::path::Path;

/// @spec projects/agentic-workflow/tech-design/core/tools/create_pre_clarifications/definition.md#source
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_create_clarifications".to_string(),
        description: "Create pre_clarifications.md with structured Q&A from user".to_string(),
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

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_pre_clarifications/execute.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/tools/create_pre_clarifications/execute.md#source
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
    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    if !change_dir.exists() {
        std::fs::create_dir_all(&change_dir)?;
    }

    let clarifications_path = change_dir.join("pre_clarifications.md");

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
            format!(".aw/changes/{}/pre_clarifications.md", change_id),
            format!(".aw/changes/{}/STATE.yaml", change_id),
        ],
        "phase": "change_inited",
        "questions_count": questions.len(),
        "next": "sdd_run_change"
    });
    Ok(result.to_string())
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_pre_clarifications/append.md#source
// CODEGEN-BEGIN
/// Append per-issue clarifications to pre_clarifications.md (DAG mode).
///
/// Called internally by `artifact_write` when `issue` param is present.
/// @spec projects/agentic-workflow/tech-design/core/tools/create_pre_clarifications/append.md#source
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
            format!(".aw/changes/{}/pre_clarifications.md", change_id),
            format!(".aw/changes/{}/STATE.yaml", change_id),
        ],
        "questions_count": qa_count,
        "next": "sdd_run_change"
    });
    Ok(result.to_string())
}
// CODEGEN-END
// =============================================================================
// New group-aware workflow + artifact tools
// =============================================================================

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_pre_clarifications/workflow-definitions.md#source
// CODEGEN-BEGIN
/// MCP tool definition for sdd_workflow_create_pre_clarifications
/// @spec projects/agentic-workflow/tech-design/core/tools/create_pre_clarifications/workflow-definitions.md#source
pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_create_pre_clarifications".to_string(),
        description: "Return prompt for mainthread to clarify the next incomplete group"
            .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path"
                },
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Change ID"
                }
            }
        }),
    }
}

/// MCP tool definition for sdd_artifact_create_pre_clarifications
/// @spec projects/agentic-workflow/tech-design/core/tools/create_pre_clarifications/workflow-definitions.md#source
pub fn artifact_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_artifact_create_pre_clarifications".to_string(),
        description: "Write answers for pre-generated questions in a group".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "group_id", "answers"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path"
                },
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Change ID"
                },
                "group_id": {
                    "type": "string",
                    "description": "Group ID to write answers for"
                },
                "answers": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": ["topic", "answer"],
                        "properties": {
                            "topic": {
                                "type": "string",
                                "description": "Topic matching the pre-generated question"
                            },
                            "answer": {
                                "type": "string",
                                "description": "User's answer to the question"
                            },
                            "follow_up_questions": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Optional follow-up questions raised by the answer"
                            }
                        }
                    }
                }
            }
        }),
    }
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_pre_clarifications/workflow.md#source
// CODEGEN-BEGIN
/// Execute sdd_workflow_create_pre_clarifications.
///
/// Determines the next group to clarify, returns prompt to mainthread.
/// @spec projects/agentic-workflow/tech-design/core/tools/create_pre_clarifications/workflow.md#source
pub async fn execute_workflow_pre_clarifications(
    args: &Value,
    project_root: &Path,
) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    // If already past this phase, advance and return
    let sm = StateManager::load(&change_dir)?;
    if *sm.phase() != StatePhase::ChangeInited {
        let result = json!({
            "status": "phase_complete",
            "prompt": "Pre-clarifications already created. Advancing.",
            "next_actions": [
                workflow_common::next_action(interface, "sdd_run_change", json!({"change_id": change_id}))
            ]
        });
        return Ok(serde_json::to_string_pretty(&result)?);
    }
    drop(sm);

    let project_path = project_root.display();

    let prompt = format!(
        r#"# Task: Create Pre-Clarifications for Change '{change_id}'

## Files to Read

- `{project_path}/.aw/changes/{change_id}/user_input.md` — user's description

## Instructions

1. Read user_input.md
2. Identify key decisions and open questions
3. Use AskUserQuestion to ask the user for clarifications
4. When sufficient, run `score artifact create-pre-clarifications` with answers

## CLI Commands

```
# Write artifact (write payload JSON first, then run)
score artifact create-pre-clarifications {change_id} .aw/changes/{change_id}/payloads/create-pre-clarifications.json
```"#,
    );

    let executor = workflow_common::get_executor_chain(
        project_root,
        WorkflowArtifact::CreatePreClarifications,
    );

    workflow_common::build_workflow_response(
        &change_dir,
        &change_id,
        "create_pre_clarifications",
        prompt,
        executor,
        json!({}),
        interface,
        project_root,
    )
    .await
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_pre_clarifications/artifact.md#source
// CODEGEN-BEGIN
/// Execute sdd_artifact_create_pre_clarifications.
///
/// Reads existing `groups/{group_id}/pre_clarifications.md` (with question stubs),
/// merges answers, sets status to answered, updates `groups_progress`.
/// @spec projects/agentic-workflow/tech-design/core/tools/create_pre_clarifications/artifact.md#source
pub fn execute_artifact_pre_clarifications(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;

    // Accept both "answers" (new) and "questions" (legacy) field names
    let answers = args
        .get("answers")
        .or_else(|| args.get("questions"))
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Missing required array field: answers"))?;

    workflow_common::validate_change_id(&change_id)?;
    let interface = workflow_common::load_interface(project_root);

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);

    if !change_dir.exists() {
        anyhow::bail!("Change directory not found: {}", change_dir.display());
    }

    let clarif_path = change_dir.join("pre_clarifications.md");
    let today = Local::now().format("%Y-%m-%d").to_string();

    let mut content = format!(
        "---\nchange: {}\ndate: {}\nstatus: answered\n---\n\n# Pre-Clarifications\n\n",
        change_id, today
    );

    for (i, qa) in answers.iter().enumerate() {
        let topic = qa
            .get("topic")
            .and_then(|v| v.as_str())
            .unwrap_or("General");
        let question = qa.get("question").and_then(|v| v.as_str()).unwrap_or("");
        let answer = qa.get("answer").and_then(|v| v.as_str()).unwrap_or("");
        let rationale = qa.get("rationale").and_then(|v| v.as_str()).unwrap_or("");
        let follow_ups: Vec<String> = qa
            .get("follow_up_questions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        content.push_str(&format!("### Q{}: {}\n", i + 1, topic));
        if !question.is_empty() {
            content.push_str(&format!("- **Question**: {}\n", question));
        }
        content.push_str(&format!("- **Answer**: {}\n", answer));
        if !rationale.is_empty() {
            content.push_str(&format!("- **Rationale**: {}\n", rationale));
        }
        for fq in &follow_ups {
            content.push_str(&format!("- **Follow-up**: {}\n", fq));
        }
        content.push('\n');
    }

    std::fs::write(&clarif_path, &content)?;

    // Advance phase to PostClarificationsCreated (pre-clarifications + reference context
    // absorbed by issue lifecycle)
    let mut sm = StateManager::load(&change_dir)?;
    sm.set_phase(StatePhase::ChangeInited)?;
    sm.save()?;

    let artifacts_written = vec![
        "pre_clarifications.md".to_string(),
        "STATE.yaml".to_string(),
    ];

    let next_actions = json!([workflow_common::next_action(
        interface,
        "sdd_run_change",
        json!({"change_id": change_id})
    )]);

    let result = json!({
        "status": "ok",
        "artifacts_written": artifacts_written,
        "next_actions": next_actions
    });

    Ok(serde_json::to_string_pretty(&result)?)
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/create_pre_clarifications/tests.md#source
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
        assert_eq!(parsed["phase"], "change_inited");
        assert_eq!(parsed["questions_count"], 1);
        assert_eq!(parsed["next"], "sdd_run_change");
        assert!(parsed["artifacts"].as_array().unwrap().len() >= 1);

        let file_path = project_root.join(".aw/changes/add-oauth/pre_clarifications.md");
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

        let file_path = project_root.join(".aw/changes/multi-test/pre_clarifications.md");
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

        // Verify content
        let file_path = project_root.join(".aw/changes/append-mcp-test/pre_clarifications.md");
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("## Q1: Initial Topic"));
        assert!(content.contains("## Issue #188"));
        assert!(content.contains("### Q1: Auth Method"));
    }

    // --- Workflow + artifact tests (post-groups removal) ---

    fn setup_change(change_id: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        std::fs::create_dir_all(&change_dir).unwrap();

        crate::test_util::write_minimal_issue(tmp.path(), change_id);
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = StatePhase::ChangeInited;
        sm.save().unwrap();

        std::fs::write(change_dir.join("user_input.md"), "Test").unwrap();

        tmp
    }

    #[test]
    fn test_artifact_pre_clarifications_writes_file() {
        let tmp = setup_change("art-test");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "art-test",
            "answers": [
                {
                    "topic": "Scope",
                    "answer": "sdd"
                }
            ]
        });
        let result = execute_artifact_pre_clarifications(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        // File written
        let file_path = tmp
            .path()
            .join(".aw/changes/art-test/pre_clarifications.md");
        assert!(file_path.exists());
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("status: answered"));
        assert!(content.contains("### Q1: Scope"));

        // Phase advanced
        let change_dir = tmp.path().join(".aw/changes/art-test");
        let sm = StateManager::load(&change_dir).unwrap();
        assert_eq!(*sm.phase(), StatePhase::ChangeInited);
    }
}
// CODEGEN-END
