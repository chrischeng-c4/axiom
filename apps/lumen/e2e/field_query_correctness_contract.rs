//! Focused field and query correctness contract.

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
async fn scalar_and_text_queries_return_the_indexed_document() {
    let dir = tempfile::tempdir().expect("checkpoint root");
    let engine = Arc::new(Engine::new());
    let aof: SharedAof = Arc::new(Mutex::new(AofWriter::open(dir.path().join("aof.log")).expect("AOF")));
    let wal: SharedWal = Arc::new(MemWal::default());
    let writer = WriteCoordinator::start_from_with_aof(wal, engine.clone(), 0, aof.clone());
    let sink_writer: Arc<dyn WriteSink> = writer;
    let store = Arc::new(SegmentRdbStore::new(dir.path().join("segments")).expect("store"));
    let checkpoint: Arc<dyn CheckpointSink> = Arc::new(SegmentCheckpointSink { engine: engine.clone(), store: store.clone(), writer: sink_writer.clone(), aof: Some(aof) });
    let state = AppState::with_components(engine, Arc::new(AuthConfig::open()), sink_writer).with_checkpoint(checkpoint);
    let server = TestServer::new(router(state)).expect("server");
    server.put("/collections/query").json(&json!({"fields":{"kw":{"type":"keyword"},"body":{"type":"text"},"num":{"type":"number"},"v":{"type":"vector","dim":3,"metric":"cosine"}}})).await.assert_status_ok();
    server.post("/collections/query/index").json(&json!({"items":[{"external_id":"q1","field":"kw","value":"stable"},{"external_id":"q1","field":"body","value":"stable text"},{"external_id":"q1","field":"num","value":7},{"external_id":"q1","field":"v","value":[0.1,0.2,0.3]}]})).await.assert_status_ok();
    for query in [json!({"term":{"field":"kw","value":"stable"}}), json!({"match":{"field":"body","text":"stable"}}), json!({"range":{"field":"num","gte":7,"lte":7}}), json!({"knn":{"field":"v","vector":[0.1,0.2,0.3],"k":1}})] {
        let response = server.post("/collections/query/search").json(&json!({"query":query})).await;
        response.assert_status_ok();
        assert_eq!(response.json::<serde_json::Value>()["total"], 1);
    }
    server.post("/admin/checkpoint").json(&json!({})).await.assert_status_ok();
    let cold = SegmentRdbStore::new(dir.path().join("segments")).expect("cold store").load_current_generation().expect("CURRENT").expect("generation");
    let cold_server = TestServer::new(router(AppState::open(cold.engine))).expect("cold server");
    let response = cold_server.post("/collections/query/search").json(&json!({"query":{"knn":{"field":"v","vector":[0.1,0.2,0.3],"k":1}}})).await;
    response.assert_status_ok();
    assert_eq!(response.json::<serde_json::Value>()["total"], 1);
}

#[path = "support/indexing_durable_catalog_fixture.rs"]
mod catalog_fixture;

use catalog_fixture::*;

/// A field that was sealed in an earlier segment generation remains mutable.
///
/// This is a public durability contract for #4164. A caller indexes a base
/// document, persists it with `/admin/checkpoint`, replaces the keyword through
/// the HTTP index route, and checkpoints again. A new `SegmentRdbStore` then
/// cold-opens `CURRENT`. Its HTTP search surface must expose only the new
/// keyword, retain `exists`, and still allow a complete document delete. The
/// final delete checks that no stale posting from the sealed base can return
/// after a cold reopen.
#[tokio::test]
async fn sealed_keyword_update_survives_checkpoint_and_cold_reopen() {
    const BASE: &str = "doc-000";
    const BEFORE: &str = "sealed-base-before";
    const AFTER: &str = "sealed-base-after";

    let fixture = fixture();
    create_schema(&fixture.server).await;

    // Use a corpus-sized base so the first checkpoint seals an ordinary
    // keyword segment rather than testing only an empty or tail-only field.
    post_index(&fixture.server, corpus_items(false, PREFIX_DOCUMENTS)).await;
    fixture
        .server
        .post(&format!("/collections/{COLLECTION}/index"))
        .json(&json!({ "items": [{
            "external_id": BASE,
            "field": "kw",
            "value": BEFORE
        }] }))
        .await
        .assert_status_ok();
    checkpoint(&fixture.server).await;

    // This targets a value that existed in the sealed base generation. The
    // second checkpoint must materialize its replacement, not resurrect the
    // old segment posting during a later cold open.
    fixture
        .server
        .post(&format!("/collections/{COLLECTION}/index"))
        .json(&json!({ "items": [{
            "external_id": BASE,
            "field": "kw",
            "value": AFTER
        }] }))
        .await
        .assert_status_ok();
    checkpoint(&fixture.server).await;

    // Re-open the active immutable generation through a new store instance.
    // This rules out the live engine and its in-memory segment handles as the
    // oracle: only persisted `CURRENT` content reaches this server.
    let cold_store = SegmentRdbStore::new(&fixture.checkpoint_root)
        .expect("open checkpoint root for cold recovery");
    let cold = cold_store
        .load_current_generation()
        .expect("load CURRENT after second checkpoint")
        .expect("second checkpoint created a generation");
    let cold_server = TestServer::new(router(AppState::open(cold.engine.clone())))
        .expect("cold recovered HTTP server");

    let new_value = http_search(
        &cold_server,
        json!({ "term": { "field": "kw", "value": AFTER } }),
        "cold reopen new keyword",
    )
    .await;
    assert_eq!(
        new_value["total"], 1,
        "cold reopen must retain the replacement keyword for the sealed base"
    );
    assert_eq!(hit_ids(&new_value), vec![BASE]);

    let old_value = http_search(
        &cold_server,
        json!({ "term": { "field": "kw", "value": BEFORE } }),
        "cold reopen old keyword",
    )
    .await;
    assert_eq!(
        old_value["total"], 0,
        "cold reopen must not resurrect the sealed base keyword that the caller replaced"
    );

    let exists = http_search(
        &cold_server,
        json!({ "exists": { "field": "kw" } }),
        "cold reopen keyword exists",
    )
    .await;
    assert_eq!(
        exists["total"], PREFIX_DOCUMENTS as u64,
        "the updated base must remain present in the keyword exists set"
    );
    assert!(
        hit_ids(&exists).contains(&BASE),
        "the updated sealed base must satisfy exists after cold reopen"
    );

    cold_server
        .delete(&format!("/collections/{COLLECTION}/docs/{BASE}"))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);
    let deleted = http_search(
        &cold_server,
        json!({ "term": { "field": "kw", "value": AFTER } }),
        "delete after cold reopen",
    )
    .await;
    assert_eq!(
        deleted["total"], 0,
        "a complete document delete must remove the recovered sealed posting"
    );
}

// Append after apps/lumen/e2e/indexing_durable_oracle.rs at sha256
// 3a1f56ba158a191b2070806b8653e9c6bbfaee5c73bf1082ee5e8f6f686401ed.
//
// Facets when applied:
// - Behavior: public keyword queries, immutable base retention, cold reopen,
//   and no resurrection after a second checkpoint.
// - Security: malformed local-row metadata and duplicate stable IDs must refuse
//   CURRENT without rewriting it; no implementation error text is assumed.
// - Performance: the inode and three-row assertions prove sparse physical work.
//   The approved stage6 release workload, not this case, measures checkpoint time.

const STAGE1_KEYWORD_DELTA_COLLECTION: &str = "keyword-sparse-delta";
const STAGE1_KEYWORD_DELTA_BASE_DOCS: usize = 128;
const STAGE1_KEYWORD_DELTA_TOUCHED_IDS: u64 = 3;
const STAGE1_KEYWORD_DELTA_BASE_SEQUENCE: u64 = 1_101;
const STAGE1_KEYWORD_DELTA_FIRST_SEQUENCE: u64 = 1_102;
const STAGE1_KEYWORD_DELTA_SECOND_SEQUENCE: u64 = 1_103;
const STAGE1_KEYWORD_DELTA_UNTOUCHED_ID: &str = "base-000";
const STAGE1_KEYWORD_DELTA_DELETED_ID: &str = "base-001";
const STAGE1_KEYWORD_DELTA_HIGH_ID: &str = "runtime-id-900000";
const STAGE1_KEYWORD_DELTA_APPENDED_ID: &str = "runtime-id-900001";
const STAGE1_KEYWORD_DELTA_HIGH_OLD: &str = "high-base-old";
const STAGE1_KEYWORD_DELTA_HIGH_NEW: &str = "high-delta-new";
const STAGE1_KEYWORD_DELTA_HIGH_FINAL: &str = "high-delta-final";
const STAGE1_KEYWORD_DELTA_APPENDED_VALUE: &str = "appended-after-base";

// `runtime-id-900000` is only a stable external-ID label. It is base row 128
// in this 129-row fixture, not a dense runtime row numbered 900000.

struct Stage1KeywordDeltaFixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    store: SegmentRdbStore,
    engine: Arc<Engine>,
    server: TestServer,
    base_name: String,
}

fn stage1_keyword_delta_base_id(index: usize) -> String {
    format!("base-{index:03}")
}

fn stage1_keyword_delta_base_value(index: usize) -> String {
    format!("base-value-{index:03}")
}

fn stage1_keyword_delta_base_items() -> Vec<Value> {
    let mut items: Vec<Value> = (0..STAGE1_KEYWORD_DELTA_BASE_DOCS)
        .map(|index| {
            let external_id = stage1_keyword_delta_base_id(index);
            let value = stage1_keyword_delta_base_value(index);
            stage1_reuse_item(&external_id, &value)
        })
        .collect();
    items.push(stage1_reuse_item(
        STAGE1_KEYWORD_DELTA_HIGH_ID,
        STAGE1_KEYWORD_DELTA_HIGH_OLD,
    ));
    items
}

async fn stage1_keyword_delta_index_items(server: &TestServer, items: Vec<Value>) {
    server
        .post(&format!(
            "/collections/{STAGE1_KEYWORD_DELTA_COLLECTION}/index"
        ))
        .json(&json!({ "items": items }))
        .await
        .assert_status_ok();
}

async fn stage1_keyword_delta_fixture() -> Stage1KeywordDeltaFixture {
    let dir = tempfile::tempdir().expect("keyword delta fixture directory");
    let root = dir.path().join("segments");
    let store = SegmentRdbStore::new(&root).expect("open keyword delta root");
    let engine = Arc::new(Engine::new());
    let server =
        TestServer::new(router(AppState::open(engine.clone()))).expect("keyword delta HTTP server");
    stage1_reuse_put_keyword_collection(&server, STAGE1_KEYWORD_DELTA_COLLECTION).await;
    stage1_keyword_delta_index_items(&server, stage1_keyword_delta_base_items()).await;
    stage1_restore_legacy_base(&engine, "keyword sparse-delta base");
    store
        .save(&engine, STAGE1_KEYWORD_DELTA_BASE_SEQUENCE)
        .expect("write keyword delta base generation");
    let base_name = stage1_reuse_current_name(&root);
    Stage1KeywordDeltaFixture {
        _dir: dir,
        root,
        store,
        engine,
        server,
        base_name,
    }
}

async fn stage1_keyword_delta_publish_first(fixture: &Stage1KeywordDeltaFixture) {
    stage1_reuse_index(
        &fixture.server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_HIGH_ID,
        STAGE1_KEYWORD_DELTA_HIGH_NEW,
    )
    .await;
    fixture
        .server
        .delete(&format!(
            "/collections/{STAGE1_KEYWORD_DELTA_COLLECTION}/docs/{STAGE1_KEYWORD_DELTA_DELETED_ID}"
        ))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);
    stage1_reuse_index(
        &fixture.server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_APPENDED_ID,
        STAGE1_KEYWORD_DELTA_APPENDED_VALUE,
    )
    .await;
    fixture
        .store
        .save(&fixture.engine, STAGE1_KEYWORD_DELTA_FIRST_SEQUENCE)
        .expect("write first keyword sparse delta generation");
}

async fn stage1_keyword_delta_publish_second(fixture: &Stage1KeywordDeltaFixture) {
    stage1_reuse_index(
        &fixture.server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_HIGH_ID,
        STAGE1_KEYWORD_DELTA_HIGH_FINAL,
    )
    .await;
    fixture
        .store
        .save(&fixture.engine, STAGE1_KEYWORD_DELTA_SECOND_SEQUENCE)
        .expect("write second keyword sparse delta generation");
}

fn stage1_keyword_delta_collection(manifest: &Value) -> &Value {
    stage1_reuse_catalog_collection(manifest, STAGE1_KEYWORD_DELTA_COLLECTION)
}

fn stage1_keyword_delta_refs<'a>(manifest: &'a Value, kind: &str) -> Vec<&'a Value> {
    stage1_keyword_delta_collection(manifest)["segments"]
        .as_array()
        .expect("keyword delta catalog segments")
        .iter()
        .filter(|segment| {
            segment["role"] == json!("field")
                && segment["field"] == json!("kw")
                && segment["kind"] == json!(kind)
        })
        .collect()
}

fn stage1_keyword_delta_base_ref(manifest: &Value) -> &Value {
    stage1_keyword_delta_refs(manifest, "base")
        .into_iter()
        .find(|segment| segment["ordinal"] == json!(0))
        .expect("keyword catalog must retain its base reference")
}

