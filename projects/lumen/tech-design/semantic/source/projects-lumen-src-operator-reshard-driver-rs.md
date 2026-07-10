---
id: projects-lumen-src-operator-reshard-driver-rs
capability_refs:
  - id: "dynamic-shard-topology"
    role: primary
    gap: "storage-pressure-operator-split-policy"
    claim: "storage-pressure-operator-split-policy"
    coverage: full
    rationale: "This source unit is the autonomous phase driver (#1319 R2 executor) that turns a reported crossed-threshold reshard policy into a real, checkpointed, resumable topology change end to end (#1381)."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/operator/reshard_driver.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/operator/reshard_driver.rs`.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DriveOutcome` | projects/lumen/src/operator/reshard_driver.rs | enum | pub | 290 |  |
| `KubeClusterControl` | projects/lumen/src/operator/reshard_driver.rs | struct | pub | 172 |  |
| `compute_target_map` | projects/lumen/src/operator/reshard_driver.rs | function | pub | 379 | compute_target_map(current: &VirtualBucketShardMap) -> Result<VirtualBucketShardMap> |
| `current_shard_map` | projects/lumen/src/operator/reshard_driver.rs | function | pub | 364 | current_shard_map(lumen: &Lumen) -> Result<VirtualBucketShardMap> |
| `drive_tick` | projects/lumen/src/operator/reshard_driver.rs | function | pub | 699 | drive_tick(     control: &dyn ClusterControl,     http: &reqwest::Client,     lumen: &Lumen, ) -> DriveOutcome |
| `new` | projects/lumen/src/operator/reshard_driver.rs | function | pub | 178 | new(client: Client) -> Self |
| `run_migration_pass` | projects/lumen/src/operator/reshard_driver.rs | function | pub | 478 | run_migration_pass(     control: &dyn ClusterControl,     http: &reqwest::Client,     namespace: &str,     name: &str,     lumen: &Lumen, ) -> Result<usize> |
| `should_start_split` | projects/lumen/src/operator/reshard_driver.rs | function | pub | 318 | should_start_split(lumen: &Lumen) -> bool |
| `spawn_reshard_driver_loop` | projects/lumen/src/operator/reshard_driver.rs | function | pub | 736 | spawn_reshard_driver_loop(client: Client) |

Not listed above (matching this project's existing mirrors' convention of
only capturing top-level `pub` structs/enums/consts/modules and inherent-impl
`pub fn`s, never trait definitions, trait methods, or trait-impl methods):
the `pub trait ClusterControl` definition and its five methods (including
`shard_base_url`), and `KubeClusterControl`'s `impl ClusterControl for
KubeClusterControl` block.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Autonomous reshard phase driver (#1319 R2 executor; #1381).
//!
//! [`super::reconcile`]'s live per-shard usage loop only *reports* a crossed
//! `prepareAtPercent` / `urgentAtPercent` threshold into
//! `status.reshard.blockingConditions`; this module is the piece that acts on
//! it — a second, independently leader-gated background loop that drives
//! `spec.reshardPolicy.workflow.phase` through
//! `PrepareSplit -> Splitting -> CatchingUp -> Complete`, growing storage by
//! exactly one physical shard per split via [`crate::routing::
//! VirtualBucketShardMap::split_one_shard`] and moving data with the already
//!-landed admin verbs (`POST /admin/backup:scoped`, `POST
//! /admin/reshard:apply`, `POST /admin/reshard:evict`, #1380).
//!
//! ## Checkpointing and resume
//!
//! There is no separate checkpoint record. Every phase's actions are
//! recomputed deterministically from three already-persisted **spec** fields
//! — `reshardPolicy.workflow.phase`, `reshardPolicy.workflow.
//! targetShardCount`, and `shardMap` (left untouched until the cutover) —
//! plus `shardCount`. `spec` (not `status`) is the checkpoint because it is
//! the operator's own desired-state write target and survives an operator
//! restart or leader handover unchanged; `status` stays a read-only
//! projection. Concretely:
//!
//! - **Complete** (idle): [`should_start_split`] gates on a crossed
//!   threshold (`status.reshard.blockingConditions` already carries
//!   `prepareThresholdCrossed` / `urgentThresholdCrossed`, computed by
//!   [`super::crd::LumenSpec::reshard_status_with_usage`]), `maxShardBytes`
//!   set (R3 safety rail — recommendation-only otherwise), single-member
//!   (`replicasPerShard <= 1`; see below), and no `maxShards` ceiling
//!   reached. On a match: compute the target map, patch `shardCount` and
//!   `workflow.{phase,targetShardCount}` in one merge patch, phase ->
//!   `PrepareSplit`.
//! - **PrepareSplit**: wait for the StatefulSet's `readyReplicas` to reach
//!   `targetShardCount` (the new pod exists once `shardCount` is bumped, via
//!   the *existing*, independently-leader-gated `libs/operator` apply loop —
//!   this driver never applies child objects itself). Once ready, phase ->
//!   `Splitting`. Restart-safe: re-reads the same live readiness fact every
//!   tick.
//! - **Splitting**: run one migration pass ([`run_migration_pass`] — the
//!   production caller of [`crate::reshard::bucket_moves`] /
//!   [`crate::reshard::snapshot_reshard_batches`]) copying every moved
//!   bucket from its old shard to the new shard via the admin verbs, then
//!   phase -> `CatchingUp`. `POST /admin/reshard:apply` is an idempotent
//!   additive merge (#1380), so re-running this same pass after a restart
//!   (still `Splitting`) is safe and simply re-applies the same batches.
//! - **CatchingUp**: run the *same* migration pass again — an idempotent
//!   re-sync that closes the gap for documents written to a moved bucket's
//!   old shard during the `Splitting` window (writes still land on the old
//!   shard until the map itself flips) — then evict every moved bucket from
//!   every old shard ([`crate::reshard`]'s `evict` is also idempotent) and
//!   flip `spec.shardMap` to the target map in the same patch that clears
//!   `workflow.targetShardCount` and resets phase -> `Complete`. Calling
//!   evict against the **new**, already-committed map (not the stale old
//!   one) means the driver never needs to retain the old map across a
//!   restart — the source of the classic "lost the old map after cutover"
//!   resumability trap.
//!
//! A driver-side error at any step ([`DriveOutcome::Blocked`]) leaves the CR
//! spec untouched; the next tick retries the same phase from the same
//! persisted fields (R3).
//!
//! ## Scope rail: single-member only
//!
//! [`should_start_split`] refuses to start a split when `replicasPerShard >
//! 1`. Growing `shardCount` reassigns `shard_index = ordinal % shardCount`
//! for *every* existing pod ordinal once raft has more than one replica per
//! shard (a full raft-group reshuffle, not an added shard), which is unsafe
//! without additional raft-membership migration this WI does not implement.
//! At `replicasPerShard <= 1`, `ordinal % (shardCount+1) == ordinal` for
//! every existing ordinal (`ordinal < shardCount`), so growing by exactly
//! one is pod-ordinal-stable: every existing pod keeps its shard/PVC
//! identity and exactly one new pod (ordinal == old `shardCount`) becomes
//! the new shard — see [`super::crd::LumenSpec::storage_pod_count`].
//!
//! ## Live query routing consumes `spec.shardMap` (#1384)
//!
//! [`super::render::render`] writes `shardMap.{version,assignments}` into the
//! serving ConfigMap, `serving_env` maps `SHARD_MAP_VERSION`/
//! `VIRTUAL_BUCKET_COUNT`/`SHARD_MAP_ASSIGNMENTS` onto container env, and
//! `src/bin/lumen.rs`'s `serve()` builds its `EngineShardSearch` via
//! `EngineShardSearch::new_with_shard_map` fed by
//! `crate::config::shard_map_from_env`, so a pod started after this driver's
//! cutover routes queries by the minimal-move target map computed by
//! [`crate::routing::VirtualBucketShardMap::split_one_shard`] rather than the
//! balanced default. [`trigger_rolling_restart`] is driven in the same
//! cutover tick that patches `spec.shardMap` so every serving pod picks up
//! the new map without manual intervention: it patches the serving
//! StatefulSet's pod-template annotations, which Kubernetes' native
//! `RollingUpdate` strategy (`serving_statefulset`'s `updateStrategy`) turns
//! into a rolling recreation of every pod against the already-updated
//! ConfigMap — no separate watch/poll loop is needed.

