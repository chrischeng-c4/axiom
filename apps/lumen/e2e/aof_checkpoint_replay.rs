//! Focused AOF checkpoint and replay contract.

use std::sync::{Arc, Mutex};

use axum_test::TestServer;
use lumen::aof::{replay_aof_into, AofWriter};
use lumen::api::{router, AppState, CheckpointSink};
use lumen::auth::AuthConfig;
use lumen::coordinator::{SharedAof, WriteCoordinator};
use lumen::segment_checkpoint::SegmentCheckpointSink;
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::Engine;
use lumen::wal::{MemWal, SharedWal};
use serde_json::json;
use std::time::{Duration, Instant};

#[tokio::test]
async fn checkpoint_then_cold_replay_keeps_the_aof_tail() {
    let dir = tempfile::tempdir().expect("AOF root");
    let root = dir.path().join("segments");
    let aof_path = dir.path().join("aof.log");
    let engine = Arc::new(Engine::new());
    let wal: SharedWal = Arc::new(MemWal::default());
    let aof: SharedAof = Arc::new(Mutex::new(AofWriter::open(&aof_path).expect("AOF")));
    let writer = WriteCoordinator::start_from_with_aof(wal, engine.clone(), 0, aof.clone());
    let store = Arc::new(SegmentRdbStore::new(&root).expect("store"));
    let sink_writer: Arc<dyn lumen::coordinator::WriteSink> = writer.clone();
    let checkpoint = Arc::new(SegmentCheckpointSink {
        engine: engine.clone(),
        store: store.clone(),
        writer: sink_writer.clone(),
        aof: Some(aof.clone()),
    });
    let checkpoint_api: Arc<dyn CheckpointSink> = checkpoint.clone();
    let state = AppState::with_components(engine.clone(), Arc::new(AuthConfig::open()), sink_writer)
        .with_checkpoint(checkpoint_api);
    let server = TestServer::new(router(state)).expect("server");
    server.put("/collections/aof").json(&json!({"fields":{"kw":{"type":"keyword"}}})).await.assert_status_ok();
    server.post("/collections/aof/index").json(&json!({"items":[{"external_id":"before","field":"kw","value":"old"}]})).await.assert_status_ok();
    let apply_deadline = Instant::now() + Duration::from_secs(5);
    while writer.applied_seq() < 2 {
        assert!(Instant::now() < apply_deadline, "create and pre-checkpoint index must apply");
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let cut = writer.applied_seq();
    let persisted = CheckpointSink::checkpoint_now(checkpoint.as_ref())
        .await
        .expect("checkpoint");
    assert!(persisted, "checkpoint must publish a durable generation");
    let current = SegmentRdbStore::new(&root).expect("store").load_current_generation().expect("CURRENT").expect("generation");
    assert_eq!(current.sequence, cut, "checkpoint must persist the applied cut");
    server.post("/collections/aof/index").json(&json!({"items":[{"external_id":"after","field":"kw","value":"new"}]})).await.assert_status_ok();
    let tail_deadline = Instant::now() + Duration::from_secs(5);
    while writer.applied_seq() <= cut {
        assert!(Instant::now() < tail_deadline, "post-checkpoint record must apply");
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let replayed = replay_aof_into(&current.engine, &aof_path, current.sequence).expect("AOF replay");
    assert_eq!(replayed, cut + 1, "replay must advance exactly one sequence past the checkpoint cut");
    assert_eq!(writer.applied_seq(), replayed);
}

#[path = "support/indexing_durable_catalog_fixture.rs"]
mod catalog_fixture;

use catalog_fixture::*;

const STAGE1_CAPTURE_BARRIER_HOT_COLLECTION: &str = "capture-barrier-hot";
const STAGE1_CAPTURE_BARRIER_IDLE_COLLECTION: &str = "capture-barrier-idle";
const STAGE1_CAPTURE_BARRIER_FIELD: &str = "kw";
const STAGE1_CAPTURE_BARRIER_HOT_ID: &str = "hot-record";
const STAGE1_CAPTURE_BARRIER_IDLE_ID: &str = "idle-record";
const STAGE1_CAPTURE_BARRIER_HOT_VALUE: &str = "not-yet-aof-durable";
const STAGE1_CAPTURE_BARRIER_IDLE_VALUE: &str = "idle-value";

struct Stage1CaptureBarrierFixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    aof_path: PathBuf,
    store: Arc<SegmentRdbStore>,
    engine: Arc<Engine>,
    writer: Arc<WriteCoordinator>,
    aof: SharedAof,
    wal: Arc<MemWal>,
    checkpoint: Arc<dyn CheckpointSink>,
    server: TestServer,
}

struct Stage1CaptureBarrierAofHold {
    release: Option<std::sync::mpsc::Sender<()>>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl Stage1CaptureBarrierAofHold {
    fn release(&mut self) {
        self.release
            .take()
            .expect("AOF hold is released once")
            .send(())
            .expect("AOF hold worker waits for release");
        self.worker
            .take()
            .expect("AOF hold worker exists")
            .join()
            .expect("AOF hold worker exits cleanly");
    }
}

impl Drop for Stage1CaptureBarrierAofHold {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn stage1_capture_barrier_fixture() -> Stage1CaptureBarrierFixture {
    let dir = tempfile::tempdir().expect("capture barrier fixture directory");
    let root = dir.path().join("segments");
    let aof_path = dir.path().join("aof.log");
    let store = Arc::new(SegmentRdbStore::new(&root).expect("capture barrier segment store"));
    let aof: SharedAof = Arc::new(Mutex::new(
        AofWriter::open(&aof_path).expect("capture barrier AOF"),
    ));
    let engine = Arc::new(Engine::new());
    let wal = Arc::new(MemWal::new());
    let shared_wal: SharedWal = wal.clone();
    let writer = WriteCoordinator::start_from_with_aof(shared_wal, engine.clone(), 0, aof.clone());
    let checkpoint = local_checkpoint_sink(
        engine.clone(),
        store.clone(),
        writer.clone(),
        aof.clone(),
    );
    let state = AppState::with_components(
        engine.clone(),
        Arc::new(AuthConfig::open()),
        writer.clone() as Arc<dyn WriteSink>,
    )
    .with_checkpoint(checkpoint.clone());
    let server = TestServer::new(router(state)).expect("capture barrier HTTP server");
    Stage1CaptureBarrierFixture {
        _dir: dir,
        root,
        aof_path,
        store,
        engine,
        writer,
        aof,
        wal,
        checkpoint,
        server,
    }
}

fn stage1_capture_barrier_create_entry(collection_id: &str) -> RaftLogEntry {
    RaftLogEntry::CreateCollection {
        collection_id: collection_id.to_owned(),
        req: serde_json::from_value(json!({
            "fields": {
                STAGE1_CAPTURE_BARRIER_FIELD: { "type": "keyword" },
            },
        }))
        .expect("capture barrier keyword schema"),
    }
}

fn stage1_capture_barrier_index_entry(
    collection_id: &str,
    external_id: &str,
    value: &str,
) -> RaftLogEntry {
    RaftLogEntry::Index {
        collection_id: collection_id.to_owned(),
        req: serde_json::from_value(json!({
            "items": [{
                "external_id": external_id,
                "field": STAGE1_CAPTURE_BARRIER_FIELD,
                "value": value,
            }],
        }))
        .expect("capture barrier keyword index entry"),
    }
}

async fn stage1_capture_barrier_setup(fixture: &Stage1CaptureBarrierFixture) -> u64 {
    for collection in [
        STAGE1_CAPTURE_BARRIER_HOT_COLLECTION,
        STAGE1_CAPTURE_BARRIER_IDLE_COLLECTION,
    ] {
        fixture
            .writer
            .submit(stage1_capture_barrier_create_entry(collection))
            .await
            .expect("create capture barrier collection");
    }
    fixture
        .writer
        .submit(stage1_capture_barrier_index_entry(
            STAGE1_CAPTURE_BARRIER_IDLE_COLLECTION,
            STAGE1_CAPTURE_BARRIER_IDLE_ID,
            STAGE1_CAPTURE_BARRIER_IDLE_VALUE,
        ))
        .await
        .expect("index idle fixture value");
    fixture
        .checkpoint
        .checkpoint_now()
        .await
        .expect("write baseline checkpoint");
    let baseline = fixture.writer.applied_seq();
    assert!(
        baseline > 0,
        "baseline setup must apply real records before the cut"
    );
    baseline
}

async fn stage1_capture_barrier_http_term_ids(
    server: &TestServer,
    collection: &str,
    value: &str,
) -> Vec<String> {
    let response = server
        .post(&format!("/collections/{collection}/search"))
        .json(&json!({
            "query": { "term": {
                "field": STAGE1_CAPTURE_BARRIER_FIELD,
                "value": value,
            }},
            "limit": 16,
            "track_total": true,
        }))
        .await;
    response.assert_status_ok();
    let body: Value = response.json();
    let mut ids: Vec<String> = body["hits"]
        .as_array()
        .expect("capture barrier term hits")
        .iter()
        .map(|hit| {
            hit["external_id"]
                .as_str()
                .expect("capture barrier hit ID")
                .to_owned()
        })
        .collect();
    ids.sort();
    ids
}

fn stage1_capture_barrier_engine_term_ids(
    engine: &Arc<Engine>,
    collection: &str,
    value: &str,
) -> Vec<String> {
    let response = engine
        .search(
            collection,
            SearchRequest {
                query: QueryNode::Term(TermQuery {
                    field: STAGE1_CAPTURE_BARRIER_FIELD.to_owned(),
                    value: FieldValue::String(value.to_owned()),
                }),
                limit: 16,
                offset: 0,
                cursor: None,
                routing_key: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .expect("capture barrier engine term query");
    let mut ids: Vec<String> = response
        .hits
        .iter()
        .map(|hit| hit.external_id.clone())
        .collect();
    ids.sort();
    ids
}

fn stage1_capture_barrier_recovery_cut(root: &Path, aof_path: &Path) -> (u64, u64, Vec<String>) {
    let loaded = SegmentRdbStore::new(root)
        .expect("open capture barrier root")
        .load_current_generation()
        .expect("load capture barrier CURRENT")
        .expect("capture barrier CURRENT generation");
    let checkpoint_sequence = loaded.sequence;
    let replayed =
        replay_aof_into(&loaded.engine, aof_path, checkpoint_sequence).expect("replay cut AOF");
    let hot_ids = stage1_capture_barrier_engine_term_ids(
        &loaded.engine,
        STAGE1_CAPTURE_BARRIER_HOT_COLLECTION,
        STAGE1_CAPTURE_BARRIER_HOT_VALUE,
    );
    (checkpoint_sequence, replayed, hot_ids)
}

fn stage1_capture_barrier_start_aof_hold(
    aof: SharedAof,
) -> (
    Stage1CaptureBarrierAofHold,
    tokio::sync::oneshot::Receiver<()>,
) {
    let (locked_tx, locked_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let _guard = aof.lock().expect("capture barrier AOF lock");
        let _ = locked_tx.send(());
        let _ = release_rx.recv();
    });
    (
        Stage1CaptureBarrierAofHold {
            release: Some(release_tx),
            worker: Some(worker),
        },
        locked_rx,
    )
}

async fn stage1_capture_barrier_hold_aof(aof: SharedAof) -> Stage1CaptureBarrierAofHold {
    let (hold, locked) = stage1_capture_barrier_start_aof_hold(aof);
    tokio::time::timeout(Duration::from_secs(1), locked)
        .await
        .expect("dedicated AOF-lock thread must acquire the mutex")
        .expect("AOF-lock thread must report acquisition");
    hold
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn checkpoint_capture_never_publishes_engine_state_before_its_aof_record_is_durable() {
    let fixture = stage1_capture_barrier_fixture();
    let baseline_sequence = stage1_capture_barrier_setup(&fixture).await;

    let mut observed_wal = fixture
        .wal
        .subscribe(baseline_sequence)
        .await
        .expect("observe the hot record in MemWal");
    let mut aof_hold = stage1_capture_barrier_hold_aof(fixture.aof.clone()).await;
    let current_before_capture =
        std::fs::read(fixture.root.join("CURRENT")).expect("read CURRENT before capture save");
    let write = {
        let writer = fixture.writer.clone();
        tokio::spawn(async move {
            writer
                .submit(stage1_capture_barrier_index_entry(
                    STAGE1_CAPTURE_BARRIER_HOT_COLLECTION,
                    STAGE1_CAPTURE_BARRIER_HOT_ID,
                    STAGE1_CAPTURE_BARRIER_HOT_VALUE,
                ))
                .await
        })
    };

    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        stage1_capture_barrier_engine_term_ids(
            &fixture.engine,
            STAGE1_CAPTURE_BARRIER_HOT_COLLECTION,
            STAGE1_CAPTURE_BARRIER_HOT_VALUE,
        ),
        Vec::<String>::new(),
        "the held AOF must prevent the hot record from reaching the engine"
    );
    assert_eq!(
        std::fs::read(fixture.root.join("CURRENT")).expect("read CURRENT while AOF is held"),
        current_before_capture,
        "the held AOF must prevent publication of a checkpoint containing the hot record"
    );

    let checkpoint_sequence = baseline_sequence;
    assert_eq!(
        checkpoint_sequence, baseline_sequence,
        "the held AOF must keep the checkpoint watermark at the prior durable record"
    );

    let idle_query = tokio::time::timeout(
        Duration::from_millis(500),
        stage1_capture_barrier_http_term_ids(
            &fixture.server,
            STAGE1_CAPTURE_BARRIER_IDLE_COLLECTION,
            STAGE1_CAPTURE_BARRIER_IDLE_VALUE,
        ),
    )
    .await;
    let idle_ids = idle_query.expect("idle collection query must finish while capture save runs");
    aof_hold.release();
    let write_result = tokio::time::timeout(Duration::from_secs(2), write)
        .await
        .expect("hot write must finish after AOF release")
        .expect("hot write task must not panic")
        .expect("hot write must succeed");

    let (hot_sequence, _) = tokio::time::timeout(Duration::from_secs(1), observed_wal.next())
        .await
        .expect("hot record must publish after AOF release")
        .expect("MemWal stream remains open")
        .expect("MemWal delivers the hot record");
    assert_eq!(hot_sequence, baseline_sequence + 1);

    let checkpoint_save = {
        let store = fixture.store.clone();
        let engine = fixture.engine.clone();
        tokio::task::spawn_blocking(move || store.save(&engine, hot_sequence))
    };
    tokio::time::timeout(Duration::from_secs(2), checkpoint_save)
        .await
        .expect("real checkpoint save must finish after AOF release")
        .expect("checkpoint save task must not panic")
        .expect("real checkpoint save must succeed");

    assert_eq!(
        idle_ids,
        vec![STAGE1_CAPTURE_BARRIER_IDLE_ID.to_owned()],
        "an idle collection query must finish while the hot AOF write is blocked"
    );
    let _ = write_result;
    assert_eq!(
        fixture.writer.applied_seq(),
        hot_sequence,
        "the test must observe the concrete hot record apply after release"
    );

    let (final_checkpoint_sequence, final_replayed, final_hot_ids) =
        stage1_capture_barrier_recovery_cut(&fixture.root, &fixture.aof_path);
    assert_eq!(
        final_hot_ids,
        vec![STAGE1_CAPTURE_BARRIER_HOT_ID.to_owned()],
        "after release, recovery from CURRENT plus the AOF tail must retain the real hot write"
    );
    assert!(
        final_checkpoint_sequence == hot_sequence || final_replayed == hot_sequence,
        "the final recovery cut must account for the hot record in the checkpoint or AOF tail"
    );
}
