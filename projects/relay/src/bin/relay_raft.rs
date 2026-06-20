// SPEC-MANAGED: projects/relay/tech-design/logic/k8s-manifests-ordinal-role-kind-failover-smoke-raft-ha-layer-2.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:82b7aa0f" tracker="pending-tracker" reason="relay-raft binary: build RaftClusterConfig::from_env(), open a RaftStore on the data dir, RaftDriver::spawn(...), and serve its router over h2c on the configured port."
//! `relay-raft`: a Raft-backed relay node for Kubernetes.
//!
//! Reads its identity and cluster shape from the (downward-API) environment,
//! opens its durable RaftStore on the mounted PVC, starts the [`RaftDriver`],
//! and serves the Raft RPCs + producer publish over h2c.

use std::sync::Arc;

use relay::{RaftClusterConfig, RaftDriver, RaftStore, Relay, RelayCoreConfig};

#[tokio::main]
async fn main() {
    let cfg = RaftClusterConfig::from_env();

    let store = RaftStore::open(&cfg.data_dir, cfg.node_id, cfg.fsync).expect("open RaftStore");
    let relay = Arc::new(Relay::new(RelayCoreConfig {
        data_dir: format!("{}/relay", cfg.data_dir),
        fsync: cfg.fsync,
        ..RelayCoreConfig::default()
    }));

    let driver = RaftDriver::spawn(
        cfg.node_id,
        cfg.membership.clone(),
        cfg.subject.clone(),
        cfg.peers.clone(),
        relay,
        store,
    );

    let addr = format!("0.0.0.0:{}", cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    println!(
        "relay-raft node {} voters={:?} learners={:?} subject={} serving on {}",
        cfg.node_id, cfg.membership.voters, cfg.membership.learners, cfg.subject, addr
    );
    let app = driver.router();
    // `driver` stays in scope for the lifetime of the server (its Drop would
    // otherwise abort the tick task).
    axum::serve(listener, app).await.expect("serve");
    drop(driver);
}
// HANDWRITE-END