fn stage1_keyword_delta_single_ref(manifest: &Value, sequence: u64) -> &Value {
    let deltas = stage1_keyword_delta_refs(manifest, "delta");
    assert_eq!(
        deltas.len(),
        1,
        "the first checkpoint after three sparse keyword mutations must publish one keyword delta"
    );
    let matching: Vec<_> = deltas
        .iter()
        .copied()
        .filter(|delta| delta["applied_seq"] == json!(sequence))
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "the keyword delta for applied sequence {sequence} must be uniquely catalogued",
    );
    let delta = matching[0];
    assert_eq!(
        delta["ordinal"],
        json!(1),
        "the first keyword delta must immediately follow its ordinal-zero base"
    );
    assert_eq!(delta["format"], json!("lseg-v1"));
    assert!(
        delta["path"].as_str().is_some(),
        "a keyword delta must name its segment through the catalog"
    );
    assert!(
        delta["local_rows"].is_object(),
        "a sparse keyword delta must carry local stable-ID rows"
    );
    delta
}

fn stage1_keyword_delta_expected_rows() -> std::collections::BTreeSet<String> {
    [
        STAGE1_KEYWORD_DELTA_HIGH_ID,
        STAGE1_KEYWORD_DELTA_DELETED_ID,
        STAGE1_KEYWORD_DELTA_APPENDED_ID,
    ]
    .iter()
    .map(|id| (*id).to_owned())
    .collect()
}

fn stage1_keyword_delta_assert_sparse_rows(generation: &Path, delta: &Value) {
    let count = stage1_keyword_delta_rows_count(delta);
    assert_eq!(
        count, STAGE1_KEYWORD_DELTA_TOUCHED_IDS,
        "the local-row count must equal the three changed stable external IDs"
    );
    assert!(
        count < STAGE1_KEYWORD_DELTA_BASE_DOCS as u64,
        "a sparse delta must not allocate local rows for the full base cardinality"
    );
    let rows = stage1_keyword_delta_read_rows(generation, delta);
    assert_eq!(
        rows.len() as u64,
        count,
        "sidecar rows match their declared count"
    );
    let unique: std::collections::BTreeSet<String> = rows.iter().cloned().collect();
    assert_eq!(
        unique.len(),
        rows.len(),
        "a sparse local-row sidecar must not repeat a stable external ID"
    );
    assert_eq!(
        unique,
        stage1_keyword_delta_expected_rows(),
        "the local-row sidecar names exactly the three changed stable IDs"
    );
}

#[cfg(unix)]
fn stage1_keyword_delta_assert_base_keyword_hardlinked(
    base_generation: &Path,
    base_manifest: &Value,
    delta_generation: &Path,
    delta_manifest: &Value,
) {
    let base = stage1_keyword_delta_base_ref(base_manifest);
    let retained = stage1_keyword_delta_base_ref(delta_manifest);
    let base_path = base_generation.join(base["path"].as_str().expect("base keyword path"));
    let retained_path =
        delta_generation.join(retained["path"].as_str().expect("retained keyword path"));
    assert_ne!(
        base_path, retained_path,
        "immutable checkpoint generations must use separate path names"
    );
    let base_metadata = std::fs::symlink_metadata(&base_path).expect("inspect keyword base");
    let retained_metadata =
        std::fs::symlink_metadata(&retained_path).expect("inspect retained keyword base");
    assert!(base_metadata.is_file() && !base_metadata.file_type().is_symlink());
    assert!(retained_metadata.is_file() && !retained_metadata.file_type().is_symlink());
    assert_eq!(
        base_metadata.ino(),
        retained_metadata.ino(),
        "the unchanged keyword base must be retained by hard link beside its delta"
    );
    assert!(
        retained_metadata.nlink() >= 2,
        "the retained keyword base must report multiple hard links"
    );
}

async fn stage1_keyword_delta_assert_base_state(server: &TestServer) {
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_HIGH_OLD,
        &[STAGE1_KEYWORD_DELTA_HIGH_ID],
    )
    .await;
    let deleted_value = stage1_keyword_delta_base_value(1);
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        &deleted_value,
        &[STAGE1_KEYWORD_DELTA_DELETED_ID],
    )
    .await;
    let untouched_value = stage1_keyword_delta_base_value(0);
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        &untouched_value,
        &[STAGE1_KEYWORD_DELTA_UNTOUCHED_ID],
    )
    .await;
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_APPENDED_VALUE,
        &[],
    )
    .await;
}

async fn stage1_keyword_delta_assert_first_live_state(server: &TestServer) {
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_HIGH_OLD,
        &[],
    )
    .await;
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_HIGH_NEW,
        &[STAGE1_KEYWORD_DELTA_HIGH_ID],
    )
    .await;
    let deleted_value = stage1_keyword_delta_base_value(1);
    stage1_reuse_assert_term_ids(server, STAGE1_KEYWORD_DELTA_COLLECTION, &deleted_value, &[])
        .await;
    let untouched_value = stage1_keyword_delta_base_value(0);
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        &untouched_value,
        &[STAGE1_KEYWORD_DELTA_UNTOUCHED_ID],
    )
    .await;
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_APPENDED_VALUE,
        &[STAGE1_KEYWORD_DELTA_APPENDED_ID],
    )
    .await;
}

async fn stage1_keyword_delta_assert_second_live_state(server: &TestServer) {
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_HIGH_OLD,
        &[],
    )
    .await;
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_HIGH_NEW,
        &[],
    )
    .await;
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_HIGH_FINAL,
        &[STAGE1_KEYWORD_DELTA_HIGH_ID],
    )
    .await;
    let deleted_value = stage1_keyword_delta_base_value(1);
    stage1_reuse_assert_term_ids(server, STAGE1_KEYWORD_DELTA_COLLECTION, &deleted_value, &[])
        .await;
    let untouched_value = stage1_keyword_delta_base_value(0);
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        &untouched_value,
        &[STAGE1_KEYWORD_DELTA_UNTOUCHED_ID],
    )
    .await;
    stage1_reuse_assert_term_ids(
        server,
        STAGE1_KEYWORD_DELTA_COLLECTION,
        STAGE1_KEYWORD_DELTA_APPENDED_VALUE,
        &[STAGE1_KEYWORD_DELTA_APPENDED_ID],
    )
    .await;
}

fn stage1_keyword_delta_ref_mut(manifest: &mut Value) -> &mut Value {
    stage1_catalog_collection_mut_by_id(manifest, STAGE1_KEYWORD_DELTA_COLLECTION)["segments"]
        .as_array_mut()
        .expect("keyword delta catalog segments")
        .iter_mut()
        .find(|segment| {
            segment["role"] == json!("field")
                && segment["field"] == json!("kw")
                && segment["kind"] == json!("delta")
        })
        .expect("keyword delta after behavior assertion")
}

#[cfg(unix)]
#[tokio::test]
async fn v2_keyword_sparse_delta_retains_base_and_masks_live_and_cold_queries() {
    let fixture = stage1_keyword_delta_fixture().await;
    let base_generation = fixture.root.join(&fixture.base_name);
    let base_manifest = stage1_read_manifest(&base_generation);
    stage1_keyword_delta_publish_first(&fixture).await;
    stage1_keyword_delta_assert_first_live_state(&fixture.server).await;

    let first_name = stage1_reuse_current_name(&fixture.root);
    assert_ne!(
        first_name, fixture.base_name,
        "a sparse keyword change publishes a new immutable generation"
    );
    let first_generation = fixture.root.join(&first_name);
    let first_manifest = stage1_read_manifest(&first_generation);
    let first_delta =
        stage1_keyword_delta_single_ref(&first_manifest, STAGE1_KEYWORD_DELTA_FIRST_SEQUENCE);
    stage1_keyword_delta_assert_sparse_rows(&first_generation, first_delta);
    stage1_keyword_delta_assert_base_keyword_hardlinked(
        &base_generation,
        &base_manifest,
        &first_generation,
        &first_manifest,
    );

    let (first_engine, first_sequence) = stage1_reuse_cold_load_current(&fixture.root);
    assert_eq!(
        first_sequence, STAGE1_KEYWORD_DELTA_FIRST_SEQUENCE,
        "cold reopen selects the first sparse-delta checkpoint"
    );
    let first_server = TestServer::new(router(AppState::open(first_engine)))
        .expect("first sparse-delta cold HTTP server");
    stage1_keyword_delta_assert_first_live_state(&first_server).await;

    stage1_keyword_delta_publish_second(&fixture).await;
    stage1_keyword_delta_assert_second_live_state(&fixture.server).await;
    let (latest_engine, latest_sequence) = stage1_reuse_cold_load_current(&fixture.root);
    assert_eq!(
        latest_sequence, STAGE1_KEYWORD_DELTA_SECOND_SEQUENCE,
        "cold reopen selects the second sparse-delta checkpoint"
    );
    let latest_server = TestServer::new(router(AppState::open(latest_engine)))
        .expect("second sparse-delta cold HTTP server");
    stage1_keyword_delta_assert_second_live_state(&latest_server).await;

    let (base_engine, base_sequence) =
        stage1_reuse_cold_load_named(&fixture.root, &fixture.base_name);
    assert_eq!(
        base_sequence, STAGE1_KEYWORD_DELTA_BASE_SEQUENCE,
        "the retained base generation keeps its original checkpoint sequence"
    );
    let base_server = TestServer::new(router(AppState::open(base_engine)))
        .expect("retained keyword base cold HTTP server");
    stage1_keyword_delta_assert_base_state(&base_server).await;
}

#[tokio::test]
async fn v2_current_refuses_keyword_delta_local_rows_count_mismatch() {
    let fixture = stage1_keyword_delta_fixture().await;
    stage1_keyword_delta_publish_first(&fixture).await;
    let generation = stage1_current_generation_dir(&fixture.root);
    let mut manifest = stage1_read_manifest(&generation);
    {
        let delta = stage1_keyword_delta_single_ref(&manifest, STAGE1_KEYWORD_DELTA_FIRST_SEQUENCE);
        stage1_keyword_delta_assert_sparse_rows(&generation, delta);
    }
    let delta = stage1_keyword_delta_ref_mut(&mut manifest);
    let local_rows = delta["local_rows"]
        .as_object_mut()
        .expect("local rows after behavior assertion");
    let count = local_rows["count"]
        .as_u64()
        .expect("local-row count after behavior assertion");
    local_rows.insert("count".to_owned(), json!(count + 1));
    stage1_write_manifest(&generation, &manifest);

    stage1_assert_current_refuses_catalog_shape(
        &fixture.root,
        "a sparse keyword delta whose local-row count does not match its sidecar",
    );
}

#[tokio::test]
async fn v2_current_refuses_keyword_delta_duplicate_stable_local_rows() {
    let fixture = stage1_keyword_delta_fixture().await;
    stage1_keyword_delta_publish_first(&fixture).await;
    let generation = stage1_current_generation_dir(&fixture.root);
    let manifest = stage1_read_manifest(&generation);
    let rows_path = {
        let delta = stage1_keyword_delta_single_ref(&manifest, STAGE1_KEYWORD_DELTA_FIRST_SEQUENCE);
        stage1_keyword_delta_assert_sparse_rows(&generation, delta);
        stage1_keyword_delta_rows_path(&generation, delta)
    };
    let mut rows: Vec<String> = ciborium::from_reader(
        std::fs::File::open(&rows_path).expect("open sparse local-row sidecar"),
    )
    .expect("decode sparse local-row sidecar");
    assert_eq!(
        rows.len(),
        STAGE1_KEYWORD_DELTA_TOUCHED_IDS as usize,
        "duplicate-mapping mutation starts from the sparse three-row contract"
    );
    rows[1] = rows[0].clone();
    stage1_keyword_delta_write_rows(&rows_path, &rows);

    stage1_assert_current_refuses_catalog_shape(
        &fixture.root,
        "a sparse keyword delta with duplicate stable external IDs in local rows",
    );
}
// Append after apps/lumen/e2e/indexing_durable_oracle.rs at sha256
// 06752f2533c2c65519ae06c8af4a1915553e8df0388b3e52bf5b89e076a7f3bc.
//
// Facets when applied:
// - Behavior: Number, Set, and Hash each publish sparse ordered delta refs;
//   public live and cold queries match an independently built reference after
//   replacement, deletion, append, and a second checkpoint.
// - Security: a duplicate external ID in a Number delta local-row sidecar must
//   refuse CURRENT without rewriting it. The shared generic refusal helper makes
//   no implementation error-string assumption.
// - Performance: exact three-row and one-row sidecars plus base-file inode
//   equality show sparse work and hard-link reuse. The approved stage6 release
//   workload, rather than this case, measures checkpoint and merge time.
//
// Source premise read only: apps/lumen/src/segment_rdb.rs:1296-1304 currently
// emits base-only catalog refs, and :1373-1380 rejects layers. These cases must
// be red until the scalar layered writer and reader land.

