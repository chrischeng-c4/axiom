//! Black-box oracle for issue #3951.
//!
//! `Engine::flush_to_segments` seals every field of a checkpointed collection,
//! but the default `vector` backend (HNSW-CPU) has no persistence path: its
//! `VectorIndex::seal_to_segment_prod` is the trait's default no-op, so it
//! writes neither a `<field>.lseg` segment nor the `<field>.eids.lseg`
//! row->eid sidecar. `Collection::open_from_segments` (the reopen side)
//! requires that sidecar UNCONDITIONALLY for every field whose schema says
//! `FieldType::Vector`, regardless of which backend sealed it. So a checkpoint
//! that includes a default-backend vector field "succeeds" (no error is
//! surfaced at save time) but is silently incomplete, and the very next
//! recovery — which the checkpoint's own AOF truncation makes mandatory,
//! since the pre-checkpoint AOF tail is gone — fails to reopen the collection
//! at all. All data, including fields that have nothing to do with vectors,
//! goes missing.
//!
//! In practice the fault surfaces even earlier than a later failed reopen:
//! `SegmentRdbStore::save_inner` does not trust what it just staged. Before
//! activating a generation it calls `validate_record`, which performs a full
//! verification reopen of the staged directory (`reopen_once` ->
//! `Engine::reopen_from_segment_dir` -> `Collection::open_from_segments`) --
//! the exact code path a real restart runs -- and deletes the staged
//! directory on any error. So `POST /admin/checkpoint` itself is refused
//! (400, "validate staged segment generation") for any collection with a
//! default-backend vector field, every time, and no generation is ever
//! committed for it at all -- not merely one that silently loses data on
//! the next reopen.
//!
//! This oracle proves the fault by driving the exact checkpoint through the
//! real HTTP surface and showing it is refused outright, which is the
//! earliest externally observable consequence of the same missing-sidecar
//! defect the rest of this file's (dead, until fixed) assertions describe:
//! that a checkpoint which "succeeded" would still be missing the vector
//! field's on-disk file and would still fail the next reopen.

use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;
use axum_test::TestServer;
use serde_json::json;

use lumen::aof::{replay_aof_into, AofWriter};
use lumen::api::{router, AppState, CheckpointSink};
use lumen::auth::AuthConfig;
use lumen::coordinator::{SharedAof, WriteCoordinator, WriteSink};
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::Engine;
use lumen::types::{FieldValue, KnnQuery, QueryNode, SearchRequest, TermQuery};
use lumen::wal::{MemWal, SharedWal};

struct LocalCheckpointSink {
    engine: Arc<Engine>,
    store: Arc<SegmentRdbStore>,
    writer: Arc<WriteCoordinator>,
    aof: SharedAof,
}

#[async_trait]
impl CheckpointSink for LocalCheckpointSink {
    async fn checkpoint_now(&self) -> Result<bool> {
        let _permit = self.writer.mutation_gate().shared().await?;
        let sequence = self.writer.applied_seq();
        self.store.save(&self.engine, sequence)?;
        self.store.prune(3)?;
        self.aof
            .lock()
            .map_err(|_| anyhow::anyhow!("aof writer poisoned"))?
            .truncate_through(sequence)?;
        Ok(true)
    }
}

struct Fixture {
    _dir: tempfile::TempDir,
    server: TestServer,
    aof_path: std::path::PathBuf,
    store: Arc<SegmentRdbStore>,
    checkpoint_root: std::path::PathBuf,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().expect("fixture directory");
    let checkpoint_root = dir.path().join("segments");
    let aof_path = dir.path().join("aof.log");
    let store = Arc::new(SegmentRdbStore::new(&checkpoint_root).expect("segment store"));
    let aof: SharedAof = Arc::new(Mutex::new(AofWriter::open(&aof_path).expect("aof")));
    let engine = Arc::new(Engine::new());
    let wal: SharedWal = Arc::new(MemWal::new());
    let writer = WriteCoordinator::start_from_with_aof(wal, engine.clone(), 0, aof.clone());
    let checkpoint = Arc::new(LocalCheckpointSink {
        engine: engine.clone(),
        store: store.clone(),
        writer: writer.clone(),
        aof: aof.clone(),
    });
    let state = AppState::with_components(
        engine,
        Arc::new(AuthConfig::open()),
        writer as Arc<dyn WriteSink>,
    )
    .with_checkpoint(checkpoint);
    let server = TestServer::new(router(state)).expect("test server");
    Fixture {
        _dir: dir,
        server,
        aof_path,
        store,
        checkpoint_root,
    }
}

fn catalogued_vector_base_path(
    manifest: &serde_json::Value,
    collection_id: &str,
    role: &str,
) -> String {
    let collection = manifest["collections"]
        .as_array()
        .expect("v2 manifest collections array")
        .iter()
        .find(|collection| collection["collection_id"] == collection_id)
        .expect("v2 manifest must catalogue vecdocs");
    let matching: Vec<&serde_json::Value> = collection["segments"]
        .as_array()
        .expect("v2 manifest collection segments array")
        .iter()
        .filter(|segment| {
            segment["role"] == role && segment["field"] == "v" && segment["kind"] == "base"
        })
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "v2 manifest must catalogue exactly one base {role} segment for vector field `v`; got {matching:?}"
    );
    matching[0]["path"]
        .as_str()
        .expect("v2 vector segment path string")
        .to_owned()
}

fn term_query(field: &str, value: &str) -> SearchRequest {
    SearchRequest {
        query: QueryNode::Term(TermQuery {
            field: field.into(),
            value: FieldValue::String(value.into()),
        }),
        limit: 20,
        offset: 0,
        cursor: None,
        routing_key: None,
        sort: None,
        track_total: true,
        collapse: None,
    }
}

