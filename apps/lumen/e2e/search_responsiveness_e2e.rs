//! #3997 deterministic reactor-liveness oracle.
//!
//! The router runs on one current-thread Tokio runtime. The injected backend
//! blocks synchronously for `slow`, which reproduced the old handler bug: a
//! direct backend call stopped that sole reactor from polling `/readyz` and an
//! unrelated collection. The production boundary must move only that backend
//! work to the bounded blocking executor.

use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use lumen::api::{router, AppState, SearchBackend};
use lumen::storage::Engine;
use lumen::types::{SearchHit, SearchRequest, SearchResponse};
use serde_json::json;

/// This prevents a broken test from hanging forever. It is not a latency
/// target: success is the channel ordering below, where both probes complete
/// before the slow backend receives its release.
const TEST_DEADLOCK_GUARD: Duration = Duration::from_secs(3);

struct GatedSearch {
    entered: mpsc::Sender<()>,
    releases: Arc<(Mutex<usize>, Condvar)>,
}

impl SearchBackend for GatedSearch {
    fn search(&self, collection_id: &str, _req: SearchRequest) -> Result<SearchResponse> {
        if collection_id == "slow" {
            self.entered
                .send(())
                .expect("test waits for slow search entry");
            let (count, wake) = &*self.releases;
            let mut count = count.lock().expect("release counter lock");
            while *count == 0 {
                count = wake.wait(count).expect("release counter wait");
            }
            *count -= 1;
        }
        Ok(SearchResponse {
            hits: vec![SearchHit {
                external_id: collection_id.to_string(),
                score: 1.0,
            }],
            total: 1,
            cursor: None,
            took_ms: 0,
            took_us: 0,
        })
    }
}

fn release_one(releases: &Arc<(Mutex<usize>, Condvar)>) {
    let (count, wake) = &**releases;
    *count.lock().expect("release counter lock") += 1;
    wake.notify_one();
}

fn search_body() -> serde_json::Value {
    json!({
        "query":{"term":{"field":"ignored","value":"ignored"}},
        "limit":1
    })
}

fn start_one_worker_server(
    backend: Arc<dyn SearchBackend>,
) -> (
    String,
    tokio::sync::oneshot::Sender<()>,
    thread::JoinHandle<()>,
) {
    let (address_tx, address_rx) = mpsc::channel();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let worker = thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("one-worker runtime");
        runtime.block_on(async move {
            let state = AppState::open(Arc::new(Engine::new())).with_search_backend(backend);
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind responsiveness listener");
            address_tx
                .send(listener.local_addr().expect("listener address"))
                .expect("send listener address");
            let _ = axum::serve(listener, router(state))
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await;
        });
    });
    let address = address_rx
        .recv_timeout(TEST_DEADLOCK_GUARD)
        .expect("one-worker server address");
    (format!("http://{address}"), shutdown_tx, worker)
}

enum LivenessEvent {
    Fast(Result<(), String>),
    Readyz(Result<(), String>),
}

async fn fast_probe(client: reqwest::Client, base: String) -> Result<(), String> {
    let response = client
        .post(format!("{base}/collections/fast/search"))
        .json(&search_body())
        .send()
        .await
        .map_err(|error| format!("unrelated search transport: {error}"))?;
    if !response.status().is_success() {
        return Err(format!("unrelated search status: {}", response.status()));
    }
    Ok(())
}

async fn readyz_probe(client: reqwest::Client, base: String) -> Result<(), String> {
    let ready = client
        .get(format!("{base}/readyz"))
        .send()
        .await
        .map_err(|error| format!("readyz transport: {error}"))?;
    if ready.status() != reqwest::StatusCode::OK {
        return Err(format!("readyz status: {}", ready.status()));
    }
    match ready.text().await {
        Ok(body) if body == "ok" => Ok(()),
        Ok(body) => Err(format!("readyz body: {body:?}")),
        Err(error) => Err(format!("readyz body read: {error}")),
    }
}

