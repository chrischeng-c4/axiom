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
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use kube::api::{Api, ApiResource, DynamicObject, Patch, PatchParams};
use kube::{Client, ResourceExt};
use serde_json::json;

use crate::auth::{Role, TokenClaims};
use crate::operator::crd::{AuthMode, Lumen, ReshardPhase};
use crate::operator::lease::{self, Election};
use crate::reshard::{
    bucket_moves, snapshot_reshard_batches, snapshot_reshard_prune_chunks, ReshardBatch,
    ReshardPruneChunk,
};
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

/// The production default [`ClusterControl::write_fence_ttl_secs`] value
/// (#1443 R1/AC1), exposed so integration tests can fall back to the real
/// default from a `fence_ttl_secs: Option<u64>`-style override field without
/// needing [`WRITE_FENCE_TTL_SECS`] itself to be `pub`.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
pub fn default_write_fence_ttl_secs() -> u64 {
    WRITE_FENCE_TTL_SECS
}

/// How many consecutive [`advance_catching_up`] ticks short-circuit on a
/// recorded [`OversizedDocumentBlock`] (#1444 R2) before attempting the full
/// fenced migration pass again. Bounds how long a document an operator has
/// since fixed (deleted or shrunk) stays wedged after the fix without
/// re-arming the write-pause fence — and reopening the recurring 503 window
/// the fix closes — on every single tick while the condition is genuinely
/// unchanged. `DRIVER_POLL_INTERVAL * OVERSIZE_RECHECK_TICKS` (5 minutes at
/// the current 20s poll interval) is the same order of magnitude as
/// [`WRITE_FENCE_TTL_SECS`]/`SHARD_USAGE_POLL_INTERVAL`-style bounds
/// elsewhere in this driver.
const OVERSIZE_RECHECK_TICKS: u32 = 15;

/// Distinguishes an apply failure caused by exactly one document's batch
/// serializing past [`crate::reshard::ADMIN_ROUTE_BODY_LIMIT_BYTES`] — the
/// `snapshot_reshard_batches`/`byte_cap_chunk` floor case
/// (`crate::reshard`'s module doc's "one document cannot be split further")
/// — from any other reason `POST /admin/reshard:apply` can fail (#1444 R2).
/// Deterministic every retry (nothing about the data or the byte cap changes
/// tick to tick), unlike a transient network/5xx error, so this is surfaced
/// as a distinct `status.reshard` blocking condition (see
/// [`oversize_block_condition`]) instead of the generic
/// [`DriveOutcome::Blocked`] message every other failure produces, and used
/// to skip re-arming the write-pause fence on a tick already known to fail
/// identically (see [`advance_catching_up`]).
#[derive(Debug, Clone, PartialEq, Eq)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
pub struct OversizedDocumentBlock {
    pub collection: String,
    pub external_id: String,
    pub bytes: usize,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
impl std::fmt::Display for OversizedDocumentBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "reshard blocked: collection `{}` document `{}` serializes to {} bytes, over the \
             {} byte /admin/reshard:apply body limit; this single document cannot be split into \
             a smaller batch — shrink or remove its large field values (long text, vectors, \
             hashes), or exclude it from the collection, before this split can continue",
            self.collection,
            self.external_id,
            self.bytes,
            crate::reshard::ADMIN_ROUTE_BODY_LIMIT_BYTES
        )
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
impl std::error::Error for OversizedDocumentBlock {}

/// `"<namespace>/<name>" -> (owning CR's metadata.uid, block, ticks skipped
/// on it so far)`, written by [`run_migration_pass_impl`] and consumed by
/// [`advance_catching_up`] (mutating, to decide/count a skip) and
/// [`oversize_block_condition`] (read-only, for `reconcile.rs`'s
/// `status_patch`) — #1444 R2. Mirrors `reconcile.rs`'s own
/// `ShardUsageCache` pattern: a synchronous status projection reads a cache
/// a background loop writes, rather than doing I/O itself.
///
/// Keyed by `namespace/name` (not `uid`, which is not stable input for a
/// lookup before an object exists) but every entry carries the `uid` of the
/// CR it was recorded for (#1458 R4): a namespace/name pair is not a stable
/// identity across a delete-and-recreate — the new CR gets a fresh `uid`
/// from the API server — so every read compares the stored `uid` against
/// the caller's current one and treats a mismatch as no entry, giving a
/// recreated CR a clean `status.reshard` immediately rather than inheriting
/// a stale wedge left by the deleted CR's last tick. [`prune_oversize_cache`]
/// bounds the map by dropping entries whose `uid` is no longer live.
type OversizeBlockCache = std::sync::Mutex<BTreeMap<String, (String, OversizedDocumentBlock, u32)>>;

fn oversize_block_cache() -> &'static OversizeBlockCache {
    static CACHE: std::sync::OnceLock<OversizeBlockCache> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::Mutex::new(BTreeMap::new()))
}

fn oversize_cache_key(namespace: &str, name: &str) -> String {
    format!("{namespace}/{name}")
}

/// Record (or refresh) a discovered oversize wedge for `namespace/name`'s
/// `uid`, resetting its skip counter — a fresh discovery, whether this is
/// the first tick to hit it or a periodic recheck (#1444 R2, see
/// [`OVERSIZE_RECHECK_TICKS`]) that hit the same wedge again. `pub(crate)`
/// rather than private so `reconcile.rs`'s `status_patch` tests can drive the
/// exact cache [`oversize_block_condition`] reads, without widening this past
/// crate-internal visibility.
pub(crate) fn record_oversize_block(
    namespace: &str,
    name: &str,
    uid: &str,
    block: OversizedDocumentBlock,
) {
    oversize_block_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(
            oversize_cache_key(namespace, name),
            (uid.to_string(), block, 0),
        );
}

/// Clear any recorded oversize wedge for `namespace/name`, regardless of
/// which `uid` recorded it — called whenever a migration pass for it
/// completes without hitting one (whatever was wedged is resolved) and when
/// the workflow returns to phase `Complete` (#1458 R4). `pub(crate)` for the
/// same test-seam reason as [`record_oversize_block`].
pub(crate) fn clear_oversize_block(namespace: &str, name: &str) {
    oversize_block_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .remove(&oversize_cache_key(namespace, name));
}

/// Drop every cached entry whose `uid` is not in `live_uids` (#1458 R4) —
/// called once per [`spawn_reshard_driver_loop`] poll, which already lists
/// every live `Lumen` CR cluster-wide, so this needs no extra k8s API call.
/// Bounds the cache's growth across an unbounded number of past
/// delete-and-recreate cycles on the same `namespace/name`.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
pub(crate) fn prune_oversize_cache(live_uids: &BTreeSet<String>) {
    oversize_block_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .retain(|_, (uid, _, _)| live_uids.contains(uid));
}

/// If `namespace/name`'s current `uid` has a recorded oversize wedge AND has
/// not yet used up its [`OVERSIZE_RECHECK_TICKS`] skip budget, bump its skip
/// counter and return it — the caller ([`advance_catching_up`]) should
/// short-circuit to [`DriveOutcome::Blocked`] without arming the
/// write-pause fence. Returns `None` (no skip) once the budget is exhausted
/// or the cached entry belongs to a different `uid` (#1458 R4 — a stale
/// entry from a deleted-and-recreated CR), letting the next real attempt
/// either clear the wedge (if fixed) or re-record it with a fresh budget.
fn should_skip_for_oversize(
    namespace: &str,
    name: &str,
    uid: &str,
) -> Option<OversizedDocumentBlock> {
    let mut cache = oversize_block_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (cached_uid, block, ticks) = cache.get_mut(&oversize_cache_key(namespace, name))?;
    if cached_uid != uid || *ticks >= OVERSIZE_RECHECK_TICKS {
        return None;
    }
    *ticks += 1;
    Some(block.clone())
}

/// The oversized-document block currently recorded for `namespace/name`'s
/// current `uid`, if any (#1444 R2; `uid`-scoped by #1458 R4) — read-only,
/// does not affect [`should_skip_for_oversize`]'s skip budget. `reconcile.
/// rs`'s `status_patch` calls this to layer a distinct `status.reshard`
/// blocking condition + remediation message onto the policy/usage-derived
/// status. A cached entry belonging to a different `uid` (a deleted-and-
/// recreated CR under the same `namespace/name`) is treated as no entry, so
/// the recreated CR's status is clean immediately rather than waiting for
/// [`prune_oversize_cache`]'s next poll.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
pub fn oversize_block_condition(
    namespace: &str,
    name: &str,
    uid: &str,
) -> Option<OversizedDocumentBlock> {
    oversize_block_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(&oversize_cache_key(namespace, name))
        .filter(|(cached_uid, _, _)| cached_uid == uid)
        .map(|(_, block, _)| block.clone())
}

