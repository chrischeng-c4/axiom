// CODEGEN-BEGIN
//! Cross-pod shard routing end-to-end (#1398 R1-R3).
//!
//! No live kind cluster: two independent [`lumen::storage::Engine`]s, each
//! behind a real bound [`axum_test::TestServer`], each wired with a real
//! [`lumen::routing_remote::RoutedRouter`] that knows the other's real
//! address — the same h2c-over-headless-DNS forwarding a live cluster's
//! pods would do, minus k8s itself. `axum_test::TestServer` real-port
//! instances are h2c-prior-knowledge compatible (both it and production
//! `service_http::serve` build on `hyper_util::server::conn::auto::Builder`),
//! so `RoutedRouter`'s `transport_h2c::H2cPool` forwarding works identically here and
//! in production. Mirrors `reshard_driver_e2e.rs`'s `spin_up_shard` pattern
//! but wires the pod-side routing layer instead of the driver-facing admin
//! surface.
//!
//! ## Contracts inherited from the retired EC shells
//!
//! This sentence was the whole of the `// Contract:` comment in 1 AW-EC shell under
//! `apps/lumen/e2e/`, which ran `cargo test -p lumen --features operator --test
//! routed_shard_e2e forward_write_and_forward_read_land_on_owning_shard -- --exact` in
//! a subprocess and asserted the child's exit status. The name filter narrows; the gate
//! runs the superset.
//!
//! Until 2026-08-20 these shells could not be deleted. The project's only declared gate
//! was `cargo test -p lumen`, and with `default = []` that command compiled every
//! `#![cfg(feature = "operator")]` target into an empty binary that printed `0 passed`
//! and exited 0 — so the shells were the sole surviving record that these checks should
//! run at all. `apps/lumen/CONTRIBUTING.md` declared `cargo test -p lumen --features
//! "operator delegated-auth"` as a required second gate row that day, and that run
//! executes this target directly. That made each shell a second, nested run of a target
//! the gate already covers, so they were deleted the same day. The sentence is the only
//! thing they held that nothing else did. Each line below is prefixed with the EC id
//! its shell was filed under.
//!
//! - `lumen-claim-dynamic-cross-pod-routing` — Cross-pod reads and writes follow the
//!   delivered ownership map to the owning shard.
#![cfg(feature = "operator")]

use std::sync::Arc;

use axum_test::TestServer;
use serde_json::{json, Value};
use tokio::net::TcpListener;

use lumen::api::{router, AppState};
use lumen::routing::VirtualBucketShardMap;
use lumen::routing_remote::RoutedRouter;
use lumen::storage::Engine;
use lumen::types::{CreateCollectionRequest, FieldSpec, FieldType};

const VIRTUAL_BUCKET_COUNT: u32 = 8;

/// Binds a real localhost listener and keeps it bound while peer URLs are
/// constructed. Passing this exact listener into `axum::serve` removes the
/// free-port reservation race between concurrent tests.
fn bind_listener() -> (TcpListener, u16) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    listener
        .set_nonblocking(true)
        .expect("make listener nonblocking");
    let port = listener.local_addr().expect("local addr").port();
    let listener = TcpListener::from_std(listener).expect("adopt bound listener");
    (listener, port)
}

struct RoutedShard {
    server: TestServer,
}

fn shard_map() -> VirtualBucketShardMap {
    VirtualBucketShardMap::balanced(1, VIRTUAL_BUCKET_COUNT, 2).expect("balanced 2-shard map")
}

/// Spins up a real two-shard routed cluster: two independent `Engine`s,
/// each behind a real bound `TestServer`, each wired with a `RoutedRouter`
/// (local_shard 0 and 1 respectively) that knows both real addresses.
fn spin_up_routed_pair() -> (RoutedShard, RoutedShard) {
    let (listener0, port0) = bind_listener();
    let (listener1, port1) = bind_listener();
    let shard_urls = vec![
        format!("http://127.0.0.1:{port0}"),
        format!("http://127.0.0.1:{port1}"),
    ];

    let engine0 = Arc::new(Engine::new());
    let state0 = AppState::open(engine0.clone());
    let router0 = RoutedRouter::new(
        engine0,
        state0.write_backend.clone(),
        shard_map(),
        0,
        shard_urls.clone(),
    )
    .expect("construct shard 0 router");
    let server0 = TestServer::new(axum::serve(
        listener0,
        router(state0.with_routed(Arc::new(router0))),
    ))
    .expect("serve shard 0");

    let engine1 = Arc::new(Engine::new());
    let state1 = AppState::open(engine1.clone());
    let router1 = RoutedRouter::new(
        engine1,
        state1.write_backend.clone(),
        shard_map(),
        1,
        shard_urls,
    )
    .expect("construct shard 1 router");
    let server1 = TestServer::new(axum::serve(
        listener1,
        router(state1.with_routed(Arc::new(router1))),
    ))
    .expect("serve shard 1");

    (
        RoutedShard { server: server0 },
        RoutedShard { server: server1 },
    )
}

