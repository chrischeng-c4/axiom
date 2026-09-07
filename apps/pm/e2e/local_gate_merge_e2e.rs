use std::net::SocketAddr;
use pm::mcp::http::router;
use pm::store::PmStore;
use serde_json::{json, Value};

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
async fn test_local_gate_merge_and_defect_lifecycle_e2e() {
    let temp_dir = std::env::temp_dir().join(format!("pm_e2e_gate_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    // Initialize git repository with main and feat branch
    let _ = std::process::Command::new("git")
        .args(["-c", "core.fsmonitor=false", "init", "-b", "main"])
        .current_dir(&temp_dir)
        .output();
    let _ = std::process::Command::new("git")
        .args(["-c", "core.fsmonitor=false", "config", "user.name", "E2E Tester"])
        .current_dir(&temp_dir)
        .output();
    let _ = std::process::Command::new("git")
        .args(["-c", "core.fsmonitor=false", "config", "user.email", "e2e@example.com"])
        .current_dir(&temp_dir)
        .output();
    let _ = std::process::Command::new("git")
        .args([
            "-c", "core.fsmonitor=false",
            "commit", "--allow-empty", "-m", "initial commit on main",
        ])
        .current_dir(&temp_dir)
        .output();
    let _ = std::process::Command::new("git")
        .args(["-c", "core.fsmonitor=false", "checkout", "-b", "feat/task_local_1"])
        .current_dir(&temp_dir)
        .output();
    std::fs::write(temp_dir.join("cache.rs"), "pub struct LruCache;").unwrap();
    let _ = std::process::Command::new("git")
        .args(["-c", "core.fsmonitor=false", "add", "cache.rs"])
        .current_dir(&temp_dir)
        .output();
    let _ = std::process::Command::new("git")
        .args([
            "-c", "core.fsmonitor=false",
            "commit", "-m", "work on task_local_1",
        ])
        .current_dir(&temp_dir)
        .output();

    let state_file = temp_dir.join("state.json");
    let store = PmStore::new(&state_file).unwrap();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind ephemeral port");
    let local_addr: SocketAddr = listener.local_addr().unwrap();
    let mcp_url = format!("http://{}/mcp", local_addr);

    tokio::spawn(async move {
        axum::serve(listener, router(store)).await.unwrap();
    });

    let client = reqwest::Client::new();
    let proj_id = "proj_gate_e2e";

    // 1. Initialize project with default_gate_cmd
    let init_res = rpc_request(
        &client,
        &mcp_url,
        1,
        "tools/call",
        json!({
            "name": "pm_init_project",
            "arguments": {
                "id": proj_id,
                "name": "Gate E2E Project",
                "description": "Demonstrating local gate verification replacing gh pr",
                "root_path": temp_dir.to_str().unwrap(),
                "default_gate_cmd": "echo 'all tests passed'"
            }
        }),
    )
    .await;
    assert!(init_res["content"][0]["text"].as_str().unwrap().contains("initialized successfully"));

    // 2. Create Task 1 and Task 2 (Task 2 blocked by Task 1)
    rpc_request(
        &client,
        &mcp_url,
        2,
        "tools/call",
        json!({
            "name": "pm_create_task",
            "arguments": {
                "project_id": proj_id,
                "task_id": "task_local_1",
                "title": "Build Cache Layer",
                "description": "Implement in-memory LRU cache.",
                "priority": "high",
                "blocked_by": []
            }
        }),
    )
    .await;

    rpc_request(
        &client,
        &mcp_url,
        3,
        "tools/call",
        json!({
            "name": "pm_create_task",
            "arguments": {
                "project_id": proj_id,
                "task_id": "task_local_2",
                "title": "Expose Cache API",
                "description": "Add HTTP endpoints for cache.",
                "priority": "critical",
                "blocked_by": ["task_local_1"]
            }
        }),
    )
    .await;

    // 3. Reviewer Agent adds a blocking review comment
    let comment_res = rpc_request(
        &client,
        &mcp_url,
        4,
        "tools/call",
        json!({
            "name": "pm_add_review_comment",
            "arguments": {
                "task_id": "task_local_1",
                "reviewer": "reviewer-agent",
                "content": "Cache size capacity must be bounded to prevent OOM.",
                "severity": "blocking"
            }
        }),
    )
    .await;
    assert!(comment_res["content"][0]["text"].as_str().unwrap().contains("added successfully"));

    // 4. Attempt pm_merge_change -> MUST FAIL (has blocking review comment)
    let merge_blocked = rpc_request(
        &client,
        &mcp_url,
        5,
        "tools/call",
        json!({
            "name": "pm_merge_change",
            "arguments": { "task_id": "task_local_1" }
        }),
    )
    .await;
    let err_text1 = merge_blocked["content"][0]["text"].as_str().unwrap();
    assert!(merge_blocked["isError"].as_bool().unwrap());
    assert!(err_text1.contains("blocking review comment"));

    // 5. Query comments and resolve the blocking review
    let list_comments = rpc_request(
        &client,
        &mcp_url,
        6,
        "tools/call",
        json!({
            "name": "pm_list_review_comments",
            "arguments": { "task_id": "task_local_1", "unresolved_only": true }
        }),
    )
    .await;
    let comments_arr: Value = serde_json::from_str(list_comments["content"][0]["text"].as_str().unwrap()).unwrap();
    let cid = comments_arr[0]["id"].as_str().unwrap();

    rpc_request(
        &client,
        &mcp_url,
        7,
        "tools/call",
        json!({
            "name": "pm_resolve_review_comment",
            "arguments": {
                "comment_id": cid,
                "resolution_note": "Capacity capped at 10,000 entries."
            }
        }),
    )
    .await;

    // 6. Attempt pm_merge_change -> MUST FAIL (no gate run recorded yet)
    let merge_no_gate = rpc_request(
        &client,
        &mcp_url,
        8,
        "tools/call",
        json!({
            "name": "pm_merge_change",
            "arguments": { "task_id": "task_local_1" }
        }),
    )
    .await;
    assert!(merge_no_gate["isError"].as_bool().unwrap());
    assert!(merge_no_gate["content"][0]["text"].as_str().unwrap().contains("no gate run recorded"));

    // 7. Run local gate check via pm_verify_gate (replaces gh pr checks)
    let gate_res = rpc_request(
        &client,
        &mcp_url,
        9,
        "tools/call",
        json!({
            "name": "pm_verify_gate",
            "arguments": {
                "task_id": "task_local_1"
            }
        }),
    )
    .await;
    let gate_text = gate_res["content"][0]["text"].as_str().unwrap();
    assert!(gate_text.contains("PASSED"));
    assert!(gate_text.contains("all tests passed"));
    assert!(gate_text.contains("Log path: .pm/gates/gate_"));
    assert!(gate_text.contains("SHA-256:"));

    // Query gate history to get gate ID and read full log with pm_get_gate_log
    let hist_res = rpc_request(
        &client,
        &mcp_url,
        91,
        "tools/call",
        json!({
            "name": "pm_get_gate_history",
            "arguments": { "task_id": "task_local_1" }
        }),
    )
    .await;
    let hist_runs: Value = serde_json::from_str(hist_res["content"][0]["text"].as_str().unwrap()).unwrap();
    let gate_id = hist_runs[0]["id"].as_str().unwrap();

    let log_res = rpc_request(
        &client,
        &mcp_url,
        92,
        "tools/call",
        json!({
            "name": "pm_get_gate_log",
            "arguments": { "project_id": proj_id, "gate_id": gate_id }
        }),
    )
    .await;
    let log_text = log_res["content"][0]["text"].as_str().unwrap();
    assert!(log_text.contains("all tests passed"));
    assert!(log_text.contains("=== GATE RUN:"));

    // 8. Execute pm_merge_change -> SUCCEEDS! (replaces gh pr merge)
    let merge_ok = rpc_request(
        &client,
        &mcp_url,
        10,
        "tools/call",
        json!({
            "name": "pm_merge_change",
            "arguments": {
                "task_id": "task_local_1",
                "target_branch": "main",
                "strategy": "squash"
            }
        }),
    )
    .await;
    if merge_ok.get("isError").and_then(Value::as_bool).unwrap_or(false) {
        panic!("merge_ok returned error: {:?}", merge_ok["content"]);
    }
    let merge_text = merge_ok["content"][0]["text"].as_str().unwrap();
    assert!(merge_text.contains("successfully merged into 'main'"));

    // 9. Verify downstream Task 2 is now automatically unblocked!
    let next_task = rpc_request(
        &client,
        &mcp_url,
        11,
        "tools/call",
        json!({
            "name": "pm_get_next_actionable_task",
            "arguments": { "project_id": proj_id }
        }),
    )
    .await;
    let next_text = next_task["content"][0]["text"].as_str().unwrap();
    assert!(next_text.contains("\"id\": \"task_local_2\""));

    // 10. QA Agent reports defect (replaces gh issue create)
    let defect_res = rpc_request(
        &client,
        &mcp_url,
        12,
        "tools/call",
        json!({
            "name": "pm_report_defect",
            "arguments": {
                "project_id": proj_id,
                "title": "Cache TTL expiry doesn't invalidate keys",
                "description": "Keys remain readable after TTL expiration.",
                "severity": "critical",
                "auto_create_task": true
            }
        }),
    )
    .await;
    let def_text = defect_res["content"][0]["text"].as_str().unwrap();
    assert!(def_text.contains("reported successfully"));
    assert!(def_text.contains("Actionable fix task"));

    // 11. Verify defect listed in project summary
    let summary_res = rpc_request(
        &client,
        &mcp_url,
        13,
        "tools/call",
        json!({
            "name": "pm_get_project_summary",
            "arguments": { "project_id": proj_id }
        }),
    )
    .await;
    let summary_val: Value = serde_json::from_str(summary_res["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(summary_val["counts"]["defects_total"], 1);
    assert_eq!(summary_val["counts"]["defects_by_status"]["open"], 1);

    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);
}
