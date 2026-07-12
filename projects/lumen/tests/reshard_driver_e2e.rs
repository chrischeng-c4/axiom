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
use std::time::Duration;

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
    // #1396 R3: `AppState::open`'s default `NoopCheckpoint` sink reports
    // `persisted: false` (vacuously — no durable store configured), which
    // the driver's `checkpoint_shard` now correctly treats as a failed
    // durability gate rather than a satisfied one. Tests using this helper
    // want a shard that behaves like a real, working checkpoint sink (a
    // shard whose data actually reaches durable storage), not one that
    // exercises the checkpoint-failure path itself — that path has its own
    // dedicated coverage via `spin_up_shard_with_checkpoint` in
    // `cutover_blocked_until_every_touched_shard_checkpoints`. So wire a
    // permanently-succeeding `ControllableCheckpoint` here instead of
    // leaving the default no-op sink, which would now wedge every full-split
    // e2e test at the cutover gate forever.
    let engine = Arc::new(Engine::new());
    let state = AppState::open(engine).with_checkpoint(Arc::new(ControllableCheckpoint::instant(
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicI64::new(0)),
    )));
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

/// #1389 AC2: a [`CheckpointSink`] test double whose success/failure is
/// controlled from outside — stands in for a real `SegmentCheckpointSink`
/// (`src/bin/lumen.rs`) hitting a transient disk error, without needing an
/// actual segment store on disk. Counts calls so a test can assert the
/// driver actually invoked `/admin/checkpoint` per touched shard, not just
/// that it happened to succeed.
struct ControllableCheckpoint {
    fail: Arc<AtomicBool>,
    calls: Arc<AtomicI64>,
    /// #1443 AC1: an artificial per-call delay standing in for a real slow
    /// durable-store write — lets a test make one phase of the fenced
    /// `CatchingUp` sequence realistically slow without a live disk, so it
    /// can prove the fence survives longer than a single un-refreshed TTL.
    /// `Duration::ZERO` (the default via [`Self::instant`]) for every
    /// pre-#1443 caller — behavior for those is unchanged.
    delay: Duration,
}

impl ControllableCheckpoint {
    fn instant(fail: Arc<AtomicBool>, calls: Arc<AtomicI64>) -> Self {
        Self {
            fail,
            calls,
            delay: Duration::ZERO,
        }
    }
}

#[async_trait]
impl CheckpointSink for ControllableCheckpoint {
    async fn checkpoint_now(&self) -> anyhow::Result<bool> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if !self.delay.is_zero() {
            tokio::time::sleep(self.delay).await;
        }
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
    spin_up_shard_with_checkpoint_delay(fail, calls, Duration::ZERO)
}

/// Like [`spin_up_shard_with_checkpoint`], but `/admin/checkpoint` sleeps
/// `delay` before resolving (#1443 AC1's slow-checkpoint double).
fn spin_up_shard_with_checkpoint_delay(
    fail: Arc<AtomicBool>,
    calls: Arc<AtomicI64>,
    delay: Duration,
) -> Shard {
    let engine = Arc::new(Engine::new());
    let state = AppState::open(engine).with_checkpoint(Arc::new(ControllableCheckpoint {
        fail,
        calls,
        delay,
    }));
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

async fn delete_user(s: &TestServer, external_id: &str) {
    s.delete(&format!("/collections/u/index/{external_id}"))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);
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
            // #1396 R5: `should_start_split` now requires the status's
            // measurement generation to match the CR's *current*
            // `spec.shardMap.version` (0 here, this fixture's initial map)
            // in addition to a crossed-threshold string condition. Model a
            // status write that was actually fresh when it landed, not one
            // that happens to fail the new freshness check purely by
            // fixture omission.
            usage_measured_at_map_version: Some(0),
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
    /// #1443 AC1: overrides [`ClusterControl::write_fence_ttl_secs`] when
    /// set, so a test can arm/re-arm with a short TTL instead of waiting out
    /// the real 120s production default.
    fence_ttl_secs: Option<u64>,
    /// #1458 R1/AC1: overrides [`ClusterControl::serving_topology_converged`]
    /// when set, standing in for a StatefulSet rollout that has not (or has
    /// just) finished reaching every pod `Ready` on the new topology —
    /// `None` (the default) means "converged", matching the trait's default
    /// `Ok(true)` so every pre-#1458 caller of this fake is unaffected.
    topology_converged: Option<Arc<AtomicBool>>,
    /// #1458 R1/AC1: counts calls to `serving_topology_converged`, so a test
    /// can assert the driver actually re-checked (and, via
    /// `fence_arm_calls`/HTTP mocks, re-armed) on repeated ticks rather than
    /// caching a stale answer in-process.
    convergence_check_calls: AtomicI64,
    /// #1467 R5/AC5: overrides [`ClusterControl::serving_pods_report_map_version`]
    /// when set — standing in for a rolling restart whose StatefulSet-level
    /// rollout has already finished (`topology_converged` true) but whose
    /// pods have not all actually loaded the new shard map yet. `None` (the
    /// default) matches the trait's own default `Ok(true)`, so every
    /// pre-#1467 caller of this fake is unaffected.
    pods_report_map_version: Option<Arc<AtomicBool>>,
    /// #1467 R5/AC5: counts calls to `serving_pods_report_map_version`, so a
    /// test can assert it is only scraped once StatefulSet-level rollout
    /// convergence is already true (never wastefully mid-rollout).
    pods_report_map_version_calls: AtomicI64,
}

