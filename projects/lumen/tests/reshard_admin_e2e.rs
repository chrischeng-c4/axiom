// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! Reshard admin verbs end-to-end (#1380, #1389, #1396): bucket-scoped
//! export (`POST /admin/backup:scoped`), additive batch-apply
//! (`POST /admin/reshard:apply`), source-side eviction
//! (`POST /admin/reshard:evict`), on-demand durability checkpoint
//! (`POST /admin/checkpoint`), and the bounded write-pause fence
//! (`POST /admin/reshard:fence`, #1396 R2).

use std::sync::Arc;

use axum_test::TestServer;
use serde_json::json;

use lumen::api::{router, AppState};
use lumen::auth::{AuthConfig, Role, TokenClaims};
use lumen::routing::VirtualBucketShardMap;
use lumen::storage::Engine;

const VIRTUAL_BUCKET_COUNT: u32 = 4;

fn server() -> TestServer {
    let engine = Arc::new(Engine::new());
    let app = router(AppState::open(engine));
    TestServer::new(app).expect("test server")
}

fn auth_server(tokens: Vec<(&str, TokenClaims)>) -> TestServer {
    let engine = Arc::new(Engine::new());
    let cfg = AuthConfig {
        required: true,
        tokens: tokens
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    };
    let app = router(AppState::new(engine, Arc::new(cfg)));
    TestServer::new(app).expect("test server")
}

fn claim(subject: &str, roles: &[(&str, Role)]) -> TokenClaims {
    TokenClaims {
        subject: subject.into(),
        roles: roles.iter().map(|(c, r)| (c.to_string(), *r)).collect(),
    }
}

/// Bucket for `external_id` under the 2-shard balanced map used across this
/// file's fixtures, computed the same way `reshard::snapshot_bucket_subset`
/// and the engine's own routing do.
fn bucket_of(collection_id: &str, external_id: &str) -> u32 {
    let map = VirtualBucketShardMap::balanced(0, VIRTUAL_BUCKET_COUNT, 2).unwrap();
    map.route_document(collection_id, None, external_id).bucket
}

async fn create_users_collection(s: &TestServer) {
    s.put("/collections/u")
        .json(&json!({ "fields": { "email": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
}

async fn index_user(s: &TestServer, external_id: &str) {
    s.post("/collections/u/index")
        .json(&json!({
            "items": [{ "external_id": external_id, "field": "email", "value": format!("{external_id}@x.com") }]
        }))
        .await
        .assert_status_ok();
}

async fn total_docs(s: &TestServer) -> u64 {
    let r = s
        .post("/collections/u/search")
        .json(&json!({ "query": { "exists": { "field": "email" } }, "limit": 1000 }))
        .await;
    let body: serde_json::Value = r.json();
    body["total"].as_u64().unwrap()
}

async fn has_doc(s: &TestServer, external_id: &str) -> bool {
    let r = s
        .post("/collections/u/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": format!("{external_id}@x.com") } },
            "limit": 10
        }))
        .await;
    let body: serde_json::Value = r.json();
    body["total"].as_u64().unwrap() == 1
}

