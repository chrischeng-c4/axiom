use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde_json::{json, Value};

use crate::models::{
    Feature, FeatureStatus, Prd, PrdStatus, Priority, Project, Task, TaskStatus,
    TechDesign, TechDesignStatus,
};
use crate::store::{get_next_actionable_task, get_task_context, PmStore};

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
                "root_path": { "type": "string", "description": "Root workspace path (default: current directory)" }
            }),
            &["id", "name", "description"],
        ),
        tool(
            "pm_get_project_summary",
            "Retrieve an executive dashboard summary of project metrics, active features, PRDs, and task distribution",
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
        // Agent Special Tools
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

            let proj = Project {
                id: id.clone(),
                name,
                description,
                root_path,
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
