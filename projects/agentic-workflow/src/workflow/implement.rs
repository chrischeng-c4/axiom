// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/workflow/implement.md#source
// CODEGEN-BEGIN
//! Implement flow: Per-task implementation with review/revise loop.
//!
//! Handles phases: ChangeImplementationCreated, ChangeImplementationReviewed,
//! ChangeImplementationRevised.
//!
//! Per-task workflow (R1-R7 from impl-workflow-refactor spec):
//! - Build TaskGraph from tasks.md with topo+lexical ordering (R1)
//! - Track current_task_id in STATE.yaml for resumption (R2)
//! - Independent review/revise loop per task (R3)
//! - 2-revision limit per task via task_revisions map (R4)
//! - Terminal failure on revision limit exceeded (R5)
//! - Legacy STATE.yaml backward compatibility (R6)
//! - Clean working directory check for in_place workflow (R7)

use super::helpers;
use crate::models::state::StatePhase;
use crate::state::StateManager;
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

/// Maximum revisions per task before terminal failure
const MAX_TASK_REVISIONS: u32 = 2;

/// Handle the implement flow.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/implement.md#source
pub fn handle(change_dir: &Path, change_id: &str, project_path: &str) -> Result<Value> {
    let sm = StateManager::load(change_dir)?;
    let phase = sm.phase().clone();
    let state = sm.state();

    let has_tasks = change_dir.join("tasks.md").exists();

    // Build per-task context if tasks.md exists
    let task_ctx = if has_tasks {
        build_task_context(
            change_dir,
            state.current_task_id.as_deref(),
            &state.task_revisions,
        )
    } else {
        None
    };

    let action = determine_action(&phase, &sm, &task_ctx, change_dir);

    let phase_str = helpers::phase_to_string(&phase);
    let mut base = json!({
        "change_id": change_id,
        "current_phase": phase_str,
        "has_tasks": has_tasks,
    });

    // Include per-task info when available
    if let Some(ref ctx) = task_ctx {
        base["current_task_id"] = json!(ctx.current_task_id.as_deref());
        base["total_tasks"] = json!(ctx.total_tasks);
        base["completed_tasks"] = json!(ctx.completed_count);
        base["pending_tasks"] = json!(ctx.total_tasks - ctx.completed_count);
    }

    build_response(
        &action,
        &mut base,
        change_id,
        project_path,
        &task_ctx,
        change_dir,
    );

    Ok(base)
}

// ---------------------------------------------------------------------------
// Per-task context
// ---------------------------------------------------------------------------

struct TaskContext {
    current_task_id: Option<String>,
    current_task_spec_ref: Option<String>,
    current_task_revisions: u32,
    total_tasks: usize,
    completed_count: usize,
    all_done: bool,
}

fn build_task_context(
    change_dir: &Path,
    state_current_task_id: Option<&str>,
    task_revisions: &std::collections::HashMap<String, u32>,
) -> Option<TaskContext> {
    let tasks_path = change_dir.join("tasks.md");
    let content = std::fs::read_to_string(&tasks_path).ok()?;
    let tasks = helpers::parse_task_blocks(&content);
    if tasks.is_empty() {
        return None;
    }

    let execution_order = helpers::build_task_execution_order(&tasks);
    let completed = helpers::find_completed_tasks(change_dir, &tasks);
    let next = helpers::find_next_task(&tasks, &execution_order, state_current_task_id, &completed);

    let revisions = next
        .as_ref()
        .and_then(|id| task_revisions.get(id))
        .copied()
        .unwrap_or(0);

    let spec_ref = next
        .as_ref()
        .and_then(|id| tasks.iter().find(|t| &t.id == id))
        .and_then(|t| t.spec_ref.clone());

    Some(TaskContext {
        current_task_id: next,
        current_task_spec_ref: spec_ref,
        current_task_revisions: revisions,
        total_tasks: tasks.len(),
        completed_count: completed.len(),
        all_done: completed.len() >= tasks.len(),
    })
}

