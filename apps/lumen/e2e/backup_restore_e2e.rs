// CODEGEN-BEGIN
//! Backup → restore round-trip.
//!
//! ## Contracts inherited from the retired EC shells
//!
//! These 3 sentences were the whole of the `// Contract:` comment in 3 AW-EC shells
//! under `apps/lumen/e2e/`, each of which ran `cargo test -p lumen --test
//! backup_restore_e2e` in a subprocess and asserted the child's exit status. `cargo
//! test -p lumen` already runs this target directly, so the shells added a second,
//! nested run and nothing else. They were deleted on 2026-08-20 with the EC machinery
//! they belonged to, and the sentence is the only thing they held that nothing else
//! did. Each line below is prefixed with the EC id its shell was filed under.
//!
//! - `lumen-claim-backup-periodic-snapshotter` — The serving process snapshot loop and
//!   restore path remain covered by the backup/restore e2e gate; live replica
//!   synchronization remains raft-owned.
//! - `lumen-claim-backup-rdb-store` — RDB snapshots restore through the LocalFsRdbStore
//!   baseline as a cold restore and future bootstrap seed surface.
//! - `lumen-topology-existing-backup-seed` — The backup/restore e2e gate proves cold
//!   snapshot restore; the empty-PVC bootstrap seed path now restores SnapshotV1 before
//!   WAL/raft catch-up.

use std::sync::Arc;

use axum_test::TestServer;
use serde_json::{json, Value};

fn server() -> TestServer {
    let engine = Arc::new(lumen::storage::Engine::new());
    let app = lumen::api::router(lumen::api::AppState::open(engine));
    TestServer::new(app).expect("test server")
}

#[cfg(feature = "backup")]
fn http_server() -> TestServer {
    let engine = Arc::new(lumen::storage::Engine::new());
    let app = lumen::api::router(lumen::api::AppState::open(engine));
    TestServer::builder()
        .http_transport()
        .build(app)
        .expect("http test server")
}

#[tokio::test]
async fn snapshot_then_restore_into_fresh_engine() {
    let src = server();
    src.put("/collections/u")
        .json(&json!({
            "fields": {
                "bio":   { "type": "text" },
                "email": { "type": "keyword" },
                "tags":  { "type": "set" },
                "age":   { "type": "number" }
            }
        }))
        .await
        .assert_status_ok();

    src.post("/collections/u/index")
        .json(&json!({
            "items": [
                { "external_id": "u1", "field": "bio",   "value": "rust engineer in taipei" },
                { "external_id": "u1", "field": "email", "value": "a@x.com" },
                { "external_id": "u1", "field": "tags",  "value": ["rust","db"] },
                { "external_id": "u1", "field": "age",   "value": 30 },
                { "external_id": "u2", "field": "email", "value": "a@x.com" },
                { "external_id": "u2", "field": "age",   "value": 25 }
            ]
        }))
        .await
        .assert_status_ok();

    let dump = src.get("/admin/backup").await;
    dump.assert_status_ok();
    let snap: Value = dump.json();
    // The snapshot format this build WRITES. It became 2 when the document
    // stopped shipping the `terms`/`elements` maps that the reader rebuilds
    // from `forward`: a 1-era reader must REFUSE such a document rather than
    // restore a `set` field empty from a map that is no longer there, so the
    // number is load-bearing and pinned here on purpose. Readers still accept
    // `1..=SNAPSHOT_VERSION`; `restore_rejects_wrong_version` covers that side.
    assert_eq!(snap["version"], 2);
    assert!(snap["collections"]["u"].is_object());

    // Boot a fresh engine and restore.
    let dst = server();
    dst.post("/admin/restore")
        .json(&snap)
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);

    // Queries against the restored engine return the same results.
    let r = dst
        .post("/collections/u/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "a@x.com" } },
            "limit": 10
        }))
        .await;
    let body: Value = r.json();
    assert_eq!(body["total"], 2);

    let r = dst
        .post("/collections/u/duplicates")
        .json(&json!({ "field": "email" }))
        .await;
    let body: Value = r.json();
    assert_eq!(body["groups"].as_array().unwrap().len(), 1);

    let r = dst
        .post("/collections/u/search")
        .json(&json!({
            "query": { "range": { "field": "age", "gte": 26 } },
            "limit": 10
        }))
        .await;
    let body: Value = r.json();
    assert_eq!(body["total"], 1);
    assert_eq!(body["hits"][0]["external_id"], "u1");

    let r = dst
        .post("/collections/u/search")
        .json(&json!({
            "query": { "match": { "field": "bio", "text": "rust" } },
            "limit": 10
        }))
        .await;
    let body: Value = r.json();
    assert_eq!(body["total"], 1);
    assert_eq!(body["hits"][0]["external_id"], "u1");

    let r = dst
        .post("/collections/u/search")
        .json(&json!({
            "query": { "term": { "field": "tags", "value": "rust" } },
            "limit": 10
        }))
        .await;
    let body: Value = r.json();
    assert_eq!(body["total"], 1);
}

