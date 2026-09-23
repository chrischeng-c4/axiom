//! Capacity admission, checkpoint progress, and committed-record HTTP E2E.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use axum_test::TestServer;
use serde_json::{json, Value};

use lumen::aof::AofWriter;
use lumen::api::{router, AppState, CheckpointSink};
use lumen::auth::AuthConfig;
use lumen::coordinator::{SharedAof, WriteCoordinator, WriteSink};
use lumen::log_entry::RaftLogEntry;
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::Engine;
use lumen::wal::{MemWal, SharedWal, WalLog, WalRecord};

mod capacity_http_contract {
    //! # Facets
    //!
    //! - Behavior: `indexing_durable_oracle.rs:10600`, `:10620`, `:10640`, and
    //!   `:10669` require a real `POST /collections/{id}/index` pre-submit refusal,
    //!   successful retry after publication, and live plus cold query recovery. Change points:
    //!   `apps/lumen/src/coordinator.rs:566-621`,
    //!   `apps/lumen/src/segment_checkpoint.rs:171-217`, and
    //!   `apps/lumen/src/api.rs:1554-1586`.
    //! - Security: `indexing_durable_oracle.rs:10603`, `:10608`, and `:10612` feed
    //!   caller-controlled large `/index` bytes and require the closed `429`, exact
    //!   `Retry-After: 1`, and unchanged MemWal/applied sequences. Change point:
    //!   `apps/lumen/src/api.rs:3269-3273` maps the pre-publication capacity error
    //!   to the HTTP boundary.
    //! - Performance: `apps/lumen/ROADMAP.md:60-70` promises a 256 MiB pending
    //!   active/frozen/reserved budget and checkpoint progress at that limit.
    //!   `indexing_durable_oracle.rs:10431-10435` pins the documented limit and
    //!   `:10593-10597` refuses a missing-429 result only after the actual raw payload
    //!   crosses it. This structural case makes no latency or RSS claim; the
    //!   30-minute workload remains the gate for those measurements.
    //! - Behavior (externally committed record): `indexing_durable_oracle.rs:11584`,
    //!   `:11592`, `:11600`, and `:11650` pin direct WAL publication, stationary
    //!   applied state while capacity is held, later application, and cold recovery.
    //!   Change points: `apps/lumen/src/coordinator.rs:297-358`
    //!   and `apps/lumen/src/segment_checkpoint.rs:172-217`.
    //! - Security (capacity boundary): `indexing_durable_oracle.rs:11530`, `:11536`,
    //!   `:11564`, and `:11592` require rejected caller-controlled input to leave WAL
    //!   and the applied watermark unchanged, and a committed record to stay invisible
    //!   before admission owns it.
    //! - Performance (same approved budget): `apps/lumen/ROADMAP.md:60-70`;
    //!   `indexing_durable_oracle.rs:10773-10803` bounds real public accounting and
    //!   `:10813-10868` establishes a staged public capacity witness without
    //!   claiming latency or RSS.

    use super::*;

    use axum::http::{header::RETRY_AFTER, StatusCode};
    use lumen::segment_checkpoint::{PendingChangeSpill, SegmentCheckpointSink};
    use lumen::segment_rdb::{MergeObserver, MergePhase};
    use std::io;
    use std::sync::mpsc;
    use storage_durable::{CommitStep, FailureInjector, FailurePoint};

    const CAPACITY_COLLECTION: &str = "pending-capacity";
    // One 6 MiB record is safely below the documented 256 MiB limit even when
    // admission conservatively reserves its future capture representation.
    const FROZEN_VALUE_COUNT: usize = 1;
    // This stays below the current 8 MiB HTTP limit even after JSON framing.
    const LARGE_KEYWORD_VALUE_BYTES: usize = 6 * 1024 * 1024 - 32 * 1024;
    const MAX_HTTP_BODY_BYTES: usize = 8 * 1024 * 1024;
    const PENDING_HARD_LIMIT_BYTES: usize = 256 * 1024 * 1024;
    const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
    // Local admission keeps the owned record, four AOF copies, and its
    // checkpointable Keyword state before publication. Start with 1 MiB rows
    // to fill quickly, then use 64 KiB rows if the first local price no longer
    // fits even though the exact 6 MiB committed value still can. A new
    // Keyword's 64 KiB value has only fixed metadata beyond its two retained
    // state copies, so that stage fits throughout the remaining <6 MiB gap
    // without mirroring the private admission estimator in this oracle.
    const FILLER_VALUE_STAGES: [usize; 2] = [1024 * 1024, 64 * 1024];
    const MAX_FILLER_VALUE_COUNT_PER_STAGE: usize = 256;
    const FILLER_RETRY_DELAY: Duration = Duration::from_millis(30);
    const MAX_FILLER_RETRIES_PER_ORDINAL: usize = 32;
    const MAX_WITNESS_SCRAPE_RETRIES: usize = 8;
    const FILL_SETUP_TIMEOUT: Duration = Duration::from_secs(30);
    // With one frozen value, this gives the active side enough unique requests
    // to cross the actual 256 MiB payload limit if the implementation never
    // applies admission. It is a finite ~270 MiB source-payload fixture.
    const MAX_LARGE_VALUE_COUNT: usize = 43;

    #[derive(Default)]
    struct NoopMergeObserver;

    impl MergeObserver for NoopMergeObserver {
        fn observe(&self, _: MergePhase) -> io::Result<()> {
            Ok(())
        }
    }

    /// Pauses the first real generation-file sync after it has captured the
    /// frozen layer. `SegmentCheckpointSink` performs this save in
    /// `spawn_blocking`, so blocking this injector never starves the Tokio
    /// workers that drive HTTP and release it.
    #[derive(Default)]
    struct HoldNextSyncFile {
        hold: Mutex<Option<(mpsc::SyncSender<()>, mpsc::Receiver<()>)>>,
    }

    impl HoldNextSyncFile {
        fn arm(&self, entered: mpsc::SyncSender<()>, release: mpsc::Receiver<()>) {
            assert!(
                self.hold
                    .lock()
                    .expect("capacity sync hold mutex")
                    .replace((entered, release))
                    .is_none(),
                "capacity test arms exactly one checkpoint sync hold",
            );
        }
    }