async fn create_users_collection(s: &TestServer) {
    s.put("/collections/users")
        .json(&json!({ "fields": { "email": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
}

/// Finds an `external_id` whose no-routing-key document route
/// (`route_document(.., None, external_id)`, which hashes on `external_id`
/// itself when no routing key is given) lands on `target_shard` under the
/// fixed 2-shard `shard_map()` — brute force, mirrors `api_e2e.rs`'s
/// `routing_key_for_bucket` for the write path.
fn external_id_for_shard(collection_id: &str, target_shard: u32) -> String {
    let map = shard_map();
    for i in 0..10_000 {
        let id = format!("doc{i}");
        if map.route_document(collection_id, None, &id).shard == target_shard {
            return id;
        }
    }
    panic!("could not find an external_id landing on shard {target_shard}");
}

// #1398 AC1: after an autonomous split, querying the Service for a moved
// bucket's document succeeds from any pod (forwarded); a write to a moved
// bucket lands on the owning shard.
#[tokio::test]
async fn forward_write_and_forward_read_land_on_owning_shard() {
    let (shard0, shard1) = spin_up_routed_pair();
    create_users_collection(&shard0.server).await;
    create_users_collection(&shard1.server).await;

    // Owned by shard 1, not shard 0.
    let remote_id = external_id_for_shard("users", 1);

    // Write: index through the NON-owning pod (shard0) — `RoutedRouter`
    // must forward this one hop to shard1, the owning shard.
    shard0
        .server
        .post("/collections/users/index")
        .json(&json!({
            "items": [
                { "external_id": remote_id, "field": "email", "value": "moved@x.com" }
            ]
        }))
        .await
        .assert_status_ok();

    // Read: searching through the SAME non-owning pod must forward and
    // return the doc — a silent local (empty) answer would fail this (R2).
    let resp = shard0
        .server
        .post("/collections/users/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "moved@x.com" } },
            "routing_key": remote_id,
            "limit": 10
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["total"], 1, "body = {body}");
    assert_eq!(body["hits"][0]["external_id"], remote_id);

    // Confirm it truly landed on the OWNING pod's local engine (searching
    // shard1 directly answers locally, no forwarding involved).
    let owner_resp = shard1
        .server
        .post("/collections/users/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "moved@x.com" } },
            "limit": 10
        }))
        .await;
    owner_resp.assert_status_ok();
    assert_eq!(owner_resp.json::<Value>()["total"], 1);
}

// #2496 AC1: `create_collection` fans out through `RoutedBackend` — creating
// a collection via ONE pod must register it on every physical shard, not
// just the pod that answered the request. Without the fix, a write hashing
// to a different shard than the one the create request happened to land on
// fails with `CollectionNotFound` (the same class #2489 fixed for the read
// path, but on the write-registration side).
#[tokio::test]
async fn create_collection_through_one_shard_fans_out_to_all_shards() {
    let (shard0, shard1) = spin_up_routed_pair();
    // Only shard0 ever sees the create request.
    create_users_collection(&shard0.server).await;

    // Owned by shard1, not shard0 — proves the collection is genuinely
    // registered on shard1's own local engine, not merely forward-reachable.
    let remote_id = external_id_for_shard("users", 1);
    shard0
        .server
        .post("/collections/users/index")
        .json(&json!({
            "items": [
                { "external_id": remote_id, "field": "email", "value": "fanned@x.com" }
            ]
        }))
        .await
        .assert_status_ok();

    let owner_resp = shard1
        .server
        .post("/collections/users/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "fanned@x.com" } },
            "limit": 10
        }))
        .await;
    owner_resp.assert_status_ok();
    assert_eq!(owner_resp.json::<Value>()["total"], 1);
}

// #2496: `drop_collection` fans out the same way — dropping through one pod
// must remove the collection everywhere, not leave stale registrations on
// the shards that never saw the delete request.
#[tokio::test]
async fn drop_collection_through_one_shard_fans_out_to_all_shards() {
    let (shard0, shard1) = spin_up_routed_pair();
    create_users_collection(&shard0.server).await;
    create_users_collection(&shard1.server).await;

    shard0
        .server
        .delete("/collections/users?force=true")
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);

    let resp = shard1
        .server
        .post("/collections/users/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "anything@x.com" } },
            "limit": 10
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}

// #1398 AC1: docs:replace + delete also route by ownership, not just index.
#[tokio::test]
async fn forward_replace_docs_and_delete_land_on_owning_shard() {
    let (shard0, shard1) = spin_up_routed_pair();
    create_users_collection(&shard0.server).await;
    create_users_collection(&shard1.server).await;

    let remote_id = external_id_for_shard("users", 1);

    shard0
        .server
        .put("/collections/users/docs:replace")
        .json(&json!({
            "docs": [
                { "external_id": remote_id, "fields": { "email": "replaced@x.com" } }
            ]
        }))
        .await
        .assert_status_ok();

    let after_replace = shard1
        .server
        .post("/collections/users/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "replaced@x.com" } },
            "limit": 10
        }))
        .await;
    after_replace.assert_status_ok();
    assert_eq!(after_replace.json::<Value>()["total"], 1);

    shard0
        .server
        .delete(&format!("/collections/users/index/{remote_id}"))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);

    let after_delete = shard1
        .server
        .post("/collections/users/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "replaced@x.com" } },
            "limit": 10
        }))
        .await;
    after_delete.assert_status_ok();
    assert_eq!(after_delete.json::<Value>()["total"], 0);
}

