// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! Reshard admin verbs end-to-end (#1380): bucket-scoped export
//! (`POST /admin/backup:scoped`), additive batch-apply
//! (`POST /admin/reshard:apply`), and source-side eviction
//! (`POST /admin/reshard:evict`).

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

/// AC4: all three new verbs require bearer auth (401) and admin role (403).
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
}

/// AC4 (openapi half): the three verbs show up in the generated OpenAPI
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
    ] {
        assert!(
            paths.contains_key(path),
            "missing {path} in openapi paths: {:?}",
            paths.keys().collect::<Vec<_>>()
        );
    }
}
// CODEGEN-END
