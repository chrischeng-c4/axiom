use pm::models::{
    Feature, FeatureStatus, Prd, PrdStatus, Priority, Project, Task, TaskStatus,
    TechDesign, TechDesignStatus,
};
use pm::store::{get_next_actionable_task, get_task_context, PmState, PmStore};

#[tokio::test]
async fn test_store_crud_and_atomic_persistence() {
    let temp_dir = std::env::temp_dir().join(format!("pm_test_{}", std::process::id()));
    let state_file = temp_dir.join("state.json");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let store = PmStore::new(&state_file).expect("Failed to initialize store");

    // 1. Create project
    let proj = Project {
        id: "proj_test".to_string(),
        name: "Test Project".to_string(),
        description: "A test project".to_string(),
        root_path: ".".to_string(),
        created_at: "2026-09-06T00:00:00Z".to_string(),
        updated_at: "2026-09-06T00:00:00Z".to_string(),
    };
    store
        .write(|s| {
            s.upsert_project(proj);
            Ok(())
        })
        .await
        .unwrap();

    // Verify file exists on disk
    assert!(state_file.exists());

    // 2. Reload store from disk
    let store2 = PmStore::new(&state_file).expect("Failed to reload store");
    let loaded_proj = store2.read(|s| s.get_project("proj_test").cloned()).await;
    assert!(loaded_proj.is_some());
    assert_eq!(loaded_proj.unwrap().name, "Test Project");

    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_scheduler_dependency_resolution() {
    let mut state = PmState::default();
    let proj_id = "proj_demo";

    // Task 1: setup_db (Priority: Medium, no deps)
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
        created_at: "2026-09-06T01:00:00Z".to_string(),
        updated_at: "2026-09-06T01:00:00Z".to_string(),
    };

    // Task 2: implement_api (Priority: High, blocked by task_1)
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
        created_at: "2026-09-06T02:00:00Z".to_string(),
        updated_at: "2026-09-06T02:00:00Z".to_string(),
    };

    // Task 3: deploy_service (Priority: Critical, blocked by task_2)
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
        created_at: "2026-09-06T03:00:00Z".to_string(),
        updated_at: "2026-09-06T03:00:00Z".to_string(),
    };

    state.upsert_task(t1);
    state.upsert_task(t2);
    state.upsert_task(t3);

    // Step 1: Next actionable task MUST be task_1 because task_2 and task_3 are blocked
    let next1 = get_next_actionable_task(&state, proj_id, None);
    assert!(next1.is_some());
    assert_eq!(next1.unwrap().id, "task_1");

    // Step 2: Mark task_1 as InProgress -> no task should be actionable because task_1 is not yet Done
    state.tasks.get_mut("task_1").unwrap().status = TaskStatus::InProgress;
    let next2 = get_next_actionable_task(&state, proj_id, None);
    assert!(next2.is_none());

    // Step 3: Mark task_1 as Done -> task_2 should now become actionable!
    state.tasks.get_mut("task_1").unwrap().status = TaskStatus::Done;
    let next3 = get_next_actionable_task(&state, proj_id, None);
    assert!(next3.is_some());
    assert_eq!(next3.unwrap().id, "task_2");

    // Step 4: Mark task_2 as Done -> task_3 (Critical) should now be actionable!
    state.tasks.get_mut("task_2").unwrap().status = TaskStatus::Done;
    let next4 = get_next_actionable_task(&state, proj_id, None);
    assert!(next4.is_some());
    assert_eq!(next4.unwrap().id, "task_3");

    // Step 5: Mark task_3 as Done -> no more tasks
    state.tasks.get_mut("task_3").unwrap().status = TaskStatus::Done;
    let next5 = get_next_actionable_task(&state, proj_id, None);
    assert!(next5.is_none());
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