// #3992: truncate has no document owner.  The coordinating pod must issue one
// command to each active physical shard, while a forwarded subrequest remains
// a one-hop local apply and verifies the sender's map version.
#[tokio::test]
async fn routed_truncate_fans_out_and_rejects_a_stale_forwarded_map() {
    let (shard0, shard1) = spin_up_routed_pair();
    create_users_collection(&shard0.server).await;
    let id0 = external_id_for_shard("users", 0);
    let id1 = external_id_for_shard("users", 1);
    shard0
        .server
        .post("/collections/users/index")
        .json(&json!({
            "items": [
                { "external_id": id0, "field": "email", "value": "old0@x.com" },
                { "external_id": id1, "field": "email", "value": "old1@x.com" }
            ]
        }))
        .await
        .assert_status_ok();

    shard0
        .server
        .post("/collections/users/docs:truncate")
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);

    for shard in [&shard0, &shard1] {
        let local = shard
            .server
            .post("/collections/users/search")
            .add_header("x-lumen-forwarded", "1")
            .add_header("x-lumen-map-version", "1")
            .json(&json!({
                "query": { "term": { "field": "email", "value": "old0@x.com" } },
                "limit": 10
            }))
            .await;
        local.assert_status_ok();
        assert_eq!(local.json::<Value>()["total"], 0);
    }

    shard0
        .server
        .post("/collections/users/index")
        .json(&json!({
            "items": [{ "external_id": id0, "field": "email", "value": "still-live@x.com" }]
        }))
        .await
        .assert_status_ok();
    let stale = shard0
        .server
        .post("/collections/users/docs:truncate")
        .add_header("x-lumen-forwarded", "1")
        .add_header("x-lumen-map-version", "999")
        .await;
    stale.assert_status(axum::http::StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(stale.json::<Value>()["error"], "shard_map_version_mismatch");

    let retained = shard0
        .server
        .post("/collections/users/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "still-live@x.com" } },
            "routing_key": id0,
            "limit": 10
        }))
        .await;
    retained.assert_status_ok();
    assert_eq!(retained.json::<Value>()["total"], 1);
}

// #3992: cross-shard atomicity is intentionally not a transaction.  A remote
// failure returns 5xx but leaves a local shard that already applied truncate
// empty.  This uses a real h2c peer that returns 502, not a mocked router.
#[tokio::test]
async fn routed_truncate_reports_remote_failure_without_rolling_back_local_shard() {
    let (local_listener, local_port) = bind_listener();
    let (remote_listener, remote_port) = bind_listener();
    let remote = TestServer::new(axum::serve(
        remote_listener,
        axum::Router::new().fallback(|| async { axum::http::StatusCode::BAD_GATEWAY }),
    ))
    .expect("serve failing remote");

    let engine = Arc::new(Engine::new());
    engine
        .create_collection(
            "users",
            CreateCollectionRequest {
                fields: std::collections::BTreeMap::from([(
                    "email".into(),
                    FieldSpec {
                        field_type: FieldType::Keyword,
                        analyzer: None,
                        multi: None,
                        dim: None,
                        metric: None,
                        backend: None,
                        quantize: None,
                    },
                )]),
            },
        )
        .unwrap();
    engine
        .index(
            "users",
            lumen::types::IndexRequest {
                items: vec![lumen::types::IndexItem {
                    external_id: "local".into(),
                    field: "email".into(),
                    value: lumen::types::FieldValue::String("local@x.com".into()),
                    version: None,
                }],
                request_id: None,
            },
        )
        .unwrap();
    let state = AppState::open(engine.clone());
    let routed = RoutedRouter::new(
        engine.clone(),
        state.write_backend.clone(),
        shard_map(),
        0,
        vec![
            format!("http://127.0.0.1:{local_port}"),
            format!("http://127.0.0.1:{remote_port}"),
        ],
    )
    .unwrap();
    let local = TestServer::new(axum::serve(
        local_listener,
        router(state.with_routed(Arc::new(routed))),
    ))
    .expect("serve local routed shard");

    let response = local.post("/collections/users/docs:truncate").await;
    response.assert_status(axum::http::StatusCode::BAD_GATEWAY);
    assert_eq!(engine.stats("users").unwrap().documents_indexed, 0);
    drop(remote);
}