const STAGE1_SCALAR_DELTA_COLLECTION: &str = "scalar-sparse-delta";
const STAGE1_SCALAR_NUMBER_FIELD: &str = "num";
const STAGE1_SCALAR_SET_FIELD: &str = "tags";
const STAGE1_SCALAR_HASH_FIELD: &str = "sig";
const STAGE1_SCALAR_FIELDS: [&str; 3] = [
    STAGE1_SCALAR_NUMBER_FIELD,
    STAGE1_SCALAR_SET_FIELD,
    STAGE1_SCALAR_HASH_FIELD,
];
const STAGE1_SCALAR_ORDINARY_BASE_ROWS: usize = 128;
const STAGE1_SCALAR_BASE_ROWS: usize = STAGE1_SCALAR_ORDINARY_BASE_ROWS + 1;
const STAGE1_SCALAR_FIRST_TOUCHED_IDS: u64 = 3;
const STAGE1_SCALAR_SECOND_TOUCHED_IDS: u64 = 1;
const STAGE1_SCALAR_BASE_SEQUENCE: u64 = 1_201;
const STAGE1_SCALAR_FIRST_SEQUENCE: u64 = 1_202;
const STAGE1_SCALAR_SECOND_SEQUENCE: u64 = 1_203;
const STAGE1_SCALAR_UNTOUCHED_ID: &str = "scalar-base-000";
const STAGE1_SCALAR_DELETED_ID: &str = "scalar-base-001";
const STAGE1_SCALAR_HIGH_ID: &str = "runtime-id-900000";
const STAGE1_SCALAR_APPENDED_ID: &str = "runtime-id-900001";
const STAGE1_SCALAR_HIGH_OLD_NUMBER: f64 = 90_000.0;
const STAGE1_SCALAR_HIGH_NEW_NUMBER: f64 = 90_001.0;
const STAGE1_SCALAR_HIGH_FINAL_NUMBER: f64 = 90_002.0;
const STAGE1_SCALAR_HIGH_OLD_TAG: &str = "scalar-high-old";
const STAGE1_SCALAR_HIGH_FINAL_TAG: &str = "scalar-high-final";
const STAGE1_SCALAR_APPENDED_TAG: &str = "scalar-appended";
const STAGE1_SCALAR_HIGH_OLD_HASH: &str = "000000000000d000";
const STAGE1_SCALAR_HIGH_NEW_HASH: &str = "000000000000d001";
const STAGE1_SCALAR_HIGH_FINAL_HASH: &str = "000000000000d002";
const STAGE1_SCALAR_APPENDED_HASH: &str = "000000000000d003";

// This opaque ID is the 129th base row (runtime row 128). Its numeric-looking
// suffix is only a caller label; the fixture does not model a dense million-row
// runtime-id space.

struct Stage1ScalarDeltaFixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    store: SegmentRdbStore,
    engine: Arc<Engine>,
    server: TestServer,
    base_name: String,
}

fn stage1_scalar_delta_base_id(index: usize) -> String {
    format!("scalar-base-{index:03}")
}

fn stage1_scalar_delta_base_number(index: usize) -> f64 {
    1_000.0 + index as f64
}

fn stage1_scalar_delta_base_tag(index: usize) -> String {
    format!("scalar-base-tag-{index:03}")
}

fn stage1_scalar_delta_base_hash(index: usize) -> String {
    format!("{:016x}", 0x1_000_u64 + index as u64)
}

fn stage1_scalar_delta_base_items() -> Vec<Value> {
    let mut items = Vec::with_capacity(STAGE1_SCALAR_BASE_ROWS * STAGE1_SCALAR_FIELDS.len());
    for index in 0..STAGE1_SCALAR_ORDINARY_BASE_ROWS {
        let external_id = stage1_scalar_delta_base_id(index);
        let tag = stage1_scalar_delta_base_tag(index);
        items.push(json!({
            "external_id": external_id.clone(),
            "field": STAGE1_SCALAR_NUMBER_FIELD,
            "value": stage1_scalar_delta_base_number(index),
        }));
        items.push(json!({
            "external_id": external_id.clone(),
            "field": STAGE1_SCALAR_SET_FIELD,
            "value": [tag],
        }));
        items.push(json!({
            "external_id": external_id,
            "field": STAGE1_SCALAR_HASH_FIELD,
            "value": stage1_scalar_delta_base_hash(index),
        }));
    }
    items.extend([
        json!({
            "external_id": STAGE1_SCALAR_HIGH_ID,
            "field": STAGE1_SCALAR_NUMBER_FIELD,
            "value": STAGE1_SCALAR_HIGH_OLD_NUMBER,
        }),
        json!({
            "external_id": STAGE1_SCALAR_HIGH_ID,
            "field": STAGE1_SCALAR_SET_FIELD,
            "value": [STAGE1_SCALAR_HIGH_OLD_TAG],
        }),
        json!({
            "external_id": STAGE1_SCALAR_HIGH_ID,
            "field": STAGE1_SCALAR_HASH_FIELD,
            "value": STAGE1_SCALAR_HIGH_OLD_HASH,
        }),
    ]);
    items
}

async fn stage1_scalar_delta_create_collection(server: &TestServer) {
    server
        .put(&format!("/collections/{STAGE1_SCALAR_DELTA_COLLECTION}"))
        .json(&json!({ "fields": {
            "num": { "type": "number" },
            "tags": { "type": "set" },
            "sig": { "type": "hash" },
        }}))
        .await
        .assert_status_ok();
}

async fn stage1_scalar_delta_post_items(server: &TestServer, items: Vec<Value>) {
    for chunk in items.chunks(1_000) {
        server
            .post(&format!(
                "/collections/{STAGE1_SCALAR_DELTA_COLLECTION}/index"
            ))
            .json(&json!({ "items": chunk }))
            .await
            .assert_status_ok();
    }
}

async fn stage1_scalar_delta_index_doc(
    server: &TestServer,
    external_id: &str,
    number: f64,
    tags: &[&str],
    hash: &str,
) {
    stage1_scalar_delta_post_items(
        server,
        vec![
            json!({
                "external_id": external_id,
                "field": STAGE1_SCALAR_NUMBER_FIELD,
                "value": number,
            }),
            json!({
                "external_id": external_id,
                "field": STAGE1_SCALAR_SET_FIELD,
                "value": tags,
            }),
            json!({
                "external_id": external_id,
                "field": STAGE1_SCALAR_HASH_FIELD,
                "value": hash,
            }),
        ],
    )
    .await;
}

async fn stage1_scalar_delta_replace_without_set(server: &TestServer, number: f64, hash: &str) {
    server
        .put(&format!(
            "/collections/{STAGE1_SCALAR_DELTA_COLLECTION}/docs:replace"
        ))
        .json(&json!({ "docs": [{
            "external_id": STAGE1_SCALAR_HIGH_ID,
            "fields": {
                "num": number,
                "sig": hash,
            },
        }]}))
        .await
        .assert_status_ok();
}

async fn stage1_scalar_delta_replace_full(server: &TestServer, number: f64, tag: &str, hash: &str) {
    server
        .put(&format!(
            "/collections/{STAGE1_SCALAR_DELTA_COLLECTION}/docs:replace"
        ))
        .json(&json!({ "docs": [{
            "external_id": STAGE1_SCALAR_HIGH_ID,
            "fields": {
                "num": number,
                "tags": [tag],
                "sig": hash,
            },
        }]}))
        .await
        .assert_status_ok();
}

async fn stage1_scalar_delta_delete_doc(server: &TestServer) {
    server
        .delete(&format!(
            "/collections/{STAGE1_SCALAR_DELTA_COLLECTION}/index/{STAGE1_SCALAR_DELETED_ID}"
        ))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);
}

async fn stage1_scalar_delta_fixture() -> Stage1ScalarDeltaFixture {
    let dir = tempfile::tempdir().expect("scalar sparse checkpoint root");
    let root = dir.path().join("segments");
    let store = SegmentRdbStore::new(&root).expect("create scalar segment store");
    let engine = Arc::new(Engine::new());
    let server =
        TestServer::new(router(AppState::open(engine.clone()))).expect("scalar sparse HTTP server");
    stage1_scalar_delta_create_collection(&server).await;
    stage1_scalar_delta_post_items(&server, stage1_scalar_delta_base_items()).await;
    stage1_restore_legacy_base(&engine, "scalar sparse-delta base");
    store
        .save(&engine, STAGE1_SCALAR_BASE_SEQUENCE)
        .expect("write scalar base generation");
    let base_name = stage1_reuse_current_name(&root);
    Stage1ScalarDeltaFixture {
        _dir: dir,
        root,
        store,
        engine,
        server,
        base_name,
    }
}

async fn stage1_scalar_delta_reference() -> TestServer {
    let engine = Arc::new(Engine::new());
    let server = TestServer::new(router(AppState::open(engine))).expect("scalar reference server");
    stage1_scalar_delta_create_collection(&server).await;
    stage1_scalar_delta_post_items(&server, stage1_scalar_delta_base_items()).await;
    server
}

async fn stage1_scalar_delta_apply_first(server: &TestServer) {
    // Full replacement deliberately omits `tags`, so the old Set value must be
    // masked by the delta rather than carried forward from its base layer.
    stage1_scalar_delta_replace_without_set(
        server,
        STAGE1_SCALAR_HIGH_NEW_NUMBER,
        STAGE1_SCALAR_HIGH_NEW_HASH,
    )
    .await;
    stage1_scalar_delta_delete_doc(server).await;
    stage1_scalar_delta_index_doc(
        server,
        STAGE1_SCALAR_APPENDED_ID,
        90_003.0,
        &[STAGE1_SCALAR_APPENDED_TAG],
        STAGE1_SCALAR_APPENDED_HASH,
    )
    .await;
}

async fn stage1_scalar_delta_apply_second(server: &TestServer) {
    stage1_scalar_delta_replace_full(
        server,
        STAGE1_SCALAR_HIGH_FINAL_NUMBER,
        STAGE1_SCALAR_HIGH_FINAL_TAG,
        STAGE1_SCALAR_HIGH_FINAL_HASH,
    )
    .await;
}

fn stage1_scalar_delta_number_query(value: f64) -> Value {
    json!({ "range": {
        "field": STAGE1_SCALAR_NUMBER_FIELD,
        "gte": value,
        "lte": value,
    }})
}

fn stage1_scalar_delta_set_query(value: &str) -> Value {
    json!({ "term": {
        "field": STAGE1_SCALAR_SET_FIELD,
        "value": value,
    }})
}

fn stage1_scalar_delta_hash_query(value: &str) -> Value {
    json!({ "hamming": {
        "field": STAGE1_SCALAR_HASH_FIELD,
        "hash": value,
        "max_distance": 0,
    }})
}

async fn stage1_scalar_delta_search_ids(server: &TestServer, query: Value) -> Vec<String> {
    let response = server
        .post(&format!(
            "/collections/{STAGE1_SCALAR_DELTA_COLLECTION}/search"
        ))
        .json(&json!({ "query": query, "limit": 32, "track_total": true }))
        .await;
    response.assert_status_ok();
    let body: Value = response.json();
    let mut ids: Vec<String> = body["hits"]
        .as_array()
        .expect("scalar search hits")
        .iter()
        .map(|hit| {
            hit["external_id"]
                .as_str()
                .expect("scalar hit ID")
                .to_owned()
        })
        .collect();
    ids.sort();
    assert_eq!(
        body["total"].as_u64(),
        Some(ids.len() as u64),
        "scalar search total must agree with the bounded fixture hits: {body}"
    );
    ids
}

async fn stage1_scalar_delta_assert_query(
    server: &TestServer,
    reference: &TestServer,
    query: Value,
    expected: &[&str],
    context: &str,
) {
    let mut expected: Vec<String> = expected.iter().map(|id| (*id).to_owned()).collect();
    expected.sort();
    let reference_ids = stage1_scalar_delta_search_ids(reference, query.clone()).await;
    assert_eq!(
        reference_ids, expected,
        "independently built reference has the expected {context} state"
    );
    let durable_ids = stage1_scalar_delta_search_ids(server, query).await;
    assert_eq!(
        durable_ids, reference_ids,
        "durable scalar layers must match independent public API state for {context}"
    );
}

async fn stage1_scalar_delta_assert_base_state(server: &TestServer, reference: &TestServer) {
    stage1_scalar_delta_assert_query(
        server,
        reference,
        stage1_scalar_delta_number_query(STAGE1_SCALAR_HIGH_OLD_NUMBER),
        &[STAGE1_SCALAR_HIGH_ID],
        "base high Number",
    )
    .await;
    stage1_scalar_delta_assert_query(
        server,
        reference,
        stage1_scalar_delta_set_query(STAGE1_SCALAR_HIGH_OLD_TAG),
        &[STAGE1_SCALAR_HIGH_ID],
        "base high Set",
    )
    .await;
    stage1_scalar_delta_assert_query(
        server,
        reference,
        stage1_scalar_delta_hash_query(STAGE1_SCALAR_HIGH_OLD_HASH),
        &[STAGE1_SCALAR_HIGH_ID],
        "base high Hash",
    )
    .await;
}

