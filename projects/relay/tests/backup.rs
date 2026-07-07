// HANDWRITE-BEGIN gap="missing-generator:unit-test:a8a111d3" tracker="pending-tracker" reason="Live-node integration over the http2_transport.rs harness (service_http::serve on 127.0.0.1:0): /admin/backup returns parseable EngineSnapshot bytes carrying a published un-acked message; the artifact round-trips through load_snapshot_bytes on a FRESH engine and the message leases back with the original payload (idempotent merge re-load asserted); with auth required the endpoint 401s tokenless, 403s a non-admin token, 200s an admin-on-* token; cfg(feature = backup): relay::backup::run_backup ships the snapshot to a file:// sink (BackupRunResult + artifact on disk) and prunes by retention."
//! Backup surface over a LIVE node (WI #1209): `GET /admin/backup` returns
//! the exact `RelayStateMachine::snapshot` bytes (`EngineSnapshot` =
//! `dump_live` + applied index), the artifact round-trips through the
//! `load_live` MERGE on a fresh engine (the restore semantics: idempotent per
//! `message_id`, leases not replicated), the endpoint is admin-guarded when
//! auth is required, and — with `--features backup` — `relay::backup` ships
//! the bytes to a `libs/service-backup` `file://` sink with age retention.
//!
//! Harness: the shared service shell's serve loop on an ephemeral loopback
//! port (the `http2_transport.rs` pattern), driven with plain reqwest.

use std::collections::BTreeMap;
use std::net::SocketAddr;

use chrono::Utc;
use serde_json::json;

use relay::server::{router, AppState};
use relay::server_config::RelayServerConfig;
use relay::{Relay, RelayCoreConfig};

async fn start_server_with_state(state: AppState) -> SocketAddr {
    let app = router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    // Tests never signal shutdown; the loop lives for the test process.
    tokio::spawn(service_http::serve(
        listener,
        app,
        std::future::pending::<()>(),
    ));
    addr
}

async fn start_server() -> (SocketAddr, AppState) {
    let state = AppState::new(RelayServerConfig::ephemeral());
    let addr = start_server_with_state(state.clone()).await;
    (addr, state)
}

