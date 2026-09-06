use pm::models::{
    DefectSeverity, DefectStatus, Priority, Project, ReviewSeverity, Task, TaskStatus,
};
use pm::store::{execute_gate, verify_and_merge_task, PmStore};

#[tokio::test]
async fn test_defects_and_auto_task_creation() {
    let store = PmStore::in_memory();
    let proj_id = "proj_test_defects";

    store
        .write(|s| {
            s.upsert_project(Project {
                id: proj_id.to_string(),
                name: "Test".to_string(),
                description: "Test".to_string(),
                root_path: ".".to_string(),
                default_gate_cmd: None,
                created_at: "2026-09-06T00:00:00Z".to_string(),
                updated_at: "2026-09-06T00:00:00Z".to_string(),
            });
            Ok(())
        })
        .await
        .unwrap();

    // Call tool to report defect
    let res = pm::mcp::tools::call_tool(
        &store,
        "pm_report_defect",
        &serde_json::json!({
            "project_id": proj_id,
            "title": "Null pointer on empty payload",
            "description": "Panics when JSON body is empty",
            "severity": "critical",
            "reproduction_steps": "curl -X POST with empty body",
            "auto_create_task": true
        }),
    )
    .await
    .unwrap();

    let text = res[0]["text"].as_str().unwrap();
    assert!(text.contains("reported successfully"));
    assert!(text.contains("Actionable fix task"));

    // Check store has 1 defect and 1 auto-created task
    store
        .read(|s| {
            let defects = s.list_defects(proj_id, Some(DefectStatus::Open), None);
            assert_eq!(defects.len(), 1);
            let d = defects[0];
            assert_eq!(d.severity, DefectSeverity::Critical);
            assert!(d.created_task_id.is_some());

            let tasks = s.list_tasks(proj_id, None, Some(TaskStatus::Todo), None);
            assert_eq!(tasks.len(), 1);
            assert_eq!(tasks[0].priority, Priority::Critical);
            assert!(tasks[0].title.contains("Fix: Null pointer"));
        })
        .await;
}

#[tokio::test]
async fn test_review_comments_blocking_gate() {
    let store = PmStore::in_memory();
    let task_id = "task_code_review";

    store
        .write(|s| {
            s.upsert_task(Task {
                id: task_id.to_string(),
                project_id: "proj_1".to_string(),
                feature_id: None,
                title: "Refactor auth".to_string(),
                description: "Refactor".to_string(),
                status: TaskStatus::Todo,
                priority: Priority::High,
                assignee: Some("dev".to_string()),
                blocked_by: vec![],
                result_summary: None,
                created_at: "2026-09-06T00:00:00Z".to_string(),
                updated_at: "2026-09-06T00:00:00Z".to_string(),
            });
            Ok(())
        })
        .await
        .unwrap();

    // Add blocking review comment
    let add_res = pm::mcp::tools::call_tool(
        &store,
        "pm_add_review_comment",
        &serde_json::json!({
            "task_id": task_id,
            "reviewer": "qa-lead",
            "content": "Missing error handling on negative timeout values",
            "severity": "blocking",
            "file_path": "src/auth.rs",
            "line_number": 42
        }),
    )
    .await
    .unwrap();
    let add_text = add_res[0]["text"].as_str().unwrap();
    assert!(add_text.contains("added successfully"));

    // Verify has unresolved blocking reviews
    store
        .read(|s| {
            assert!(s.has_unresolved_blocking_reviews(task_id));
            let comments = s.list_review_comments(task_id, true);
            assert_eq!(comments.len(), 1);
            assert_eq!(comments[0].severity, ReviewSeverity::Blocking);
        })
        .await;

    // Resolve review comment
    let comment_id = store
        .read(|s| s.list_review_comments(task_id, true)[0].id.clone())
        .await;

    pm::mcp::tools::call_tool(
        &store,
        "pm_resolve_review_comment",
        &serde_json::json!({
            "comment_id": comment_id,
            "resolution_note": "Added saturating_sub to prevent underflow"
        }),
    )
    .await
    .unwrap();

    // Verify no longer blocking
    store
        .read(|s| {
            assert!(!s.has_unresolved_blocking_reviews(task_id));
        })
        .await;
}

#[tokio::test]
async fn test_gate_run_execution_and_merge_guard() {
    let store = PmStore::in_memory();
    let task_id = "task_gate_test";

    store
        .write(|s| {
            s.upsert_task(Task {
                id: task_id.to_string(),
                project_id: "proj_gate".to_string(),
                feature_id: None,
                title: "Test Gate".to_string(),
                description: "Testing gate checks".to_string(),
                status: TaskStatus::Todo,
                priority: Priority::High,
                assignee: Some("dev".to_string()),
                blocked_by: vec![],
                result_summary: None,
                created_at: "2026-09-06T00:00:00Z".to_string(),
                updated_at: "2026-09-06T00:00:00Z".to_string(),
            });
            Ok(())
        })
        .await
        .unwrap();

    let temp_dir = std::env::temp_dir().join(format!("pm_test_gate_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    // 1. Attempt merge before any gate run -> MUST fail
    let err1 = verify_and_merge_task(&store, task_id, "main", "squash", &temp_dir).await;
    assert!(err1.is_err());
    assert!(err1.unwrap_err().to_string().contains("no gate run recorded"));

    // 2. Execute a FAILING gate command: `exit 1`
    let fail_run = execute_gate(task_id, "exit 1", &temp_dir).await.unwrap();
    assert_eq!(fail_run.exit_code, 1);
    assert!(!fail_run.passed);
    store
        .write(|s| {
            s.record_gate_run(fail_run);
            Ok(())
        })
        .await
        .unwrap();

    // Attempt merge with failing gate run -> MUST fail
    let err2 = verify_and_merge_task(&store, task_id, "main", "squash", &temp_dir).await;
    assert!(err2.is_err());
    assert!(err2.unwrap_err().to_string().contains("failed with exit code"));

    // 3. Execute a PASSING gate command: `echo "all tests passed"`
    let pass_run = execute_gate(task_id, "echo 'all tests passed'", &temp_dir).await.unwrap();
    assert_eq!(pass_run.exit_code, 0);
    assert!(pass_run.passed);
    assert!(pass_run.stdout.contains("all tests passed"));
    store
        .write(|s| {
            s.record_gate_run(pass_run);
            Ok(())
        })
        .await
        .unwrap();

    // 4. Merge now succeeds!
    let merge_commit = verify_and_merge_task(&store, task_id, "main", "squash", &temp_dir)
        .await
        .unwrap();
    assert!(!merge_commit.is_empty());

    // Verify task is now Done and summary contains commit
    store
        .read(|s| {
            let t = s.get_task(task_id).unwrap();
            assert_eq!(t.status, TaskStatus::Done);
            assert!(t.result_summary.as_ref().unwrap().contains(&merge_commit));
        })
        .await;

    let _ = std::fs::remove_dir_all(&temp_dir);
}
