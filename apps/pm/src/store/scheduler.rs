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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        Feature, FeatureStatus, Prd, PrdStatus, Priority, Project, Task, TaskStatus,
        TechDesign, TechDesignStatus,
    };

    #[test]
    fn test_scheduler_dependency_resolution() {
        let mut state = PmState::default();
        let proj_id = "proj_demo";

        let t1 = Task {
            id: "task_1".to_string(),
            project_id: proj_id.to_string(),
            feature_id: None,
            title: "Setup Database".to_string(),
            description: "Initialize DB".to_string(),
            status: TaskStatus::Todo,
            priority: Priority::Medium,
            assignee: Some("dev".to_string()),
            blocked_by: vec![],
            result_summary: None,
            e2e_red_commit: None,
            impl_red_commit: None,
            created_at: "2026-09-06T01:00:00Z".to_string(),
            updated_at: "2026-09-06T01:00:00Z".to_string(),
        };

        let t2 = Task {
            id: "task_2".to_string(),
            project_id: proj_id.to_string(),
            feature_id: None,
            title: "Implement API".to_string(),
            description: "Build API routes".to_string(),
            status: TaskStatus::Todo,
            priority: Priority::High,
            assignee: Some("dev".to_string()),
            blocked_by: vec!["task_1".to_string()],
            result_summary: None,
            e2e_red_commit: None,
            impl_red_commit: None,
            created_at: "2026-09-06T02:00:00Z".to_string(),
            updated_at: "2026-09-06T02:00:00Z".to_string(),
        };

        let t3 = Task {
            id: "task_3".to_string(),
            project_id: proj_id.to_string(),
            feature_id: None,
            title: "Deploy Service".to_string(),
            description: "Deploy to cloud".to_string(),
            status: TaskStatus::Todo,
            priority: Priority::Critical,
            assignee: Some("dev".to_string()),
            blocked_by: vec!["task_2".to_string()],
            result_summary: None,
            e2e_red_commit: None,
            impl_red_commit: None,
            created_at: "2026-09-06T03:00:00Z".to_string(),
            updated_at: "2026-09-06T03:00:00Z".to_string(),
        };

        state.upsert_task(t1);
        state.upsert_task(t2);
        state.upsert_task(t3);

        let next1 = get_next_actionable_task(&state, proj_id, None);
        assert!(next1.is_some());
        assert_eq!(next1.unwrap().id, "task_1");

        state.tasks.get_mut("task_1").unwrap().status = TaskStatus::InProgress;
        let next2 = get_next_actionable_task(&state, proj_id, None);
        assert!(next2.is_none());

        state.tasks.get_mut("task_1").unwrap().status = TaskStatus::Done;
        let next3 = get_next_actionable_task(&state, proj_id, None);
        assert!(next3.is_some());
        assert_eq!(next3.unwrap().id, "task_2");

        state.tasks.get_mut("task_2").unwrap().status = TaskStatus::Done;
        let next4 = get_next_actionable_task(&state, proj_id, None);
        assert!(next4.is_some());
        assert_eq!(next4.unwrap().id, "task_3");

        state.tasks.get_mut("task_3").unwrap().status = TaskStatus::Done;
        let next5 = get_next_actionable_task(&state, proj_id, None);
        assert!(next5.is_none());
    }

    #[test]
    fn test_scheduler_unblock_dependent_tasks() {
        let mut state = PmState::default();
        let proj_id = "proj_unblock";

        let t1 = Task {
            id: "t_a".to_string(),
            project_id: proj_id.to_string(),
            feature_id: None,
            title: "Task A".to_string(),
            description: "A".to_string(),
            status: TaskStatus::Todo,
            priority: Priority::Medium,
            assignee: None,
            blocked_by: vec![],
            result_summary: None,
            e2e_red_commit: None,
            impl_red_commit: None,
            created_at: "2026-09-06T01:00:00Z".to_string(),
            updated_at: "2026-09-06T01:00:00Z".to_string(),
        };

        let t2 = Task {
            id: "t_b".to_string(),
            project_id: proj_id.to_string(),
            feature_id: None,
            title: "Task B".to_string(),
            description: "B".to_string(),
            status: TaskStatus::Blocked,
            priority: Priority::High,
            assignee: None,
            blocked_by: vec!["t_a".to_string()],
            result_summary: None,
            e2e_red_commit: None,
            impl_red_commit: None,
            created_at: "2026-09-06T02:00:00Z".to_string(),
            updated_at: "2026-09-06T02:00:00Z".to_string(),
        };

        state.upsert_task(t1);
        state.upsert_task(t2);

        // Before completing A, B is blocked and cannot be scheduled
        assert_eq!(state.get_task("t_b").unwrap().status, TaskStatus::Blocked);
        assert_eq!(get_next_actionable_task(&state, proj_id, None).unwrap().id, "t_a");

        // Complete A and run unblock
        state.tasks.get_mut("t_a").unwrap().status = TaskStatus::Done;
        let unblocked = state.unblock_dependent_tasks("t_a");
        assert_eq!(unblocked, vec!["t_b".to_string()]);
        assert_eq!(state.get_task("t_b").unwrap().status, TaskStatus::Todo);

        // Now B is actionable
        assert_eq!(get_next_actionable_task(&state, proj_id, None).unwrap().id, "t_b");
    }

    #[test]
    fn test_task_context_aggregation() {
        let mut state = PmState::default();
        let proj_id = "proj_axiom";

        state.upsert_project(Project {
            id: proj_id.to_string(),
            name: "Axiom".to_string(),
            description: "Monorepo".to_string(),
            root_path: ".".to_string(),
            default_gate_cmd: None,
            created_at: "2026-09-06T00:00:00Z".to_string(),
            updated_at: "2026-09-06T00:00:00Z".to_string(),
        });

        state.upsert_prd(Prd {
            id: "prd_auth".to_string(),
            project_id: proj_id.to_string(),
            title: "User Authentication PRD".to_string(),
            version: "1.0".to_string(),
            status: PrdStatus::Approved,
            content: "## Requirements\nMust support JWT and Argon2 password hashing.".to_string(),
            created_at: "2026-09-06T00:00:00Z".to_string(),
            updated_at: "2026-09-06T00:00:00Z".to_string(),
        });

        state.upsert_tech_design(TechDesign {
            id: "td_auth".to_string(),
            project_id: proj_id.to_string(),
            prd_id: Some("prd_auth".to_string()),
            title: "Authentication Tech Design".to_string(),
            version: "1.0".to_string(),
            status: TechDesignStatus::Approved,
            content: "## Architecture\nStateless JWT verification using ed25519 keys.".to_string(),
            created_at: "2026-09-06T00:00:00Z".to_string(),
            updated_at: "2026-09-06T00:00:00Z".to_string(),
        });

        state.upsert_feature(Feature {
            id: "feat_jwt".to_string(),
            project_id: proj_id.to_string(),
            prd_id: Some("prd_auth".to_string()),
            tech_design_id: Some("td_auth".to_string()),
            title: "JWT Token Engine".to_string(),
            description: "Implement JWT creation and verification modules.".to_string(),
            status: FeatureStatus::InProgress,
            priority: Priority::High,
            created_at: "2026-09-06T00:00:00Z".to_string(),
            updated_at: "2026-09-06T00:00:00Z".to_string(),
        });

        state.upsert_task(Task {
            id: "task_jwt_sign".to_string(),
            project_id: proj_id.to_string(),
            feature_id: Some("feat_jwt".to_string()),
            title: "Implement Token Signing".to_string(),
            description: "Write ed25519 token signing function with 1hr expiry.".to_string(),
            status: TaskStatus::Todo,
            priority: Priority::High,
            assignee: Some("dev".to_string()),
            blocked_by: vec![],
            result_summary: None,
            e2e_red_commit: None,
            impl_red_commit: None,
            created_at: "2026-09-06T00:00:00Z".to_string(),
            updated_at: "2026-09-06T00:00:00Z".to_string(),
        });

        let context = get_task_context(&state, "task_jwt_sign").expect("Failed to get context");

        assert!(context.contains("# Task: Implement Token Signing"));
        assert!(context.contains("Write ed25519 token signing function"));
        assert!(context.contains("Parent Feature: JWT Token Engine"));
        assert!(context.contains("Product Requirements Document: User Authentication PRD"));
        assert!(context.contains("Must support JWT and Argon2"));
        assert!(context.contains("Technical Design: Authentication Tech Design"));
        assert!(context.contains("Stateless JWT verification"));
    }
}