#[tokio::test]
async fn restore_rejects_wrong_version() {
    let s = server();
    let resp = s
        .post("/admin/restore")
        .json(&json!({ "version": 999, "collections": {} }))
        .await;
    resp.assert_status_bad_request();
}

/// #1095: the CLI helper path can export SnapshotV1 bytes over HTTP and import
/// them into a fresh server.
#[cfg(feature = "backup")]
#[tokio::test]
async fn http_snapshot_helpers_export_then_import() {
    let src = http_server();
    src.put("/collections/u")
        .json(&json!({ "fields": { "email": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    src.post("/collections/u/index")
        .json(&json!({
            "items": [
                { "external_id": "u1", "field": "email", "value": "a@x.com" },
                { "external_id": "u2", "field": "email", "value": "b@x.com" }
            ]
        }))
        .await
        .assert_status_ok();

    let base = src.server_url("/").expect("server url").to_string();
    let payload = lumen::backup::fetch_snapshot_bytes(&base, None)
        .await
        .expect("export snapshot bytes");
    let snap: Value = serde_json::from_slice(&payload).expect("snapshot json");
    // The snapshot format this build WRITES. It became 2 when the document
    // stopped shipping the `terms`/`elements` maps that the reader rebuilds
    // from `forward`: a 1-era reader must REFUSE such a document rather than
    // restore a `set` field empty from a map that is no longer there, so the
    // number is load-bearing and pinned here on purpose. Readers still accept
    // `1..=SNAPSHOT_VERSION`; `restore_rejects_wrong_version` covers that side.
    assert_eq!(snap["version"], 2);

    let file = tempfile::NamedTempFile::new().expect("snapshot file");
    std::fs::write(file.path(), &payload).expect("write snapshot");
    let imported = std::fs::read(file.path()).expect("read snapshot");

    let dst = http_server();
    let dst_base = dst.server_url("/").expect("server url").to_string();
    lumen::backup::restore_snapshot_bytes(&dst_base, None, &imported)
        .await
        .expect("import snapshot bytes");

    let r = dst
        .post("/collections/u/search")
        .json(&json!({
            "query": { "term": { "field": "email", "value": "a@x.com" } },
            "limit": 10
        }))
        .await;
    let body: Value = r.json();
    assert_eq!(body["total"], 1);
    assert_eq!(body["hits"][0]["external_id"], "u1");
}
// CODEGEN-END
mod restore_publication_race_contract {
    //! # Facets
    //!
    //! - Behavior: `backup_restore_e2e.rs:426`, `:437`, `:444`, `:539`, `:548`,
    //!   and `:566` require old checkpoints to wait or refuse and cold/live
    //!   recovery plus a later write to use the restored data.
    //! - Security: `CURRENT` is persisted input read at restart. `:391`, `:403`,
    //!   `:426`, `:500`, `:504`, `:539`, and `:548` reject an old publication or
    //!   caller cancellation that would replace the selected restore generation.
    //! - Performance: Gap — work-item change point `segment_restore.rs` is at
    //!   `.aw/workitems/deliveries/lumen061-05-incremental-failure-recovery-backup.md:23`;
    //!   its save/reload/activation path is `segment_restore.rs:184`, `:229`, `:288`.
    //!   `README.md:359-372` has recovery behavior and a gate, but no current numeric budget;
    //!   the 250-ms/2-s test limits are cleanup bounds only, so lumen-pm needs that budget.

    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use async_trait::async_trait;
    use axum_test::TestServer;
    use serde_json::Value;
    use tokio::sync::{oneshot, Notify};

    use lumen::aof::AofWriter;
    use lumen::api::{router, AppState, RestoreSink};
    use lumen::auth::AuthConfig;
    use lumen::coordinator::{SharedAof, WriteCoordinator, WriteSink};
    use lumen::log_entry::RaftLogEntry;
    use lumen::segment_rdb::SegmentRdbStore;
    use lumen::segment_restore::{RestorePublicationObserver, SegmentRestoreSink};
    use lumen::storage::{Engine, SnapshotV1};
    use lumen::types::CreateCollectionRequest;
    use lumen::wal::{MemWal, SharedWal};

    const WATERMARK: u64 = 7;

    #[derive(Default)]
    struct PublicationHold {
        entered: Notify,
        release: Notify,
        observed: Mutex<Option<(String, u64)>>,
    }

    impl PublicationHold {
        async fn wait_until_candidate_current(&self) -> (String, u64) {
            loop {
                if let Some(observed) = self.observed.lock().expect("hold state").clone() {
                    return observed;
                }
                self.entered.notified().await;
            }
        }

        fn release_restore(&self) {
            self.release.notify_one();
        }
    }

    #[async_trait]
    impl RestorePublicationObserver for PublicationHold {
        async fn after_candidate_current_published(&self, generation: String, sequence: u64) {
            *self.observed.lock().expect("hold state") = Some((generation, sequence));
            self.entered.notify_one();
            self.release.notified().await;
        }
    }

    fn schema() -> CreateCollectionRequest {
        CreateCollectionRequest {
            fields: BTreeMap::new(),
        }
    }

    async fn collection_ids_http(server: &TestServer) -> Vec<String> {
        let response = server.get("/collections").await;
        response.assert_status_ok();
        let mut ids: Vec<String> = response
            .json::<Value>()
            .as_array()
            .expect("collection list")
            .iter()
            .map(|value| value.as_str().expect("collection ID").to_owned())
            .collect();
        ids.sort();
        ids
    }

    fn restored_snapshot() -> SnapshotV1 {
        let candidate = Engine::new();
        candidate
            .create_collection("restored", schema())
            .expect("build independent restored candidate");
        candidate.snapshot().expect("snapshot restored candidate")
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn restore_candidate_current_cannot_be_overwritten_before_live_activation() {
        let dir = tempfile::tempdir().expect("restore race directory");
        let store = Arc::new(SegmentRdbStore::new(dir.path()).expect("segment store"));
        let live = Arc::new(Engine::new());
        live.create_collection("old", schema())
            .expect("build old live state");
        store
            .save_required(&live, WATERMARK)
            .expect("seed old CURRENT");

        let aof: SharedAof = Arc::new(Mutex::new(
            AofWriter::open(dir.path().join("aof.log")).expect("open AOF"),
        ));
        let wal: SharedWal = Arc::new(MemWal::starting_at(WATERMARK));
        let writer =
            WriteCoordinator::start_from_with_aof(wal, live.clone(), WATERMARK, aof.clone());
        let hold = Arc::new(PublicationHold::default());
        let restore_sink = Arc::new(
            SegmentRestoreSink::new(
                live.clone(),
                store.clone(),
                writer.clone() as Arc<dyn WriteSink>,
                aof,
            )
            .expect("durable restore sink")
            .with_publication_observer(hold.clone()),
        );
        let server = TestServer::new(router(
            AppState::with_components(
                live.clone(),
                Arc::new(AuthConfig::open()),
                writer as Arc<dyn WriteSink>,
            )
            .with_restore_sink(restore_sink.clone()),
        ))
        .expect("live HTTP server");

        let restore = tokio::spawn({
            let restore_sink = restore_sink.clone();
            async move { restore_sink.restore(restored_snapshot()).await }
        });
        let (candidate_name, candidate_sequence) =
            tokio::time::timeout(Duration::from_secs(1), hold.wait_until_candidate_current())
                .await
                .expect("restore must publish candidate CURRENT before the latch");
        assert_eq!(candidate_sequence, WATERMARK);
        let candidate_current = std::fs::read(dir.path().join("CURRENT"))
            .expect("read candidate CURRENT while restore is held");
        assert!(
            String::from_utf8_lossy(&candidate_current).contains(&candidate_name),
            "the latch must run after the candidate generation owns CURRENT"
        );
        let candidate_loaded = store
            .load_current_generation()
            .expect("cold-open candidate CURRENT")
            .expect("candidate CURRENT exists");
        assert_eq!(
            candidate_loaded
                .engine
                .list_collections()
                .expect("candidate collections"),
            vec!["restored"],
            "the test must observe a real restored candidate before old checkpoint starts"
        );

        let (checkpoint_started_tx, checkpoint_started_rx) = oneshot::channel();
        let mut old_checkpoint = tokio::task::spawn_blocking({
            let store = store.clone();
            let live = live.clone();
            move || {
                checkpoint_started_tx
                    .send(())
                    .expect("checkpoint task start observer");
                store.save_with_sequence(&live, WATERMARK)
            }
        });
        tokio::time::timeout(Duration::from_secs(1), checkpoint_started_rx)
            .await
            .expect("old checkpoint task must start while restore is held")
            .expect("old checkpoint task start observer stays connected");

        let old_checkpoint_finished_before_activation =
            tokio::time::timeout(Duration::from_millis(250), &mut old_checkpoint).await;
        let (unsafe_old_checkpoint_sequence, old_checkpoint_was_blocked) =
            match old_checkpoint_finished_before_activation {
                Ok(Ok(Ok(sequence))) => (Some(sequence), false),
                Ok(Ok(Err(_refusal))) => {
                    assert_eq!(
                        std::fs::read(dir.path().join("CURRENT")).expect("read refused CURRENT"),
                        candidate_current,
                        "a refused old checkpoint must leave candidate CURRENT selected"
                    );
                    (None, false)
                }
                Ok(Err(join)) => panic!("old checkpoint task panicked: {join}"),
                Err(_) => (None, true),
            };
        if old_checkpoint_was_blocked {
            assert_eq!(
                std::fs::read(dir.path().join("CURRENT")).expect("read waiting CURRENT"),
                candidate_current,
                "a waiting old checkpoint must not move CURRENT before live activation"
            );
        }

        // The test owns both controlled tasks. It must always release the
        // observer, then require each task to finish; a timeout never counts as
        // a safe result by itself.
        hold.release_restore();
        let restore_result = tokio::time::timeout(Duration::from_secs(2), restore)
            .await
            .expect("restore must finish after the controlled latch releases")
            .expect("restore task must not panic");
        if old_checkpoint_was_blocked {
            tokio::time::timeout(Duration::from_secs(2), old_checkpoint)
                .await
                .expect("waiting checkpoint must finish after restore activation")
                .expect("waiting checkpoint task must not panic")
                .expect("post-activation checkpoint may save restored live state");
        }

        assert!(
            unsafe_old_checkpoint_sequence.is_none(),
            "old live checkpoint must wait or refuse after candidate CURRENT; it published sequence {:?} before live activation",
            unsafe_old_checkpoint_sequence
        );
        restore_result.expect("restore must activate the candidate");

        let cold = store
            .load_current_generation()
            .expect("cold-open final CURRENT")
            .expect("final CURRENT exists");
        assert_eq!(
            cold.engine
                .list_collections()
                .expect("cold collection list"),
            vec!["restored"],
            "final CURRENT must describe restored data, never the old live engine"
        );
        assert_eq!(
            collection_ids_http(&server).await,
            vec!["restored"],
            "live HTTP query must agree with final CURRENT after the race"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cancelling_caller_after_candidate_current_keeps_owned_restore_to_activation() {
        let dir = tempfile::tempdir().expect("restore cancellation directory");
        let store = Arc::new(SegmentRdbStore::new(dir.path()).expect("segment store"));
        let live = Arc::new(Engine::new());
        live.create_collection("old", schema())
            .expect("build old live state");
        store
            .save_required(&live, WATERMARK)
            .expect("seed old CURRENT");

        let aof: SharedAof = Arc::new(Mutex::new(
            AofWriter::open(dir.path().join("aof.log")).expect("open AOF"),
        ));
        let wal: SharedWal = Arc::new(MemWal::starting_at(WATERMARK));
        let writer =
            WriteCoordinator::start_from_with_aof(wal, live.clone(), WATERMARK, aof.clone());
        let hold = Arc::new(PublicationHold::default());
        let restore_sink = Arc::new(
            SegmentRestoreSink::new(
                live.clone(),
                store.clone(),
                writer.clone() as Arc<dyn WriteSink>,
                aof,
            )
            .expect("durable restore sink")
            .with_publication_observer(hold.clone()),
        );
        let server = TestServer::new(router(
            AppState::with_components(
                live.clone(),
                Arc::new(AuthConfig::open()),
                writer.clone() as Arc<dyn WriteSink>,
            )
            .with_restore_sink(restore_sink.clone()),
        ))
        .expect("live HTTP server");

        let restore = tokio::spawn({
            let restore_sink = restore_sink.clone();
            async move { restore_sink.restore(restored_snapshot()).await }
        });
        let (candidate_name, candidate_sequence) =
            tokio::time::timeout(Duration::from_secs(1), hold.wait_until_candidate_current())
                .await
                .expect("restore must publish candidate CURRENT before cancellation");
        assert_eq!(candidate_sequence, WATERMARK);
        let candidate_current = std::fs::read(dir.path().join("CURRENT"))
            .expect("read candidate CURRENT before cancellation");
        assert!(
            String::from_utf8_lossy(&candidate_current).contains(&candidate_name),
            "the cancellation latch must hold after candidate CURRENT publishes"
        );
        assert_eq!(
            store
                .load_current_generation()
                .expect("cold-open candidate CURRENT")
                .expect("candidate CURRENT exists")
                .engine
                .list_collections()
                .expect("candidate collections"),
            vec!["restored"],
            "the case must observe a durable candidate before cancelling its caller"
        );

        restore.abort();
        let cancelled = restore
            .await
            .expect_err("the test must cancel the caller-owned restore future");
        assert!(
            cancelled.is_cancelled(),
            "the restore caller must be cancelled while the publication observer holds it"
        );

        // The durable candidate already owns CURRENT. Releasing the observer
        // must let the restore-owned continuation finish activation even
        // though the original caller no longer awaits it.
        hold.release_restore();
        let live_activated = tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                if collection_ids_http(&server).await == vec!["restored"] {
                    return;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .is_ok();
        assert!(
            live_activated,
            "cancelling the caller after candidate CURRENT must not cancel the durable restore continuation"
        );

        let cold = store
            .load_current_generation()
            .expect("cold-open final CURRENT after cancelled caller")
            .expect("final CURRENT exists after cancelled caller");
        assert_eq!(
            cold.engine
                .list_collections()
                .expect("cold collection list after cancelled caller"),
            vec!["restored"],
            "cold CURRENT must agree with the restore-owned live activation"
        );

        tokio::time::timeout(
            Duration::from_secs(2),
            writer.submit(RaftLogEntry::CreateCollection {
                collection_id: "after-cancelled-restore".to_owned(),
                req: schema(),
            }),
        )
        .await
        .expect("later write must finish after restore continuation activates")
        .expect("later write must apply after restore continuation activates");
        assert_eq!(
            collection_ids_http(&server).await,
            vec!["after-cancelled-restore", "restored"],
            "a later write must operate on the activated restored dataset"
        );
    }
}
