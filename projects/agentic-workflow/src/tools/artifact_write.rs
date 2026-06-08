// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/artifact_write.md#source
// CODEGEN-BEGIN
//! sdd_write_artifact — Unified Artifact Writer
//!
//! Adapter that dispatches to existing handlers based on (artifact, action).
//! See spec: .aw/tech-design/sdd/tools/write-artifact.md

use super::{get_required_string, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

/// @spec projects/agentic-workflow/tech-design/core/tools/artifact_write.md#source
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_write_artifact".to_string(),
        description: "Create, revise, review, or fetch any workflow artifact. Routes by (artifact, action) to the correct handler. Use artifact='change' action='create' to initialize a new change. Use artifact='issues_context' action='fetch' to fetch GitHub issues by labels or refs. Use artifact='main_spec' action='write' to write specs.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "artifact", "action", "payload"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path"
                },
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Required for change artifacts. Not used for main_spec."
                },
                "artifact": {
                    "type": "string",
                    "enum": [
                        "change",
                        "context_clarifications", "spec_clarifications",
                        "codebase_context", "spec_context", "knowledge_context",
                        "gap_codebase_spec", "gap_codebase_knowledge", "gap_spec_knowledge",
                        "proposal", "spec",
                        "main_spec",
                        "issues_context"
                    ],
                    "description": "Artifact type to write"
                },
                "action": {
                    "type": "string",
                    "enum": ["create", "revise", "review", "write", "fetch"],
                    "description": "create/revise/review for change artifacts. write for main_spec. fetch for issues_context."
                },
                "caller": {
                    "type": "string",
                    "enum": ["agent", "mainthread"],
                    "default": "mainthread",
                    "description": "Who is calling. Controls whether next hint is included."
                },
                "payload": {
                    "type": "object",
                    "description": "Action-specific parameters. Schema depends on (artifact, action)."
                },
                "issue": {
                    "type": "integer",
                    "description": "Issue number (DAG multi-issue mode). Controls per-issue append."
                },
                "iteration": {
                    "type": "integer",
                    "minimum": 1,
                    "default": 1
                }
            }
        }),
    }
}

/// @spec projects/agentic-workflow/tech-design/core/tools/artifact_write.md#source
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let artifact = get_required_string(args, "artifact")?;
    let action = get_required_string(args, "action")?;
    // MCP clients may send payload as a JSON string instead of an object.
    // Parse string payloads back into objects for uniform handling.
    let payload = match args.get("payload") {
        Some(v) if v.is_string() => {
            let s = v.as_str().unwrap_or("{}");
            serde_json::from_str(s).unwrap_or(json!({}))
        }
        Some(v) => v.clone(),
        None => json!({}),
    };

    // Build args with the normalized (parsed) payload for downstream handlers
    let normalized_args = {
        let mut a = args.clone();
        if let Some(obj) = a.as_object_mut() {
            obj.insert("payload".to_string(), payload.clone());
        }
        a
    };

    match (artifact.as_str(), action.as_str()) {
        // ── Change initialization ──
        ("change", "create") => super::init_change::execute(&normalized_args, project_root),

        // ── Static artifacts (write) ──
        ("main_spec", "write") => {
            let handler_args = merge_base_and_payload(args, &payload);
            super::knowledge::execute_write_main_spec(&handler_args, project_root)
        }

        // ── Change artifacts: create / revise ──
        ("context_clarifications", "create" | "revise") => {
            dispatch_clarifications_create(args, &payload, project_root)
        }
        ("spec_clarifications", act @ ("create" | "revise")) => {
            let handler_args = merge_base_and_payload(args, &payload);
            super::clarifications::execute_post_clarifications(&handler_args, project_root, act)
        }
        (
            "codebase_context"
            | "spec_context"
            | "knowledge_context"
            | "gap_codebase_spec"
            | "gap_codebase_knowledge"
            | "gap_spec_knowledge",
            "create" | "revise",
        ) => dispatch_context_create(&artifact, args, &payload, project_root),
        ("proposal", "create" | "revise") => {
            anyhow::bail!("The proposal artifact type has been removed. Use spec artifacts directly via sdd_run_change.")
        }
        ("spec", "create" | "revise") => {
            let handler_args = merge_base_and_payload(args, &payload);
            super::spec::execute(&handler_args, project_root)
        }

        // ── Issues context: fetch ──
        ("issues_context", "fetch") => dispatch_issues_fetch(args, &payload, project_root),

        // ── Change artifacts: review ──
        (art, "review") => dispatch_review(art, args, &payload, project_root),

        _ => anyhow::bail!(
            "Invalid (artifact, action) combination: ({}, {})",
            artifact,
            action
        ),
    }
}