/// Starts both probes after the slow backend has entered its gate. The caller
/// waits for success events from both probes before it releases the gate.
fn spawn_liveness_probes(
    client: &reqwest::Client,
    base: &str,
) -> (
    tokio::sync::mpsc::UnboundedReceiver<LivenessEvent>,
    Vec<tokio::task::JoinHandle<()>>,
) {
    let (events_tx, events_rx) = tokio::sync::mpsc::unbounded_channel();

    let fast_tx = events_tx.clone();
    let fast_client = client.clone();
    let fast_base = base.to_string();
    let fast = tokio::spawn(async move {
        let _ = fast_tx.send(LivenessEvent::Fast(
            fast_probe(fast_client, fast_base).await,
        ));
    });

    let ready_tx = events_tx;
    let ready_client = client.clone();
    let ready_base = base.to_string();
    let readyz = tokio::spawn(async move {
        let _ = ready_tx.send(LivenessEvent::Readyz(
            readyz_probe(ready_client, ready_base).await,
        ));
    });

    (events_rx, vec![fast, readyz])
}

async fn liveness_before_slow_release(client: &reqwest::Client, base: &str) -> Result<(), String> {
    let (mut events, probes) = spawn_liveness_probes(client, base);
    let ordering = tokio::time::timeout(TEST_DEADLOCK_GUARD, async {
        let mut fast_complete = false;
        let mut readyz_complete = false;
        while !fast_complete || !readyz_complete {
            match events.recv().await {
                Some(LivenessEvent::Fast(Ok(()))) => fast_complete = true,
                Some(LivenessEvent::Readyz(Ok(()))) => readyz_complete = true,
                Some(LivenessEvent::Fast(Err(error))) => {
                    return Err(format!("unrelated search failed before release: {error}"));
                }
                Some(LivenessEvent::Readyz(Err(error))) => {
                    return Err(format!("readyz failed before release: {error}"));
                }
                None => return Err("liveness probe channel closed before both successes".into()),
            }
        }
        Ok(())
    })
    .await
    .map_err(|_| "liveness probes did not complete before the deadlock guard".to_string())?;

    for probe in probes {
        tokio::time::timeout(TEST_DEADLOCK_GUARD, probe)
            .await
            .map_err(|_| "liveness probe task did not finish".to_string())
            .and_then(|result| {
                result.map_err(|error| format!("liveness probe task panicked: {error}"))
            })?;
    }
    ordering
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn slow_single_and_batch_search_do_not_stall_readyz_or_unrelated_search() {
    let (entered_tx, entered_rx) = mpsc::channel();
    let releases = Arc::new((Mutex::new(0usize), Condvar::new()));
    let backend = Arc::new(GatedSearch {
        entered: entered_tx,
        releases: Arc::clone(&releases),
    });
    let (base, shutdown, worker) = start_one_worker_server(backend);
    let client = reqwest::Client::new();

    let slow_client = client.clone();
    let slow_base = base.clone();
    let slow = tokio::spawn(async move {
        slow_client
            .post(format!("{slow_base}/collections/slow/search"))
            .json(&search_body())
            .send()
            .await
    });
    entered_rx
        .recv_timeout(TEST_DEADLOCK_GUARD)
        .expect("single slow search entered backend");
    let single_liveness = liveness_before_slow_release(&client, &base).await;
    release_one(&releases);
    let slow_response = tokio::time::timeout(TEST_DEADLOCK_GUARD, slow)
        .await
        .expect("single slow request completes after release")
        .expect("single slow task join")
        .expect("single slow transport");
    assert!(slow_response.status().is_success());
    assert!(
        single_liveness.is_ok(),
        "single slow search stalled the one-worker server before release: {single_liveness:?}"
    );

    let batch_client = client.clone();
    let batch_base = base.clone();
    let batch = tokio::spawn(async move {
        batch_client
            .post(format!("{batch_base}/collections:search"))
            .json(&json!({
                // Batch items flatten SearchRequest on the wire.
                "searches":[{
                    "collection":"slow",
                    "query":{"term":{"field":"ignored","value":"ignored"}},
                    "limit":1
                }]
            }))
            .send()
            .await
    });
    entered_rx
        .recv_timeout(TEST_DEADLOCK_GUARD)
        .expect("batch slow search entered backend");
    let batch_liveness = liveness_before_slow_release(&client, &base).await;
    release_one(&releases);
    let batch_response = tokio::time::timeout(TEST_DEADLOCK_GUARD, batch)
        .await
        .expect("batch slow request completes after release")
        .expect("batch slow task join")
        .expect("batch slow transport");
    assert!(batch_response.status().is_success());
    assert!(
        batch_liveness.is_ok(),
        "batch slow search stalled the one-worker server before release: {batch_liveness:?}"
    );

    shutdown.send(()).expect("server shutdown");
    worker.join().expect("one-worker server joins");
}