// ---------------------------------------------------------------------------
// Action determination
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum Action {
    BeginImplementation,
    ImplementTask {
        task_id: String,
    },
    /// Structured codegen via Prism: task has a SpecIR-eligible spec_ref
    ImplementTaskWithCodegen {
        task_id: String,
        spec_ref: String,
    },
    ReviewTask {
        task_id: String,
    },
    ReviseTask {
        task_id: String,
        issues: Vec<String>,
    },
    TaskTerminalFailure {
        task_id: String,
        revisions: u32,
    },
    AllTasksDone,
    /// Legacy: no tasks.md or unrecognized phase
    LegacyReview,
    LegacyResolve {
        issues: Vec<String>,
    },
    LegacyComplete,
}

fn determine_action(
    phase: &StatePhase,
    sm: &StateManager,
    task_ctx: &Option<TaskContext>,
    change_dir: &Path,
) -> Action {
    // If no task context, fall back to legacy behavior
    let ctx = match task_ctx {
        Some(ctx) => ctx,
        None => return determine_legacy_action(phase, sm, change_dir),
    };

    match phase {
        StatePhase::ChangeImplementationCreated => {
            if ctx.all_done {
                Action::AllTasksDone
            } else if let Some(ref task_id) = ctx.current_task_id {
                if ctx.current_task_revisions > MAX_TASK_REVISIONS {
                    Action::TaskTerminalFailure {
                        task_id: task_id.clone(),
                        revisions: ctx.current_task_revisions,
                    }
                } else {
                    // Check if task has been implemented (impl file exists) → review
                    let impl_path = change_dir.join(format!("impl_{}.md", task_id));
                    if impl_path.exists() {
                        Action::ReviewTask {
                            task_id: task_id.clone(),
                        }
                    } else if let Some(ref spec_ref) = ctx.current_task_spec_ref {
                        if helpers::is_codegen_eligible(change_dir, spec_ref) {
                            Action::ImplementTaskWithCodegen {
                                task_id: task_id.clone(),
                                spec_ref: spec_ref.clone(),
                            }
                        } else {
                            Action::ImplementTask {
                                task_id: task_id.clone(),
                            }
                        }
                    } else {
                        Action::ImplementTask {
                            task_id: task_id.clone(),
                        }
                    }
                }
            } else {
                Action::AllTasksDone
            }
        }
        StatePhase::ChangeImplementationReviewed => {
            if let Some(ref task_id) = ctx.current_task_id {
                // Check task-scoped review verdict (inline in impl_{task_id}.md)
                let review_path = change_dir.join(format!("impl_{}.md", task_id));
                let (verdict, issues) = if review_path.exists() {
                    helpers::extract_review_info(&review_path)
                } else {
                    // Fall back to global impl.md or legacy review_impl.md
                    let global_impl = change_dir.join("impl.md");
                    let legacy_review = change_dir.join("review_impl.md");
                    if global_impl.exists() {
                        helpers::extract_review_info(&global_impl)
                    } else if legacy_review.exists() {
                        helpers::extract_review_info(&legacy_review)
                    } else {
                        (None, vec![])
                    }
                };

                match verdict.as_deref() {
                    Some("PASS") | Some("APPROVED") => {
                        // Task passed — move to next task
                        Action::AllTasksDone // Will be re-evaluated as ImplementTask on next call
                    }
                    _ => {
                        // Task needs revision — check limit
                        if ctx.current_task_revisions >= MAX_TASK_REVISIONS {
                            Action::TaskTerminalFailure {
                                task_id: task_id.clone(),
                                revisions: ctx.current_task_revisions,
                            }
                        } else {
                            Action::ReviseTask {
                                task_id: task_id.clone(),
                                issues,
                            }
                        }
                    }
                }
            } else {
                // No current task — legacy fallback
                let revision_count = sm.revision_count("implementation");
                if revision_count >= 1 {
                    Action::LegacyComplete
                } else {
                    let (_, issues) =
                        helpers::extract_review_info(&change_dir.join("review_impl.md"));
                    Action::LegacyResolve { issues }
                }
            }
        }
        StatePhase::ChangeImplementationRevised => {
            if let Some(ref task_id) = ctx.current_task_id {
                Action::ReviewTask {
                    task_id: task_id.clone(),
                }
            } else {
                Action::LegacyReview
            }
        }
        _ => Action::BeginImplementation,
    }
}

