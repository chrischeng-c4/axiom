---
id: projects-sdd-src-tools-create-post-clarifications-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# Standardized projects/agentic-workflow/src/tools/create_post_clarifications.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_post_clarifications.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/create_post_clarifications.rs | function | pub | 48 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/create_post_clarifications.rs | function | pub | 166 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/create_post_clarifications.rs | function | pub | 136 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_post_clarifications.rs | function | pub | 22 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
//! Create tools for post-clarifications.
//!
//! - `sdd_workflow_create_post_clarifications` — per-group router returning prompt or done
//! - `sdd_artifact_create_post_clarifications` — writes `groups/{group_id}/post_clarifications.md`

use crate::models::WorkflowArtifact;
use crate::state::StateManager;
use crate::tools::workflow_common;
use crate::tools::{get_required_string, ToolDefinition};
use crate::workflow::scope;
use crate::Result;
use chrono::Local;
use serde_json::{json, Value};
use std::path::Path;

// ─── Tool Definitions ────────────────────────────────────────────────────────

/// MCP tool definition for sdd_workflow_create_post_clarifications
pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_create_post_clarifications".to_string(),
        description:
            "Return prompt for mainthread to create post-clarifications for next incomplete group"
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

/// MCP tool definition for sdd_artifact_create_post_clarifications
pub fn artifact_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_artifact_create_post_clarifications".to_string(),
        description: "Write post-clarifications artifact for a group".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "group_id"],
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
                    "description": "Group ID"
                },
                "skipped": {
                    "type": "boolean",
                    "description": "True if no clarifications needed (skip-fast path)"
                },
                "questions": {
                    "type": "array",
                    "description": "Q&A pairs from user clarification",
                    "items": {
                        "type": "object",
                        "required": ["topic", "question", "answer"],
                        "properties": {
                            "topic": {
                                "type": "string",
                                "description": "Short topic label"
                            },
                            "question": {
                                "type": "string",
                                "description": "The question asked"
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
                },
                "contradictions": {
                    "type": "array",
                    "description": "Contradictions found between specs and requirements",
                    "items": {
                        "type": "object",
                        "required": ["spec_id", "requirement", "conflict", "resolution"],
                        "properties": {
                            "spec_id": {
                                "type": "string",
                                "description": "Spec that conflicts"
                            },
                            "requirement": {
                                "type": "string",
                                "description": "The requirement being contradicted"
                            },
                            "conflict": {
                                "type": "string",
                                "description": "Description of the conflict"
                            },
                            "resolution": {
                                "type": "string",
                                "description": "How the conflict was resolved"
                            }
                        }
                    }
                }
            }
        }),
    }
}

// ─── Workflow Orchestration ──────────────────────────────────────────────────

/// Execute sdd_workflow_create_post_clarifications.
///
/// Returns a prompt for the agent to author post-clarifications.
pub async fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    workflow_common::validate_change_id(&change_id)?;

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);
    workflow_common::validate_change_dir(&change_dir, project_root)?;
    let interface = workflow_common::load_interface(project_root);

    // If artifact already written, return phase_complete
    let post_clar_path = change_dir.join("post_clarifications.md");
    if post_clar_path.exists() {
        let result = json!({
            "status": "phase_complete",
            "prompt": "Post-clarifications already created. Proceeding to spec creation.",
            "next_actions": [
                workflow_common::next_action(interface, "sdd_run_change", json!({"change_id": change_id}))
            ]
        });
        return Ok(serde_json::to_string_pretty(&result)?);
    }

    build_create_prompt(&change_id, &change_dir, project_root).await
}

// ─── Artifact Write ──────────────────────────────────────────────────────────

/// Execute sdd_artifact_create_post_clarifications.
///
/// Writes `post_clarifications.md` for the change.
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let skipped = args
        .get("skipped")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
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

    workflow_common::validate_change_id(&change_id)?;
    let interface = workflow_common::load_interface(project_root);

    let change_dir = workflow_common::resolve_change_dir(project_root, &change_id);

    if !change_dir.exists() {
        anyhow::bail!("Change directory not found: {}", change_dir.display());
    }

    let artifact_path = change_dir.join("post_clarifications.md");
    let today = Local::now().format("%Y-%m-%d").to_string();

    let scope_summary = args.get("scope_summary");
    let content = render_post_clarifications(
        &change_id,
        &today,
        skipped,
        &questions,
        &contradictions,
        scope_summary,
    );
    std::fs::write(&artifact_path, &content)?;

    // Phase stays at PostClarificationsCreated (already set by the state machine)
    // No groups_progress tracking needed — single scope per change.
    let _ = StateManager::load(&change_dir)?;

    let artifacts_written = vec!["post_clarifications.md".to_string()];

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

// ─── Prompt Builder ──────────────────────────────────────────────────────────

