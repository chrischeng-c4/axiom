use std::collections::HashSet;
use anyhow::{bail, Result};

use crate::models::{Task, TaskStatus};
use crate::store::state::PmState;

/// Finds the highest-priority `todo` task whose dependencies in `blocked_by`
/// are all in `done` state.
pub fn get_next_actionable_task(
    state: &PmState,
    project_id: &str,
    assignee: Option<&str>,
) -> Option<Task> {
    // Set of all completed task IDs in this project
    let done_task_ids: HashSet<&str> = state
        .tasks
        .values()
        .filter(|t| t.project_id == project_id && t.status == TaskStatus::Done)
        .map(|t| t.id.as_str())
        .collect();

    let mut eligible: Vec<&Task> = state
        .tasks
        .values()
        .filter(|t| {
            if t.project_id != project_id || t.status != TaskStatus::Todo {
                return false;
            }
            if let Some(target_assignee) = assignee {
                if t.assignee.as_deref() != Some(target_assignee) {
                    return false;
                }
            }
            // Check if all blocked_by tasks are done
            t.blocked_by
                .iter()
                .all(|dep_id| done_task_ids.contains(dep_id.as_str()))
        })
        .collect();

    // Sort by priority descending (Critical > High > Medium > Low), then created_at ascending
    eligible.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| a.created_at.cmp(&b.created_at))
    });

    eligible.into_iter().next().cloned()
}

/// Builds a rich, markdown-formatted execution context block for an AI Agent.
pub fn get_task_context(state: &PmState, task_id: &str) -> Result<String> {
    let task = match state.get_task(task_id) {
        Some(t) => t,
        None => bail!("Task '{}' not found", task_id),
    };

    let mut out = String::new();
    out.push_str(&format!("# Task: {} ({})\n\n", task.title, task.id));
    out.push_str(&format!("- **Project**: `{}`\n", task.project_id));
    out.push_str(&format!("- **Status**: `{}`\n", task.status));
    out.push_str(&format!("- **Priority**: `{}`\n", task.priority));
    if let Some(ref assignee) = task.assignee {
        out.push_str(&format!("- **Assignee**: `{}`\n", assignee));
    }
    out.push_str("\n## Task Description & Instructions\n\n");
    out.push_str(&task.description);
    out.push_str("\n\n");

    // Parent Feature
    if let Some(ref feat_id) = task.feature_id {
        if let Some(feature) = state.get_feature(feat_id) {
            out.push_str(&format!("## Parent Feature: {} (`{}`)\n\n", feature.title, feature.id));
            out.push_str(&format!("- **Status**: `{}` | **Priority**: `{}`\n\n", feature.status, feature.priority));
            out.push_str(&feature.description);
            out.push_str("\n\n");

            // Linked PRD
            if let Some(ref prd_id) = feature.prd_id {
                if let Some(prd) = state.get_prd(prd_id) {
                    out.push_str(&format!("## Product Requirements Document: {} (`{}` v{})\n\n", prd.title, prd.id, prd.version));
                    out.push_str(&format!("- **PRD Status**: `{}`\n\n", prd.status));
                    out.push_str(&prd.content);
                    out.push_str("\n\n");
                }
            }

            // Linked Tech Design
            if let Some(ref td_id) = feature.tech_design_id {
                if let Some(td) = state.get_tech_design(td_id) {
                    out.push_str(&format!("## Technical Design: {} (`{}` v{})\n\n", td.title, td.id, td.version));
                    out.push_str(&format!("- **Design Status**: `{}`\n\n", td.status));
                    out.push_str(&td.content);
                    out.push_str("\n\n");
                }
            }
        }
    }

    // Dependencies
    if !task.blocked_by.is_empty() {
        out.push_str("## Prerequisite Dependencies (`blocked_by`)\n\n");
        for dep_id in &task.blocked_by {
            if let Some(dep_task) = state.get_task(dep_id) {
                let check = if dep_task.status == TaskStatus::Done { "[x]" } else { "[ ]" };
                out.push_str(&format!("- {} `{}` - {} (Status: `{}`)\n", check, dep_task.id, dep_task.title, dep_task.status));
            } else {
                out.push_str(&format!("- [ ] `{}` (Unknown task ID)\n", dep_id));
            }
        }
        out.push_str("\n");
    }

    // Previous execution notes
    if let Some(ref summary) = task.result_summary {
        out.push_str("## Prior Result Summary\n\n");
        out.push_str(summary);
        out.push_str("\n\n");
    }

    Ok(out)
}