fn determine_legacy_action(phase: &StatePhase, sm: &StateManager, change_dir: &Path) -> Action {
    match phase {
        StatePhase::ChangeImplementationCreated => Action::LegacyReview,
        StatePhase::ChangeImplementationReviewed => {
            let revision_count = sm.revision_count("implementation");
            if revision_count >= 1 {
                Action::LegacyComplete
            } else {
                let (_, issues) = helpers::extract_review_info(&change_dir.join("review_impl.md"));
                Action::LegacyResolve { issues }
            }
        }
        StatePhase::ChangeImplementationRevised => Action::LegacyReview,
        _ => Action::BeginImplementation,
    }
}

// ---------------------------------------------------------------------------
// Response building
// ---------------------------------------------------------------------------

fn build_response(
    action: &Action,
    base: &mut Value,
    change_id: &str,
    project_path: &str,
    task_ctx: &Option<TaskContext>,
    change_dir: &Path,
) {
    let _pp = project_path;
    let cid = change_id;

    match action {
        Action::BeginImplementation => {
            base["action"] = json!("begin_implementation");
            base["message"] = json!("Start implementation for this change.");
            base["next_phase"] = json!("change_implementation_created");
            base["files_to_read"] = json!(["tasks.md", "specs/*.md"]);

            let task_section = if let Some(ctx) = task_ctx {
                if let Some(ref tid) = ctx.current_task_id {
                    format!(
                        "\n## Per-Task Workflow\n\n\
                         Starting with task **{}** ({} of {} tasks).\n\
                         After implementing, set phase to 'change_implementation_created' and call `score run-change` for review.\n\
                         Save current_task_id in state.\n",
                        tid, ctx.completed_count + 1, ctx.total_tasks
                    )
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            base["prompt"] = json!(format!(
                "# Task: Begin Implementation for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read all requirements via `score workflow read-artifact`\n\
                 2. Update STATE.yaml phase to 'change_implementation_created'\n\
                 3. Implement code according to tasks and specs\n\
                 4. Run tests to verify\n\
                 5. Update STATE.yaml phase to 'change_implementation_created' when done\n\
                 {task_section}\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"requirements\"}}'\n\
                 score run-change --change-id {cid}\n\
                 ```",
            ));
        }

        Action::ImplementTask { task_id } => {
            base["action"] = json!("implement_task");
            base["message"] = json!(format!("Implement task {}.", task_id));
            base["next_phase"] = json!("change_implementation_created");
            base["task_id"] = json!(task_id);

            let revision_info = if let Some(ctx) = task_ctx {
                if ctx.current_task_revisions > 0 {
                    format!(
                        "\n**Revision attempt {} of {}** — fix issues from previous review.\n\
                         Read `impl_{}.md` for the list of issues to address.\n",
                        ctx.current_task_revisions, MAX_TASK_REVISIONS, task_id
                    )
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let progress = if let Some(ctx) = task_ctx {
                format!("Task {} of {}", ctx.completed_count + 1, ctx.total_tasks)
            } else {
                String::new()
            };

            base["prompt"] = json!(format!(
                "# Task: Implement Task '{task_id}' for Change '{cid}'\n\n\
                 Progress: {progress}\n\
                 {revision_info}\n\
                 ## Instructions\n\n\
                 1. Read task details via `score workflow read-artifact`\n\
                 2. Read relevant specs via `score workflow read-artifact`\n\
                 3. Implement ONLY task {task_id}\n\
                 4. Run tests to verify\n\
                 5. Update STATE.yaml phase to 'change_implementation_created' when task {task_id} is done\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"tasks\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"requirements\"}}'\n\
                 score run-change --change-id {cid}\n\
                 ```",
            ));
        }

        Action::ImplementTaskWithCodegen { task_id, spec_ref } => {
            let spec_name = spec_ref.split(':').next().unwrap_or(spec_ref);
            base["action"] = json!("implement_task_with_codegen");
            base["message"] = json!(format!(
                "Implement task {} using structured codegen from spec '{}'.",
                task_id, spec_name
            ));
            base["next_phase"] = json!("change_implementation_created");
            base["task_id"] = json!(task_id);
            base["spec_ref"] = json!(spec_ref);
            base["codegen"] = json!(true);

            let revision_info = if let Some(ctx) = task_ctx {
                if ctx.current_task_revisions > 0 {
                    format!(
                        "\n**Revision attempt {} of {}** — fix issues from previous review.\n\
                         Read `impl_{}.md` for the list of issues to address.\n",
                        ctx.current_task_revisions, MAX_TASK_REVISIONS, task_id
                    )
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let progress = if let Some(ctx) = task_ctx {
                format!("Task {} of {}", ctx.completed_count + 1, ctx.total_tasks)
            } else {
                String::new()
            };

            base["prompt"] = json!(format!(
                "# Task: Implement Task '{task_id}' with Structured Codegen for Change '{cid}'\n\n\
                 Progress: {progress}\n\
                 {revision_info}\n\
                 ## Structured Codegen Path\n\n\
                 This task has a SpecIR-eligible spec (`{spec_name}`). Use Prism's code generation \
                 pipeline as the primary implementation path:\n\n\
                 1. Read the spec via `score workflow read-artifact`\n\
                 2. Call `prism generate-from-spec` with the spec content to generate code\n\
                 3. Review generated output against spec requirements\n\
                 4. Apply any manual adjustments needed\n\
                 5. Run tests to verify\n\
                 6. Update STATE.yaml phase to 'change_implementation_created' when task {task_id} is done\n\n\
                 ## Fallback\n\n\
                 If `prism generate-from-spec` fails or produces insufficient output, \
                 fall back to manual implementation using the spec as reference.\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"{spec_name}\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"tasks\"}}'\n\
                 prism generate-from-spec <spec_json>\n\
                 score run-change --change-id {cid}\n\
                 ```",
            ));
        }

        Action::ReviewTask { task_id } => {
            base["action"] = json!("review_task");
            base["message"] = json!(format!("Review implementation of task {}.", task_id));
            base["next_phase"] = json!("change_implementation_reviewed");
            base["task_id"] = json!(task_id);
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Review Implementation of Task '{task_id}' for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read all requirements and task {task_id} specs\n\
                 2. List changed files via `score workflow list-changed-files`\n\
                 3. Review changes relevant to task {task_id} against specs\n\
                 4. Call `score artifact write-artifact` with task_id=\"{task_id}\" and findings\n\n\
                 ## Verdict Guidelines\n\
                 - APPROVED: Task code matches specs, tests pass\n\
                 - REVIEWED: Has fixable issues\n\
                 - REJECTED: Fundamental implementation problems\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"requirements\"}}'\n\
                 score workflow list-changed-files {cid}\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
            ));
        }

        Action::ReviseTask { task_id, issues } => {
            base["action"] = json!("revise_task");
            base["message"] = json!(format!("Fix issues for task {}.", task_id));
            base["next_phase"] = json!("change_implementation_revised");
            base["task_id"] = json!(task_id);
            base["review_file"] = json!(format!("impl_{}.md", task_id));
            base["issues"] = json!(issues);
            base["issues_count"] = json!(issues.len());
            base["prompt"] = json!(format!(
                "# Task: Revise Task '{task_id}' for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read impl_{task_id}.md for review issues (in # Reviews section)\n\
                 2. Fix all identified issues in the code\n\
                 3. Re-run tests to verify fixes\n\
                 4. Update STATE.yaml phase to 'change_implementation_revised'\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"review_impl:{task_id}\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"requirements\"}}'\n\
                 score run-change --change-id {cid}\n\
                 ```",
            ));
        }

        Action::TaskTerminalFailure { task_id, revisions } => {
            base["action"] = json!("task_terminal_failure");
            base["message"] = json!(format!(
                "Task {} failed after {} revisions. Manual intervention required.",
                task_id, revisions
            ));
            base["next_phase"] = json!("rejected");
            base["task_id"] = json!(task_id);
            base["prompt"] = json!(format!(
                "# TERMINAL FAILURE: Task '{task_id}' for Change '{cid}'\n\n\
                 Task {task_id} has failed review {revisions} times (limit: {max}).\n\n\
                 **The workflow has been halted.** Manual intervention is required.\n\n\
                 ## Options\n\n\
                 1. Fix the task manually and reset task_revisions for {task_id}\n\
                 2. Remove/simplify the task in tasks.md\n\
                 3. Reject the change entirely\n\n\
                 No further tasks will be executed.",
                max = MAX_TASK_REVISIONS
            ));
        }

        Action::AllTasksDone => {
            base["action"] = json!("all_tasks_done");
            base["message"] = json!("All tasks implemented and reviewed!");
            base["next_phase"] = json!("change_merge_created");
            base["prompt"] = json!(format!(
                "# All Tasks Complete for Change '{cid}'\n\n\
                 All implementation tasks have been completed and approved.\n\n\
                 Update STATE.yaml phase to 'change_implementation_approved' to transition to merge phase.\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score run-change --change-id {cid}\n\
                 ```",
            ));
        }

        // Legacy actions (no tasks.md or no per-task routing)
        Action::LegacyReview => {
            base["action"] = json!("review_implementation");
            base["message"] = json!("Create implementation review.");
            base["next_phase"] =
                json!("change_implementation_approved or change_implementation_reviewed");
            base["has_review_impl"] = json!(change_dir.join("review_impl.md").exists());
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Review Implementation for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read all requirements and specs\n\
                 2. List changed files via `score workflow list-changed-files`\n\
                 3. Review each changed file against specs\n\
                 4. Call `score artifact write-artifact` with findings\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"requirements\"}}'\n\
                 score workflow list-changed-files {cid}\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
            ));
        }

        Action::LegacyResolve { issues } => {
            base["action"] = json!("resolve_implementation");
            base["message"] = json!("Fix issues from code review.");
            base["next_phase"] = json!("change_implementation_revised");
            base["has_review_impl"] = json!(true);
            base["review_file"] = json!("review_impl.md");
            base["issues"] = json!(issues);
            base["issues_count"] = json!(issues.len());
            base["prompt"] = json!(format!(
                "# Task: Resolve Implementation Issues for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read review_impl.md for issues\n\
                 2. Fix all identified issues in the code\n\
                 3. Re-run tests to verify fixes\n\
                 4. Delete review_impl.md after fixing\n\
                 5. Update STATE.yaml phase to 'change_implementation_revised'\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"review_impl\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"requirements\"}}'\n\
                 score run-change --change-id {cid}\n\
                 ```",
            ));
        }

        Action::LegacyComplete => {
            base["action"] = json!("implementation_complete");
            base["message"] = json!("Implementation is complete!");
            base["next_phase"] = json!("change_merge_created");
            base["has_review_impl"] = json!(change_dir.join("review_impl.md").exists());
            base["prompt"] = json!(format!(
                "# Implementation Complete for Change '{cid}'\n\n\
                 Implementation has been approved:\n\
                 - All code written\n\
                 - Tests passing\n\
                 - Code review approved\n\n\
                 Run `score run-change` to continue the workflow.\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score run-change --change-id {cid}\n\
                 ```",
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_change_dir(phase_str: &str) -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        let state_content = format!(
            "change_id: test-change\nphase: {}\niteration: 1\n",
            phase_str
        );
        std::fs::write(change_dir.join("STATE.yaml"), state_content).unwrap();
        crate::test_util::write_minimal_issue(temp_dir.path(), "test-change");
        (temp_dir, change_dir)
    }

    fn write_tasks_md(change_dir: &Path, content: &str) {
        std::fs::write(change_dir.join("tasks.md"), content).unwrap();
    }

    fn tasks_with_deps() -> &'static str {
        r#"---
id: test
type: tasks
---
<tasks>

## Task 2.1

```yaml
id: 2.1
action: CREATE
status: pending
file: src/impl.rs
```

## Task 3.1

```yaml
id: 3.1
action: CREATE
status: pending
file: src/review.rs
depends_on: [2.1]
```

## Task 4.1

```yaml
id: 4.1
action: CREATE
status: pending
file: tests/test.rs
depends_on: [3.1]
```

</tasks>"#
    }

    // -----------------------------------------------------------------------
    // Legacy tests (backward compat without per-task routing)
    // -----------------------------------------------------------------------

    #[test]
    fn test_planned_begins_implementation() {
        let (_temp_dir, change_dir) = setup_change_dir("change_implementation_created");
        std::fs::write(change_dir.join("tasks.md"), "---\nid: test\n---\n").unwrap();
        let result = handle(&change_dir, "test-change", "/tmp").unwrap();
        // No parseable task blocks → legacy begin_implementation
        assert_eq!(result["action"], "begin_implementation");
    }

    #[test]
    fn test_change_implementation_created_reviews_legacy() {
        let (_temp_dir, change_dir) = setup_change_dir("change_implementation_created");
        std::fs::write(change_dir.join("tasks.md"), "---\nid: test\n---\n").unwrap();
        let result = handle(&change_dir, "test-change", "/tmp").unwrap();
        assert_eq!(result["action"], "review_implementation");
    }

    #[test]
    fn test_change_implementation_created_triggers_review_legacy() {
        let (_temp_dir, change_dir) = setup_change_dir("change_implementation_created");
        std::fs::write(change_dir.join("tasks.md"), "---\nid: test\n---\n").unwrap();
        let result = handle(&change_dir, "test-change", "/tmp").unwrap();
        assert_eq!(result["action"], "review_implementation");
    }

    #[test]
    fn test_change_impl_reviewed_resolves_legacy() {
        let (_temp_dir, change_dir) = setup_change_dir("change_implementation_reviewed");
        std::fs::write(change_dir.join("tasks.md"), "---\nid: test\n---\n").unwrap();
        std::fs::write(
            change_dir.join("review_impl.md"),
            "---\nverdict: REVIEWED\n---\n# Review\n## Issues\n- Fix bug in auth\n",
        )
        .unwrap();
        let result = handle(&change_dir, "test-change", "/tmp").unwrap();
        assert_eq!(result["action"], "resolve_implementation");
        assert_eq!(result["issues_count"], 1);
    }

    #[test]
    fn test_change_impl_revised_triggers_re_review_legacy() {
        let (_temp_dir, change_dir) = setup_change_dir("change_implementation_revised");
        std::fs::write(change_dir.join("tasks.md"), "---\nid: test\n---\n").unwrap();
        let result = handle(&change_dir, "test-change", "/tmp").unwrap();
        assert_eq!(result["action"], "review_implementation");
    }

    #[test]
    fn test_change_impl_reviewed_fallback_at_revision_limit_legacy() {
        let (_temp_dir, change_dir) = setup_change_dir("change_implementation_reviewed");
        std::fs::write(change_dir.join("tasks.md"), "---\nid: test\n---\n").unwrap();
        let mut sm = StateManager::load(&change_dir).unwrap();
        sm.increment_revision_count("implementation");
        sm.increment_revision_count("implementation");
        sm.save().unwrap();
        let result = handle(&change_dir, "test-change", "/tmp").unwrap();
        assert_eq!(result["action"], "implementation_complete");
    }

    // -----------------------------------------------------------------------
    // Per-task tests (S1-S7 from spec)
    // -----------------------------------------------------------------------

    #[test]
    fn test_planned_with_tasks_begins_first_task() {
        let (_temp_dir, change_dir) = setup_change_dir("change_implementation_created");
        write_tasks_md(&change_dir, tasks_with_deps());
        let result = handle(&change_dir, "test-change", "/tmp").unwrap();
        assert_eq!(result["action"], "begin_implementation");
        // First task in topo order should be 2.1
        assert_eq!(result["current_task_id"], "2.1");
        assert_eq!(result["total_tasks"], 3);
        assert_eq!(result["completed_tasks"], 0);
    }

    #[test]
    fn test_implementing_routes_to_implement_task() {
        let (_temp_dir, change_dir) = setup_change_dir("change_implementation_created");
        write_tasks_md(&change_dir, tasks_with_deps());
        let result = handle(&change_dir, "test-change", "/tmp").unwrap();
        assert_eq!(result["action"], "implement_task");
        assert_eq!(result["task_id"], "2.1");
    }

    #[test]
    fn test_implemented_reviews_current_task() {
        let (_temp_dir, change_dir) = setup_change_dir("change_implementation_created");
        write_tasks_md(&change_dir, tasks_with_deps());
        // Create impl file to indicate task 2.1 has been implemented (pending review)
        std::fs::write(
            change_dir.join("impl_2.1.md"),
            "---\ntask_id: 2.1\n---\n# Implementation\n",
        )
        .unwrap();
        let result = handle(&change_dir, "test-change", "/tmp").unwrap();
        assert_eq!(result["action"], "review_task");
        assert_eq!(result["task_id"], "2.1");
    }

    #[test]
    fn test_task_approved_moves_to_next() {
        let (_temp_dir, change_dir) = setup_change_dir("change_implementation_created");
        write_tasks_md(&change_dir, tasks_with_deps());
        // Mark task 2.1 as approved
        std::fs::write(
            change_dir.join("impl_2.1.md"),
            "---\nverdict: PASS\ntask_id: 2.1\n---\n# Review\n",
        )
        .unwrap();
        let result = handle(&change_dir, "test-change", "/tmp").unwrap();
        // Should move to task 3.1
        assert_eq!(result["action"], "implement_task");
        assert_eq!(result["task_id"], "3.1");
        assert_eq!(result["completed_tasks"], 1);
    }

    #[test]
    fn test_all_tasks_done() {
        let (_temp_dir, change_dir) = setup_change_dir("change_implementation_created");
        write_tasks_md(&change_dir, tasks_with_deps());
        // Mark all tasks as approved
        for tid in &["2.1", "3.1", "4.1"] {
            std::fs::write(
                change_dir.join(format!("impl_{}.md", tid)),
                format!("---\nverdict: PASS\ntask_id: {}\n---\n# Review\n", tid),
            )
            .unwrap();
        }
        let result = handle(&change_dir, "test-change", "/tmp").unwrap();
        assert_eq!(result["action"], "all_tasks_done");
        assert_eq!(result["completed_tasks"], 3);
    }

    #[test]
    fn test_task_revision_revise_action() {
        let (_temp_dir, change_dir) = setup_change_dir("change_implementation_reviewed");
        write_tasks_md(&change_dir, tasks_with_deps());
        // Write state with current_task_id
        let state_content = "change_id: test-change\nphase: change_implementation_reviewed\niteration: 1\ncurrent_task_id: '2.1'\n";
        std::fs::write(change_dir.join("STATE.yaml"), state_content).unwrap();
        // Write task-scoped review with NEEDS_REVISION
        std::fs::write(
            change_dir.join("impl_2.1.md"),
            "---\nverdict: REVIEWED\ntask_id: 2.1\n---\n# Review\n## Issues\n- Fix error handling\n",
        ).unwrap();
        let result = handle(&change_dir, "test-change", "/tmp").unwrap();
        assert_eq!(result["action"], "revise_task");
        assert_eq!(result["task_id"], "2.1");
        assert_eq!(result["issues_count"], 1);
    }

    #[test]
    fn test_task_terminal_failure_at_revision_limit() {
        let (_temp_dir, change_dir) = setup_change_dir("change_implementation_reviewed");
        write_tasks_md(&change_dir, tasks_with_deps());
        // Write state with current_task_id and 2 revisions already done
        let state_content = "change_id: test-change\nphase: change_implementation_reviewed\niteration: 1\ncurrent_task_id: '2.1'\ntask_revisions:\n  '2.1': 2\n";
        std::fs::write(change_dir.join("STATE.yaml"), state_content).unwrap();
        // Write review with NEEDS_REVISION
        std::fs::write(
            change_dir.join("impl_2.1.md"),
            "---\nverdict: REVIEWED\n---\n# Review\n## Issues\n- Still broken\n",
        )
        .unwrap();
        let result = handle(&change_dir, "test-change", "/tmp").unwrap();
        assert_eq!(result["action"], "task_terminal_failure");
        assert_eq!(result["task_id"], "2.1");
    }

    #[test]
    fn test_task_implementing_terminal_failure_at_revision_limit() {
        let (_temp_dir, change_dir) = setup_change_dir("change_implementation_created");
        write_tasks_md(&change_dir, tasks_with_deps());
        // Write state with current_task_id and 3 revisions (> MAX)
        let state_content = "change_id: test-change\nphase: change_implementation_created\niteration: 1\ncurrent_task_id: '2.1'\ntask_revisions:\n  '2.1': 3\n";
        std::fs::write(change_dir.join("STATE.yaml"), state_content).unwrap();
        let result = handle(&change_dir, "test-change", "/tmp").unwrap();
        assert_eq!(result["action"], "task_terminal_failure");
        assert_eq!(result["task_id"], "2.1");
    }

    #[test]
    fn test_resume_from_current_task_id() {
        let (_temp_dir, change_dir) = setup_change_dir("change_implementation_created");
        write_tasks_md(&change_dir, tasks_with_deps());
        // Mark 2.1 as completed, set current_task_id to 3.1
        std::fs::write(
            change_dir.join("impl_2.1.md"),
            "---\nverdict: PASS\n---\n# Review\n",
        )
        .unwrap();
        let state_content = "change_id: test-change\nphase: change_implementation_created\niteration: 1\ncurrent_task_id: '3.1'\n";
        std::fs::write(change_dir.join("STATE.yaml"), state_content).unwrap();
        let result = handle(&change_dir, "test-change", "/tmp").unwrap();
        assert_eq!(result["action"], "implement_task");
        assert_eq!(result["task_id"], "3.1");
        assert_eq!(result["completed_tasks"], 1);
    }

    #[test]
    fn test_impl_revised_reviews_task() {
        let (_temp_dir, change_dir) = setup_change_dir("change_implementation_revised");
        write_tasks_md(&change_dir, tasks_with_deps());
        let state_content = "change_id: test-change\nphase: change_implementation_revised\niteration: 1\ncurrent_task_id: '2.1'\n";
        std::fs::write(change_dir.join("STATE.yaml"), state_content).unwrap();
        let result = handle(&change_dir, "test-change", "/tmp").unwrap();
        assert_eq!(result["action"], "review_task");
        assert_eq!(result["task_id"], "2.1");
    }

    // -----------------------------------------------------------------------
    // Codegen routing tests
    // -----------------------------------------------------------------------

    fn tasks_with_spec_ref() -> &'static str {
        r#"---
id: test
type: tasks
---
<tasks>

## Task 2.1

```yaml
id: 2.1
action: CREATE
status: pending
file: src/model.rs
spec_ref: codegen-spec:*
```

## Task 3.1

```yaml
id: 3.1
action: CREATE
status: pending
file: src/manual.rs
depends_on: [2.1]
```

</tasks>"#
    }

    #[test]
    fn test_codegen_eligible_task_routes_to_codegen() {
        let (_temp_dir, change_dir) = setup_change_dir("change_implementation_created");
        write_tasks_md(&change_dir, tasks_with_spec_ref());
        // Create a codegen-eligible spec
        let spec_dir = change_dir.join("specs/test-group");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(
            spec_dir.join("codegen-spec.md"),
            "---\ndesign_elements:\n  has_json_schema: true\n---\n# Spec\n",
        )
        .unwrap();
        let result = handle(&change_dir, "test-change", "/tmp").unwrap();
        assert_eq!(result["action"], "implement_task_with_codegen");
        assert_eq!(result["task_id"], "2.1");
        assert_eq!(result["codegen"], true);
        assert!(result["spec_ref"]
            .as_str()
            .unwrap()
            .contains("codegen-spec"));
    }

    #[test]
    fn test_non_codegen_task_routes_to_manual() {
        let (_temp_dir, change_dir) = setup_change_dir("change_implementation_created");
        write_tasks_md(&change_dir, tasks_with_spec_ref());
        // Create a spec that is NOT codegen-eligible
        let spec_dir = change_dir.join("specs/test-group");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(
            spec_dir.join("codegen-spec.md"),
            "---\ndesign_elements:\n  has_json_schema: false\n  has_api_spec: false\n---\n# Spec\n",
        )
        .unwrap();
        let result = handle(&change_dir, "test-change", "/tmp").unwrap();
        assert_eq!(result["action"], "implement_task");
        assert_eq!(result["task_id"], "2.1");
    }

    #[test]
    fn test_task_without_spec_ref_routes_to_manual() {
        let (_temp_dir, change_dir) = setup_change_dir("change_implementation_created");
        write_tasks_md(&change_dir, tasks_with_spec_ref());
        // Mark 2.1 as done, so 3.1 (no spec_ref) is next
        std::fs::write(
            change_dir.join("impl_2.1.md"),
            "---\nverdict: PASS\n---\n# Review\n",
        )
        .unwrap();
        let result = handle(&change_dir, "test-change", "/tmp").unwrap();
        assert_eq!(result["action"], "implement_task");
        assert_eq!(result["task_id"], "3.1");
    }
}

// CODEGEN-END