/// R2 / AC2 (endpoint): `GET /admin/backup` on a live node returns the exact
/// bytes `RelayStateMachine::snapshot` would produce — the shared
/// `raft::snapshot_bytes` serialization (single-node: applied floor 0) — and
/// they parse as an `EngineSnapshot` carrying the published un-acked message.
#[tokio::test]
async fn admin_backup_returns_the_state_machine_snapshot_bytes() {
    let (addr, state) = start_server().await;
    state
        .relay_handle()
        .publish(
            "jobs",
            "m-1",
            json!({"task": "t"}),
            BTreeMap::new(),
            Utc::now(),
        )
        .unwrap();

    let resp = reqwest::get(format!("http://{addr}/admin/backup"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let bytes = resp.bytes().await.unwrap();

    // ONE format: the endpoint bytes equal the state machine's serialization.
    let expected = relay::snapshot_bytes(&state.relay_handle(), 0).unwrap();
    assert_eq!(
        &bytes[..],
        &expected[..],
        "endpoint bytes == RelayStateMachine::snapshot bytes"
    );

    let snap: relay::EngineSnapshot = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(snap.up_to, 0, "no raft attached: applied floor is 0");
    assert_eq!(snap.subjects.len(), 1);
    assert_eq!(snap.subjects[0].subject, "jobs");
    assert_eq!(snap.subjects[0].entries.len(), 1);
    assert_eq!(snap.subjects[0].entries[0].message_id, "m-1");
}

/// R2 / AC2 (restore): a backup artifact written through the service-backup
/// local sink round-trips: `load_snapshot_bytes` (the `load_live` merge) on a
/// FRESH engine re-publishes the un-acked message, a consumer leases it back
/// with the original payload, and loading the same artifact twice does NOT
/// duplicate it (merge is idempotent per message_id).
#[tokio::test]
async fn backup_artifact_round_trips_through_load_live_on_a_fresh_engine() {
    let (addr, state) = start_server().await;
    state
        .relay_handle()
        .publish("jobs", "m-1", json!({"n": 42}), BTreeMap::new(), Utc::now())
        .unwrap();

    let bytes = reqwest::get(format!("http://{addr}/admin/backup"))
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();

    // Persist the artifact exactly the way a backup run does (local sink).
    let dir = tempfile::tempdir().unwrap();
    let sink = service_backup::LocalFsSink::new(dir.path(), "relay").unwrap();
    let key = service_backup::BackupSink::put(&sink, std::time::SystemTime::now(), &bytes).unwrap();
    let artifact = std::fs::read(dir.path().join(&key)).unwrap();

    // Fresh node: restore = load_live MERGE.
    let fresh = Relay::new(RelayCoreConfig::in_memory());
    let up_to = relay::load_snapshot_bytes(&fresh, &artifact).unwrap();
    assert_eq!(up_to, 0);
    // Idempotent: a second load of the same artifact dedupes, never appends.
    relay::load_snapshot_bytes(&fresh, &artifact).unwrap();

    let now = Utc::now();
    let lease = fresh
        .lease("jobs", "w-1", now)
        .unwrap()
        .expect("restored message leases back");
    let entry = fresh
        .entry(&lease.subject, lease.shard, lease.seq)
        .unwrap()
        .expect("leased entry body");
    assert_eq!(entry.message_id, "m-1");
    assert_eq!(entry.payload, json!({"n": 42}));
    // Exactly one message exists after the double load.
    assert!(
        fresh.lease("jobs", "w-2", now).unwrap().is_none(),
        "double load must not duplicate the message"
    );
}

/// R2 (guard): with auth required, `GET /admin/backup` rejects a missing
/// token (401) and a non-admin token (403), and serves an admin-on-`*` token
/// (200); probes stay tokenless either way (lumen's guard shape).
#[tokio::test]
async fn admin_backup_requires_admin_when_auth_required() {
    let tokens = json!({
        "admin-token": { "subject": "ops", "roles": { "*": "admin" } },
        "reader-token": { "subject": "worker", "roles": { "*": "read" } },
    })
    .to_string();
    let auth = relay::auth::AuthConfig::resolve("required", None, Some(&tokens)).unwrap();
    let state = AppState::with_auth(RelayServerConfig::ephemeral(), auth);
    let addr = start_server_with_state(state).await;

    let client = reqwest::Client::new();
    let url = format!("http://{addr}/admin/backup");
    assert_eq!(
        client.get(&url).send().await.unwrap().status(),
        401,
        "tokenless is rejected"
    );
    assert_eq!(
        client
            .get(&url)
            .bearer_auth("reader-token")
            .send()
            .await
            .unwrap()
            .status(),
        403,
        "a cluster-wide admin op needs admin on `*`"
    );
    assert_eq!(
        client
            .get(&url)
            .bearer_auth("admin-token")
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    // Probes stay tokenless.
    assert_eq!(
        client
            .get(format!("http://{addr}/healthz"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
}

/// R2 / AC2 (verb, feature `backup`): the library path `relay backup` drives —
/// fetch the snapshot over HTTP, ship it to a `file://` destination sink, and
/// apply age retention (the pre-aged object is pruned, the fresh artifact
/// survives and carries the message).
#[cfg(feature = "backup")]
#[tokio::test]
async fn run_backup_ships_snapshot_to_local_sink() {
    use service_backup::{BackupDestination, RetentionPolicy};

    let (addr, state) = start_server().await;
    state
        .relay_handle()
        .publish(
            "jobs",
            "m-1",
            json!({"task": "t"}),
            BTreeMap::new(),
            Utc::now(),
        )
        .unwrap();

    let dir = tempfile::tempdir().unwrap();
    // Pre-age an object so the retention pass has something to prune.
    std::fs::write(dir.path().join("relay-0.json"), b"old").unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;

    let dest = BackupDestination::from_uri(&format!("file://{}", dir.path().display())).unwrap();
    let result = relay::backup::run_backup(
        &format!("http://{addr}"),
        None,
        &dest,
        &RetentionPolicy::max_age_seconds(1),
    )
    .await
    .unwrap();

    assert!(result.object.bytes > 0);
    let artifact_path = dir.path().join(&result.object.key);
    assert!(artifact_path.exists(), "artifact written to the sink");
    assert_eq!(result.pruned, 1, "the pre-aged object is pruned");

    // The artifact is the same EngineSnapshot the endpoint serves.
    let artifact = std::fs::read(&artifact_path).unwrap();
    let snap: relay::EngineSnapshot = serde_json::from_slice(&artifact).unwrap();
    assert_eq!(snap.subjects[0].entries[0].message_id, "m-1");
}
// HANDWRITE-END