/// #1467 R7: bounded escalation budget for [`advance_convergence`] — after
/// this many consecutive `AwaitingTopologyConvergence` ticks for the same
/// `(uid, map_version)` pair without observing convergence, the driver
/// raises a distinct `topologyConvergenceStalled` status condition. The
/// fence itself is NEVER dropped when this budget is exceeded — re-arming
/// continues every tick exactly as before — this only makes an
/// abnormally-long convergence wait observable to operators.
/// `DRIVER_POLL_INTERVAL * CONVERGENCE_STALL_TICKS` = 10 minutes at the
/// current 20s poll interval, the same order of magnitude as
/// `OVERSIZE_RECHECK_TICKS`'s ~5 minutes.
///
/// #1485 R2: [`convergence_stall_cache`]/[`record_convergence_await`] below
/// (this tick-count budget) stay in place as a fast, driver-memory-only
/// signal, but they are no longer the authoritative source for whether the
/// stall budget has been exceeded — [`CONVERGENCE_STALL_SECS`], checked
/// against the durable `workflow.convergenceWaitStartedAt` timestamp, is.
const CONVERGENCE_STALL_TICKS: u32 = 30;

/// #1485 R2: wall-clock equivalent of [`CONVERGENCE_STALL_TICKS`] at the
/// current [`DRIVER_POLL_INTERVAL`] — the durable stall budget
/// [`convergence_stall_condition`] applies to `workflow.
/// convergenceWaitStartedAt`. Computing the budget this way (elapsed time
/// since a persisted CR timestamp) rather than from an in-process tick
/// count is what makes both the budget and the `topologyConvergenceStalled`
/// condition it gates survive an operator restart mid-wait. `pub(crate)` so
/// `reconcile.rs`'s own tests can position a wait-start timestamp precisely
/// past the budget without sleeping in a unit test.
pub(crate) const CONVERGENCE_STALL_SECS: u64 =
    CONVERGENCE_STALL_TICKS as u64 * DRIVER_POLL_INTERVAL.as_secs();

/// The production [`CONVERGENCE_STALL_SECS`] value (#1485 R2), exposed the
/// same way [`default_write_fence_ttl_secs`] exposes [`WRITE_FENCE_TTL_SECS`]
/// — so integration tests can back-date `workflow.convergenceWaitStartedAt`
/// past the real budget (simulating an extended wait without sleeping)
/// without needing the constant itself to be `pub`.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
pub fn convergence_stall_budget_secs() -> u64 {
    CONVERGENCE_STALL_SECS
}

/// Current wall-clock time as epoch seconds, saturating to `0` on a clock
/// error (mirrors [`KubeClusterControl::trigger_rolling_restart`]'s own
/// inline `SystemTime::now()` call) — the source of every `#1485` durable
/// timestamp this module stamps into `workflow.convergenceWaitStartedAt` /
/// `workflow.convergenceRemediationRestartedAt`.
fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// `"<namespace>/<name>" -> (uid, map_version being awaited, consecutive
/// awaiting ticks)` — tracks how long [`advance_convergence`] has been
/// waiting for [`ClusterControl::serving_topology_converged`] to confirm
/// one particular `map_version`, for the R7 stall escalation. Mirrors
/// [`OversizeBlockCache`]'s shape and `uid`-scoping rationale (a
/// namespace/name pair is not stable identity across delete-and-recreate).
type ConvergenceStallCache = std::sync::Mutex<BTreeMap<String, (String, u64, u32)>>;

fn convergence_stall_cache() -> &'static ConvergenceStallCache {
    static CACHE: std::sync::OnceLock<ConvergenceStallCache> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::Mutex::new(BTreeMap::new()))
}

fn convergence_stall_key(namespace: &str, name: &str) -> String {
    format!("{namespace}/{name}")
}

/// Bump (or start) `namespace/name`'s consecutive-awaiting-ticks counter
/// for `map_version` and return `true` once [`CONVERGENCE_STALL_TICKS`] has
/// been exceeded (this tick should report the stalled condition). A
/// `uid`/`map_version` change (a delete-and-recreate, or a fresh split
/// starting a new convergence wait before the prior one finished) resets
/// the counter rather than carrying over an unrelated wait's budget.
/// `pub(crate)` for the same test-seam reason as
/// [`record_oversize_block`].
pub(crate) fn record_convergence_await(
    namespace: &str,
    name: &str,
    uid: &str,
    map_version: u64,
) -> bool {
    let mut cache = convergence_stall_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let entry = cache
        .entry(convergence_stall_key(namespace, name))
        .or_insert_with(|| (uid.to_string(), map_version, 0));
    if entry.0 != uid || entry.1 != map_version {
        *entry = (uid.to_string(), map_version, 0);
    }
    entry.2 = entry.2.saturating_add(1);
    entry.2 > CONVERGENCE_STALL_TICKS
}

/// Clear `namespace/name`'s convergence-stall tracker — called once
/// convergence is observed (or the workflow is no longer awaiting it), so a
/// resolved wait never leaves the next, unrelated wait starting from a
/// stale budget. `pub(crate)` for the same test-seam reason as
/// [`clear_oversize_block`].
pub(crate) fn clear_convergence_stall(namespace: &str, name: &str) {
    convergence_stall_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .remove(&convergence_stall_key(namespace, name));
}

/// Drop every cached convergence-stall entry whose `uid` is not in
/// `live_uids` — the [`prune_oversize_cache`] counterpart for this cache,
/// called from the same poll loop with the same already-listed live-CR set.
pub(crate) fn prune_convergence_stall_cache(live_uids: &BTreeSet<String>) {
    convergence_stall_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .retain(|_, (uid, _, _)| live_uids.contains(uid));
}