// #3994: an incoming id list is partitioned by the active map.  A forwarded
// subrequest is still one hop only and must verify both map version and every
// id's ownership before it may remove local state.
#[tokio::test]
async fn routed_batch_unindex_partitions_and_refuses_stale_or_misrouted_hops() {
    let (shard0, shard1) = spin_up_routed_pair();
    create_users_collection(&shard0.server).await;

    let id0 = external_id_for_shard("users", 0);
    let id1 = external_id_for_shard("users", 1);
    shard0
        .server
        .post("/collections/users/index")
        .json(&json!({
            "items": [
                { "external_id": id0, "field": "email", "value": "old0@x.com" },
                { "external_id": id1, "field": "email", "value": "old1@x.com" }
            ]
        }))
        .await
        .assert_status_ok();

    shard0
        .server
        .post("/collections/users/docs:unindex")
        .json(&json!({ "external_ids": [id0, id1] }))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);

    for (shard, id, value) in [(&shard0, &id0, "old0@x.com"), (&shard1, &id1, "old1@x.com")] {
        let local = shard
            .server
            .post("/collections/users/search")
            .add_header("x-lumen-forwarded", "1")
            .add_header("x-lumen-map-version", "1")
            .json(&json!({
                "query": { "term": { "field": "email", "value": value } },
                "routing_key": id,
                "limit": 10
            }))
            .await;
        local.assert_status_ok();
        assert_eq!(local.json::<Value>()["total"], 0);
    }

    shard0
        .server
        .post("/collections/users/index")
        .json(&json!({
            "items": [
                { "external_id": id0, "field": "email", "value": "retain0@x.com" },
                { "external_id": id1, "field": "email", "value": "retain1@x.com" }
            ]
        }))
        .await
        .assert_status_ok();

    let stale = shard0
        .server
        .post("/collections/users/docs:unindex")
        .add_header("x-lumen-forwarded", "1")
        .add_header("x-lumen-map-version", "999")
        .json(&json!({ "external_ids": [id0] }))
        .await;
    stale.assert_status(axum::http::StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(stale.json::<Value>()["error"], "shard_map_version_mismatch");

    let misrouted = shard0
        .server
        .post("/collections/users/docs:unindex")
        .add_header("x-lumen-forwarded", "1")
        .add_header("x-lumen-map-version", "1")
        .json(&json!({ "external_ids": [id1] }))
        .await;
    misrouted.assert_status(axum::http::StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        misrouted.json::<Value>()["error"],
        "shard_forward_misrouted"
    );

    for (id, value) in [(&id0, "retain0@x.com"), (&id1, "retain1@x.com")] {
        let retained = shard0
            .server
            .post("/collections/users/search")
            .json(&json!({
                "query": { "term": { "field": "email", "value": value } },
                "routing_key": id,
                "limit": 10
            }))
            .await;
        retained.assert_status_ok();
        assert_eq!(retained.json::<Value>()["total"], 1);
    }
}

// #3994: a shard-local unindex is atomic, but a routed batch is not a
// cross-shard transaction.  A failed remote still leaves the local shard's
// already-applied removal visible and returns its 5xx without rollback.
#[tokio::test]
async fn routed_batch_unindex_reports_remote_failure_without_rolling_back_local_shard() {
    let (local_listener, local_port) = bind_listener();
    let (remote_listener, remote_port) = bind_listener();
    let remote = TestServer::new(axum::serve(
        remote_listener,
        axum::Router::new().fallback(|| async { axum::http::StatusCode::BAD_GATEWAY }),
    ))
    .expect("serve failing remote");

    let local_id = external_id_for_shard("users", 0);
    let remote_id = external_id_for_shard("users", 1);
    let engine = Arc::new(Engine::new());
    engine
        .create_collection(
            "users",
            CreateCollectionRequest {
                fields: std::collections::BTreeMap::from([(
                    "email".into(),
                    FieldSpec {
                        field_type: FieldType::Keyword,
                        analyzer: None,
                        multi: None,
                        dim: None,
                        metric: None,
                        backend: None,
                        quantize: None,
                    },
                )]),
            },
        )
        .unwrap();
    engine
        .index(
            "users",
            lumen::types::IndexRequest {
                items: vec![lumen::types::IndexItem {
                    external_id: local_id.clone(),
                    field: "email".into(),
                    value: lumen::types::FieldValue::String("local@x.com".into()),
                    version: None,
                }],
                request_id: None,
            },
        )
        .unwrap();
    let state = AppState::open(engine.clone());
    let routed = RoutedRouter::new(
        engine.clone(),
        state.write_backend.clone(),
        shard_map(),
        0,
        vec![
            format!("http://127.0.0.1:{local_port}"),
            format!("http://127.0.0.1:{remote_port}"),
        ],
    )
    .unwrap();
    let local = TestServer::new(axum::serve(
        local_listener,
        router(state.with_routed(Arc::new(routed))),
    ))
    .expect("serve local routed shard");

    let response = local
        .post("/collections/users/docs:unindex")
        .json(&json!({ "external_ids": [local_id, remote_id] }))
        .await;
    response.assert_status(axum::http::StatusCode::BAD_GATEWAY);
    assert_eq!(engine.stats("users").unwrap().documents_indexed, 0);
    drop(remote);
}