/// Build CREATE prompt for post-clarifications.
async fn build_create_prompt(
    change_id: &str,
    change_dir: &Path,
    project_root: &Path,
) -> Result<String> {
    let project_path = project_root.display();

    // Extract scope for spec filtering
    let scope_info = scope::extract_scope(change_dir);
    let sdd_config = crate::models::SddConfig::load(project_root).ok();
    let filtered_specs =
        scope::pre_filter_specs(&scope_info.spec_groups, project_root, sdd_config.as_ref());

    let spec_reading_hint = if filtered_specs.is_empty() {
        format!(
            "- List specs under `{}/.aw/tech-design/` using Glob and read the most relevant ones",
            project_path
        )
    } else {
        format!(
            "- Read high/medium relevance specs listed in reference_context.md (under `{}/.aw/tech-design/`)",
            project_path
        )
    };

    let prompt = format!(
        r#"# Task: Post-Clarification for Change '{change_id}'

## Context Sources

Read these files before analysis:
1. `{project_path}/.aw/changes/{change_id}/user_input.md`
2. `{project_path}/.aw/changes/{change_id}/pre_clarifications.md`
3. `{project_path}/.aw/changes/{change_id}/reference_context.md`
4. Actual specs — read high/medium relevance specs from reference_context.md

{spec_reading_hint}

## Instructions

### Step 1: Systematic Contradiction Mining

For each high-relevance spec from reference_context.md:
1. Read the spec file
2. For each requirement, explicitly ask: "Does this spec define a convention or pattern that conflicts with this requirement?"
3. Look specifically for:
   - Naming conventions that differ from the user's proposal
   - Data formats or API patterns that would be inconsistent
   - Error handling approaches that conflict
   - Existing constraints that limit the proposed approach

### Step 2: Assumption Surfacing

List implicit assumptions from user input that the referenced specs don't address.

### Step 3: Scope Summary (MANDATORY)

Write a Scope Summary with cross-references:

- **Problem**: ref to user_input.md sections that define the gap
- **Success Criteria**: acceptance criteria + pre_clarifications answers that confirmed behavior
- **Boundary**: in scope, out of scope, constraints

Use → refs to point to specific sections — do NOT duplicate content.

### Step 4: Decision

- **No conflicts found** → Call artifact tool with `skipped: true` + `scope_summary`.
- **Conflicts found** → Use AskUserQuestion, then call artifact tool with resolved questions/contradictions + `scope_summary`.

## CLI Commands

```
score artifact create-post-clarifications {change_id} .aw/changes/{change_id}/payloads/create-post-clarifications.json
```"#,
    );

    let interface = workflow_common::load_interface(project_root);
    let executor = workflow_common::get_executor_chain(
        project_root,
        WorkflowArtifact::CreatePostClarifications,
    );

    workflow_common::build_workflow_response(
        change_dir,
        change_id,
        "create_post_clarifications",
        prompt,
        executor,
        json!({}),
        interface,
        project_root,
    )
    .await
}

// ─── Rendering ───────────────────────────────────────────────────────────────