    impl FailureInjector for HoldNextSyncFile {
        fn check(&self, point: &FailurePoint) -> io::Result<()> {
            if point.step != CommitStep::SyncFile {
                return Ok(());
            }
            let Some((entered, release)) =
                self.hold.lock().expect("capacity sync hold mutex").take()
            else {
                return Ok(());
            };
            entered.send(()).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "capacity checkpoint readiness receiver dropped",
                )
            })?;
            // The test owns this pause until `SyncRelease` sends. A timeout here
            // would let a still-running HTTP loop publish behind its back.
            release.recv().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "capacity checkpoint release sender dropped",
                )
            })?;
            Ok(())
        }
    }

    /// Guarantees that the real checkpoint's blocking sync is released even
    /// when a later HTTP assertion fails.
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

    struct CapacityFixture {
        _dir: tempfile::TempDir,
        root: PathBuf,
        server: TestServer,
        store: Arc<SegmentRdbStore>,
        checkpoint: Arc<SegmentCheckpointSink>,
        writer: Arc<WriteCoordinator>,
        wal: Arc<MemWal>,
        hold: Arc<HoldNextSyncFile>,
        // The externally committed Full path must reuse this root. Otherwise
        // it can create a temporary spill store whose publication bypasses the
        // held configured-root SyncFile in this contract.
        _configured_capacity_owner: Option<PendingChangeSpill>,
    }

    fn capacity_fixture(configured_capacity_owner: bool) -> CapacityFixture {
        let dir = tempfile::tempdir().expect("capacity fixture directory");
        let root = dir.path().join("segments");
        let hold = Arc::new(HoldNextSyncFile::default());
        let store = Arc::new(
            SegmentRdbStore::with_failure_injector_and_merge_observer(
                &root,
                hold.clone(),
                Arc::new(NoopMergeObserver),
            )
            .expect("open observed capacity segment store"),
        );
        let aof: SharedAof = Arc::new(Mutex::new(
            AofWriter::open(dir.path().join("aof.log")).expect("open capacity AOF"),
        ));
        let engine = Arc::new(Engine::new());
        let configured_capacity_owner = configured_capacity_owner.then(|| {
            PendingChangeSpill::configured_replay(
                engine.clone(),
                store.clone(),
                Duration::from_secs(3600),
            )
        });
        let wal = Arc::new(MemWal::new());
        let shared_wal: SharedWal = wal.clone();
        let writer =
            WriteCoordinator::start_from_with_aof(shared_wal, engine.clone(), 0, aof.clone());
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
        let server = TestServer::new(router(state)).expect("capacity HTTP server");
        CapacityFixture {
            _dir: dir,
            root,
            server,
            store,
            checkpoint,
            writer,
            wal,
            hold,
            _configured_capacity_owner: configured_capacity_owner,
        }
    }

    fn capacity_external_id(ordinal: usize) -> String {
        format!("capacity-large-{ordinal:03}")
    }

    #[derive(Clone, Copy, Debug)]
    struct FillerRecord {
        stage: usize,
        ordinal: usize,
        value_bytes: usize,
    }

    fn capacity_filler_external_id(record: FillerRecord) -> String {
        format!("capacity-filler-{:02}-{:03}", record.stage, record.ordinal)
    }

    /// Each ordinal owns one unique `String` allocation. These are neither
    /// duplicate values nor overwrites, so the fixture never inflates its raw
    /// pending-byte count by resubmitting one field.
    fn capacity_payload(label: &str, ordinal: usize, bytes_len: usize) -> String {
        let mut bytes = vec![b'a' + (ordinal % 26) as u8; bytes_len];
        for offset in (0..bytes.len()).step_by(4096) {
            bytes[offset] = b'A' + ((ordinal + offset / 4096) % 26) as u8;
        }
        let prefix = format!("{label}-{ordinal:03}-");
        assert!(
            prefix.len() <= bytes.len(),
            "capacity fixture prefix must fit its legal Keyword value",
        );
        bytes[..prefix.len()].copy_from_slice(prefix.as_bytes());
        String::from_utf8(bytes).expect("ASCII capacity payload")
    }

    fn capacity_value(ordinal: usize) -> String {
        capacity_payload("capacity-payload", ordinal, LARGE_KEYWORD_VALUE_BYTES)
    }

    fn capacity_filler_value(record: FillerRecord) -> String {
        capacity_payload(
            &format!("capacity-filler-{:02}", record.stage),
            record.ordinal,
            record.value_bytes,
        )
    }

    fn capacity_index_request(ordinal: usize) -> Value {
        let value = capacity_value(ordinal);
        assert_eq!(
            value.len(),
            LARGE_KEYWORD_VALUE_BYTES,
            "each capacity item must carry the counted retained Keyword bytes",
        );
        json!({
            "items": [{
                "external_id": capacity_external_id(ordinal),
                "field": "kw",
                "value": value,
            }]
        })
    }

    fn capacity_filler_index_request(record: FillerRecord) -> Value {
        let value = capacity_filler_value(record);
        assert_eq!(
            value.len(),
            record.value_bytes,
            "each filler item must carry its distinct retained Keyword bytes",
        );
        json!({
            "items": [{
                "external_id": capacity_filler_external_id(record),
                "field": "kw",
                "value": value,
            }]
        })
    }

    async fn post_capacity_index(
        server: &TestServer,
        request: Value,
        label: &str,
    ) -> (StatusCode, Option<String>) {
        assert_eq!(
            request["items"].as_array().map(Vec::len),
            Some(1),
            "{label} uses one legal field item per /index request",
        );
        let body = serde_json::to_vec(&request).expect("serialize capacity index body");
        assert!(
            body.len() < MAX_HTTP_BODY_BYTES,
            "{label} must stay below the current 8 MiB HTTP body limit: {} bytes",
            body.len(),
        );
        let response = server
            .post(&format!("/collections/{CAPACITY_COLLECTION}/index"))
            .json(&request)
            .await;
        let retry_after = response
            .maybe_header(RETRY_AFTER)
            .and_then(|value| value.to_str().ok().map(ToOwned::to_owned));
        (response.status_code(), retry_after)
    }

    async fn capacity_index(server: &TestServer, ordinal: usize) -> (StatusCode, Option<String>) {
        post_capacity_index(server, capacity_index_request(ordinal), "capacity test").await
    }

    async fn capacity_filler_index(
        server: &TestServer,
        record: FillerRecord,
    ) -> (StatusCode, Option<String>) {
        post_capacity_index(
            server,
            capacity_filler_index_request(record),
            "capacity filler",
        )
        .await
    }

    async fn capacity_index_bounded(
        server: &TestServer,
        ordinal: usize,
    ) -> Result<(StatusCode, Option<String>)> {
        tokio::time::timeout(REQUEST_TIMEOUT, capacity_index(server, ordinal))
            .await
            .map_err(|_| anyhow::anyhow!("capacity /index request {ordinal} did not finish"))
    }

    async fn capacity_filler_index_bounded(
        server: &TestServer,
        record: FillerRecord,
    ) -> Result<(StatusCode, Option<String>)> {
        tokio::time::timeout(REQUEST_TIMEOUT, capacity_filler_index(server, record))
            .await
            .map_err(|_| {
                anyhow::anyhow!("capacity filler /index request {record:?} did not finish")
            })
    }

    async fn capacity_term_ids(server: &TestServer, value: &str) -> Vec<String> {
        let request = json!({
            "query": { "term": { "field": "kw", "value": value } },
            "limit": 8,
        });
        assert!(
            serde_json::to_vec(&request)
                .expect("serialize capacity search body")
                .len()
                < MAX_HTTP_BODY_BYTES,
            "capacity query must remain a legal HTTP body",
        );
        let response = server
            .post(&format!("/collections/{CAPACITY_COLLECTION}/search"))
            .json(&request)
            .await;
        response.assert_status_ok();
        let mut ids = response.json::<Value>()["hits"]
            .as_array()
            .expect("capacity search hits")
            .iter()
            .map(|hit| {
                hit["external_id"]
                    .as_str()
                    .expect("capacity search hit external ID")
                    .to_owned()
            })
            .collect::<Vec<_>>();
        ids.sort();
        ids
    }

    fn metric_u64(metrics: &str, name: &str) -> u64 {
        let values = metrics
            .lines()
            .filter_map(|line| {
                let (metric, value) =
                    line.split_once(|character: char| character.is_whitespace())?;
                (metric == name).then_some(value.trim())
            })
            .collect::<Vec<_>>();
        assert_eq!(
            values.len(),
            1,
            "public /metrics must publish exactly one {name} sample: {metrics}",
        );
        values[0]
            .parse::<u64>()
            .unwrap_or_else(|_| panic!("{name} must be an unsigned integer: {}", values[0]))
    }

    #[derive(Clone, Copy, Debug)]
    struct PendingBudget {
        reserved: u64,
        active: u64,
        frozen: u64,
        total: u64,
        high_water: u64,
    }

    impl PendingBudget {
        fn checkpointable_bytes(self) -> u64 {
            self.active
                .checked_add(self.frozen)
                .expect("public active plus frozen bytes must not overflow")
        }

        fn assert_documented_limit(self, phase: &str) {
            let hard_limit = PENDING_HARD_LIMIT_BYTES as u64;
            assert_eq!(
                self.total,
                self.reserved + self.active + self.frozen,
                "{phase}: public pending-change total must equal reserved plus active plus frozen: {self:?}",
            );
            assert!(
                self.total <= hard_limit,
                "{phase}: public pending-change total must stay within the documented 256 MiB budget: {self:?}",
            );
            assert!(
                self.high_water <= hard_limit,
                "{phase}: public pending-change high water must stay within the documented 256 MiB budget: {self:?}",
            );
        }
    }

    async fn public_pending_budget(server: &TestServer, phase: &str) -> PendingBudget {
        let response = server.get("/metrics").await;
        response.assert_status_ok();
        let metrics = response.text();
        let budget = PendingBudget {
            reserved: metric_u64(&metrics, "lumen_pending_change_reserved_bytes"),
            active: metric_u64(&metrics, "lumen_pending_change_active_bytes"),
            frozen: metric_u64(&metrics, "lumen_pending_change_frozen_bytes"),
            total: metric_u64(&metrics, "lumen_pending_change_total_bytes"),
            high_water: metric_u64(&metrics, "lumen_pending_change_high_water_bytes"),
        };
        budget.assert_documented_limit(phase);
        budget
    }

    fn external_value_cannot_fit_checkpointable_budget(budget: PendingBudget) -> bool {
        budget
            .checkpointable_bytes()
            .checked_add(LARGE_KEYWORD_VALUE_BYTES as u64)
            .map_or(true, |required| required > PENDING_HARD_LIMIT_BYTES as u64)
    }

    async fn fill_checkpointable_budget_until_external_value_cannot_fit(
        server: &TestServer,
    ) -> Result<Option<FillerRecord>> {
        let mut last_accepted = None;
        let mut last_local_refusal = None;
        'stage: for (stage, value_bytes) in FILLER_VALUE_STAGES.into_iter().enumerate() {
            for ordinal in 0..MAX_FILLER_VALUE_COUNT_PER_STAGE {
                let record = FillerRecord {
                    stage,
                    ordinal,
                    value_bytes,
                };
                for retry in 0..=MAX_FILLER_RETRIES_PER_ORDINAL {
                    let before =
                        public_pending_budget(server, "before capacity filler request").await;
                    if external_value_cannot_fit_checkpointable_budget(before) {
                        return Ok(last_accepted);
                    }
                    match capacity_filler_index_bounded(server, record).await? {
                        (StatusCode::OK, _) => {
                            last_accepted = Some(record);
                            break;
                        }
                        (StatusCode::TOO_MANY_REQUESTS, retry_after) => {
                            let after = public_pending_budget(
                                server,
                                "after transient capacity filler refusal",
                            )
                            .await;
                            if external_value_cannot_fit_checkpointable_budget(after) {
                                return Ok(last_accepted);
                            }
                            last_local_refusal = Some((record, retry_after, after));
                            if retry == MAX_FILLER_RETRIES_PER_ORDINAL {
                                // This size cannot consume the remaining gap.
                                // Move to the next legal value size rather than
                                // retrying an identical admission price forever.
                                continue 'stage;
                            }
                            tokio::time::sleep(FILLER_RETRY_DELAY).await;
                        }
                        (status, retry_after) => anyhow::bail!(
                            "legal capacity filler {record:?} returned {status}, retry-after={retry_after:?}"
                        ),
                    }
                }
            }
        }
        let final_budget =
            public_pending_budget(server, "after bounded staged capacity filler loop").await;
        if external_value_cannot_fit_checkpointable_budget(final_budget) {
            Ok(last_accepted)
        } else {
            anyhow::bail!(
                "{} staged legal capacity fillers did not leave less than the exact 6 MiB external Keyword lower bound in public active-plus-frozen capacity; last local refusal={last_local_refusal:?}, budget={final_budget:?}",
                FILLER_VALUE_STAGES.len() * MAX_FILLER_VALUE_COUNT_PER_STAGE,
            );
        }
    }

    #[derive(Debug)]
    struct Refusal {
        ordinal: usize,
        retry_after: Option<String>,
        applied_before: u64,
        applied_after: u64,
        wal_before: u64,
        wal_after: u64,
    }

    const CAPACITY_CHILD_CASE_ENV: &str = "LUMEN_CAPACITY_CHILD_CASE";
    const CAPACITY_CHILD_HANDSHAKE_ENV: &str = "LUMEN_CAPACITY_CHILD_HANDSHAKE";
    const LOCAL_CAPACITY_CHILD_CASE: &str = "local-http-capacity";
    const LOCAL_CAPACITY_TEST_NAME: &str = "capacity_http_contract::pending_capacity_refuses_precommit_http_index_until_paused_checkpoint_publishes";
    const EXTERNAL_CAPACITY_CHILD_CASE: &str = "externally-committed-capacity";
    const EXTERNAL_CAPACITY_TEST_NAME: &str = "capacity_http_contract::committed_external_wal_capacity_contract::externally_committed_record_waits_at_full_pending_capacity_then_survives_cold_reopen";

    /// The capacity ledger is process-global.  The parent process runs this
    /// exact test body in a fresh e2e child, and the child proves it entered the
    /// intended body before any fixture allocation.  A wrong test filter cannot
    /// turn an empty child run into a false green.
    fn capacity_child_enters(case_key: &str) -> bool {
        if std::env::var(CAPACITY_CHILD_CASE_ENV).ok().as_deref() != Some(case_key) {
            return false;
        }
        let handshake = std::env::var_os(CAPACITY_CHILD_HANDSHAKE_ENV)
            .unwrap_or_else(|| panic!("{case_key}: child needs a handshake path"));
        std::fs::write(&handshake, case_key)
            .unwrap_or_else(|error| panic!("{case_key}: write child handshake: {error}"));
        true
    }

    async fn run_isolated_capacity_case(case_key: &'static str, test_name: &'static str) {
        let child_dir = tempfile::tempdir().expect("capacity child fixture directory");
        let handshake = child_dir.path().join("entered-case");
        let executable = std::env::current_exe().expect("current e2e test executable");
        let child_handshake = handshake.clone();
        let output = tokio::task::spawn_blocking(move || {
            std::process::Command::new(executable)
                .env(CAPACITY_CHILD_CASE_ENV, case_key)
                .env(CAPACITY_CHILD_HANDSHAKE_ENV, &child_handshake)
                .arg(test_name)
                .arg("--exact")
                .arg("--nocapture")
                .arg("--test-threads=1")
                .output()
        })
        .await
        .expect("capacity child command task must join")
        .expect("start exact capacity child test");
        let entered = std::fs::read_to_string(&handshake).unwrap_or_else(|error| {
            panic!(
                "capacity child for {test_name} did not reach its intended body: {error}; stdout={} stderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            )
        });
        assert_eq!(
            entered, case_key,
            "capacity child must enter exactly {test_name}, not a different filtered body"
        );
        assert!(
            output.status.success(),
            "isolated capacity child {test_name} failed: status={} stdout={} stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    async fn pending_capacity_refuses_precommit_http_index_until_paused_checkpoint_publishes_body()
    {
        assert_eq!(
            PENDING_HARD_LIMIT_BYTES,
            256 * 1024 * 1024,
            "this real fixture is pinned to the approved 256 MiB pending-change limit",
        );
        let fixture = capacity_fixture(false);
        fixture
            .server
            .put(&format!("/collections/{CAPACITY_COLLECTION}"))
            .json(&json!({ "fields": { "kw": { "type": "keyword" } } }))
            .await
            .assert_status_ok();
        fixture
            .server
            .post(&format!("/collections/{CAPACITY_COLLECTION}/index"))
            .json(&json!({ "items": [{
                "external_id": "durable-base",
                "field": "kw",
                "value": "capacity-base",
            }] }))
            .await
            .assert_status_ok();
        assert!(
            CheckpointSink::checkpoint_now(fixture.checkpoint.as_ref())
                .await
                .expect("publish initial durable capacity base"),
            "initial capacity base must publish",
        );

        let mut frozen_ids = BTreeMap::new();
        let mut frozen_payload_bytes = 0usize;
        for ordinal in 0..FROZEN_VALUE_COUNT {
            let (status, retry_after) = capacity_index_bounded(&fixture.server, ordinal)
                .await
                .expect("the bounded frozen seed request must finish");
            assert_eq!(
                status,
                StatusCode::OK,
                "the one frozen seed must be accepted before the 256 MiB boundary, retry-after={retry_after:?}",
            );
            assert!(
                frozen_ids
                    .insert(capacity_external_id(ordinal), ordinal)
                    .is_none(),
                "frozen fixture rows must use distinct external IDs",
            );
            frozen_payload_bytes += LARGE_KEYWORD_VALUE_BYTES;
        }
        assert_eq!(
            frozen_payload_bytes, LARGE_KEYWORD_VALUE_BYTES,
            "the real paused checkpoint must retain its one distinct frozen payload",
        );

        let (entered_tx, entered_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::sync_channel(1);
        fixture.hold.arm(entered_tx, release_rx);
        let mut release = SyncRelease(Some(release_tx));
        let mut paused_checkpoint = tokio::spawn({
            let checkpoint = fixture.checkpoint.clone();
            async move { CheckpointSink::checkpoint_now(checkpoint.as_ref()).await }
        });
        let ready =
            tokio::task::spawn_blocking(move || entered_rx.recv_timeout(Duration::from_secs(30)))
                .await;
        if !matches!(ready, Ok(Ok(()))) {
            release.release();
            paused_checkpoint.abort();
            let _ = paused_checkpoint.await;
            panic!(
                "fixture checkpoint did not capture and reach its real SyncFile pause: {ready:?}"
            );
        }

        let mut active_ids = BTreeMap::new();
        let mut active_payload_bytes = 0usize;
        let mut refusal = None;
        let mut unexpected_status = None;
        for ordinal in FROZEN_VALUE_COUNT..MAX_LARGE_VALUE_COUNT {
            let applied_before = fixture.writer.applied_seq();
            let wal_before = fixture
                .wal
                .latest_seq()
                .await
                .expect("read MemWal before index");
            let response = capacity_index_bounded(&fixture.server, ordinal).await;
            let applied_after = fixture.writer.applied_seq();
            let wal_after = fixture
                .wal
                .latest_seq()
                .await
                .expect("read MemWal after index");
            let (status, retry_after) = match response {
                Ok(response) => response,
                Err(error) => {
                    unexpected_status = Some((ordinal, None, Some(error.to_string())));
                    break;
                }
            };
            match status {
                StatusCode::OK => {
                    assert!(
                        active_ids
                            .insert(capacity_external_id(ordinal), ordinal)
                            .is_none(),
                        "active fixture rows must use distinct external IDs",
                    );
                    active_payload_bytes += LARGE_KEYWORD_VALUE_BYTES;
                }
                StatusCode::TOO_MANY_REQUESTS => {
                    refusal = Some(Refusal {
                        ordinal,
                        retry_after,
                        applied_before,
                        applied_after,
                        wal_before,
                        wal_after,
                    });
                    break;
                }
                other => {
                    unexpected_status = Some((ordinal, Some(other), retry_after));
                    break;
                }
            }
        }

        // Do all worker cleanup before making any capacity assertion. The drop
        // guard also protects every panic path above.
        release.release();
        let checkpoint_result =
            tokio::time::timeout(Duration::from_secs(30), &mut paused_checkpoint).await;
        let checkpoint_error = match checkpoint_result {
            Ok(Ok(Ok(true))) => None,
            Ok(Ok(Ok(false))) => Some("paused checkpoint reported persisted=false".to_owned()),
            Ok(Ok(Err(error))) => Some(format!("paused checkpoint returned {error:#}")),
            Ok(Err(error)) => Some(format!("paused checkpoint task failed: {error}")),
            Err(_) => {
                paused_checkpoint.abort();
                let _ = paused_checkpoint.await;
                Some("paused checkpoint did not finish after release".to_owned())
            }
        };
        assert!(
            checkpoint_error.is_none(),
            "paused checkpoint must publish after its SyncFile release: {checkpoint_error:?}",
        );
        assert!(
            !frozen_ids.is_empty(),
            "fixture must retain a distinct frozen external-ID payload before capacity refusal",
        );
        assert!(
            frozen_ids
                .keys()
                .all(|external_id| !active_ids.contains_key(external_id)),
            "the active side must not overwrite frozen rows to manufacture pending bytes",
        );
        if unexpected_status.is_some() {
            panic!(
                "legal one-item /index request returned an unrelated status before capacity admission: {unexpected_status:?}",
            );
        }
        if refusal.is_none() {
            let accepted_raw_payload_bytes = frozen_payload_bytes + active_payload_bytes;
            assert!(
                accepted_raw_payload_bytes > PENDING_HARD_LIMIT_BYTES,
                "the missing-429 oracle must cross 256 MiB of actually retained distinct Keyword payload before it fails: {accepted_raw_payload_bytes}",
            );
        }
        let refusal = refusal.expect(
            "a legal pre-submit /index request must receive 429 while the captured frozen payload remains unpublished",
        );
        assert_eq!(
            refusal.retry_after.as_deref(),
            Some("1"),
            "capacity refusal must expose the documented Retry-After: 1 header",
        );
        assert_eq!(
            refusal.wal_after, refusal.wal_before,
            "capacity refusal must not publish a caller-controlled record to MemWal",
        );
        assert_eq!(
            refusal.applied_after, refusal.applied_before,
            "capacity refusal must not advance the local applied sequence",
        );

        let (retry_status, retry_after) = capacity_index_bounded(&fixture.server, refusal.ordinal)
            .await
            .expect("the bounded capacity retry must finish");
        assert_eq!(
            retry_status,
            StatusCode::OK,
            "the same request must become admissible after the frozen checkpoint publishes, retry-after={retry_after:?}",
        );
        assert_eq!(
            fixture.writer.applied_seq(),
            refusal.applied_before + 1,
            "only the retry, never the refused pre-submit request, may allocate the next sequence",
        );
        assert_eq!(
            fixture
                .wal
                .latest_seq()
                .await
                .expect("read MemWal after retry"),
            refusal.wal_before + 1,
            "MemWal must contain exactly the retry after capacity publication",
        );

        assert_eq!(
            capacity_term_ids(&fixture.server, "capacity-base").await,
            vec!["durable-base".to_owned()],
            "the initial durable base remains queryable after capacity backpressure",
        );
        assert_eq!(
            capacity_term_ids(&fixture.server, &capacity_value(0)).await,
            vec![capacity_external_id(0)],
            "a frozen distinct Keyword payload remains live after publication",
        );
        assert_eq!(
            capacity_term_ids(&fixture.server, &capacity_value(refusal.ordinal)).await,
            vec![capacity_external_id(refusal.ordinal)],
            "the retry's caller-controlled payload becomes live exactly once",
        );

        assert!(
            CheckpointSink::checkpoint_now(fixture.checkpoint.as_ref())
                .await
                .expect("publish retry checkpoint"),
            "retry checkpoint must publish",
        );
        let cold = fixture
            .store
            .load_current_generation()
            .expect("cold-open post-capacity CURRENT")
            .expect("post-capacity CURRENT exists");
        let cold_server = TestServer::new(router(AppState::open(cold.engine)))
            .expect("cold capacity HTTP server");
        assert_eq!(
            capacity_term_ids(&cold_server, "capacity-base").await,
            vec!["durable-base".to_owned()],
            "cold CURRENT preserves the durable base",
        );
        assert_eq!(
            capacity_term_ids(&cold_server, &capacity_value(0)).await,
            vec![capacity_external_id(0)],
            "cold CURRENT preserves frozen work published after the pause",
        );
        assert_eq!(
            capacity_term_ids(&cold_server, &capacity_value(refusal.ordinal)).await,
            vec![capacity_external_id(refusal.ordinal)],
            "cold CURRENT preserves the admitted retry and never a refused phantom write",
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pending_capacity_refuses_precommit_http_index_until_paused_checkpoint_publishes() {
        if capacity_child_enters(LOCAL_CAPACITY_CHILD_CASE) {
            pending_capacity_refuses_precommit_http_index_until_paused_checkpoint_publishes_body()
                .await;
        } else {
            run_isolated_capacity_case(LOCAL_CAPACITY_CHILD_CASE, LOCAL_CAPACITY_TEST_NAME).await;
        }
    }

    async fn http_checkpoint_releases_pending_capacity_relay_body() {
        let fixture = capacity_fixture(true);
        fixture
            .server
            .put(&format!("/collections/{CAPACITY_COLLECTION}"))
            .json(&json!({ "fields": { "kw": { "type": "keyword" } } }))
            .await
            .assert_status_ok();
        fixture
            .server
            .post(&format!("/collections/{CAPACITY_COLLECTION}/index"))
            .json(&json!({ "items": [{
                "external_id": "http-checkpoint-base",
                "field": "kw",
                "value": "http-checkpoint-base",
            }] }))
            .await
            .assert_status_ok();
        assert!(CheckpointSink::checkpoint_now(fixture.checkpoint.as_ref())
            .await
            .expect("publish HTTP checkpoint baseline"));

        let (seed_status, _) = capacity_index_bounded(&fixture.server, 0)
            .await
            .expect("seed capacity record");
        assert_eq!(seed_status, StatusCode::OK);
        let (entered_tx, entered_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::sync_channel(1);
        fixture.hold.arm(entered_tx, release_rx);
        let mut release = SyncRelease(Some(release_tx));
        let mut first_checkpoint = tokio::spawn({
            let checkpoint = fixture.checkpoint.clone();
            async move { CheckpointSink::checkpoint_now(checkpoint.as_ref()).await }
        });
        tokio::task::spawn_blocking(move || entered_rx.recv_timeout(REQUEST_TIMEOUT))
            .await
            .expect("checkpoint readiness task")
            .expect("first checkpoint must enter its real SyncFile pause");

        let mut refusal = None;
        for ordinal in FROZEN_VALUE_COUNT..MAX_LARGE_VALUE_COUNT {
            let result = capacity_index_bounded(&fixture.server, ordinal)
                .await
                .expect("capacity request must finish");
            if result.0 == StatusCode::TOO_MANY_REQUESTS {
                refusal = Some(ordinal);
                break;
            }
            assert_eq!(result.0, StatusCode::OK);
        }
        let filler = tokio::time::timeout(
            FILL_SETUP_TIMEOUT,
            fill_checkpointable_budget_until_external_value_cannot_fit(&fixture.server),
        )
        .await
        .expect("capacity filler setup deadline")
        .expect("capacity filler setup");
        let ordinal = refusal.expect("fixture must enter the public capacity boundary");
        let entry = RaftLogEntry::Index {
            collection_id: CAPACITY_COLLECTION.to_owned(),
            req: serde_json::from_value(capacity_index_request(ordinal))
                .expect("construct committed capacity index record"),
        };
        let sequence = fixture
            .wal
            .publish(WalRecord::new(entry))
            .await
            .expect("publish committed record behind capacity relay");

        release.release();
        let first_result = tokio::time::timeout(REQUEST_TIMEOUT, &mut first_checkpoint)
            .await
            .expect("first checkpoint completion deadline")
            .expect("first checkpoint task")
            .expect("first checkpoint result");
        assert!(first_result, "first checkpoint must publish");

        let response = tokio::time::timeout(
            REQUEST_TIMEOUT,
            fixture.server.post("/admin/checkpoint").json(&json!({})),
        )
        .await
        .expect("HTTP checkpoint deadline");
        response.assert_status_ok();
        tokio::time::timeout(REQUEST_TIMEOUT, async {
            while fixture.writer.applied_seq() < sequence {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("HTTP checkpoint must let the pending committed record leave capacity relay");
        assert_eq!(
            capacity_term_ids(&fixture.server, &capacity_value(ordinal)).await,
            vec![capacity_external_id(ordinal).to_owned()]
        );
        let _ = filler;

        let cold = fixture
            .store
            .load_current_generation()
            .expect("load CURRENT after HTTP checkpoint")
            .expect("CURRENT after HTTP checkpoint");
        let cold_server = TestServer::new(router(AppState::open(cold.engine)))
            .expect("cold HTTP checkpoint server");
        assert_eq!(
            capacity_term_ids(&cold_server, &capacity_value(ordinal)).await,
            vec![capacity_external_id(ordinal).to_owned()]
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn http_checkpoint_releases_pending_capacity_relay() {
        http_checkpoint_releases_pending_capacity_relay_body().await;
    }

    mod committed_external_wal_capacity_contract {
        //! # Facets
        //!
        //! - Behavior: `indexing_durable_oracle.rs:11584-11597` requires the
        //!   committed record to wait behind the held checkpoint and remain
        //!   query-invisible. `:11600-11634` and `:11636-11676` then require
        //!   exactly-once live application and cold recovery. These assertions
        //!   exercise externally delivered capacity handling in
        //!   `apps/lumen/src/coordinator.rs:481-531` and configured checkpoint
        //!   ownership in `apps/lumen/src/segment_checkpoint.rs:411-435`.
        //! - Security: `indexing_durable_oracle.rs:11530-11541` proves the
        //!   caller-controlled local input rejected at the HTTP boundary did
        //!   not publish a WAL record or advance the applied watermark.
        //!   `:11564-11597` keeps the later committed copy in the WAL but
        //!   invisible until it owns budget. This change reads only the process's
        //!   public metrics; it opens no new caller path, parser, or file input
        //!   boundary.
        //! - Performance: `apps/lumen/ROADMAP.md:60-70` promises, verbatim,
        //!   "Pending active, frozen, and reserved changes have a 256 MiB total
        //!   budget." `indexing_durable_oracle.rs:10773-10803` checks that
        //!   public total and high-water accounting stay within that limit.
        //!   `:10813-10868` and `:11543-11552` use public active plus frozen
        //!   bytes as the stable retained-work lower bound before the 6 MiB
        //!   committed value enters. Test timeouts bound cleanup only; they make
        //!   no latency claim.

        use super::*;
        use lumen::wal::{WalLog, WalRecord};

        fn externally_committed_refused_entry(ordinal: usize) -> RaftLogEntry {
            RaftLogEntry::Index {
                collection_id: CAPACITY_COLLECTION.to_owned(),
                // This is the same valid logical item that the local HTTP route
                // rejected before publication. It enters through a real
                // already-committed WAL publisher, not `submit`.
                req: serde_json::from_value(capacity_index_request(ordinal))
                    .expect("construct externally committed capacity Index record"),
            }
        }

        async fn wait_for_applied_sequence(
            writer: &WriteCoordinator,
            sequence: u64,
            context: &str,
        ) {
            let observed = tokio::time::timeout(Duration::from_secs(30), async {
                loop {
                    if writer.applied_seq() >= sequence {
                        return;
                    }
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await;
            assert!(
                observed.is_ok(),
                "{context}: applied sequence stayed {} below committed sequence {sequence}",
                writer.applied_seq(),
            );
        }

        async fn externally_committed_record_waits_at_full_pending_capacity_then_survives_cold_reopen_body(
        ) {
            assert_eq!(
                PENDING_HARD_LIMIT_BYTES,
                256 * 1024 * 1024,
                "this contract uses the approved 256 MiB pending-change limit",
            );
            let fixture = capacity_fixture(true);
            fixture
                .server
                .put(&format!("/collections/{CAPACITY_COLLECTION}"))
                .json(&json!({ "fields": { "kw": { "type": "keyword" } } }))
                .await
                .assert_status_ok();
            fixture
                .server
                .post(&format!("/collections/{CAPACITY_COLLECTION}/index"))
                .json(&json!({ "items": [{
                        "external_id": "committed-capacity-base",
                        "field": "kw",
                        "value": "committed-capacity-base-value",
                    }] }))
                .await
                .assert_status_ok();
            assert!(
                CheckpointSink::checkpoint_now(fixture.checkpoint.as_ref())
                    .await
                    .expect("publish committed-capacity baseline"),
                "baseline must publish before the held checkpoint",
            );

            // Capture one real, distinct frozen payload. The configured capacity
            // owner already shares this root, and its SyncFile pause is the only
            // artificial block; all admission and WAL work remains production.
            let (frozen_status, frozen_retry_after) = capacity_index_bounded(&fixture.server, 0)
                .await
                .expect("frozen seed request must finish");
            assert_eq!(
                frozen_status,
                StatusCode::OK,
                "the bounded frozen seed must fit before capacity is full, retry-after={frozen_retry_after:?}",
            );
            let (entered_tx, entered_rx) = mpsc::sync_channel(1);
            let (release_tx, release_rx) = mpsc::sync_channel(1);
            fixture.hold.arm(entered_tx, release_rx);
            let mut release = SyncRelease(Some(release_tx));
            let mut paused_checkpoint = tokio::spawn({
                let checkpoint = fixture.checkpoint.clone();
                async move { CheckpointSink::checkpoint_now(checkpoint.as_ref()).await }
            });
            let readiness = tokio::task::spawn_blocking(move || {
                entered_rx.recv_timeout(Duration::from_secs(30))
            })
            .await;
            if !matches!(readiness, Ok(Ok(()))) {
                release.release();
                paused_checkpoint.abort();
                let _ = paused_checkpoint.await;
                panic!(
                    "held checkpoint did not reach its real SyncFile pause before capacity setup: {readiness:?}"
                );
            }

            // The initial 429 remains a public HTTP boundary assertion. It does
            // not itself prove that a foreign record's lower retained-state cost
            // cannot fit, so the filler below establishes that premise separately.
            let mut refusal = None;
            let mut unexpected = None;
            let mut first_active_ordinal = None;
            for ordinal in FROZEN_VALUE_COUNT..MAX_LARGE_VALUE_COUNT {
                let applied_before = fixture.writer.applied_seq();
                let wal_before = fixture
                    .wal
                    .latest_seq()
                    .await
                    .expect("read WAL before local capacity request");
                let result = capacity_index_bounded(&fixture.server, ordinal).await;
                let applied_after = fixture.writer.applied_seq();
                let wal_after = fixture
                    .wal
                    .latest_seq()
                    .await
                    .expect("read WAL after local capacity request");
                match result {
                    Ok((StatusCode::OK, _)) => {
                        first_active_ordinal.get_or_insert(ordinal);
                    }
                    Ok((StatusCode::TOO_MANY_REQUESTS, retry_after)) => {
                        refusal = Some(Refusal {
                            ordinal,
                            retry_after,
                            applied_before,
                            applied_after,
                            wal_before,
                            wal_after,
                        });
                        break;
                    }
                    Ok((status, retry_after)) => {
                        unexpected = Some((ordinal, status, retry_after));
                        break;
                    }
                    Err(error) => {
                        unexpected = Some((
                            ordinal,
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Some(error.to_string()),
                        ));
                        break;
                    }
                }
            }

            // Stage-source relief may release the transient local reservation,
            // but cannot lower active plus frozen retained work. Fill through
            // the real HTTP/coordinator path until that public lower bound leaves
            // less than the exact external Keyword value itself.
            let filler_result = tokio::time::timeout(
                FILL_SETUP_TIMEOUT,
                fill_checkpointable_budget_until_external_value_cannot_fit(&fixture.server),
            )
            .await
            .map_err(|_| anyhow::anyhow!("capacity filler setup exceeded its bounded test timeout"))
            .and_then(|result| result);

            // Adjacent public snapshots reject a transient source-ledger
            // observation. If retained work changes while the witness is being
            // sampled, resample before publication. The held shared-root
            // SyncFile prevents a successful checkpoint publication from
            // reducing active plus frozen during this setup.
            let capacity_witness = if filler_result.is_ok()
                && refusal.is_some()
                && unexpected.is_none()
            {
                let mut changed = None;
                let mut stable = None;
                for _ in 0..MAX_WITNESS_SCRAPE_RETRIES {
                    let first = public_pending_budget(
                        &fixture.server,
                        "first external-capacity witness scrape",
                    )
                    .await;
                    let second = public_pending_budget(
                        &fixture.server,
                        "second external-capacity witness scrape",
                    )
                    .await;
                    if first.checkpointable_bytes() == second.checkpointable_bytes() {
                        stable = Some(second);
                        break;
                    }
                    changed = Some((first, second));
                    tokio::time::sleep(FILLER_RETRY_DELAY).await;
                }
                match stable {
                    None => Err(anyhow::anyhow!(
                        "active-plus-frozen retained capacity did not stabilize across {} bounded public witness retries: {changed:?}",
                        MAX_WITNESS_SCRAPE_RETRIES,
                    )),
                    Some(second) if !external_value_cannot_fit_checkpointable_budget(second) => {
                        Err(anyhow::anyhow!(
                            "public active-plus-frozen capacity still leaves room for the exact {}-byte external Keyword lower bound: {second:?}",
                            LARGE_KEYWORD_VALUE_BYTES,
                        ))
                    }
                    Some(_) if paused_checkpoint.is_finished() => Err(anyhow::anyhow!(
                        "the held shared-root checkpoint completed before external publication"
                    )),
                    Some(second) => Ok(second),
                }
            } else {
                Err(anyhow::anyhow!(
                    "capacity filler, required local HTTP refusal, or local HTTP status was unavailable"
                ))
            };

            let external_baseline = if capacity_witness.is_ok() {
                Some((
                    fixture.writer.applied_seq(),
                    fixture
                        .wal
                        .latest_seq()
                        .await
                        .expect("read WAL before externally committed publish"),
                ))
            } else {
                None
            };
            let external_publish =
                if let (Some(refusal), Some(_)) = (refusal.as_ref(), external_baseline) {
                    let entry = externally_committed_refused_entry(refusal.ordinal);
                    Some(
                        tokio::time::timeout(
                            Duration::from_secs(2),
                            fixture.wal.publish(WalRecord::new(entry)),
                        )
                        .await,
                    )
                } else {
                    None
                };
            let externally_committed_sequence = external_publish
                .as_ref()
                .and_then(|timed| timed.as_ref().ok())
                .and_then(|published| published.as_ref().ok())
                .copied();
            let applied_while_held = if let Some(sequence) = externally_committed_sequence {
                Some(
                    tokio::time::timeout(Duration::from_secs(2), async {
                        loop {
                            if fixture.writer.applied_seq() >= sequence {
                                return fixture.writer.applied_seq();
                            }
                            tokio::time::sleep(Duration::from_millis(5)).await;
                        }
                    })
                    .await,
                )
            } else {
                None
            };
            let applied_after_held_observation = fixture.writer.applied_seq();
            let visible_while_held = if let Some(refusal) = refusal.as_ref() {
                Some(
                    tokio::time::timeout(
                        Duration::from_secs(2),
                        capacity_term_ids(&fixture.server, &capacity_value(refusal.ordinal)),
                    )
                    .await,
                )
            } else {
                None
            };

            // This checkpoint was selected and captured before the external
            // record was published. Its successful completion independently
            // frees the frozen charge; it must not wait for a record that cannot
            // enter the public active-plus-frozen capacity witness.
            release.release();
            let paused_completion =
                tokio::time::timeout(Duration::from_secs(30), &mut paused_checkpoint).await;
            let paused_error = match paused_completion {
                Ok(Ok(Ok(true))) => None,
                Ok(Ok(Ok(false))) => Some("held checkpoint reported persisted=false".to_owned()),
                Ok(Ok(Err(error))) => Some(format!("held checkpoint returned {error:#}")),
                Ok(Err(error)) => Some(format!("held checkpoint task failed: {error}")),
                Err(_) => {
                    paused_checkpoint.abort();
                    let _ = paused_checkpoint.await;
                    Some("held checkpoint did not finish after its release".to_owned())
                }
            };

            assert!(
                unexpected.is_none(),
                "a legal local capacity request returned an unrelated result: {unexpected:?}",
            );
            let refusal = refusal.expect(
                "fixture must reach real pre-submit 429 before testing committed external WAL input",
            );
            let first_active_ordinal = first_active_ordinal.expect(
                "a full-capacity fixture must retain at least one distinct active payload before refusal",
            );
            assert_eq!(
                refusal.retry_after.as_deref(),
                Some("1"),
                "the local full-capacity boundary must retain Retry-After: 1",
            );
            assert_eq!(
                refusal.wal_after, refusal.wal_before,
                "the rejected local request must not become a committed WAL record",
            );
            assert_eq!(
                refusal.applied_after, refusal.applied_before,
                "the rejected local request must not advance the applied watermark",
            );
            let last_filler = filler_result.expect(
                "bounded legal HTTP filler must establish the public active-plus-frozen capacity witness",
            );
            let witness = capacity_witness.expect(
                "public active-plus-frozen capacity must prove the external Keyword cannot fit before publication",
            );
            assert!(
                external_value_cannot_fit_checkpointable_budget(witness),
                "the final public witness must leave less than the exact external Keyword lower bound: {witness:?}",
            );
            assert!(
                paused_error.is_none(),
                "the independently captured checkpoint must complete after release: {paused_error:?}",
            );
            let (applied_before_external, wal_before_external) = external_baseline.expect(
                "external publish baseline must be sampled only after the stable capacity witness",
            );
            let external_sequence = external_publish
                .expect("externally committed publish must run after the stable capacity witness")
                .expect("externally committed MemWal publish must not block at full capacity")
                .expect("externally committed MemWal publish must allocate a durable sequence");
            assert_eq!(
                external_sequence,
                wal_before_external + 1,
                "the exact request rejected before submission must become the next committed WAL record after filler work",
            );
            assert!(
                external_sequence > refusal.wal_before,
                "the local pre-submit refusal must not consume a WAL sequence before the later committed record",
            );
            assert_eq!(
                fixture
                    .wal
                    .latest_seq()
                    .await
                    .expect("read WAL after externally committed publish"),
                external_sequence,
                "the externally committed record must remain in the WAL while apply waits",
            );
            let applied_while_held = applied_while_held
                .expect("applied-watermark observation must run after external publication");
            assert!(
                applied_while_held.is_err(),
                "a committed record whose exact retained Keyword value cannot fit the public active-plus-frozen budget must wait for checkpoint release instead of applying early: {applied_while_held:?}",
            );
            assert_eq!(
                applied_after_held_observation, applied_before_external,
                "the applied watermark must remain stationary while the full-budget checkpoint is held",
            );
            assert_eq!(
                visible_while_held
                    .expect("read query observation must run after external publication")
                    .expect("read query must finish while capacity is held"),
                Vec::<String>::new(),
                "an externally committed record must not become query-visible before capacity owns it",
            );

            wait_for_applied_sequence(
                fixture.writer.as_ref(),
                external_sequence,
                "the independently completed checkpoint must let the committed record apply",
            )
            .await;
            let external_value = capacity_value(refusal.ordinal);
            assert_eq!(
                capacity_term_ids(&fixture.server, &external_value).await,
                vec![capacity_external_id(refusal.ordinal)],
                "the formerly refused request must become live exactly once through its committed WAL record",
            );
            assert_eq!(
                capacity_term_ids(&fixture.server, "committed-capacity-base-value").await,
                vec!["committed-capacity-base".to_owned()],
                "the independently published checkpoint must retain prior durable data",
            );
            assert_eq!(
                capacity_term_ids(&fixture.server, &capacity_value(0)).await,
                vec![capacity_external_id(0)],
                "the frozen payload that released capacity remains live",
            );
            assert_eq!(
                capacity_term_ids(&fixture.server, &capacity_value(first_active_ordinal)).await,
                vec![capacity_external_id(first_active_ordinal)],
                "the later live state must retain active work from before the external record",
            );
            if let Some(last_filler) = last_filler {
                assert_eq!(
                    capacity_term_ids(&fixture.server, &capacity_filler_value(last_filler))
                        .await,
                    vec![capacity_filler_external_id(last_filler)],
                    "the final public filler record must remain live after it establishes the capacity witness",
                );
            }

            assert!(
                CheckpointSink::checkpoint_now(fixture.checkpoint.as_ref())
                    .await
                    .expect("publish externally committed record"),
                "a later ordinary checkpoint must persist the applied external record",
            );
            let cold = fixture
                .store
                .load_current_generation()
                .expect("cold-open externally committed capacity CURRENT")
                .expect("externally committed capacity CURRENT exists");
            let cold_server = TestServer::new(router(AppState::open(cold.engine)))
                .expect("cold externally committed capacity HTTP server");
            assert_eq!(
                capacity_term_ids(&cold_server, &external_value).await,
                vec![capacity_external_id(refusal.ordinal)],
                "cold reopen must retain the externally committed record after checkpoint release",
            );
            assert_eq!(
                capacity_term_ids(&cold_server, "committed-capacity-base-value").await,
                vec!["committed-capacity-base".to_owned()],
                "cold reopen must retain the prior durable base",
            );
            assert_eq!(
                capacity_term_ids(&cold_server, &capacity_value(0)).await,
                vec![capacity_external_id(0)],
                "cold reopen must retain the frozen checkpoint payload that made capacity available",
            );
            assert_eq!(
                capacity_term_ids(&cold_server, &capacity_value(first_active_ordinal)).await,
                vec![capacity_external_id(first_active_ordinal)],
                "cold reopen must retain active work from before the external record",
            );
            if let Some(last_filler) = last_filler {
                assert_eq!(
                    capacity_term_ids(&cold_server, &capacity_filler_value(last_filler))
                        .await,
                    vec![capacity_filler_external_id(last_filler)],
                    "cold reopen must retain the final public filler record that established capacity",
                );
            }
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn externally_committed_record_waits_at_full_pending_capacity_then_survives_cold_reopen(
        ) {
            if super::capacity_child_enters(super::EXTERNAL_CAPACITY_CHILD_CASE) {
                externally_committed_record_waits_at_full_pending_capacity_then_survives_cold_reopen_body().await;
            } else {
                super::run_isolated_capacity_case(
                    super::EXTERNAL_CAPACITY_CHILD_CASE,
                    super::EXTERNAL_CAPACITY_TEST_NAME,
                )
                .await;
            }
        }
    }
}