use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use kube::api::{Api, ApiResource, DynamicObject, Patch, PatchParams};
use kube::{Client, ResourceExt};
use serde_json::json;

use crate::auth::{Role, TokenClaims};
use crate::operator::crd::{AuthMode, Lumen, ReshardPhase};
use crate::operator::lease::{self, Election};
use crate::reshard::{bucket_moves, snapshot_reshard_batches, ReshardBatch};
use crate::routing::VirtualBucketShardMap;
use crate::storage::SnapshotV1;

/// The client-facing port lumen's serving Service/StatefulSet expose. Kept
/// duplicated from `render::CLIENT_PORT`/`reconcile::CLIENT_PORT` the same
/// way those two already duplicate it from each other — the smallest private
/// constant beats a new `pub` cross-module symbol-table row.
const CLIENT_PORT: u16 = 7373;

/// Poll interval for the reshard driver loop.
const DRIVER_POLL_INTERVAL: Duration = Duration::from_secs(20);

/// Leader-election Lease name for [`spawn_reshard_driver_loop`] — distinct
/// from `libs/operator`'s own `S::MANAGER`-named apply-loop Lease so the two
/// independently-leader-gated loops (which may pick different leaders) never
/// contend on one Lease object.
const DRIVER_LEASE_NAME: &str = "lumen-reshard-driver";

/// Upper bound on external_ids carried per `POST /admin/reshard:apply` call,
/// matching the batching contract [`crate::reshard::snapshot_reshard_batches`]
/// already documents (checkpoint after every batch, not after one full-shard
/// copy).
const MAX_EXTERNAL_IDS_PER_BATCH: usize = 2000;

/// Everything [`drive_tick`] needs from a live cluster, abstracted so the
/// state machine is testable without a real k8s API server. [`KubeClusterControl`]
/// is the production implementation; tests supply an in-memory fake.
#[async_trait]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
pub trait ClusterControl: Send + Sync {
    /// JSON-merge-patch this `Lumen`'s `.spec` (see `Patch::Merge` semantics:
    /// nested objects merge recursively, a `null` leaf deletes that key,
    /// sibling fields not mentioned are untouched).
    async fn patch_spec(&self, namespace: &str, name: &str, patch: serde_json::Value)
        -> Result<()>;

    /// The serving StatefulSet's `.status.readyReplicas` (0 if absent/not
    /// found yet).
    async fn statefulset_ready_replicas(&self, namespace: &str, name: &str) -> Result<i64>;

    /// Bump a `kubectl rollout restart`-style pod-template annotation so a
    /// shard-map-only ConfigMap change gets picked up by a fresh generation
    /// of pods (see the module-level "known gap" note: a no-op today until
    /// serving actually reads that ConfigMap data, but still the correct
    /// operator action to take at cutover).
    async fn trigger_rolling_restart(&self, namespace: &str, name: &str) -> Result<()>;