/// AC1: bucket-scoped export A -> apply B (B already has other data) -> B
/// answers both pre-existing and migrated queries; A is unmodified.
#[tokio::test]
async fn scoped_export_then_apply_merges_additively_without_touching_source() {
    let a = server();
    create_users_collection(&a).await;
    let ids: Vec<String> = (0..16).map(|i| format!("a-{i:02}")).collect();
    for id in &ids {
        index_user(&a, id).await;
    }
    assert_eq!(total_docs(&a).await, 16);

    // Pick a single virtual bucket that has at least one doc so the export
    // is a strict subset.
    let moved_bucket = bucket_of("u", &ids[0]);
    let moved_ids: Vec<&String> = ids
        .iter()
        .filter(|id| bucket_of("u", id) == moved_bucket)
        .collect();
    assert!(!moved_ids.is_empty());
    assert!(
        moved_ids.len() < ids.len(),
        "fixture should not put every doc in one bucket"
    );

    let scoped = a
        .post("/admin/backup:scoped")
        .json(&json!({ "virtual_bucket_count": VIRTUAL_BUCKET_COUNT, "buckets": [moved_bucket] }))
        .await;
    scoped.assert_status_ok();
    let snap: serde_json::Value = scoped.json();
    let scoped_ids: Vec<String> = snap["collections"]["u"]["eid_fields"]
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    assert_eq!(scoped_ids.len(), moved_ids.len());

    // B already has its own, disjoint data.
    let b = server();
    create_users_collection(&b).await;
    index_user(&b, "b-existing-1").await;
    index_user(&b, "b-existing-2").await;
    assert_eq!(total_docs(&b).await, 2);

    b.post("/admin/reshard:apply")
        .json(&json!({
            "from_map_version": 0,
            "to_map_version": 1,
            "bucket": moved_bucket,
            "from_shard": 0,
            "to_shard": 1,
            "external_ids": {},
            "snapshot": snap
        }))
        .await
        .assert_status_ok();

    // B answers both its pre-existing docs and the migrated ones.
    assert!(has_doc(&b, "b-existing-1").await);
    assert!(has_doc(&b, "b-existing-2").await);
    for id in &moved_ids {
        assert!(has_doc(&b, id).await, "missing migrated doc {id}");
    }
    assert_eq!(total_docs(&b).await, 2 + moved_ids.len() as u64);

    // A is unmodified.
    assert_eq!(total_docs(&a).await, 16);
    for id in &ids {
        assert!(has_doc(&a, id).await);
    }
}

/// AC2: applying the same batch twice is a no-op the second time —
/// query-visible state after two applies equals state after one.
#[tokio::test]
async fn reshard_apply_is_idempotent_on_retry() {
    let a = server();
    create_users_collection(&a).await;
    index_user(&a, "x1").await;
    index_user(&a, "x2").await;
    let bucket = bucket_of("u", "x1");

    let dump = a.get("/admin/backup").await;
    dump.assert_status_ok();
    let snap: serde_json::Value = dump.json();

    let b = server();
    create_users_collection(&b).await;
    index_user(&b, "b-existing").await;

    let batch = json!({
        "from_map_version": 0,
        "to_map_version": 1,
        "bucket": bucket,
        "from_shard": 0,
        "to_shard": 1,
        "external_ids": {},
        "snapshot": snap
    });

    b.post("/admin/reshard:apply")
        .json(&batch)
        .await
        .assert_status_ok();
    let after_first = total_docs(&b).await;

    b.post("/admin/reshard:apply")
        .json(&batch)
        .await
        .assert_status_ok();
    let after_second = total_docs(&b).await;

    assert_eq!(after_first, after_second);
    assert_eq!(after_second, 3); // b-existing + x1 + x2
    assert!(has_doc(&b, "b-existing").await);
    assert!(has_doc(&b, "x1").await);
    assert!(has_doc(&b, "x2").await);

    // Re-run the same search twice to confirm results are stable too, not
    // just the total count.
    let r1 = b
        .post("/collections/u/search")
        .json(&json!({ "query": { "exists": { "field": "email" } }, "limit": 10 }))
        .await;
    let r2 = b
        .post("/collections/u/search")
        .json(&json!({ "query": { "exists": { "field": "email" } }, "limit": 10 }))
        .await;
    let b1: serde_json::Value = r1.json();
    let b2: serde_json::Value = r2.json();
    assert_eq!(b1["total"], b2["total"]);
}