async fn stage1_scalar_delta_assert_first_state(server: &TestServer, reference: &TestServer) {
    for (query, expected, context) in [
        (
            stage1_scalar_delta_number_query(STAGE1_SCALAR_HIGH_NEW_NUMBER),
            vec![STAGE1_SCALAR_HIGH_ID],
            "first checkpoint updated Number",
        ),
        (
            stage1_scalar_delta_number_query(STAGE1_SCALAR_HIGH_OLD_NUMBER),
            vec![],
            "first checkpoint masks old Number",
        ),
        (
            stage1_scalar_delta_number_query(stage1_scalar_delta_base_number(1)),
            vec![],
            "first checkpoint masks deleted Number",
        ),
        (
            stage1_scalar_delta_number_query(stage1_scalar_delta_base_number(0)),
            vec![STAGE1_SCALAR_UNTOUCHED_ID],
            "first checkpoint retains untouched Number",
        ),
        (
            stage1_scalar_delta_number_query(90_003.0),
            vec![STAGE1_SCALAR_APPENDED_ID],
            "first checkpoint appends Number",
        ),
        (
            stage1_scalar_delta_set_query(STAGE1_SCALAR_HIGH_OLD_TAG),
            vec![],
            "full replacement omission masks old Set membership",
        ),
        (
            stage1_scalar_delta_set_query(&stage1_scalar_delta_base_tag(1)),
            vec![],
            "first checkpoint masks deleted Set membership",
        ),
        (
            stage1_scalar_delta_set_query(&stage1_scalar_delta_base_tag(0)),
            vec![STAGE1_SCALAR_UNTOUCHED_ID],
            "first checkpoint retains untouched Set membership",
        ),
        (
            stage1_scalar_delta_set_query(STAGE1_SCALAR_APPENDED_TAG),
            vec![STAGE1_SCALAR_APPENDED_ID],
            "first checkpoint appends Set membership",
        ),
        (
            stage1_scalar_delta_hash_query(STAGE1_SCALAR_HIGH_NEW_HASH),
            vec![STAGE1_SCALAR_HIGH_ID],
            "first checkpoint updated Hash",
        ),
        (
            stage1_scalar_delta_hash_query(STAGE1_SCALAR_HIGH_OLD_HASH),
            vec![],
            "first checkpoint masks old Hash",
        ),
        (
            stage1_scalar_delta_hash_query(&stage1_scalar_delta_base_hash(1)),
            vec![],
            "first checkpoint masks deleted Hash",
        ),
        (
            stage1_scalar_delta_hash_query(&stage1_scalar_delta_base_hash(0)),
            vec![STAGE1_SCALAR_UNTOUCHED_ID],
            "first checkpoint retains untouched Hash",
        ),
        (
            stage1_scalar_delta_hash_query(STAGE1_SCALAR_APPENDED_HASH),
            vec![STAGE1_SCALAR_APPENDED_ID],
            "first checkpoint appends Hash",
        ),
    ] {
        stage1_scalar_delta_assert_query(server, reference, query, &expected, context).await;
    }
}

async fn stage1_scalar_delta_assert_second_state(server: &TestServer, reference: &TestServer) {
    for (query, expected, context) in [
        (
            stage1_scalar_delta_number_query(STAGE1_SCALAR_HIGH_FINAL_NUMBER),
            vec![STAGE1_SCALAR_HIGH_ID],
            "second checkpoint newest Number",
        ),
        (
            stage1_scalar_delta_number_query(STAGE1_SCALAR_HIGH_NEW_NUMBER),
            vec![],
            "second checkpoint prevents Number resurrection",
        ),
        (
            stage1_scalar_delta_number_query(stage1_scalar_delta_base_number(1)),
            vec![],
            "second checkpoint keeps deleted Number masked",
        ),
        (
            stage1_scalar_delta_number_query(stage1_scalar_delta_base_number(0)),
            vec![STAGE1_SCALAR_UNTOUCHED_ID],
            "second checkpoint keeps untouched Number",
        ),
        (
            stage1_scalar_delta_number_query(90_003.0),
            vec![STAGE1_SCALAR_APPENDED_ID],
            "second checkpoint keeps appended Number",
        ),
        (
            stage1_scalar_delta_set_query(STAGE1_SCALAR_HIGH_OLD_TAG),
            vec![],
            "second checkpoint keeps omitted Set value masked",
        ),
        (
            stage1_scalar_delta_set_query(STAGE1_SCALAR_HIGH_FINAL_TAG),
            vec![STAGE1_SCALAR_HIGH_ID],
            "second checkpoint newest Set membership",
        ),
        (
            stage1_scalar_delta_set_query(&stage1_scalar_delta_base_tag(1)),
            vec![],
            "second checkpoint keeps deleted Set membership masked",
        ),
        (
            stage1_scalar_delta_set_query(&stage1_scalar_delta_base_tag(0)),
            vec![STAGE1_SCALAR_UNTOUCHED_ID],
            "second checkpoint keeps untouched Set membership",
        ),
        (
            stage1_scalar_delta_set_query(STAGE1_SCALAR_APPENDED_TAG),
            vec![STAGE1_SCALAR_APPENDED_ID],
            "second checkpoint keeps appended Set membership",
        ),
        (
            stage1_scalar_delta_hash_query(STAGE1_SCALAR_HIGH_FINAL_HASH),
            vec![STAGE1_SCALAR_HIGH_ID],
            "second checkpoint newest Hash",
        ),
        (
            stage1_scalar_delta_hash_query(STAGE1_SCALAR_HIGH_NEW_HASH),
            vec![],
            "second checkpoint prevents Hash resurrection",
        ),
        (
            stage1_scalar_delta_hash_query(&stage1_scalar_delta_base_hash(1)),
            vec![],
            "second checkpoint keeps deleted Hash masked",
        ),
        (
            stage1_scalar_delta_hash_query(&stage1_scalar_delta_base_hash(0)),
            vec![STAGE1_SCALAR_UNTOUCHED_ID],
            "second checkpoint keeps untouched Hash",
        ),
        (
            stage1_scalar_delta_hash_query(STAGE1_SCALAR_APPENDED_HASH),
            vec![STAGE1_SCALAR_APPENDED_ID],
            "second checkpoint keeps appended Hash",
        ),
    ] {
        stage1_scalar_delta_assert_query(server, reference, query, &expected, context).await;
    }
}

fn stage1_scalar_delta_collection(manifest: &Value) -> &Value {
    stage1_reuse_catalog_collection(manifest, STAGE1_SCALAR_DELTA_COLLECTION)
}

fn stage1_scalar_delta_field_refs<'a>(
    manifest: &'a Value,
    field: &str,
    kind: &str,
) -> Vec<&'a Value> {
    stage1_scalar_delta_collection(manifest)["segments"]
        .as_array()
        .expect("scalar catalog segments")
        .iter()
        .filter(|segment| {
            segment["role"] == json!("field")
                && segment["field"] == json!(field)
                && segment["kind"] == json!(kind)
        })
        .collect()
}

fn stage1_scalar_delta_field_ref<'a>(
    manifest: &'a Value,
    field: &str,
    kind: &str,
    ordinal: u64,
) -> &'a Value {
    stage1_scalar_delta_field_refs(manifest, field, kind)
        .into_iter()
        .find(|segment| segment["ordinal"] == json!(ordinal))
        .unwrap_or_else(|| panic!("scalar catalog needs {kind} ordinal {ordinal} for {field}"))
}

fn stage1_scalar_delta_for_sequence<'a>(
    manifest: &'a Value,
    field: &str,
    sequence: u64,
) -> &'a Value {
    let matching: Vec<_> = stage1_scalar_delta_field_refs(manifest, field, "delta")
        .into_iter()
        .filter(|segment| segment["applied_seq"] == json!(sequence))
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "scalar catalog must contain exactly one {field} delta for applied sequence {sequence}",
    );
    matching[0]
}

fn stage1_scalar_delta_first_touched_ids() -> std::collections::BTreeSet<String> {
    [
        STAGE1_SCALAR_HIGH_ID,
        STAGE1_SCALAR_DELETED_ID,
        STAGE1_SCALAR_APPENDED_ID,
    ]
    .iter()
    .map(|id| (*id).to_owned())
    .collect()
}

fn stage1_scalar_delta_second_touched_ids() -> std::collections::BTreeSet<String> {
    [STAGE1_SCALAR_HIGH_ID]
        .iter()
        .map(|id| (*id).to_owned())
        .collect()
}

fn stage1_scalar_delta_assert_sparse_rows(
    generation: &Path,
    field: &str,
    delta: &Value,
    expected_count: u64,
    expected_ids: std::collections::BTreeSet<String>,
) {
    assert_eq!(delta["role"], json!("field"));
    assert_eq!(delta["field"], json!(field));
    assert_eq!(delta["kind"], json!("delta"));
    assert_eq!(delta["format"], json!("lseg-v1"));
    assert!(
        delta["path"].as_str().is_some(),
        "{field} delta must name its segment through the catalog"
    );
    let count = stage1_keyword_delta_rows_count(delta);
    assert_eq!(
        count, expected_count,
        "{field} delta local rows must cover exactly its changed stable IDs"
    );
    assert!(
        count < STAGE1_SCALAR_BASE_ROWS as u64,
        "{field} delta must not allocate local rows for the full base cardinality"
    );
    let rows = stage1_keyword_delta_read_rows(generation, delta);
    assert_eq!(
        rows.len() as u64,
        count,
        "{field} local rows match their count"
    );
    let unique: std::collections::BTreeSet<String> = rows.iter().cloned().collect();
    assert_eq!(
        unique.len(),
        rows.len(),
        "{field} local rows must not repeat stable external IDs"
    );
    assert_eq!(
        unique, expected_ids,
        "{field} local rows name exactly the changed stable external IDs"
    );
}

fn stage1_scalar_delta_assert_first_catalog(generation: &Path, manifest: &Value) {
    for field in STAGE1_SCALAR_FIELDS {
        let deltas = stage1_scalar_delta_field_refs(manifest, field, "delta");
        assert_eq!(
            deltas.len(),
            1,
            "the first scalar checkpoint must publish one {field} delta"
        );
        let delta = stage1_scalar_delta_for_sequence(manifest, field, STAGE1_SCALAR_FIRST_SEQUENCE);
        assert_eq!(
            delta["ordinal"],
            json!(1),
            "first scalar {field} delta immediately follows the restored base",
        );
        stage1_scalar_delta_assert_sparse_rows(
            generation,
            field,
            delta,
            STAGE1_SCALAR_FIRST_TOUCHED_IDS,
            stage1_scalar_delta_first_touched_ids(),
        );
    }
}

fn stage1_scalar_delta_assert_second_catalog(generation: &Path, manifest: &Value) {
    for field in STAGE1_SCALAR_FIELDS {
        let deltas = stage1_scalar_delta_field_refs(manifest, field, "delta");
        assert_eq!(
            deltas.len(),
            2,
            "the second scalar checkpoint must retain ordinal-one and add ordinal-two {field} deltas"
        );
        let first = stage1_scalar_delta_for_sequence(manifest, field, STAGE1_SCALAR_FIRST_SEQUENCE);
        assert_eq!(
            first["ordinal"],
            json!(1),
            "first scalar {field} delta remains ordinal one",
        );
        stage1_scalar_delta_assert_sparse_rows(
            generation,
            field,
            first,
            STAGE1_SCALAR_FIRST_TOUCHED_IDS,
            stage1_scalar_delta_first_touched_ids(),
        );
        let second =
            stage1_scalar_delta_for_sequence(manifest, field, STAGE1_SCALAR_SECOND_SEQUENCE);
        assert_eq!(
            second["ordinal"],
            json!(2),
            "second scalar {field} delta is ordinal two",
        );
        stage1_scalar_delta_assert_sparse_rows(
            generation,
            field,
            second,
            STAGE1_SCALAR_SECOND_TOUCHED_IDS,
            stage1_scalar_delta_second_touched_ids(),
        );
    }
}

#[cfg(unix)]
fn stage1_scalar_delta_assert_base_hardlinked(
    base_generation: &Path,
    base_manifest: &Value,
    later_generation: &Path,
    later_manifest: &Value,
) {
    for field in STAGE1_SCALAR_FIELDS {
        let base = stage1_scalar_delta_field_ref(base_manifest, field, "base", 0);
        let retained = stage1_scalar_delta_field_ref(later_manifest, field, "base", 0);
        let base_path = base_generation.join(base["path"].as_str().expect("scalar base path"));
        let retained_path = later_generation.join(
            retained["path"]
                .as_str()
                .expect("retained scalar base path"),
        );
        assert_ne!(
            base_path, retained_path,
            "immutable scalar generations use distinct {field} paths"
        );
        let base_metadata = std::fs::symlink_metadata(&base_path).expect("inspect scalar base");
        let retained_metadata =
            std::fs::symlink_metadata(&retained_path).expect("inspect retained scalar base");
        assert!(base_metadata.is_file() && !base_metadata.file_type().is_symlink());
        assert!(retained_metadata.is_file() && !retained_metadata.file_type().is_symlink());
        assert_eq!(
            base_metadata.ino(),
            retained_metadata.ino(),
            "unchanged {field} base must be retained by hard link, not a copy"
        );
        assert!(
            retained_metadata.nlink() >= 2,
            "retained {field} base must report multiple hard links"
        );
    }
}

