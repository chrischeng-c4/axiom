//! Focused restore and cold recovery contract.

use std::sync::Arc;
use axum::http::StatusCode;
use axum_test::TestServer;
use lumen::api::{router, AppState};
use lumen::storage::Engine;
use serde_json::json;

#[tokio::test]
async fn snapshot_restore_preserves_query_results() {
    let source = TestServer::new(router(AppState::open(Arc::new(Engine::new())))).expect("source");
    source.put("/collections/restore").json(&json!({"fields":{"kw":{"type":"keyword"}}})).await.assert_status_ok();
    source.post("/collections/restore/index").json(&json!({"items":[{"external_id":"r1","field":"kw","value":"kept"}]})).await.assert_status_ok();
    let backup = source.get("/admin/backup").await;
    backup.assert_status_ok();
    let snapshot: serde_json::Value = backup.json();
    let restored = TestServer::new(router(AppState::open(Arc::new(Engine::new())))).expect("restored");
    let response = restored.post("/admin/restore").json(&snapshot).await;
    response.assert_status(StatusCode::NO_CONTENT);
    let query = restored
        .post("/collections/restore/search")
        .json(&json!({"query":{"term":{"field":"kw","value":"kept"}}}))
        .await;
    query.assert_status_ok();
    let body: serde_json::Value = query.json();
    assert_eq!(body["total"], 1, "restored state must retain the indexed document");
    assert_eq!(body["hits"][0]["external_id"], "r1");
}

#[path = "support/indexing_durable_catalog_fixture.rs"]
mod catalog_fixture;

use catalog_fixture::*;

/// Construct bytes in the exact v0.6.0 revision-generation format. This is
/// deliberately not a current writer round-trip: it pins the reader's upgrade
/// boundary to the fields that shipped before the v2 catalog existed.
#[tokio::test]
async fn shipped_v060_v1_revision_generation_cold_loads_before_v2_upgrade() {
    let dir = tempfile::tempdir().expect("v0.6.0 fixture directory");
    let root = dir.path().join("segments");
    let generation_name = "gen-41-rev-1";
    let generation = root.join(generation_name);
    let source = Arc::new(Engine::new());
    let source_server =
        TestServer::new(router(AppState::open(source.clone()))).expect("v0.6.0 source HTTP server");
    create_schema(&source_server).await;
    post_index(&source_server, field_items(0..1, "kw")).await;

    std::fs::create_dir_all(&generation).expect("create v0.6.0 generation");
    source
        .flush_to_segments(&generation, 41)
        .expect("write v0.6.0 segment payload");
    let mut manifest = serde_json::to_vec_pretty(&json!({
        "schema_version": 1,
        "sequence": 41,
        "revision": 1,
        "previous": Value::Null,
    }))
    .expect("encode v0.6.0 generation manifest");
    manifest.push(b'\n');
    std::fs::write(generation.join("_generation.json"), manifest)
        .expect("write v0.6.0 generation manifest");
    std::fs::write(
        root.join("CURRENT"),
        format!("generation:{generation_name}\n"),
    )
    .expect("point CURRENT at v0.6.0 generation");

    let store = SegmentRdbStore::new(&root).expect("open v0.6.0 checkpoint root");
    let loaded = store.load_current_generation();
    let load_error = loaded
        .as_ref()
        .err()
        .map(ToString::to_string)
        .unwrap_or_default();
    assert!(
        loaded.is_ok(),
        "stage1: the exact v0.6.0 v1 generation must cold-load before a v2 save; got {}",
        load_error
    );
    let loaded = loaded
        .expect("v0.6.0 current load succeeds")
        .expect("v0.6.0 CURRENT names a generation");
    assert_eq!(loaded.sequence, 41, "v0.6.0 sequence survives cold load");
    let cold_server =
        TestServer::new(router(AppState::open(loaded.engine))).expect("v0.6.0 cold HTTP server");
    assert_eq!(
        http_search(
            &cold_server,
            json!({ "term": { "field": "kw", "value": COMMON } }),
            "v0.6.0 cold keyword query",
        )
        .await["total"],
        1,
        "the v0.6.0 cold-loaded collection must still answer public searches"
    );
}

#[tokio::test]
async fn v2_current_refuses_duplicate_catalogued_segment_without_predecessor_fallback() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_duplicate_first_segment(&mut manifest);
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_input(
        &root,
        "duplicate segment reference",
        "a duplicate catalogued segment reference",
    );
}

#[tokio::test]
async fn v2_current_refuses_catalogue_path_escape_without_predecessor_fallback() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_first_segment_mut(&mut manifest)["path"] = json!("../outside.lseg");
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_input(
        &root,
        "segment reference path escapes generation",
        "a catalogued path that escapes the CURRENT generation",
    );
}