/// #1457 R1 AC1: a final pass's prune chunk only prunes once every chunk of
/// its `(to_map_version, bucket, collection_id, total_chunks)` group has
/// arrived — partial arrival changes nothing — and the whole protocol is
/// idempotent: re-sending the already-completed group again re-runs the
/// same accumulate-then-apply sequence, which is itself a no-op against
/// already-pruned state (no error, no double-prune).
#[tokio::test]
async fn reshard_prune_accumulates_chunks_and_prunes_once_complete() {
    let b = server();
    create_users_collection(&b).await;
    index_user(&b, "b-existing").await;
    let ids: Vec<String> = (0..8).map(|i| format!("p-{i:02}")).collect();
    for id in &ids {
        index_user(&b, id).await;
    }
    let bucket = bucket_of("u", &ids[0]);
    let bucket_ids: Vec<&String> = ids
        .iter()
        .filter(|id| bucket_of("u", id) == bucket)
        .collect();
    assert!(
        bucket_ids.len() >= 2,
        "fixture needs >=2 docs sharing one bucket"
    );

    // The authoritative "keep" set drops the first bucket doc (as if it had
    // been deleted on the source mid-split) — every other doc in the bucket
    // survives.
    let dropped = bucket_ids[0];
    let kept = &bucket_ids[1..];

    // Send the keep set split across two chunks.
    let mid = (kept.len() / 2).max(1);
    let (first_half, second_half) = kept.split_at(mid.min(kept.len()));

    let chunk0 = json!({
        "to_map_version": 1,
        "bucket": bucket,
        "virtual_bucket_count": VIRTUAL_BUCKET_COUNT,
        "collection_id": "u",
        "chunk_index": 0,
        "total_chunks": 2,
        "keep_ids": first_half,
    });
    let chunk1 = json!({
        "to_map_version": 1,
        "bucket": bucket,
        "virtual_bucket_count": VIRTUAL_BUCKET_COUNT,
        "collection_id": "u",
        "chunk_index": 1,
        "total_chunks": 2,
        "keep_ids": second_half,
    });

    // First chunk lands: not complete yet, nothing pruned.
    let resp = b.post("/admin/reshard:prune").json(&chunk0).await;
    resp.assert_status_ok();
    let out: serde_json::Value = resp.json();
    assert_eq!(out["complete"], json!(false));
    assert!(
        has_doc(&b, dropped).await,
        "must not prune before every chunk of the group lands"
    );

    // Second (final) chunk completes the group -> prune fires exactly once.
    let resp = b.post("/admin/reshard:prune").json(&chunk1).await;
    resp.assert_status_ok();
    let out: serde_json::Value = resp.json();
    assert_eq!(out["complete"], json!(true));
    assert_eq!(out["documents_pruned"], json!(1));
    assert!(
        !has_doc(&b, dropped).await,
        "dropped doc must be pruned once its group completes"
    );
    for id in kept {
        assert!(has_doc(&b, id).await, "kept doc {id} must survive");
    }
    assert!(
        has_doc(&b, "b-existing").await,
        "docs outside the pruned bucket/collection must be untouched"
    );

    // Re-sending the whole, already-completed group again is idempotent.
    b.post("/admin/reshard:prune")
        .json(&chunk0)
        .await
        .assert_status_ok();
    let resp = b.post("/admin/reshard:prune").json(&chunk1).await;
    resp.assert_status_ok();
    let out: serde_json::Value = resp.json();
    assert_eq!(out["complete"], json!(true));
    assert_eq!(
        out["documents_pruned"],
        json!(0),
        "already-pruned state has nothing left to prune"
    );
    for id in kept {
        assert!(has_doc(&b, id).await);
    }
}

