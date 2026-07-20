// HANDWRITE-BEGIN gap="missing-generator:unit-test:2322ee50" tracker="pending-tracker" reason="scaffold for apps/beam/tests/security_hardening.rs — fill in by hand and update tracker when codegen is ready"
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::RwLock;
use serde_json::json;
use beam::service::{AuthConfig, router_with_state};

async fn wait_healthy(client: &reqwest::Client, base: &str) {
    for _ in 0..100 {
        if let Ok(resp) = client.get(format!("{base}/healthz")).send().await {
            if resp.status().is_success() {
                return;
            }
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("server never became healthy");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_security_hardening_routes() {
    // 1. Startup/resolve configuration validation checks
    assert!(AuthConfig::resolve("unknown", None, None).is_err());
    assert!(AuthConfig::resolve("required", None, None).is_err());

    let tokens_json = json!({
        "read-token-docs": {
            "subject": "reader",
            "roles": { "docs": "read" }
        },
        "write-token-docs": {
            "subject": "writer",
            "roles": { "docs": "write" }
        },
        "write-token-other": {
            "subject": "other-writer",
            "roles": { "other": "write" }
        },
        "admin-token": {
            "subject": "admin",
            "roles": { "*": "admin" }
        }
    }).to_string();

    let auth_config = AuthConfig::resolve("required", None, Some(&tokens_json)).unwrap();
    assert!(auth_config.required);
    assert_eq!(auth_config.tokens.len(), 4);

    // Bind an ephemeral port; skip gracefully if the sandbox has no networking.
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(l) => l,
        Err(e) => {
            if std::env::var("BEAM_REQUIRED_GATES").is_ok() {
                panic!("honest-gate-failure: required service listener could not bind: {e}");
            } else {
                eprintln!("skipping service test: cannot bind 127.0.0.1:0 ({e})");
                return;
            }
        }
    };
    let addr = listener.local_addr().expect("bound local addr");
    let base = format!("http://{addr}");

    let gpu = beam::gpu::GpuContext::new().map(Arc::new);
    let registry = Arc::new(RwLock::new(HashMap::new()));
    let app = router_with_state(
        registry.clone(),
        gpu,
        None,
        Arc::new(auth_config.verifier()),
    );

    let server = tokio::spawn(async move {
        beam::service::serve_on(listener, app, std::future::pending::<()>()).await;
    });

    let client = reqwest::Client::new();
    wait_healthy(&client, &base).await;

    // Helper request closures
    let req_get = |url: &str, token: Option<&str>| {
        let mut r = client.get(url);
        if let Some(t) = token {
            r = r.header("authorization", format!("Bearer {t}"));
        }
        async move { r.send().await.unwrap() }
    };

    let req_post = |url: &str, token: Option<&str>, body: serde_json::Value| {
        let mut r = client.post(url);
        if let Some(t) = token {
            r = r.header("authorization", format!("Bearer {t}"));
        }
        let r = r.json(&body);
        async move { r.send().await.unwrap() }
    };

    let req_delete = |url: &str, token: Option<&str>| {
        let mut r = client.delete(url);
        if let Some(t) = token {
            r = r.header("authorization", format!("Bearer {t}"));
        }
        async move { r.send().await.unwrap() }
    };

    // A. Probes must stay exempt/tokenless
    assert_eq!(req_get(&format!("{base}/healthz"), None).await.status(), 200);
    assert_eq!(req_get(&format!("{base}/readyz"), None).await.status(), 200);

    // B. Anonymous requests to data-plane must be rejected with 401
    let resp = req_post(&format!("{base}/v1/collections"), None, json!({
        "name": "docs", "dim": 2, "metric": "l2"
    })).await;
    assert_eq!(resp.status(), 401);
    let err_body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(err_body["error"], "unauthenticated");

    // C. Invalid token must be rejected with 401
    let resp = req_post(&format!("{base}/v1/collections"), Some("invalid-token"), json!({
        "name": "docs", "dim": 2, "metric": "l2"
    })).await;
    assert_eq!(resp.status(), 401);

    // D. Insufficient role (e.g. read-token-docs tries to create collection docs)
    let resp = req_post(&format!("{base}/v1/collections"), Some("read-token-docs"), json!({
        "name": "docs", "dim": 2, "metric": "l2"
    })).await;
    assert_eq!(resp.status(), 403);
    let err_body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(err_body["error"], "forbidden");

    // E. Valid role creates collection docs
    let resp = req_post(&format!("{base}/v1/collections"), Some("write-token-docs"), json!({
        "name": "docs", "dim": 2, "metric": "l2"
    })).await;
    assert_eq!(resp.status(), 201);

    // F. Cross-collection authorization: write-token-other tries to write to docs
    let resp = req_post(&format!("{base}/v1/collections/docs/vectors"), Some("write-token-other"), json!({
        "items": [{"id": "v1", "vector": [1.0, 2.0], "payload": {}}]
    })).await;
    assert_eq!(resp.status(), 403);

    // G. Valid role writes to docs
    let resp = req_post(&format!("{base}/v1/collections/docs/vectors"), Some("write-token-docs"), json!({
        "items": [{"id": "v1", "vector": [1.0, 2.0], "payload": {}}]
    })).await;
    assert_eq!(resp.status(), 200);

    // H. Read-only token queries docs (passes)
    let resp = req_post(&format!("{base}/v1/collections/docs/query"), Some("read-token-docs"), json!({
        "vector": [1.0, 2.0],
        "k": 1
    })).await;
    assert_eq!(resp.status(), 200);

    // I. Read-only token tries to delete vector (forbidden 403)
    let resp = req_delete(&format!("{base}/v1/collections/docs/vectors/v1"), Some("read-token-docs")).await;
    assert_eq!(resp.status(), 403);

    // J. Write token deletes vector (passes)
    let resp = req_delete(&format!("{base}/v1/collections/docs/vectors/v1"), Some("write-token-docs")).await;
    assert_eq!(resp.status(), 200);

    // K. Stricter admin role for backup/restore
    // Write token tries to backup (forbidden)
    assert_eq!(req_get(&format!("{base}/admin/backup"), Some("write-token-docs")).await.status(), 403);
    // Admin token backups (passes)
    let backup_resp = req_get(&format!("{base}/admin/backup"), Some("admin-token")).await;
    assert_eq!(backup_resp.status(), 200);
    let backup_bytes = backup_resp.bytes().await.unwrap();

    // Write token tries to restore (forbidden)
    let req_r = client.post(format!("{base}/admin/restore"))
        .header("authorization", "Bearer write-token-docs")
        .body(backup_bytes.clone());
    assert_eq!(req_r.send().await.unwrap().status(), 403);

    // Admin token restores (passes)
    let req_r = client.post(format!("{base}/admin/restore"))
        .header("authorization", "Bearer admin-token")
        .body(backup_bytes);
    assert_eq!(req_r.send().await.unwrap().status(), 200);

    // L. Oversized restore body (DefaultBodyLimit test)
    let oversized_body = vec![0u8; 11 * 1024 * 1024]; // 11 MiB
    let req_r = client.post(format!("{base}/admin/restore"))
        .header("authorization", "Bearer admin-token")
        .body(oversized_body);
    let status = match req_r.send().await {
        Ok(r) => r.status().as_u16(),
        Err(_) => 413, // Connection aborted early by server is expected for oversized bodies
    };
    assert_eq!(status, 413);

    // Clean shutdown
    server.abort();
}
// HANDWRITE-END
