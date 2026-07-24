//! `beam serve` service integration test — an end-to-end round trip over the
//! real HTTP/2 (h2c) service, driven with `reqwest` (plain HTTP/1.1 over the same
//! port the h2c server multiplexes).
//!
//! The test binds an ephemeral port inside the process, spawns the serve future,
//! and asserts the full lifecycle: probes, collection create/list, batch upsert
//! with payloads, exact + filtered k-NN query, single-vector delete, and the
//! error cases (wrong dim → 400, unknown collection → 404, duplicate → 409).
//!
//! Deterministic by construction: a tiny fixed corpus with a known answer, and an
//! ephemeral port. Query runs on the GPU flat path when a GPU is present on the
//! host (this Mac) and on the exact CPU flat oracle otherwise — the answer is the
//! same either way. Skips gracefully if the sandbox cannot bind a socket.

use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};

/// Poll `/healthz` until the server answers 200 (the listen socket is already
/// bound before the accept loop spawns, so this succeeds almost immediately).
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
async fn service_end_to_end() {
    // Bind an ephemeral port; skip gracefully if the sandbox has no networking.
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("skipping service test: cannot bind 127.0.0.1:0 ({e})");
            return;
        }
    };
    let addr = listener.local_addr().expect("bound local addr");

    // GPU flat path when a GPU is reachable (this Mac / Metal), else the CPU flat
    // oracle — the graceful GPU-or-CPU choice, identical results.
    let gpu = beam::gpu::GpuContext::new().map(Arc::new);
    let query_path = if gpu.is_some() {
        "GPU flat"
    } else {
        "CPU flat oracle"
    };
    eprintln!("beam service test: query path = {query_path}");

    let app = beam::service::router(gpu, 8 * 1024 * 1024);
    let server = tokio::spawn(async move {
        beam::service::serve_on(listener, app, std::future::pending::<()>()).await;
    });

    let client = reqwest::Client::new();
    let base = format!("http://{addr}");
    wait_healthy(&client, &base).await;

    // 1. Health/readiness probes → 200.
    assert_eq!(
        client
            .get(format!("{base}/healthz"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        client
            .get(format!("{base}/readyz"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );

    // 1b. Create a collection, then list shows it (size 0).
    let create = client
        .post(format!("{base}/v1/collections"))
        .json(&json!({ "name": "docs", "dim": 3, "metric": "l2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(create.status(), 201, "create collection → 201");

    let list: Value = client
        .get(format!("{base}/v1/collections"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let cols = list["collections"].as_array().unwrap();
    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0]["name"], "docs");
    assert_eq!(cols[0]["size"], 0);

    // 2. Batch add vectors with payloads. Deterministic corpus (dim 3, L2):
    //    a=[0,0,0] c1/en, b=[1,0,0] c2/en, c=[0,5,0] c1/fr, d=[0,0,9] c2/fr.
    let add = client
        .post(format!("{base}/v1/collections/docs/vectors"))
        .json(&json!({ "items": [
            { "id": "a", "vector": [0.0, 0.0, 0.0], "payload": { "category": 1, "lang": "en" } },
            { "id": "b", "vector": [1.0, 0.0, 0.0], "payload": { "category": 2, "lang": "en" } },
            { "id": "c", "vector": [0.0, 5.0, 0.0], "payload": { "category": 1, "lang": "fr" } },
            { "id": "d", "vector": [0.0, 0.0, 9.0], "payload": { "category": 2, "lang": "fr" } }
        ] }))
        .send()
        .await
        .unwrap();
    assert_eq!(add.status(), 200);
    let added: Value = add.json().await.unwrap();
    assert_eq!(added["upserted"], 4);

    // list size now reflects the live count.
    let list: Value = client
        .get(format!("{base}/v1/collections"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list["collections"][0]["size"], 4);

    // Query near a=[0,0,0]: nearest two are a then b, L2 scores ascending.
    let q: Value = client
        .post(format!("{base}/v1/collections/docs/query"))
        .json(&json!({ "vector": [0.1, 0.0, 0.0], "k": 2 }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let ns = q["neighbors"].as_array().unwrap();
    assert_eq!(ns.len(), 2);
    assert_eq!(ns[0]["id"], "a");
    assert_eq!(ns[1]["id"], "b");
    let (s0, s1) = (
        ns[0]["score"].as_f64().unwrap(),
        ns[1]["score"].as_f64().unwrap(),
    );
    assert!(
        s0 < s1,
        "L2 scores must be ascending (smaller = nearer): {s0} !< {s1}"
    );
    // Payload round-trips on the returned neighbor.
    assert_eq!(ns[0]["payload"]["category"], 1);
    assert_eq!(ns[0]["payload"]["lang"], "en");

    // 3. Filtered query (category == 2) returns ONLY category-2 rows — even though
    //    the overall-nearest row `a` (category 1) would otherwise win.
    let qf: Value = client
        .post(format!("{base}/v1/collections/docs/query"))
        .json(&json!({
            "vector": [0.1, 0.0, 0.0],
            "k": 4,
            "filter": { "clauses": [ { "op": "eq", "key": "category", "value": 2 } ] }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let nf = qf["neighbors"].as_array().unwrap();
    assert_eq!(nf.len(), 2, "only b and d match category == 2");
    assert_eq!(nf[0]["id"], "b");
    assert_eq!(nf[1]["id"], "d");
    assert!(nf.iter().all(|n| n["payload"]["category"] == 2));
    assert!(
        nf.iter().all(|n| n["id"] != "a"),
        "filtered-out `a` must be absent"
    );

    // 3b. Range filter over an integer attribute.
    let qr: Value = client
        .post(format!("{base}/v1/collections/docs/query"))
        .json(&json!({
            "vector": [0.1, 0.0, 0.0],
            "k": 4,
            "filter": { "clauses": [ { "op": "range", "key": "category", "lo": 1, "hi": 1 } ] }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let nr = qr["neighbors"].as_array().unwrap();
    assert_eq!(nr.len(), 2, "only a and c have category in [1,1]");
    assert_eq!(nr[0]["id"], "a");
    assert_eq!(nr[1]["id"], "c");

    // 4. Delete a vector → subsequent query no longer returns it.
    let del = client
        .delete(format!("{base}/v1/collections/docs/vectors/a"))
        .send()
        .await
        .unwrap();
    assert_eq!(del.status(), 200);
    let qd: Value = client
        .post(format!("{base}/v1/collections/docs/query"))
        .json(&json!({ "vector": [0.1, 0.0, 0.0], "k": 2 }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let nd = qd["neighbors"].as_array().unwrap();
    assert_eq!(nd.len(), 2);
    assert_eq!(nd[0]["id"], "b", "b is nearest once a is deleted");
    assert_eq!(nd[1]["id"], "c");
    assert!(
        nd.iter().all(|n| n["id"] != "a"),
        "deleted `a` must not appear"
    );

    // 5. Error cases.
    // Wrong query dim → 400.
    let bad_dim = client
        .post(format!("{base}/v1/collections/docs/query"))
        .json(&json!({ "vector": [0.1, 0.0], "k": 2 }))
        .send()
        .await
        .unwrap();
    assert_eq!(bad_dim.status(), 400, "dim mismatch → 400");

    // Unknown collection → 404.
    let unknown = client
        .post(format!("{base}/v1/collections/nope/query"))
        .json(&json!({ "vector": [0.1, 0.0, 0.0], "k": 2 }))
        .send()
        .await
        .unwrap();
    assert_eq!(unknown.status(), 404, "unknown collection → 404");

    // Duplicate create → 409.
    let dup = client
        .post(format!("{base}/v1/collections"))
        .json(&json!({ "name": "docs", "dim": 3, "metric": "l2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(dup.status(), 409, "duplicate collection → 409");

    // 6. Backup / Restore integration.
    let backup_resp = client
        .get(format!("{base}/admin/backup"))
        .send()
        .await
        .unwrap();
    assert_eq!(backup_resp.status(), 200);
    let backup_bytes = backup_resp.bytes().await.unwrap();

    // Drop the collection, verify listing is empty, then restore and verify it returns.
    assert_eq!(
        client
            .delete(format!("{base}/v1/collections/docs"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let list_empty: Value = client
        .get(format!("{base}/v1/collections"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list_empty["collections"].as_array().unwrap().len(), 0);

    let restore_resp = client
        .post(format!("{base}/admin/restore"))
        .body(backup_bytes)
        .send()
        .await
        .unwrap();
    assert_eq!(restore_resp.status(), 200);

    let list_restored: Value = client
        .get(format!("{base}/v1/collections"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let cols_restored = list_restored["collections"].as_array().unwrap();
    assert_eq!(cols_restored.len(), 1);
    assert_eq!(cols_restored[0]["name"], "docs");
    assert_eq!(cols_restored[0]["size"], 3); // size is 3 since we deleted vector 'a' earlier

    // Drop the collection → 200, and dropping again → 404.
    assert_eq!(
        client
            .delete(format!("{base}/v1/collections/docs"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        client
            .delete(format!("{base}/v1/collections/docs"))
            .send()
            .await
            .unwrap()
            .status(),
        404
    );

    server.abort();
}

/// #2556: an upsert body over the shared data-plane body cap is rejected
/// with 413 rather than buffered/accepted, guarding against unbounded
/// request bodies on the data plane (probes stay exempt/unbounded).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn oversized_upsert_body_is_rejected_with_413() {
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("skipping oversized_upsert_body test: cannot bind 127.0.0.1:0 ({e})");
            return;
        }
    };
    let addr = listener.local_addr().expect("bound local addr");

    // Test with a small body limit (100 KiB) so we don't have to upload 8 MiB.
    let small_limit = 100 * 1024;
    let gpu = beam::gpu::GpuContext::new().map(Arc::new);
    let app = beam::service::router(gpu, small_limit);

    let server = tokio::spawn(async move {
        beam::service::serve_on(listener, app, std::future::pending::<()>()).await;
    });

    let client = reqwest::Client::new();
    let base = format!("http://{addr}");
    wait_healthy(&client, &base).await;

    // Create a collection first.
    client
        .post(format!("{base}/v1/collections"))
        .json(&json!({ "name": "test", "dim": 3, "metric": "l2" }))
        .send()
        .await
        .unwrap();

    // Build an oversized payload (just over the 100 KiB limit).
    // One vector item is roughly: {"id":"X","vector":[0.0,0.0,0.0],"payload":{...}}
    // Let's create many items to exceed the limit.
    let oversized_id = "a".repeat(100 * 1024 + 1);
    let result = client
        .post(format!("{base}/v1/collections/test/vectors"))
        .header("content-type", "application/json")
        .body(format!(
            r#"{{"items":[{{"id":"{oversized_id}","vector":[0.1,0.2,0.3]}}]}}"#
        ))
        .send()
        .await;

    // The Content-Length short-circuit is the point of the layer: the server
    // answers and closes without reading 100 KiB it has already decided to
    // refuse. The client is still uploading when that happens, so it races
    // between reading the 413 and having its own write fail with
    // ECONNRESET — both outcomes are the same refusal, and asserting only on
    // the status code makes this test flaky (~1 run in 5).
    match result {
        Ok(resp) => {
            assert_eq!(resp.status(), 413, "oversized upsert must be refused");
            // Check that the response is the structured error envelope.
            if let Ok(body) = resp.json::<serde_json::Value>().await {
                assert_eq!(body["error"], "payload_too_large");
            }
        }
        Err(err) => assert!(
            err.is_request(),
            "expected a transport-level refusal from the early close, got: {err}"
        ),
    }

    // Race-free invariant: whichever way the client observed the refusal,
    // the oversized body never reached the collection. Query and verify
    // the collection is still empty.
    let list: Value = client
        .get(format!("{base}/v1/collections"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let cols = list["collections"].as_array().unwrap();
    assert_eq!(
        cols[0]["size"], 0,
        "the refused body must not be inserted: {list}"
    );

    server.abort();
}

/// #2556: an under-limit upsert body succeeds, so the cap doesn't reject
/// everything (sanity check that the layer is configured correctly and
/// doesn't block legitimate traffic).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn under_limit_upsert_body_succeeds() {
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("skipping under_limit_upsert_body test: cannot bind 127.0.0.1:0 ({e})");
            return;
        }
    };
    let addr = listener.local_addr().expect("bound local addr");

    // Test with a reasonable body limit.
    let body_limit = 1024 * 1024; // 1 MiB
    let gpu = beam::gpu::GpuContext::new().map(Arc::new);
    let app = beam::service::router(gpu, body_limit);

    let server = tokio::spawn(async move {
        beam::service::serve_on(listener, app, std::future::pending::<()>()).await;
    });

    let client = reqwest::Client::new();
    let base = format!("http://{addr}");
    wait_healthy(&client, &base).await;

    // Create a collection.
    client
        .post(format!("{base}/v1/collections"))
        .json(&json!({ "name": "test", "dim": 128, "metric": "l2" }))
        .send()
        .await
        .unwrap();

    // Upsert a normal batch of vectors (well under the limit).
    let resp = client
        .post(format!("{base}/v1/collections/test/vectors"))
        .json(&json!({ "items": [
            { "id": "v1", "vector": vec![0.1; 128] },
            { "id": "v2", "vector": vec![0.2; 128] },
        ] }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200, "under-limit upsert must succeed");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["upserted"], 2);

    // Verify the vectors were inserted.
    let list: Value = client
        .get(format!("{base}/v1/collections"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let cols = list["collections"].as_array().unwrap();
    assert_eq!(cols[0]["size"], 2, "under-limit upsert must insert vectors");

    server.abort();
}