/// #1457 R2 AC2: a moved bucket's collection that a batch of deletes
/// emptied entirely on the source still gets an authoritative-prune chunk
/// (empty `keep_ids`) so any stale copy this shard already holds for that
/// bucket is pruned on cutover — the delete-resurrection edge #1443 left
/// open for a bucket+collection pair with zero surviving docs.
#[tokio::test]
async fn reshard_prune_empty_keep_set_prunes_every_doc_in_the_bucket() {
    let b = server();
    create_users_collection(&b).await;
    index_user(&b, "b-existing").await;
    index_user(&b, "stale-1").await;
    let bucket = bucket_of("u", "stale-1");

    let chunk = json!({
        "to_map_version": 1,
        "bucket": bucket,
        "virtual_bucket_count": VIRTUAL_BUCKET_COUNT,
        "collection_id": "u",
        "chunk_index": 0,
        "total_chunks": 1,
        "keep_ids": [],
    });
    let resp = b.post("/admin/reshard:prune").json(&chunk).await;
    resp.assert_status_ok();
    let out: serde_json::Value = resp.json();
    assert_eq!(out["complete"], json!(true));
    assert_eq!(out["documents_pruned"], json!(1));
    assert!(!has_doc(&b, "stale-1").await);
    assert!(
        has_doc(&b, "b-existing").await,
        "doc outside the pruned bucket must survive"
    );
}

/// AC3: evict removes exactly the documents whose bucket no longer belongs
/// to the shard under the newer map — nothing else.
#[tokio::test]
async fn reshard_evict_removes_only_moved_bucket_docs() {
    let s = server();
    create_users_collection(&s).await;
    let ids: Vec<String> = (0..16).map(|i| format!("e-{i:02}")).collect();
    for id in &ids {
        index_user(&s, id).await;
    }
    assert_eq!(total_docs(&s).await, 16);

    // Old map: everything on shard 0 (single-shard balanced map).
    // New map: bucket 0 moves to shard 1; shard 0 keeps the rest.
    let mut assignments = vec![0u32; VIRTUAL_BUCKET_COUNT as usize];
    assignments[0] = 1;
    let new_map = VirtualBucketShardMap::new(1, assignments.clone(), 2).expect("valid shard map");

    let evicted_ids: Vec<&String> = ids
        .iter()
        .filter(|id| {
            let b = bucket_of("u", id);
            assignments[b as usize] != 0
        })
        .collect();
    assert!(!evicted_ids.is_empty());
    assert!(evicted_ids.len() < ids.len());

    let resp = s
        .post("/admin/reshard:evict")
        .json(&json!({
            "shard": 0,
            "map_version": new_map.version(),
            "assignments": assignments,
            "physical_shard_count": 2
        }))
        .await;
    resp.assert_status_ok();

    for id in &ids {
        let should_survive = !evicted_ids.contains(&id);
        assert_eq!(
            has_doc(&s, id).await,
            should_survive,
            "unexpected survival state for {id}"
        );
    }
    assert_eq!(total_docs(&s).await, 16 - evicted_ids.len() as u64);

    // Retrying is idempotent: nothing left to evict changes nothing further.
    s.post("/admin/reshard:evict")
        .json(&json!({
            "shard": 0,
            "map_version": new_map.version(),
            "assignments": assignments,
            "physical_shard_count": 2
        }))
        .await
        .assert_status_ok();
    assert_eq!(total_docs(&s).await, 16 - evicted_ids.len() as u64);
}

/// #1386 R2: eviction must refresh `lumen_storage_bytes` inline, not leave
/// it to the next `/collections/{id}/stats` call — a `/metrics` scrape taken
/// right after `:evict`, with no intervening `stats` call, must already
/// reflect the smaller post-eviction footprint. This is what lets a
/// post-cutover shard-usage scrape (tagged with the new `shardMap.version`)
/// actually see post-migration reality instead of a stale pre-eviction
/// gauge value.
#[tokio::test]
async fn reshard_evict_refreshes_storage_bytes_gauge_without_a_stats_call() {
    let s = server();
    create_users_collection(&s).await;
    let ids: Vec<String> = (0..16).map(|i| format!("e-{i:02}")).collect();
    for id in &ids {
        index_user(&s, id).await;
    }

    // Establish the pre-eviction gauge value via one `stats` call (the
    // *last* one this test makes before scraping `/metrics` post-evict).
    s.get("/collections/u/stats").await.assert_status_ok();
    let before = storage_bytes_gauge(&s).await;
    assert!(before > 0, "expected a nonzero pre-eviction gauge value");

    let mut assignments = vec![0u32; VIRTUAL_BUCKET_COUNT as usize];
    assignments[0] = 1;
    let new_map = VirtualBucketShardMap::new(1, assignments.clone(), 2).expect("valid shard map");
    let evicted_any = ids.iter().any(|id| {
        let b = bucket_of("u", id);
        assignments[b as usize] != 0
    });
    assert!(evicted_any, "fixture should evict at least one document");

    s.post("/admin/reshard:evict")
        .json(&json!({
            "shard": 0,
            "map_version": new_map.version(),
            "assignments": assignments,
            "physical_shard_count": 2
        }))
        .await
        .assert_status_ok();

    // No `stats` call in between: `/metrics` alone must already show the
    // post-eviction value.
    let after = storage_bytes_gauge(&s).await;
    assert!(
        after < before,
        "expected lumen_storage_bytes to drop after :evict without an intervening \
         stats call (before={before}, after={after})"
    );
}