/// Render post-clarifications markdown content.
fn render_post_clarifications(
    change_id: &str,
    date: &str,
    skipped: bool,
    questions: &[Value],
    contradictions: &[Value],
    scope_summary: Option<&Value>,
) -> String {
    let status = if skipped { "skipped" } else { "clarified" };

    let mut md = format!(
        "---\nchange: {}\ndate: {}\nstatus: {}\n---\n\n# Post-Clarifications\n\n",
        change_id, date, status
    );

    // Scope Summary — always rendered (mandatory)
    md.push_str("## Scope Summary\n\n");
    if let Some(summary) = scope_summary {
        let problem = summary
            .get("problem")
            .and_then(|v| v.as_str())
            .unwrap_or("(not provided)");
        let success = summary
            .get("success")
            .and_then(|v| v.as_str())
            .unwrap_or("(not provided)");
        let boundary = summary
            .get("boundary")
            .and_then(|v| v.as_str())
            .unwrap_or("(not provided)");
        md.push_str(&format!("### Problem\n{}\n\n", problem));
        md.push_str(&format!("### Success Criteria\n{}\n\n", success));
        md.push_str(&format!("### Boundary\n{}\n\n", boundary));
    } else {
        md.push_str("### Problem\n→ See requirements.md\n\n");
        md.push_str("### Success Criteria\n→ See requirements.md § Acceptance Criteria\n\n");
        md.push_str("### Boundary\n- In scope: See reference_context.md § Spec Plan\n\n");
    }

    if skipped && questions.is_empty() && contradictions.is_empty() {
        return md;
    }

    // Questions section
    if !questions.is_empty() {
        md.push_str("## Questions\n\n");
        for (i, qa) in questions.iter().enumerate() {
            let topic = qa
                .get("topic")
                .and_then(|v| v.as_str())
                .unwrap_or("General");
            let question = qa.get("question").and_then(|v| v.as_str()).unwrap_or("");
            let answer = qa.get("answer").and_then(|v| v.as_str()).unwrap_or("");
            let rationale = qa.get("rationale").and_then(|v| v.as_str()).unwrap_or("");

            md.push_str(&format!("### Q{}: {}\n", i + 1, topic));
            md.push_str(&format!("- **Question**: {}\n", question));
            md.push_str(&format!("- **Answer**: {}\n", answer));
            if !rationale.is_empty() {
                md.push_str(&format!("- **Rationale**: {}\n", rationale));
            }
            md.push('\n');
        }
    }

    // Contradictions section
    if !contradictions.is_empty() {
        md.push_str("## Contradictions\n\n");
        for (i, c) in contradictions.iter().enumerate() {
            let spec_id = c.get("spec_id").and_then(|v| v.as_str()).unwrap_or("");
            let requirement = c.get("requirement").and_then(|v| v.as_str()).unwrap_or("");
            let conflict = c.get("conflict").and_then(|v| v.as_str()).unwrap_or("");
            let resolution = c.get("resolution").and_then(|v| v.as_str()).unwrap_or("");

            md.push_str(&format!("### C{}: {} vs requirement\n", i + 1, spec_id));
            md.push_str(&format!("- **Spec**: {}\n", spec_id));
            md.push_str(&format!("- **Requirement**: {}\n", requirement));
            md.push_str(&format!("- **Conflict**: {}\n", conflict));
            md.push_str(&format!("- **Resolution**: {}\n\n", resolution));
        }
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::state::StatePhase;
    use tempfile::TempDir;

    fn setup_change(change_id: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes").join(change_id);
        std::fs::create_dir_all(&change_dir).unwrap();

        crate::test_util::write_minimal_issue(tmp.path(), change_id);
        // Create STATE.yaml
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.state_mut().phase = StatePhase::ChangeInited;
        sm.save().unwrap();

        // user_input.md + reference_context.md (pre-existing)
        std::fs::write(change_dir.join("user_input.md"), "Test change").unwrap();
        std::fs::write(
            change_dir.join("reference_context.md"),
            "---\nchange: test\n---\n\n# Reference Context\n",
        )
        .unwrap();

        tmp
    }

    fn read_prompt(parsed: &Value, project_root: &std::path::Path) -> String {
        if let Some(p) = parsed["prompt"].as_str() {
            return p.to_string();
        }
        let rel = parsed["prompt_path"]
            .as_str()
            .expect("Expected prompt_path in response");
        let prompt_path = project_root.join(rel);
        std::fs::read_to_string(&prompt_path)
            .unwrap_or_else(|_| panic!("No prompt at {:?}", prompt_path))
    }

    #[tokio::test]
    async fn test_workflow_returns_create_prompt() {
        let tmp = setup_change("post-test");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "post-test"
        });
        let result = execute_workflow(&args, tmp.path()).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        let prompt = read_prompt(&parsed, tmp.path());
        assert!(prompt.contains("Post-Clarification"));
        assert!(prompt.contains("Contradiction Mining"));
    }

    #[test]
    fn test_artifact_writes_skipped() {
        let tmp = setup_change("skip-test");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "skip-test",
            "skipped": true
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert!(parsed["artifacts_written"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str().unwrap().contains("post_clarifications.md")));

        // Verify file content
        let file_path = tmp
            .path()
            .join(".aw/changes/skip-test/post_clarifications.md");
        assert!(file_path.exists());
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("status: skipped"));
        assert!(content.contains("## Scope Summary"));
    }

    #[test]
    fn test_artifact_writes_with_questions_and_contradictions() {
        let tmp = setup_change("clarify-test");
        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "change_id": "clarify-test",
            "questions": [{
                "topic": "Naming",
                "question": "Spec uses snake_case but you proposed camelCase. Which?",
                "answer": "snake_case",
                "rationale": "Follow existing convention"
            }],
            "contradictions": [{
                "spec_id": "api-conventions",
                "requirement": "Use REST endpoints",
                "conflict": "Spec defines GraphQL-first approach",
                "resolution": "Use REST with optional GraphQL"
            }]
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");

        let file_path = tmp
            .path()
            .join(".aw/changes/clarify-test/post_clarifications.md");
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("status: clarified"));
        assert!(content.contains("### Q1: Naming"));
        assert!(content.contains("### C1: api-conventions vs requirement"));
        assert!(content.contains("snake_case"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/create_post_clarifications.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Regenerate the post-clarifications tool module directly from the source
      template. The module is a legacy MCP workflow surface; source ownership
      is the current regenerable bridge until smaller semantic tool templates
      are worth extracting.
```