// #3994: every id must pass its own reshard fence before routing begins.  A
// fenced id leaves the whole incoming batch untouched, including an unfenced
// sibling that would route to a different physical shard.
#[tokio::test]
async fn routed_batch_unindex_rejects_any_fenced_id_before_partitioning() {
    let (shard0, _shard1) = spin_up_routed_pair();
    create_users_collection(&shard0.server).await;
    let id0 = external_id_for_shard("users", 0);
    let id1 = external_id_for_shard("users", 1);
    shard0
        .server
        .post("/collections/users/index")
        .json(&json!({
            "items": [
                { "external_id": id0, "field": "email", "value": "keep0@x.com" },
                { "external_id": id1, "field": "email", "value": "keep1@x.com" }
            ]
        }))
        .await
        .assert_status_ok();
    let bucket = shard_map().route_document("users", None, &id0).bucket;
    shard0
        .server
        .post("/admin/reshard:fence")
        .json(&json!({
            "virtual_bucket_count": VIRTUAL_BUCKET_COUNT,
            "buckets": [bucket],
            "ttl_secs": 30
        }))
        .await
        .assert_status_ok();

    let rejected = shard0
        .server
        .post("/collections/users/docs:unindex")
        .json(&json!({ "external_ids": [id0, id1] }))
        .await;
    rejected.assert_status(axum::http::StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(rejected.json::<Value>()["error"], "bucket_write_paused");

    let still_live = shard0
        .server
        .post("/collections/users/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "keep1@x.com" } },
            "routing_key": id1,
            "limit": 10
        }))
        .await;
    still_live.assert_status_ok();
    assert_eq!(still_live.json::<Value>()["total"], 1);
}

// #1398 AC2: routing-key-less search through the Service returns merged
// results spanning both shards.
#[tokio::test]
async fn routing_key_less_search_merges_across_both_shards() {
    let (shard0, shard1) = spin_up_routed_pair();
    create_users_collection(&shard0.server).await;
    create_users_collection(&shard1.server).await;

    let id_shard0 = external_id_for_shard("users", 0);
    let id_shard1 = external_id_for_shard("users", 1);

    // Index both docs through shard0 — one stays local, one forwards.
    shard0
        .server
        .post("/collections/users/index")
        .json(&json!({
            "items": [
                { "external_id": id_shard0, "field": "email", "value": "both@x.com" },
                { "external_id": id_shard1, "field": "email", "value": "both@x.com" }
            ]
        }))
        .await
        .assert_status_ok();

    // No routing_key -> scatter/gather across both shards, reachable
    // through either pod.
    let resp = shard0
        .server
        .post("/collections/users/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "both@x.com" } },
            "limit": 10
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["total"], 2, "body = {body}");
    let mut eids: Vec<String> = body["hits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["external_id"].as_str().unwrap().to_string())
        .collect();
    eids.sort();
    let mut expected = vec![id_shard0, id_shard1];
    expected.sort();
    assert_eq!(eids, expected);
}

// #1398 R3: cross-pod forwarding depth is bounded to one hop — a request
// that already carries the internal forwarded marker header and is legally
// owned by the receiving pod's shard (matching map version, matching
// ownership) must be answered from the local engine, never forwarded again.
#[tokio::test]
async fn already_forwarded_request_never_forwards_again() {
    let (shard0, shard1) = spin_up_routed_pair();
    create_users_collection(&shard0.server).await;
    create_users_collection(&shard1.server).await;

    let local_id = external_id_for_shard("users", 0);

    shard0
        .server
        .post("/collections/users/index")
        .json(&json!({
            "items": [
                { "external_id": local_id, "field": "email", "value": "onlyshard0@x.com" }
            ]
        }))
        .await
        .assert_status_ok();

    // Send the internal forwarded-marker header (with the correct map
    // version) straight to shard0 for a bucket shard0 actually owns — must
    // be answered locally, not re-forwarded.
    let resp = shard0
        .server
        .post("/collections/users/search")
        .add_header("x-lumen-forwarded", "1")
        .add_header("x-lumen-map-version", "1")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "onlyshard0@x.com" } },
            "routing_key": local_id,
            "limit": 10
        }))
        .await;
    resp.assert_status_ok();
    assert_eq!(resp.json::<Value>()["total"], 1);
}