/// Whether an `awaitingTopologyConvergence` wait that began at
/// `wait_started_at` (`workflow.convergenceWaitStartedAt`, #1485 R2) has run
/// longer than [`CONVERGENCE_STALL_SECS`], for `reconcile.rs`'s
/// `status_patch` to layer a `topologyConvergenceStalled` blocking condition
/// onto the policy/usage-derived status. Computed purely from this one
/// persisted CR timestamp — not driver memory — so the answer is the same
/// whether or not the driver process has restarted since the wait began;
/// [`advance_convergence`]'s own bounded-remediation gate uses the exact
/// same computation. `None` (convergence not pending, or no wait recorded
/// yet) is never stalled.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
pub fn convergence_stall_condition(wait_started_at: Option<u64>) -> bool {
    wait_started_at
        .is_some_and(|started| now_epoch_secs().saturating_sub(started) > CONVERGENCE_STALL_SECS)
}

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

    /// TTL (seconds) [`advance_catching_up`]/[`advance_catching_up_fenced`]
    /// arm/re-arm the write-pause fence with (#1443 R1). Defaults to
    /// [`WRITE_FENCE_TTL_SECS`] — production behavior is unchanged; this
    /// exists purely as a test seam so a short-TTL/slow-checkpoint scenario
    /// can be exercised deterministically without waiting 120 real seconds.
    fn write_fence_ttl_secs(&self) -> u64 {
        WRITE_FENCE_TTL_SECS
    }

    /// Whether every serving pod is confirmed `Ready` on the serving
    /// StatefulSet's current rollout (#1458 R1) — the same k8s "rollout
    /// status" pattern `kubectl rollout status` checks:
    /// `.status.updateRevision == .status.currentRevision` (no rollout
    /// in-flight) and `.status.readyReplicas == desired_replicas`. Reuses
    /// [`advance_prepare_split`]'s existing readiness-polling seam rather
    /// than adding a new one. Defaults to `Ok(true)` — production behavior
    /// only changes once [`KubeClusterControl`]'s override actually observes
    /// an in-progress rollout; every test double that does not override this
    /// keeps its prior "instantly converged" behavior.
    async fn serving_topology_converged(
        &self,
        _namespace: &str,
        _name: &str,
        _desired_replicas: i64,
    ) -> Result<bool> {
        Ok(true)
    }

    /// #1467 R5: whether every serving pod (`0..shard_count`, one pod per
    /// shard — the reshard driver's admin plane already assumes
    /// `replicas_per_shard <= 1` in the routed topology it operates over,
    /// same as [`Self::shard_base_url`]) reports `lumen_shard_map_version
    /// == map_version` on its `/metrics` endpoint. [`Self::
    /// serving_topology_converged`] alone only proves the StatefulSet
    /// rollout *finished* (every pod `Ready` on the latest pod template) —
    /// not that each pod's process actually holds `map_version`, since the
    /// shard map itself is read from a ConfigMap the pod loads at startup,
    /// and a ConfigMap write racing a rollout's pod-recreate order is not
    /// something StatefulSet status observes at all. Defaults to `Ok(true)`
    /// for the same test-seam reason as `serving_topology_converged` —
    /// every test double that does not override this keeps its prior
    /// "instantly converged" behavior.
    async fn serving_pods_report_map_version(
        &self,
        _http: &reqwest::Client,
        _namespace: &str,
        _name: &str,
        _shard_count: u32,
        _map_version: u64,
    ) -> Result<bool> {
        Ok(true)
    }
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

    async fn serving_topology_converged(
        &self,
        namespace: &str,
        name: &str,
        desired_replicas: i64,
    ) -> Result<bool> {
        let ar = statefulset_api_resource();
        let api: Api<DynamicObject> = Api::namespaced_with(self.client.clone(), namespace, &ar);
        let Some(sts) = api
            .get_opt(name)
            .await
            .context("read serving StatefulSet for topology convergence")?
        else {
            // No StatefulSet yet is not "converged" — the caller keeps
            // treating this as pending rather than assuming success.
            return Ok(false);
        };
        let status = &sts.data["status"];
        let ready_replicas = status["readyReplicas"].as_i64().unwrap_or(0);
        let updated_replicas = status["updatedReplicas"].as_i64().unwrap_or(0);
        // A rollout still in flight has distinct current/update revisions;
        // once it completes, k8s converges them onto the same value. Absent
        // fields (any StatefulSet old enough not to report them) fail this
        // check open on the safe side — never assumed identical.
        let current_revision = status["currentRevision"].as_str();
        let update_revision = status["updateRevision"].as_str();
        let revisions_converged =
            matches!((current_revision, update_revision), (Some(c), Some(u)) if c == u);
        Ok(revisions_converged
            && ready_replicas >= desired_replicas
            && updated_replicas >= desired_replicas)
    }

    async fn serving_pods_report_map_version(
        &self,
        http: &reqwest::Client,
        namespace: &str,
        name: &str,
        shard_count: u32,
        map_version: u64,
    ) -> Result<bool> {
        for shard in 0..shard_count {
            let url = format!("{}/metrics", self.shard_base_url(namespace, name, shard));
            // An unreachable pod (mid-rollout, mid-restart) or a decode
            // failure is "not converged yet", not an error — the caller
            // just keeps the fence armed and retries next tick, exactly
            // like an unready StatefulSet replica.
            let Ok(resp) = http.get(&url).send().await else {
                return Ok(false);
            };
            if !resp.status().is_success() {
                return Ok(false);
            }
            let Ok(body) = resp.text().await else {
                return Ok(false);
            };
            if super::reconcile::parse_metric(&body, "lumen_shard_map_version") != Some(map_version)
            {
                return Ok(false);
            }
        }
        Ok(true)
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
    /// #1458 R1: `Complete`, but not every serving pod is confirmed `Ready`
    /// on `map_version` yet — the write-pause fence over the buckets that
    /// moved into `map_version` was re-armed this tick, and
    /// `awaitingTopologyConvergence` should surface in `status.reshard`.
    AwaitingTopologyConvergence { map_version: u64 },
    /// #1458 R1: `Complete`, and every serving pod just got confirmed
    /// `Ready` on `map_version` — `workflow.convergedShardMapVersion` was
    /// patched to `map_version` and the write-pause fence was cleared.
    TopologyConverged { map_version: u64 },
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

/// `GET /collections` (#1457 R2): the full list of collections that exist on
/// this shard right now, independent of any bucket scope. The reshard
/// driver's admin token carries wildcard `Role::Admin` on `"*"`, which
/// already satisfies this data-plane route's per-collection `Role::Read`
/// filter for every collection id, so no new admin-only endpoint is needed
/// here. This is deliberately **not** derived from a bucket-scoped
/// snapshot's own `collections` keys: [`crate::reshard::snapshot_bucket_subset`]
/// (backing `POST /admin/backup:scoped`) omits a collection entirely from
/// its output when it has zero matching docs in the requested buckets — a
/// collection a batch of deletes emptied out of a moved bucket would then be
/// silently skipped by [`snapshot_reshard_prune_chunks`], leaving its stale
/// copies on the target unpruned (the exact edge #1443 disclosed and #1457
/// R2 closes).
async fn fetch_all_collection_ids(
    http: &reqwest::Client,
    base_url: &str,
    token: Option<&str>,
) -> Result<BTreeSet<String>> {
    let mut req = http.get(format!("{base_url}/collections"));
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let resp = req
        .send()
        .await
        .with_context(|| format!("GET {base_url}/collections"))?;
    if !resp.status().is_success() {
        bail!("{base_url}/collections returned {}", resp.status());
    }
    let ids: Vec<String> = resp
        .json()
        .await
        .with_context(|| format!("decode {base_url}/collections response"))?;
    Ok(ids.into_iter().collect())
}

/// If `batch`'s actual wire payload is over
/// [`crate::reshard::ADMIN_ROUTE_BODY_LIMIT_BYTES`], name the collection and
/// external_id to blame (#1444 R2). `snapshot_reshard_batches`'
/// `byte_cap_chunk` only ever emits an over-the-limit batch when it floored
/// at a single external_id (a bucket group's byte cap already keeps every
/// multi-id batch under half the route limit), so the first id found in
/// `external_ids` is that one document.
fn detect_oversized_batch(batch: &ReshardBatch) -> Option<OversizedDocumentBlock> {
    let bytes = serde_json::to_vec(batch)
        .map(|bytes| bytes.len())
        .unwrap_or(usize::MAX);
    if bytes <= crate::reshard::ADMIN_ROUTE_BODY_LIMIT_BYTES {
        return None;
    }
    let (collection, external_id) = batch.external_ids.iter().find_map(|(collection, ids)| {
        ids.iter()
            .next()
            .map(|external_id| (collection.clone(), external_id.clone()))
    })?;
    Some(OversizedDocumentBlock {
        collection,
        external_id,
        bytes,
    })
}

async fn apply_reshard_batch(
    http: &reqwest::Client,
    base_url: &str,
    token: Option<&str>,
    batch: &ReshardBatch,
) -> Result<()> {
    // Pre-flight (#1444 R2): a batch this crate can already tell is over the
    // route's body limit is skipped rather than sent — no wasted round trip,
    // and the classification never depends on how a given HTTP stack renders
    // its own 413.
    if let Some(oversized) = detect_oversized_batch(batch) {
        return Err(oversized.into());
    }
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
        // Defense in depth: even if the pre-flight estimate above missed it
        // (e.g. framing/compression skew), classify a live 413 on this exact
        // batch shape the same way rather than a generic Blocked message.
        if resp.status() == reqwest::StatusCode::PAYLOAD_TOO_LARGE {
            if let Some(oversized) = detect_oversized_batch(batch) {
                return Err(oversized.into());
            }
        }
        bail!("{base_url}/admin/reshard:apply returned {}", resp.status());
    }
    Ok(())
}

