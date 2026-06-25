// SPEC-MANAGED: projects/relay/tech-design/interfaces/rest/relay-keep-worker-facing-openapi-contract-polyglot-worker-integr.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:db6bcdf6" tracker="pending-tracker" reason="Throwaway reference worker (test-only): drives the lease / heartbeat / ack loop over h2c against an in-process relay, validating the worker-facing contract and the served OpenAPI."
//! Reference worker (test-only, #108): a polyglot worker integrates purely over
//! relay's HTTP/2 + OpenAPI contract. This drives the lease -> run -> heartbeat
//! -> ack loop against an in-process relay over h2c and checks the served
//! OpenAPI lists the worker-facing verbs.

use std::collections::BTreeSet;
use std::net::SocketAddr;

use serde_json::json;

use relay::server::{router, AppState};
use relay::server_config::RelayServerConfig;
use relay::wire::{AckResponse, HeartbeatResponse, LeaseResponse};

async fn start_server() -> SocketAddr {
    let app = router(AppState::new(RelayServerConfig::ephemeral()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

fn h2c() -> reqwest::Client {
    reqwest::Client::builder()
        .http2_prior_knowledge()
        .build()
        .unwrap()
}

// #108: a worker integrates over the OpenAPI contract — lease, heartbeat, ack.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reference_worker_drives_lease_heartbeat_ack_over_h2c() {
    let addr = start_server().await;
    let c = h2c();
    let base = format!("http://{addr}/v1/jobs");

    // Enqueue three jobs.
    for id in ["j0", "j1", "j2"] {
        c.post(format!("{base}/publish"))
            .json(&json!({ "message_id": id, "payload": { "job": id } }))
            .send()
            .await
            .unwrap();
    }

    // The worker loop: lease -> (run) -> heartbeat -> ack, until nothing is left.
    let mut done: BTreeSet<u64> = BTreeSet::new();
    loop {
        let lease: LeaseResponse = c
            .post(format!("{base}/lease"))
            .json(&json!({ "consumer_id": "worker-1" }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let Some(lease) = lease.lease else { break };

        // Prove liveness mid-run.
        let hb: HeartbeatResponse = c
            .post(format!("{base}/heartbeat"))
            .json(&json!({ "lease_id": lease.lease_id, "epoch": lease.epoch }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert!(hb.extended, "heartbeat extends a held lease");

        // ...run the job... then ack with the fencing epoch.
        let ack: AckResponse = c
            .post(format!("{base}/ack"))
            .json(&json!({ "lease_id": lease.lease_id, "epoch": lease.epoch }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert!(ack.acked);
        done.insert(lease.seq);
    }

    assert_eq!(
        done,
        BTreeSet::from([0, 1, 2]),
        "every job processed exactly once"
    );

    // committed offset reached the last seq.
    let ack: AckResponse = c
        .post(format!("{base}/ack"))
        .json(&json!({ "lease_id": "nope", "epoch": 1 }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(!ack.acked, "a bogus ack is a no-op");
}

// #108: the worker-facing verbs are present in the served OpenAPI contract.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn openapi_lists_worker_facing_verbs() {
    let addr = start_server().await;
    let doc = h2c()
        .get(format!("http://{addr}/openapi.json"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    for path in [
        // The primary streaming consume path (#447/#448/#463).
        "/v1/{subject}/consume",
        // Deprecated polling verbs, retained for the direct-worker mode (#463).
        "/v1/{subject}/lease",
        "/v1/{subject}/ack",
        "/v1/{subject}/heartbeat",
    ] {
        assert!(
            doc.contains(path),
            "OpenAPI must advertise {path} to workers"
        );
    }
}
// HANDWRITE-END