#[cfg(unix)]
#[tokio::test]
async fn v2_number_set_hash_sparse_deltas_match_independent_reference_across_two_checkpoints() {
    let fixture = stage1_scalar_delta_fixture().await;
    let reference = stage1_scalar_delta_reference().await;
    let base_generation = fixture.root.join(&fixture.base_name);
    let base_manifest = stage1_read_manifest(&base_generation);
    stage1_scalar_delta_assert_base_state(&fixture.server, &reference).await;

    stage1_scalar_delta_apply_first(&fixture.server).await;
    stage1_scalar_delta_apply_first(&reference).await;
    fixture
        .store
        .save(&fixture.engine, STAGE1_SCALAR_FIRST_SEQUENCE)
        .expect("write first scalar sparse checkpoint");
    stage1_scalar_delta_assert_first_state(&fixture.server, &reference).await;

    let first_name = stage1_reuse_current_name(&fixture.root);
    assert_ne!(
        first_name, fixture.base_name,
        "scalar mutations publish a new immutable checkpoint generation"
    );
    let first_generation = fixture.root.join(&first_name);
    let first_manifest = stage1_read_manifest(&first_generation);
    stage1_scalar_delta_assert_first_catalog(&first_generation, &first_manifest);
    stage1_scalar_delta_assert_base_hardlinked(
        &base_generation,
        &base_manifest,
        &first_generation,
        &first_manifest,
    );

    let (first_engine, first_sequence) = stage1_reuse_cold_load_current(&fixture.root);
    assert_eq!(
        first_sequence, STAGE1_SCALAR_FIRST_SEQUENCE,
        "cold reopen selects the first scalar checkpoint"
    );
    let first_server = TestServer::new(router(AppState::open(first_engine)))
        .expect("first scalar cold HTTP server");
    stage1_scalar_delta_assert_first_state(&first_server, &reference).await;

    stage1_scalar_delta_apply_second(&fixture.server).await;
    stage1_scalar_delta_apply_second(&reference).await;
    fixture
        .store
        .save(&fixture.engine, STAGE1_SCALAR_SECOND_SEQUENCE)
        .expect("write second scalar sparse checkpoint");
    stage1_scalar_delta_assert_second_state(&fixture.server, &reference).await;

    let latest_generation = stage1_current_generation_dir(&fixture.root);
    let latest_manifest = stage1_read_manifest(&latest_generation);
    stage1_scalar_delta_assert_second_catalog(&latest_generation, &latest_manifest);
    stage1_scalar_delta_assert_base_hardlinked(
        &base_generation,
        &base_manifest,
        &latest_generation,
        &latest_manifest,
    );

    let (latest_engine, latest_sequence) = stage1_reuse_cold_load_current(&fixture.root);
    assert_eq!(
        latest_sequence, STAGE1_SCALAR_SECOND_SEQUENCE,
        "cold reopen selects the second scalar checkpoint"
    );
    let latest_server = TestServer::new(router(AppState::open(latest_engine)))
        .expect("second scalar cold HTTP server");
    stage1_scalar_delta_assert_second_state(&latest_server, &reference).await;
}

#[tokio::test]
async fn v2_current_refuses_number_sparse_delta_duplicate_stable_local_rows() {
    let fixture = stage1_scalar_delta_fixture().await;
    stage1_scalar_delta_apply_first(&fixture.server).await;
    fixture
        .store
        .save(&fixture.engine, STAGE1_SCALAR_FIRST_SEQUENCE)
        .expect("write scalar delta before local-row corruption");
    let generation = stage1_current_generation_dir(&fixture.root);
    let manifest = stage1_read_manifest(&generation);
    let delta = stage1_scalar_delta_for_sequence(
        &manifest,
        STAGE1_SCALAR_NUMBER_FIELD,
        STAGE1_SCALAR_FIRST_SEQUENCE,
    );
    stage1_scalar_delta_assert_sparse_rows(
        &generation,
        STAGE1_SCALAR_NUMBER_FIELD,
        delta,
        STAGE1_SCALAR_FIRST_TOUCHED_IDS,
        stage1_scalar_delta_first_touched_ids(),
    );
    let rows_path = stage1_keyword_delta_rows_path(&generation, delta);
    let mut rows = stage1_keyword_delta_read_rows(&generation, delta);
    assert_eq!(
        rows.len(),
        STAGE1_SCALAR_FIRST_TOUCHED_IDS as usize,
        "duplicate local-row mutation starts from the three-ID Number contract"
    );
    rows[1] = rows[0].clone();
    stage1_keyword_delta_write_rows(&rows_path, &rows);

    stage1_assert_current_refuses_catalog_shape(
        &fixture.root,
        "a Number delta with duplicate stable external IDs in its local rows",
    );
}

// Scratch-only append for apps/lumen/e2e/indexing_durable_oracle.rs.
//
// Facets when applied:
// - Behavior: public one-token and two-token Match requests compare ordered
//   IDs and serialized BM25 scores with an independent, never-checkpointed
//   Engine across two checkpoints and cold opens. Public stats check document
//   counts and avg_doc_len.
// - Security: v2_current_refuses_number_sparse_delta_duplicate_stable_local_rows
//   already corrupts the shared lumen-local-eids-cbor-v1 LocalRows format and
//   requires a refusal without a CURRENT rewrite. Text uses that same shared
//   catalog descriptor and decoder, so another field-only duplicate test would
//   repeat that trust-boundary coverage.
// - Performance: exact three-ID local-row sidecars and inode equality prove
//   sparse layers and hard-link reuse. They do not measure latency or RSS; the
//   approved stage6 release workload owns those budgets.
//
// Text BM25 is exposed through Match. A Term query on a Text field is invalid,
// so the one-token Match is the public term-level scoring case.

const STAGE1_TEXT_DELTA_COLLECTION: &str = "text-sparse-delta";
const STAGE1_TEXT_FIELD: &str = "body";
const STAGE1_TEXT_ORDINARY_BASE_ROWS: usize = 128;
const STAGE1_TEXT_BASE_ROWS: usize = STAGE1_TEXT_ORDINARY_BASE_ROWS + 1;
const STAGE1_TEXT_BASE_SEQUENCE: u64 = 1_301;
const STAGE1_TEXT_FIRST_SEQUENCE: u64 = 1_302;
const STAGE1_TEXT_SECOND_SEQUENCE: u64 = 1_303;
const STAGE1_TEXT_UNTOUCHED_ID: &str = "text-base-000";
const STAGE1_TEXT_FIRST_DELETED_ID: &str = "text-base-001";
const STAGE1_TEXT_SECOND_DELETED_ID: &str = "text-base-002";
const STAGE1_TEXT_HIGH_ID: &str = "runtime-id-900000";
const STAGE1_TEXT_APPENDED_ID: &str = "runtime-id-900001";
const STAGE1_TEXT_COMMON: &str = "text-common";
const STAGE1_TEXT_SHARED: &str = "text-shared";
const STAGE1_TEXT_OLD: &str = "text-legacy";
const STAGE1_TEXT_UPDATED: &str = "text-updated";
const STAGE1_TEXT_APPENDED: &str = "text-appended";
const STAGE1_TEXT_FINAL: &str = "text-final";

struct Stage1TextDeltaFixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    store: SegmentRdbStore,
    engine: Arc<Engine>,
    server: TestServer,
    base_name: String,
}

fn stage1_text_delta_base_id(index: usize) -> String {
    format!("text-base-{index:03}")
}

fn stage1_text_delta_base_body(index: usize) -> String {
    format!("{STAGE1_TEXT_COMMON} {STAGE1_TEXT_SHARED} text-base-{index:03}")
}

fn stage1_text_delta_base_items() -> Vec<Value> {
    let mut items: Vec<Value> = (0..STAGE1_TEXT_ORDINARY_BASE_ROWS)
        .map(|index| {
            json!({
                "external_id": stage1_text_delta_base_id(index),
                "field": STAGE1_TEXT_FIELD,
                "value": stage1_text_delta_base_body(index),
            })
        })
        .collect();
    items.push(json!({
        "external_id": STAGE1_TEXT_HIGH_ID,
        "field": STAGE1_TEXT_FIELD,
        "value": format!("{STAGE1_TEXT_OLD} {STAGE1_TEXT_OLD} {STAGE1_TEXT_SHARED}"),
    }));
    items
}

async fn stage1_text_delta_create_collection(server: &TestServer) {
    server
        .put(&format!("/collections/{STAGE1_TEXT_DELTA_COLLECTION}"))
        .json(&json!({ "fields": {
            "body": { "type": "text", "analyzer": "whitespace_lower" },
        }}))
        .await
        .assert_status_ok();
}

async fn stage1_text_delta_post_items(server: &TestServer, items: Vec<Value>) {
    for chunk in items.chunks(1_000) {
        server
            .post(&format!(
                "/collections/{STAGE1_TEXT_DELTA_COLLECTION}/index"
            ))
            .json(&json!({ "items": chunk }))
            .await
            .assert_status_ok();
    }
}

async fn stage1_text_delta_index_body(server: &TestServer, external_id: &str, body: String) {
    stage1_text_delta_post_items(
        server,
        vec![json!({
            "external_id": external_id,
            "field": STAGE1_TEXT_FIELD,
            "value": body,
        })],
    )
    .await;
}

async fn stage1_text_delta_delete(server: &TestServer, external_id: &str) {
    server
        .delete(&format!(
            "/collections/{STAGE1_TEXT_DELTA_COLLECTION}/docs/{external_id}"
        ))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);
}

async fn stage1_text_delta_replace_without_body(server: &TestServer) {
    let response = server
        .put(&format!(
            "/collections/{STAGE1_TEXT_DELTA_COLLECTION}/docs:replace"
        ))
        .json(&json!({ "docs": [{
            "external_id": STAGE1_TEXT_HIGH_ID,
            "fields": {},
        }]}))
        .await;
    response.assert_status_ok();
    let body: Value = response.json();
    assert_eq!(
        body["results"][0]["status"], "ok",
        "full replacement omitting Text must succeed: {body}"
    );
}

async fn stage1_text_delta_fixture() -> Stage1TextDeltaFixture {
    let dir = tempfile::tempdir().expect("text sparse checkpoint root");
    let root = dir.path().join("segments");
    let store = SegmentRdbStore::new(&root).expect("create text segment store");
    let engine = Arc::new(Engine::new());
    let server =
        TestServer::new(router(AppState::open(engine.clone()))).expect("text sparse HTTP server");
    stage1_text_delta_create_collection(&server).await;
    stage1_text_delta_post_items(&server, stage1_text_delta_base_items()).await;
    stage1_restore_legacy_base(&engine, "Text sparse-delta base");
    store
        .save(&engine, STAGE1_TEXT_BASE_SEQUENCE)
        .expect("write Text base generation");
    let base_name = stage1_reuse_current_name(&root);
    Stage1TextDeltaFixture {
        _dir: dir,
        root,
        store,
        engine,
        server,
        base_name,
    }
}

async fn stage1_text_delta_reference() -> TestServer {
    let engine = Arc::new(Engine::new());
    let server =
        TestServer::new(router(AppState::open(engine))).expect("Text reference HTTP server");
    stage1_text_delta_create_collection(&server).await;
    stage1_text_delta_post_items(&server, stage1_text_delta_base_items()).await;
    server
}

async fn stage1_text_delta_apply_first(server: &TestServer) {
    stage1_text_delta_index_body(
        server,
        STAGE1_TEXT_HIGH_ID,
        format!(
            "{STAGE1_TEXT_UPDATED} {STAGE1_TEXT_UPDATED} {STAGE1_TEXT_UPDATED} {STAGE1_TEXT_SHARED}"
        ),
    )
    .await;
    stage1_text_delta_delete(server, STAGE1_TEXT_FIRST_DELETED_ID).await;
    stage1_text_delta_index_body(
        server,
        STAGE1_TEXT_APPENDED_ID,
        format!("{STAGE1_TEXT_APPENDED} {STAGE1_TEXT_APPENDED} {STAGE1_TEXT_SHARED}"),
    )
    .await;
}

async fn stage1_text_delta_apply_second_after_cold_open(server: &TestServer) {
    stage1_text_delta_index_body(
        server,
        STAGE1_TEXT_APPENDED_ID,
        format!("{STAGE1_TEXT_FINAL} {STAGE1_TEXT_FINAL} {STAGE1_TEXT_SHARED}"),
    )
    .await;
    stage1_text_delta_replace_without_body(server).await;
    stage1_text_delta_delete(server, STAGE1_TEXT_SECOND_DELETED_ID).await;
}

fn stage1_text_delta_match_query(text: &str) -> Value {
    json!({ "match": {
        "field": STAGE1_TEXT_FIELD,
        "text": text,
        "op": "and",
    }})
}

async fn stage1_text_delta_search(server: &TestServer, query: Value) -> Value {
    let response = server
        .post(&format!(
            "/collections/{STAGE1_TEXT_DELTA_COLLECTION}/search"
        ))
        .json(&json!({ "query": query, "limit": 512, "track_total": true }))
        .await;
    response.assert_status_ok();
    let body: Value = response.json();
    let hits = body["hits"].as_array().expect("Text search hits array");
    assert_eq!(
        body["total"].as_u64(),
        Some(hits.len() as u64),
        "bounded Text fixture returns every hit: {body}"
    );
    assert!(
        hits.iter().all(|hit| {
            hit["external_id"].as_str().is_some() && hit["score"].as_f64().is_some()
        }),
        "Text Match exposes IDs and serialized BM25 scores: {body}"
    );
    body
}

fn stage1_text_delta_hit_ids(body: &Value) -> Vec<String> {
    body["hits"]
        .as_array()
        .expect("Text search hits array")
        .iter()
        .map(|hit| hit["external_id"].as_str().expect("Text hit ID").to_owned())
        .collect()
}