#[tokio::test]
async fn v2_current_refuses_missing_catalogued_segment_without_predecessor_fallback() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    let segment_path = stage1_first_segment_mut(&mut manifest)["path"]
        .as_str()
        .expect("catalogued segment path")
        .to_owned();
    let segment = generation.join(&segment_path);
    assert!(
        segment.is_file(),
        "catalogued segment must exist before removal"
    );
    std::fs::remove_file(&segment).expect("remove CURRENT-referenced catalogued segment");

    stage1_assert_current_refuses_catalog_input(
        &root,
        "catalogued segment is missing",
        "a missing catalogued segment",
    );
}

const STAGE1_WIDE_CATALOG_COLLECTIONS: usize = 182;

fn stage1_wide_catalog_id(index: usize) -> String {
    format!("catalog-{index:03}")
}

/// A complete catalog must scale with the number of live collections, not a
/// small fixed manifest cap. Empty keyword collections make the test metadata
/// only: no document volume or timing claim is needed to expose the format bug.
#[tokio::test]
async fn v2_complete_catalog_of_182_live_collections_cold_opens_every_collection() {
    let dir = tempfile::tempdir().expect("wide catalog fixture directory");
    let root = dir.path().join("segments");
    let store = SegmentRdbStore::new(&root).expect("open wide catalog checkpoint root");
    let engine = Arc::new(Engine::new());
    let server = TestServer::new(router(AppState::open(engine.clone())))
        .expect("wide catalog fixture HTTP server");

    for index in 0..STAGE1_WIDE_CATALOG_COLLECTIONS {
        server
            .put(&format!("/collections/{}", stage1_wide_catalog_id(index)))
            .json(&json!({ "fields": { "kw": { "type": "keyword" } } }))
            .await
            .assert_status_ok();
    }

    let saved = store.save(&engine, 182);
    let save_error = saved
        .as_ref()
        .err()
        .map(ToString::to_string)
        .unwrap_or_default();
    assert!(
        saved.is_ok(),
        "stage1: a complete v2 catalog with 182 live collections must checkpoint; got {save_error}"
    );

    let generation = stage1_current_generation_dir(&root);
    let manifest = stage1_read_manifest(&generation);
    let catalog_ids: Vec<String> = manifest["collections"]
        .as_array()
        .expect("v2 collections array")
        .iter()
        .map(|collection| {
            collection["collection_id"]
                .as_str()
                .expect("catalog collection id")
                .to_owned()
        })
        .collect();
    let expected_ids: Vec<String> = (0..STAGE1_WIDE_CATALOG_COLLECTIONS)
        .map(stage1_wide_catalog_id)
        .collect();
    assert_eq!(
        catalog_ids, expected_ids,
        "the complete catalog must contain every live collection exactly once and in lexical order"
    );

    let cold_store = SegmentRdbStore::new(&root).expect("reopen wide catalog root");
    let cold = cold_store.load_current_generation();
    let cold_error = cold
        .as_ref()
        .err()
        .map(ToString::to_string)
        .unwrap_or_default();
    assert!(
        cold.is_ok(),
        "stage1: the 182-collection v2 catalog must cold-open; got {cold_error}"
    );
    let cold = cold
        .expect("wide catalog cold load succeeds")
        .expect("wide catalog CURRENT names a generation");
    let cold_server = TestServer::new(router(AppState::open(cold.engine)))
        .expect("wide catalog cold HTTP server");
    let listed: Value = cold_server.get("/collections").await.json();
    let listed_ids: Vec<String> = listed
        .as_array()
        .expect("cold collection listing")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("cold listed collection id")
                .to_owned()
        })
        .collect();
    assert_eq!(
        listed_ids, expected_ids,
        "cold open must restore every catalogued collection to the public listing"
    );
}

#[tokio::test]
async fn v2_current_refuses_collection_eids_role_that_names_a_field() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_catalog_segment_with_role_mut(&mut manifest, "collection_eids")["field"] = json!("kw");
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &root,
        "a collection_eids reference that names a field",
    );
}

#[tokio::test]
async fn v2_current_refuses_field_role_without_a_field_name() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_catalog_segment_with_role_mut(&mut manifest, "field")["field"] = Value::Null;
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(&root, "a field reference without a field name");
}

#[tokio::test]
async fn v2_current_refuses_catalog_that_omits_a_live_collection() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    manifest["collections"]
        .as_array_mut()
        .expect("v2 catalog collections array")
        .clear();
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(&root, "a catalog that omits a live collection");
}

#[tokio::test]
async fn v2_current_refuses_catalog_schema_that_omits_a_live_field() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_catalog_collection_mut(&mut manifest)["schema"] = json!({});
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &root,
        "a catalog schema that omits the persisted keyword field",
    );
}

