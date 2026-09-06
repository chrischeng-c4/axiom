use pm::mcp::resources::{list_resources, read_resource};
use pm::mcp::tools::{call_tool, tool_definitions};
use pm::store::PmStore;
use serde_json::json;

#[tokio::test]
async fn test_mcp_tool_definitions_and_invocations() {
    let store = PmStore::in_memory();

    // 1. Check tool definitions count
    let tools = tool_definitions();
    assert!(tools.len() >= 16);

    // 2. Initialize project via pm_init_project
    let res = call_tool(
        &store,
        "pm_init_project",
        &json!({
            "id": "proj_demo",
            "name": "Demo System",
            "description": "A demo project management system"
        }),
    )
    .await
    .expect("Failed to init project");
    assert!(res[0]["text"].as_str().unwrap().contains("initialized successfully"));

    // 3. Save PRD
    call_tool(
        &store,
        "pm_save_prd",
        &json!({
            "project_id": "proj_demo",
            "prd_id": "prd_001",
            "title": "Core Specifications",
            "content": "## Requirements\nMust be lightweight and fast.",
            "status": "approved"
        }),
    )
    .await
    .unwrap();

    // 4. Save Tech Design
    call_tool(
        &store,
        "pm_save_tech_design",
        &json!({
            "project_id": "proj_demo",
            "td_id": "td_001",
            "prd_id": "prd_001",
            "title": "Rust MCP Server Architecture",
            "content": "Single native binary with stdio JSON-RPC loop."
        }),
    )
    .await
    .unwrap();

    // 5. Create Feature
    call_tool(
        &store,
        "pm_create_feature",
        &json!({
            "project_id": "proj_demo",
            "feature_id": "feat_001",
            "prd_id": "prd_001",
            "tech_design_id": "td_001",
            "title": "Task Dependency Engine",
            "description": "Scheduler that unblocks tasks sequentially.",
            "priority": "critical"
        }),
    )
    .await
    .unwrap();

    // 6. Create Task 1 (prerequisite)
    call_tool(
        &store,
        "pm_create_task",
        &json!({
            "project_id": "proj_demo",
            "feature_id": "feat_001",
            "task_id": "task_1",
            "title": "Write state engine",
            "description": "Implement in-memory BTreeMap state.",
            "priority": "high",
            "blocked_by": []
        }),
    )
    .await
    .unwrap();

    // 7. Create Task 2 (blocked by Task 1)
    call_tool(
        &store,
        "pm_create_task",
        &json!({
            "project_id": "proj_demo",
            "feature_id": "feat_001",
            "task_id": "task_2",
            "title": "Write dependency scheduler",
            "description": "Resolve blocked_by arrays.",
            "priority": "critical",
            "blocked_by": ["task_1"]
        }),
    )
    .await
    .unwrap();

    // 8. Query next actionable task -> Must be task_1
    let next1 = call_tool(
        &store,
        "pm_get_next_actionable_task",
        &json!({ "project_id": "proj_demo" }),
    )
    .await
    .unwrap();
    let text1 = next1[0]["text"].as_str().unwrap();
    assert!(text1.contains("\"id\": \"task_1\""));

    // 9. Query task context for task_1
    let ctx = call_tool(
        &store,
        "pm_get_task_context",
        &json!({ "task_id": "task_1" }),
    )
    .await
    .unwrap();
    let ctx_text = ctx[0]["text"].as_str().unwrap();
    assert!(ctx_text.contains("# Task: Write state engine"));
    assert!(ctx_text.contains("Parent Feature: Task Dependency Engine"));
    assert!(ctx_text.contains("Product Requirements Document: Core Specifications"));

    // 10. Complete task_1
    call_tool(
        &store,
        "pm_update_task_status",
        &json!({
            "task_id": "task_1",
            "status": "done",
            "result_summary": "State engine verified with unit tests."
        }),
    )
    .await
    .unwrap();

    // 11. Query next actionable task -> Now task_2 is unblocked!
    let next2 = call_tool(
        &store,
        "pm_get_next_actionable_task",
        &json!({ "project_id": "proj_demo" }),
    )
    .await
    .unwrap();
    let text2 = next2[0]["text"].as_str().unwrap();
    assert!(text2.contains("\"id\": \"task_2\""));

    // 12. Test resources
    let resources = list_resources(&store).await;
    assert!(resources.len() >= 4); // project, prd, td, tasks

    let res_read = read_resource(&store, "pm://prds/prd_001").await.unwrap();
    assert!(res_read["contents"][0]["text"].as_str().unwrap().contains("Must be lightweight and fast"));
}
