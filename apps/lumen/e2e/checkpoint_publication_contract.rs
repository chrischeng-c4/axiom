//! Focused checkpoint publication contract.

use std::sync::{Arc, Mutex};
use axum_test::TestServer;
use lumen::aof::AofWriter;
use lumen::api::{router, AppState, CheckpointSink};
use lumen::auth::AuthConfig;
use lumen::coordinator::{SharedAof, WriteCoordinator, WriteSink};
use lumen::segment_checkpoint::SegmentCheckpointSink;
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::Engine;
use lumen::wal::{MemWal, SharedWal};
use serde_json::json;

#[tokio::test]
async fn checkpoint_publishes_a_cold_readable_generation() {
    let dir = tempfile::tempdir().expect("checkpoint root");
    let engine = Arc::new(Engine::new());
    let aof: SharedAof = Arc::new(Mutex::new(AofWriter::open(dir.path().join("aof.log")).expect("AOF")));
    let wal: SharedWal = Arc::new(MemWal::default());
    let writer = WriteCoordinator::start_from_with_aof(wal, engine.clone(), 0, aof.clone());
    let sink_writer: Arc<dyn WriteSink> = writer;
    let store = Arc::new(SegmentRdbStore::new(dir.path().join("segments")).expect("store"));
    let checkpoint: Arc<dyn CheckpointSink> = Arc::new(SegmentCheckpointSink { engine: engine.clone(), store, writer: sink_writer.clone(), aof: Some(aof) });
    let state = AppState::with_components(engine, Arc::new(AuthConfig::open()), sink_writer).with_checkpoint(checkpoint);
    let server = TestServer::new(router(state)).expect("server");
    server.put("/collections/publication").json(&json!({"fields":{"kw":{"type":"keyword"}}})).await.assert_status_ok();
    server.post("/collections/publication/index").json(&json!({"items":[{"external_id":"doc","field":"kw","value":"published"}]})).await.assert_status_ok();
    let response = server.post("/admin/checkpoint").json(&json!({})).await;
    response.assert_status_ok();
    let search = server.post("/collections/publication/search").json(&json!({"query":{"term":{"field":"kw","value":"published"}}})).await;
    search.assert_status_ok();
    assert_eq!(search.json::<serde_json::Value>()["total"], 1);
    let cold = SegmentRdbStore::new(dir.path().join("segments"))
        .expect("cold store")
        .load_current_generation()
        .expect("load CURRENT")
        .expect("published generation");
    let cold_server = TestServer::new(router(AppState::open(cold.engine))).expect("cold server");
    let cold_search = cold_server
        .post("/collections/publication/search")
        .json(&json!({"query":{"term":{"field":"kw","value":"published"}}}))
        .await;
    cold_search.assert_status_ok();
    assert_eq!(cold_search.json::<serde_json::Value>()["total"], 1);
}

#[path = "support/indexing_durable_catalog_fixture.rs"]
mod catalog_fixture;

use catalog_fixture::*;

