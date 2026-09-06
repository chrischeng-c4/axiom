use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde_json::{json, Value};

use crate::models::{
    Defect, DefectSeverity, DefectStatus, Feature, FeatureStatus, Prd, PrdStatus,
    Priority, Project, ReviewComment, ReviewSeverity, Task, TaskStatus, TechDesign,
    TechDesignStatus,
};
use crate::store::{
    execute_gate, get_next_actionable_task, get_task_context, verify_and_merge_task, PmStore,
};

fn tool(name: &str, description: &str, props: Value, required: &[&str]) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": props,
            "required": required,
        }
    })
}

pub fn tool_definitions() -> Vec<Value> {
    vec![
        // Projects
        tool(
            "pm_init_project",
            "Initialize or register a project in the local PM repository",
            json!({
                "id": { "type": "string", "description": "Unique identifier for the project (e.g. 'proj_axiom')" },
                "name": { "type": "string", "description": "Human-readable name of the project" },
                "description": { "type": "string", "description": "High level description of project purpose" },
                "root_path": { "type": "string", "description": "Root workspace path (default: current directory)" },
                "default_gate_cmd": { "type": "string", "description": "Default test verification command for this project (e.g. 'cap cargo test -p pm')" }
            }),
            &["id", "name", "description"],
        ),
        tool(
            "pm_get_project_summary",
            "Retrieve an executive dashboard summary of project metrics, active features, PRDs, task distribution, and defects",
            json!({
                "project_id": { "type": "string", "description": "Project ID" }
            }),
            &["project_id"],
        ),
        tool(
            "pm_list_projects",
            "List all registered projects in the PM repository",
            json!({}),
            &[],
        ),
        // PRDs
        tool(
            "pm_save_prd",
            "Create or update a Product Requirements Document (PRD) with full Markdown specification",
            json!({
                "project_id": { "type": "string", "description": "Project ID this PRD belongs to" },
                "prd_id": { "type": "string", "description": "Unique PRD ID (e.g. 'prd_auth')" },
                "title": { "type": "string", "description": "Title of the PRD" },
                "content": { "type": "string", "description": "Full Markdown content containing goals, user stories, acceptance criteria" },
                "status": { "type": "string", "enum": ["draft", "in_review", "approved", "deprecated"], "description": "Status of the PRD (default: draft)" },
                "version": { "type": "string", "description": "Version string (e.g. 'v1.0')" }
            }),
            &["project_id", "prd_id", "title", "content"],
        ),
        tool(
            "pm_get_prd",
            "Retrieve a PRD by its unique identifier",
            json!({
                "prd_id": { "type": "string", "description": "PRD ID" }
            }),
            &["prd_id"],
        ),
        tool(
            "pm_list_prds",
            "List all PRDs belonging to a project",
            json!({
                "project_id": { "type": "string", "description": "Project ID" }
            }),
            &["project_id"],
        ),
        // Tech Designs
        tool(
            "pm_save_tech_design",
            "Create or update a Technical Design document with full Markdown architecture and technical specs",
            json!({
                "project_id": { "type": "string", "description": "Project ID" },
                "td_id": { "type": "string", "description": "Unique Tech Design ID (e.g. 'td_auth_flow')" },
                "title": { "type": "string", "description": "Title of the technical design" },
                "content": { "type": "string", "description": "Markdown technical design: architecture, data models, APIs, trade-offs" },
                "prd_id": { "type": "string", "description": "Optional associated PRD ID" },
                "status": { "type": "string", "enum": ["draft", "in_review", "approved", "superseded"], "description": "Status (default: draft)" },
                "version": { "type": "string", "description": "Version string (e.g. 'v1.0')" }
            }),
            &["project_id", "td_id", "title", "content"],
        ),
        tool(
            "pm_get_tech_design",
            "Retrieve a Technical Design document by ID",
            json!({
                "td_id": { "type": "string", "description": "Tech Design ID" }
            }),
            &["td_id"],
        ),
        tool(
            "pm_list_tech_designs",
            "List all Technical Design documents for a project",
            json!({
                "project_id": { "type": "string", "description": "Project ID" }
            }),
            &["project_id"],
        ),
        // Features
        tool(
            "pm_create_feature",
            "Define a high-level feature or epic linking PRD and Tech Design specs",
            json!({
                "project_id": { "type": "string", "description": "Project ID" },
                "feature_id": { "type": "string", "description": "Feature ID (e.g. 'feat_jwt_auth')" },
                "title": { "type": "string", "description": "Feature title" },
                "description": { "type": "string", "description": "Detailed feature description and objectives" },
                "prd_id": { "type": "string", "description": "Optional linked PRD ID" },
                "tech_design_id": { "type": "string", "description": "Optional linked Tech Design ID" },
                "priority": { "type": "string", "enum": ["low", "medium", "high", "critical"], "description": "Priority (default: medium)" }
            }),
            &["project_id", "feature_id", "title", "description"],
        ),
        tool(
            "pm_update_feature",
            "Update the status, priority, or description of an existing feature",
            json!({
                "feature_id": { "type": "string", "description": "Feature ID" },
                "status": { "type": "string", "enum": ["backlog", "planned", "in_progress", "completed", "cancelled"], "description": "New feature status" },
                "priority": { "type": "string", "enum": ["low", "medium", "high", "critical"], "description": "New priority" },
                "description": { "type": "string", "description": "Updated description" }
            }),
            &["feature_id"],
        ),
        tool(
            "pm_get_feature",
            "Retrieve feature details and all attached sub-tasks",
            json!({
                "feature_id": { "type": "string", "description": "Feature ID" }
            }),
            &["feature_id"],
        ),
        tool(
            "pm_list_features",
            "List features of a project, optionally filtered by status",
            json!({
                "project_id": { "type": "string", "description": "Project ID" },
                "status": { "type": "string", "enum": ["backlog", "planned", "in_progress", "completed", "cancelled"], "description": "Optional status filter" }
            }),
            &["project_id"],
        ),
        // Tasks
        tool(
            "pm_create_task",
            "Create an actionable task for an AI Agent, specifying instructions, priority, assignee, and blocked_by dependencies",
            json!({
                "project_id": { "type": "string", "description": "Project ID" },
                "task_id": { "type": "string", "description": "Task ID (e.g. 'task_user_model')" },
                "title": { "type": "string", "description": "Concise task title" },
                "description": { "type": "string", "description": "Detailed execution instructions, context, and acceptance criteria" },
                "feature_id": { "type": "string", "description": "Optional parent Feature ID" },
                "priority": { "type": "string", "enum": ["low", "medium", "high", "critical"], "description": "Priority (default: medium)" },
                "assignee": { "type": "string", "description": "Optional agent role (e.g. 'dev', 'architect', 'e2e-dev')" },
                "blocked_by": { "type": "array", "items": { "type": "string" }, "description": "Array of prerequisite task IDs that must be 'done' before this task can start" }
            }),
            &["project_id", "task_id", "title", "description"],
        ),
        tool(
            "pm_update_task_status",
            "Update a task's status and optionally record execution result notes/evidence",
            json!({
                "task_id": { "type": "string", "description": "Task ID" },
                "status": { "type": "string", "enum": ["todo", "in_progress", "in_review", "done", "blocked"], "description": "New status" },
                "result_summary": { "type": "string", "description": "Optional completion notes, execution summary, or verification evidence" }
            }),
            &["task_id", "status"],
        ),
        tool(
            "pm_list_tasks",
            "List tasks with flexible filtering by project, feature, status, or assignee",
            json!({
                "project_id": { "type": "string", "description": "Project ID" },
                "feature_id": { "type": "string", "description": "Optional parent Feature ID filter" },
                "status": { "type": "string", "enum": ["todo", "in_progress", "in_review", "done", "blocked"], "description": "Optional status filter" },
                "assignee": { "type": "string", "description": "Optional assignee role filter" }
            }),
            &["project_id"],
        ),
        tool(
            "pm_get_next_actionable_task",
            "Autonomous Agent Dispatcher: Finds the highest-priority 'todo' task whose prerequisites ('blocked_by') are all 'done'",
            json!({
                "project_id": { "type": "string", "description": "Project ID" },
                "assignee": { "type": "string", "description": "Optional filter for specific agent role" }
            }),
            &["project_id"],
        ),
        tool(
            "pm_get_task_context",
            "Agent Single-Shot Context Tool: Generates a complete, prompt-ready markdown payload containing the task spec, parent feature, linked PRD, and tech design",
            json!({
                "task_id": { "type": "string", "description": "Task ID" }
            }),
            &["task_id"],
        ),
        // Local Gate & Merge (Replacing gh pr create & gh pr merge)
        tool(
            "pm_verify_gate",
            "Execute local test verification gate for a task, recording immutable output, exit code, and commit hash receipt",
            json!({
                "task_id": { "type": "string", "description": "Task ID being verified" },
                "command": { "type": "string", "description": "Verification command to execute (e.g. 'cap cargo test -p pm'). If omitted, uses project's default_gate_cmd." }
            }),
            &["task_id"],
        ),
        tool(
            "pm_get_gate_history",
            "Retrieve verification gate history and receipts for a task",
            json!({
                "task_id": { "type": "string", "description": "Task ID" }
            }),
            &["task_id"],
        ),
        tool(
            "pm_merge_change",
            "Local Auto-Merge: Verifies passing gate run and zero blocking review comments, then merges local working branch into target branch, advances task to 'done', and unblocks downstream tasks",
            json!({
                "task_id": { "type": "string", "description": "Task ID to merge" },
                "target_branch": { "type": "string", "description": "Target branch to merge into (default: 'main')" },
                "strategy": { "type": "string", "enum": ["squash", "merge"], "description": "Merge strategy (default: 'squash')" }
            }),
            &["task_id"],
        ),
        // Defect Tracking (Replacing gh issue)
        tool(
            "pm_report_defect",
            "Report a bug or defect, optionally auto-creating a high-priority blocker task for resolution",
            json!({
                "project_id": { "type": "string", "description": "Project ID" },
                "title": { "type": "string", "description": "Defect summary" },
                "description": { "type": "string", "description": "Detailed explanation of defect" },
                "severity": { "type": "string", "enum": ["critical", "major", "minor", "cosmetic"], "description": "Defect severity (default: major)" },
                "feature_id": { "type": "string", "description": "Optional associated feature ID" },
                "task_id": { "type": "string", "description": "Optional associated task ID" },
                "reproduction_steps": { "type": "string", "description": "Steps to reproduce the bug" },
                "error_log": { "type": "string", "description": "Error logs or stack trace" },
                "auto_create_task": { "type": "boolean", "description": "Automatically spawn an actionable task to fix this defect (default: true)" }
            }),
            &["project_id", "title", "description"],
        ),
        tool(
            "pm_resolve_defect",
            "Mark a defect as resolved or won't fix with resolution notes",
            json!({
                "defect_id": { "type": "string", "description": "Defect ID" },
                "status": { "type": "string", "enum": ["resolved", "wont_fix"], "description": "Resolution status (default: resolved)" },
                "resolution_notes": { "type": "string", "description": "Explanation of fix or reasoning" }
            }),
            &["defect_id"],
        ),
        tool(
            "pm_list_defects",
            "List defects for a project with optional status and severity filters",
            json!({
                "project_id": { "type": "string", "description": "Project ID" },
                "status": { "type": "string", "enum": ["open", "in_progress", "resolved", "wont_fix"], "description": "Optional status filter" },
                "severity": { "type": "string", "enum": ["critical", "major", "minor", "cosmetic"], "description": "Optional severity filter" }
            }),
            &["project_id"],
        ),
        // Code Review System (Replacing gh pr review)
        tool(
            "pm_add_review_comment",
            "Add a structured code review comment to a task. If severity is 'blocking', prevents pm_merge_change until resolved.",
            json!({
                "task_id": { "type": "string", "description": "Task ID under review" },
                "reviewer": { "type": "string", "description": "Reviewer identity (e.g. 'qa-agent', 'human')" },
                "content": { "type": "string", "description": "Review comment, suggestion, or requested change" },
                "severity": { "type": "string", "enum": ["blocking", "suggestion", "praise"], "description": "Comment severity (default: blocking)" },
                "file_path": { "type": "string", "description": "Optional target file path" },
                "line_number": { "type": "integer", "description": "Optional target line number" }
            }),
            &["task_id", "reviewer", "content"],
        ),
        tool(
            "pm_resolve_review_comment",
            "Mark a review comment as resolved",
            json!({
                "comment_id": { "type": "string", "description": "Review comment ID" },
                "resolution_note": { "type": "string", "description": "Optional resolution note" }
            }),
            &["comment_id"],
        ),
        tool(
            "pm_list_review_comments",
            "List review comments attached to a task",
            json!({
                "task_id": { "type": "string", "description": "Task ID" },
                "unresolved_only": { "type": "boolean", "description": "Filter only unresolved comments (default: false)" }
            }),
            &["task_id"],
        ),
    ]
}