// #1442 R1: a spoofed `x-lumen-forwarded` marker — set directly by an
// external caller, not produced by `RoutedRouter` itself — must never force
// a wrong-shard local answer. Sending it straight to the NON-owning pod for
// a keyed request must be rejected (`shard_forward_misrouted`), not silently
// answered from the wrong pod's local (empty-or-stale) engine.
#[tokio::test]
async fn spoofed_forwarded_header_on_wrong_shard_is_rejected_not_answered_locally() {
    let (shard0, shard1) = spin_up_routed_pair();
    create_users_collection(&shard0.server).await;
    create_users_collection(&shard1.server).await;

    let remote_id = external_id_for_shard("users", 1);

    // Index directly on the OWNING shard (bypass shard0 entirely) so the
    // doc exists only on shard1's engine.
    shard1
        .server
        .post("/collections/users/index")
        .json(&json!({
            "items": [
                { "external_id": remote_id, "field": "email", "value": "onlyshard1@x.com" }
            ]
        }))
        .await
        .assert_status_ok();

    // Spoof the internal forwarded-marker header straight to shard0 (the
    // NON-owning pod) with a correct map version but a routing key that
    // belongs to shard1. Pre-#1442 this forced a silent wrong-shard local
    // answer (empty result, masking the miss); it must now be rejected.
    let resp = shard0
        .server
        .post("/collections/users/search")
        .add_header("x-lumen-forwarded", "1")
        .add_header("x-lumen-map-version", "1")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "onlyshard1@x.com" } },
            "routing_key": remote_id,
            "limit": 10
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::SERVICE_UNAVAILABLE);
    let body: Value = resp.json();
    assert_eq!(body["error"], "shard_forward_misrouted", "body = {body}");
}

// #1442 R1: a spoofed forwarded marker on a WRITE to a wrong-shard bucket
// must be rejected, not silently written to the wrong pod's local engine
// (AC1: "wrong-shard local write no longer possible").
#[tokio::test]
async fn spoofed_forwarded_header_on_wrong_shard_write_is_rejected() {
    let (shard0, _shard1) = spin_up_routed_pair();
    create_users_collection(&shard0.server).await;

    let remote_id = external_id_for_shard("users", 1);

    let resp = shard0
        .server
        .post("/collections/users/index")
        .add_header("x-lumen-forwarded", "1")
        .add_header("x-lumen-map-version", "1")
        .json(&json!({
            "items": [
                { "external_id": remote_id, "field": "email", "value": "spoofed@x.com" }
            ]
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::SERVICE_UNAVAILABLE);
    let body: Value = resp.json();
    assert_eq!(body["error"], "shard_forward_misrouted", "body = {body}");
}

// #1442 R2: a forwarded request whose sender map version disagrees with the
// receiver's live map (the rolling-restart mixed-map window) must be
// rejected with a distinct, retryable error instead of the one-hop guard
// forcing a (possibly stale) local answer.
#[tokio::test]
async fn forwarded_request_with_stale_map_version_is_rejected() {
    let (shard0, _shard1) = spin_up_routed_pair();
    create_users_collection(&shard0.server).await;

    let local_id = external_id_for_shard("users", 0);
    let resp = shard0
        .server
        .post("/collections/users/index")
        .add_header("x-lumen-forwarded", "1")
        .add_header("x-lumen-map-version", "999")
        .json(&json!({
            "items": [
                { "external_id": local_id, "field": "email", "value": "x@x.com" }
            ]
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::SERVICE_UNAVAILABLE);
    let body: Value = resp.json();
    assert_eq!(body["error"], "shard_map_version_mismatch", "body = {body}");
}

// #1442 R4: forwarding must percent-encode the external_id/field path
// segments — an id containing reserved URL characters must round-trip
// through a forwarded delete instead of corrupting the forward URL.
#[tokio::test]
async fn forward_delete_with_reserved_characters_in_external_id() {
    let (shard0, shard1) = spin_up_routed_pair();
    create_users_collection(&shard0.server).await;
    create_users_collection(&shard1.server).await;

    // Find a base id that routes to shard1, then decorate it with reserved
    // URL characters (still the same route since routing hashes on the
    // provided external_id string itself).
    let base_id = external_id_for_shard("users", 1);
    let remote_id = format!("{base_id}/weird?id=1&x=2 y");

    shard0
        .server
        .post("/collections/users/index")
        .json(&json!({
            "items": [
                { "external_id": remote_id, "field": "email", "value": "weird@x.com" }
            ]
        }))
        .await
        .assert_status_ok();

    let after_index = shard1
        .server
        .post("/collections/users/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "weird@x.com" } },
            "limit": 10
        }))
        .await;
    after_index.assert_status_ok();
    assert_eq!(after_index.json::<Value>()["total"], 1);

    shard0
        .server
        .delete(&format!(
            "/collections/users/index/{}",
            urlencoding_for_test(&remote_id)
        ))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);

    let after_delete = shard1
        .server
        .post("/collections/users/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "weird@x.com" } },
            "limit": 10
        }))
        .await;
    after_delete.assert_status_ok();
    assert_eq!(after_delete.json::<Value>()["total"], 0);
}