    /// A bearer token carrying wildcard `Role::Admin`, if `lumen.spec.auth`
    /// requires one. `Ok(None)` when auth is off.
    async fn admin_token(&self, namespace: &str, lumen: &Lumen) -> Result<Option<String>>;

    /// The client-facing admin API base URL for one shard's serving pod.
    /// [`KubeClusterControl`] resolves the real per-shard headless-Service
    /// DNS name (matching [`super::reconcile::pod_metrics_urls`]'s
    /// convention); an integration-test fake resolves to whatever real local
    /// address that shard's `TestServer` is actually bound to — the seam
    /// that lets [`run_migration_pass`] / [`evict_old_shards`] run against
    /// real HTTP servers + real `Engine`s without a live cluster.
    fn shard_base_url(&self, namespace: &str, name: &str, shard: u32) -> String;
}

/// Production [`ClusterControl`]: real `kube::Client` calls.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
pub struct KubeClusterControl {
    client: Client,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
impl KubeClusterControl {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

fn statefulset_api_resource() -> ApiResource {
    ApiResource {
        group: "apps".to_string(),
        version: "v1".to_string(),
        api_version: "apps/v1".to_string(),
        kind: "StatefulSet".to_string(),
        plural: "statefulsets".to_string(),
    }
}

#[async_trait]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
impl ClusterControl for KubeClusterControl {
    async fn patch_spec(
        &self,
        namespace: &str,
        name: &str,
        patch: serde_json::Value,
    ) -> Result<()> {
        let api: Api<Lumen> = Api::namespaced(self.client.clone(), namespace);
        api.patch(name, &PatchParams::default(), &Patch::Merge(&patch))
            .await
            .context("patch Lumen spec")?;
        Ok(())
    }

    async fn statefulset_ready_replicas(&self, namespace: &str, name: &str) -> Result<i64> {
        let ar = statefulset_api_resource();
        let api: Api<DynamicObject> = Api::namespaced_with(self.client.clone(), namespace, &ar);
        let ready = api
            .get_opt(name)
            .await
            .context("read serving StatefulSet")?
            .and_then(|o| o.data["status"]["readyReplicas"].as_i64())
            .unwrap_or(0);
        Ok(ready)
    }

    async fn trigger_rolling_restart(&self, namespace: &str, name: &str) -> Result<()> {
        let ar = statefulset_api_resource();
        let api: Api<DynamicObject> = Api::namespaced_with(self.client.clone(), namespace, &ar);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let patch = json!({
            "spec": {
                "template": {
                    "metadata": {
                        "annotations": {
                            "lumen.dev/reshard-restarted-at": now.to_string(),
                        }
                    }
                }
            }
        });
        api.patch(name, &PatchParams::default(), &Patch::Merge(&patch))
            .await
            .context("trigger serving StatefulSet rolling restart")?;
        Ok(())
    }

    async fn admin_token(&self, namespace: &str, lumen: &Lumen) -> Result<Option<String>> {
        if !matches!(lumen.spec.auth, AuthMode::Required) {
            return Ok(None);
        }
        let Some(secret_name) = lumen.spec.tokens_secret.as_deref() else {
            // CSI-only (`tokensSecretProviderClass`) deployments have no
            // Secret object for the driver to read here — a documented,
            // not-yet-closed gap (#1381): every admin call this tick fails
            // closed (401), which `run_migration_pass`/`evict_old_shards`
            // surface as `DriveOutcome::Blocked`, leaving the workflow
            // resumable rather than silently stuck.
            bail!(
                "tokensSecretProviderClass-only auth is not supported by the reshard driver yet; \
                 set spec.tokensSecret so the driver can resolve an admin-role bearer token"
            );
        };
        let api: kube::Api<k8s_openapi::api::core::v1::Secret> =
            kube::Api::namespaced(self.client.clone(), namespace);
        let secret = api.get(secret_name).await.context("read tokens secret")?;
        let bytes = secret
            .data
            .as_ref()
            .and_then(|d| d.get("token-registry.json"))
            .ok_or_else(|| {
                anyhow!("tokens secret `{secret_name}` missing token-registry.json key")
            })?;
        let registry: BTreeMap<String, TokenClaims> =
            serde_json::from_slice(&bytes.0).context("parse token-registry.json")?;
        let token = registry
            .into_iter()
            .find(|(_, claims)| claims.roles.get("*") == Some(&Role::Admin))
            .map(|(token, _)| token);
        Ok(token)
    }