async fn checkpoint(server: &TestServer) {
    let response = server.post("/admin/checkpoint").await;
    response.assert_status_ok();
    assert_eq!(response.json::<serde_json::Value>()["persisted"], true);
}

#[tokio::test]
async fn checkpoint_persists_vector_field_and_survives_recovery() {
    let fixture = fixture();

    // Collection 1: a keyword field plus a `vector` field using the DEFAULT
    // backend (no explicit "backend" key => HNSW-CPU, see
    // `VectorBackend::default()`), which is exactly the schema shape any
    // caller gets by following the README's kNN example verbatim.
    fixture
        .server
        .put("/collections/vecdocs")
        .json(&json!({
            "fields": {
                "kw": { "type": "keyword" },
                "v": { "type": "vector", "dim": 3, "metric": "cosine" }
            }
        }))
        .await
        .assert_status_ok();

    // Collection 2: keyword-only, so the oracle can show the fault destroys
    // an UNRELATED collection's data too (any vector field anywhere in the
    // checkpoint poisons the whole recovery).
    fixture
        .server
        .put("/collections/plaindocs")
        .json(&json!({ "fields": { "kw": { "type": "keyword" } } }))
        .await
        .assert_status_ok();

    fixture
        .server
        .post("/collections/vecdocs/index")
        .json(&json!({ "items": [
            { "external_id": "d1", "field": "kw", "value": "hello" },
            { "external_id": "d1", "field": "v", "value": [0.1, 0.2, 0.3] }
        ]}))
        .await
        .assert_status_ok();

    fixture
        .server
        .post("/collections/plaindocs/index")
        .json(&json!({ "items": [
            { "external_id": "p1", "field": "kw", "value": "world" }
        ]}))
        .await
        .assert_status_ok();

    // Checkpoint: seals every field to segments, then truncates the AOF
    // through the checkpointed sequence — exactly the sequence a production
    // periodic checkpoint runs, and the same one that makes the checkpoint
    // the ONLY durable copy of everything indexed above.
    checkpoint(&fixture.server).await;

    // #3951, part 1: the v2 catalog is the physical contract. Field names
    // are encoded in the generation layout, so a filename prefix such as
    // `v.` is neither required nor reliable. The catalog must instead name
    // both the vector's base segment and its row->external-ID sidecar, and
    // both paths must name real files in the committed generation.
    let current = std::fs::read_to_string(fixture.checkpoint_root.join("CURRENT"))
        .expect("read committed generation pointer");
    let generation_name = current
        .strip_prefix("generation:")
        .expect("CURRENT names a generation")
        .trim();
    let generation = fixture.checkpoint_root.join(generation_name);
    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(generation.join("_generation.json"))
            .expect("read committed v2 generation manifest"),
    )
    .expect("decode committed v2 generation manifest");
    for (role, path) in [
        (
            "field",
            catalogued_vector_base_path(&manifest, "vecdocs", "field"),
        ),
        (
            "vector_eids",
            catalogued_vector_base_path(&manifest, "vecdocs", "vector_eids"),
        ),
    ] {
        assert!(
            generation.join(&path).is_file(),
            "v2 catalogued {role} base for vector field `v` must exist at {path:?}"
        );
    }

    // #3951, part 2: recovery — reopen the checkpoint's active generation and
    // replay the (now-truncated, effectively empty) AOF tail, exactly as a
    // cold restart does. This is the step that actually loses the data: it
    // must succeed, since the checkpoint above is the only durable record.
    let recovered = match fixture.store.load_current_generation() {
        Ok(loaded) => loaded.expect("a checkpoint generation must exist after /admin/checkpoint"),
        Err(err) => panic!(
            "recovery from a checkpoint that included a default-backend vector \
             field failed to reopen (issue #3951): {err:#}"
        ),
    };
    let replayed = replay_aof_into(&recovered.engine, &fixture.aof_path, recovered.sequence)
        .expect("replay AOF tail after checkpoint truncation");
    assert_eq!(
        replayed, 0,
        "the AOF was truncated through the checkpoint sequence; nothing should replay"
    );

    let snapshot = recovered.engine.snapshot().expect("engine snapshot");
    assert!(
        snapshot.collections.contains_key("vecdocs"),
        "vecdocs collection missing after recovery"
    );
    assert!(
        snapshot.collections.contains_key("plaindocs"),
        "plaindocs collection missing after recovery"
    );
    assert!(
        snapshot.collections["vecdocs"].schema.contains_key("v"),
        "recovered vecdocs schema must still declare field `v`"
    );

    let vecdocs_kw = recovered
        .engine
        .search("vecdocs", term_query("kw", "hello"))
        .expect("kw term query on recovered vecdocs");
    assert_eq!(
        vecdocs_kw.total, 1,
        "keyword doc in vecdocs must survive recovery"
    );

    let plaindocs_kw = recovered
        .engine
        .search("plaindocs", term_query("kw", "world"))
        .expect("kw term query on recovered plaindocs");
    assert_eq!(
        plaindocs_kw.total, 1,
        "keyword doc in the unrelated plaindocs collection must survive recovery"
    );

    let knn = recovered
        .engine
        .search(
            "vecdocs",
            SearchRequest {
                query: QueryNode::Knn(KnnQuery {
                    field: "v".into(),
                    vector: vec![0.1, 0.2, 0.3],
                    k: 1,
                }),
                limit: 1,
                offset: 0,
                cursor: None,
                routing_key: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .expect("knn query on recovered vector field");
    assert_eq!(
        knn.hits.first().map(|hit| hit.external_id.as_str()),
        Some("d1"),
        "vector doc must be retrievable via kNN after recovery"
    );
}
