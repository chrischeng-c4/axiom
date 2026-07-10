// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! Autonomous reshard phase driver end-to-end (#1319 R2 executor; #1381).
//!
//! No live kind cluster: [`lumen::operator::reshard_driver::ClusterControl`]
//! is the seam. This harness fakes only the k8s-facing half (spec patches +
//! StatefulSet readiness) with an in-memory `Arc<Mutex<Lumen>>` standing in
//! for the persisted CR (matching how a real restart only loses in-process
//! state, never the API server's copy) and points the driver's admin-verb
//! HTTP calls at real [`axum_test::TestServer`]s bound to real local TCP
//! ports, each backed by a real [`lumen::storage::Engine`] — the same
//! `POST /admin/backup:scoped` / `POST /admin/reshard:apply` / `POST
//! /admin/reshard:evict` wire calls a live cluster's pods would answer.
#![cfg(feature = "operator")]

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum_test::{TestServer, TestServerConfig, Transport};
use serde_json::json;

use lumen::api::{router, AppState, CheckpointSink};
use lumen::operator::crd::{
    Lumen, LumenReshardStatus, LumenSpec, LumenStatus, ReshardPhase, ReshardPolicy, ServingSpec,
    ShardMapSpec,
};
use lumen::operator::reshard_driver::{drive_tick, ClusterControl, DriveOutcome};
use lumen::routing::VirtualBucketShardMap;
use lumen::storage::Engine;

const VIRTUAL_BUCKET_COUNT: u32 = 8;
const NAMESPACE: &str = "acme";
const NAME: &str = "search";

/// A shard's admin/query surface: a real bound HTTP server backed by a real
/// engine, plus the base URL the driver should reach it at.
struct Shard {
    server: TestServer,
    base_url: String,
}

fn spin_up_shard() -> Shard {
    let engine = Arc::new(Engine::new());
    let app = router(AppState::open(engine));
    let server = TestServer::new_with_config(
        app,
        TestServerConfig {
            transport: Some(Transport::HttpRandomPort),
            ..TestServerConfig::default()
        },
    )
    .expect("bind real test server");
    let base_url = server
        .server_address()
        .expect("server bound to a real address")
        .to_string()
        .trim_end_matches('/')
        .to_string();
    Shard { server, base_url }
}

/// #1389 AC2: a [`CheckpointSink`] test double whose success/failure is
/// controlled from outside — stands in for a real `SegmentCheckpointSink`
/// (`src/bin/lumen.rs`) hitting a transient disk error, without needing an
/// actual segment store on disk. Counts calls so a test can assert the
/// driver actually invoked `/admin/checkpoint` per touched shard, not just
/// that it happened to succeed.
struct ControllableCheckpoint {
    fail: Arc<AtomicBool>,
    calls: Arc<AtomicI64>,
}

#[async_trait]
impl CheckpointSink for ControllableCheckpoint {
    async fn checkpoint_now(&self) -> anyhow::Result<bool> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.fail.load(Ordering::SeqCst) {
            anyhow::bail!("simulated checkpoint failure");
        }
        Ok(true)
    }
}

