//! Black-box oracle for the reopen half of #3951's vector seal.
//!
//! #3951 gave `HnswCpuIndex` a `seal_to_segment_prod` so a default-backend
//! vector field finally writes its `<field>.lseg` + `<field>.eids.lseg` pair
//! and a checkpoint stops being refused. The reopen side was left alone:
//! `FieldIndex::open_from_segment` builds a `FlatCpuIndex` for EVERY
//! `FieldType::Vector` field, whatever `spec.backend` the schema declares, and
//! hands it `bytes: 0`.
//!
//! So a field declared `hnsw-cpu` is HNSW until the first restart and an exact
//! flat scan forever after, and nothing in the product says so. The schema
//! still reports `backend: hnsw-cpu`. Two nodes of one cluster — one restarted,
//! one not — answer the same kNN with different algorithms, different recall,
//! and different latency, and the operator has no signal that they differ. A
//! collection sized against HNSW's sub-linear search silently becomes an O(N)
//! scan per query at the exact moment a restart happens, which is the moment
//! nobody is watching latency.
//!
//! The observable is `GET /collections/{id}/stats`, whose per-field `bytes` is
//! the engine's own resident-footprint figure (`FieldIndex::bytes`). It
//! separates the two backends by design and the seal already relies on that:
//!
//! | backend | after seal | why |
//! |---|---|---|
//! | `flat-cpu` | 0 | the scan buffer is dropped; the mmap is the only copy |
//! | `hnsw-cpu` | resident | the graph and its raw vectors stay in RAM |
//!
//! A reopen that honours the declared backend therefore has to reproduce that
//! split. Today it reports 0 for both, which is the truth about what it built
//! and a lie about what the schema promises.
//!
//! The flat field is the control, and it is load-bearing twice over: it shows
//! `bytes` is not simply "whatever was there before the checkpoint" (its own
//! figure legitimately drops to 0 and must stay 0), and it shows the assertion
//! is reading a real per-backend distinction rather than a constant.
//!
//! The third case is regression coverage rather than a defect oracle. The
//! sidecar oracle (`segment_checkpoint_vector_sidecar.rs`) checkpoints exactly
//! once, over a collection with no deletes, and never checkpoints the engine it
//! recovered — so neither tombstone-aware sealing nor re-sealing an already
//! reopened vector field had a black-box case at all.

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
use lumen::types::{KnnQuery, QueryNode, SearchRequest};
use lumen::wal::{MemWal, SharedWal};

/// Dimensionality of both vector fields. Small enough to write out by hand,
/// wide enough that one-hot rows are unambiguously nearest to themselves.
const DIM: usize = 8;

/// The HNSW-declared field.
const HNSW_FIELD: &str = "graph";
/// The flat-declared field, the control.
const FLAT_FIELD: &str = "scan";

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
    }
}

/// A one-hot row: doc `i` is the unit basis vector `e_i`. Under cosine, `e_i`
/// is exactly nearest to itself and equidistant from every other row, so the
/// expected top-1 is decided by the data rather than by tie-breaking.
fn one_hot(i: usize) -> Vec<f32> {
    let mut v = vec![0.0f32; DIM];
    v[i] = 1.0;
    v
}

fn knn(field: &str, i: usize, k: u32) -> SearchRequest {
    SearchRequest {
        query: QueryNode::Knn(KnnQuery {
            field: field.into(),
            vector: one_hot(i),
            k,
        }),
        limit: k,
        offset: 0,
        cursor: None,
        routing_key: None,
        sort: None,
        track_total: true,
        collapse: None,
    }
}