    fn shard_base_url(&self, namespace: &str, name: &str, shard: u32) -> String {
        format!("http://{name}-{shard}.{name}-headless.{namespace}.svc.cluster.local:{CLIENT_PORT}")
    }
}

/// What one [`drive_tick`] call did, for logging/tests. Never panics; a
/// failed step reports [`DriveOutcome::Blocked`] and leaves the CR spec
/// exactly as it was, so the next tick retries from the same persisted phase.
#[derive(Debug, Clone, PartialEq)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
pub enum DriveOutcome {
    /// Nothing to do this tick (`Complete` with no crossed threshold, an
    /// unsupported topology, or a `maxShards` ceiling reached).
    NoOp(&'static str),
    /// `Complete -> PrepareSplit`: `shardCount`/`targetShardCount` patched.
    StartedSplit { target_shard_count: u32 },
    /// Still `PrepareSplit`: the new pod is not `Ready` yet.
    WaitingForNewShard { target_shard_count: u32 },
    /// `PrepareSplit -> Splitting`: the new pod is `Ready`.
    AdvancedToSplitting,
    /// Still `Splitting`: one migration pass ran (batch count included; `0`
    /// only if there is nothing to move, which should not happen for a
    /// freshly started split).
    MigratedBatches { batches: usize },
    /// `Splitting -> CatchingUp`.
    AdvancedToCatchingUp,
    /// `CatchingUp -> Complete`: re-sync pass ran, old shards evicted, and
    /// `shardMap` flipped to the new version.
    CompletedSplit { new_map_version: u64 },
    /// A step failed; phase unchanged, safe to retry next tick.
    Blocked(String),
}

/// Pure trigger gate (R3 safety rail; AC4): whether `lumen` should start a
/// **new** split this tick. `false` whenever `maxShardBytes` is unset —
/// recommendation-only mode never auto-splits, regardless of any other
/// field, including a stale/manually-forced `status.reshard`.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
pub fn should_start_split(lumen: &Lumen) -> bool {
    if lumen.spec.reshard_policy.max_shard_bytes.is_none() {
        return false;
    }
    if lumen.spec.replicas_per_shard > 1 {
        // Raft-HA: growing shardCount reshuffles ordinal->shard for every
        // existing pod, not just an added one. See the module doc's "Scope
        // rail" note.
        return false;
    }
    if !matches!(
        lumen.spec.reshard_policy.workflow.phase,
        ReshardPhase::Complete
    ) {
        // Already mid-workflow: PrepareSplit/Splitting/CatchingUp resume via
        // drive_tick's other branches, never restart from should_start_split.
        return false;
    }
    if let Some(max) = lumen.spec.reshard_policy.max_shards {
        if lumen.spec.shard_count >= max {
            return false;
        }
    }
    let Some(status) = lumen.status.as_ref() else {
        return false;
    };
    status
        .reshard
        .blocking_conditions
        .iter()
        .any(|c| c == "prepareThresholdCrossed" || c == "urgentThresholdCrossed")
}

/// The virtual-bucket map `lumen.spec.shardMap` currently describes — the
/// map still live for routing/data placement right now, as opposed to
/// `spec.shardCount`'s StatefulSet-sizing intent.
///
/// While a split is in flight (`workflow.targetShardCount` is set),
/// `start_split` has already bumped `spec.shardCount` to the target so the
/// new StatefulSet replica can come up, but the actual live topology —
/// what `bucket_moves`/`snapshot_reshard_batches` must diff against, and
/// what eviction must iterate — is still the pre-split shard count until
/// the `Complete`-phase cutover commits `shardMap`. This driver only ever
/// grows a map by exactly one shard per split (R1), so the pre-split count
/// is always `targetShardCount - 1`.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
pub fn current_shard_map(lumen: &Lumen) -> Result<VirtualBucketShardMap> {
    let sm = &lumen.spec.shard_map;
    let physical = match lumen.spec.reshard_policy.workflow.target_shard_count {
        Some(target) => target.saturating_sub(1).max(1),
        None => lumen.spec.shard_count.max(1),
    };
    if sm.assignments.is_empty() {
        VirtualBucketShardMap::balanced(sm.version, sm.virtual_bucket_count, physical)
    } else {
        VirtualBucketShardMap::new(sm.version, sm.assignments.clone(), physical)
    }
}

/// The target map for growing `current` by exactly one shard (R1).
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
pub fn compute_target_map(current: &VirtualBucketShardMap) -> Result<VirtualBucketShardMap> {
    current.split_one_shard(current.version() + 1)
}

async fn fetch_scoped_backup(
    http: &reqwest::Client,
    base_url: &str,
    token: Option<&str>,
    virtual_bucket_count: u32,
    buckets: &BTreeSet<u32>,
) -> Result<SnapshotV1> {
    let mut req = http
        .post(format!("{base_url}/admin/backup:scoped"))
        .json(&json!({
            "virtual_bucket_count": virtual_bucket_count,
            "buckets": buckets,
        }));
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let resp = req
        .send()
        .await
        .with_context(|| format!("POST {base_url}/admin/backup:scoped"))?;
    if !resp.status().is_success() {
        bail!("{base_url}/admin/backup:scoped returned {}", resp.status());
    }
    resp.json::<SnapshotV1>()
        .await
        .context("decode backup:scoped response")
}

async fn apply_reshard_batch(
    http: &reqwest::Client,
    base_url: &str,
    token: Option<&str>,
    batch: &ReshardBatch,
) -> Result<()> {
    let mut req = http
        .post(format!("{base_url}/admin/reshard:apply"))
        .json(batch);
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let resp = req
        .send()
        .await
        .with_context(|| format!("POST {base_url}/admin/reshard:apply"))?;
    if !resp.status().is_success() {
        bail!("{base_url}/admin/reshard:apply returned {}", resp.status());
    }
    Ok(())
}

async fn evict_shard(
    http: &reqwest::Client,
    base_url: &str,
    token: Option<&str>,
    shard: u32,
    map_version: u64,
    assignments: &[u32],
    physical_shard_count: u32,
) -> Result<()> {
    let mut req = http
        .post(format!("{base_url}/admin/reshard:evict"))
        .json(&json!({
            "shard": shard,
            "map_version": map_version,
            "assignments": assignments,
            "physical_shard_count": physical_shard_count,
        }));
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let resp = req
        .send()
        .await
        .with_context(|| format!("POST {base_url}/admin/reshard:evict"))?;
    if !resp.status().is_success() {
        bail!("{base_url}/admin/reshard:evict returned {}", resp.status());
    }
    Ok(())
}

fn map_assignments(map: &VirtualBucketShardMap) -> Vec<u32> {
    (0..map.virtual_bucket_count())
        .map(|bucket| map.assignment_for_bucket(bucket).unwrap_or(0))
        .collect()
}

/// One migration pass: every bucket [`bucket_moves`] says moved between
/// `current_shard_map` and [`compute_target_map`], grouped by its old
/// (`from_shard`) owner, fetched via `POST /admin/backup:scoped` and applied
/// to its new owner via `POST /admin/reshard:apply`
/// ([`snapshot_reshard_batches`] builds the bounded batches). Real,
/// non-test caller of both — AC3. Idempotent: re-running against unchanged
/// data re-applies the same batches, which `POST /admin/reshard:apply`
/// already treats as a no-op (#1380).
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
pub async fn run_migration_pass(
    control: &dyn ClusterControl,
    http: &reqwest::Client,
    namespace: &str,
    name: &str,
    lumen: &Lumen,
) -> Result<usize> {
    let current = current_shard_map(lumen)?;
    let target = compute_target_map(&current)?;
    let moves = bucket_moves(&current, &target)?;
    if moves.is_empty() {
        return Ok(0);
    }

    let mut buckets_by_from_shard: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    for mv in &moves {
        buckets_by_from_shard
            .entry(mv.from_shard)
            .or_default()
            .insert(mv.bucket);
    }

    let token = control.admin_token(namespace, lumen).await?;
    let mut total_batches = 0usize;
    for (from_shard, buckets) in buckets_by_from_shard {
        let source_url = control.shard_base_url(namespace, name, from_shard);
        let snapshot = fetch_scoped_backup(
            http,
            &source_url,
            token.as_deref(),
            current.virtual_bucket_count(),
            &buckets,
        )
        .await?;
        let batches =
            snapshot_reshard_batches(&snapshot, &current, &target, MAX_EXTERNAL_IDS_PER_BATCH)?;
        for batch in &batches {
            let dest_url = control.shard_base_url(namespace, name, batch.to_shard);
            apply_reshard_batch(http, &dest_url, token.as_deref(), batch).await?;
        }
        total_batches += batches.len();
    }
    Ok(total_batches)
}

/// Post-cutover eviction (idempotent, #1380) on every **old** shard, using
/// only the already-committed target map — the driver never needs to retain
/// the old map across a restart.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
async fn evict_old_shards(
    control: &dyn ClusterControl,
    http: &reqwest::Client,
    namespace: &str,
    name: &str,
    lumen: &Lumen,
    current: &VirtualBucketShardMap,
    target: &VirtualBucketShardMap,
) -> Result<()> {
    let token = control.admin_token(namespace, lumen).await?;
    let assignments = map_assignments(target);
    for shard in 0..current.physical_shard_count() {
        let url = control.shard_base_url(namespace, name, shard);
        evict_shard(
            http,
            &url,
            token.as_deref(),
            shard,
            target.version(),
            &assignments,
            target.physical_shard_count(),
        )
        .await?;
    }
    Ok(())
}

async fn start_split(
    control: &dyn ClusterControl,
    namespace: &str,
    name: &str,
    lumen: &Lumen,
) -> DriveOutcome {
    let current = match current_shard_map(lumen) {
        Ok(m) => m,
        Err(err) => return DriveOutcome::Blocked(err.to_string()),
    };
    let target = match compute_target_map(&current) {
        Ok(m) => m,
        Err(err) => return DriveOutcome::Blocked(err.to_string()),
    };
    let target_shard_count = target.physical_shard_count();
    let patch = json!({
        "spec": {
            "shardCount": target_shard_count,
            "reshardPolicy": {
                "workflow": {
                    "phase": "PrepareSplit",
                    "targetShardCount": target_shard_count,
                }
            }
        }
    });
    match control.patch_spec(namespace, name, patch).await {
        Ok(()) => DriveOutcome::StartedSplit { target_shard_count },
        Err(err) => DriveOutcome::Blocked(err.to_string()),
    }
}

async fn advance_prepare_split(
    control: &dyn ClusterControl,
    namespace: &str,
    name: &str,
    lumen: &Lumen,
) -> DriveOutcome {
    let Some(target_shard_count) = lumen.spec.reshard_policy.workflow.target_shard_count else {
        return DriveOutcome::Blocked("PrepareSplit with no targetShardCount set".to_string());
    };
    let ready = match control.statefulset_ready_replicas(namespace, name).await {
        Ok(r) => r,
        Err(err) => return DriveOutcome::Blocked(err.to_string()),
    };
    if ready < i64::from(target_shard_count) {
        return DriveOutcome::WaitingForNewShard { target_shard_count };
    }
    let patch = json!({
        "spec": { "reshardPolicy": { "workflow": { "phase": "Splitting" } } }
    });
    match control.patch_spec(namespace, name, patch).await {
        Ok(()) => DriveOutcome::AdvancedToSplitting,
        Err(err) => DriveOutcome::Blocked(err.to_string()),
    }
}

async fn advance_splitting(
    control: &dyn ClusterControl,
    http: &reqwest::Client,
    namespace: &str,
    name: &str,
    lumen: &Lumen,
) -> DriveOutcome {
    let batches = match run_migration_pass(control, http, namespace, name, lumen).await {
        Ok(n) => n,
        Err(err) => return DriveOutcome::Blocked(err.to_string()),
    };
    let patch = json!({
        "spec": { "reshardPolicy": { "workflow": { "phase": "CatchingUp" } } }
    });
    match control.patch_spec(namespace, name, patch).await {
        Ok(()) => {
            if batches == 0 {
                // Nothing moved on this pass (already caught up from a prior
                // attempt); still safe to advance.
                DriveOutcome::AdvancedToCatchingUp
            } else {
                DriveOutcome::MigratedBatches { batches }
            }
        }
        Err(err) => DriveOutcome::Blocked(err.to_string()),
    }
}

async fn advance_catching_up(
    control: &dyn ClusterControl,
    http: &reqwest::Client,
    namespace: &str,
    name: &str,
    lumen: &Lumen,
) -> DriveOutcome {
    if let Err(err) = run_migration_pass(control, http, namespace, name, lumen).await {
        return DriveOutcome::Blocked(err.to_string());
    }

    let current = match current_shard_map(lumen) {
        Ok(m) => m,
        Err(err) => return DriveOutcome::Blocked(err.to_string()),
    };
    let target = match compute_target_map(&current) {
        Ok(m) => m,
        Err(err) => return DriveOutcome::Blocked(err.to_string()),
    };

    if let Err(err) =
        evict_old_shards(control, http, namespace, name, lumen, &current, &target).await
    {
        return DriveOutcome::Blocked(err.to_string());
    }

    let patch = json!({
        "spec": {
            "shardMap": {
                "version": target.version(),
                "virtualBucketCount": target.virtual_bucket_count(),
                "assignments": map_assignments(&target),
            },
            "reshardPolicy": {
                "workflow": {
                    "phase": "Complete",
                    "targetShardCount": null,
                }
            }
        }
    });
    if let Err(err) = control.patch_spec(namespace, name, patch).await {
        return DriveOutcome::Blocked(err.to_string());
    }
    if let Err(err) = control.trigger_rolling_restart(namespace, name).await {
        // Non-fatal: the map has already flipped; a failed restart trigger
        // only delays picking up the new ConfigMap once consumption exists
        // (see the module doc's "known gap"), it does not corrupt data.
        tracing::warn!(error = %err, "reshard driver: cutover rolling-restart trigger failed");
    }
    DriveOutcome::CompletedSplit {
        new_map_version: target.version(),
    }
}

/// One phase-driver tick for `lumen`: dispatches on `spec.reshardPolicy.
/// workflow.phase` and performs at most one state transition's worth of
/// work. Safe to call every [`DRIVER_POLL_INTERVAL`] forever — a `Complete`
/// CR with nothing to do returns [`DriveOutcome::NoOp`] immediately.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
pub async fn drive_tick(
    control: &dyn ClusterControl,
    http: &reqwest::Client,
    lumen: &Lumen,
) -> DriveOutcome {
    let Some(namespace) = lumen.namespace() else {
        return DriveOutcome::Blocked("Lumen object missing metadata.namespace".to_string());
    };
    let name = lumen.name_any();

    match lumen.spec.reshard_policy.workflow.phase {
        ReshardPhase::Complete => {
            if should_start_split(lumen) {
                start_split(control, &namespace, &name, lumen).await
            } else {
                DriveOutcome::NoOp(
                    "no crossed threshold, unsupported topology, or maxShards reached",
                )
            }
        }
        ReshardPhase::PrepareSplit => {
            advance_prepare_split(control, &namespace, &name, lumen).await
        }
        ReshardPhase::Splitting => advance_splitting(control, http, &namespace, &name, lumen).await,
        ReshardPhase::CatchingUp => {
            advance_catching_up(control, http, &namespace, &name, lumen).await
        }
    }
}

/// Background loop: every [`DRIVER_POLL_INTERVAL`], list every `Lumen` CR
/// cluster-wide and [`drive_tick`] it. Independently leader-gated (its own
/// [`DRIVER_LEASE_NAME`] Lease) from the shared `libs/operator` apply loop —
/// either loop's leader may or may not be this replica, and both are safe to
/// run concurrently since every driver action is an idempotent-or-checkpointed
/// spec patch / additive data-plane call.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
pub fn spawn_reshard_driver_loop(client: Client) {
    // Mirrors `libs/operator::controller`'s own `identity`/`lease_namespace`
    // helpers (private to that crate, so duplicated here) so both
    // independently-leader-gated loops resolve the same pod identity and
    // Lease namespace from the same env vars.
    let identity = std::env::var("POD_NAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| DRIVER_LEASE_NAME.to_string());
    let namespace =
        std::env::var("POD_NAMESPACE").unwrap_or_else(|_| "lumen-operator-system".to_string());
    let election = Election::new(identity);
    lease::spawn(
        client.clone(),
        namespace,
        DRIVER_LEASE_NAME.to_string(),
        election.clone(),
    );
    let control = KubeClusterControl::new(client.clone());
    tokio::spawn(async move {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        let api: kube::Api<Lumen> = kube::Api::all(client);
        loop {
            if election
                .is_leader
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                match api.list(&Default::default()).await {
                    Ok(list) => {
                        for lumen in list.items {
                            let outcome = drive_tick(&control, &http, &lumen).await;
                            if !matches!(outcome, DriveOutcome::NoOp(_)) {
                                tracing::info!(
                                    lumen = lumen.name_any(),
                                    namespace = lumen.namespace(),
                                    ?outcome,
                                    "reshard driver tick"
                                );
                            }
                        }
                    }
                    Err(err) => {
                        tracing::warn!(error = %err, "reshard driver: list Lumen failed");
                    }
                }
            }
            tokio::time::sleep(DRIVER_POLL_INTERVAL).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operator::crd::{
        LumenReshardStatus, LumenSpec, LumenStatus, ReshardPolicy, ReshardWorkflowSpec,
        ServingSpec, ShardMapSpec,
    };
    use std::sync::atomic::{AtomicI64, Ordering};
    use std::sync::Mutex;

    fn spec(shard_count: u32, replicas_per_shard: u32, max_shard_bytes: Option<u64>) -> LumenSpec {
        LumenSpec {
            image: "lumen:latest".into(),
            image_pull_policy: None,
            shard_count,
            shard_map: ShardMapSpec {
                version: 0,
                virtual_bucket_count: 8,
                assignments: Vec::new(),
            },
            replicas_per_shard,
            voter_count: replicas_per_shard,
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
        }
    }

    fn lumen_with(spec: LumenSpec, status: Option<LumenStatus>) -> Lumen {
        let mut lumen = Lumen::new("search", spec);
        lumen.metadata.namespace = Some("acme".to_string());
        lumen.status = status;
        lumen
    }

    fn status_with_blocking(condition: &str) -> LumenStatus {
        LumenStatus {
            reshard: LumenReshardStatus {
                blocking_conditions: vec![condition.to_string()],
                ..Default::default()
            },
            ..Default::default()
        }
    }

    // ---- should_start_split (AC4 + R3) -------------------------------

    #[test]
    fn should_start_split_false_when_max_shard_bytes_unset() {
        let lumen = lumen_with(
            spec(1, 1, None),
            Some(status_with_blocking("urgentThresholdCrossed")),
        );
        assert!(!should_start_split(&lumen));
    }

    #[test]
    fn should_start_split_false_without_a_crossed_threshold() {
        let lumen = lumen_with(spec(1, 1, Some(1_000_000)), Some(LumenStatus::default()));
        assert!(!should_start_split(&lumen));
    }

    #[test]
    fn should_start_split_true_on_prepare_threshold_crossed() {
        let lumen = lumen_with(
            spec(1, 1, Some(1_000_000)),
            Some(status_with_blocking("prepareThresholdCrossed")),
        );
        assert!(should_start_split(&lumen));
    }

    #[test]
    fn should_start_split_false_for_raft_ha() {
        let lumen = lumen_with(
            spec(2, 3, Some(1_000_000)),
            Some(status_with_blocking("urgentThresholdCrossed")),
        );
        assert!(!should_start_split(&lumen));
    }

    #[test]
    fn should_start_split_false_when_already_mid_workflow() {
        let mut s = spec(1, 1, Some(1_000_000));
        s.reshard_policy.workflow = ReshardWorkflowSpec {
            phase: ReshardPhase::Splitting,
            target_shard_count: Some(2),
        };
        let lumen = lumen_with(s, Some(status_with_blocking("urgentThresholdCrossed")));
        assert!(!should_start_split(&lumen));
    }

    #[test]
    fn should_start_split_false_when_max_shards_reached() {
        let mut s = spec(4, 1, Some(1_000_000));
        s.reshard_policy.max_shards = Some(4);
        let lumen = lumen_with(s, Some(status_with_blocking("urgentThresholdCrossed")));
        assert!(!should_start_split(&lumen));
    }

    #[test]
    fn should_start_split_false_with_no_status_yet() {
        let lumen = lumen_with(spec(1, 1, Some(1_000_000)), None);
        assert!(!should_start_split(&lumen));
    }

    // ---- current/target map helpers -----------------------------------

    #[test]
    fn current_shard_map_derives_balanced_map_from_shard_count_when_no_explicit_assignments() {
        let lumen = lumen_with(spec(2, 1, None), None);
        let map = current_shard_map(&lumen).unwrap();
        assert_eq!(map.physical_shard_count(), 2);
        assert_eq!(map.virtual_bucket_count(), 8);
    }

    #[test]
    fn compute_target_map_grows_by_exactly_one_shard() {
        let lumen = lumen_with(spec(2, 1, None), None);
        let current = current_shard_map(&lumen).unwrap();
        let target = compute_target_map(&current).unwrap();
        assert_eq!(target.physical_shard_count(), 3);
        assert_eq!(target.version(), current.version() + 1);
    }

    // ---- drive_tick state machine (fake control, no real k8s) ---------

    /// In-memory [`ClusterControl`]: records the last patch applied to a
    /// shared `Lumen` snapshot and simulates a StatefulSet's ready-replica
    /// count. No HTTP admin calls are faked here — those go through a real
    /// [`axum_test`] server in the integration test below; this fake only
    /// covers the k8s-shaped operations `drive_tick` needs before/around
    /// them.
    struct FakeControl {
        ready_replicas: AtomicI64,
        last_patch: Mutex<Option<serde_json::Value>>,
        restart_calls: AtomicI64,
    }

    impl FakeControl {
        fn new(ready_replicas: i64) -> Self {
            Self {
                ready_replicas: AtomicI64::new(ready_replicas),
                last_patch: Mutex::new(None),
                restart_calls: AtomicI64::new(0),
            }
        }
    }

    #[async_trait]
    impl ClusterControl for FakeControl {
        async fn patch_spec(&self, _ns: &str, _name: &str, patch: serde_json::Value) -> Result<()> {
            *self.last_patch.lock().unwrap() = Some(patch);
            Ok(())
        }

        async fn statefulset_ready_replicas(&self, _ns: &str, _name: &str) -> Result<i64> {
            Ok(self.ready_replicas.load(Ordering::SeqCst))
        }

        async fn trigger_rolling_restart(&self, _ns: &str, _name: &str) -> Result<()> {
            self.restart_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn admin_token(&self, _ns: &str, _lumen: &Lumen) -> Result<Option<String>> {
            Ok(None)
        }

        fn shard_base_url(&self, _ns: &str, _name: &str, shard: u32) -> String {
            format!("http://unused-in-this-test.invalid/shard-{shard}")
        }
    }

    fn http_client() -> reqwest::Client {
        reqwest::Client::new()
    }

    #[tokio::test]
    async fn drive_tick_complete_with_no_trigger_is_noop() {
        let lumen = lumen_with(spec(1, 1, Some(1_000_000)), Some(LumenStatus::default()));
        let control = FakeControl::new(0);
        let outcome = drive_tick(&control, &http_client(), &lumen).await;
        assert_eq!(
            outcome,
            DriveOutcome::NoOp("no crossed threshold, unsupported topology, or maxShards reached")
        );
        assert!(control.last_patch.lock().unwrap().is_none());
    }

    #[tokio::test]
    async fn drive_tick_starts_split_on_crossed_threshold() {
        let lumen = lumen_with(
            spec(2, 1, Some(1_000_000)),
            Some(status_with_blocking("prepareThresholdCrossed")),
        );
        let control = FakeControl::new(0);
        let outcome = drive_tick(&control, &http_client(), &lumen).await;
        assert_eq!(
            outcome,
            DriveOutcome::StartedSplit {
                target_shard_count: 3
            }
        );
        let patch = control.last_patch.lock().unwrap().clone().unwrap();
        assert_eq!(patch["spec"]["shardCount"], json!(3));
        assert_eq!(
            patch["spec"]["reshardPolicy"]["workflow"]["phase"],
            json!("PrepareSplit")
        );
        assert_eq!(
            patch["spec"]["reshardPolicy"]["workflow"]["targetShardCount"],
            json!(3)
        );
    }

    #[tokio::test]
    async fn drive_tick_prepare_split_waits_for_new_pod() {
        let mut s = spec(3, 1, Some(1_000_000));
        s.reshard_policy.workflow = ReshardWorkflowSpec {
            phase: ReshardPhase::PrepareSplit,
            target_shard_count: Some(3),
        };
        let lumen = lumen_with(s, None);
        // Only 2 of the 3 desired pods are ready yet.
        let control = FakeControl::new(2);
        let outcome = drive_tick(&control, &http_client(), &lumen).await;
        assert_eq!(
            outcome,
            DriveOutcome::WaitingForNewShard {
                target_shard_count: 3
            }
        );
        assert!(control.last_patch.lock().unwrap().is_none());
    }

    #[tokio::test]
    async fn drive_tick_prepare_split_advances_once_new_pod_ready() {
        let mut s = spec(3, 1, Some(1_000_000));
        s.reshard_policy.workflow = ReshardWorkflowSpec {
            phase: ReshardPhase::PrepareSplit,
            target_shard_count: Some(3),
        };
        let lumen = lumen_with(s, None);
        let control = FakeControl::new(3);
        let outcome = drive_tick(&control, &http_client(), &lumen).await;
        assert_eq!(outcome, DriveOutcome::AdvancedToSplitting);
        let patch = control.last_patch.lock().unwrap().clone().unwrap();
        assert_eq!(
            patch["spec"]["reshardPolicy"]["workflow"]["phase"],
            json!("Splitting")
        );
    }

    #[tokio::test]
    async fn drive_tick_resumable_after_simulated_restart_mid_prepare_split() {
        // AC2 (narrowed to the k8s-facing half): a driver restart mid
        // PrepareSplit re-derives the exact same wait/advance decision from
        // the persisted CR alone — no in-process state survives between the
        // two calls below (a fresh FakeControl each time simulates a fresh
        // process).
        let mut s = spec(3, 1, Some(1_000_000));
        s.reshard_policy.workflow = ReshardWorkflowSpec {
            phase: ReshardPhase::PrepareSplit,
            target_shard_count: Some(3),
        };
        let lumen = lumen_with(s, None);

        let before_restart = FakeControl::new(2);
        assert_eq!(
            drive_tick(&before_restart, &http_client(), &lumen).await,
            DriveOutcome::WaitingForNewShard {
                target_shard_count: 3
            }
        );

        // "Restart": brand-new control + a freshly-deserialized-shaped Lumen
        // (same spec/status values, simulating a re-fetch from the API
        // server), new pod now ready.
        let lumen_after_restart = lumen_with(lumen.spec.clone(), lumen.status.clone());
        let after_restart = FakeControl::new(3);
        assert_eq!(
            drive_tick(&after_restart, &http_client(), &lumen_after_restart).await,
            DriveOutcome::AdvancedToSplitting
        );
    }
}
// CODEGEN-END
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/operator/reshard_driver.rs
    action: create
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1381 (#1319 R2 executor): new autonomous reshard phase driver.
      Checkpointed, resumable state machine over `spec.reshardPolicy.
      workflow.phase` (`Complete -> PrepareSplit -> Splitting -> CatchingUp
      -> Complete`), gated by `should_start_split` (R3 safety rails:
      `maxShardBytes` must be set, single-member only, `maxShards` ceiling
      respected). `run_migration_pass` is the first real, non-test caller of
      `crate::reshard::bucket_moves` / `crate::reshard::
      snapshot_reshard_batches` (AC3), driving the `#1380` admin verbs
      (`POST /admin/backup:scoped`, `POST /admin/reshard:apply`, `POST
      /admin/reshard:evict`) against real shards via the injectable
      `ClusterControl::shard_base_url` seam. Independently leader-gated
      background loop (`spawn_reshard_driver_loop`) spawned alongside the
      existing live-usage loop from `reconcile::run`.
```