// #1457 R3: forwarding must also percent-encode the collection_id path
// segment, not just external_id/field — a collection id containing reserved
// URL characters must round-trip through a forwarded index, search, and
// delete instead of corrupting the forward URL (the same class of bug
// #1442 R4 fixed for external_id/field, now closed for collection_id too).
#[tokio::test]
async fn forward_index_search_delete_with_reserved_characters_in_collection_id() {
    let (shard0, shard1) = spin_up_routed_pair();
    let hostile_collection_id = "docs weird/id?x=1&y=2";
    for shard in [&shard0, &shard1] {
        shard
            .server
            .put(&format!(
                "/collections/{}",
                urlencoding_for_test(hostile_collection_id)
            ))
            .json(&json!({ "fields": { "email": { "type": "keyword" } } }))
            .await
            .assert_status_ok();
    }

    let remote_id = external_id_for_shard(hostile_collection_id, 1);

    // Write through the NON-owning pod (shard0): `RoutedRouter::index` must
    // forward this one hop to shard1 using a correctly percent-encoded
    // collection_id path segment.
    shard0
        .server
        .post(&format!(
            "/collections/{}/index",
            urlencoding_for_test(hostile_collection_id)
        ))
        .json(&json!({
            "items": [
                { "external_id": remote_id, "field": "email", "value": "weird@x.com" }
            ]
        }))
        .await
        .assert_status_ok();

    // Keyed forward-search through the same non-owning pod must also
    // percent-encode collection_id and find the doc.
    let search_resp = shard0
        .server
        .post(&format!(
            "/collections/{}/search",
            urlencoding_for_test(hostile_collection_id)
        ))
        .json(&json!({
            "query": { "term": { "field": "email", "value": "weird@x.com" } },
            "routing_key": remote_id,
            "limit": 10
        }))
        .await;
    search_resp.assert_status_ok();
    assert_eq!(search_resp.json::<Value>()["total"], 1);

    // Confirm it truly landed on the owning pod's local engine.
    let owner_resp = shard1
        .server
        .post(&format!(
            "/collections/{}/search",
            urlencoding_for_test(hostile_collection_id)
        ))
        .json(&json!({
            "query": { "term": { "field": "email", "value": "weird@x.com" } },
            "limit": 10
        }))
        .await;
    owner_resp.assert_status_ok();
    assert_eq!(owner_resp.json::<Value>()["total"], 1);

    // Forward-delete through the non-owning pod must also encode
    // collection_id correctly.
    shard0
        .server
        .delete(&format!(
            "/collections/{}/index/{}",
            urlencoding_for_test(hostile_collection_id),
            urlencoding_for_test(&remote_id)
        ))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);

    let after_delete = shard1
        .server
        .post(&format!(
            "/collections/{}/search",
            urlencoding_for_test(hostile_collection_id)
        ))
        .json(&json!({
            "query": { "term": { "field": "email", "value": "weird@x.com" } },
            "limit": 10
        }))
        .await;
    after_delete.assert_status_ok();
    assert_eq!(after_delete.json::<Value>()["total"], 0);
}

/// Minimal RFC 3986 percent-encoder for this test's own outbound request
/// path — deliberately independent of `routing_remote`'s
/// `percent_encode_component` (that function is exercised directly by
/// `routing_remote`'s unit tests); this is the client side of the same
/// round-trip a real HTTP client (or another lumen pod) would perform.
fn urlencoding_for_test(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

// #1442 R6: `reindex_stream` bypasses per-item shard ownership and the write
// fence, so it must be rejected outright in routed multi-shard mode rather
// than silently reindexing only locally-owned buckets.
#[tokio::test]
async fn reindex_stream_is_rejected_in_routed_mode() {
    let (shard0, _shard1) = spin_up_routed_pair();
    create_users_collection(&shard0.server).await;

    let resp = shard0
        .server
        .post("/collections/users/reindex/stream")
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_IMPLEMENTED);
    let body: Value = resp.json();
    assert_eq!(body["error"], "reindex_stream_not_routed", "body = {body}");
}

// #1442 R6: `duplicates` cannot merge duplicate groups across shards, so
// routed multi-shard mode must reject it with a distinct error rather than
// silently answering from only the local shard's view.
#[tokio::test]
async fn duplicates_is_rejected_in_routed_mode() {
    let (shard0, _shard1) = spin_up_routed_pair();
    create_users_collection(&shard0.server).await;

    let resp = shard0
        .server
        .post("/collections/users/duplicates")
        .json(&json!({ "field": "email" }))
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_IMPLEMENTED);
    let body: Value = resp.json();
    assert_eq!(body["error"], "duplicates_not_routed", "body = {body}");
}