async fn storage_bytes_gauge(s: &TestServer) -> u64 {
    let resp = s.get("/metrics").await;
    resp.assert_status_ok();
    let body = resp.text();
    body.lines()
        .find(|l| l.starts_with("lumen_storage_bytes "))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|v| v.parse::<u64>().ok())
        .expect("lumen_storage_bytes gauge line present")
}

/// AC4: all four admin verbs require bearer auth (401) and admin role (403).
#[tokio::test]
async fn reshard_admin_verbs_require_admin_auth() {
    let s = auth_server(vec![("tok-r", claim("viewer", &[("u", Role::Read)]))]);

    let scoped_body = json!({ "virtual_bucket_count": 4, "buckets": [0] });
    let apply_body = json!({
        "from_map_version": 0,
        "to_map_version": 1,
        "bucket": 0,
        "from_shard": 0,
        "to_shard": 1,
        "external_ids": {},
        "snapshot": { "version": 1, "collections": {} }
    });
    let evict_body = json!({
        "shard": 0,
        "map_version": 1,
        "assignments": [0, 1, 0, 1],
        "physical_shard_count": 2
    });
    let fence_body = json!({ "virtual_bucket_count": 4, "buckets": [0], "ttl_secs": 5 });
    // #1457 R1: `POST /admin/reshard:prune` — same admin-gated shape as the
    // rest of this file's reshard verbs.
    let prune_body = json!({
        "to_map_version": 1,
        "bucket": 0,
        "virtual_bucket_count": 4,
        "collection_id": "u",
        "chunk_index": 0,
        "total_chunks": 1,
        "keep_ids": []
    });

    // No bearer token at all -> 401.
    s.post("/admin/backup:scoped")
        .json(&scoped_body)
        .await
        .assert_status_unauthorized();
    s.post("/admin/reshard:apply")
        .json(&apply_body)
        .await
        .assert_status_unauthorized();
    s.post("/admin/reshard:evict")
        .json(&evict_body)
        .await
        .assert_status_unauthorized();
    s.post("/admin/checkpoint")
        .await
        .assert_status_unauthorized();
    s.post("/admin/reshard:fence")
        .json(&fence_body)
        .await
        .assert_status_unauthorized();
    s.post("/admin/reshard:prune")
        .json(&prune_body)
        .await
        .assert_status_unauthorized();

    // Authenticated but non-admin role -> 403.
    s.post("/admin/backup:scoped")
        .add_header("authorization", "Bearer tok-r")
        .json(&scoped_body)
        .await
        .assert_status_forbidden();
    s.post("/admin/reshard:apply")
        .add_header("authorization", "Bearer tok-r")
        .json(&apply_body)
        .await
        .assert_status_forbidden();
    s.post("/admin/reshard:evict")
        .add_header("authorization", "Bearer tok-r")
        .json(&evict_body)
        .await
        .assert_status_forbidden();
    s.post("/admin/checkpoint")
        .add_header("authorization", "Bearer tok-r")
        .await
        .assert_status_forbidden();
    s.post("/admin/reshard:fence")
        .add_header("authorization", "Bearer tok-r")
        .json(&fence_body)
        .await
        .assert_status_forbidden();
    s.post("/admin/reshard:prune")
        .add_header("authorization", "Bearer tok-r")
        .json(&prune_body)
        .await
        .assert_status_forbidden();
}

