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
//! - **CatchingUp** ([`advance_catching_up`], resequenced by #1396 R1/R2):
//!   arm a write-pause fence over every still-moving bucket on its current
//!   (source) owner (R2, see below), run the *same* migration pass again —
//!   an idempotent re-sync that, under the fence, is guaranteed a converged
//!   snapshot of every moving bucket — checkpoint **only the new/target
//!   shard** durably, evict every moved bucket from every old shard
//!   ([`crate::reshard`]'s `evict` is also idempotent), checkpoint every
//!   **source** shard durably, then flip `spec.shardMap` to the target map
//!   in the same patch that clears `workflow.targetShardCount` and resets
//!   phase -> `Complete`, followed by [`trigger_rolling_restart`], then
//!   clear the fence (unconditionally, on every exit path). Calling evict
//!   against the **new**, already-committed map (not the stale old one)
//!   means the driver never needs to retain the old map across a restart —
//!   the source of the classic "lost the old map after cutover" resumability
//!   trap. Checkpointing the target *before* evicting sources — rather than
//!   checkpointing everything only after both migration and eviction, as
//!   this driver did before #1396 — is R1: an eviction becoming durable (or
//!   even being attempted) before the target's copy of the same data is
//!   durably checkpointed can lose data that exists on no durable shard at
//!   all if the process crashes in between; see [`advance_catching_up`]'s
//!   own doc for the full crash-at-every-step analysis.
//!
//! A driver-side error at any step ([`DriveOutcome::Blocked`]) leaves the CR
//! spec untouched; the next tick retries the same phase from the same
//! persisted fields (R3).
//!
//! ## Write-pause fence during the final CatchingUp pass (#1396 R2)
//!
//! Even with the `Splitting` + `CatchingUp` double migration pass, a write
//! that lands on a moved bucket's old (source) shard after the *last*
//! migration-copy read but before that bucket's eviction is never re-copied
//! anywhere — eviction then silently drops it. [`advance_catching_up`] closes
//! this gap with a bounded, status-visible write pause (the mechanism #1381's
//! R5 review sanctioned: "a bounded final pause of writes to still-moving
//! buckets is acceptable if needed for convergence, but must be bounded and
//! reported in status") rather than a repeat-until-converged loop: arming the
//! fence *before* the tick's migration pass means that single pass is already
//! a complete snapshot of every fenced bucket, because no write can land on
//! them while it runs. [`crate::api::WriteFence`] (`POST
//! /admin/reshard:fence`) is the serving-side seam; a fenced write gets `503
//! bucket_write_paused` rather than being silently dropped or racing the map
//! flip. The fence is armed on every **source** shard (the live owners until
//! this tick's own cutover patch), and cleared — unconditionally, on every
//! exit path of [`advance_catching_up`], success or `Blocked` — once the
//! sequence finishes. A driver process that crashes before that explicit
//! clear cannot wedge writes forever: [`WriteFence::blocks`] enforces
//! [`WRITE_FENCE_TTL_SECS`] as a deadline on the *serving* pod itself,
//! independent of the driver's liveness.
//!
//! ## Migration durability (#1389)
//!
//! `Engine::apply_reshard_batch`/`evict_not_owned` (`storage.rs`, #1380)
//! mutate engine state directly rather than through `WriteCoordinator`/the
//! AOF, so — unlike ordinary writes — their durability is not implied by the
//! engine's normal write path; with #1387's embedded persistence it was
//! previously captured only by the next periodic `LUMEN_SNAPSHOT_SECS`
//! checkpoint (default 300s), well after this driver's own cutover restart
//! (~60-90s later) — observed live as 806 migrated batches lost on the
//! target and an eviction silently undone on the source (#1387's report).
//!
//! Two designs were considered: (a) route `:apply`/`:evict` through the AOF/
//! `WriteCoordinator` path as new `RaftLogEntry` variants, or (b) an explicit
//! synchronous checkpoint step the driver invokes and awaits per touched
//! shard before cutover. **(b) was chosen**: a whole `ReshardBatch`'s
//! `SnapshotV1` delta can be large (bounded by `MAX_EXTERNAL_IDS_PER_BATCH`,
//! but still potentially many collections/fields), which sits awkwardly as a
//! single `WalRecord`/AOF frame designed around one bounded mutation
//! (`Index`/`ReplaceDocs`/etc); it would also need new apply-loop branches
//! and idempotency semantics distinct from every existing `RaftLogEntry`
//! variant, whose apply methods mutate exactly one collection deterministically
//! rather than merge a whole delta. (b) reuses `segment_rdb.rs`'s
//! `SegmentRdbStore::save` verbatim — the exact call the periodic snapshotter
//! already makes, just invoked synchronously on demand via a new
//! `POST /admin/checkpoint` admin verb ([`crate::api::CheckpointSink`]) — no
//! new WAL record shape, no new apply-loop branch, no new idempotency
//! reasoning: `save` already re-seals the *entire* current engine state
//! (including whatever `:apply`/`:evict` already mutated) atomically
//! (stage-then-rename), so one checkpoint call captures every migration
//! mutation made so far, not just the most recent one.
//!
//! [`checkpoint_shards`] calls `POST /admin/checkpoint` on an explicit shard
//! set; [`advance_catching_up_fenced`] calls it twice per tick — once for
//! just the new/target shard immediately after migration and before any
//! source eviction is attempted (#1396 R1), and again for every source shard
//! (`0..current.physical_shard_count()`, `evict_old_shards`'s target set)
//! after eviction and before the `shardMap` cutover patch and
//! [`trigger_rolling_restart`]. A checkpoint failure on either call reports
//! [`DriveOutcome::Blocked`] and leaves `spec` at `CatchingUp` untouched
//! (R3): the next tick re-runs the whole idempotent
//! migrate/checkpoint/evict/checkpoint sequence, consistent with #1381's
//! spec-is-the-checkpoint semantics — cutover never fires on a shard whose
//! migration mutations are not yet durable, and eviction is never even
//! attempted before the target's copy of the same data is durable.
//! `checkpoint_shard` (#1396 R3) also now requires the response body to
//! report `persisted == true`; a `200 {"persisted": false}` — the vacuous
//! "no durable store configured" response `admin_checkpoint` returns for
//! [`crate::api::NoopCheckpoint`] deployments — is treated as a failed
//! checkpoint, not a satisfied durability gate.
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