async fn stage1_text_delta_assert_match(
    server: &TestServer,
    reference: &TestServer,
    text: &str,
    expected_ids: Vec<String>,
    context: &str,
) {
    let query = stage1_text_delta_match_query(text);
    let reference_body = stage1_text_delta_search(reference, query.clone()).await;
    let mut reference_ids = stage1_text_delta_hit_ids(&reference_body);
    reference_ids.sort();
    let mut expected_ids = expected_ids;
    expected_ids.sort();
    assert_eq!(
        reference_ids, expected_ids,
        "independent Text reference has expected {context} IDs"
    );

    let durable_body = stage1_text_delta_search(server, query).await;
    assert_eq!(
        durable_body["total"], reference_body["total"],
        "durable Text layers preserve {context} total"
    );
    assert_eq!(
        durable_body["hits"], reference_body["hits"],
        "durable Text layers preserve ordered {context} IDs and serialized BM25 scores"
    );
}

async fn stage1_text_delta_stats(server: &TestServer) -> Value {
    let response = server
        .get(&format!(
            "/collections/{STAGE1_TEXT_DELTA_COLLECTION}/stats"
        ))
        .await;
    response.assert_status_ok();
    response.json()
}

async fn stage1_text_delta_assert_stats(
    server: &TestServer,
    reference: &TestServer,
    expected_documents: u64,
    expected_avg_doc_len: f64,
    context: &str,
) {
    let reference_stats = stage1_text_delta_stats(reference).await;
    assert_eq!(
        reference_stats["documents_indexed"].as_u64(),
        Some(expected_documents),
        "independent reference has expected {context} document count"
    );
    let reference_avg = reference_stats["fields"][STAGE1_TEXT_FIELD]["avg_doc_len"]
        .as_f64()
        .expect("Text stats expose avg_doc_len");
    // avg_doc_len is a public f32, so only this representation check has a
    // small f32 serialization tolerance. BM25 hit scores above remain exact.
    assert!(
        (reference_avg - expected_avg_doc_len).abs() < 1e-6,
        "independent reference {context} avg_doc_len must be {expected_avg_doc_len}, got {reference_avg}"
    );

    let durable_stats = stage1_text_delta_stats(server).await;
    assert_eq!(
        durable_stats["documents_indexed"], reference_stats["documents_indexed"],
        "durable Text layers preserve {context} document count"
    );
    assert_eq!(
        durable_stats["fields"][STAGE1_TEXT_FIELD]["avg_doc_len"],
        reference_stats["fields"][STAGE1_TEXT_FIELD]["avg_doc_len"],
        "durable Text layers preserve {context} serialized avg_doc_len"
    );
}

fn stage1_text_delta_common_ids(excluded: &[&str]) -> Vec<String> {
    (0..STAGE1_TEXT_ORDINARY_BASE_ROWS)
        .map(stage1_text_delta_base_id)
        .filter(|id| !excluded.iter().any(|excluded| id == excluded))
        .collect()
}

async fn stage1_text_delta_assert_state(
    server: &TestServer,
    reference: &TestServer,
    queries: &[(&str, Vec<String>, &str)],
    expected_documents: u64,
    expected_avg_doc_len: f64,
    context: &str,
) {
    for (text, expected_ids, query_context) in queries {
        stage1_text_delta_assert_match(
            server,
            reference,
            text,
            expected_ids.clone(),
            query_context,
        )
        .await;
    }
    stage1_text_delta_assert_stats(
        server,
        reference,
        expected_documents,
        expected_avg_doc_len,
        context,
    )
    .await;
}

fn stage1_text_delta_refs<'a>(manifest: &'a Value, kind: &str) -> Vec<&'a Value> {
    stage1_reuse_catalog_collection(manifest, STAGE1_TEXT_DELTA_COLLECTION)["segments"]
        .as_array()
        .expect("Text catalog segments")
        .iter()
        .filter(|segment| {
            segment["role"] == json!("field")
                && segment["field"] == json!(STAGE1_TEXT_FIELD)
                && segment["kind"] == json!(kind)
        })
        .collect()
}

fn stage1_text_delta_ref<'a>(manifest: &'a Value, kind: &str, ordinal: u64) -> &'a Value {
    stage1_text_delta_refs(manifest, kind)
        .into_iter()
        .find(|segment| segment["ordinal"] == json!(ordinal))
        .unwrap_or_else(|| panic!("Text catalog needs {kind} ordinal {ordinal}"))
}

fn stage1_text_delta_for_sequence(manifest: &Value, sequence: u64) -> &Value {
    let matching: Vec<_> = stage1_text_delta_refs(manifest, "delta")
        .into_iter()
        .filter(|segment| segment["applied_seq"] == json!(sequence))
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "Text catalog must contain exactly one delta for applied sequence {sequence}",
    );
    matching[0]
}

fn stage1_text_delta_assert_delta(
    generation: &Path,
    delta: &Value,
    expected_ids: std::collections::BTreeSet<String>,
) {
    assert_eq!(delta["role"], json!("field"));
    assert_eq!(delta["field"], json!(STAGE1_TEXT_FIELD));
    assert_eq!(delta["kind"], json!("delta"));
    assert_eq!(delta["format"], json!("lseg-v1"));
    assert!(
        delta["path"].as_str().is_some(),
        "Text delta names a segment"
    );
    let count = stage1_keyword_delta_rows_count(delta);
    assert_eq!(
        count,
        expected_ids.len() as u64,
        "Text local rows cover exactly changed stable IDs"
    );
    assert!(
        count < STAGE1_TEXT_BASE_ROWS as u64,
        "Text local rows must stay sparse"
    );
    let rows = stage1_keyword_delta_read_rows(generation, delta);
    assert_eq!(
        rows.len() as u64,
        count,
        "Text local row count matches bytes"
    );
    let unique: std::collections::BTreeSet<String> = rows.iter().cloned().collect();
    assert_eq!(
        unique.len(),
        rows.len(),
        "Text local rows have no duplicate IDs"
    );
    assert_eq!(
        unique, expected_ids,
        "Text local rows name changed IDs exactly"
    );
}