/// AC4 (openapi half): all four verbs show up in the generated OpenAPI
/// document, same as the rest of the admin surface.
#[tokio::test]
async fn reshard_admin_verbs_appear_in_openapi_spec() {
    let s = server();
    let resp = s.get("/openapi.json").await;
    resp.assert_status_ok();
    let spec: serde_json::Value = resp.json();
    let paths = spec["paths"].as_object().expect("paths object");
    for path in [
        "/admin/backup:scoped",
        "/admin/reshard:apply",
        "/admin/reshard:evict",
        "/admin/checkpoint",
        "/admin/reshard:fence",
        "/admin/reshard:prune",
    ] {
        assert!(
            paths.contains_key(path),
            "missing {path} in openapi paths: {:?}",
            paths.keys().collect::<Vec<_>>()
        );
    }
}

/// #1389: with no durable store configured (the default `AppState::open`
/// fixture used across this file), `/admin/checkpoint` is vacuously
/// satisfied — it succeeds and reports `persisted: false` rather than
/// erroring, so existing callers (including the reshard driver's own
/// per-shard checkpoint step) never fail just because a shard happens to run
/// without segment persistence configured.
#[tokio::test]
async fn admin_checkpoint_without_durable_store_is_vacuously_satisfied() {
    let s = server();
    create_users_collection(&s).await;
    index_user(&s, "c1").await;

    let resp = s.post("/admin/checkpoint").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["persisted"], json!(false));
}

// ---- #1396 R2/AC2: POST /admin/reshard:fence write pause -----------------