/// TTL for the write-pause fence [`advance_catching_up`] arms over still-moving
/// buckets during its final migration pass (#1396 R2) — generous relative to
/// one tick's HTTP round trips (a scoped-backup fetch + apply batches across
/// however many source shards a split touches, then evict + checkpoint) while
/// still bounded; the fence is a crash-safety backstop the *serving* pod
/// enforces independent of the driver's own liveness, see
/// [`crate::api::WriteFence`]. Re-armed fresh every tick that needs one, so a
/// healthy, slow-but-progressing driver never races its own TTL.
const WRITE_FENCE_TTL_SECS: u64 = 120;

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
    // #1396 R5: re-derive freshness here rather than trusting the status
    // subresource's own `blockingConditions` alone. `reshard_status_with_usage`
    // (crd.rs) already refuses to report a *threshold* condition against a
    // stale usage measurement (it reports `usageStalePostCutover` instead —
    // see the #1386 tests below), but that only protects the write path: a
    // status write from an in-flight scrape can still be the one currently
    // stored when a *later* `spec.shardMap` cutover lands (a second,
    // independent split racing this one, or an operator restart reordering
    // writes), leaving a `prepareThresholdCrossed`/`urgentThresholdCrossed`
    // condition on disk that was computed against a map version the CR has
    // already moved past. Requiring the status's `usageMeasuredAtMapVersion`
    // to equal the CR's *current* `spec.shardMap.version` at the moment this
    // trigger decision is made closes that race without needing the status
    // writer and this reader to be perfectly ordered.
    if status.reshard.usage_measured_at_map_version != Some(lumen.spec.shard_map.version) {
        return false;
    }
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