/// Stage 1 begins with a physical format boundary. A caller writes a small
/// collection, requests the normal checkpoint route, and then reads only the
/// committed generation selected by `CURRENT`. Version 1 contains sequence and
/// predecessor facts only. Version 2 must begin the complete catalog contract.
#[tokio::test]
async fn checkpoint_generation_manifest_is_v2_complete_catalog() {
    let fixture = fixture();
    create_schema(&fixture.server).await;
    post_index(&fixture.server, field_items(0..1, "kw")).await;
    let checkpoint_sequence = fixture.writer.applied_seq();

    checkpoint(&fixture.server).await;

    let current = std::fs::read_to_string(fixture.checkpoint_root.join("CURRENT"))
        .expect("read committed CURRENT pointer");
    let generation = current
        .strip_prefix("generation:")
        .expect("CURRENT names a generation")
        .trim();
    assert!(
        !generation.is_empty(),
        "CURRENT must name the committed checkpoint generation"
    );
    let manifest_path = fixture
        .checkpoint_root
        .join(generation)
        .join("_generation.json");
    let manifest: Value = serde_json::from_slice(
        &std::fs::read(&manifest_path).expect("read committed generation manifest"),
    )
    .expect("decode committed generation manifest");

    assert_eq!(
        manifest["schema_version"],
        json!(2),
        "stage1: a checkpoint after the shipped v1 format must publish the v2 complete catalog"
    );
    assert_eq!(
        manifest["checkpoint_sequence"],
        json!(checkpoint_sequence),
        "v2 catalog must carry the checkpoint watermark"
    );
    assert!(
        manifest["collections"].is_array(),
        "v2 catalog must list every live collection"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn v2_hardlinks_unchanged_collection_retains_old_generation_and_cold_opens_new_complete_state(
) {
    let dir = tempfile::tempdir().expect("hard-link reuse fixture directory");
    let root = dir.path().join("segments");
    let store = SegmentRdbStore::new(&root).expect("open hard-link reuse root");
    let engine = Arc::new(Engine::new());
    let server = TestServer::new(router(AppState::open(engine.clone())))
        .expect("hard-link reuse HTTP server");
    for collection in [STAGE1_REUSE_STABLE, STAGE1_REUSE_CHANGED] {
        stage1_reuse_put_keyword_collection(&server, collection).await;
    }
    stage1_reuse_index(&server, STAGE1_REUSE_STABLE, "stable-1", "stable-v1").await;
    stage1_reuse_index(&server, STAGE1_REUSE_CHANGED, "changed-1", "changed-v1").await;
    stage1_restore_legacy_base(&engine, "hard-link reuse base");
    store.save(&engine, 501).expect("write base generation");

    let base_name = stage1_reuse_current_name(&root);
    let base_generation = root.join(&base_name);
    let base_manifest = stage1_read_manifest(&base_generation);
    let base_stable = stage1_reuse_catalog_collection(&base_manifest, STAGE1_REUSE_STABLE);
    let base_changed = stage1_reuse_catalog_collection(&base_manifest, STAGE1_REUSE_CHANGED);

    stage1_reuse_index(&server, STAGE1_REUSE_CHANGED, "changed-1", "changed-v2").await;
    store.save(&engine, 502).expect("write changed generation");

    let next_name = stage1_reuse_current_name(&root);
    let next_generation = root.join(&next_name);
    let next_manifest = stage1_read_manifest(&next_generation);
    assert_eq!(next_manifest["checkpoint_sequence"], json!(502));
    assert_eq!(next_manifest["previous"], json!(base_name));
    let next_stable = stage1_reuse_catalog_collection(&next_manifest, STAGE1_REUSE_STABLE);
    let next_changed = stage1_reuse_catalog_collection(&next_manifest, STAGE1_REUSE_CHANGED);
    stage1_reuse_assert_hardlinked_collection(
        &base_generation,
        base_stable,
        &next_generation,
        next_stable,
    );
    assert_eq!(
        stage1_reuse_collection_u64(next_stable, "collection_generation"),
        stage1_reuse_collection_u64(base_stable, "collection_generation"),
        "an unchanged collection retains its durable collection generation"
    );
    assert_eq!(
        stage1_reuse_collection_u64(next_stable, "data_version"),
        stage1_reuse_collection_u64(base_stable, "data_version"),
        "an unchanged collection retains its data version"
    );
    assert!(
        stage1_reuse_collection_u64(next_changed, "data_version")
            > stage1_reuse_collection_u64(base_changed, "data_version"),
        "an effective mutation advances only the changed collection data version"
    );

    let (old_engine, old_sequence) = stage1_reuse_cold_load_named(&root, &base_name);
    assert_eq!(
        old_sequence, 501,
        "retained base generation has its original sequence"
    );
    let old_server = TestServer::new(router(AppState::open(old_engine)))
        .expect("retained base cold HTTP server");
    stage1_reuse_assert_term_ids(&old_server, STAGE1_REUSE_STABLE, "stable-v1", &["stable-1"])
        .await;
    stage1_reuse_assert_term_ids(
        &old_server,
        STAGE1_REUSE_CHANGED,
        "changed-v1",
        &["changed-1"],
    )
    .await;

    let (next_engine, next_sequence) = stage1_reuse_cold_load_current(&root);
    assert_eq!(
        next_sequence, 502,
        "new generation has its checkpoint sequence"
    );
    let next_server = TestServer::new(router(AppState::open(next_engine)))
        .expect("new generation cold HTTP server");
    stage1_reuse_assert_term_ids(
        &next_server,
        STAGE1_REUSE_STABLE,
        "stable-v1",
        &["stable-1"],
    )
    .await;
    stage1_reuse_assert_term_ids(&next_server, STAGE1_REUSE_CHANGED, "changed-v1", &[]).await;
    stage1_reuse_assert_term_ids(
        &next_server,
        STAGE1_REUSE_CHANGED,
        "changed-v2",
        &["changed-1"],
    )
    .await;
}

/// A checkpoint with more than one collection must carry forward an unchanged
/// collection when a sibling collection changes. The second server uses only
/// the committed checkpoint selected by `CURRENT`, so the assertions observe
/// the public search API after a cold restart rather than a live engine or a
/// physical generation layout.
#[tokio::test]
async fn public_checkpoint_cold_restart_retains_unchanged_collection_when_sibling_changes() {
    let fixture = fixture();
    for collection in [STAGE1_REUSE_STABLE, STAGE1_REUSE_CHANGED] {
        stage1_reuse_put_keyword_collection(&fixture.server, collection).await;
    }
    stage1_reuse_index(
        &fixture.server,
        STAGE1_REUSE_STABLE,
        "stable-1",
        "stable-v1",
    )
    .await;
    stage1_reuse_index(
        &fixture.server,
        STAGE1_REUSE_CHANGED,
        "changed-1",
        "changed-v1",
    )
    .await;
    checkpoint(&fixture.server).await;

    stage1_reuse_index(
        &fixture.server,
        STAGE1_REUSE_CHANGED,
        "changed-1",
        "changed-v2",
    )
    .await;
    checkpoint(&fixture.server).await;

    let cold = SegmentRdbStore::new(&fixture.checkpoint_root)
        .expect("open checkpoint root for public cold restart")
        .load_current_generation()
        .expect("load CURRENT for public cold restart")
        .expect("second checkpoint must publish a generation");
    let cold_server = TestServer::new(router(AppState::open(cold.engine)))
        .expect("public cold restart HTTP server");

    stage1_reuse_assert_term_ids(
        &cold_server,
        STAGE1_REUSE_STABLE,
        "stable-v1",
        &["stable-1"],
    )
    .await;
    stage1_reuse_assert_term_ids(&cold_server, STAGE1_REUSE_CHANGED, "changed-v1", &[]).await;
    stage1_reuse_assert_term_ids(
        &cold_server,
        STAGE1_REUSE_CHANGED,
        "changed-v2",
        &["changed-1"],
    )
    .await;
}

#[tokio::test]
async fn v2_collection_epoch_never_reuses_after_truncate_force_drop_sweep_recreate_and_restart() {
    let dir = tempfile::tempdir().expect("collection epoch fixture directory");
    let root = dir.path().join("segments");
    let store = SegmentRdbStore::new(&root).expect("open collection epoch root");
    let engine = Arc::new(Engine::new());
    let server = TestServer::new(router(AppState::open(engine.clone())))
        .expect("collection epoch HTTP server");
    stage1_reuse_put_keyword_collection(&server, STAGE1_EPOCH_COLLECTION).await;
    stage1_reuse_index(&server, STAGE1_EPOCH_COLLECTION, "epoch-first", "first").await;
    store
        .save(&engine, 601)
        .expect("write initial epoch generation");
    let initial_manifest = stage1_read_manifest(&stage1_current_generation_dir(&root));
    let initial = stage1_reuse_catalog_collection(&initial_manifest, STAGE1_EPOCH_COLLECTION);
    let first_epoch = stage1_reuse_collection_u64(initial, "collection_generation");
    let schema_version = stage1_reuse_collection_u64(initial, "schema_version");

    server
        .post(&format!(
            "/collections/{STAGE1_EPOCH_COLLECTION}/docs:truncate"
        ))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);
    store
        .save(&engine, 602)
        .expect("write truncated epoch generation");
    let truncated_manifest = stage1_read_manifest(&stage1_current_generation_dir(&root));
    let truncated = stage1_reuse_catalog_collection(&truncated_manifest, STAGE1_EPOCH_COLLECTION);
    let truncated_epoch = stage1_reuse_collection_u64(truncated, "collection_generation");
    assert!(
        truncated_epoch > first_epoch,
        "truncate clears into a new durable collection epoch"
    );

    server
        .delete(&format!(
            "/collections/{STAGE1_EPOCH_COLLECTION}?force=true"
        ))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);
    store
        .save(&engine, 603)
        .expect("write force-drop generation");
    assert!(
        stage1_read_manifest(&stage1_current_generation_dir(&root))["collections"]
            .as_array()
            .expect("force-drop catalog collections")
            .is_empty(),
        "force drop removes the collection from the next complete catalog"
    );
    assert_eq!(
        engine
            .sweep_deleted(Duration::from_millis(0))
            .expect("sweep after force drop"),
        0,
        "force drop leaves no tombstone for a later sweep"
    );

    stage1_reuse_put_keyword_collection(&server, STAGE1_EPOCH_COLLECTION).await;
    stage1_reuse_index(&server, STAGE1_EPOCH_COLLECTION, "epoch-second", "second").await;
    store
        .save(&engine, 604)
        .expect("write force-recreated generation");
    let force_recreated_manifest = stage1_read_manifest(&stage1_current_generation_dir(&root));
    let second_epoch = stage1_reuse_collection_u64(
        stage1_reuse_catalog_collection(&force_recreated_manifest, STAGE1_EPOCH_COLLECTION),
        "collection_generation",
    );
    assert!(
        second_epoch > truncated_epoch,
        "force-drop recreate must allocate after the truncate epoch"
    );

    server
        .delete(&format!("/collections/{STAGE1_EPOCH_COLLECTION}"))
        .await
        .assert_status(axum::http::StatusCode::ACCEPTED);
    tokio::time::sleep(Duration::from_millis(2)).await;
    assert_eq!(
        engine
            .sweep_deleted(Duration::from_millis(1))
            .expect("sweep soft-deleted collection"),
        1,
        "the second recreate must follow a physical sweep"
    );
    store.save(&engine, 605).expect("write swept generation");
    stage1_reuse_put_keyword_collection(&server, STAGE1_EPOCH_COLLECTION).await;
    stage1_reuse_index(&server, STAGE1_EPOCH_COLLECTION, "epoch-third", "third").await;
    store
        .save(&engine, 606)
        .expect("write swept-recreated generation");
    let final_manifest = stage1_read_manifest(&stage1_current_generation_dir(&root));
    let final_collection =
        stage1_reuse_catalog_collection(&final_manifest, STAGE1_EPOCH_COLLECTION);
    let third_epoch = stage1_reuse_collection_u64(final_collection, "collection_generation");
    assert!(
        third_epoch > second_epoch && third_epoch > truncated_epoch,
        "no recreate path may reuse an observed collection epoch"
    );
    assert_eq!(
        stage1_reuse_collection_u64(final_collection, "schema_version"),
        schema_version,
        "same-schema recreate does not fabricate a schema edit"
    );
    let observed_epoch_max = first_epoch
        .max(truncated_epoch)
        .max(second_epoch)
        .max(third_epoch);
    assert!(
        final_manifest["next_collection_generation"]
            .as_u64()
            .expect("durable collection-generation allocator")
            > observed_epoch_max,
        "allocator cursor stays above every issued collection epoch"
    );

    let (cold_engine, cold_sequence) = stage1_reuse_cold_load_current(&root);
    assert_eq!(
        cold_sequence, 606,
        "restart selects the final recreated generation"
    );
    let cold_server = TestServer::new(router(AppState::open(cold_engine.clone())))
        .expect("epoch restart HTTP server");
    stage1_reuse_assert_term_ids(&cold_server, STAGE1_EPOCH_COLLECTION, "first", &[]).await;
    stage1_reuse_assert_term_ids(&cold_server, STAGE1_EPOCH_COLLECTION, "second", &[]).await;
    stage1_reuse_assert_term_ids(
        &cold_server,
        STAGE1_EPOCH_COLLECTION,
        "third",
        &["epoch-third"],
    )
    .await;

    stage1_reuse_put_keyword_collection(&cold_server, STAGE1_EPOCH_AFTER_RESTART).await;
    stage1_reuse_index(
        &cold_server,
        STAGE1_EPOCH_AFTER_RESTART,
        "epoch-after-restart",
        "after-restart",
    )
    .await;
    let restart_store = SegmentRdbStore::new(&root).expect("reopen allocator after restart");
    restart_store
        .save(&cold_engine, 607)
        .expect("write post-restart allocation generation");
    let post_restart_manifest = stage1_read_manifest(&stage1_current_generation_dir(&root));
    let post_restart_epoch = stage1_reuse_collection_u64(
        stage1_reuse_catalog_collection(&post_restart_manifest, STAGE1_EPOCH_AFTER_RESTART),
        "collection_generation",
    );
    assert!(
        post_restart_epoch > observed_epoch_max,
        "the allocator must continue above all epochs after cold restart"
    );
    assert!(
        post_restart_manifest["next_collection_generation"]
            .as_u64()
            .expect("post-restart allocator cursor")
            > post_restart_epoch,
        "post-restart allocation advances the durable allocator cursor"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn v2_save_required_fresh_engine_same_versions_never_reuses_other_data() {
    let dir = tempfile::tempdir().expect("fresh-engine provenance fixture directory");
    let root = dir.path().join("segments");
    let store = SegmentRdbStore::new(&root).expect("open fresh-engine provenance root");

    let original = Arc::new(Engine::new());
    let original_server = TestServer::new(router(AppState::open(original.clone())))
        .expect("original provenance HTTP server");
    stage1_reuse_put_keyword_collection(&original_server, STAGE1_PROVENANCE_COLLECTION).await;
    stage1_reuse_index(
        &original_server,
        STAGE1_PROVENANCE_COLLECTION,
        "provenance-old",
        "old",
    )
    .await;
    store
        .save_required(&original, 701)
        .expect("write original required generation");
    let original_name = stage1_reuse_current_name(&root);
    let original_generation = root.join(&original_name);
    let original_manifest = stage1_read_manifest(&original_generation);

    let fresh = Arc::new(Engine::new());
    let fresh_server = TestServer::new(router(AppState::open(fresh.clone())))
        .expect("fresh provenance HTTP server");
    stage1_reuse_put_keyword_collection(&fresh_server, STAGE1_PROVENANCE_COLLECTION).await;
    stage1_reuse_index(
        &fresh_server,
        STAGE1_PROVENANCE_COLLECTION,
        "provenance-new",
        "new",
    )
    .await;
    store
        .save_required(&fresh, 701)
        .expect("write fresh required generation at the same sequence");
    let fresh_name = stage1_reuse_current_name(&root);
    let fresh_generation = root.join(&fresh_name);
    let fresh_manifest = stage1_read_manifest(&fresh_generation);
    assert_ne!(
        original_name, fresh_name,
        "save_required publishes a new immutable revision"
    );
    let original_collection =
        stage1_reuse_catalog_collection(&original_manifest, STAGE1_PROVENANCE_COLLECTION);
    let fresh_collection =
        stage1_reuse_catalog_collection(&fresh_manifest, STAGE1_PROVENANCE_COLLECTION);
    assert_eq!(
        stage1_reuse_collection_u64(original_collection, "schema_version"),
        stage1_reuse_collection_u64(fresh_collection, "schema_version"),
        "the regression fixture uses identical schema versions"
    );
    assert_eq!(
        stage1_reuse_collection_u64(original_collection, "data_version"),
        stage1_reuse_collection_u64(fresh_collection, "data_version"),
        "the regression fixture uses identical data versions"
    );
    stage1_reuse_assert_not_hardlinked_collection(
        &original_generation,
        original_collection,
        &fresh_generation,
        fresh_collection,
    );

    let (old_engine, old_sequence) = stage1_reuse_cold_load_named(&root, &original_name);
    assert_eq!(
        old_sequence, 701,
        "original required generation keeps its sequence"
    );
    let old_server = TestServer::new(router(AppState::open(old_engine)))
        .expect("original provenance cold HTTP server");
    stage1_reuse_assert_term_ids(
        &old_server,
        STAGE1_PROVENANCE_COLLECTION,
        "old",
        &["provenance-old"],
    )
    .await;

    let (fresh_engine, fresh_sequence) = stage1_reuse_cold_load_current(&root);
    assert_eq!(
        fresh_sequence, 701,
        "fresh required revision keeps its sequence"
    );
    let fresh_server = TestServer::new(router(AppState::open(fresh_engine)))
        .expect("fresh provenance cold HTTP server");
    stage1_reuse_assert_term_ids(&fresh_server, STAGE1_PROVENANCE_COLLECTION, "old", &[]).await;
    stage1_reuse_assert_term_ids(
        &fresh_server,
        STAGE1_PROVENANCE_COLLECTION,
        "new",
        &["provenance-new"],
    )
    .await;
}
mod published_checkpoint_overlay_release {
    //! # Facets
    //!
    //! - Behavior: `apps/lumen/e2e/indexing_durable_oracle.rs:5338` asserts
    //!   public IDs; `:5522` compares live/cold snapshots; calls at `:5540`,
    //!   `:5544`, `:5552`, and `:5556` cover both checkpoint layers.
    //! - Security: `apps/lumen/src/segment_rdb.rs:500` starts the persisted
    //!   generation read; `:2543` and `:2569` mutate LocalRows; `:1138` and
    //!   `:1145` assert refusal and unchanged `CURRENT`. This release-only
    //!   path adds no caller-controlled byte, path, or identifier input.
    //! - Performance: `:5505` measures actual forward/token driver size;
    //!   `:5546`, `:5547`, `:5558`, and `:5559` require zero after durable publish.
    //!   This case does not claim latency/RSS. The user-approved #4246 stage6
    //!   budget is 2.5 CPU and 16 GiB for 30 minutes, 100 mixed doc ops/s at >=95%,
    //!   10 QPS p99 <=1s, every query <=5s, zero errors, and RSS <=12 GiB.
    //!   This bounded structural check does not run that workload.

    use super::*;

    const FIELDS: [&str; 5] = ["kw", "num", "tags", "sig", "body"];
    const UPDATED_ID: &str = "overlay-doc-000";
    const DELETED_ID: &str = "overlay-doc-001";
    const UNTOUCHED_ID: &str = "overlay-doc-002";
    const APPENDED_ID: &str = "overlay-doc-003";

    async fn create_all_field_schema(server: &TestServer) {
        create_schema(server).await;
        server
            .put("/collections/docs")
            .json(&json!({
                "fields": {
                    "tags": { "type": "set" },
                    "sig": { "type": "hash" }
                }
            }))
            .await
            .assert_status_ok();
    }

    async fn index_document(
        server: &TestServer,
        external_id: &str,
        keyword: &str,
        number: f64,
        tags: &[&str],
        hash: &str,
        body: &str,
    ) {
        server
            .post("/collections/docs/index")
            .json(&json!({
                "items": [
                    {
                        "external_id": external_id,
                        "field": "kw",
                        "value": keyword,
                    },
                    {
                        "external_id": external_id,
                        "field": "num",
                        "value": number,
                    },
                    {
                        "external_id": external_id,
                        "field": "tags",
                        "value": tags,
                    },
                    {
                        "external_id": external_id,
                        "field": "sig",
                        "value": hash,
                    },
                    {
                        "external_id": external_id,
                        "field": "body",
                        "value": body,
                    }
                ]
            }))
            .await
            .assert_status_ok();
    }

    async fn index_base_documents(server: &TestServer) {
        index_document(
            server,
            UPDATED_ID,
            "overlay-kw-base-0",
            1_000.0,
            &["overlay-tag-base-0"],
            "000000000000a000",
            "overlay-text-base-0 shared",
        )
        .await;
        index_document(
            server,
            DELETED_ID,
            "overlay-kw-base-1",
            1_001.0,
            &["overlay-tag-base-1"],
            "000000000000a001",
            "overlay-text-base-1 shared",
        )
        .await;
        index_document(
            server,
            UNTOUCHED_ID,
            "overlay-kw-base-2",
            1_002.0,
            &["overlay-tag-base-2"],
            "000000000000a002",
            "overlay-text-base-2 shared",
        )
        .await;
    }

    async fn apply_first_delta(server: &TestServer) {
        index_document(
            server,
            UPDATED_ID,
            "overlay-kw-first",
            9_001.0,
            &["overlay-tag-first"],
            "000000000000f001",
            "overlay-text-first shared",
        )
        .await;
        server
            .delete(&format!("/collections/docs/docs/{DELETED_ID}"))
            .await
            .assert_status(axum::http::StatusCode::NO_CONTENT);
        index_document(
            server,
            APPENDED_ID,
            "overlay-kw-appended",
            9_003.0,
            &["overlay-tag-appended"],
            "000000000000f003",
            "overlay-text-appended shared",
        )
        .await;
    }

    async fn apply_second_delta(server: &TestServer) {
        index_document(
            server,
            UPDATED_ID,
            "overlay-kw-final",
            9_002.0,
            &["overlay-tag-final"],
            "000000000000f002",
            "overlay-text-final shared",
        )
        .await;
    }

    async fn search_ids(server: &TestServer, query: Value, context: &str) -> Vec<String> {
        let body = http_search(server, query, context).await;
        let mut ids: Vec<String> = hit_ids(&body).into_iter().map(|id| id.to_owned()).collect();
        ids.sort();
        assert_eq!(
            body["total"].as_u64(),
            Some(ids.len() as u64),
            "{context}: total must equal the complete small fixture result: {body}"
        );
        ids
    }

    async fn assert_query(server: &TestServer, query: Value, expected: &[&str], context: &str) {
        let mut expected: Vec<String> = expected.iter().map(|id| (*id).to_owned()).collect();
        expected.sort();
        assert_eq!(
            search_ids(server, query, context).await,
            expected,
            "{context}: public search result"
        );
    }

    async fn assert_first_semantics(server: &TestServer, phase: &str) {
        assert_query(
            server,
            json!({ "term": { "field": "kw", "value": "overlay-kw-first" } }),
            &[UPDATED_ID],
            &format!("{phase}: updated Keyword"),
        )
        .await;
        assert_query(
            server,
            json!({ "term": { "field": "kw", "value": "overlay-kw-base-0" } }),
            &[],
            &format!("{phase}: old Keyword is masked"),
        )
        .await;
        assert_query(
            server,
            json!({ "term": { "field": "kw", "value": "overlay-kw-base-1" } }),
            &[],
            &format!("{phase}: deleted document remains masked"),
        )
        .await;
        assert_query(
            server,
            json!({ "range": {
                "field": "num",
                "gte": 9_001.0,
                "lte": 9_001.0,
            }}),
            &[UPDATED_ID],
            &format!("{phase}: updated Number"),
        )
        .await;
        assert_query(
            server,
            json!({ "term": { "field": "tags", "value": "overlay-tag-first" } }),
            &[UPDATED_ID],
            &format!("{phase}: updated Set"),
        )
        .await;
        assert_query(
            server,
            json!({ "hamming": {
                "field": "sig",
                "hash": "000000000000f001",
                "max_distance": 0,
            }}),
            &[UPDATED_ID],
            &format!("{phase}: updated Hash"),
        )
        .await;
        assert_query(
            server,
            json!({ "match": {
                "field": "body",
                "text": "overlay-text-first",
                "op": "and",
            }}),
            &[UPDATED_ID],
            &format!("{phase}: updated Text"),
        )
        .await;
        assert_query(
            server,
            json!({ "term": { "field": "kw", "value": "overlay-kw-appended" } }),
            &[APPENDED_ID],
            &format!("{phase}: appended document"),
        )
        .await;
    }

    async fn assert_second_semantics(server: &TestServer, phase: &str) {
        assert_query(
            server,
            json!({ "term": { "field": "kw", "value": "overlay-kw-final" } }),
            &[UPDATED_ID],
            &format!("{phase}: newest Keyword"),
        )
        .await;
        assert_query(
            server,
            json!({ "term": { "field": "kw", "value": "overlay-kw-first" } }),
            &[],
            &format!("{phase}: first Keyword does not resurrect"),
        )
        .await;
        assert_query(
            server,
            json!({ "range": {
                "field": "num",
                "gte": 9_002.0,
                "lte": 9_002.0,
            }}),
            &[UPDATED_ID],
            &format!("{phase}: newest Number"),
        )
        .await;
        assert_query(
            server,
            json!({ "term": { "field": "tags", "value": "overlay-tag-final" } }),
            &[UPDATED_ID],
            &format!("{phase}: newest Set"),
        )
        .await;
        assert_query(
            server,
            json!({ "hamming": {
                "field": "sig",
                "hash": "000000000000f002",
                "max_distance": 0,
            }}),
            &[UPDATED_ID],
            &format!("{phase}: newest Hash"),
        )
        .await;
        assert_query(
            server,
            json!({ "match": {
                "field": "body",
                "text": "overlay-text-final",
                "op": "and",
            }}),
            &[UPDATED_ID],
            &format!("{phase}: newest Text"),
        )
        .await;
        assert_query(
            server,
            json!({ "term": { "field": "kw", "value": "overlay-kw-base-1" } }),
            &[],
            &format!("{phase}: deleted document stays masked"),
        )
        .await;
    }

    fn assert_probe_observes_live_delta(engine: &Arc<Engine>, phase: &str) {
        for field in FIELDS {
            let (driver_len, has_segment) = engine
                .segment_field_probe(COLLECTION, field)
                .expect("all tested overlay fields remain probeable");
            assert!(
                has_segment,
                "{phase}: {field} must retain its published base segment while a delta is live"
            );
            assert!(
                driver_len > 0,
                "{phase}: {field} probe must see the test's live overlay before publication"
            );
        }
    }

    fn assert_probe_is_empty(engine: &Arc<Engine>, phase: &str) {
        for field in FIELDS {
            let (driver_len, has_segment) = engine
                .segment_field_probe(COLLECTION, field)
                .expect("all tested overlay fields remain probeable");
            assert!(
                has_segment,
                "{phase}: {field} must retain an attached immutable segment after publication"
            );
            assert_eq!(
                driver_len, 0,
                "{phase}: published {field} delta must not remain in the live forward/token driver"
            );
        }
    }

    fn load_current_engine(fixture: &Fixture) -> Arc<Engine> {
        fixture
            .store
            .load_current_generation()
            .expect("load checkpoint CURRENT")
            .expect("published checkpoint generation")
            .engine
    }

    fn assert_snapshot_matches_cold(live: &Arc<Engine>, cold: &Arc<Engine>, phase: &str) {
        assert_eq!(
            digest(live),
            digest(cold),
            "{phase}: public snapshots must agree across the published recovery cut"
        );
    }

    #[tokio::test]
    async fn published_incremental_checkpoints_release_scalar_and_text_overlays_live_and_cold() {
        let fixture = fixture();
        create_all_field_schema(&fixture.server).await;
        index_base_documents(&fixture.server).await;
        checkpoint(&fixture.server).await;
        assert_probe_is_empty(&fixture.engine, "sealed base");

        apply_first_delta(&fixture.server).await;
        assert_probe_observes_live_delta(&fixture.engine, "first live delta");
        checkpoint(&fixture.server).await;
        assert_first_semantics(&fixture.server, "first live checkpoint").await;
        let first_cold = load_current_engine(&fixture);
        let first_cold_server = TestServer::new(router(AppState::open(first_cold.clone())))
            .expect("first cold query server");
        assert_first_semantics(&first_cold_server, "first cold checkpoint").await;
        assert_snapshot_matches_cold(&fixture.engine, &first_cold, "first checkpoint");
        assert_probe_is_empty(&fixture.engine, "first live checkpoint");
        assert_probe_is_empty(&first_cold, "first cold checkpoint");

        apply_second_delta(&fixture.server).await;
        assert_probe_observes_live_delta(&fixture.engine, "second live delta");
        checkpoint(&fixture.server).await;
        assert_second_semantics(&fixture.server, "second live checkpoint").await;
        let second_cold = load_current_engine(&fixture);
        let second_cold_server = TestServer::new(router(AppState::open(second_cold.clone())))
            .expect("second cold query server");
        assert_second_semantics(&second_cold_server, "second cold checkpoint").await;
        assert_snapshot_matches_cold(&fixture.engine, &second_cold, "second checkpoint");
        assert_probe_is_empty(&fixture.engine, "second live checkpoint");
        assert_probe_is_empty(&second_cold, "second cold checkpoint");
    }
}

mod first_sparse_checkpoint_contract {
    //! # Facets
    //!
    //! - Behavior: the assertions at
    //!   indexing_durable_oracle.rs:11854-12024 create one fresh seven-field
    //!   collection, asserts each zero-row base and first sparse delta, writes
    //!   during real checkpoint file I/O, and checks live and cold results. It
    //!   covers the first-capture changes in apps/lumen/src/storage.rs:13668-13691
    //!   and apps/lumen/src/segment_rdb.rs:540-604.
    //! - Security: apps/lumen/src/segment_rdb.rs:1910-2070 validates the
    //!   persisted catalog and local-row bytes this path reads. Existing
    //!   v2_current_refuses_keyword_delta_local_rows_count_mismatch and
    //!   v2_current_refuses_keyword_delta_duplicate_stable_local_rows at
    //!   indexing_durable_oracle.rs:2605-2656 feed the shared
    //!   lumen-local-eids-cbor-v1 boundary malformed bytes and require refusal
    //!   without changing CURRENT. This valid-generation case carries behavior.
    //! - Performance: apps/lumen/ROADMAP.md:60-70 promises a 256 MiB total
    //!   active/frozen/reserved budget and checkpoint progress at that limit.
    //!   indexing_durable_oracle.rs:10714-10941 already measures that budget.
    //!   The zero-row-base and bounded-local-row assertions below are
    //!   structural only. They make no latency or RSS claim.

    use super::*;

    use lumen::segment_checkpoint::SegmentCheckpointSink;
    use lumen::segment_rdb::{MergeObserver, MergePhase};
    use std::collections::BTreeSet;
    use std::io;
    use std::sync::{mpsc, Mutex};
    use storage_durable::{CommitStep, FailureInjector, FailurePoint};

    const COLLECTION: &str = "first-sparse-all-fields";
    const KW: &str = "kw";
    const NUM: &str = "num";
    const SET: &str = "tags";
    const HASH: &str = "sig";
    const TEXT: &str = "body";
    const FLAT: &str = "flat";
    const HNSW: &str = "hnsw";

    const LIVE: &str = "first-sparse-live";
    const DELETED: &str = "first-sparse-deleted";
    const REPLACED: &str = "first-sparse-replaced";
    const EMPTY: &str = "first-sparse-empty";
    const CONCURRENT: &str = "first-sparse-concurrent";
    const INITIAL_ROWS: u64 = 4;

    #[derive(Default)]
    struct NoopMergeObserver;

    impl MergeObserver for NoopMergeObserver {
        fn observe(&self, _: MergePhase) -> io::Result<()> {
            Ok(())
        }
    }

    /// The real sync hook fires after capture. SegmentCheckpointSink offloads
    /// its save, so the test can safely drive HTTP while this hook is held.
    #[derive(Default)]
    struct HoldNextSyncFile {
        hold: Mutex<Option<(mpsc::SyncSender<()>, mpsc::Receiver<()>)>>,
    }

    impl HoldNextSyncFile {
        fn arm(&self, entered: mpsc::SyncSender<()>, release: mpsc::Receiver<()>) {
            assert!(
                self.hold
                    .lock()
                    .expect("first sparse sync hold mutex")
                    .replace((entered, release))
                    .is_none(),
                "first sparse fixture arms one checkpoint sync hold",
            );
        }
    }

    impl FailureInjector for HoldNextSyncFile {
        fn check(&self, point: &FailurePoint) -> io::Result<()> {
            if point.step != CommitStep::SyncFile {
                return Ok(());
            }
            let Some((entered, release)) = self
                .hold
                .lock()
                .expect("first sparse sync hold mutex")
                .take()
            else {
                return Ok(());
            };
            entered.send(()).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "first sparse checkpoint readiness receiver dropped",
                )
            })?;
            release.recv().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "first sparse checkpoint release sender dropped",
                )
            })?;
            Ok(())
        }
    }

    /// Every failure path releases the blocking SyncFile callback.
    struct SyncRelease(Option<mpsc::SyncSender<()>>);

    impl SyncRelease {
        fn release(&mut self) {
            if let Some(sender) = self.0.take() {
                let _ = sender.send(());
            }
        }
    }

    impl Drop for SyncRelease {
        fn drop(&mut self) {
            self.release();
        }
    }

    struct Fixture {
        _dir: tempfile::TempDir,
        root: PathBuf,
        server: TestServer,
        store: Arc<SegmentRdbStore>,
        checkpoint: Arc<SegmentCheckpointSink>,
        writer: Arc<WriteCoordinator>,
        hold: Arc<HoldNextSyncFile>,
    }

    fn fixture() -> Fixture {
        let dir = tempfile::tempdir().expect("first sparse fixture directory");
        let root = dir.path().join("segments");
        let hold = Arc::new(HoldNextSyncFile::default());
        let store = Arc::new(
            SegmentRdbStore::with_failure_injector_and_merge_observer(
                &root,
                hold.clone(),
                Arc::new(NoopMergeObserver),
            )
            .expect("open first sparse segment store"),
        );
        let aof: SharedAof = Arc::new(Mutex::new(
            AofWriter::open(dir.path().join("aof.log")).expect("open first sparse AOF"),
        ));
        let engine = Arc::new(Engine::new());
        let wal: SharedWal = Arc::new(MemWal::new());
        let writer = WriteCoordinator::start_from_with_aof(wal, engine.clone(), 0, aof.clone());
        let sink_writer: Arc<dyn WriteSink> = writer.clone();
        let checkpoint = Arc::new(SegmentCheckpointSink {
            engine: engine.clone(),
            store: store.clone(),
            writer: sink_writer.clone(),
            aof: Some(aof),
        });
        let checkpoint_api: Arc<dyn CheckpointSink> = checkpoint.clone();
        let state = AppState::with_components(engine, Arc::new(AuthConfig::open()), sink_writer)
            .with_checkpoint(checkpoint_api);
        Fixture {
            _dir: dir,
            root,
            server: TestServer::new(router(state)).expect("first sparse HTTP server"),
            store,
            checkpoint,
            writer,
            hold,
        }
    }

    fn vector(seed: f64) -> Value {
        json!([
            seed,
            seed * 0.37 + 0.11,
            seed * -0.23 + 0.07,
            seed * 0.19 - 0.13,
        ])
    }

    async fn create_collection(server: &TestServer) {
        server
            .put(&format!("/collections/{COLLECTION}"))
            .json(&json!({ "fields": {
                KW: { "type": "keyword" },
                NUM: { "type": "number" },
                SET: { "type": "set" },
                HASH: { "type": "hash" },
                TEXT: { "type": "text", "analyzer": "whitespace_lower" },
                FLAT: { "type": "vector", "dim": 4, "metric": "l2", "backend": "flat-cpu" },
                HNSW: { "type": "vector", "dim": 4, "metric": "l2", "backend": "hnsw-cpu" },
            }}))
            .await
            .assert_status_ok();
    }

    async fn index_full(
        server: &TestServer,
        id: &str,
        keyword: &str,
        number: f64,
        tag: &str,
        hash: &str,
        text: &str,
        flat: f64,
        hnsw: f64,
    ) {
        server
            .post(&format!("/collections/{COLLECTION}/index"))
            .json(&json!({ "items": [
                { "external_id": id, "field": KW, "value": keyword },
                { "external_id": id, "field": NUM, "value": number },
                { "external_id": id, "field": SET, "value": [tag] },
                { "external_id": id, "field": HASH, "value": hash },
                { "external_id": id, "field": TEXT, "value": text },
                { "external_id": id, "field": FLAT, "value": vector(flat) },
                { "external_id": id, "field": HNSW, "value": vector(hnsw) },
            ] }))
            .await
            .assert_status_ok();
    }

    async fn replace_without_set(
        server: &TestServer,
        id: &str,
        keyword: &str,
        number: f64,
        hash: &str,
        text: &str,
        flat: f64,
        hnsw: f64,
    ) {
        server
            .put(&format!("/collections/{COLLECTION}/docs:replace"))
            .json(&json!({ "docs": [{
                "external_id": id,
                "fields": {
                    KW: keyword,
                    NUM: number,
                    HASH: hash,
                    TEXT: text,
                    FLAT: vector(flat),
                    HNSW: vector(hnsw),
                },
            }]}))
            .await
            .assert_status_ok();
    }

    async fn replace_empty(server: &TestServer, id: &str) {
        server
            .put(&format!("/collections/{COLLECTION}/docs:replace"))
            .json(&json!({ "docs": [{ "external_id": id, "fields": {} }] }))
            .await
            .assert_status_ok();
    }

    async fn search_ids(server: &TestServer, query: Value) -> Vec<String> {
        let response = server
            .post(&format!("/collections/{COLLECTION}/search"))
            .json(&json!({ "query": query, "limit": 32, "track_total": true }))
            .await;
        response.assert_status_ok();
        let body: Value = response.json();
        let hits = body["hits"].as_array().expect("first sparse search hits");
        let mut ids: Vec<_> = hits
            .iter()
            .map(|hit| {
                hit["external_id"]
                    .as_str()
                    .expect("first sparse hit external ID")
                    .to_owned()
            })
            .collect();
        ids.sort();
        ids
    }

    async fn assert_ids(server: &TestServer, query: Value, expected: &[&str], context: &str) {
        let mut expected: Vec<_> = expected.iter().map(|id| (*id).to_owned()).collect();
        expected.sort();
        assert_eq!(
            search_ids(server, query).await,
            expected,
            "{context}: public search must retain exactly the expected IDs",
        );
    }

    async fn assert_text(server: &TestServer, text: &str, expected: &[&str], context: &str) {
        let response = server
            .post(&format!("/collections/{COLLECTION}/search"))
            .json(&json!({ "query": {
                "match": { "field": TEXT, "text": text, "op": "and" }
            }, "limit": 32, "track_total": true }))
            .await;
        response.assert_status_ok();
        let body: Value = response.json();
        assert!(
            body["hits"]
                .as_array()
                .expect("first sparse Text hits")
                .iter()
                .all(|hit| hit["score"].as_f64().is_some()),
            "{context}: Text Match exposes serialized BM25 scores: {body}",
        );
        let mut actual: Vec<_> = body["hits"]
            .as_array()
            .expect("first sparse Text hits")
            .iter()
            .map(|hit| hit["external_id"].as_str().expect("Text ID").to_owned())
            .collect();
        actual.sort();
        let mut expected: Vec<_> = expected.iter().map(|id| (*id).to_owned()).collect();
        expected.sort();
        assert_eq!(
            actual, expected,
            "{context}: Text Match retains expected IDs"
        );
    }

    fn term(field: &str, value: &str) -> Value {
        json!({ "term": { "field": field, "value": value } })
    }

    fn number(value: f64) -> Value {
        json!({ "range": { "field": NUM, "gte": value, "lte": value } })
    }

    fn hash(value: &str) -> Value {
        json!({ "hamming": { "field": HASH, "hash": value, "max_distance": 0 } })
    }

    fn knn(field: &str, seed: f64) -> Value {
        json!({ "knn": { "field": field, "vector": vector(seed), "k": 1 } })
    }

    fn collection(manifest: &Value) -> &Value {
        stage1_reuse_catalog_collection(manifest, COLLECTION)
    }

    fn base<'a>(manifest: &'a Value, role: &str, field: Option<&str>) -> &'a Value {
        collection(manifest)["segments"]
            .as_array()
            .expect("first sparse catalog segments")
            .iter()
            .find(|segment| {
                segment["role"] == json!(role)
                    && segment["field"] == field.map_or(Value::Null, |value| json!(value))
                    && segment["kind"] == json!("base")
                    && segment["ordinal"] == json!(0)
            })
            .unwrap_or_else(|| panic!("first sparse catalog needs {role}/{field:?} base"))
    }

    fn delta<'a>(manifest: &'a Value, field: &str) -> &'a Value {
        let deltas: Vec<_> = collection(manifest)["segments"]
            .as_array()
            .expect("first sparse catalog segments")
            .iter()
            .filter(|segment| {
                segment["role"] == json!("field")
                    && segment["field"] == json!(field)
                    && segment["kind"] == json!("delta")
            })
            .collect();
        assert_eq!(
            deltas.len(),
            1,
            "first sparse checkpoint publishes exactly one {field} delta",
        );
        assert_eq!(
            deltas[0]["ordinal"],
            json!(1),
            "first {field} delta immediately follows its zero-row base",
        );
        deltas[0]
    }

    fn lseg_rows(generation: &Path, reference: &Value) -> u32 {
        assert_eq!(reference["format"], json!("lseg-v1"));
        let path = generation.join(reference["path"].as_str().expect("catalog segment path"));
        let metadata = std::fs::symlink_metadata(&path).expect("inspect first sparse segment");
        assert!(
            metadata.is_file() && !metadata.file_type().is_symlink(),
            "first sparse catalog names a regular local segment: {}",
            path.display(),
        );
        let bytes = std::fs::read(&path).expect("read lseg-v1 header");
        assert!(
            bytes.len() >= 24,
            "lseg-v1 segment contains its fixed 24-byte header prefix",
        );
        u32::from_le_bytes(bytes[20..24].try_into().expect("read lseg n_docs"))
    }

    fn assert_manifest(generation: &Path, manifest: &Value) {
        assert_eq!(manifest["schema_version"], json!(2));
        assert_eq!(
            manifest["collections"].as_array().map(Vec::len),
            Some(1),
            "first sparse catalog contains exactly its fresh collection",
        );
        for (role, field) in [
            ("collection_eids", None),
            ("field", Some(KW)),
            ("field", Some(NUM)),
            ("field", Some(SET)),
            ("field", Some(HASH)),
            ("field", Some(TEXT)),
            ("field", Some(FLAT)),
            ("field", Some(HNSW)),
            ("vector_eids", Some(FLAT)),
            ("vector_eids", Some(HNSW)),
        ] {
            let base = base(manifest, role, field);
            assert!(base["local_rows"].is_null());
            assert_eq!(
                lseg_rows(generation, base),
                0,
                "fresh {role}/{field:?} base contains zero rows; captured records belong in sparse deltas",
            );
        }
        let expected: BTreeSet<_> = [LIVE, DELETED, REPLACED, EMPTY]
            .into_iter()
            .map(ToOwned::to_owned)
            .collect();
        for field in [KW, NUM, SET, HASH, TEXT, FLAT, HNSW] {
            let delta = delta(manifest, field);
            assert_eq!(delta["format"], json!("lseg-v1"));
            assert_eq!(
                stage1_keyword_delta_rows_count(delta),
                INITIAL_ROWS,
                "first {field} delta maps exactly the four fresh IDs touched before capture",
            );
            assert_eq!(
                stage1_keyword_delta_read_rows(generation, delta)
                    .into_iter()
                    .collect::<BTreeSet<_>>(),
                expected,
                "first {field} local rows retain update, delete, omission, and empty replacement",
            );
        }
    }

    async fn assert_state(server: &TestServer, include_concurrent: bool, context: &str) {
        assert_ids(server, term(KW, "live-keyword"), &[LIVE], context).await;
        assert_ids(server, term(KW, "deleted-keyword"), &[], context).await;
        assert_ids(server, term(KW, "empty-keyword"), &[], context).await;
        assert_ids(server, term(KW, "replace-old"), &[], context).await;
        assert_ids(server, term(KW, "replace-new"), &[REPLACED], context).await;
        assert_ids(server, number(101.0), &[LIVE], context).await;
        assert_ids(server, number(202.0), &[REPLACED], context).await;
        assert_ids(server, term(SET, "live-tag"), &[LIVE], context).await;
        assert_ids(server, term(SET, "replace-old-tag"), &[], context).await;
        assert_ids(server, hash("0000000000000a11"), &[LIVE], context).await;
        assert_ids(server, hash("0000000000000a13"), &[REPLACED], context).await;
        assert_text(server, "live text", &[LIVE], context).await;
        assert_text(server, "replaced text", &[REPLACED], context).await;
        assert_text(server, "deleted text", &[], context).await;
        for (field, seed, expected) in [
            (FLAT, 1.0, LIVE),
            (HNSW, 2.0, LIVE),
            (FLAT, 11.0, REPLACED),
            (HNSW, 12.0, REPLACED),
        ] {
            assert_ids(server, knn(field, seed), &[expected], context).await;
        }
        if include_concurrent {
            assert_ids(
                server,
                term(KW, "concurrent-keyword"),
                &[CONCURRENT],
                context,
            )
            .await;
            assert_ids(server, number(303.0), &[CONCURRENT], context).await;
            assert_ids(server, term(SET, "concurrent-tag"), &[CONCURRENT], context).await;
            assert_ids(server, hash("0000000000000a14"), &[CONCURRENT], context).await;
            assert_text(server, "concurrent text", &[CONCURRENT], context).await;
            assert_ids(server, knn(FLAT, 21.0), &[CONCURRENT], context).await;
            assert_ids(server, knn(HNSW, 22.0), &[CONCURRENT], context).await;
        }
        let response = server.get("/admin/backup").await;
        response.assert_status_ok();
        let snapshot: Value = response.json();
        assert_eq!(
            snapshot["collections"][COLLECTION]["fields"][FLAT]["spec"]["backend"],
            json!("flat-cpu"),
            "{context}: Flat backend survives checkpoint",
        );
        assert_eq!(
            snapshot["collections"][COLLECTION]["fields"][HNSW]["spec"]["backend"],
            json!("hnsw-cpu"),
            "{context}: HNSW backend survives checkpoint",
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn first_checkpoint_from_fresh_collection_uses_zero_bases_sparse_deltas_and_preserves_post_capture_write(
    ) {
        let fixture = fixture();
        create_collection(&fixture.server).await;
        index_full(
            &fixture.server,
            LIVE,
            "live-keyword",
            101.0,
            "live-tag",
            "0000000000000a11",
            "first sparse live text",
            1.0,
            2.0,
        )
        .await;
        index_full(
            &fixture.server,
            DELETED,
            "deleted-keyword",
            111.0,
            "deleted-tag",
            "0000000000000a10",
            "first sparse deleted text",
            3.0,
            4.0,
        )
        .await;
        index_full(
            &fixture.server,
            REPLACED,
            "replace-old",
            201.0,
            "replace-old-tag",
            "0000000000000a12",
            "first sparse stale text",
            5.0,
            6.0,
        )
        .await;
        index_full(
            &fixture.server,
            EMPTY,
            "empty-keyword",
            211.0,
            "empty-tag",
            "0000000000000a15",
            "first sparse empty text",
            7.0,
            8.0,
        )
        .await;
        fixture
            .server
            .delete(&format!("/collections/{COLLECTION}/index/{DELETED}"))
            .await
            .assert_status(axum::http::StatusCode::NO_CONTENT);
        replace_without_set(
            &fixture.server,
            REPLACED,
            "replace-new",
            202.0,
            "0000000000000a13",
            "first sparse replaced text",
            11.0,
            12.0,
        )
        .await;
        replace_empty(&fixture.server, EMPTY).await;

        let cut = fixture.writer.applied_seq();
        assert!(
            cut > 0,
            "first sparse checkpoint captures real public writes"
        );
        let (entered_tx, entered_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::sync_channel(1);
        fixture.hold.arm(entered_tx, release_rx);
        let mut release = SyncRelease(Some(release_tx));
        let mut checkpoint = tokio::spawn({
            let checkpoint = fixture.checkpoint.clone();
            async move { CheckpointSink::checkpoint_now(checkpoint.as_ref()).await }
        });
        let ready =
            tokio::task::spawn_blocking(move || entered_rx.recv_timeout(Duration::from_secs(30)))
                .await;
        if !matches!(ready, Ok(Ok(()))) {
            release.release();
            checkpoint.abort();
            let _ = checkpoint.await;
            panic!("first sparse checkpoint never reached the real SyncFile pause: {ready:?}");
        }

        let concurrent = tokio::time::timeout(
            Duration::from_secs(2),
            index_full(
                &fixture.server,
                CONCURRENT,
                "concurrent-keyword",
                303.0,
                "concurrent-tag",
                "0000000000000a14",
                "first sparse concurrent text",
                21.0,
                22.0,
            ),
        )
        .await;
        release.release();
        let checkpoint_error =
            match tokio::time::timeout(Duration::from_secs(30), &mut checkpoint).await {
                Ok(Ok(Ok(true))) => None,
                Ok(Ok(Ok(false))) => Some("checkpoint reported persisted=false".to_owned()),
                Ok(Ok(Err(error))) => Some(format!("checkpoint returned {error:#}")),
                Ok(Err(error)) => Some(format!("checkpoint task failed: {error}")),
                Err(_) => {
                    checkpoint.abort();
                    let _ = checkpoint.await;
                    Some("checkpoint did not finish after SyncFile release".to_owned())
                }
            };
        assert!(
            concurrent.is_ok(),
            "public all-field write must finish while first checkpoint is in file I/O: {concurrent:?}",
        );
        assert!(
            checkpoint_error.is_none(),
            "first sparse checkpoint finishes after release: {checkpoint_error:?}",
        );

        let first_generation = stage1_current_generation_dir(&fixture.root);
        let first_manifest = stage1_read_manifest(&first_generation);
        assert_eq!(
            first_manifest["checkpoint_sequence"],
            json!(cut),
            "paused first checkpoint retains the pre-pause capture cut",
        );
        assert_manifest(&first_generation, &first_manifest);
        let first_cold = fixture
            .store
            .load_current_generation()
            .expect("cold-open first sparse CURRENT")
            .expect("first sparse CURRENT");
        assert_eq!(first_cold.sequence, cut);
        let first_server =
            TestServer::new(router(AppState::open(first_cold.engine))).expect("first cold server");
        assert_state(&first_server, false, "first cold capture cut").await;
        assert_ids(
            &first_server,
            term(KW, "concurrent-keyword"),
            &[],
            "first cold capture cut excludes post-capture write",
        )
        .await;

        assert!(
            CheckpointSink::checkpoint_now(fixture.checkpoint.as_ref())
                .await
                .expect("publish post-capture write"),
            "second checkpoint persists the write that completed during first file I/O",
        );
        assert_state(&fixture.server, true, "live after second checkpoint").await;
        let latest = fixture
            .store
            .load_current_generation()
            .expect("cold-open latest first sparse CURRENT")
            .expect("latest first sparse CURRENT");
        assert!(latest.sequence > cut);
        let latest_server =
            TestServer::new(router(AppState::open(latest.engine))).expect("latest cold server");
        assert_state(&latest_server, true, "cold after second checkpoint").await;
    }
}