/// Like [`spin_up_shard`] but with a controllable checkpoint sink wired in
/// place of the default no-op, so a test can force `/admin/checkpoint` to
/// fail on demand.
fn spin_up_shard_with_checkpoint(fail: Arc<AtomicBool>, calls: Arc<AtomicI64>) -> Shard {
    let engine = Arc::new(Engine::new());
    let state =
        AppState::open(engine).with_checkpoint(Arc::new(ControllableCheckpoint { fail, calls }));
    let app = router(state);
    let server = TestServer::new_with_config(
        app,
        TestServerConfig {
            transport: Some(Transport::HttpRandomPort),
            ..TestServerConfig::default()
        },
    )
    .expect("bind real test server");
    let base_url = server
        .server_address()
        .expect("server bound to a real address")
        .to_string()
        .trim_end_matches('/')
        .to_string();
    Shard { server, base_url }
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

/// Bucket for `external_id` under the pre-split, single-shard map — the
/// exact map the driver itself derives from `spec.shardCount == 1` with no
/// explicit `spec.shardMap.assignments` (see
/// `reshard_driver::current_shard_map`).
fn bucket_of(external_id: &str) -> u32 {
    let map = VirtualBucketShardMap::balanced(0, VIRTUAL_BUCKET_COUNT, 1).unwrap();
    map.route_document("u", None, external_id).bucket
}

fn initial_lumen(max_shard_bytes: Option<u64>, blocking_condition: Option<&str>) -> Lumen {
    let spec = LumenSpec {
        image: "lumen:latest".into(),
        image_pull_policy: None,
        shard_count: 1,
        shard_map: ShardMapSpec {
            version: 0,
            virtual_bucket_count: VIRTUAL_BUCKET_COUNT,
            assignments: Vec::new(),
        },
        replicas_per_shard: 1,
        voter_count: 1,
        log_format: Default::default(),
        log_level: None,
        auth: Default::default(),
        tokens_secret: None,
        tokens_secret_provider_class: None,
        serving: ServingSpec::default(),
        reshard_policy: ReshardPolicy {
            max_shard_bytes,
            ..Default::default()
        },
        observability: false,
    };
    let mut lumen = Lumen::new(NAME, spec);
    lumen.metadata.namespace = Some(NAMESPACE.to_string());
    lumen.status = Some(LumenStatus {
        reshard: LumenReshardStatus {
            blocking_conditions: blocking_condition.into_iter().map(str::to_string).collect(),
            ..Default::default()
        },
        ..Default::default()
    });
    lumen
}

/// RFC 7386 JSON Merge Patch: nested objects merge recursively, a `null`
/// leaf deletes that key, everything else replaces. Mirrors real k8s
/// `Patch::Merge` semantics closely enough for this harness's `spec`-only
/// patches.
fn merge_patch(base: &mut serde_json::Value, patch: &serde_json::Value) {
    match (base.as_object_mut(), patch.as_object()) {
        (Some(base_obj), Some(patch_obj)) => {
            for (key, value) in patch_obj {
                if value.is_null() {
                    base_obj.remove(key);
                } else if value.is_object()
                    && base_obj.get(key).map(|b| b.is_object()).unwrap_or(false)
                {
                    merge_patch(base_obj.get_mut(key).unwrap(), value);
                } else {
                    base_obj.insert(key.clone(), value.clone());
                }
            }
        }
        _ => *base = patch.clone(),
    }
}

/// Fakes only the k8s-facing half of [`ClusterControl`] (spec patches +
/// StatefulSet readiness): `cluster` is an `Arc<Mutex<Lumen>>` shared across
/// however many `FakeControl`s are constructed in a test, standing in for
/// the API server's persisted copy of the CR — the thing that survives an
/// operator restart. Constructing a brand new `FakeControl` over the same
/// `Arc` is this harness's "operator process restarted" simulation (AC2):
/// zero in-process state carries over, only what is in `cluster`.
///
/// StatefulSet readiness always reports the CR's own `shardCount` (the new
/// pod is "ready" the instant `shardCount` is bumped) — the generic
/// `libs/operator` apply loop that would actually create it is exercised
/// separately by `tests/operator_render.rs`; this harness only needs to
/// prove the reshard driver's own phase/data-migration logic.
struct FakeControl {
    cluster: Arc<Mutex<Lumen>>,
    shard_urls: Vec<String>,
    restart_trigger_calls: AtomicI64,
}

impl FakeControl {
    fn new(cluster: Arc<Mutex<Lumen>>, shard_urls: Vec<String>) -> Self {
        Self {
            cluster,
            shard_urls,
            restart_trigger_calls: AtomicI64::new(0),
        }
    }

    fn snapshot(&self) -> Lumen {
        self.cluster.lock().unwrap().clone()
    }
}

#[async_trait]
impl ClusterControl for FakeControl {
    async fn patch_spec(
        &self,
        _namespace: &str,
        _name: &str,
        patch: serde_json::Value,
    ) -> anyhow::Result<()> {
        let mut guard = self.cluster.lock().unwrap();
        let mut value = serde_json::to_value(&*guard).expect("Lumen serializes");
        merge_patch(&mut value, &patch);
        *guard = serde_json::from_value(value).expect("Lumen deserializes after patch");
        Ok(())
    }

    async fn statefulset_ready_replicas(
        &self,
        _namespace: &str,
        _name: &str,
    ) -> anyhow::Result<i64> {
        Ok(self.cluster.lock().unwrap().spec.shard_count as i64)
    }

    async fn trigger_rolling_restart(&self, _namespace: &str, _name: &str) -> anyhow::Result<()> {
        self.restart_trigger_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn admin_token(
        &self,
        _namespace: &str,
        _lumen: &Lumen,
    ) -> anyhow::Result<Option<String>> {
        Ok(None)
    }

    fn shard_base_url(&self, _namespace: &str, _name: &str, shard: u32) -> String {
        self.shard_urls[shard as usize].clone()
    }
}

/// AC1 (narrowed; see #1381's report for the full live-kind-cluster
/// discussion) + AC2 + AC3: a full `Complete -> PrepareSplit -> Splitting ->
/// CatchingUp -> Complete` run, including a simulated operator-process
/// restart mid-`Splitting`, against two real HTTP servers backed by real
/// engines. Proves: real documents physically move shard0 -> shard1 via the
/// real `#1380` admin verbs (a genuine non-test caller of `bucket_moves` /
/// `snapshot_reshard_batches`, AC3), the source shard evicts exactly the
/// moved documents, `spec.shardMap` only flips at the very end, and the
/// workflow reaches `Complete` even though the driving process was rebuilt
/// from scratch mid-workflow (AC2).
#[tokio::test]
async fn full_split_resumes_after_restart_and_reaches_complete() {
    let shard0 = spin_up_shard();
    let shard1 = spin_up_shard();
    create_users_collection(&shard0.server).await;
    create_users_collection(&shard1.server).await;

    let ids: Vec<String> = (0..40).map(|i| format!("u-{i:03}")).collect();
    for id in &ids {
        index_user(&shard0.server, id).await;
    }
    assert_eq!(total_docs(&shard0.server).await, 40);

    // Buckets 0..3 move to the new shard under split_one_shard (8 buckets,
    // 1 -> 2 shards: 8/2 = 4 lowest-numbered buckets move).
    let moving: Vec<&String> = ids.iter().filter(|id| bucket_of(id) < 4).collect();
    let staying: Vec<&String> = ids.iter().filter(|id| bucket_of(id) >= 4).collect();
    assert!(
        !moving.is_empty() && !staying.is_empty(),
        "fixture must split across both groups"
    );

    let cluster = Arc::new(Mutex::new(initial_lumen(
        Some(1_000_000_000),
        Some("urgentThresholdCrossed"),
    )));
    let shard_urls = vec![shard0.base_url.clone(), shard1.base_url.clone()];
    let http = reqwest::Client::new();

    // Tick 1: Complete -> PrepareSplit.
    let control = FakeControl::new(cluster.clone(), shard_urls.clone());
    let lumen = control.snapshot();
    assert_eq!(
        lumen.spec.reshard_policy.workflow.phase,
        ReshardPhase::Complete
    );
    let outcome = drive_tick(&control, &http, &lumen).await;
    assert_eq!(
        outcome,
        DriveOutcome::StartedSplit {
            target_shard_count: 2
        }
    );
    assert_eq!(
        control.snapshot().spec.reshard_policy.workflow.phase,
        ReshardPhase::PrepareSplit
    );

    // Tick 2: PrepareSplit -> Splitting (readiness is instant in this fake).
    let lumen = control.snapshot();
    let outcome = drive_tick(&control, &http, &lumen).await;
    assert_eq!(outcome, DriveOutcome::AdvancedToSplitting);
    assert_eq!(
        control.snapshot().spec.reshard_policy.workflow.phase,
        ReshardPhase::Splitting
    );

    // --- Simulated operator-process restart: drop `control`, rebuild a
    // brand new one over the same persisted `cluster`. No migration has run
    // yet (still `Splitting`, no batches applied) — this is the mid-Splitting
    // restart AC2 asks for.
    drop(control);
    assert_eq!(
        total_docs(&shard1.server).await,
        0,
        "no migration before restart"
    );
    let control = FakeControl::new(cluster.clone(), shard_urls.clone());

    // Tick 3 (post-restart): Splitting -> CatchingUp; the real migration
    // pass runs here (real bucket_moves + snapshot_reshard_batches +
    // POST /admin/backup:scoped + POST /admin/reshard:apply calls).
    let lumen = control.snapshot();
    let outcome = drive_tick(&control, &http, &lumen).await;
    match &outcome {
        DriveOutcome::MigratedBatches { batches } => assert!(*batches > 0),
        other => panic!("expected MigratedBatches with batches > 0, got {other:?}"),
    }
    assert_eq!(
        control.snapshot().spec.reshard_policy.workflow.phase,
        ReshardPhase::CatchingUp
    );

    // Data is on shard1 now; shard0 has not been evicted yet (eviction is a
    // separate, later step).
    for id in &moving {
        assert!(
            has_doc(&shard1.server, id).await,
            "missing migrated doc {id}"
        );
    }
    assert_eq!(total_docs(&shard1.server).await, moving.len() as u64);
    assert_eq!(
        total_docs(&shard0.server).await,
        40,
        "source not evicted yet"
    );

    // Tick 4: CatchingUp -> Complete (re-sync pass + eviction + shardMap
    // cutover).
    let lumen = control.snapshot();
    let outcome = drive_tick(&control, &http, &lumen).await;
    assert_eq!(outcome, DriveOutcome::CompletedSplit { new_map_version: 1 });

    let final_lumen = control.snapshot();
    assert_eq!(
        final_lumen.spec.reshard_policy.workflow.phase,
        ReshardPhase::Complete
    );
    assert!(final_lumen
        .spec
        .reshard_policy
        .workflow
        .target_shard_count
        .is_none());
    assert_eq!(final_lumen.spec.shard_map.version, 1);
    assert_eq!(
        final_lumen.spec.shard_map.assignments.len(),
        VIRTUAL_BUCKET_COUNT as usize
    );
    assert_eq!(control.restart_trigger_calls.load(Ordering::SeqCst), 1);

    // Source shard evicted exactly the moved documents; nothing else.
    for id in &moving {
        assert!(
            !has_doc(&shard0.server, id).await,
            "{id} should have been evicted from shard0"
        );
    }
    for id in &staying {
        assert!(
            has_doc(&shard0.server, id).await,
            "{id} should still be on shard0"
        );
    }
    assert_eq!(total_docs(&shard0.server).await, staying.len() as u64);
    assert_eq!(total_docs(&shard1.server).await, moving.len() as u64);

    // Re-running the driver on an already-`Complete` workflow with a
    // cleared threshold (what the separate live-usage loop would report
    // now that capacity has doubled) is a no-op: it does not loop forever
    // re-driving a workflow that has nothing left to do.
    control
        .cluster
        .lock()
        .unwrap()
        .status
        .as_mut()
        .unwrap()
        .reshard
        .blocking_conditions = vec![];
    let lumen = control.snapshot();
    let outcome = drive_tick(&control, &http, &lumen).await;
    assert!(matches!(outcome, DriveOutcome::NoOp(_)));
}

/// AC4: `maxShardBytes` unset means the driver never starts a split, no
/// matter how many ticks run or what `status.reshard.blockingConditions`
/// says — R3's core safety rail, proven at the `drive_tick` entry point
/// (not just the `should_start_split` unit) and across repeated ticks.
#[tokio::test]
async fn drive_tick_never_transitions_when_max_shard_bytes_unset() {
    let cluster = Arc::new(Mutex::new(initial_lumen(
        None,
        Some("urgentThresholdCrossed"),
    )));
    let control = FakeControl::new(cluster.clone(), vec!["http://unused.invalid".to_string()]);
    let http = reqwest::Client::new();

    for _ in 0..5 {
        let lumen = control.snapshot();
        let outcome = drive_tick(&control, &http, &lumen).await;
        assert!(matches!(outcome, DriveOutcome::NoOp(_)));
    }
    assert_eq!(
        control.snapshot().spec.reshard_policy.workflow.phase,
        ReshardPhase::Complete
    );
    assert_eq!(control.snapshot().spec.shard_count, 1);
}

/// #1389 AC2: the reshard driver's cutover (`shardMap` patch +
/// `trigger_rolling_restart`) does not fire until `POST /admin/checkpoint`
/// succeeds on every touched shard — a failing checkpoint on either shard
/// leaves the workflow in `CatchingUp` (resumable, #1381 semantics) rather
/// than advancing to `Complete`, and never triggers the restart that would
/// otherwise race the not-yet-durable migration against a pod restart.
#[tokio::test]
async fn cutover_blocked_until_every_touched_shard_checkpoints() {
    let checkpoint_fail = Arc::new(AtomicBool::new(false));
    let checkpoint_calls = Arc::new(AtomicI64::new(0));
    let shard0 = spin_up_shard_with_checkpoint(checkpoint_fail.clone(), checkpoint_calls.clone());
    let shard1 = spin_up_shard_with_checkpoint(checkpoint_fail.clone(), checkpoint_calls.clone());
    create_users_collection(&shard0.server).await;
    create_users_collection(&shard1.server).await;

    let ids: Vec<String> = (0..40).map(|i| format!("u-{i:03}")).collect();
    for id in &ids {
        index_user(&shard0.server, id).await;
    }
    let moving: Vec<&String> = ids.iter().filter(|id| bucket_of(id) < 4).collect();
    assert!(!moving.is_empty());

    let cluster = Arc::new(Mutex::new(initial_lumen(
        Some(1_000_000_000),
        Some("urgentThresholdCrossed"),
    )));
    let shard_urls = vec![shard0.base_url.clone(), shard1.base_url.clone()];
    let http = reqwest::Client::new();
    let control = FakeControl::new(cluster.clone(), shard_urls.clone());

    // Drive to CatchingUp (Complete -> PrepareSplit -> Splitting -> CatchingUp,
    // the real migration pass runs on tick 3).
    for _ in 0..3 {
        let lumen = control.snapshot();
        drive_tick(&control, &http, &lumen).await;
    }
    assert_eq!(
        control.snapshot().spec.reshard_policy.workflow.phase,
        ReshardPhase::CatchingUp
    );
    assert_eq!(control.restart_trigger_calls.load(Ordering::SeqCst), 0);

    // Force every touched shard's `/admin/checkpoint` to fail. The next tick
    // (CatchingUp -> would-be Complete) must report `Blocked`, leave the
    // phase at `CatchingUp`, and must NOT have triggered a rolling restart —
    // the durability gate sits strictly before the cutover patch/restart.
    checkpoint_fail.store(true, Ordering::SeqCst);
    let lumen = control.snapshot();
    let outcome = drive_tick(&control, &http, &lumen).await;
    match &outcome {
        DriveOutcome::Blocked(msg) => assert!(
            msg.contains("checkpoint") || msg.contains("admin/checkpoint"),
            "expected a checkpoint-related Blocked message, got: {msg}"
        ),
        other => panic!("expected Blocked while checkpoint fails, got {other:?}"),
    }
    assert_eq!(
        control.snapshot().spec.reshard_policy.workflow.phase,
        ReshardPhase::CatchingUp,
        "must not advance to Complete while a touched shard's checkpoint is failing"
    );
    assert_eq!(
        control.restart_trigger_calls.load(Ordering::SeqCst),
        0,
        "must not trigger a rolling restart before every touched shard is durable"
    );
    assert_eq!(
        control.snapshot().spec.shard_map.version,
        0,
        "cutover patch must not have applied"
    );

    // Recover: once checkpoints succeed again, the very next tick completes
    // the workflow normally — proving the earlier failure left it resumable
    // rather than wedged.
    checkpoint_fail.store(false, Ordering::SeqCst);
    let lumen = control.snapshot();
    let outcome = drive_tick(&control, &http, &lumen).await;
    assert_eq!(outcome, DriveOutcome::CompletedSplit { new_map_version: 1 });
    assert_eq!(
        control.snapshot().spec.reshard_policy.workflow.phase,
        ReshardPhase::Complete
    );
    assert_eq!(control.restart_trigger_calls.load(Ordering::SeqCst), 1);
    assert!(
        checkpoint_calls.load(Ordering::SeqCst) >= 2,
        "expected at least one /admin/checkpoint call per touched shard across the failing \
         and succeeding attempts, got {}",
        checkpoint_calls.load(Ordering::SeqCst)
    );
}
// CODEGEN-END
