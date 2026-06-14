// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Smoke test for `jet dev --wasm` — spawns the dev server against
//! `examples/counter-demo/`, verifies it serves `index.html` +
//! `app_bg.wasm` with the right content-types, then shuts down.
//!
//! Requires `wasm-pack`. Missing prerequisites fail so wasm dev-server
//! readiness cannot be claimed by skipped tests.

#[path = "../common/mod.rs"]
mod common;

use jet::wasm_dev::{self, DevOptions};
use std::fs;

async fn free_port() -> u16 {
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    l.local_addr().unwrap().port()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dev_server_serves_wasm_bundle() {
    common::require_wasm_pack_env();

    let workspace = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let demo = workspace.join("examples").join("counter-demo");
    // Clean prior artifacts so this test exercises the fresh path.
    let _ = fs::remove_dir_all(demo.join("dist"));

    let port = free_port().await;
    let host = "127.0.0.1".to_string();

    // Spawn the dev server. Drop its JoinHandle at end of test to
    // trigger cancellation — shutdown_signal() only listens for
    // Ctrl-C, so abort is how we reclaim the port cleanly.
    let demo_for_task = demo.clone();
    let handle = tokio::spawn(async move {
        wasm_dev::serve(
            &demo_for_task,
            DevOptions {
                host: "127.0.0.1".to_string(),
                port,
                debug: false,
            },
        )
        .await
    });

    // Wait for the server to be listening — poll the root URL until
    // it returns. Cold wasm-pack + wasm-bindgen can take tens of
    // seconds on a fresh cache.
    let base = format!("http://{host}:{port}");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .unwrap();

    let ready = common::wait_for_http_ready(&client, &base).await;
    if !ready {
        handle.abort();
        panic!("dev server never came up on {base} after 90s");
    }

    // index.html
    let index = client
        .get(format!("{base}/"))
        .send()
        .await
        .expect("GET /")
        .error_for_status()
        .expect("index 200");
    assert!(
        index
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .starts_with("text/html"),
        "index content-type should be text/html",
    );
    let body = index.text().await.unwrap();
    assert!(
        body.contains("jet-canvas"),
        "index.html should reference the jet-canvas host element",
    );

    // app_bg.wasm served as application/wasm
    let wasm = client
        .get(format!("{base}/app_bg.wasm"))
        .send()
        .await
        .expect("GET /app_bg.wasm")
        .error_for_status()
        .expect("wasm 200");
    assert_eq!(
        wasm.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok()),
        Some("application/wasm"),
    );
    let wasm_bytes = wasm.bytes().await.unwrap();
    assert!(
        wasm_bytes.starts_with(b"\0asm"),
        "app_bg.wasm should start with the WASM magic header",
    );

    // app.js (wasm-bindgen glue)
    let js = client
        .get(format!("{base}/app.js"))
        .send()
        .await
        .expect("GET /app.js")
        .error_for_status()
        .expect("app.js 200");
    assert!(js
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .starts_with("application/javascript"));

    handle.abort();
}
// CODEGEN-END
