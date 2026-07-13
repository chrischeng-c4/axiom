// SPEC-MANAGED: apps/lumen/tech-design/semantic/lumen-tests.md#unit-test
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
//! so `RoutedRouter`'s `h2c::H2cPool` forwarding works identically here and
//! in production. Mirrors `reshard_driver_e2e.rs`'s `spin_up_shard` pattern
//! but wires the pod-side routing layer instead of the driver-facing admin
//! surface.
#![cfg(feature = "operator")]

use std::net::TcpListener;
use std::sync::Arc;

use axum_test::{TestServer, TestServerConfig, Transport};
use serde_json::{json, Value};

use lumen::api::{router, AppState};
use lumen::routing::VirtualBucketShardMap;
use lumen::routing_remote::RoutedRouter;
use lumen::storage::Engine;

const VIRTUAL_BUCKET_COUNT: u32 = 8;

/// Reserves a real, currently-free localhost port by binding then
/// immediately dropping a `TcpListener` — [`RoutedRouter::new`] needs both
/// peers' base URLs at construction time, before either `TestServer` binds,
/// so the port has to be known up front rather than assigned by
/// `Transport::HttpRandomPort` after the fact. Same small TOCTOU window
/// every "pick a free port, bind it again later" test helper accepts.
fn reserve_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral port")
        .local_addr()
        .expect("local addr")
        .port()
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
    let port0 = reserve_port();
    let port1 = reserve_port();
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
    let server0 = TestServer::new_with_config(
        router(state0.with_routed(Arc::new(router0))),
        TestServerConfig {
            transport: Some(Transport::HttpIpPort {
                ip: None,
                port: Some(port0),
            }),
            ..TestServerConfig::default()
        },
    )
    .expect("bind shard 0");

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
    let server1 = TestServer::new_with_config(
        router(state1.with_routed(Arc::new(router1))),
        TestServerConfig {
            transport: Some(Transport::HttpIpPort {
                ip: None,
                port: Some(port1),
            }),
            ..TestServerConfig::default()
        },
    )
    .expect("bind shard 1");

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

// #1398 R2: a forward failure (owning pod unreachable) must surface as a
// clear, retryable error — never a silent local answer.
#[tokio::test]
async fn forward_to_unreachable_shard_surfaces_retryable_error() {
    // Only shard0 is really bound; shard1's URL points at a
    // reserved-but-never-listened-on port, simulating the owning pod being
    // down or mid-roll.
    let port0 = reserve_port();
    let dead_port = reserve_port();
    let shard_urls = vec![
        format!("http://127.0.0.1:{port0}"),
        format!("http://127.0.0.1:{dead_port}"),
    ];
    let engine0 = Arc::new(Engine::new());
    let state0 = AppState::open(engine0.clone());
    let router0 = RoutedRouter::new(
        engine0,
        state0.write_backend.clone(),
        shard_map(),
        0,
        shard_urls,
    )
    .expect("construct shard 0 router");
    let server0 = TestServer::new_with_config(
        router(state0.with_routed(Arc::new(router0))),
        TestServerConfig {
            transport: Some(Transport::HttpIpPort {
                ip: None,
                port: Some(port0),
            }),
            ..TestServerConfig::default()
        },
    )
    .expect("bind shard 0");
    create_users_collection(&server0).await;

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