/// `POST /admin/reshard:fence` (#1396 R2) against one shard: `buckets`
/// non-empty arms a bounded write pause over those virtual buckets;
/// `buckets` empty clears any currently-armed pause. See
/// [`crate::api::WriteFence`].
async fn reshard_fence_call(
    http: &reqwest::Client,
    base_url: &str,
    token: Option<&str>,
    virtual_bucket_count: u32,
    buckets: &BTreeSet<u32>,
    ttl_secs: u64,
) -> Result<()> {
    let mut req = http
        .post(format!("{base_url}/admin/reshard:fence"))
        .json(&json!({
            "virtual_bucket_count": virtual_bucket_count,
            "buckets": buckets,
            "ttl_secs": ttl_secs,
        }));
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let resp = req
        .send()
        .await
        .with_context(|| format!("POST {base_url}/admin/reshard:fence"))?;
    if !resp.status().is_success() {
        bail!("{base_url}/admin/reshard:fence returned {}", resp.status());
    }
    Ok(())
}

/// Arm (non-empty `buckets`) or clear (empty `buckets`) the write-pause
/// fence on every shard `current` owns — the live map's current owners,
/// where writes to a still-moving bucket land until this tick's own cutover
/// patch flips `spec.shardMap`.
async fn set_write_fence(
    control: &dyn ClusterControl,
    http: &reqwest::Client,
    namespace: &str,
    name: &str,
    lumen: &Lumen,
    current: &VirtualBucketShardMap,
    buckets: &BTreeSet<u32>,
    ttl_secs: u64,
) -> Result<()> {
    let token = control.admin_token(namespace, lumen).await?;
    for shard in 0..current.physical_shard_count() {
        let url = control.shard_base_url(namespace, name, shard);
        reshard_fence_call(
            http,
            &url,
            token.as_deref(),
            current.virtual_bucket_count(),
            buckets,
            ttl_secs,
        )
        .await?;
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

/// `POST /admin/checkpoint` (#1389 R1/R2; durability gate hardened by #1396
/// R3) against one shard: force its migration mutations (`:apply`/`:evict`,
/// which bypass `WriteCoordinator`/the AOF) into the same durability domain
/// ordinary writes reach, and wait for the response before this shard is
/// considered safe to restart.
///
/// A 200 response alone is not proof of durability: [`crate::api`]'s
/// `admin_checkpoint` handler returns `200 {"persisted": false}` — not an
/// error status — when the shard has no durable store configured (the
/// vacuous, RAM-only [`crate::api::NoopCheckpoint`] sink; see that type's
/// docs), which is exactly the "checkpoint looked like it worked but nothing
/// was actually made durable" gap #1396's review confirmed (a bare
/// `is_success()` check treated that response as a satisfied gate). This
/// function now parses the body and requires `persisted == true`; anything
/// else — `false`, or a body this shard's response doesn't even carry the
/// key for — is treated as a failed checkpoint, surfacing as
/// [`DriveOutcome::Blocked`] naming the shard rather than a cutover that
/// proceeds over undurable data.
async fn checkpoint_shard(
    http: &reqwest::Client,
    base_url: &str,
    token: Option<&str>,
) -> Result<()> {
    let mut req = http.post(format!("{base_url}/admin/checkpoint"));
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let resp = req
        .send()
        .await
        .with_context(|| format!("POST {base_url}/admin/checkpoint"))?;
    if !resp.status().is_success() {
        bail!("{base_url}/admin/checkpoint returned {}", resp.status());
    }
    let body: serde_json::Value = resp
        .json()
        .await
        .with_context(|| format!("decode {base_url}/admin/checkpoint response"))?;
    let persisted = body
        .get("persisted")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !persisted {
        bail!(
            "{base_url}/admin/checkpoint did not report persisted=true (shard has no durable \
             checkpoint sink configured, or the checkpoint failed) — cutover cannot proceed \
             over undurable migration mutations on this shard"
        );
    }
    Ok(())
}

/// #1389 R3, generalized by #1396 R1 into an explicit shard set: checkpoint
/// exactly `shards`. [`advance_catching_up`] now calls this twice per tick —
/// once for just the target/new shard immediately after migration and
/// *before* any source eviction is attempted, and again for every source
/// shard after eviction — rather than once for `0..target.physical_shard_
/// count()` after both migration and eviction had already run (the ordering
/// #1396's review found: an eviction becoming durable, or even being
/// attempted, before the target's copy of the same data was durably
/// checkpointed, could lose data on a crash between the two). A failure here
/// leaves the workflow in `CatchingUp` — resumable, never mid-cutover with
/// undurable data — and the next tick retries the same idempotent
/// migration/checkpoint/eviction/checkpoint sequence.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
async fn checkpoint_shards(
    control: &dyn ClusterControl,
    http: &reqwest::Client,
    namespace: &str,
    name: &str,
    lumen: &Lumen,
    shards: impl Iterator<Item = u32>,
) -> Result<()> {
    let token = control.admin_token(namespace, lumen).await?;
    for shard in shards {
        let url = control.shard_base_url(namespace, name, shard);
        checkpoint_shard(http, &url, token.as_deref()).await?;
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

/// #1396 R1/R2: durably ordered cutover. Old order was migrate -> evict ->
/// checkpoint-everything -> cutover, which could make a source shard's
/// eviction durable (or even attempt it) before the target shard's copy of
/// the same data was durably checkpointed — a crash between eviction and
/// that too-late checkpoint could lose data that, at that instant, existed
/// on no durable shard at all (#1387's exact failure shape, reintroduced by
/// evicting ahead of a per-shard-ordered checkpoint). New order: migrate ->
/// checkpoint target only -> evict sources -> checkpoint sources -> cutover.
/// Crash-safety at every boundary (moved data is durable on at least one
/// shard at every point once migration completes):
/// - Crash after migrate, before the target checkpoint: sources still hold
///   their data (not yet evicted); retry re-migrates (idempotent, #1380)
///   and the target checkpoint eventually succeeds.
/// - Crash after the target checkpoint, before eviction: the target already
///   durably holds the moved data; retry replays migrate (no-op) and the
///   target checkpoint (no-op re-confirm), then proceeds to evict.
/// - Crash after eviction (RAM-only until its own checkpoint), before the
///   source checkpoint: even if the pod restart the driver is about to
///   trigger loses the in-RAM eviction, a retry is still safe — the target
///   already durably has the moved data from the earlier target checkpoint,
///   so a retried migrate is a no-op, a retried evict is idempotent, and the
///   source checkpoint retries until it succeeds. Eviction is never durable
///   nor attempted before the target's copy is durable, so this crash can
///   never lose data.
/// - Crash after the source checkpoint, before the cutover patch: both
///   sides are durable; retry replays every step as a no-op until the
///   cutover patch finally lands.
///
/// R2: the whole sequence below runs under a write-pause fence (`POST
/// /admin/reshard:fence`) armed over every still-moving bucket on its
/// current (source) owners, so this tick's migration pass is guaranteed a
/// converged snapshot of those buckets — closing the gap where a write
/// lands on a source shard after the last migration-copy read but before
/// that bucket's eviction and is silently dropped. The fence is cleared on
/// every exit path (success or `Blocked`); see [`crate::api::WriteFence`]
/// for why a crashed driver can never leave it armed permanently.
async fn advance_catching_up(
    control: &dyn ClusterControl,
    http: &reqwest::Client,
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
    let moves = match bucket_moves(&current, &target) {
        Ok(m) => m,
        Err(err) => return DriveOutcome::Blocked(err.to_string()),
    };
    let moving_buckets: BTreeSet<u32> = moves.iter().map(|m| m.bucket).collect();

    if !moving_buckets.is_empty() {
        if let Err(err) = set_write_fence(
            control,
            http,
            namespace,
            name,
            lumen,
            &current,
            &moving_buckets,
            WRITE_FENCE_TTL_SECS,
        )
        .await
        {
            return DriveOutcome::Blocked(err.to_string());
        }
    }

    let outcome =
        advance_catching_up_fenced(control, http, namespace, name, lumen, &current, &target).await;

    if !moving_buckets.is_empty() {
        // Always clear, on every exit path: the fence must never outlive
        // this tick. If this clear itself fails (or the process dies before
        // reaching it), WRITE_FENCE_TTL_SECS still bounds how long writes to
        // these buckets stay paused — the serving pod enforces that
        // deadline on its own, independent of the driver ever coming back.
        if let Err(err) = set_write_fence(
            control,
            http,
            namespace,
            name,
            lumen,
            &current,
            &BTreeSet::new(),
            0,
        )
        .await
        {
            tracing::warn!(
                error = %err,
                "reshard driver: failed to clear write fence after CatchingUp tick; \
                 bounded by WRITE_FENCE_TTL_SECS"
            );
        }
    }

    outcome
}

/// The migrate/checkpoint/evict/checkpoint/cutover sequence proper, run
/// under [`advance_catching_up`]'s write fence. Split out so the fence's
/// arm/clear bracket is unconditional (always runs, regardless of which
/// step below fails) without duplicating the sequence itself.
async fn advance_catching_up_fenced(
    control: &dyn ClusterControl,
    http: &reqwest::Client,
    namespace: &str,
    name: &str,
    lumen: &Lumen,
    current: &VirtualBucketShardMap,
    target: &VirtualBucketShardMap,
) -> DriveOutcome {
    if let Err(err) = run_migration_pass(control, http, namespace, name, lumen).await {
        return DriveOutcome::Blocked(err.to_string());
    }

    // R1: the target/new shard's copy of the just-migrated data must be
    // durable BEFORE any source eviction is even attempted.
    let new_shard = target.physical_shard_count().saturating_sub(1);
    if let Err(err) = checkpoint_shards(
        control,
        http,
        namespace,
        name,
        lumen,
        std::iter::once(new_shard),
    )
    .await
    {
        return DriveOutcome::Blocked(err.to_string());
    }

    if let Err(err) = evict_old_shards(control, http, namespace, name, lumen, current, target).await
    {
        return DriveOutcome::Blocked(err.to_string());
    }

    // R1: sources' eviction must itself be durable before cutover, same
    // rationale #1389 already established — a crash-then-restart must never
    // resurrect data this shard no longer owns.
    if let Err(err) = checkpoint_shards(
        control,
        http,
        namespace,
        name,
        lumen,
        0..current.physical_shard_count(),
    )
    .await
    {
        return DriveOutcome::Blocked(err.to_string());
    }

    let patch = json!({
        "spec": {
            "shardMap": {
                "version": target.version(),
                "virtualBucketCount": target.virtual_bucket_count(),
                "assignments": map_assignments(target),
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
                // R5's freshness gate requires this to match the CR's
                // current `spec.shard_map.version`; every fixture built with
                // `spec()` hardcodes `shard_map.version: 0`, so `Some(0)`
                // here models a status write that was actually fresh at the
                // scenario's map version, not a value that happens to fail
                // the new check by fixture omission.
                usage_measured_at_map_version: Some(0),
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

    // ---- #1396 AC5: should_start_split re-derives freshness itself -----

    #[test]
    fn should_start_split_false_on_stale_status_map_version() {
        // A `blockingConditions` entry alone is not enough: if the status
        // subresource's `usageMeasuredAtMapVersion` predates the CR's
        // *current* `spec.shardMap.version` (a lagging/stale status write —
        // e.g. an in-flight scrape landing after a later cutover), the
        // trigger must refuse to fire even though the string condition is
        // present, regardless of what produced that stale status.
        let mut s = spec(2, 1, Some(1_000_000));
        s.shard_map.version = 1; // CR has already moved to map version 1.
        let status = LumenStatus {
            reshard: LumenReshardStatus {
                blocking_conditions: vec!["urgentThresholdCrossed".to_string()],
                usage_measured_at_map_version: Some(0), // stale: still map 0.
                ..Default::default()
            },
            ..Default::default()
        };
        let lumen = lumen_with(s, Some(status));
        assert!(!should_start_split(&lumen));
    }

    #[test]
    fn should_start_split_true_on_fresh_status_map_version() {
        // Same shape, but the status was measured at the CR's current map
        // version: a legitimate trigger and must still fire.
        let mut s = spec(2, 1, Some(1_000_000));
        s.shard_map.version = 1;
        let status = LumenStatus {
            reshard: LumenReshardStatus {
                blocking_conditions: vec!["urgentThresholdCrossed".to_string()],
                usage_measured_at_map_version: Some(1), // fresh: matches map 1.
                ..Default::default()
            },
            ..Default::default()
        };
        let lumen = lumen_with(s, Some(status));
        assert!(should_start_split(&lumen));
    }

    #[test]
    fn should_start_split_false_with_no_status_yet() {
        let lumen = lumen_with(spec(1, 1, Some(1_000_000)), None);
        assert!(!should_start_split(&lumen));
    }

    // ---- #1386 AC1/AC2: post-cutover usage freshness -------------------

    #[test]
    fn should_start_split_false_on_stale_pre_cutover_usage() {
        // AC1: at `Complete` with a usage measurement whose generation
        // (`usageMeasuredAtMapVersion`) predates the CR's current
        // `shardMap.version` — the exact shape the shard-usage cache is in
        // for one scrape tick right after a split's cutover — the driver
        // must not start a split, regardless of how far past the urgent
        // threshold the (stale) cached percentage is.
        let mut s = spec(2, 1, Some(1_000_000));
        s.shard_map.version = 1; // just cut over to the post-split map
        let mut usage = BTreeMap::new();
        usage.insert(0u32, 900_000u64); // 90%, well past urgent(85%)
        let status = s.reshard_status_with_usage(&usage, 0 /* stale: pre-cutover */);
        assert_eq!(status.blocking_conditions, vec!["usageStalePostCutover"]);
        let lumen = lumen_with(
            s,
            Some(LumenStatus {
                reshard: status,
                ..Default::default()
            }),
        );
        assert!(!should_start_split(&lumen));
    }

    #[test]
    fn should_start_split_true_on_fresh_post_cutover_usage_above_urgent() {
        // AC2: once the usage cache carries a measurement tagged with the
        // CR's *current* `shardMap.version`, a genuinely still-hot shard is
        // a legitimate cascade trigger and must start the next split.
        let mut s = spec(2, 1, Some(1_000_000));
        s.shard_map.version = 1;
        let mut usage = BTreeMap::new();
        usage.insert(1u32, 900_000u64); // 90%, past urgent(85%), fresh
        let status =
            s.reshard_status_with_usage(&usage, 1 /* fresh: matches shardMap.version */);
        assert_eq!(status.blocking_conditions, vec!["urgentThresholdCrossed"]);
        let lumen = lumen_with(
            s,
            Some(LumenStatus {
                reshard: status,
                ..Default::default()
            }),
        );
        assert!(should_start_split(&lumen));
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

    // ---- #1396 AC3: checkpoint_shard requires persisted == true --------

    #[tokio::test]
    async fn checkpoint_shard_blocked_when_response_reports_persisted_false() {
        // A 200 with `persisted: false` is the exact shape `admin_checkpoint`
        // returns when the shard has no durable checkpoint sink configured
        // (the vacuous NoopCheckpoint case) — this must never be treated as
        // a satisfied durability gate.
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("POST"))
            .and(wiremock::matchers::path("/admin/checkpoint"))
            .respond_with(
                wiremock::ResponseTemplate::new(200).set_body_json(json!({ "persisted": false })),
            )
            .mount(&server)
            .await;
        let result = checkpoint_shard(&http_client(), &server.uri(), None).await;
        assert!(
            result.is_err(),
            "persisted: false must not satisfy the checkpoint gate"
        );
    }

    #[tokio::test]
    async fn checkpoint_shard_blocked_when_response_omits_persisted_key() {
        // A malformed/older response with no `persisted` key at all must
        // fail closed (defaults to not-durable), not be treated as success.
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("POST"))
            .and(wiremock::matchers::path("/admin/checkpoint"))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(json!({})))
            .mount(&server)
            .await;
        let result = checkpoint_shard(&http_client(), &server.uri(), None).await;
        assert!(
            result.is_err(),
            "a response missing the persisted key must fail closed"
        );
    }

    #[tokio::test]
    async fn checkpoint_shard_ok_when_response_reports_persisted_true() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("POST"))
            .and(wiremock::matchers::path("/admin/checkpoint"))
            .respond_with(
                wiremock::ResponseTemplate::new(200).set_body_json(json!({ "persisted": true })),
            )
            .mount(&server)
            .await;
        let result = checkpoint_shard(&http_client(), &server.uri(), None).await;
        assert!(
            result.is_ok(),
            "persisted: true must satisfy the checkpoint gate: {result:?}"
        );
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