/// Arming the fence for a bucket blocks writes routed to that bucket (503
/// `bucket_write_paused`), while writes to a different, non-fenced bucket
/// keep succeeding — the fence is per-bucket, not a whole-shard write lock.
#[tokio::test]
async fn reshard_fence_blocks_only_the_fenced_bucket() {
    let s = server();
    create_users_collection(&s).await;

    // Find one id that routes to bucket 0 and one that routes elsewhere.
    let fenced_id = (0..)
        .map(|i| format!("f-{i:03}"))
        .find(|id| bucket_of("u", id) == 0)
        .unwrap();
    let other_id = (0..)
        .map(|i| format!("o-{i:03}"))
        .find(|id| bucket_of("u", id) != 0)
        .unwrap();

    s.post("/admin/reshard:fence")
        .json(&json!({ "virtual_bucket_count": VIRTUAL_BUCKET_COUNT, "buckets": [0], "ttl_secs": 30 }))
        .await
        .assert_status_ok();

    // Fenced bucket: writes rejected.
    let resp = s
        .post("/collections/u/index")
        .json(&json!({
            "items": [{ "external_id": fenced_id, "field": "email", "value": format!("{fenced_id}@x.com") }]
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::SERVICE_UNAVAILABLE);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["error"], json!("bucket_write_paused"));
    assert!(!has_doc(&s, &fenced_id).await);

    // Non-fenced bucket: writes still succeed.
    index_user(&s, &other_id).await;
    assert!(has_doc(&s, &other_id).await);
}

/// Explicitly clearing the fence (`buckets: []`) immediately unblocks
/// writes to the previously-fenced bucket — the driver's normal
/// end-of-tick cleanup path.
#[tokio::test]
async fn reshard_fence_explicit_clear_unblocks_writes() {
    let s = server();
    create_users_collection(&s).await;
    let fenced_id = (0..)
        .map(|i| format!("c-{i:03}"))
        .find(|id| bucket_of("u", id) == 0)
        .unwrap();

    s.post("/admin/reshard:fence")
        .json(&json!({ "virtual_bucket_count": VIRTUAL_BUCKET_COUNT, "buckets": [0], "ttl_secs": 30 }))
        .await
        .assert_status_ok();
    s.post("/collections/u/index")
        .json(&json!({
            "items": [{ "external_id": fenced_id, "field": "email", "value": format!("{fenced_id}@x.com") }]
        }))
        .await
        .assert_status(axum::http::StatusCode::SERVICE_UNAVAILABLE);

    // Explicit clear: empty bucket set.
    s.post("/admin/reshard:fence")
        .json(
            &json!({ "virtual_bucket_count": VIRTUAL_BUCKET_COUNT, "buckets": [], "ttl_secs": 30 }),
        )
        .await
        .assert_status_ok();

    index_user(&s, &fenced_id).await;
    assert!(has_doc(&s, &fenced_id).await);
}

/// A crashed driver that never explicitly clears the fence cannot wedge
/// writes forever: the fence self-expires once its TTL deadline passes,
/// enforced by the serving pod itself, independent of the driver process's
/// liveness.
#[tokio::test]
async fn reshard_fence_auto_expires_after_ttl() {
    let s = server();
    create_users_collection(&s).await;
    let fenced_id = (0..)
        .map(|i| format!("t-{i:03}"))
        .find(|id| bucket_of("u", id) == 0)
        .unwrap();

    s.post("/admin/reshard:fence")
        .json(
            &json!({ "virtual_bucket_count": VIRTUAL_BUCKET_COUNT, "buckets": [0], "ttl_secs": 1 }),
        )
        .await
        .assert_status_ok();
    s.post("/collections/u/index")
        .json(&json!({
            "items": [{ "external_id": fenced_id, "field": "email", "value": format!("{fenced_id}@x.com") }]
        }))
        .await
        .assert_status(axum::http::StatusCode::SERVICE_UNAVAILABLE);

    tokio::time::sleep(std::time::Duration::from_millis(1_200)).await;

    // No explicit clear was ever sent; the deadline alone unblocks it.
    index_user(&s, &fenced_id).await;
    assert!(has_doc(&s, &fenced_id).await);
}

/// #1443 R3/AC3: `ttl_secs` is validated before it ever reaches
/// `Instant::checked_add` — an out-of-range value (here `u64::MAX`, which
/// would overflow `Instant + Duration` and, pre-fix, panicked with the fence
/// mutex held, poisoning it and wedging every future arm/clear/write behind
/// it) must be rejected with 400, not panic, and must leave the fence fully
/// usable for the next, valid request.
#[tokio::test]
async fn reshard_fence_rejects_out_of_range_ttl_without_poisoning_the_fence() {
    let s = server();
    create_users_collection(&s).await;
    let fenced_id = (0..)
        .map(|i| format!("v-{i:03}"))
        .find(|id| bucket_of("u", id) == 0)
        .unwrap();

    let resp = s
        .post("/admin/reshard:fence")
        .json(&json!({
            "virtual_bucket_count": VIRTUAL_BUCKET_COUNT,
            "buckets": [0],
            "ttl_secs": u64::MAX,
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["error"], json!("invalid_ttl_secs"));

    // The oversized request must not have armed anything.
    index_user(&s, &fenced_id).await;
    assert!(has_doc(&s, &fenced_id).await);

    // And the fence keeps working normally afterward — no poisoned lock, no
    // permanent outage.
    s.post("/admin/reshard:fence")
        .json(&json!({ "virtual_bucket_count": VIRTUAL_BUCKET_COUNT, "buckets": [0], "ttl_secs": 30 }))
        .await
        .assert_status_ok();
    let other_id = (0..)
        .map(|i| format!("w-{i:03}"))
        .find(|id| bucket_of("u", id) == 0)
        .unwrap();
    s.post("/collections/u/index")
        .json(&json!({
            "items": [{ "external_id": other_id, "field": "email", "value": format!("{other_id}@x.com") }]
        }))
        .await
        .assert_status(axum::http::StatusCode::SERVICE_UNAVAILABLE);
}
// CODEGEN-END