/// Dispatch clarifications create/revise.
/// When `issue` param is present, use append mode.
fn dispatch_clarifications_create(
    args: &Value,
    payload: &Value,
    project_root: &Path,
) -> Result<String> {
    let has_issue = args.get("issue").is_some();
    let handler_args = merge_base_and_payload(args, payload);

    if has_issue {
        super::clarifications::execute_append(&handler_args, project_root)
    } else {
        super::clarifications::execute(&handler_args, project_root)
    }
}

/// Dispatch context create/revise by injecting context_type from artifact name.
fn dispatch_context_create(
    artifact: &str,
    args: &Value,
    payload: &Value,
    project_root: &Path,
) -> Result<String> {
    let mut handler_args = merge_base_and_payload(args, payload);
    // Inject context_type from the artifact name
    if let Some(obj) = handler_args.as_object_mut() {
        obj.insert("context_type".to_string(), json!(artifact));
    }
    super::context::execute(&handler_args, project_root)
}

/// Dispatch review by mapping artifact name to the `file` param of sdd_review_file.
fn dispatch_review(
    artifact: &str,
    args: &Value,
    payload: &Value,
    project_root: &Path,
) -> Result<String> {
    let mut handler_args = merge_base_and_payload(args, payload);
    if let Some(obj) = handler_args.as_object_mut() {
        obj.insert("file".to_string(), json!(artifact));
    }
    super::review::execute(&handler_args, project_root)
}

/// Dispatch issues_context fetch.
/// Supports `payload.labels` (list via gh, then fetch) or `payload.issue_refs` (direct fetch).
fn dispatch_issues_fetch(args: &Value, payload: &Value, project_root: &Path) -> Result<String> {
    let has_labels = payload.get("labels").and_then(|v| v.as_array()).is_some();
    let has_issue_refs = payload
        .get("issue_refs")
        .and_then(|v| v.as_array())
        .is_some();

    if !has_labels && !has_issue_refs {
        anyhow::bail!("issues_context/fetch requires either 'labels' or 'issue_refs' in payload");
    }

    let issue_refs: Vec<String> = if has_labels {
        let labels: Vec<String> = payload["labels"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        if labels.is_empty() {
            anyhow::bail!("payload.labels must contain at least one label");
        }
        let nums = super::fetch_issues::list_issues_by_labels(&labels, None, Some(project_root))?;
        if nums.is_empty() {
            anyhow::bail!("No issues found matching labels: {}", labels.join(", "));
        }
        nums.iter().map(|n| format!("#{}", n)).collect()
    } else {
        payload["issue_refs"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    };

    // Build args for fetch_issues::execute
    let handler_args = json!({
        "project_path": args.get("project_path"),
        "change_id": args.get("change_id"),
        "issue_refs": issue_refs,
    });

    super::fetch_issues::execute(&handler_args, project_root)
}

/// Merge top-level base fields (project_path, change_id, caller, iteration)
/// with payload fields into a flat args object for the underlying handler.
fn merge_base_and_payload(base: &Value, payload: &Value) -> Value {
    let mut merged = serde_json::Map::new();

    // Copy base fields
    if let Some(obj) = base.as_object() {
        for key in &["project_path", "change_id", "caller", "iteration"] {
            if let Some(v) = obj.get(*key) {
                merged.insert((*key).to_string(), v.clone());
            }
        }
    }

    // Merge all payload fields (payload fields override base on collision)
    if let Some(p) = payload.as_object() {
        for (k, v) in p {
            merged.insert(k.clone(), v.clone());
        }
    }

    Value::Object(merged)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_base_and_payload() {
        let base = json!({
            "project_path": "/tmp",
            "change_id": "test",
            "artifact": "proposal",
            "action": "create",
            "caller": "agent",
            "payload": {"scope": "patch"}
        });
        let payload = json!({"scope": "patch", "spec_plan": []});
        let merged = merge_base_and_payload(&base, &payload);

        assert_eq!(merged["project_path"], "/tmp");
        assert_eq!(merged["change_id"], "test");
        assert_eq!(merged["caller"], "agent");
        assert_eq!(merged["scope"], "patch");
        // artifact and action should NOT be in merged (not in base field list)
        assert!(merged.get("artifact").is_none());
        assert!(merged.get("action").is_none());
    }

    #[test]
    fn test_issues_context_requires_labels_or_refs() {
        let args = json!({
            "project_path": "/tmp",
            "change_id": "test",
            "artifact": "issues_context",
            "action": "fetch",
            "payload": {}
        });
        let result = execute(&args, Path::new("/tmp"));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires either 'labels' or 'issue_refs'"));
    }

    #[test]
    fn test_invalid_combination() {
        let args = json!({
            "artifact": "nonexistent_artifact",
            "action": "create",
            "payload": {}
        });
        let result = execute(&args, Path::new("/tmp"));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid (artifact, action)"));
    }
}

// CODEGEN-END
