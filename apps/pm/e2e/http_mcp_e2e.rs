use std::net::SocketAddr;
use pm::mcp::http::router;
use pm::store::PmStore;
use serde_json::{json, Value};

/// Helper to send an HTTP POST request with a JSON-RPC 2.0 body and assert valid response.
async fn rpc_request(
    client: &reqwest::Client,
    url: &str,
    id: i64,
    method: &str,
    params: Value,
) -> Value {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    });

    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .expect("HTTP request to MCP server failed");

    assert_eq!(resp.status(), reqwest::StatusCode::OK);

    let json_resp: Value = resp
        .json()
        .await
        .expect("Failed to parse HTTP response as JSON");

    assert_eq!(json_resp["jsonrpc"], "2.0");
    assert_eq!(json_resp["id"], id);
    assert!(
        json_resp.get("error").is_none() || json_resp["error"].is_null(),
        "RPC error returned: {:?}",
        json_resp["error"]
    );

    json_resp["result"].clone()
}

#[tokio::test]
async fn test_e2e_http_mcp_agent_workflow() {
    // 1. Boot an in-memory MCP HTTP server on a random local port (127.0.0.1:0)
    let store = PmStore::in_memory();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to ephemeral port");
    let local_addr: SocketAddr = listener.local_addr().expect("Failed to get local addr");
    let server_url = format!("http://{}", local_addr);
    let mcp_endpoint = format!("{}/mcp", server_url);

    // Spawn the real axum HTTP server in the background
    tokio::spawn(async move {
        axum::serve(listener, router(store)).await.unwrap();
    });

    let client = reqwest::Client::new();

    // 2. Health check via GET /
    let health_resp = client
        .get(&server_url)
        .send()
        .await
        .expect("Failed GET /");
    assert_eq!(health_resp.status(), reqwest::StatusCode::OK);
    let health_body = health_resp.text().await.unwrap();
    assert!(health_body.contains("pm MCP server running"));

    // 3. Handshake: initialize
    let init_res = rpc_request(
        &client,
        &mcp_endpoint,
        1,
        "initialize",
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "e2e-agent-client", "version": "1.0" }
        }),
    )
    .await;
    assert_eq!(init_res["protocolVersion"], "2024-11-05");
    assert_eq!(init_res["serverInfo"]["name"], "pm");
    assert!(init_res["capabilities"]["tools"].is_object());
    assert!(init_res["capabilities"]["resources"].is_object());

    // 4. Discover tools: tools/list
    let tools_res = rpc_request(&client, &mcp_endpoint, 2, "tools/list", json!({})).await;
    let tool_list = tools_res["tools"].as_array().expect("tools array expected");
    assert!(tool_list.len() >= 18);

    // 5. Agent Action: Initialize Project
    let proj_res = rpc_request(
        &client,
        &mcp_endpoint,
        3,
        "tools/call",
        json!({
            "name": "pm_init_project",
            "arguments": {
                "id": "proj_axiom_e2e",
                "name": "Axiom E2E Project",
                "description": "Demonstrating autonomous agent execution via HTTP MCP",
                "root_path": "."
            }
        }),
    )
    .await;
    let proj_text = proj_res["content"][0]["text"].as_str().unwrap();
    assert!(proj_text.contains("initialized successfully"));

    // 6. Agent Action: Author PRD
    let prd_res = rpc_request(
        &client,
        &mcp_endpoint,
        4,
        "tools/call",
        json!({
            "name": "pm_save_prd",
            "arguments": {
                "project_id": "proj_axiom_e2e",
                "prd_id": "prd_auth_e2e",
                "title": "Agent Token Authentication Spec",
                "content": "## Goal\nProvide secure, scoped claim-check tokens for distributed agents.\n\n## Acceptance Criteria\n1. Tokens expire after 1 hour.\n2. Ed25519 asymmetric signature.",
                "status": "approved",
                "version": "v1.0"
            }
        }),
    )
    .await;
    assert!(prd_res["content"][0]["text"].as_str().unwrap().contains("saved successfully"));

    // 7. Agent Action: Author Tech Design
    let td_res = rpc_request(
        &client,
        &mcp_endpoint,
        5,
        "tools/call",
        json!({
            "name": "pm_save_tech_design",
            "arguments": {
                "project_id": "proj_axiom_e2e",
                "td_id": "td_auth_e2e",
                "prd_id": "prd_auth_e2e",
                "title": "Stateless Claim Token Architecture",
                "content": "## Architecture\nUse ring or ed25519-dalek to verify claims without centralized auth database.\n\n## Data Model\nPayload: { sub, aud, exp, roles }.",
                "status": "approved"
            }
        }),
    )
    .await;
    assert!(td_res["content"][0]["text"].as_str().unwrap().contains("saved successfully"));

    // 8. Agent Action: Create Feature linked to PRD & Tech Design
    let feat_res = rpc_request(
        &client,
        &mcp_endpoint,
        6,
        "tools/call",
        json!({
            "name": "pm_create_feature",
            "arguments": {
                "project_id": "proj_axiom_e2e",
                "feature_id": "feat_tokens",
                "prd_id": "prd_auth_e2e",
                "tech_design_id": "td_auth_e2e",
                "title": "Token Verification Engine",
                "description": "Implement verify_token function and integration middleware.",
                "priority": "critical"
            }
        }),
    )
    .await;
    assert!(feat_res["content"][0]["text"].as_str().unwrap().contains("created successfully"));

    // 9. Agent Action: Decompose Feature into Tasks with Dependencies
    // Task 1: No dependency (Priority: High)
    rpc_request(
        &client,
        &mcp_endpoint,
        7,
        "tools/call",
        json!({
            "name": "pm_create_task",
            "arguments": {
                "project_id": "proj_axiom_e2e",
                "feature_id": "feat_tokens",
                "task_id": "task_crypto_keys",
                "title": "Generate Ed25519 Keypairs",
                "description": "Provide key generation and PEM loader functions.",
                "priority": "high",
                "assignee": "dev",
                "blocked_by": []
            }
        }),
    )
    .await;

    // Task 2: Blocked by Task 1 (Priority: Critical)
    rpc_request(
        &client,
        &mcp_endpoint,
        8,
        "tools/call",
        json!({
            "name": "pm_create_task",
            "arguments": {
                "project_id": "proj_axiom_e2e",
                "feature_id": "feat_tokens",
                "task_id": "task_verify_middleware",
                "title": "Implement Verification Middleware",
                "description": "Axum middleware that validates Bearer ed25519 tokens.",
                "priority": "critical",
                "assignee": "dev",
                "blocked_by": ["task_crypto_keys"]
            }
        }),
    )
    .await;

    // Task 3: Blocked by Task 2 (Priority: Medium)
    rpc_request(
        &client,
        &mcp_endpoint,
        9,
        "tools/call",
        json!({
            "name": "pm_create_task",
            "arguments": {
                "project_id": "proj_axiom_e2e",
                "feature_id": "feat_tokens",
                "task_id": "task_e2e_tests",
                "title": "End-to-End Security Tests",
                "description": "Write black-box test cases for valid and expired tokens.",
                "priority": "medium",
                "assignee": "e2e-dev",
                "blocked_by": ["task_verify_middleware"]
            }
        }),
    )
    .await;

    // 10. Autonomous Agent Dispatch: Query next actionable task
    // Even though task_verify_middleware is Critical, it is blocked by task_crypto_keys (High).
    // Therefore, task_crypto_keys MUST be scheduled first!
    let next_task_res = rpc_request(
        &client,
        &mcp_endpoint,
        10,
        "tools/call",
        json!({
            "name": "pm_get_next_actionable_task",
            "arguments": { "project_id": "proj_axiom_e2e" }
        }),
    )
    .await;
    let next_task_text = next_task_res["content"][0]["text"].as_str().unwrap();
    assert!(next_task_text.contains("\"id\": \"task_crypto_keys\""));

    // 11. Agent Single-Shot Context Tool: Query prompt payload for task_crypto_keys
    let ctx_res = rpc_request(
        &client,
        &mcp_endpoint,
        11,
        "tools/call",
        json!({
            "name": "pm_get_task_context",
            "arguments": { "task_id": "task_crypto_keys" }
        }),
    )
    .await;
    let ctx_text = ctx_res["content"][0]["text"].as_str().unwrap();
    assert!(ctx_text.contains("# Task: Generate Ed25519 Keypairs"));
    assert!(ctx_text.contains("Parent Feature: Token Verification Engine"));
    assert!(ctx_text.contains("Product Requirements Document: Agent Token Authentication Spec"));
    assert!(ctx_text.contains("Acceptance Criteria"));
    assert!(ctx_text.contains("Technical Design: Stateless Claim Token Architecture"));

    // 12. Dev Agent executes and completes task_crypto_keys
    rpc_request(
        &client,
        &mcp_endpoint,
        12,
        "tools/call",
        json!({
            "name": "pm_update_task_status",
            "arguments": {
                "task_id": "task_crypto_keys",
                "status": "done",
                "result_summary": "Implemented keypair generation with unit test suite passing."
            }
        }),
    )
    .await;

    // 13. Autonomous Dispatch: Now task_verify_middleware is unblocked and must be scheduled!
    let next_task2 = rpc_request(
        &client,
        &mcp_endpoint,
        13,
        "tools/call",
        json!({
            "name": "pm_get_next_actionable_task",
            "arguments": { "project_id": "proj_axiom_e2e" }
        }),
    )
    .await;
    let next_text2 = next_task2["content"][0]["text"].as_str().unwrap();
    assert!(next_text2.contains("\"id\": \"task_verify_middleware\""));

    // 14. Complete remaining tasks
    rpc_request(
        &client,
        &mcp_endpoint,
        14,
        "tools/call",
        json!({
            "name": "pm_update_task_status",
            "arguments": {
                "task_id": "task_verify_middleware",
                "status": "done",
                "result_summary": "Axum token layer completed."
            }
        }),
    )
    .await;

    rpc_request(
        &client,
        &mcp_endpoint,
        15,
        "tools/call",
        json!({
            "name": "pm_update_task_status",
            "arguments": {
                "task_id": "task_e2e_tests",
                "status": "done",
                "result_summary": "All 15 security test vectors passed."
            }
        }),
    )
    .await;

    // 15. Verify all tasks are complete
    let final_dispatch = rpc_request(
        &client,
        &mcp_endpoint,
        16,
        "tools/call",
        json!({
            "name": "pm_get_next_actionable_task",
            "arguments": { "project_id": "proj_axiom_e2e" }
        }),
    )
    .await;
    assert!(final_dispatch["content"][0]["text"].as_str().unwrap().contains("No actionable tasks available"));

    // 16. Verify Project Summary dashboard over HTTP
    let summary_res = rpc_request(
        &client,
        &mcp_endpoint,
        17,
        "tools/call",
        json!({
            "name": "pm_get_project_summary",
            "arguments": { "project_id": "proj_axiom_e2e" }
        }),
    )
    .await;
    let summary_val: Value = serde_json::from_str(summary_res["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(summary_val["counts"]["prds"], 1);
    assert_eq!(summary_val["counts"]["tech_designs"], 1);
    assert_eq!(summary_val["counts"]["features"], 1);
    assert_eq!(summary_val["counts"]["tasks_total"], 3);
    assert_eq!(summary_val["counts"]["tasks_by_status"]["done"], 3);
    assert_eq!(summary_val["counts"]["tasks_by_status"]["todo"], 0);

    // 17. Inspect Resources over HTTP
    let resources_res = rpc_request(&client, &mcp_endpoint, 18, "resources/list", json!({})).await;
    let r_list = resources_res["resources"].as_array().unwrap();
    assert!(r_list.len() >= 6); // 1 project, 1 prd, 1 td, 3 tasks

    let read_prd_res = rpc_request(
        &client,
        &mcp_endpoint,
        19,
        "resources/read",
        json!({ "uri": "pm://prds/prd_auth_e2e" }),
    )
    .await;
    assert!(read_prd_res["contents"][0]["text"].as_str().unwrap().contains("Ed25519 asymmetric signature"));

    let read_task_res = rpc_request(
        &client,
        &mcp_endpoint,
        20,
        "resources/read",
        json!({ "uri": "pm://tasks/task_verify_middleware" }),
    )
    .await;
    assert!(read_task_res["contents"][0]["text"].as_str().unwrap().contains("Axum middleware that validates Bearer"));
    assert!(read_task_res["contents"][0]["text"].as_str().unwrap().contains("Axum token layer completed"));
}