/// Create the two-vector-field collection and index `DIM` one-hot docs into
/// both fields, so the two backends hold byte-identical corpora and any
/// difference the assertions see is a difference in backend, not in data.
async fn seed(server: &TestServer) {
    server
        .put("/collections/vecdocs")
        .json(&json!({
            "fields": {
                HNSW_FIELD: { "type": "vector", "dim": DIM, "metric": "cosine",
                              "backend": "hnsw-cpu" },
                FLAT_FIELD: { "type": "vector", "dim": DIM, "metric": "cosine",
                              "backend": "flat-cpu" }
            }
        }))
        .await
        .assert_status_ok();

    let items: Vec<serde_json::Value> = (0..DIM)
        .flat_map(|i| {
            let id = format!("d{i}");
            [
                json!({ "external_id": id, "field": HNSW_FIELD, "value": one_hot(i) }),
                json!({ "external_id": id, "field": FLAT_FIELD, "value": one_hot(i) }),
            ]
        })
        .collect();
    server
        .post("/collections/vecdocs/index")
        .json(&json!({ "items": items }))
        .await
        .assert_status_ok();
}

/// Read one field's reported resident footprint off the live HTTP stats route.
async fn field_bytes(server: &TestServer, field: &str) -> u64 {
    let response = server.get("/collections/vecdocs/stats").await;
    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    body["fields"][field]["bytes"]
        .as_u64()
        .unwrap_or_else(|| panic!("stats must report `bytes` for field `{field}`: {body}"))
}

async fn checkpoint(server: &TestServer) {
    let response = server.post("/admin/checkpoint").await;
    response.assert_status_ok();
    assert_eq!(response.json::<serde_json::Value>()["persisted"], true);
}

/// Reopen the active generation and drain the (truncated) AOF tail, exactly as
/// a cold restart does.
fn recover(fixture: &Fixture) -> lumen::segment_rdb::LoadedSegmentGeneration {
    let recovered = fixture
        .store
        .load_current_generation()
        .expect("reopen the active checkpoint generation")
        .expect("a generation must exist after /admin/checkpoint");
    replay_aof_into(&recovered.engine, &fixture.aof_path, recovered.sequence)
        .expect("replay the AOF tail after checkpoint truncation");
    recovered
}

/// THE CASE. A field the schema declares `hnsw-cpu` must still be HNSW-backed
/// after a reopen. The engine's own per-field footprint is what says so: an
/// HNSW field keeps its graph and raw vectors resident, so its `bytes` must
/// come back the way it went in, not as the 0 a flat segment-backed field
/// legitimately reports.
#[tokio::test]
async fn an_hnsw_declared_field_is_still_hnsw_backed_after_a_reopen() {
    let fixture = fixture();
    seed(&fixture.server).await;

    let live_hnsw_bytes = field_bytes(&fixture.server, HNSW_FIELD).await;
    assert!(
        live_hnsw_bytes > 0,
        "a live HNSW field holds its graph in RAM and must report a footprint"
    );

    checkpoint(&fixture.server).await;
    let recovered = recover(&fixture);

    let stats = recovered
        .engine
        .stats("vecdocs")
        .expect("stats on the recovered collection");
    let reopened_hnsw_bytes = stats.fields[HNSW_FIELD].bytes;

    assert_eq!(
        reopened_hnsw_bytes, live_hnsw_bytes,
        "field `{HNSW_FIELD}` is declared `hnsw-cpu`, so the reopen must rebuild an \
         HNSW graph from the segment and report the same resident footprint the live \
         field reported before the checkpoint; reporting {reopened_hnsw_bytes} instead \
         of {live_hnsw_bytes} means the reopen built a flat scan and the declared \
         backend is now a lie"
    );

    // A rebuilt graph must also still answer. Without this, a fix that made the
    // number right by inventing it would pass the assertion above.
    for i in 0..DIM {
        let hits = recovered
            .engine
            .search("vecdocs", knn(HNSW_FIELD, i, 1))
            .expect("knn on the reopened hnsw field");
        assert_eq!(
            hits.hits.first().map(|h| h.external_id.as_str()),
            Some(format!("d{i}").as_str()),
            "reopened `{HNSW_FIELD}` must return d{i} as the nearest neighbour of e_{i}"
        );
    }
}