#[tokio::test]
async fn v2_current_refuses_catalogue_dot_path_without_predecessor_fallback() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_first_segment_mut(&mut manifest)["path"] = json!("./catalogued.lseg");
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_input(
        &root,
        "segment reference path escapes generation",
        "a catalogued path containing a dot component",
    );
}

#[tokio::test]
async fn v2_current_refuses_field_reference_absent_from_catalog_schema() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_catalog_segment_mut(
        stage1_catalog_collection_mut(&mut manifest),
        "field",
        Some("kw"),
    )["field"] = json!("field-not-in-schema");
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &root,
        "a field reference whose field is absent from the catalog schema",
    );
}

#[tokio::test]
async fn v2_current_refuses_vector_eids_reference_for_keyword_field() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_catalog_segment_mut(
        stage1_catalog_collection_mut(&mut manifest),
        "field",
        Some("kw"),
    )["role"] = json!("vector_eids");
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &root,
        "a vector_eids reference for a keyword field",
    );
}

#[tokio::test]
async fn v2_current_refuses_segment_reference_owned_by_another_collection() {
    let (_dir, root) = stage1_catalog_root_with_two_keyword_collections().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    let left_path = stage1_catalog_segment_mut(
        stage1_catalog_collection_mut_by_id(&mut manifest, STAGE1_CATALOG_LEFT),
        "field",
        Some("kw"),
    )["path"]
        .clone();
    let right_path = stage1_catalog_segment_mut(
        stage1_catalog_collection_mut_by_id(&mut manifest, STAGE1_CATALOG_RIGHT),
        "field",
        Some("kw"),
    )["path"]
        .clone();
    stage1_catalog_segment_mut(
        stage1_catalog_collection_mut_by_id(&mut manifest, STAGE1_CATALOG_LEFT),
        "field",
        Some("kw"),
    )["path"] = right_path;
    stage1_catalog_segment_mut(
        stage1_catalog_collection_mut_by_id(&mut manifest, STAGE1_CATALOG_RIGHT),
        "field",
        Some("kw"),
    )["path"] = left_path;
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &root,
        "a segment reference that points into another catalogued collection",
    );
}

#[tokio::test]
async fn v2_current_refuses_catalog_omitting_required_field_segment() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_catalog_remove_segment(
        stage1_catalog_collection_mut(&mut manifest),
        "field",
        Some("kw"),
    );
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &root,
        "a catalog that omits a required keyword field segment",
    );
}

#[tokio::test]
async fn v2_current_refuses_catalog_omitting_required_collection_eids_segment() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_catalog_remove_segment(
        stage1_catalog_collection_mut(&mut manifest),
        "collection_eids",
        None,
    );
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &root,
        "a catalog that omits the required collection external-id segment",
    );
}

#[tokio::test]
async fn v2_current_refuses_nonzero_ordinal_for_base_segment() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_catalog_segment_mut(
        stage1_catalog_collection_mut(&mut manifest),
        "field",
        Some("kw"),
    )["ordinal"] = json!(1);
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(&root, "a base segment with a nonzero ordinal");
}

#[tokio::test]
async fn v2_current_refuses_catalog_schema_version_mismatch() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    let collection = stage1_catalog_collection_mut(&mut manifest);
    let schema_version = collection["schema_version"]
        .as_u64()
        .expect("catalog schema version");
    collection["schema_version"] = json!(schema_version + 1);
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &root,
        "a catalog schema version that disagrees with its checkpoint schema",
    );
}

#[tokio::test]
async fn v2_current_refuses_zero_collection_generation() {
    let (_dir, root) = stage1_v2_root_with_predecessor().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    stage1_catalog_collection_mut(&mut manifest)["collection_generation"] = json!(0);
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(&root, "a zero durable collection generation");
}

#[tokio::test]
async fn v2_current_refuses_duplicate_collection_generation() {
    let (_dir, root) = stage1_catalog_root_with_two_keyword_collections().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    let left_generation = stage1_catalog_collection_mut_by_id(&mut manifest, STAGE1_CATALOG_LEFT)
        ["collection_generation"]
        .clone();
    stage1_catalog_collection_mut_by_id(&mut manifest, STAGE1_CATALOG_RIGHT)
        ["collection_generation"] = left_generation;
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &root,
        "duplicate durable collection generations in one catalog",
    );
}

#[tokio::test]
async fn v2_current_refuses_allocator_not_above_issued_collection_generations() {
    let (_dir, root) = stage1_catalog_root_with_two_keyword_collections().await;
    let generation = stage1_current_generation_dir(&root);
    let mut manifest = stage1_read_manifest(&generation);
    let max_generation = stage1_catalog_max_collection_generation(&manifest);
    manifest["next_collection_generation"] = json!(max_generation);
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &root,
        "a collection-generation allocator at the highest issued epoch",
    );
}