// #2489: reproduces the GKE post-split read-visibility defect. The
// collection is created (and indexed) only through shard0 — shard1 never
// locally registers it, standing in for a just-completed reshard split that
// migrated zero of a collection's documents to the new shard (the driver's
// migration pipeline is purely doc-driven and has no way to propagate a
// collection's existence on its own). A routing-key-less search reaching the
// Service-pinned pod with no local record of the collection (shard1) must
// still merge in the other shard's real results instead of failing the
// whole query with `collection not found`.
#[tokio::test]
async fn routing_key_less_search_tolerates_collection_missing_on_one_shard() {
    let (shard0, shard1) = spin_up_routed_pair();
    // Deliberately shard1-only omission: create the collection on shard0
    // alone, and only index a document whose bucket routes to shard0 — the
    // exact shape a reshard split leaves behind when a collection's sole
    // document never moves to the new shard.
    create_users_collection(&shard0.server).await;

    let id_shard0 = external_id_for_shard("users", 0);
    shard0
        .server
        .post("/collections/users/index")
        .json(&json!({
            "items": [
                { "external_id": id_shard0, "field": "email", "value": "onlyshard0@x.com" }
            ]
        }))
        .await
        .assert_status_ok();

    // Keyless search through shard1 — the pod that never created the
    // collection locally — must still find the document via shard0's
    // participation in the scatter, not fail with `collection not found`.
    let resp = shard1
        .server
        .post("/collections/users/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "onlyshard0@x.com" } },
            "limit": 10
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["total"], 1, "body = {body}");
    assert_eq!(body["hits"][0]["external_id"], id_shard0);

    // Keyless search through shard0 (the shard that does hold the
    // collection) must be unaffected by shard1's missing local registration
    // — same total either direction.
    let resp0 = shard0
        .server
        .post("/collections/users/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "onlyshard0@x.com" } },
            "limit": 10
        }))
        .await;
    resp0.assert_status_ok();
    assert_eq!(resp0.json::<Value>()["total"], 1);
}

// #2489: when EVERY shard reports `CollectionNotFound` (a collection that
// genuinely does not exist anywhere), a keyless search must still 404 —
// the per-shard not-found tolerance above must not manufacture an empty
// 200 for a truly nonexistent collection.
#[tokio::test]
async fn routing_key_less_search_404s_when_no_shard_has_the_collection() {
    let (shard0, _shard1) = spin_up_routed_pair();
    // Neither shard ever creates "ghost" — no collection exists anywhere.
    let resp = shard0
        .server
        .post("/collections/ghost/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "x" } },
            "limit": 10
        }))
        .await;
    // The surfaced error tag depends on which participant's not-found was
    // the last one recorded (local `not_found` vs. a forwarded shard's
    // `shard_forwarded_error` wrapping the same underlying 404) — an
    // implementation detail, not part of the contract. The contract is the
    // status code: every participant reporting not-found must still 404,
    // never a manufactured empty 200.
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
    let body: Value = resp.json();
    assert!(
        body["message"]
            .as_str()
            .is_some_and(|m| m.contains("ghost")),
        "body = {body}"
    );
}

// #1398 R2: a forward failure (owning pod unreachable) must surface as a
// clear, retryable error — never a silent local answer.
#[tokio::test]
async fn forward_to_unreachable_shard_surfaces_retryable_error() {
    // Only shard0 is bound. Destination port zero cannot identify a listener,
    // so shard1 stays unreachable without reserving and releasing a real port.
    let (listener0, port0) = bind_listener();
    let shard_urls = vec![
        format!("http://127.0.0.1:{port0}"),
        "http://127.0.0.1:0".to_string(),
    ];
    let engine0 = Arc::new(Engine::new());
    // #2496: `create_collection` now fans out through the router to every
    // physical shard, so it can't be used here once the router is wired
    // (shard1 is deliberately unreachable) — this test is about *search*
    // forwarding to a dead shard, not collection-creation fan-out (that has
    // its own coverage in `create_collection_through_one_shard_fans_out_to_all_shards`),
    // so seed shard0's collection directly against the engine first.
    engine0
        .create_collection(
            "users",
            CreateCollectionRequest {
                fields: std::collections::BTreeMap::from([(
                    "email".to_string(),
                    FieldSpec {
                        field_type: FieldType::Keyword,
                        analyzer: None,
                        multi: None,
                        dim: None,
                        metric: None,
                        backend: None,
                        quantize: None,
                    },
                )]),
            },
        )
        .expect("seed collection directly on shard0's engine");
    let state0 = AppState::open(engine0.clone());
    let router0 = RoutedRouter::new(
        engine0,
        state0.write_backend.clone(),
        shard_map(),
        0,
        shard_urls,
    )
    .expect("construct shard 0 router");
    let server0 = TestServer::new(axum::serve(
        listener0,
        router(state0.with_routed(Arc::new(router0))),
    ))
    .expect("serve shard 0");

    let remote_id = external_id_for_shard("users", 1);
    let resp = server0
        .post("/collections/users/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "x" } },
            "routing_key": remote_id,
            "limit": 10
        }))
        .await;
    // A silent local answer would come back 200 with total: 0; R2 requires
    // a distinct, clearly-kinded retryable error instead.
    resp.assert_status(axum::http::StatusCode::SERVICE_UNAVAILABLE);
    let body: Value = resp.json();
    assert_eq!(body["error"], "shard_forward_unavailable", "body = {body}");
}

// #1398 AC5: a non-routed deployment (shardCount:1 or standalone) never
// constructs a router at all — zero forwarding overhead, not merely a
// router that happens to route everything locally.
#[tokio::test]
async fn appstate_open_has_no_router_by_default() {
    let engine = Arc::new(Engine::new());
    let state = AppState::open(engine);
    assert!(
        state.routed.is_none(),
        "AppState::open must not construct a RoutedRouter (AC5)"
    );
}
// CODEGEN-END