/// `POST /admin/reshard:prune` (#1457 R1): send one [`ReshardPruneChunk`] of
/// the final migration pass's authoritative keep set. Unlike
/// [`apply_reshard_batch`], a chunk carries only external_id strings (no
/// document content), so it never needs the same pre-flight/live-413
/// oversize classification — `snapshot_reshard_prune_chunks`'s recursive
/// byte-cap halving already keeps every chunk under `max_chunk_bytes` short
/// of a single id long enough alone to exceed it, an unrealistic edge this
/// function does not special-case. A failure here
/// propagates as a generic error, surfaced by every caller as
/// [`DriveOutcome::Blocked`] the same as any other step; the next tick's
/// retry recomputes and re-sends the same deterministic chunk set (the final
/// pass runs under the write fence, so bucket population cannot change
/// between ticks), converging via [`crate::storage::Engine::
/// apply_reshard_prune_chunk`]'s idempotent accumulator.
async fn apply_reshard_prune_chunk(
    http: &reqwest::Client,
    base_url: &str,
    token: Option<&str>,
    chunk: &ReshardPruneChunk,
) -> Result<()> {
    let mut req = http
        .post(format!("{base_url}/admin/reshard:prune"))
        .json(chunk);
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let resp = req
        .send()
        .await
        .with_context(|| format!("POST {base_url}/admin/reshard:prune"))?;
    if !resp.status().is_success() {
        bail!("{base_url}/admin/reshard:prune returned {}", resp.status());
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
///
/// #1443 R4: arming loops over shards sequentially and can fail partway
/// through (one shard unreachable). A failure used to return immediately via
/// `?`, leaving every shard armed *before* the failing one fenced with no
/// caller ever reaching the clear bracket — an indefinite intermittent write
/// outage on those shards (re-armed every tick) even though the migration
/// made zero progress. Now tracks which shards actually armed and, on
/// failure, best-effort clears exactly those before surfacing the original
/// error, so a partial arm never outlives this call.
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
    let mut armed_urls: Vec<String> = Vec::new();
    for shard in 0..current.physical_shard_count() {
        let url = control.shard_base_url(namespace, name, shard);
        if let Err(err) = reshard_fence_call(
            http,
            &url,
            token.as_deref(),
            current.virtual_bucket_count(),
            buckets,
            ttl_secs,
        )
        .await
        {
            if !buckets.is_empty() {
                for armed_url in &armed_urls {
                    if let Err(clear_err) = reshard_fence_call(
                        http,
                        armed_url,
                        token.as_deref(),
                        current.virtual_bucket_count(),
                        &BTreeSet::new(),
                        0,
                    )
                    .await
                    {
                        tracing::warn!(
                            shard_url = %armed_url,
                            error = %clear_err,
                            "reshard driver: best-effort fence clear after a partial arm \
                             failure also failed; this shard stays fenced until its own TTL \
                             expires"
                        );
                    }
                }
            }
            return Err(err);
        }
        if !buckets.is_empty() {
            armed_urls.push(url);
        }
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
///
/// `moving_buckets`/`last_armed_at` (#1458 R3) thread the same
/// [`maybe_rearm_fence`] time-based re-arm into this loop: a real cutover
/// can checkpoint many source shards sequentially, and the caller's
/// unconditional phase-boundary re-arm (immediately before this call) only
/// covers the moment this loop starts, not however long the loop itself
/// takes.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
async fn checkpoint_shards(
    control: &dyn ClusterControl,
    http: &reqwest::Client,
    namespace: &str,
    name: &str,
    lumen: &Lumen,
    shards: impl Iterator<Item = u32>,
    current: &VirtualBucketShardMap,
    moving_buckets: Option<&BTreeSet<u32>>,
    last_armed_at: &mut Instant,
) -> Result<()> {
    let token = control.admin_token(namespace, lumen).await?;
    for shard in shards {
        maybe_rearm_fence(
            control,
            http,
            namespace,
            name,
            lumen,
            current,
            moving_buckets,
            last_armed_at,
        )
        .await?;
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

/// Every virtual bucket currently assigned to `map`'s highest-index
/// (newest) physical shard (#1458 R1). [`VirtualBucketShardMap::
/// split_one_shard`] only ever moves a bucket directly into the new shard
/// it appends — never between two pre-existing shards — so immediately
/// after a cutover to `map`, this is exactly the set of buckets that just
/// moved, recoverable purely from the already-persisted `spec.shardMap`
/// with no separate bookkeeping. [`advance_convergence`] re-fences this same
/// set every tick until every serving pod is confirmed Ready on `map`.
fn buckets_on_newest_shard(map: &VirtualBucketShardMap) -> BTreeSet<u32> {
    let newest = map.physical_shard_count().saturating_sub(1);
    (0..map.virtual_bucket_count())
        .filter(|&bucket| map.assignment_for_bucket(bucket) == Some(newest))
        .collect()
}

/// #1458 R3: re-arm the write fence once more than
/// `write_fence_ttl_secs() / FENCE_REARM_FRACTION` has elapsed since the
/// last arm. Replaces the earlier fixed-count re-arm (#1443 R1, every
/// `FENCE_REARM_BATCH_INTERVAL = 20` applied batches/chunks): a count-based
/// clock can still be outrun by a sequence whose batches are individually
/// slow (a large scoped-backup fetch, a slow network, or a handful of huge
/// byte-capped batches) even though few *batches* have been applied — a
/// time-based clock, checked between every batch/chunk apply and around
/// each fetch/prune step, cannot.
const FENCE_REARM_FRACTION: u32 = 4;

/// Re-arm the write-pause fence if more than `write_fence_ttl_secs() /
/// [`FENCE_REARM_FRACTION`]` has elapsed since `*last_armed_at` (#1458 R3).
/// No-op — and leaves `*last_armed_at` untouched — when `moving_buckets` is
/// `None`/empty (this pass/step is not running under a fence) or the
/// fraction has not yet elapsed. A re-arm failure propagates as `Err`,
/// which every caller already surfaces as [`DriveOutcome::Blocked`] before
/// eviction ever runs (R3's "a failed re-arm still aborts to `Blocked`
/// before eviction").
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
async fn maybe_rearm_fence(
    control: &dyn ClusterControl,
    http: &reqwest::Client,
    namespace: &str,
    name: &str,
    lumen: &Lumen,
    current: &VirtualBucketShardMap,
    moving_buckets: Option<&BTreeSet<u32>>,
    last_armed_at: &mut Instant,
) -> Result<()> {
    let Some(buckets) = moving_buckets.filter(|b| !b.is_empty()) else {
        return Ok(());
    };
    let ttl_secs = control.write_fence_ttl_secs();
    let rearm_after = Duration::from_secs(ttl_secs) / FENCE_REARM_FRACTION;
    if last_armed_at.elapsed() < rearm_after {
        return Ok(());
    }
    set_write_fence(
        control, http, namespace, name, lumen, current, buckets, ttl_secs,
    )
    .await
    .context("re-arm write fence mid migration pass")?;
    *last_armed_at = Instant::now();
    Ok(())
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
    run_migration_pass_impl(control, http, namespace, name, lumen, false, None).await
}

/// Shared migration-pass implementation. `final_pass` is `true` only for the
/// final, fenced `CatchingUp` pass: every `snapshot_reshard_batches` apply
/// below stays purely additive regardless (#1457 R1), but a `final_pass`
/// additionally sends the authoritative-replace scope for every moved bucket
/// via `POST /admin/reshard:prune` — see [`snapshot_reshard_prune_chunks`].
/// `moving_buckets` (#1443 R1), when `Some`, marks this pass as running
/// under a write fence and re-arms it with a fresh
/// [`ClusterControl::write_fence_ttl_secs`] deadline via [`maybe_rearm_fence`]
/// (#1458 R3: time-based, checked around every fetch/apply/prune step, not
/// a fixed applied-batch count) — a re-arm failure aborts the whole pass
/// immediately (propagated as `Err`, which every caller already surfaces as
/// `DriveOutcome::Blocked` before eviction ever runs).
async fn run_migration_pass_impl(
    control: &dyn ClusterControl,
    http: &reqwest::Client,
    namespace: &str,
    name: &str,
    lumen: &Lumen,
    final_pass: bool,
    moving_buckets: Option<&BTreeSet<u32>>,
) -> Result<usize> {
    let current = current_shard_map(lumen)?;
    let target = compute_target_map(&current)?;
    let moves = bucket_moves(&current, &target)?;
    if moves.is_empty() {
        return Ok(0);
    }

    let mut buckets_by_from_shard: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    let mut to_shard_by_bucket: BTreeMap<u32, u32> = BTreeMap::new();
    for mv in &moves {
        buckets_by_from_shard
            .entry(mv.from_shard)
            .or_default()
            .insert(mv.bucket);
        to_shard_by_bucket.insert(mv.bucket, mv.to_shard);
    }

    let token = control.admin_token(namespace, lumen).await?;
    let mut total_batches = 0usize;
    // #1458 R3: the fence was armed by the caller immediately before this
    // call, so `Instant::now()` here is that arm's timestamp — the clock
    // [`maybe_rearm_fence`] measures elapsed time against at every
    // fetch/apply/prune step below, independent of which of the two loops
    // is currently running or how many calls each has made.
    let mut last_armed_at = Instant::now();
    for (from_shard, buckets) in buckets_by_from_shard {
        maybe_rearm_fence(
            control,
            http,
            namespace,
            name,
            lumen,
            &current,
            moving_buckets,
            &mut last_armed_at,
        )
        .await?;
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
            if let Err(err) = apply_reshard_batch(http, &dest_url, token.as_deref(), batch).await {
                // #1444 R2: record the wedge distinctly before propagating, so
                // callers that turn this `Err` into `DriveOutcome::Blocked`
                // still leave a structured trace behind for `status.reshard`
                // and for `advance_catching_up`'s fence-skip check, even
                // though the `Err` itself stays a generic message.
                if let Some(oversized) = err.downcast_ref::<OversizedDocumentBlock>() {
                    record_oversize_block(
                        namespace,
                        name,
                        &lumen.uid().unwrap_or_default(),
                        oversized.clone(),
                    );
                }
                return Err(err);
            }
            total_batches += 1;
            maybe_rearm_fence(
                control,
                http,
                namespace,
                name,
                lumen,
                &current,
                moving_buckets,
                &mut last_armed_at,
            )
            .await?;
        }

        // #1457 R1/R2: the final pass's authoritative-replace scope, sent as
        // its own independently byte-capped `POST /admin/reshard:prune`
        // chunks rather than stamped onto every `ReshardBatch` above — see
        // `reshard.rs`'s `ReshardBatch`/`ReshardPruneChunk` docs for why. The
        // full collection list is fetched from the source shard directly
        // (#1457 R2) rather than derived from `snapshot`'s own keys, so a
        // collection a batch of deletes emptied out of these buckets still
        // gets an (empty) keep scope instead of being silently skipped.
        if final_pass {
            let collection_ids = fetch_all_collection_ids(http, &source_url, token.as_deref())
                .await
                .context("fetch source shard's full collection list for the final reshard pass")?;
            maybe_rearm_fence(
                control,
                http,
                namespace,
                name,
                lumen,
                &current,
                moving_buckets,
                &mut last_armed_at,
            )
            .await?;
            let prune_chunks = snapshot_reshard_prune_chunks(
                &snapshot,
                &target,
                &buckets,
                &collection_ids,
                crate::reshard::MAX_BATCH_BYTES,
            )?;
            for chunk in &prune_chunks {
                let Some(&to_shard) = to_shard_by_bucket.get(&chunk.bucket) else {
                    bail!(
                        "prune chunk for bucket {} has no known destination shard",
                        chunk.bucket
                    );
                };
                let dest_url = control.shard_base_url(namespace, name, to_shard);
                apply_reshard_prune_chunk(http, &dest_url, token.as_deref(), chunk).await?;
                maybe_rearm_fence(
                    control,
                    http,
                    namespace,
                    name,
                    lumen,
                    &current,
                    moving_buckets,
                    &mut last_armed_at,
                )
                .await?;
            }
        }
    }
    // A full pass completed without hitting the oversize wedge (whether or
    // not one was ever recorded) — clear any stale block so a fixed document
    // doesn't leave `status.reshard` reporting a condition that no longer
    // applies.
    clear_oversize_block(namespace, name);
    Ok(total_batches)
}

/// Post-cutover eviction (idempotent, #1380) on every **old** shard, using
/// only the already-committed target map — the driver never needs to retain
/// the old map across a restart.
///
/// `moving_buckets`/`last_armed_at` (#1467 R3) thread the same
/// [`maybe_rearm_fence`] time-based re-arm [`checkpoint_shards`] already
/// has into this loop: eviction round-trips one HTTP call per **old**
/// physical shard, and a slow round (many old shards, a slow network) could
/// otherwise outlive the fence TTL mid-eviction with no re-arm to catch it
/// — the caller's unconditional phase-boundary re-arm immediately before
/// this call only covers the moment the loop starts.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
async fn evict_old_shards(
    control: &dyn ClusterControl,
    http: &reqwest::Client,
    namespace: &str,
    name: &str,
    lumen: &Lumen,
    current: &VirtualBucketShardMap,
    target: &VirtualBucketShardMap,
    moving_buckets: Option<&BTreeSet<u32>>,
    last_armed_at: &mut Instant,
) -> Result<()> {
    let token = control.admin_token(namespace, lumen).await?;
    let assignments = map_assignments(target);
    for shard in 0..current.physical_shard_count() {
        maybe_rearm_fence(
            control,
            http,
            namespace,
            name,
            lumen,
            current,
            moving_buckets,
            last_armed_at,
        )
        .await?;
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
/// that bucket's eviction and is silently dropped. See
/// [`crate::api::WriteFence`] for why a crashed driver can never leave it
/// armed permanently.
///
/// The fence is cleared immediately on every exit path *except*
/// [`DriveOutcome::CompletedSplit`] (#1442 R2): a completed split just
/// called [`ClusterControl::trigger_rolling_restart`], and pods only read
/// `SHARD_MAP_*`/`SHARD_COUNT` env at boot, so old-map pods keep serving
/// (and, without this, keep accepting local writes for) the just-evicted
/// source buckets until the rolling restart actually reaches them —
/// clearing the fence right after triggering the restart would open exactly
/// that mixed-map window back up. Leaving it armed here lets
/// [`WRITE_FENCE_TTL_SECS`] bound the window instead (the design's simpler,
/// non-blocking alternative to synchronously polling every serving pod
/// Ready on the new topology from inside one CR's tick, which would stall
/// `drive_tick`'s other CRs); the next split for this CR can only start once
/// `drive_tick` sees the phase back at `Complete`, well after the TTL, so
/// there is no risk of a subsequent tick trying to arm a fence that is
/// already armed from a prior split.
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

    // #1444 R2: a tick already known-wedged on an oversized single-document
    // batch is a permanent no-progress condition until the document shrinks
    // (see [`OversizedDocumentBlock`]) — arming the write fence anyway would
    // pause writes to these buckets for a pass that cannot possibly finish,
    // recurring every tick's `WRITE_FENCE_TTL_SECS` window for no benefit.
    // `should_skip_for_oversize` still periodically lets a real attempt
    // through (`OVERSIZE_RECHECK_TICKS`) so a fixed document self-heals.
    if let Some(block) = should_skip_for_oversize(namespace, name, &lumen.uid().unwrap_or_default())
    {
        return DriveOutcome::Blocked(block.to_string());
    }

    if !moving_buckets.is_empty() {
        if let Err(err) = set_write_fence(
            control,
            http,
            namespace,
            name,
            lumen,
            &current,
            &moving_buckets,
            control.write_fence_ttl_secs(),
        )
        .await
        {
            return DriveOutcome::Blocked(err.to_string());
        }
    }

    let outcome = advance_catching_up_fenced(
        control,
        http,
        namespace,
        name,
        lumen,
        &current,
        &target,
        &moving_buckets,
    )
    .await;

    let completed_split = matches!(outcome, DriveOutcome::CompletedSplit { .. });
    if !moving_buckets.is_empty() && !completed_split {
        // Clear on every exit path except a completed split (#1442 R2, see
        // this fn's doc comment): the fence must not outlive this tick
        // *unless* the cutover it guarded just triggered a rolling restart,
        // in which case leaving it armed and TTL-bounded closes the
        // mixed-map window instead of reopening it. If this clear itself
        // fails (or the process dies before reaching it), WRITE_FENCE_TTL_SECS
        // still bounds how long writes to these buckets stay paused — the
        // serving pod enforces that deadline on its own, independent of the
        // driver ever coming back.
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
    } else if !moving_buckets.is_empty() && completed_split {
        tracing::info!(
            "reshard driver: split completed and rolling restart triggered; leaving write fence \
             armed for WRITE_FENCE_TTL_SECS={WRITE_FENCE_TTL_SECS}s to close the old-map pods' \
             mixed-map window instead of clearing it immediately (#1442 R2)"
        );
    }

    outcome
}

/// The migrate/checkpoint/evict/checkpoint/cutover sequence proper, run
/// under [`advance_catching_up`]'s write fence. Split out so the fence's
/// arm/clear bracket is unconditional (always runs, regardless of which
/// step below fails) without duplicating the sequence itself.
///
/// #1443 R1: this sequence's migration pass is `final_pass` (R2, reworked
/// #1457 R1 into a separate `POST /admin/reshard:prune` step) and runs
/// under time-based in-loop re-arming (#1458 R3); the fence is additionally
/// re-armed with a fresh TTL at every phase boundary below (after the
/// migration pass and before the target checkpoint, again before eviction,
/// and again before the sources' checkpoint round) so a real pass —
/// hundreds of sequential batch/checkpoint HTTP round trips — can never
/// silently outlive [`ClusterControl::write_fence_ttl_secs`] mid-sequence.
/// [`checkpoint_shards`] itself also re-arms on the same TTL/4 clock between
/// individual shard checkpoints (#1458 R3), so a slow multi-shard
/// checkpoint round is covered too, not just the boundary before it. Any
/// re-arm failure aborts to [`DriveOutcome::Blocked`] immediately, always
/// strictly before [`evict_old_shards`] runs.
async fn advance_catching_up_fenced(
    control: &dyn ClusterControl,
    http: &reqwest::Client,
    namespace: &str,
    name: &str,
    lumen: &Lumen,
    current: &VirtualBucketShardMap,
    target: &VirtualBucketShardMap,
    moving_buckets: &BTreeSet<u32>,
) -> DriveOutcome {
    if let Err(err) = run_migration_pass_impl(
        control,
        http,
        namespace,
        name,
        lumen,
        true,
        Some(moving_buckets),
    )
    .await
    {
        return DriveOutcome::Blocked(err.to_string());
    }

    // #1458 R3: re-armed (unconditionally) at every phase boundary below;
    // reset alongside each one so `checkpoint_shards`' own in-loop
    // time-based re-arm measures elapsed time from the boundary that
    // actually just armed the fence, not from this sequence's start.
    let mut last_armed_at = Instant::now();
    let fence_buckets = (!moving_buckets.is_empty()).then_some(moving_buckets);

    if !moving_buckets.is_empty() {
        if let Err(err) = set_write_fence(
            control,
            http,
            namespace,
            name,
            lumen,
            current,
            moving_buckets,
            control.write_fence_ttl_secs(),
        )
        .await
        {
            return DriveOutcome::Blocked(format!(
                "re-arm write fence before target checkpoint: {err}"
            ));
        }
        last_armed_at = Instant::now();
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
        current,
        fence_buckets,
        &mut last_armed_at,
    )
    .await
    {
        return DriveOutcome::Blocked(err.to_string());
    }

    if !moving_buckets.is_empty() {
        if let Err(err) = set_write_fence(
            control,
            http,
            namespace,
            name,
            lumen,
            current,
            moving_buckets,
            control.write_fence_ttl_secs(),
        )
        .await
        {
            return DriveOutcome::Blocked(format!("re-arm write fence before eviction: {err}"));
        }
        last_armed_at = Instant::now();
    }

    if let Err(err) = evict_old_shards(
        control,
        http,
        namespace,
        name,
        lumen,
        current,
        target,
        fence_buckets,
        &mut last_armed_at,
    )
    .await
    {
        return DriveOutcome::Blocked(err.to_string());
    }

    if !moving_buckets.is_empty() {
        if let Err(err) = set_write_fence(
            control,
            http,
            namespace,
            name,
            lumen,
            current,
            moving_buckets,
            control.write_fence_ttl_secs(),
        )
        .await
        {
            return DriveOutcome::Blocked(format!(
                "re-arm write fence before sources' checkpoint round: {err}"
            ));
        }
        last_armed_at = Instant::now();
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
        current,
        fence_buckets,
        &mut last_armed_at,
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
                    // #1467 R7: stamped in the SAME patch as `shardMap.
                    // version` — proof this cutover, and not a hand-authored
                    // or restored `shardMap`, is what produced this map
                    // version, gating `advance_convergence`'s engagement.
                    "lastCutoverShardMapVersion": target.version(),
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

/// #1458 R1: `Complete`-phase convergence step, checked ahead of
/// [`should_start_split`] so a CR is never allowed to start a *new* split
/// while a prior one's serving pods have not all confirmed `Ready` on the
/// map that prior split cutover to. Keeps the write-pause fence armed over
/// [`buckets_on_newest_shard`] — the buckets that moved into the current
/// `spec.shardMap` — re-arming it every tick this returns `Some(
/// AwaitingTopologyConvergence)`, until [`ClusterControl::
/// serving_topology_converged`] confirms every serving pod is `Ready` on
/// the new topology, at which point the fence is cleared and
/// `workflow.convergedShardMapVersion` is patched to the converged version.
///
/// "Converging" is derived purely from persisted state (`spec.shardMap.
/// version` compared against `workflow.convergedShardMapVersion`), not
/// driver memory, so this resumes correctly across a driver restart:
/// [`buckets_on_newest_shard`] recomputes the exact same bucket set from
/// `spec.shardMap` alone (see that function's doc for the invariant this
/// relies on), and [`ClusterControl::serving_topology_converged`] reuses
/// the same StatefulSet readiness plumbing [`advance_prepare_split`]
/// already polls rather than adding a new seam.
///
/// This replaces #1442 R2's "leave the fence armed once for a fixed
/// [`WRITE_FENCE_TTL_SECS`]" behavior: a slow rolling restart across many
/// pods could outlive that single fixed TTL, silently reopening the
/// mixed-map write-loss window the fence exists to close.
///
/// Returns `None` when convergence is not pending — either `shard_map.
/// version == 0` (a CR that has never resharded; no cutover has ever run
/// to converge from), the current version is already recorded converged, or
/// (#1467 R7) `workflow.lastCutoverShardMapVersion` does not equal the
/// current `shard_map.version` — letting the caller fall through to
/// [`should_start_split`].
///
/// #1467 R7: the `lastCutoverShardMapVersion` check is what keeps this
/// function from ever engaging the write-pause fence for a CR whose
/// `spec.shardMap` was hand-authored (or restored from a backup/migration)
/// rather than reached via a cutover this driver actually ran —
/// `advance_catching_up_fenced`'s cutover patch is the ONLY writer of
/// `lastCutoverShardMapVersion`, and it always sets it to the exact
/// `target.version()` it patches into `shardMap.version` in the same call,
/// so the two fields are equal immediately after every real cutover. A
/// manually-set `shardMap.version` therefore leaves
/// `lastCutoverShardMapVersion` unequal (usually `None`) forever, and this
/// function never engages for it — closing the gap where convergence would
/// otherwise fence indefinitely over a topology the driver never actually
/// changed.
async fn advance_convergence(
    control: &dyn ClusterControl,
    http: &reqwest::Client,
    namespace: &str,
    name: &str,
    lumen: &Lumen,
) -> Option<DriveOutcome> {
    let map_version = lumen.spec.shard_map.version;
    let workflow = &lumen.spec.reshard_policy.workflow;
    if map_version == 0
        || workflow.converged_shard_map_version == Some(map_version)
        || workflow.last_cutover_shard_map_version != Some(map_version)
    {
        clear_convergence_stall(namespace, name);
        return None;
    }

    let current = match current_shard_map(lumen) {
        Ok(m) => m,
        Err(err) => return Some(DriveOutcome::Blocked(err.to_string())),
    };
    let moving_buckets = buckets_on_newest_shard(&current);
    let desired_replicas = lumen.spec.storage_pod_count() as i64;

    let rollout_converged = match control
        .serving_topology_converged(namespace, name, desired_replicas)
        .await
    {
        Ok(converged) => converged,
        Err(err) => return Some(DriveOutcome::Blocked(err.to_string())),
    };

    // #1467 R5: StatefulSet rollout completion alone doesn't prove every
    // serving pod actually holds the new shard map — it only proves the
    // pod template/generation converged. Require every pod to also report
    // the new map version on its own `/metrics` before treating topology
    // as converged. Gated behind `rollout_converged` so we don't scrape
    // every shard on every tick while a rollout is still in flight.
    let converged = if rollout_converged {
        match control
            .serving_pods_report_map_version(
                http,
                namespace,
                name,
                current.physical_shard_count(),
                map_version,
            )
            .await
        {
            Ok(reported) => reported,
            Err(err) => return Some(DriveOutcome::Blocked(err.to_string())),
        }
    } else {
        false
    };

    if converged {
        // #1467 R7: convergence resolved — clear the stall tracker so a
        // future, unrelated wait (a later split's own convergence) starts
        // from a fresh budget instead of inheriting this one's tick count.
        //
        // #1467 R5: once every serving pod has been observed reporting
        // `map_version` on `/metrics`, the fence is cleared below. A
        // *subsequent* rollout that only changes the pod template (image,
        // resources, env — not the shard map) is safe by construction:
        // every pod already holds `map_version` before that rollout
        // starts, so no re-arm or re-verification is needed for it. Only a
        // *new* cutover (which bumps `shardMap.version` again and re-stamps
        // `lastCutoverShardMapVersion`) re-engages this convergence gate.
        clear_convergence_stall(namespace, name);
        if !moving_buckets.is_empty() {
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
                    "reshard driver: failed to clear write fence after topology convergence; \
                     bounded by WRITE_FENCE_TTL_SECS"
                );
            }
        }
        let patch = json!({
            "spec": {
                "reshardPolicy": {
                    "workflow": {
                        "convergedShardMapVersion": map_version,
                        // #1485 R1/R2: episode resolved — clear the durable
                        // wait-start/remediation bookkeeping in the SAME
                        // patch so a future, unrelated wait (a later
                        // split's own convergence) starts from a fresh
                        // budget and a fresh one-shot remediation slot,
                        // instead of inheriting this episode's state.
                        "convergenceWaitStartedAt": null,
                        "convergenceRemediationRestartCount": 0,
                        "convergenceRemediationRestartedAt": null,
                    }
                }
            }
        });
        if let Err(err) = control.patch_spec(namespace, name, patch).await {
            return Some(DriveOutcome::Blocked(err.to_string()));
        }
        return Some(DriveOutcome::TopologyConverged { map_version });
    }

    // #1467 R7: bounded escalation — bump this map_version's
    // consecutive-awaiting-ticks counter. This in-process cache stays as a
    // fast-path/logging-only signal (#1485 R2); it is no longer what decides
    // whether the budget is exceeded (see below).
    record_convergence_await(
        namespace,
        name,
        &lumen.uid().unwrap_or_default(),
        map_version,
    );

    // #1485 R2: the durable wait-start checkpoint. Stamped once, on the
    // first tick this map_version is observed unconverged — every later
    // tick (including after an operator restart, when the in-process cache
    // above is empty again) reads the SAME persisted value back off `lumen`,
    // so the elapsed-time budget below is computed identically regardless of
    // driver process lifetime.
    let now = now_epoch_secs();
    let wait_started_at = workflow.convergence_wait_started_at;
    if wait_started_at.is_none() {
        let patch = json!({
            "spec": {
                "reshardPolicy": {
                    "workflow": {
                        "convergenceWaitStartedAt": now,
                    }
                }
            }
        });
        if let Err(err) = control.patch_spec(namespace, name, patch).await {
            return Some(DriveOutcome::Blocked(format!(
                "persist convergence-wait start: {err}"
            )));
        }
    }
    // `wait_started_at.or(Some(now))`: on this very first tick the patch
    // above just persisted `now`, but `lumen` itself (this tick's snapshot)
    // still predates it — treat this tick as freshly started (elapsed 0),
    // exactly like the pre-#1485 tick-count budget did.
    let stalled = convergence_stall_condition(wait_started_at.or(Some(now)));
    if stalled {
        tracing::warn!(
            namespace,
            name,
            map_version,
            "reshard driver: topology convergence has not been confirmed after \
             CONVERGENCE_STALL_SECS; fence stays armed, raising topologyConvergenceStalled"
        );
    }

    // #1485 R1: bounded remediation restart. The ConfigMap-race signature is
    // exactly what this branch already establishes above: the StatefulSet
    // rollout itself is done (`rollout_converged`) but at least one pod is
    // still reporting the old shard-map version (`!converged`, this
    // function's outer `if converged` already returned). Bounded to exactly
    // one re-trigger per episode via `convergenceRemediationRestartCount`
    // (persisted, so a driver restart never re-triggers a second time for
    // the same episode) — the fence stays armed and `stalled` stays raised
    // either way; this only attempts a self-heal, it never changes whether
    // the wait keeps being reported.
    if stalled && rollout_converged && workflow.convergence_remediation_restart_count == 0 {
        tracing::warn!(
            namespace,
            name,
            map_version,
            "reshard driver: convergence stalled on a version mismatch (rollout complete, pod(s) \
             still on the old shard-map version); triggering one bounded remediation rolling \
             restart"
        );
        if let Err(err) = control.trigger_rolling_restart(namespace, name).await {
            // Non-fatal, matching the cutover-tick trigger's own handling —
            // the re-trigger attempt is still bounded to one per episode
            // below regardless of whether k8s actually accepted it; a
            // repeatedly-failing rolling-restart trigger is a cluster-level
            // problem the stalled condition already surfaces.
            tracing::warn!(error = %err, "reshard driver: convergence remediation rolling-restart trigger failed");
        }
        let patch = json!({
            "spec": {
                "reshardPolicy": {
                    "workflow": {
                        "convergenceRemediationRestartCount": 1,
                        "convergenceRemediationRestartedAt": now,
                    }
                }
            }
        });
        if let Err(err) = control.patch_spec(namespace, name, patch).await {
            return Some(DriveOutcome::Blocked(format!(
                "persist convergence remediation re-trigger: {err}"
            )));
        }
    }

    if !moving_buckets.is_empty() {
        if let Err(err) = set_write_fence(
            control,
            http,
            namespace,
            name,
            lumen,
            &current,
            &moving_buckets,
            control.write_fence_ttl_secs(),
        )
        .await
        {
            return Some(DriveOutcome::Blocked(format!(
                "re-arm write fence while awaiting topology convergence: {err}"
            )));
        }
    }
    Some(DriveOutcome::AwaitingTopologyConvergence { map_version })
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
            // #1458 R4: a workflow back at `Complete` has no legitimate
            // oversize wedge left to report — clear defensively (idempotent
            // if already clear from `run_migration_pass_impl`'s own
            // end-of-pass clear) so a manually-forced phase reset never
            // leaves a stale condition behind.
            clear_oversize_block(&namespace, &name);
            if let Some(outcome) =
                advance_convergence(control, http, &namespace, &name, lumen).await
            {
                outcome
            } else if should_start_split(lumen) {
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
                        // #1458 R4: this list is already the authoritative
                        // live-CR set, so pruning stale oversize-cache
                        // entries here needs no extra k8s API call.
                        let live_uids: BTreeSet<String> =
                            list.items.iter().filter_map(|l| l.uid()).collect();
                        prune_oversize_cache(&live_uids);
                        // #1467 R7: same already-listed live-CR set bounds
                        // the convergence-stall cache too.
                        prune_convergence_stall_cache(&live_uids);
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
            ..Default::default()
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

    // ---- #1443 AC4: set_write_fence partial-arm cleanup -----------------

    /// Minimal control exposing exactly the shard URLs [`set_write_fence`]
    /// needs; used only by the AC4 test below, which calls `set_write_fence`
    /// directly rather than driving a full `drive_tick`.
    struct TwoShardFenceControl {
        shard_urls: Vec<String>,
    }

    #[async_trait]
    impl ClusterControl for TwoShardFenceControl {
        async fn patch_spec(
            &self,
            _ns: &str,
            _name: &str,
            _patch: serde_json::Value,
        ) -> Result<()> {
            unreachable!("not used by set_write_fence")
        }
        async fn statefulset_ready_replicas(&self, _ns: &str, _name: &str) -> Result<i64> {
            unreachable!("not used by set_write_fence")
        }
        async fn trigger_rolling_restart(&self, _ns: &str, _name: &str) -> Result<()> {
            unreachable!("not used by set_write_fence")
        }
        async fn admin_token(&self, _ns: &str, _lumen: &Lumen) -> Result<Option<String>> {
            Ok(None)
        }
        fn shard_base_url(&self, _ns: &str, _name: &str, shard: u32) -> String {
            self.shard_urls[shard as usize].clone()
        }
    }

    #[tokio::test]
    async fn set_write_fence_clears_already_armed_shards_on_partial_failure() {
        // Shard A: a real endpoint that records every /admin/reshard:fence
        // call it receives — both the arm attempt and, if R4 works, the
        // best-effort clear triggered by shard B's failure.
        let shard_a = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("POST"))
            .and(wiremock::matchers::path("/admin/reshard:fence"))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(json!({})))
            .mount(&shard_a)
            .await;

        // Shard B: a bound-then-closed port — nothing listens there, so
        // every call to it fails outright, simulating an unreachable shard
        // mid-arm.
        let dead_listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let dead_addr = dead_listener.local_addr().unwrap();
        drop(dead_listener);
        let shard_b_url = format!("http://{dead_addr}");

        let control = TwoShardFenceControl {
            shard_urls: vec![shard_a.uri(), shard_b_url],
        };
        let current = VirtualBucketShardMap::balanced(0, 8, 2).unwrap();
        let mut buckets = BTreeSet::new();
        buckets.insert(0u32);
        let lumen = lumen_with(spec(2, 1, None), None);

        let result = set_write_fence(
            &control,
            &http_client(),
            "acme",
            "search",
            &lumen,
            &current,
            &buckets,
            30,
        )
        .await;
        assert!(result.is_err(), "arm must surface shard B's failure");

        // Shard A must have received exactly 2 requests: the original arm,
        // then the best-effort clear triggered by shard B's failure — R4's
        // whole point is that shard A never stays fenced indefinitely just
        // because shard B was unreachable.
        let requests = shard_a
            .received_requests()
            .await
            .expect("wiremock request recording enabled");
        assert_eq!(
            requests.len(),
            2,
            "shard A must be armed once, then cleared once after shard B's arm failed"
        );
        let clear_body: serde_json::Value = requests[1].body_json().unwrap();
        assert_eq!(
            clear_body["buckets"].as_array().map(Vec::len),
            Some(0),
            "the second call to shard A must be a clear (empty buckets), not another arm"
        );
    }

    // ---- #1467 R3: evict_old_shards's in-loop fence re-arm --------------

    /// A [`ClusterControl`] over a fixed list of already-bound shard URLs
    /// with a test-controlled `write_fence_ttl_secs` — everything
    /// [`evict_old_shards`]/[`maybe_rearm_fence`] needs, nothing more.
    struct FenceRearmControl {
        shard_urls: Vec<String>,
        ttl_secs: u64,
    }

    #[async_trait]
    impl ClusterControl for FenceRearmControl {
        async fn patch_spec(
            &self,
            _ns: &str,
            _name: &str,
            _patch: serde_json::Value,
        ) -> Result<()> {
            unreachable!("not used by evict_old_shards")
        }
        async fn statefulset_ready_replicas(&self, _ns: &str, _name: &str) -> Result<i64> {
            unreachable!("not used by evict_old_shards")
        }
        async fn trigger_rolling_restart(&self, _ns: &str, _name: &str) -> Result<()> {
            unreachable!("not used by evict_old_shards")
        }
        async fn admin_token(&self, _ns: &str, _lumen: &Lumen) -> Result<Option<String>> {
            Ok(None)
        }
        fn shard_base_url(&self, _ns: &str, _name: &str, shard: u32) -> String {
            self.shard_urls[shard as usize].clone()
        }
        fn write_fence_ttl_secs(&self) -> u64 {
            self.ttl_secs
        }
    }

    /// #1467 R3: a slow, multi-shard eviction round (many old physical
    /// shards, each `POST /admin/reshard:evict` round-trip taking real time)
    /// must not run on a single fence arm taken once before the loop starts
    /// — [`evict_old_shards`] re-checks/re-arms via [`maybe_rearm_fence`]
    /// before *every* shard's evict call, not just at the phase boundary
    /// immediately before this function is invoked. Proven by driving 3 old
    /// shards through a real (mocked) eviction round with a tiny fence TTL
    /// and an artificial per-call delay large enough that the un-refreshed
    /// TTL fraction would already have lapsed by the final shard — the
    /// number of `/admin/reshard:fence` arm requests observed across all 3
    /// shards must reflect more than the single caller-side arm.
    #[tokio::test]
    async fn evict_old_shards_rearms_fence_mid_loop_across_slow_multi_shard_round() {
        let mut mock_shards = Vec::new();
        for _ in 0..3 {
            let mock = wiremock::MockServer::start().await;
            wiremock::Mock::given(wiremock::matchers::method("POST"))
                .and(wiremock::matchers::path("/admin/reshard:fence"))
                .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(json!({})))
                .mount(&mock)
                .await;
            wiremock::Mock::given(wiremock::matchers::method("POST"))
                .and(wiremock::matchers::path("/admin/reshard:evict"))
                .respond_with(
                    wiremock::ResponseTemplate::new(200)
                        .set_body_json(json!({}))
                        .set_delay(Duration::from_millis(150)),
                )
                .mount(&mock)
                .await;
            mock_shards.push(mock);
        }
        let shard_urls: Vec<String> = mock_shards.iter().map(|m| m.uri()).collect();

        // ttl_secs=1 -> rearm_after = 250ms (FENCE_REARM_FRACTION=4).
        // `last_armed_at` starts at "just now" (as if the caller armed it
        // immediately before this call, matching the real phase-boundary
        // arm) so the first two iterations' pre-checks (elapsed ~0ms, then
        // ~150ms) skip re-arming, but by the third iteration's pre-check
        // (elapsed ~300ms) the 250ms fraction has lapsed and an in-loop
        // rearm must fire — proving it is *evict_old_shards's own loop*,
        // not just the caller, keeping the fence fresh across a slow round.
        let control = FenceRearmControl {
            shard_urls,
            ttl_secs: 1,
        };
        let current = VirtualBucketShardMap::balanced(0, 8, 3).unwrap();
        let target = VirtualBucketShardMap::balanced(1, 8, 3).unwrap();
        let mut moving_buckets = BTreeSet::new();
        moving_buckets.insert(0u32);
        let lumen = lumen_with(spec(3, 1, None), None);
        let mut last_armed_at = Instant::now();

        evict_old_shards(
            &control,
            &http_client(),
            "acme",
            "search",
            &lumen,
            &current,
            &target,
            Some(&moving_buckets),
            &mut last_armed_at,
        )
        .await
        .unwrap();

        let mut total_fence_calls = 0usize;
        let mut total_evict_calls = 0usize;
        for mock in &mock_shards {
            let requests = mock
                .received_requests()
                .await
                .expect("wiremock request recording enabled");
            total_fence_calls += requests
                .iter()
                .filter(|r| r.url.path() == "/admin/reshard:fence")
                .count();
            total_evict_calls += requests
                .iter()
                .filter(|r| r.url.path() == "/admin/reshard:evict")
                .count();
        }
        assert_eq!(
            total_evict_calls, 3,
            "every one of the 3 old shards must be evicted exactly once"
        );
        assert!(
            total_fence_calls > 0 && total_fence_calls % 3 == 0,
            "evict_old_shards's own loop must have re-armed at least once (a full arm round \
             is 3 fence calls, one per old shard) — this test never arms the fence itself \
             before calling evict_old_shards, so any fence calls observed at all are proof \
             of the in-loop rearm, got {total_fence_calls}"
        );
    }

    // ---- #1444 R2: oversized-doc reshard remediation --------------------

    /// A minimal, otherwise-empty [`ReshardBatch`] whose `external_ids` holds
    /// one collection/id pair with an `external_id` long enough on its own to
    /// push the batch's serialized size over
    /// [`crate::reshard::ADMIN_ROUTE_BODY_LIMIT_BYTES`] — the exact shape
    /// `byte_cap_chunk` produces when it floors at a single oversized id.
    fn oversized_batch(collection: &str, external_id_len: usize) -> ReshardBatch {
        let mut external_ids = BTreeMap::new();
        let mut ids = BTreeSet::new();
        ids.insert("x".repeat(external_id_len));
        external_ids.insert(collection.to_string(), ids);
        ReshardBatch {
            from_map_version: 1,
            to_map_version: 2,
            bucket: 0,
            from_shard: 0,
            to_shard: 1,
            external_ids,
            snapshot: SnapshotV1 {
                version: 1,
                collections: BTreeMap::new(),
            },
        }
    }

    #[test]
    fn detect_oversized_batch_none_when_under_limit() {
        let batch = oversized_batch("widgets", 64);
        assert!(
            detect_oversized_batch(&batch).is_none(),
            "a small batch must not be classified as oversized"
        );
    }

    #[test]
    fn detect_oversized_batch_some_when_over_limit_names_first_id() {
        let batch = oversized_batch(
            "widgets",
            crate::reshard::ADMIN_ROUTE_BODY_LIMIT_BYTES + 1024,
        );
        let block = detect_oversized_batch(&batch)
            .expect("a batch over ADMIN_ROUTE_BODY_LIMIT_BYTES must be classified as oversized");
        assert_eq!(block.collection, "widgets");
        assert_eq!(
            block.external_id.len(),
            crate::reshard::ADMIN_ROUTE_BODY_LIMIT_BYTES + 1024
        );
        assert!(block.bytes > crate::reshard::ADMIN_ROUTE_BODY_LIMIT_BYTES);
    }

    #[tokio::test]
    async fn apply_reshard_batch_rejects_oversized_batch_without_sending_request() {
        // #1444 R2 AC2: the pre-flight check in `apply_reshard_batch` must
        // reject an oversized batch itself — no HTTP round trip at all, let
        // alone one that could 413. `.expect(0)` on the mount makes wiremock
        // panic if the driver ever calls out.
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("POST"))
            .and(wiremock::matchers::path("/admin/reshard:apply"))
            .respond_with(wiremock::ResponseTemplate::new(200))
            .expect(0)
            .mount(&server)
            .await;
        let batch = oversized_batch(
            "widgets",
            crate::reshard::ADMIN_ROUTE_BODY_LIMIT_BYTES + 1024,
        );
        let result = apply_reshard_batch(&http_client(), &server.uri(), None, &batch).await;
        let err = result.expect_err("an oversized batch must be rejected pre-flight");
        assert!(
            err.downcast_ref::<OversizedDocumentBlock>().is_some(),
            "the error must downcast to OversizedDocumentBlock, got: {err:?}"
        );
    }

    #[test]
    fn oversize_block_cache_records_skips_then_exhausts_recheck_budget() {
        // Each test in this crate shares the process-global oversize cache,
        // so use a namespace/name unique to this test to avoid cross-test
        // interference under parallel execution.
        let namespace = "ac2-cache-ns";
        let name = "ac2-cache-name";
        let uid = "ac2-cache-uid";
        assert!(
            oversize_block_condition(namespace, name, uid).is_none(),
            "no wedge recorded yet"
        );
        assert!(
            should_skip_for_oversize(namespace, name, uid).is_none(),
            "nothing to skip before a wedge is ever recorded"
        );

        let block = OversizedDocumentBlock {
            collection: "widgets".to_string(),
            external_id: "abc".to_string(),
            bytes: crate::reshard::ADMIN_ROUTE_BODY_LIMIT_BYTES + 1,
        };
        record_oversize_block(namespace, name, uid, block.clone());
        assert_eq!(
            oversize_block_condition(namespace, name, uid),
            Some(block.clone()),
            "the recorded wedge must be readable without affecting the skip budget"
        );

        // The recheck budget is consumed by `should_skip_for_oversize`, not
        // by the read-only `oversize_block_condition` above.
        for _ in 0..OVERSIZE_RECHECK_TICKS {
            assert_eq!(
                should_skip_for_oversize(namespace, name, uid),
                Some(block.clone()),
                "every tick within the recheck budget must skip on the same wedge"
            );
        }
        assert!(
            should_skip_for_oversize(namespace, name, uid).is_none(),
            "once the recheck budget is exhausted, the next tick must be let through"
        );

        clear_oversize_block(namespace, name);
        assert!(
            oversize_block_condition(namespace, name, uid).is_none(),
            "clearing must remove the wedge entirely"
        );
    }

    #[tokio::test]
    async fn advance_catching_up_skips_fence_arm_when_oversize_wedge_recorded() {
        // #1444 R2 AC2: a tick already known-wedged on an oversized document
        // must short-circuit to `Blocked` before arming the write fence —
        // `.expect(0)` on the fence-route mount makes wiremock panic if the
        // driver ever arms it.
        let namespace = "ac2-fence-ns";
        let name = "ac2-fence-name";
        record_oversize_block(
            namespace,
            name,
            "",
            OversizedDocumentBlock {
                collection: "widgets".to_string(),
                external_id: "abc".to_string(),
                bytes: crate::reshard::ADMIN_ROUTE_BODY_LIMIT_BYTES + 1,
            },
        );

        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("POST"))
            .and(wiremock::matchers::path("/admin/reshard:fence"))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(json!({})))
            .expect(0)
            .mount(&server)
            .await;
        let control = TwoShardFenceControl {
            shard_urls: vec![server.uri()],
        };
        let lumen = lumen_with(spec(2, 1, None), None);

        let outcome = advance_catching_up(&control, &http_client(), namespace, name, &lumen).await;
        assert!(
            matches!(outcome, DriveOutcome::Blocked(_)),
            "a known-wedged tick must report Blocked, got: {outcome:?}"
        );

        clear_oversize_block(namespace, name);
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