#[cfg(unix)]
fn stage1_text_delta_assert_base_hardlinked(
    base_generation: &Path,
    base_manifest: &Value,
    later_generation: &Path,
    later_manifest: &Value,
) {
    let base = stage1_text_delta_ref(base_manifest, "base", 0);
    let retained = stage1_text_delta_ref(later_manifest, "base", 0);
    let base_path = base_generation.join(base["path"].as_str().expect("Text base path"));
    let retained_path =
        later_generation.join(retained["path"].as_str().expect("retained Text base path"));
    assert_ne!(
        base_path, retained_path,
        "immutable Text generations use new paths"
    );
    let base_metadata = std::fs::symlink_metadata(&base_path).expect("inspect Text base");
    let retained_metadata =
        std::fs::symlink_metadata(&retained_path).expect("inspect retained Text base");
    assert!(base_metadata.is_file() && !base_metadata.file_type().is_symlink());
    assert!(retained_metadata.is_file() && !retained_metadata.file_type().is_symlink());
    assert_eq!(
        base_metadata.ino(),
        retained_metadata.ino(),
        "unchanged Text base is retained by hard link, not a copy"
    );
    assert!(
        retained_metadata.nlink() >= 2,
        "retained Text base has multiple links"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn v2_text_sparse_delta_preserves_bm25_docs_and_cold_mutations() {
    let fixture = stage1_text_delta_fixture().await;
    let reference = stage1_text_delta_reference().await;
    let base_generation = fixture.root.join(&fixture.base_name);
    let base_manifest = stage1_read_manifest(&base_generation);
    stage1_text_delta_assert_state(
        &fixture.server,
        &reference,
        &[
            (
                STAGE1_TEXT_OLD,
                vec![STAGE1_TEXT_HIGH_ID.to_owned()],
                "base one-token Match",
            ),
            (
                &format!("{STAGE1_TEXT_COMMON} {STAGE1_TEXT_SHARED}"),
                stage1_text_delta_common_ids(&[]),
                "base two-token BM25 Match",
            ),
        ],
        129,
        3.0,
        "base Text corpus",
    )
    .await;

    stage1_text_delta_apply_first(&fixture.server).await;
    stage1_text_delta_apply_first(&reference).await;
    fixture
        .store
        .save(&fixture.engine, STAGE1_TEXT_FIRST_SEQUENCE)
        .expect("write first Text sparse checkpoint");
    let first_common = stage1_text_delta_common_ids(&[STAGE1_TEXT_FIRST_DELETED_ID]);
    stage1_text_delta_assert_state(
        &fixture.server,
        &reference,
        &[
            (STAGE1_TEXT_OLD, vec![], "first checkpoint masks old Text"),
            (
                STAGE1_TEXT_UPDATED,
                vec![STAGE1_TEXT_HIGH_ID.to_owned()],
                "first checkpoint updates Text",
            ),
            (
                STAGE1_TEXT_FIRST_DELETED_ID,
                vec![],
                "first checkpoint masks deleted Text",
            ),
            (
                STAGE1_TEXT_APPENDED,
                vec![STAGE1_TEXT_APPENDED_ID.to_owned()],
                "first checkpoint appends Text",
            ),
            (
                &format!("{STAGE1_TEXT_COMMON} {STAGE1_TEXT_SHARED}"),
                first_common,
                "first checkpoint two-token BM25 Match",
            ),
        ],
        129,
        388.0 / 129.0,
        "first checkpoint Text corpus",
    )
    .await;

    let first_name = stage1_reuse_current_name(&fixture.root);
    assert_ne!(
        first_name, fixture.base_name,
        "Text mutation creates a generation"
    );
    let first_generation = fixture.root.join(&first_name);
    let first_manifest = stage1_read_manifest(&first_generation);
    let first_delta = stage1_text_delta_for_sequence(&first_manifest, STAGE1_TEXT_FIRST_SEQUENCE);
    assert_eq!(
        stage1_text_delta_refs(&first_manifest, "delta").len(),
        1,
        "first Text checkpoint publishes one delta"
    );
    assert_eq!(
        first_delta["ordinal"],
        json!(1),
        "first Text delta immediately follows the restored base",
    );
    stage1_text_delta_assert_delta(
        &first_generation,
        first_delta,
        [
            STAGE1_TEXT_HIGH_ID,
            STAGE1_TEXT_FIRST_DELETED_ID,
            STAGE1_TEXT_APPENDED_ID,
        ]
        .iter()
        .map(|id| (*id).to_owned())
        .collect(),
    );
    stage1_text_delta_assert_base_hardlinked(
        &base_generation,
        &base_manifest,
        &first_generation,
        &first_manifest,
    );

    let (first_engine, first_sequence) = stage1_reuse_cold_load_current(&fixture.root);
    assert_eq!(first_sequence, STAGE1_TEXT_FIRST_SEQUENCE);
    let first_server = TestServer::new(router(AppState::open(first_engine.clone())))
        .expect("first Text cold HTTP server");
    let first_common = stage1_text_delta_common_ids(&[STAGE1_TEXT_FIRST_DELETED_ID]);
    stage1_text_delta_assert_state(
        &first_server,
        &reference,
        &[
            (
                STAGE1_TEXT_UPDATED,
                vec![STAGE1_TEXT_HIGH_ID.to_owned()],
                "cold first checkpoint update",
            ),
            (
                &format!("{STAGE1_TEXT_COMMON} {STAGE1_TEXT_SHARED}"),
                first_common,
                "cold first checkpoint BM25",
            ),
        ],
        129,
        388.0 / 129.0,
        "cold first checkpoint Text corpus",
    )
    .await;

    stage1_text_delta_apply_second_after_cold_open(&first_server).await;
    stage1_text_delta_apply_second_after_cold_open(&reference).await;
    SegmentRdbStore::new(&fixture.root)
        .expect("reopen Text store after cold mutations")
        .save(&first_engine, STAGE1_TEXT_SECOND_SEQUENCE)
        .expect("write second Text sparse checkpoint");
    let second_common = stage1_text_delta_common_ids(&[
        STAGE1_TEXT_FIRST_DELETED_ID,
        STAGE1_TEXT_SECOND_DELETED_ID,
    ]);
    stage1_text_delta_assert_state(
        &first_server,
        &reference,
        &[
            (
                STAGE1_TEXT_UPDATED,
                vec![],
                "second checkpoint masks Text omitted by replacement",
            ),
            (
                STAGE1_TEXT_APPENDED,
                vec![],
                "second checkpoint masks old post-cold Text",
            ),
            (
                STAGE1_TEXT_FINAL,
                vec![STAGE1_TEXT_APPENDED_ID.to_owned()],
                "second checkpoint keeps post-cold Text update",
            ),
            (
                STAGE1_TEXT_SECOND_DELETED_ID,
                vec![],
                "second checkpoint masks post-cold deletion",
            ),
            (
                &format!("{STAGE1_TEXT_COMMON} {STAGE1_TEXT_SHARED}"),
                second_common,
                "second checkpoint two-token BM25 Match",
            ),
        ],
        127,
        3.0,
        "second checkpoint Text corpus",
    )
    .await;

    let latest_generation = stage1_current_generation_dir(&fixture.root);
    let latest_manifest = stage1_read_manifest(&latest_generation);
    assert_eq!(
        stage1_text_delta_refs(&latest_manifest, "delta").len(),
        2,
        "second Text checkpoint retains ordinal one and adds ordinal two"
    );
    for (ordinal, sequence, expected_ids) in [
        (
            1,
            STAGE1_TEXT_FIRST_SEQUENCE,
            [
                STAGE1_TEXT_HIGH_ID,
                STAGE1_TEXT_FIRST_DELETED_ID,
                STAGE1_TEXT_APPENDED_ID,
            ],
        ),
        (
            2,
            STAGE1_TEXT_SECOND_SEQUENCE,
            [
                STAGE1_TEXT_HIGH_ID,
                STAGE1_TEXT_SECOND_DELETED_ID,
                STAGE1_TEXT_APPENDED_ID,
            ],
        ),
    ] {
        let delta = stage1_text_delta_for_sequence(&latest_manifest, sequence);
        assert_eq!(
            delta["ordinal"],
            json!(ordinal),
            "Text delta for applied sequence {sequence} keeps ordinal {ordinal}",
        );
        stage1_text_delta_assert_delta(
            &latest_generation,
            delta,
            expected_ids.iter().map(|id| (*id).to_owned()).collect(),
        );
    }
    stage1_text_delta_assert_base_hardlinked(
        &base_generation,
        &base_manifest,
        &latest_generation,
        &latest_manifest,
    );

    let (latest_engine, latest_sequence) = stage1_reuse_cold_load_current(&fixture.root);
    assert_eq!(latest_sequence, STAGE1_TEXT_SECOND_SEQUENCE);
    let latest_server = TestServer::new(router(AppState::open(latest_engine)))
        .expect("second Text cold HTTP server");
    let second_common = stage1_text_delta_common_ids(&[
        STAGE1_TEXT_FIRST_DELETED_ID,
        STAGE1_TEXT_SECOND_DELETED_ID,
    ]);
    stage1_text_delta_assert_state(
        &latest_server,
        &reference,
        &[
            (
                STAGE1_TEXT_FINAL,
                vec![STAGE1_TEXT_APPENDED_ID.to_owned()],
                "cold second checkpoint post-cold update",
            ),
            (
                &format!("{STAGE1_TEXT_COMMON} {STAGE1_TEXT_SHARED}"),
                second_common,
                "cold second checkpoint BM25",
            ),
        ],
        127,
        3.0,
        "cold second checkpoint Text corpus",
    )
    .await;
}
// Scratch-only append for apps/lumen/e2e/indexing_durable_oracle.rs.
//
// Facets when applied:
// - Behavior: the two backend wrappers below call one parameterized contract.
//   It drives public vector indexing and kNN, then checks v2 sparse local rows,
//   hard-linked base files, cold reopen, and a second ordered layer.
// - Security: this contract uses the existing v2 local-row descriptor. The
//   established duplicate-local-row refusal in
//   `v2_current_refuses_number_sparse_delta_duplicate_stable_local_rows` covers
//   the shared CBOR decoder and CURRENT preservation. Vector-specific corrupt
//   payload refusal remains a later catalog-validation case once a vector delta
//   reader exists.
// - Performance: three local rows and inode equality are structure assertions,
//   not a latency or RSS claim. The approved stage6 30-minute release workload
//   owns that budget; this structural case does not run it. The controller-owned
//   vector scan-counter unit seam covers full-corpus enumeration separately.
//
// The high external ID is an opaque caller label. It is the 129th base row, not
// a dense runtime row numbered 900000.

const STAGE1_VECTOR_DELTA_COLLECTION: &str = "vector-sparse-delta";
const STAGE1_VECTOR_DELTA_FIELD: &str = "embedding";
const STAGE1_VECTOR_DELTA_ORDINARY_BASE_ROWS: usize = 128;
const STAGE1_VECTOR_DELTA_BASE_ROWS: usize = STAGE1_VECTOR_DELTA_ORDINARY_BASE_ROWS + 1;
const STAGE1_VECTOR_DELTA_BASE_SEQUENCE: u64 = 1_401;
const STAGE1_VECTOR_DELTA_FIRST_SEQUENCE: u64 = 1_402;
const STAGE1_VECTOR_DELTA_SECOND_SEQUENCE: u64 = 1_403;
const STAGE1_VECTOR_DELTA_HIGH_ID: &str = "runtime-id-900000";
const STAGE1_VECTOR_DELTA_APPENDED_ID: &str = "runtime-id-900001";
const STAGE1_VECTOR_DELTA_FIRST_DELETED_ID: &str = "vector-base-002";
const STAGE1_VECTOR_DELTA_SECOND_DELETED_ID: &str = "vector-base-004";

struct Stage1VectorDeltaFixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    store: SegmentRdbStore,
    engine: Arc<Engine>,
    server: TestServer,
    base_name: String,
}

fn stage1_vector_delta_base_id(index: usize) -> String {
    format!("vector-base-{index:03}")
}

fn stage1_vector_delta_base_x(index: usize) -> f32 {
    match index {
        // Each anchor is only 0.1 from one changed or deleted vector. This
        // gives a clear nearest result without relying on HNSW score values.
        0 => 0.1,
        1 => 10.1,
        2 => 20.0,
        3 => 20.1,
        4 => 30.0,
        5 => 30.1,
        6 => 40.1,
        _ => 1_000.0 + index as f32,
    }
}

fn stage1_vector_delta_vector(x: f32) -> Value {
    json!([x, 0.0])
}

fn stage1_vector_delta_base_items() -> Vec<Value> {
    let mut items: Vec<Value> = (0..STAGE1_VECTOR_DELTA_ORDINARY_BASE_ROWS)
        .map(|index| {
            json!({
                "external_id": stage1_vector_delta_base_id(index),
                "field": STAGE1_VECTOR_DELTA_FIELD,
                "value": stage1_vector_delta_vector(stage1_vector_delta_base_x(index)),
            })
        })
        .collect();
    items.push(json!({
        "external_id": STAGE1_VECTOR_DELTA_HIGH_ID,
        "field": STAGE1_VECTOR_DELTA_FIELD,
        "value": stage1_vector_delta_vector(0.0),
    }));
    items
}

async fn stage1_vector_delta_create_collection(server: &TestServer, backend: &str) {
    server
        .put(&format!("/collections/{STAGE1_VECTOR_DELTA_COLLECTION}"))
        .json(&json!({ "fields": {
            STAGE1_VECTOR_DELTA_FIELD: {
                "type": "vector",
                "dim": 2,
                "metric": "l2",
                "backend": backend,
            },
        }}))
        .await
        .assert_status_ok();
}

async fn stage1_vector_delta_post_items(server: &TestServer, items: Vec<Value>) {
    for chunk in items.chunks(1_000) {
        server
            .post(&format!(
                "/collections/{STAGE1_VECTOR_DELTA_COLLECTION}/index"
            ))
            .json(&json!({ "items": chunk }))
            .await
            .assert_status_ok();
    }
}

async fn stage1_vector_delta_index(server: &TestServer, external_id: &str, x: f32) {
    stage1_vector_delta_post_items(
        server,
        vec![json!({
            "external_id": external_id,
            "field": STAGE1_VECTOR_DELTA_FIELD,
            "value": stage1_vector_delta_vector(x),
        })],
    )
    .await;
}

async fn stage1_vector_delta_delete(server: &TestServer, external_id: &str) {
    server
        .delete(&format!(
            "/collections/{STAGE1_VECTOR_DELTA_COLLECTION}/index/{external_id}"
        ))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);
}

async fn stage1_vector_delta_fixture(backend: &str) -> Stage1VectorDeltaFixture {
    let dir = tempfile::tempdir().expect("vector sparse checkpoint root");
    let root = dir.path().join("segments");
    let store = SegmentRdbStore::new(&root).expect("create vector segment store");
    let engine = Arc::new(Engine::new());
    let server =
        TestServer::new(router(AppState::open(engine.clone()))).expect("vector sparse HTTP server");
    stage1_vector_delta_create_collection(&server, backend).await;
    stage1_vector_delta_post_items(&server, stage1_vector_delta_base_items()).await;
    stage1_restore_legacy_base(&engine, "vector sparse-delta base");
    store
        .save(&engine, STAGE1_VECTOR_DELTA_BASE_SEQUENCE)
        .expect("write vector base generation");
    let base_name = stage1_reuse_current_name(&root);
    Stage1VectorDeltaFixture {
        _dir: dir,
        root,
        store,
        engine,
        server,
        base_name,
    }
}

async fn stage1_vector_delta_reference(backend: &str) -> TestServer {
    let engine = Arc::new(Engine::new());
    let server = TestServer::new(router(AppState::open(engine))).expect("vector reference server");
    stage1_vector_delta_create_collection(&server, backend).await;
    stage1_vector_delta_post_items(&server, stage1_vector_delta_base_items()).await;
    server
}

async fn stage1_vector_delta_apply_first(server: &TestServer) {
    stage1_vector_delta_index(server, STAGE1_VECTOR_DELTA_HIGH_ID, 10.0).await;
    stage1_vector_delta_delete(server, STAGE1_VECTOR_DELTA_FIRST_DELETED_ID).await;
    stage1_vector_delta_index(server, STAGE1_VECTOR_DELTA_APPENDED_ID, 40.0).await;
}

async fn stage1_vector_delta_apply_second_after_cold_open(server: &TestServer) {
    stage1_vector_delta_index(server, STAGE1_VECTOR_DELTA_HIGH_ID, 60.0).await;
    stage1_vector_delta_delete(server, STAGE1_VECTOR_DELTA_SECOND_DELETED_ID).await;
    stage1_vector_delta_index(server, STAGE1_VECTOR_DELTA_APPENDED_ID, 50.0).await;
}

async fn stage1_vector_delta_knn_ids(server: &TestServer, x: f32, k: usize) -> Vec<String> {
    let response = server
        .post(&format!(
            "/collections/{STAGE1_VECTOR_DELTA_COLLECTION}/search"
        ))
        .json(&json!({
            "query": { "knn": {
                "field": STAGE1_VECTOR_DELTA_FIELD,
                "vector": stage1_vector_delta_vector(x),
                "k": k,
            }},
            "limit": k,
            "track_total": true,
        }))
        .await;
    response.assert_status_ok();
    let body: Value = response.json();
    body["hits"]
        .as_array()
        .expect("vector kNN hits array")
        .iter()
        .map(|hit| {
            hit["external_id"]
                .as_str()
                .expect("vector hit external ID")
                .to_owned()
        })
        .collect()
}

async fn stage1_vector_delta_assert_nearest(
    server: &TestServer,
    reference: &TestServer,
    x: f32,
    expected: &str,
    context: &str,
) {
    let reference_ids = stage1_vector_delta_knn_ids(reference, x, 1).await;
    assert_eq!(
        reference_ids,
        vec![expected.to_owned()],
        "independent vector reference has the expected nearest ID for {context}"
    );
    assert_eq!(
        stage1_vector_delta_knn_ids(server, x, 1).await,
        reference_ids,
        "durable vector layers preserve the nearest ID for {context} without claiming a score"
    );
}

async fn stage1_vector_delta_assert_live_ids(
    server: &TestServer,
    reference: &TestServer,
    expected: &std::collections::BTreeSet<String>,
    context: &str,
) {
    let reference_ids: std::collections::BTreeSet<String> =
        stage1_vector_delta_knn_ids(reference, 0.0, STAGE1_VECTOR_DELTA_BASE_ROWS)
            .await
            .into_iter()
            .collect();
    assert_eq!(
        reference_ids, *expected,
        "independent vector reference has exactly the expected live IDs for {context}"
    );
    let durable_ids: std::collections::BTreeSet<String> =
        stage1_vector_delta_knn_ids(server, 0.0, STAGE1_VECTOR_DELTA_BASE_ROWS)
            .await
            .into_iter()
            .collect();
    assert_eq!(
        durable_ids, reference_ids,
        "durable vector layers retain and mask the same IDs for {context}"
    );
}

async fn stage1_vector_delta_assert_backend(server: &TestServer, backend: &str, context: &str) {
    let response = server.get("/admin/backup").await;
    response.assert_status_ok();
    let snapshot: Value = response.json();
    assert_eq!(
        snapshot["collections"][STAGE1_VECTOR_DELTA_COLLECTION]["fields"]
            [STAGE1_VECTOR_DELTA_FIELD]["spec"]["backend"],
        json!(backend),
        "cold and live vector schema must retain backend {backend} for {context}"
    );
}

fn stage1_vector_delta_base_ids() -> std::collections::BTreeSet<String> {
    (0..STAGE1_VECTOR_DELTA_ORDINARY_BASE_ROWS)
        .map(stage1_vector_delta_base_id)
        .chain(std::iter::once(STAGE1_VECTOR_DELTA_HIGH_ID.to_owned()))
        .collect()
}

fn stage1_vector_delta_first_ids() -> std::collections::BTreeSet<String> {
    let mut ids = stage1_vector_delta_base_ids();
    ids.remove(STAGE1_VECTOR_DELTA_FIRST_DELETED_ID);
    ids.insert(STAGE1_VECTOR_DELTA_APPENDED_ID.to_owned());
    ids
}

fn stage1_vector_delta_second_ids() -> std::collections::BTreeSet<String> {
    let mut ids = stage1_vector_delta_first_ids();
    ids.remove(STAGE1_VECTOR_DELTA_SECOND_DELETED_ID);
    ids
}