impl FakeControl {
    fn new(cluster: Arc<Mutex<Lumen>>, shard_urls: Vec<String>) -> Self {
        Self {
            cluster,
            shard_urls,
            restart_trigger_calls: AtomicI64::new(0),
            fence_ttl_secs: None,
            topology_converged: None,
            convergence_check_calls: AtomicI64::new(0),
            pods_report_map_version: None,
            pods_report_map_version_calls: AtomicI64::new(0),
        }
    }

    fn with_fence_ttl_secs(mut self, ttl_secs: u64) -> Self {
        self.fence_ttl_secs = Some(ttl_secs);
        self
    }

    fn with_topology_converged(mut self, converged: Arc<AtomicBool>) -> Self {
        self.topology_converged = Some(converged);
        self
    }

    fn with_pods_report_map_version(mut self, reported: Arc<AtomicBool>) -> Self {
        self.pods_report_map_version = Some(reported);
        self
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

    fn write_fence_ttl_secs(&self) -> u64 {
        self.fence_ttl_secs
            .unwrap_or_else(lumen::operator::reshard_driver::default_write_fence_ttl_secs)
    }

    async fn serving_topology_converged(
        &self,
        _namespace: &str,
        _name: &str,
        _desired_replicas: i64,
    ) -> anyhow::Result<bool> {
        self.convergence_check_calls.fetch_add(1, Ordering::SeqCst);
        Ok(self
            .topology_converged
            .as_ref()
            .map(|flag| flag.load(Ordering::SeqCst))
            .unwrap_or(true))
    }

    async fn serving_pods_report_map_version(
        &self,
        _http: &reqwest::Client,
        _namespace: &str,
        _name: &str,
        _shard_count: u32,
        _map_version: u64,
    ) -> anyhow::Result<bool> {
        self.pods_report_map_version_calls
            .fetch_add(1, Ordering::SeqCst);
        Ok(self
            .pods_report_map_version
            .as_ref()
            .map(|flag| flag.load(Ordering::SeqCst))
            .unwrap_or(true))
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

    // Tick 5 (#1458 R1): `Complete` with `shardMap.version` freshly bumped
    // and no `convergedShardMapVersion` recorded yet must run the
    // topology-convergence step before anything else — this `FakeControl`
    // reports every StatefulSet instantly ready (the trait default), so
    // convergence is confirmed on the very next tick.
    let lumen = control.snapshot();
    let outcome = drive_tick(&control, &http, &lumen).await;
    assert_eq!(outcome, DriveOutcome::TopologyConverged { map_version: 1 });
    assert_eq!(
        control
            .snapshot()
            .spec
            .reshard_policy
            .workflow
            .converged_shard_map_version,
        Some(1)
    );

    // Re-running the driver on an already-`Complete`, already-converged
    // workflow with a cleared threshold (what the separate live-usage loop
    // would report now that capacity has doubled) is a no-op: it does not
    // loop forever re-driving a workflow that has nothing left to do.
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
/// AC1 (#1396 R1): a target-shard checkpoint failure — modeling a crash or
/// transient durability fault on the shard the driver just migrated data
/// to — must block the workflow strictly *before* any source eviction is
/// even attempted, not just before the cutover patch. Proves the new
/// migrate -> checkpoint(target) -> evict(sources) -> checkpoint(sources) ->
/// cutover ordering: while the target's checkpoint is failing, every moved
/// document is still fully present on the (never-evicted) source shard, so
/// a resume can always re-derive the missing durable copy without data
/// loss. Once the fault clears, the very next tick evicts, checkpoints the
/// sources, and cuts over normally — proving the earlier block left the
/// workflow resumable rather than wedged.
#[tokio::test]
async fn cutover_ordering_never_evicts_before_target_checkpoint_succeeds() {
    let source_fail = Arc::new(AtomicBool::new(false));
    let source_calls = Arc::new(AtomicI64::new(0));
    let target_fail = Arc::new(AtomicBool::new(true)); // target starts "crashed".
    let target_calls = Arc::new(AtomicI64::new(0));
    let shard0 = spin_up_shard_with_checkpoint(source_fail.clone(), source_calls.clone());
    let shard1 = spin_up_shard_with_checkpoint(target_fail.clone(), target_calls.clone());
    create_users_collection(&shard0.server).await;
    create_users_collection(&shard1.server).await;

    let ids: Vec<String> = (0..40).map(|i| format!("u-{i:03}")).collect();
    for id in &ids {
        index_user(&shard0.server, id).await;
    }
    let moving: Vec<&String> = ids.iter().filter(|id| bucket_of(id) < 4).collect();
    let staying: Vec<&String> = ids.iter().filter(|id| bucket_of(id) >= 4).collect();
    assert!(!moving.is_empty() && !staying.is_empty());

    let cluster = Arc::new(Mutex::new(initial_lumen(
        Some(1_000_000_000),
        Some("urgentThresholdCrossed"),
    )));
    let shard_urls = vec![shard0.base_url.clone(), shard1.base_url.clone()];
    let http = reqwest::Client::new();
    let control = FakeControl::new(cluster.clone(), shard_urls.clone());

    // Drive to CatchingUp with the real migration pass already run (tick 3):
    // the target-shard fault has not been reached yet (checkpointing only
    // happens on the CatchingUp -> Complete attempt).
    for _ in 0..3 {
        let lumen = control.snapshot();
        drive_tick(&control, &http, &lumen).await;
    }
    assert_eq!(
        control.snapshot().spec.reshard_policy.workflow.phase,
        ReshardPhase::CatchingUp
    );
    // Migration already copied the moving docs onto the target, even though
    // the target's checkpoint sink is still "crashed" — checkpointing and
    // migrating are independent steps, and migration always runs first.
    for id in &moving {
        assert!(
            has_doc(&shard1.server, id).await,
            "migration should have already copied {id}"
        );
    }

    // CatchingUp -> would-be Complete: the target's checkpoint fails, so the
    // whole tick must Block *before* any eviction is attempted.
    let lumen = control.snapshot();
    let outcome = drive_tick(&control, &http, &lumen).await;
    assert!(
        matches!(outcome, DriveOutcome::Blocked(_)),
        "expected Blocked while the target shard's checkpoint is failing, got {outcome:?}"
    );
    assert_eq!(
        control.snapshot().spec.reshard_policy.workflow.phase,
        ReshardPhase::CatchingUp,
        "must not advance past CatchingUp while the target checkpoint is failing"
    );
    assert_eq!(control.restart_trigger_calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        control.snapshot().spec.shard_map.version,
        0,
        "cutover must not have applied"
    );

    // The crux of R1: the source shard must still hold every moved
    // document, untouched, because eviction is only ever attempted *after*
    // the target's checkpoint succeeds — a target-shard crash before that
    // point can never lose data, because the source copy was never removed.
    assert_eq!(
        total_docs(&shard0.server).await,
        40,
        "source must retain every document (nothing evicted) while the target checkpoint fails"
    );
    for id in &moving {
        assert!(
            has_doc(&shard0.server, id).await,
            "{id} must still be recoverable from the source shard"
        );
    }
    assert_eq!(
        source_calls.load(Ordering::SeqCst),
        0,
        "the source shard's own checkpoint must not even be attempted before eviction, and \
         eviction must not have run yet either"
    );

    // Recover: once the target shard's fault clears, the very next tick
    // evicts, checkpoints the sources, and cuts over — proving the block
    // above left the workflow resumable, not wedged, and that no data was
    // lost across the simulated crash.
    target_fail.store(false, Ordering::SeqCst);
    let lumen = control.snapshot();
    let outcome = drive_tick(&control, &http, &lumen).await;
    assert_eq!(outcome, DriveOutcome::CompletedSplit { new_map_version: 1 });
    assert_eq!(
        control.snapshot().spec.reshard_policy.workflow.phase,
        ReshardPhase::Complete
    );
    assert_eq!(control.restart_trigger_calls.load(Ordering::SeqCst), 1);
    assert!(
        source_calls.load(Ordering::SeqCst) >= 1,
        "source shard must now be checkpointed too"
    );

    for id in &moving {
        assert!(
            has_doc(&shard1.server, id).await,
            "{id} should be on the target shard"
        );
        assert!(
            !has_doc(&shard0.server, id).await,
            "{id} should have been evicted from the source"
        );
    }
    for id in &staying {
        assert!(
            has_doc(&shard0.server, id).await,
            "{id} should still be on the source shard"
        );
    }
    assert_eq!(total_docs(&shard0.server).await, staying.len() as u64);
    assert_eq!(total_docs(&shard1.server).await, moving.len() as u64);
}

/// AC2 (#1396 R2): a document written into a moving bucket on the source
/// shard *during* `CatchingUp` — after the initial migration pass already
/// ran, but before the final convergence pass that closes the cutover —
/// must still end up present on exactly one shard once the workflow
/// reaches `Complete`: never silently dropped by eviction, and never left
/// duplicated on both shards. This is exactly the gap #1396's review
/// flagged (a document indexed to the source after that source's data was
/// already copied to the target, but before the source got evicted, used
/// to only exist wherever the copy-then-evict race happened to land it).
/// The final `CatchingUp` tick's write-fence-protected pass — which always
/// re-migrates a fresh snapshot of the source before evicting anything —
/// is what closes this window; this proves it end to end with a real late
/// write and a real driver run, no mocked fence calls.
#[tokio::test]
async fn late_write_to_moving_bucket_during_catching_up_survives_cutover_exactly_once() {
    let shard0 = spin_up_shard();
    let shard1 = spin_up_shard();
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

    // Drive to CatchingUp with the initial migration pass already run.
    for _ in 0..3 {
        let lumen = control.snapshot();
        drive_tick(&control, &http, &lumen).await;
    }
    assert_eq!(
        control.snapshot().spec.reshard_policy.workflow.phase,
        ReshardPhase::CatchingUp
    );

    // A late write lands directly on the source shard, in a moving bucket,
    // strictly after the migration pass above already ran — the source
    // still legitimately owns writes for this bucket until cutover
    // (`spec.shardMap` has not flipped yet), so this write is indexed
    // there, exactly like a real client write racing the reshard.
    let late_id = (0..)
        .map(|i| format!("late-{i:03}"))
        .find(|id| bucket_of(id) < 4)
        .unwrap();
    index_user(&shard0.server, &late_id).await;
    assert!(has_doc(&shard0.server, &late_id).await);
    assert!(!has_doc(&shard1.server, &late_id).await, "not migrated yet");

    // Final CatchingUp -> Complete tick: the real fence-protected pass
    // re-migrates a fresh snapshot of the source (picking up the late
    // write), checkpoints, evicts, checkpoints, and cuts over.
    let lumen = control.snapshot();
    let outcome = drive_tick(&control, &http, &lumen).await;
    assert_eq!(outcome, DriveOutcome::CompletedSplit { new_map_version: 1 });

    // The late write must be present on exactly one shard afterward — the
    // target it now belongs to — never lost, never duplicated.
    let on_target = has_doc(&shard1.server, &late_id).await;
    let on_source = has_doc(&shard0.server, &late_id).await;
    assert!(
        on_target && !on_source,
        "late write to a moving bucket must survive cutover on exactly the target shard \
         (on_target={on_target}, on_source={on_source})"
    );

    // Every originally-moved document also still converged correctly.
    for id in &moving {
        assert!(has_doc(&shard1.server, id).await);
        assert!(!has_doc(&shard0.server, id).await);
    }
}

/// #1442 R2 regression: the write fence armed over `CatchingUp`'s moving
/// buckets must still be armed immediately after `CompletedSplit` returns —
/// clearing it right after `trigger_rolling_restart` reopens exactly the
/// window the fence exists to close, since pods only read `SHARD_MAP_*` env
/// at boot and a rolling restart takes real time to reach every pod. Proven
/// behaviorally: a write landing directly on the (evicted) source shard for
/// a bucket the just-completed split moved away must still be rejected
/// (503 `bucket_write_paused`), not silently accepted, in the same tick
/// `CompletedSplit` was returned from — before `WRITE_FENCE_TTL_SECS` or any
/// later tick could have cleared it.
#[tokio::test]
async fn write_fence_stays_armed_immediately_after_completed_split() {
    let shard0 = spin_up_shard();
    let shard1 = spin_up_shard();
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

    // Drive all the way to the CatchingUp -> Complete tick that completes
    // the split and triggers the rolling restart.
    for _ in 0..3 {
        let lumen = control.snapshot();
        drive_tick(&control, &http, &lumen).await;
    }
    let lumen = control.snapshot();
    let outcome = drive_tick(&control, &http, &lumen).await;
    assert_eq!(outcome, DriveOutcome::CompletedSplit { new_map_version: 1 });
    assert_eq!(control.restart_trigger_calls.load(Ordering::SeqCst), 1);

    // A write for one of the moved buckets, landing directly on the source
    // shard exactly as it would from a client reaching an old-map pod that
    // hasn't been restarted yet, must still be paused by the write fence —
    // not silently accepted onto a shard that no longer owns this bucket.
    let moved_id = moving[0];
    let resp = shard0
        .server
        .post("/collections/u/index")
        .json(&json!({
            "items": [{
                "external_id": moved_id,
                "field": "email",
                "value": format!("{moved_id}@late.example"),
            }]
        }))
        .await;
    assert_eq!(
        resp.status_code(),
        503,
        "write fence must still be armed immediately after CompletedSplit, got body: {:?}",
        resp.text()
    );
}

/// #1443 R2/AC2: a DELETE acked on the still-owning source during
/// `CatchingUp`, strictly after that bucket's document was already
/// additively copied to the target by an earlier migration pass, must not
/// resurrect at cutover. The final fenced pass is authoritative (replace,
/// not merge) for every moving bucket's document set, so the now-deleted id
/// is absent from the live source snapshot it reads and gets pruned off the
/// target instead of surviving there as a stale copy.
///
/// #1458 R2: `delete_external_id` is fence-covered like every other write
/// now (a DELETE landing on a fenced bucket gets 503 `bucket_write_paused`,
/// see `reshard_admin_e2e.rs`'s `reshard_fence_blocks_delete_on_the_fenced_
/// bucket`) — this scenario stays green unchanged because `advance_catching_
/// up` clears the fence at the end of every non-final `CatchingUp` tick
/// (before this test's `delete_user` call runs), so the bucket is unfenced
/// at the moment of this particular delete.
#[tokio::test]
async fn delete_during_split_does_not_resurrect_after_cutover() {
    let shard0 = spin_up_shard();
    let shard1 = spin_up_shard();
    create_users_collection(&shard0.server).await;
    create_users_collection(&shard1.server).await;

    let ids: Vec<String> = (0..40).map(|i| format!("u-{i:03}")).collect();
    for id in &ids {
        index_user(&shard0.server, id).await;
    }
    let moving: Vec<&String> = ids.iter().filter(|id| bucket_of(id) < 4).collect();
    assert!(moving.len() >= 2, "need at least two moving docs");

    let cluster = Arc::new(Mutex::new(initial_lumen(
        Some(1_000_000_000),
        Some("urgentThresholdCrossed"),
    )));
    let shard_urls = vec![shard0.base_url.clone(), shard1.base_url.clone()];
    let http = reqwest::Client::new();
    let control = FakeControl::new(cluster.clone(), shard_urls.clone());

    // Drive to CatchingUp with the initial migration pass already run — the
    // doc we're about to delete has already been additively copied onto the
    // target shard by that pass.
    for _ in 0..3 {
        let lumen = control.snapshot();
        drive_tick(&control, &http, &lumen).await;
    }
    assert_eq!(
        control.snapshot().spec.reshard_policy.workflow.phase,
        ReshardPhase::CatchingUp
    );
    let deleted_id = moving[0].clone();
    assert!(has_doc(&shard1.server, &deleted_id).await, "already copied");

    // The DELETE is acked directly on the source — still the owner of writes
    // for this bucket until cutover, exactly like the late-write scenario —
    // and is fence-exempt, so it must succeed even mid-split.
    delete_user(&shard0.server, &deleted_id).await;
    assert!(!has_doc(&shard0.server, &deleted_id).await);
    // Not yet reconciled off the target: the additive copy from the earlier
    // pass is still sitting there until the final authoritative pass runs.
    assert!(has_doc(&shard1.server, &deleted_id).await, "not pruned yet");

    // Final CatchingUp -> Complete tick: the authoritative re-sync must not
    // resurrect the deleted id on the target.
    let lumen = control.snapshot();
    let outcome = drive_tick(&control, &http, &lumen).await;
    assert_eq!(outcome, DriveOutcome::CompletedSplit { new_map_version: 1 });

    assert!(
        !has_doc(&shard0.server, &deleted_id).await && !has_doc(&shard1.server, &deleted_id).await,
        "a delete acked mid-split must not resurrect on either shard after cutover"
    );

    // Every other originally-moved (non-deleted) document still converged
    // correctly, proving the prune is scoped to exactly the deleted id.
    for id in moving.iter().skip(1) {
        assert!(has_doc(&shard1.server, id).await);
        assert!(!has_doc(&shard0.server, id).await);
    }
}

/// #1443 R1/AC1: a single fenced `CatchingUp -> Complete` tick whose own
/// checkpoint round alone takes longer than one un-refreshed TTL must never
/// let the fence expire mid-sequence. Proven with a deliberately short 2s
/// TTL and a 1.5s-per-call artificial checkpoint delay on both shards (the
/// tick's two sequential checkpoint rounds — target, then sources — alone
/// sum to ~3s, comfortably past the 2s TTL if it were armed only once at the
/// start): a write probe fired straight at the source shard ~2.3s into the
/// tick — after the original single-arm deadline would already have lapsed,
/// but well inside the window R1's before-each-phase-boundary re-arms keep
/// open — must still be rejected (503), not silently accepted onto a shard
/// whose bucket is mid-cutover.
#[tokio::test]
async fn write_fence_survives_a_tick_longer_than_a_single_ttl() {
    let never_fail = Arc::new(AtomicBool::new(false));
    let source_calls = Arc::new(AtomicI64::new(0));
    let target_calls = Arc::new(AtomicI64::new(0));
    let checkpoint_delay = Duration::from_millis(1500);
    let shard0 =
        spin_up_shard_with_checkpoint_delay(never_fail.clone(), source_calls, checkpoint_delay);
    let shard1 = spin_up_shard_with_checkpoint_delay(never_fail, target_calls, checkpoint_delay);
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
    let control = FakeControl::new(cluster.clone(), shard_urls.clone()).with_fence_ttl_secs(2);

    // Drive to CatchingUp (the initial, unfenced migration pass that lands
    // this phase transition has no checkpoints in it, so the delayed
    // checkpoint sinks above are not yet exercised).
    for _ in 0..3 {
        let lumen = control.snapshot();
        drive_tick(&control, &http, &lumen).await;
    }
    assert_eq!(
        control.snapshot().spec.reshard_policy.workflow.phase,
        ReshardPhase::CatchingUp
    );

    let moved_id = moving[0].clone();
    let probe_client = reqwest::Client::new();
    let probe_url = format!("{}/collections/u/index", shard0.base_url);
    let probe_body = json!({
        "items": [{ "external_id": moved_id, "field": "email", "value": "late@x.com" }]
    });

    // Run the long, checkpoint-delayed final tick concurrently with a probe
    // write timed to land inside the gap an un-refreshed 2s TTL would have
    // already missed.
    let lumen = control.snapshot();
    let final_tick = drive_tick(&control, &http, &lumen);
    let probe = async {
        tokio::time::sleep(Duration::from_millis(2300)).await;
        probe_client.post(&probe_url).json(&probe_body).send().await
    };
    let (outcome, probe_result) = tokio::join!(final_tick, probe);
    assert_eq!(outcome, DriveOutcome::CompletedSplit { new_map_version: 1 });

    let probe_resp = probe_result.expect("probe request completes");
    assert_eq!(
        probe_resp.status(),
        reqwest::StatusCode::SERVICE_UNAVAILABLE,
        "write fence must still be armed ~2.3s into a tick whose checkpoints alone take ~3s; a \
         single un-refreshed 2s TTL would already have expired by then"
    );
}

/// #1458 R1/AC1: after `CompletedSplit`, the driver must keep re-arming the
/// write-pause fence over the buckets that just moved — and keep reporting
/// `AwaitingTopologyConvergence` — every tick the rolling restart has not
/// yet reached every serving pod, deriving "converging" purely from
/// persisted `spec.shardMap.version` vs `workflow.convergedShardMapVersion`
/// (not driver memory: a fresh `FakeControl` over the same `cluster`
/// mid-test simulates a driver restart). Only once
/// [`ClusterControl::serving_topology_converged`] reports `true` does the
/// fence actually clear and `convergedShardMapVersion` get persisted.
#[tokio::test]
async fn fence_stays_armed_across_ticks_until_topology_convergence_confirmed() {
    let shard0 = spin_up_shard();
    let shard1 = spin_up_shard();
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
    let converged = Arc::new(AtomicBool::new(false));
    let control = FakeControl::new(cluster.clone(), shard_urls.clone())
        .with_topology_converged(converged.clone());

    // Drive Complete -> PrepareSplit -> Splitting -> CatchingUp -> Complete.
    for _ in 0..4 {
        let lumen = control.snapshot();
        drive_tick(&control, &http, &lumen).await;
    }
    let after_cutover = control.snapshot();
    assert_eq!(
        after_cutover.spec.reshard_policy.workflow.phase,
        ReshardPhase::Complete
    );
    assert_eq!(after_cutover.spec.shard_map.version, 1);
    assert_eq!(
        after_cutover
            .spec
            .reshard_policy
            .workflow
            .converged_shard_map_version,
        None,
        "cutover itself must not claim convergence — only a confirmed-Ready poll may"
    );

    let moved_id = moving[0];
    let probe = || async {
        shard0
            .server
            .post("/collections/u/index")
            .json(&json!({
                "items": [{
                    "external_id": moved_id,
                    "field": "email",
                    "value": "late@x.com",
                }]
            }))
            .await
    };

    // Topology not converged yet: repeated ticks must keep re-arming the
    // fence (writes to the moved bucket stay paused) and keep reporting
    // AwaitingTopologyConvergence, no matter how many ticks run.
    for _ in 0..3 {
        let lumen = control.snapshot();
        let outcome = drive_tick(&control, &http, &lumen).await;
        assert_eq!(
            outcome,
            DriveOutcome::AwaitingTopologyConvergence { map_version: 1 }
        );
        assert_eq!(
            probe().await.status_code(),
            503,
            "the fence must stay re-armed every tick topology has not converged"
        );
    }
    assert!(
        control.convergence_check_calls.load(Ordering::SeqCst) >= 3,
        "serving_topology_converged must be re-checked every tick, not cached in-process"
    );

    // #1458 R1/AC2: resumable across a simulated driver restart — a brand
    // new `FakeControl` over the same persisted `cluster` (zero in-process
    // state carried over) must derive the same still-converging decision
    // purely from `spec.shardMap.version` vs `workflow.
    // convergedShardMapVersion`.
    let restarted = FakeControl::new(cluster.clone(), shard_urls.clone())
        .with_topology_converged(converged.clone());
    let lumen = restarted.snapshot();
    let outcome = drive_tick(&restarted, &http, &lumen).await;
    assert_eq!(
        outcome,
        DriveOutcome::AwaitingTopologyConvergence { map_version: 1 }
    );
    assert_eq!(probe().await.status_code(), 503);

    // Topology now converged: the very next tick must clear the fence and
    // persist convergedShardMapVersion.
    converged.store(true, Ordering::SeqCst);
    let lumen = restarted.snapshot();
    let outcome = drive_tick(&restarted, &http, &lumen).await;
    assert_eq!(outcome, DriveOutcome::TopologyConverged { map_version: 1 });
    assert_eq!(
        restarted
            .snapshot()
            .spec
            .reshard_policy
            .workflow
            .converged_shard_map_version,
        Some(1)
    );
    assert_eq!(
        probe().await.status_code(),
        200,
        "the fence must be cleared once topology convergence is confirmed"
    );

    // And a further tick is a clean no-op — no repeated fence churn once
    // convergence is already recorded.
    let lumen = restarted.snapshot();
    let outcome = drive_tick(&restarted, &http, &lumen).await;
    assert!(matches!(outcome, DriveOutcome::NoOp(_)));
}

/// #1467 R5/AC5: StatefulSet-level rollout convergence
/// (`serving_topology_converged`) alone must not clear the write-pause
/// fence — every serving pod must also be observed actually reporting the
/// new shard map version (`serving_pods_report_map_version`) before
/// convergence is confirmed. Proves both directions: (1) rollout converged
/// but pods not yet reporting the new map keeps `AwaitingTopologyConvergence`
/// and the fence armed, and the pod-report scrape is only even attempted
/// once the rollout itself is converged (never wasted mid-rollout); (2) once
/// pods catch up, the very next tick clears the fence exactly like the
/// StatefulSet-only check did before #1467.
#[tokio::test]
async fn topology_convergence_requires_every_pod_to_report_the_new_map_version() {
    let shard0 = spin_up_shard();
    let shard1 = spin_up_shard();
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
    // StatefulSet rollout itself is instantly converged (the trait/fake
    // default), but the pods have not yet all loaded the new map.
    let pods_report = Arc::new(AtomicBool::new(false));
    let control = FakeControl::new(cluster.clone(), shard_urls.clone())
        .with_pods_report_map_version(pods_report.clone());

    // Drive Complete -> PrepareSplit -> Splitting -> CatchingUp -> Complete.
    for _ in 0..4 {
        let lumen = control.snapshot();
        drive_tick(&control, &http, &lumen).await;
    }
    assert_eq!(
        control.snapshot().spec.reshard_policy.workflow.phase,
        ReshardPhase::Complete
    );
    assert_eq!(control.snapshot().spec.shard_map.version, 1);

    let moved_id = moving[0];
    let probe = || async {
        shard0
            .server
            .post("/collections/u/index")
            .json(&json!({
                "items": [{
                    "external_id": moved_id,
                    "field": "email",
                    "value": "late@x.com",
                }]
            }))
            .await
    };

    // Rollout is converged but pods aren't reporting the new map yet: must
    // stay AwaitingTopologyConvergence, fence stays armed, and the pod-report
    // scrape is actually being invoked (rollout-converged gate lets it
    // through) even though it reports false.
    let lumen = control.snapshot();
    let outcome = drive_tick(&control, &http, &lumen).await;
    assert_eq!(
        outcome,
        DriveOutcome::AwaitingTopologyConvergence { map_version: 1 }
    );
    assert_eq!(
        probe().await.status_code(),
        503,
        "fence must stay armed while pods have not confirmed the new map version"
    );
    assert!(
        control.pods_report_map_version_calls.load(Ordering::SeqCst) >= 1,
        "the pod-level scrape must actually run once the StatefulSet-level rollout is \
         already converged"
    );
    assert_eq!(
        control
            .snapshot()
            .spec
            .reshard_policy
            .workflow
            .converged_shard_map_version,
        None,
        "rollout convergence alone must not persist convergedShardMapVersion"
    );

    // Pods catch up: the very next tick must clear the fence and persist
    // convergedShardMapVersion, exactly like the pre-#1467 StatefulSet-only
    // check did once its own signal flipped true.
    pods_report.store(true, Ordering::SeqCst);
    let lumen = control.snapshot();
    let outcome = drive_tick(&control, &http, &lumen).await;
    assert_eq!(outcome, DriveOutcome::TopologyConverged { map_version: 1 });
    assert_eq!(
        control
            .snapshot()
            .spec
            .reshard_policy
            .workflow
            .converged_shard_map_version,
        Some(1)
    );
    assert_eq!(
        probe().await.status_code(),
        200,
        "fence must clear once every pod is confirmed on the new map version"
    );
}

/// #1467 R7/AC7: a `shardMap.version` the driver never itself cut over to
/// (a manually-authored/edited value, or one left over from a workflow the
/// driver never actually drove — `lastCutoverShardMapVersion` absent or
/// stale) must never engage the convergence-await loop at all — no
/// `AwaitingTopologyConvergence`, no fence re-arm, no
/// `serving_topology_converged`/`serving_pods_report_map_version` calls —
/// since nothing durable in the workflow's own history claims responsibility
/// for having moved data to that version.
#[tokio::test]
async fn convergence_never_engages_for_a_shard_map_version_the_driver_never_cut_over_to() {
    let mut lumen = initial_lumen(None, None);
    // Simulate a manually-edited/foreign shardMap.version: bumped directly,
    // with no matching lastCutoverShardMapVersion ever stamped (the driver's
    // own cutover patch always sets both fields together in the same call —
    // see `advance_catching_up_fenced`).
    lumen.spec.shard_map.version = 5;
    let cluster = Arc::new(Mutex::new(lumen));
    let control = FakeControl::new(cluster.clone(), vec!["http://unused.invalid".to_string()]);
    let http = reqwest::Client::new();

    for _ in 0..3 {
        let lumen = control.snapshot();
        let outcome = drive_tick(&control, &http, &lumen).await;
        assert!(
            matches!(outcome, DriveOutcome::NoOp(_)),
            "a shardMap.version with no matching lastCutoverShardMapVersion must never \
             engage convergence, got {outcome:?}"
        );
    }
    assert_eq!(
        control.convergence_check_calls.load(Ordering::SeqCst),
        0,
        "serving_topology_converged must never be called for an uncut-over map version"
    );
    assert_eq!(
        control.pods_report_map_version_calls.load(Ordering::SeqCst),
        0,
        "serving_pods_report_map_version must never be called for an uncut-over map version"
    );
    assert_eq!(
        control
            .snapshot()
            .spec
            .reshard_policy
            .workflow
            .converged_shard_map_version,
        None
    );
}

/// #1467 R7/AC7: bounded escalation, driver half — a topology convergence
/// wait that runs for far longer than `CONVERGENCE_STALL_TICKS` (30, private
/// to `reshard_driver`; this test drives 35 ticks) must never silently drop
/// the write-pause fence or stop re-checking — every tick keeps reporting
/// `AwaitingTopologyConvergence` and keeps writes to the moved bucket
/// rejected, no matter how long the wait runs. The status-side half (a
/// distinct `topologyConvergenceStalled` condition once the budget is
/// exceeded) is covered by `reconcile.rs`'s own
/// `status_patch_surfaces_topology_convergence_stall_as_distinct_condition`,
/// which reads the same driver-side stall cache (`record_convergence_await`)
/// this loop updates every tick.
#[tokio::test]
async fn convergence_stall_never_drops_the_fence_across_an_extended_wait() {
    let shard0 = spin_up_shard();
    let shard1 = spin_up_shard();
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
    // Never converges: StatefulSet rollout itself never finishes.
    let converged = Arc::new(AtomicBool::new(false));
    let control =
        FakeControl::new(cluster.clone(), shard_urls.clone()).with_topology_converged(converged);

    // Drive Complete -> PrepareSplit -> Splitting -> CatchingUp -> Complete.
    for _ in 0..4 {
        let lumen = control.snapshot();
        drive_tick(&control, &http, &lumen).await;
    }
    assert_eq!(control.snapshot().spec.shard_map.version, 1);

    let moved_id = moving[0];
    let probe = || async {
        shard0
            .server
            .post("/collections/u/index")
            .json(&json!({
                "items": [{
                    "external_id": moved_id,
                    "field": "email",
                    "value": "late@x.com",
                }]
            }))
            .await
    };

    // Drive well past CONVERGENCE_STALL_TICKS (30) — every tick must keep
    // reporting AwaitingTopologyConvergence and keep the fence armed no
    // matter how long the wait runs; the driver never gives up and never
    // silently drops the fence just because a budget was exceeded.
    for _ in 0..35 {
        let lumen = control.snapshot();
        let outcome = drive_tick(&control, &http, &lumen).await;
        assert_eq!(
            outcome,
            DriveOutcome::AwaitingTopologyConvergence { map_version: 1 }
        );
        assert_eq!(
            probe().await.status_code(),
            503,
            "fence must stay armed through the entire stalled wait, well past the stall \
             escalation budget"
        );
    }
}
// CODEGEN-END