pub async fn call_tool(store: &PmStore, name: &str, args: &Value) -> Result<Value> {
    let now = Utc::now().to_rfc3339();

    match name {
        "pm_init_project" => {
            let id = get_str(args, "id")?;
            let name = get_str(args, "name")?;
            let description = get_str(args, "description")?;
            let root_path = args.get("root_path").and_then(Value::as_str).unwrap_or(".").to_string();
            let default_gate_cmd = args.get("default_gate_cmd").and_then(Value::as_str).map(ToString::to_string);

            let proj = Project {
                id: id.clone(),
                name,
                description,
                root_path,
                default_gate_cmd,
                created_at: now.clone(),
                updated_at: now,
            };

            store.write(|s| {
                s.upsert_project(proj.clone());
                Ok(())
            }).await?;

            text_response(format!("Project '{}' initialized successfully.", id))
        }

        "pm_get_project_summary" => {
            let project_id = get_str(args, "project_id")?;
            let summary = store.read(|s| s.get_project_summary(&project_id)).await;
            text_response(serde_json::to_string_pretty(&summary)?)
        }

        "pm_list_projects" => {
            let list = store.read(|s| s.list_projects().into_iter().cloned().collect::<Vec<_>>()).await;
            text_response(serde_json::to_string_pretty(&list)?)
        }

        "pm_save_prd" => {
            let project_id = get_str(args, "project_id")?;
            let prd_id = get_str(args, "prd_id")?;
            let title = get_str(args, "title")?;
            let content = get_str(args, "content")?;
            let status_str = args.get("status").and_then(Value::as_str).unwrap_or("draft");
            let version = args.get("version").and_then(Value::as_str).unwrap_or("v1.0").to_string();

            let status = match status_str {
                "in_review" => PrdStatus::InReview,
                "approved" => PrdStatus::Approved,
                "deprecated" => PrdStatus::Deprecated,
                _ => PrdStatus::Draft,
            };

            store.write(|s| {
                let created_at = s.get_prd(&prd_id).map(|p| p.created_at.clone()).unwrap_or_else(|| now.clone());
                let prd = Prd {
                    id: prd_id.clone(),
                    project_id,
                    title,
                    version,
                    status,
                    content,
                    created_at,
                    updated_at: now,
                };
                s.upsert_prd(prd);
                Ok(())
            }).await?;

            text_response(format!("PRD '{}' saved successfully.", prd_id))
        }

        "pm_get_prd" => {
            let prd_id = get_str(args, "prd_id")?;
            let prd = store.read(|s| s.get_prd(&prd_id).cloned()).await;
            match prd {
                Some(p) => text_response(serde_json::to_string_pretty(&p)?),
                None => bail!("PRD '{}' not found", prd_id),
            }
        }

        "pm_list_prds" => {
            let project_id = get_str(args, "project_id")?;
            let list = store.read(|s| s.list_prds(&project_id).into_iter().cloned().collect::<Vec<_>>()).await;
            text_response(serde_json::to_string_pretty(&list)?)
        }

        "pm_save_tech_design" => {
            let project_id = get_str(args, "project_id")?;
            let td_id = get_str(args, "td_id")?;
            let title = get_str(args, "title")?;
            let content = get_str(args, "content")?;
            let prd_id = args.get("prd_id").and_then(Value::as_str).map(ToString::to_string);
            let status_str = args.get("status").and_then(Value::as_str).unwrap_or("draft");
            let version = args.get("version").and_then(Value::as_str).unwrap_or("v1.0").to_string();

            let status = match status_str {
                "in_review" => TechDesignStatus::InReview,
                "approved" => TechDesignStatus::Approved,
                "superseded" => TechDesignStatus::Superseded,
                _ => TechDesignStatus::Draft,
            };

            store.write(|s| {
                let created_at = s.get_tech_design(&td_id).map(|t| t.created_at.clone()).unwrap_or_else(|| now.clone());
                let td = TechDesign {
                    id: td_id.clone(),
                    project_id,
                    prd_id,
                    title,
                    version,
                    status,
                    content,
                    created_at,
                    updated_at: now,
                };
                s.upsert_tech_design(td);
                Ok(())
            }).await?;

            text_response(format!("Tech Design '{}' saved successfully.", td_id))
        }

        "pm_get_tech_design" => {
            let td_id = get_str(args, "td_id")?;
            let td = store.read(|s| s.get_tech_design(&td_id).cloned()).await;
            match td {
                Some(t) => text_response(serde_json::to_string_pretty(&t)?),
                None => bail!("Tech Design '{}' not found", td_id),
            }
        }

        "pm_list_tech_designs" => {
            let project_id = get_str(args, "project_id")?;
            let list = store.read(|s| s.list_tech_designs(&project_id).into_iter().cloned().collect::<Vec<_>>()).await;
            text_response(serde_json::to_string_pretty(&list)?)
        }

        "pm_create_feature" => {
            let project_id = get_str(args, "project_id")?;
            let feature_id = get_str(args, "feature_id")?;
            let title = get_str(args, "title")?;
            let description = get_str(args, "description")?;
            let prd_id = args.get("prd_id").and_then(Value::as_str).map(ToString::to_string);
            let tech_design_id = args.get("tech_design_id").and_then(Value::as_str).map(ToString::to_string);
            let priority_str = args.get("priority").and_then(Value::as_str).unwrap_or("medium");
            let priority = Priority::parse_str(priority_str).unwrap_or_default();

            let feat = Feature {
                id: feature_id.clone(),
                project_id,
                prd_id,
                tech_design_id,
                title,
                description,
                status: FeatureStatus::Backlog,
                priority,
                created_at: now.clone(),
                updated_at: now,
            };

            store.write(|s| {
                s.upsert_feature(feat);
                Ok(())
            }).await?;

            text_response(format!("Feature '{}' created successfully.", feature_id))
        }

        "pm_update_feature" => {
            let feature_id = get_str(args, "feature_id")?;
            store.write(|s| {
                let mut feat = match s.get_feature(&feature_id).cloned() {
                    Some(f) => f,
                    None => bail!("Feature '{}' not found", feature_id),
                };

                if let Some(status_str) = args.get("status").and_then(Value::as_str) {
                    feat.status = match status_str {
                        "planned" => FeatureStatus::Planned,
                        "in_progress" => FeatureStatus::InProgress,
                        "completed" => FeatureStatus::Completed,
                        "cancelled" => FeatureStatus::Cancelled,
                        _ => FeatureStatus::Backlog,
                    };
                }
                if let Some(prio_str) = args.get("priority").and_then(Value::as_str) {
                    if let Some(p) = Priority::parse_str(prio_str) {
                        feat.priority = p;
                    }
                }
                if let Some(desc) = args.get("description").and_then(Value::as_str) {
                    feat.description = desc.to_string();
                }
                feat.updated_at = now;
                s.upsert_feature(feat);
                Ok(())
            }).await?;

            text_response(format!("Feature '{}' updated successfully.", feature_id))
        }

        "pm_get_feature" => {
            let feature_id = get_str(args, "feature_id")?;
            let (feat, tasks) = store.read(|s| {
                let f = s.get_feature(&feature_id).cloned();
                let t = s.tasks.values().filter(|task| task.feature_id.as_deref() == Some(&feature_id)).cloned().collect::<Vec<_>>();
                (f, t)
            }).await;

            match feat {
                Some(f) => {
                    let mut val = serde_json::to_value(&f)?;
                    val["tasks"] = json!(tasks);
                    text_response(serde_json::to_string_pretty(&val)?)
                }
                None => bail!("Feature '{}' not found", feature_id),
            }
        }

        "pm_list_features" => {
            let project_id = get_str(args, "project_id")?;
            let status = args.get("status").and_then(Value::as_str).and_then(|s| match s {
                "backlog" => Some(FeatureStatus::Backlog),
                "planned" => Some(FeatureStatus::Planned),
                "in_progress" => Some(FeatureStatus::InProgress),
                "completed" => Some(FeatureStatus::Completed),
                "cancelled" => Some(FeatureStatus::Cancelled),
                _ => None,
            });

            let list = store.read(|s| s.list_features(&project_id, status).into_iter().cloned().collect::<Vec<_>>()).await;
            text_response(serde_json::to_string_pretty(&list)?)
        }

        "pm_create_task" => {
            let project_id = get_str(args, "project_id")?;
            let task_id = get_str(args, "task_id")?;
            let title = get_str(args, "title")?;
            let description = get_str(args, "description")?;
            let feature_id = args.get("feature_id").and_then(Value::as_str).map(ToString::to_string);
            let priority_str = args.get("priority").and_then(Value::as_str).unwrap_or("medium");
            let priority = Priority::parse_str(priority_str).unwrap_or_default();
            let assignee = args.get("assignee").and_then(Value::as_str).map(ToString::to_string);
            let blocked_by: Vec<String> = args
                .get("blocked_by")
                .and_then(Value::as_array)
                .map(|arr| arr.iter().filter_map(Value::as_str).map(ToString::to_string).collect())
                .unwrap_or_default();

            let task = Task {
                id: task_id.clone(),
                project_id,
                feature_id,
                title,
                description,
                status: TaskStatus::Todo,
                priority,
                assignee,
                blocked_by,
                result_summary: None,
                created_at: now.clone(),
                updated_at: now,
            };

            store.write(|s| {
                s.upsert_task(task);
                Ok(())
            }).await?;

            text_response(format!("Task '{}' created successfully.", task_id))
        }

        "pm_update_task_status" => {
            let task_id = get_str(args, "task_id")?;
            let status_str = get_str(args, "status")?;
            let status = match TaskStatus::parse_str(&status_str) {
                Some(st) => st,
                None => bail!("Invalid task status '{}'", status_str),
            };
            let result_summary = args.get("result_summary").and_then(Value::as_str).map(ToString::to_string);

            store.write(|s| {
                let mut task = match s.get_task(&task_id).cloned() {
                    Some(t) => t,
                    None => bail!("Task '{}' not found", task_id),
                };
                task.status = status;
                if let Some(summary) = result_summary {
                    task.result_summary = Some(summary);
                }
                task.updated_at = now;
                s.upsert_task(task);
                Ok(())
            }).await?;

            text_response(format!("Task '{}' status updated to '{}'.", task_id, status))
        }

        "pm_list_tasks" => {
            let project_id = get_str(args, "project_id")?;
            let feature_id = args.get("feature_id").and_then(Value::as_str);
            let status = args.get("status").and_then(Value::as_str).and_then(TaskStatus::parse_str);
            let assignee = args.get("assignee").and_then(Value::as_str);

            let list = store.read(|s| s.list_tasks(&project_id, feature_id, status, assignee).into_iter().cloned().collect::<Vec<_>>()).await;
            text_response(serde_json::to_string_pretty(&list)?)
        }

        "pm_get_next_actionable_task" => {
            let project_id = get_str(args, "project_id")?;
            let assignee = args.get("assignee").and_then(Value::as_str);

            let next_task = store.read(|s| get_next_actionable_task(s, &project_id, assignee)).await;
            match next_task {
                Some(task) => text_response(serde_json::to_string_pretty(&task)?),
                None => text_response("No actionable tasks available at this time (either all tasks are complete, or pending tasks are blocked by incomplete dependencies)."),
            }
        }

        "pm_get_task_context" => {
            let task_id = get_str(args, "task_id")?;
            let context_md = store.read(|s| get_task_context(s, &task_id)).await?;
            text_response(context_md)
        }

        // Local Gate & Merge
        "pm_verify_gate" => {
            let task_id = get_str(args, "task_id")?;
            let (default_cmd, root_path) = store.read(|s| {
                let t = s.get_task(&task_id);
                let p = t.and_then(|task| s.get_project(&task.project_id));
                (
                    p.and_then(|proj| proj.default_gate_cmd.clone()),
                    p.map(|proj| proj.root_path.clone()).unwrap_or_else(|| ".".to_string()),
                )
            }).await;

            let cmd = args.get("command")
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .or(default_cmd)
                .ok_or_else(|| anyhow::anyhow!("No gate command provided and no project default_gate_cmd configured for task '{}'", task_id))?;

            let work_dir = std::path::PathBuf::from(root_path);
            let run = execute_gate(&task_id, &cmd, &work_dir).await?;
            let passed = run.passed;
            let run_clone = run.clone();

            store.write(|s| {
                s.record_gate_run(run_clone);
                Ok(())
            }).await?;

            let status_str = if passed { "PASSED" } else { "FAILED" };
            text_response(format!(
                "Gate run '{}' {} in {}ms (exit code: {}).\nCommand: {}\nStdout:\n{}\nStderr:\n{}",
                run.id, status_str, run.duration_ms, run.exit_code, run.command, run.stdout.trim(), run.stderr.trim()
            ))
        }

        "pm_get_gate_history" => {
            let task_id = get_str(args, "task_id")?;
            let runs = store.read(|s| s.get_gate_runs(&task_id).into_iter().cloned().collect::<Vec<_>>()).await;
            text_response(serde_json::to_string_pretty(&runs)?)
        }

        "pm_merge_change" => {
            let task_id = get_str(args, "task_id")?;
            let target_branch = args.get("target_branch").and_then(Value::as_str).unwrap_or("main");
            let strategy = args.get("strategy").and_then(Value::as_str).unwrap_or("squash");

            let root_path = store.read(|s| {
                let t = s.get_task(&task_id);
                t.and_then(|task| s.get_project(&task.project_id)).map(|p| p.root_path.clone()).unwrap_or_else(|| ".".to_string())
            }).await;
            let work_dir = std::path::PathBuf::from(root_path);

            let commit = verify_and_merge_task(
                store,
                &task_id,
                target_branch,
                strategy,
                &work_dir,
            ).await?;

            text_response(format!(
                "Task '{}' successfully merged into '{}' (strategy: {}, commit: {}). Task status set to 'done'.",
                task_id, target_branch, strategy, commit
            ))
        }

        // Defect Tracking
        "pm_report_defect" => {
            let project_id = get_str(args, "project_id")?;
            let title = get_str(args, "title")?;
            let description = get_str(args, "description")?;
            let severity_str = args.get("severity").and_then(Value::as_str).unwrap_or("major");
            let severity = DefectSeverity::parse_str(severity_str).unwrap_or(DefectSeverity::Major);
            let feature_id = args.get("feature_id").and_then(Value::as_str).map(ToString::to_string);
            let task_id = args.get("task_id").and_then(Value::as_str).map(ToString::to_string);
            let reproduction_steps = args.get("reproduction_steps").and_then(Value::as_str).map(ToString::to_string);
            let error_log = args.get("error_log").and_then(Value::as_str).map(ToString::to_string);
            let auto_create_task = args.get("auto_create_task").and_then(Value::as_bool).unwrap_or(true);

            let defect_id = format!("def_{}", Utc::now().timestamp_micros());
            let mut created_task_id = None;

            store.write(|s| {
                if auto_create_task {
                    let tid = format!("task_fix_{}", Utc::now().timestamp_micros());
                    let prio = match severity {
                        DefectSeverity::Critical => Priority::Critical,
                        DefectSeverity::Major => Priority::High,
                        DefectSeverity::Minor => Priority::Medium,
                        DefectSeverity::Cosmetic => Priority::Low,
                    };
                    let task_desc = format!(
                        "Fix Defect: {}\n\n## Description\n{}\n\n## Reproduction Steps\n{}\n\n## Error Log\n{}",
                        title,
                        description,
                        reproduction_steps.as_deref().unwrap_or("N/A"),
                        error_log.as_deref().unwrap_or("N/A")
                    );
                    let t = Task {
                        id: tid.clone(),
                        project_id: project_id.clone(),
                        feature_id: feature_id.clone(),
                        title: format!("Fix: {}", title),
                        description: task_desc,
                        status: TaskStatus::Todo,
                        priority: prio,
                        assignee: None,
                        blocked_by: vec![],
                        result_summary: None,
                        created_at: now.clone(),
                        updated_at: now.clone(),
                    };
                    s.upsert_task(t);
                    created_task_id = Some(tid);
                }

                let defect = Defect {
                    id: defect_id.clone(),
                    project_id,
                    feature_id,
                    task_id,
                    title,
                    description,
                    severity,
                    status: DefectStatus::Open,
                    reproduction_steps,
                    error_log,
                    created_task_id: created_task_id.clone(),
                    created_at: now.clone(),
                    updated_at: now,
                };
                s.upsert_defect(defect);
                Ok(())
            }).await?;

            let mut msg = format!("Defect '{}' reported successfully.", defect_id);
            if let Some(ref tid) = created_task_id {
                msg.push_str(&format!(" Actionable fix task '{}' created automatically.", tid));
            }
            text_response(msg)
        }

        "pm_resolve_defect" => {
            let defect_id = get_str(args, "defect_id")?;
            let status_str = args.get("status").and_then(Value::as_str).unwrap_or("resolved");
            let status = match status_str {
                "wont_fix" | "wontfix" => DefectStatus::WontFix,
                _ => DefectStatus::Resolved,
            };
            let resolution_notes = args.get("resolution_notes").and_then(Value::as_str);

            store.write(|s| {
                let mut defect = match s.get_defect(&defect_id).cloned() {
                    Some(d) => d,
                    None => bail!("Defect '{}' not found", defect_id),
                };
                defect.status = status;
                if let Some(notes) = resolution_notes {
                    let updated_desc = format!("{}\n\n### Resolution Notes\n{}", defect.description, notes);
                    defect.description = updated_desc;
                }
                defect.updated_at = now;
                s.upsert_defect(defect);
                Ok(())
            }).await?;

            text_response(format!("Defect '{}' marked as '{}'.", defect_id, status))
        }

        "pm_list_defects" => {
            let project_id = get_str(args, "project_id")?;
            let status = args.get("status").and_then(Value::as_str).and_then(DefectStatus::parse_str);
            let severity = args.get("severity").and_then(Value::as_str).and_then(DefectSeverity::parse_str);

            let list = store.read(|s| s.list_defects(&project_id, status, severity).into_iter().cloned().collect::<Vec<_>>()).await;
            text_response(serde_json::to_string_pretty(&list)?)
        }

        // Code Review System
        "pm_add_review_comment" => {
            let task_id = get_str(args, "task_id")?;
            let reviewer = get_str(args, "reviewer")?;
            let content = get_str(args, "content")?;
            let severity_str = args.get("severity").and_then(Value::as_str).unwrap_or("blocking");
            let severity = ReviewSeverity::parse_str(severity_str).unwrap_or(ReviewSeverity::Blocking);
            let file_path = args.get("file_path").and_then(Value::as_str).map(ToString::to_string);
            let line_number = args.get("line_number").and_then(Value::as_u64).map(|n| n as usize);

            let comment_id = format!("rev_{}", Utc::now().timestamp_micros());
            let comment = ReviewComment {
                id: comment_id.clone(),
                task_id,
                reviewer,
                file_path,
                line_number,
                severity,
                content,
                resolved: false,
                resolution_note: None,
                created_at: now.clone(),
                updated_at: now,
            };

            store.write(|s| {
                s.upsert_review_comment(comment);
                Ok(())
            }).await?;

            text_response(format!("Review comment '{}' added successfully.", comment_id))
        }

        "pm_resolve_review_comment" => {
            let comment_id = get_str(args, "comment_id")?;
            let resolution_note = args.get("resolution_note").and_then(Value::as_str).map(ToString::to_string);

            store.write(|s| {
                let mut comment = match s.get_review_comment(&comment_id).cloned() {
                    Some(c) => c,
                    None => bail!("Review comment '{}' not found", comment_id),
                };
                comment.resolved = true;
                comment.resolution_note = resolution_note;
                comment.updated_at = now;
                s.upsert_review_comment(comment);
                Ok(())
            }).await?;

            text_response(format!("Review comment '{}' resolved.", comment_id))
        }

        "pm_list_review_comments" => {
            let task_id = get_str(args, "task_id")?;
            let unresolved_only = args.get("unresolved_only").and_then(Value::as_bool).unwrap_or(false);

            let list = store.read(|s| s.list_review_comments(&task_id, unresolved_only).into_iter().cloned().collect::<Vec<_>>()).await;
            text_response(serde_json::to_string_pretty(&list)?)
        }

        unknown => bail!("Unknown tool '{}'", unknown),
    }
}

fn get_str(val: &Value, field: &str) -> Result<String> {
    val.get(field)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .with_context(|| format!("Missing required string argument '{}'", field))
}

fn text_response(text: impl Into<String>) -> Result<Value> {
    Ok(json!([
        {
            "type": "text",
            "text": text.into(),
        }
    ]))
}