fn stage1_vector_delta_field_refs<'a>(manifest: &'a Value, kind: &str) -> Vec<&'a Value> {
    stage1_reuse_catalog_collection(manifest, STAGE1_VECTOR_DELTA_COLLECTION)["segments"]
        .as_array()
        .expect("vector catalog segments")
        .iter()
        .filter(|segment| {
            segment["role"] == json!("field")
                && segment["field"] == json!(STAGE1_VECTOR_DELTA_FIELD)
                && segment["kind"] == json!(kind)
        })
        .collect()
}

fn stage1_vector_delta_for_sequence(manifest: &Value, sequence: u64) -> &Value {
    let matching: Vec<_> = stage1_vector_delta_field_refs(manifest, "delta")
        .into_iter()
        .filter(|segment| segment["applied_seq"] == json!(sequence))
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "vector catalog must contain exactly one delta for applied sequence {sequence}",
    );
    matching[0]
}

fn stage1_vector_delta_base_ref<'a>(manifest: &'a Value, role: &str) -> &'a Value {
    stage1_reuse_catalog_collection(manifest, STAGE1_VECTOR_DELTA_COLLECTION)["segments"]
        .as_array()
        .expect("vector catalog segments")
        .iter()
        .find(|segment| {
            segment["role"] == json!(role)
                && segment["field"] == json!(STAGE1_VECTOR_DELTA_FIELD)
                && segment["kind"] == json!("base")
                && segment["ordinal"] == json!(0)
        })
        .unwrap_or_else(|| panic!("vector catalog needs base {role} reference"))
}

fn stage1_vector_delta_touched_ids() -> std::collections::BTreeSet<String> {
    [
        STAGE1_VECTOR_DELTA_HIGH_ID,
        STAGE1_VECTOR_DELTA_FIRST_DELETED_ID,
        STAGE1_VECTOR_DELTA_APPENDED_ID,
    ]
    .iter()
    .map(|id| (*id).to_owned())
    .collect()
}

fn stage1_vector_delta_second_touched_ids() -> std::collections::BTreeSet<String> {
    [
        STAGE1_VECTOR_DELTA_HIGH_ID,
        STAGE1_VECTOR_DELTA_SECOND_DELETED_ID,
        STAGE1_VECTOR_DELTA_APPENDED_ID,
    ]
    .iter()
    .map(|id| (*id).to_owned())
    .collect()
}

fn stage1_vector_delta_assert_delta(
    generation: &Path,
    delta: &Value,
    expected_ids: std::collections::BTreeSet<String>,
) {
    assert_eq!(delta["role"], json!("field"));
    assert_eq!(delta["field"], json!(STAGE1_VECTOR_DELTA_FIELD));
    assert_eq!(delta["kind"], json!("delta"));
    assert_eq!(delta["format"], json!("lseg-v1"));
    assert!(
        delta["path"].as_str().is_some(),
        "vector delta names a segment"
    );
    let count = stage1_keyword_delta_rows_count(delta);
    assert_eq!(
        count, 3,
        "vector delta local rows must cover exactly update, delete, and append"
    );
    assert!(
        count < STAGE1_VECTOR_DELTA_BASE_ROWS as u64,
        "vector delta must not allocate local rows for every base vector"
    );
    let rows = stage1_keyword_delta_read_rows(generation, delta);
    let unique: std::collections::BTreeSet<String> = rows.iter().cloned().collect();
    assert_eq!(
        rows.len() as u64,
        count,
        "vector local rows match their count"
    );
    assert_eq!(
        unique.len(),
        rows.len(),
        "vector local rows must not repeat stable external IDs"
    );
    assert_eq!(
        unique, expected_ids,
        "vector local rows must name exactly the vectors changed in this layer"
    );
}

#[cfg(unix)]
fn stage1_vector_delta_assert_base_hardlinked(
    base_generation: &Path,
    base_manifest: &Value,
    later_generation: &Path,
    later_manifest: &Value,
) {
    for role in ["field", "vector_eids"] {
        let base = stage1_vector_delta_base_ref(base_manifest, role);
        let retained = stage1_vector_delta_base_ref(later_manifest, role);
        let base_path = base_generation.join(base["path"].as_str().expect("vector base path"));
        let retained_path = later_generation.join(
            retained["path"]
                .as_str()
                .expect("retained vector base path"),
        );
        let base_metadata = std::fs::symlink_metadata(&base_path).expect("inspect vector base");
        let retained_metadata =
            std::fs::symlink_metadata(&retained_path).expect("inspect retained vector base");
        assert!(base_metadata.is_file() && !base_metadata.file_type().is_symlink());
        assert!(retained_metadata.is_file() && !retained_metadata.file_type().is_symlink());
        assert_eq!(
            base_metadata.ino(),
            retained_metadata.ino(),
            "unchanged vector {role} base is retained by hard link, not a copy"
        );
        assert!(
            retained_metadata.nlink() >= 2,
            "retained vector {role} base has multiple links"
        );
    }
}

#[cfg(unix)]
async fn stage1_vector_delta_contract(backend: &str) {
    let fixture = stage1_vector_delta_fixture(backend).await;
    let reference = stage1_vector_delta_reference(backend).await;
    let base_generation = fixture.root.join(&fixture.base_name);
    let base_manifest = stage1_read_manifest(&base_generation);
    stage1_vector_delta_assert_backend(&fixture.server, backend, "base checkpoint").await;
    stage1_vector_delta_assert_live_ids(
        &fixture.server,
        &reference,
        &stage1_vector_delta_base_ids(),
        "base checkpoint",
    )
    .await;
    stage1_vector_delta_assert_nearest(
        &fixture.server,
        &reference,
        0.0,
        STAGE1_VECTOR_DELTA_HIGH_ID,
        "base high vector",
    )
    .await;

    stage1_vector_delta_apply_first(&fixture.server).await;
    stage1_vector_delta_apply_first(&reference).await;
    fixture
        .store
        .save(&fixture.engine, STAGE1_VECTOR_DELTA_FIRST_SEQUENCE)
        .expect("write first vector sparse checkpoint");
    let first_name = stage1_reuse_current_name(&fixture.root);
    assert_ne!(
        first_name, fixture.base_name,
        "vector mutation creates a generation"
    );
    let first_generation = fixture.root.join(&first_name);
    let first_manifest = stage1_read_manifest(&first_generation);
    assert_eq!(
        stage1_vector_delta_field_refs(&first_manifest, "delta").len(),
        1,
        "first vector checkpoint publishes one delta"
    );
    let first_delta =
        stage1_vector_delta_for_sequence(&first_manifest, STAGE1_VECTOR_DELTA_FIRST_SEQUENCE);
    assert_eq!(
        first_delta["ordinal"],
        json!(1),
        "first vector delta immediately follows the restored base",
    );
    stage1_vector_delta_assert_delta(
        &first_generation,
        first_delta,
        stage1_vector_delta_touched_ids(),
    );
    stage1_vector_delta_assert_base_hardlinked(
        &base_generation,
        &base_manifest,
        &first_generation,
        &first_manifest,
    );
    stage1_vector_delta_assert_backend(&fixture.server, backend, "first live checkpoint").await;
    stage1_vector_delta_assert_live_ids(
        &fixture.server,
        &reference,
        &stage1_vector_delta_first_ids(),
        "first live checkpoint",
    )
    .await;
    for (x, expected, context) in [
        (
            0.0,
            "vector-base-000",
            "first checkpoint masks old high vector",
        ),
        (
            20.0,
            "vector-base-003",
            "first checkpoint masks deleted vector",
        ),
        (
            10.0,
            STAGE1_VECTOR_DELTA_HIGH_ID,
            "first checkpoint retains updated high vector",
        ),
        (
            40.0,
            STAGE1_VECTOR_DELTA_APPENDED_ID,
            "first checkpoint retains appended vector",
        ),
        (
            stage1_vector_delta_base_x(7),
            "vector-base-007",
            "first checkpoint retains untouched base vector",
        ),
    ] {
        stage1_vector_delta_assert_nearest(&fixture.server, &reference, x, expected, context).await;
    }

    let (first_engine, first_sequence) = stage1_reuse_cold_load_current(&fixture.root);
    assert_eq!(first_sequence, STAGE1_VECTOR_DELTA_FIRST_SEQUENCE);
    let first_server = TestServer::new(router(AppState::open(first_engine.clone())))
        .expect("first vector cold HTTP server");
    stage1_vector_delta_assert_backend(&first_server, backend, "first cold checkpoint").await;
    stage1_vector_delta_assert_live_ids(
        &first_server,
        &reference,
        &stage1_vector_delta_first_ids(),
        "first cold checkpoint",
    )
    .await;
    stage1_vector_delta_assert_nearest(
        &first_server,
        &reference,
        10.0,
        STAGE1_VECTOR_DELTA_HIGH_ID,
        "first cold checkpoint updated high vector",
    )
    .await;

    stage1_vector_delta_apply_second_after_cold_open(&first_server).await;
    stage1_vector_delta_apply_second_after_cold_open(&reference).await;
    SegmentRdbStore::new(&fixture.root)
        .expect("reopen vector store after cold mutations")
        .save(&first_engine, STAGE1_VECTOR_DELTA_SECOND_SEQUENCE)
        .expect("write second vector sparse checkpoint");
    let latest_generation = stage1_current_generation_dir(&fixture.root);
    let latest_manifest = stage1_read_manifest(&latest_generation);
    let deltas = stage1_vector_delta_field_refs(&latest_manifest, "delta");
    assert_eq!(
        deltas.len(),
        2,
        "second vector checkpoint retains ordinal one and adds ordinal two"
    );
    let ordinals: Vec<u64> = deltas
        .iter()
        .map(|segment| segment["ordinal"].as_u64().expect("vector delta ordinal"))
        .collect();
    assert_eq!(ordinals, [1, 2], "vector delta layers remain ordered");
    let first_delta =
        stage1_vector_delta_for_sequence(&latest_manifest, STAGE1_VECTOR_DELTA_FIRST_SEQUENCE);
    let second_delta =
        stage1_vector_delta_for_sequence(&latest_manifest, STAGE1_VECTOR_DELTA_SECOND_SEQUENCE);
    assert_eq!(first_delta["ordinal"], json!(1));
    assert_eq!(second_delta["ordinal"], json!(2));
    stage1_vector_delta_assert_delta(
        &latest_generation,
        first_delta,
        stage1_vector_delta_touched_ids(),
    );
    stage1_vector_delta_assert_delta(
        &latest_generation,
        second_delta,
        stage1_vector_delta_second_touched_ids(),
    );
    stage1_vector_delta_assert_base_hardlinked(
        &base_generation,
        &base_manifest,
        &latest_generation,
        &latest_manifest,
    );
    stage1_vector_delta_assert_live_ids(
        &first_server,
        &reference,
        &stage1_vector_delta_second_ids(),
        "second live checkpoint",
    )
    .await;
    for (x, expected, context) in [
        (
            10.0,
            "vector-base-001",
            "second checkpoint masks old high vector",
        ),
        (
            30.0,
            "vector-base-005",
            "second checkpoint masks post-cold deleted vector",
        ),
        (
            40.0,
            "vector-base-006",
            "second checkpoint masks old appended vector",
        ),
        (
            60.0,
            STAGE1_VECTOR_DELTA_HIGH_ID,
            "second checkpoint retains post-cold high update",
        ),
        (
            50.0,
            STAGE1_VECTOR_DELTA_APPENDED_ID,
            "second checkpoint retains post-cold appended update",
        ),
        (
            stage1_vector_delta_base_x(7),
            "vector-base-007",
            "second checkpoint retains untouched base vector",
        ),
    ] {
        stage1_vector_delta_assert_nearest(&first_server, &reference, x, expected, context).await;
    }

    let (latest_engine, latest_sequence) = stage1_reuse_cold_load_current(&fixture.root);
    assert_eq!(latest_sequence, STAGE1_VECTOR_DELTA_SECOND_SEQUENCE);
    let latest_server = TestServer::new(router(AppState::open(latest_engine)))
        .expect("second vector cold HTTP server");
    stage1_vector_delta_assert_backend(&latest_server, backend, "second cold checkpoint").await;
    stage1_vector_delta_assert_live_ids(
        &latest_server,
        &reference,
        &stage1_vector_delta_second_ids(),
        "second cold checkpoint",
    )
    .await;
    for (x, expected, context) in [
        (60.0, STAGE1_VECTOR_DELTA_HIGH_ID, "second cold high update"),
        (
            50.0,
            STAGE1_VECTOR_DELTA_APPENDED_ID,
            "second cold appended update",
        ),
        (
            stage1_vector_delta_base_x(7),
            "vector-base-007",
            "second cold untouched base vector",
        ),
    ] {
        stage1_vector_delta_assert_nearest(&latest_server, &reference, x, expected, context).await;
    }
}

#[cfg(unix)]
#[tokio::test]
async fn v2_flat_cpu_vector_sparse_delta_reuses_base_and_preserves_cold_layers() {
    stage1_vector_delta_contract("flat-cpu").await;
}

#[cfg(unix)]
#[tokio::test]
async fn v2_hnsw_cpu_vector_sparse_delta_reuses_base_and_preserves_cold_layers() {
    stage1_vector_delta_contract("hnsw-cpu").await;
}