/// THE CONTROL. The same reopen, on a field declared `flat-cpu`, must report a
/// footprint of 0 — its vectors genuinely left RAM for the mmap. This is what
/// makes the case above an assertion about the backend rather than about the
/// number happening to survive a round trip.
#[tokio::test]
async fn a_flat_declared_field_reports_no_resident_footprint_after_a_reopen() {
    let fixture = fixture();
    seed(&fixture.server).await;

    let live_flat_bytes = field_bytes(&fixture.server, FLAT_FIELD).await;
    assert!(
        live_flat_bytes > 0,
        "before any seal, a flat field's scan buffer is in RAM and must be reported"
    );

    checkpoint(&fixture.server).await;

    assert_eq!(
        field_bytes(&fixture.server, FLAT_FIELD).await,
        0,
        "sealing a flat field drops its scan buffer, so the LIVE engine must \
         already report 0 for it"
    );

    let recovered = recover(&fixture);
    let stats = recovered
        .engine
        .stats("vecdocs")
        .expect("stats on the recovered collection");
    assert_eq!(
        stats.fields[FLAT_FIELD].bytes, 0,
        "a reopened `flat-cpu` field reads its vectors off the mmap and must report \
         no resident footprint"
    );

    for i in 0..DIM {
        let hits = recovered
            .engine
            .search("vecdocs", knn(FLAT_FIELD, i, 1))
            .expect("knn on the reopened flat field");
        assert_eq!(
            hits.hits.first().map(|h| h.external_id.as_str()),
            Some(format!("d{i}").as_str()),
            "reopened `{FLAT_FIELD}` must return d{i} as the nearest neighbour of e_{i}"
        );
    }
}

/// REGRESSION COVERAGE for the two gaps the sidecar oracle left: a document
/// deleted before a checkpoint must not come back through the seal, and an
/// engine recovered from a checkpoint must be checkpointable again.
///
/// Both halves cover every vector backend at once, because the seal and reopen
/// paths differ per backend and a tombstone leak or a failed re-seal on either
/// one is a lost delete or a node that can never checkpoint after a restart.
#[tokio::test]
async fn a_delete_survives_the_seal_and_the_reopened_engine_checkpoints_again() {
    let fixture = fixture();
    seed(&fixture.server).await;

    fixture
        .server
        .delete("/collections/vecdocs/index/d3")
        .await
        .assert_status_success();

    checkpoint(&fixture.server).await;
    let recovered = recover(&fixture);

    for field in [HNSW_FIELD, FLAT_FIELD] {
        let hits = recovered
            .engine
            .search("vecdocs", knn(field, 3, DIM as u32))
            .expect("knn over the whole reopened corpus");
        assert!(
            hits.hits.iter().all(|h| h.external_id != "d3"),
            "d3 was deleted before the checkpoint; the seal must not resurrect it \
             on field `{field}`, but kNN returned {:?}",
            hits.hits.iter().map(|h| &h.external_id).collect::<Vec<_>>()
        );
        assert_eq!(
            hits.hits.len(),
            DIM - 1,
            "field `{field}` must hold the {} surviving docs after the reopen",
            DIM - 1
        );
    }

    // Second checkpoint, over the engine that was itself reopened from the
    // first one. This is the step a restarted node runs on its next periodic
    // checkpoint, and no case covered it: a vector field whose base now lives
    // on a segment has to be re-sealable.
    fixture
        .store
        .save(&recovered.engine, recovered.sequence)
        .expect("a recovered engine must be checkpointable again");

    let twice = fixture
        .store
        .load_current_generation()
        .expect("reopen the second generation")
        .expect("the second checkpoint must have committed a generation");
    for field in [HNSW_FIELD, FLAT_FIELD] {
        let hits = twice
            .engine
            .search("vecdocs", knn(field, 5, 1))
            .expect("knn after a second checkpoint round trip");
        assert_eq!(
            hits.hits.first().map(|h| h.external_id.as_str()),
            Some("d5"),
            "field `{field}` must still answer kNN after a second seal/reopen round"
        );
    }
}
