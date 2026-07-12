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
| `DriveOutcome` | projects/lumen/src/operator/reshard_driver.rs | enum | pub | 532 |  |
| `KubeClusterControl` | projects/lumen/src/operator/reshard_driver.rs | struct | pub | 414 |  |
| `OversizedDocumentBlock` | projects/lumen/src/operator/reshard_driver.rs | struct | pub | 264 | #1444 R2: distinguishes an apply failure caused by exactly one document's batch serializing past `crate::reshard::ADMIN_ROUTE_BODY_LIMIT_BYTES` (the `snapshot_reshard_batches`/`byte_cap_chunk` floor case) from any other reason `POST /admin/reshard:apply` can fail — surfaced as a distinct `status.reshard` blocking condition and used to skip re-arming the write-pause fence on a tick already known to fail identically. |
| `compute_target_map` | projects/lumen/src/operator/reshard_driver.rs | function | pub | 638 | compute_target_map(current: &VirtualBucketShardMap) -> Result<VirtualBucketShardMap> |
| `current_shard_map` | projects/lumen/src/operator/reshard_driver.rs | function | pub | 623 | current_shard_map(lumen: &Lumen) -> Result<VirtualBucketShardMap> |
| `default_write_fence_ttl_secs` | projects/lumen/src/operator/reshard_driver.rs | function | pub | 234 | #1443 R1/AC1: the production default [`ClusterControl::write_fence_ttl_secs`] value, exposed so integration tests can fall back to the real default from a `fence_ttl_secs: Option<u64>`-style override field without needing the private `WRITE_FENCE_TTL_SECS` const itself to be `pub`. default_write_fence_ttl_secs() -> u64 |
| `drive_tick` | projects/lumen/src/operator/reshard_driver.rs | function | pub | 1627 | drive_tick(     control: &dyn ClusterControl,     http: &reqwest::Client,     lumen: &Lumen, ) -> DriveOutcome |
| `new` | projects/lumen/src/operator/reshard_driver.rs | function | pub | 420 | new(client: Client) -> Self |
| `oversize_block_condition` | projects/lumen/src/operator/reshard_driver.rs | function | pub | 358 | #1444 R2: the oversized-document block currently recorded for `namespace/name`, if any — read-only, does not affect the skip budget. `reconcile.rs`'s `status_patch` calls this to layer a distinct `status.reshard` blocking condition + remediation message onto the policy/usage-derived status. oversize_block_condition(namespace: &str, name: &str) -> Option<OversizedDocumentBlock> |
| `run_migration_pass` | projects/lumen/src/operator/reshard_driver.rs | function | pub | 1038 | One migration pass: every bucket `bucket_moves` says moved, fetched via `POST /admin/backup:scoped` and applied to its new owner via `POST /admin/reshard:apply` (`snapshot_reshard_batches` builds the bounded, purely-additive batches — #1457 R1). Thin wrapper over the shared `run_migration_pass_impl(..., final_pass: bool, moving_buckets: Option<&BTreeSet<u32>>)`, called here with `final_pass=false`; the final fenced `CatchingUp` pass calls `run_migration_pass_impl` directly with `final_pass=true` to additionally send each moved bucket's authoritative-replace scope via `POST /admin/reshard:prune` (`snapshot_reshard_prune_chunks`). run_migration_pass(     control: &dyn ClusterControl,     http: &reqwest::Client,     namespace: &str,     name: &str,     lumen: &Lumen, ) -> Result<usize> |
| `should_start_split` | projects/lumen/src/operator/reshard_driver.rs | function | pub | 560 | #1396 R5: also requires `status.reshard.usage_measured_at_map_version == spec.shardMap.version` — a lagging/stale status subresource can never start a split. should_start_split(lumen: &Lumen) -> bool |
| `spawn_reshard_driver_loop` | projects/lumen/src/operator/reshard_driver.rs | function | pub | 1664 | spawn_reshard_driver_loop(client: Client) |

Not listed above (matching this project's existing mirrors' convention of
only capturing top-level `pub` structs/enums/consts/modules and inherent-impl
`pub fn`s, never trait definitions, trait methods, or trait-impl methods):
the `pub trait ClusterControl` definition and its six methods (including
`shard_base_url` and the new #1443 R1 `write_fence_ttl_secs` default-impl
method, which returns the private `WRITE_FENCE_TTL_SECS` const unless a
test fake overrides it), and `KubeClusterControl`'s `impl ClusterControl for
KubeClusterControl` block. Also not listed (private): `checkpoint_shard`
(#1389, rewritten #1396 R3 to require `persisted == true`) and
`checkpoint_shards` (#1389 as `checkpoint_touched_shards`, generalized and
renamed by #1396 R1 to take an explicit shard set — called once for just
the target shard right after migration, and again for every source shard
after eviction), plus three #1396 R1/R2 private helpers: `reshard_fence_call`,
`set_write_fence` (arm/clear `POST /admin/reshard:fence` on every current
source shard; rewritten by #1443 R4 to track which shards it has
successfully armed and best-effort clear them before returning an error, so
a mid-sequence arm failure never leaves a partially-fenced cluster), and
`advance_catching_up_fenced` (the actual migrate -> checkpoint(target) ->
evict -> checkpoint(sources) -> cutover sequence `advance_catching_up`
brackets with a fence arm/clear; #1443 R1 adds intra-sequence re-arm calls
before each of those steps so a slow real pass — checkpoints, evictions —
never outlives a single TTL window, and a failed re-arm aborts to `Blocked`
before the next step runs rather than silently reopening the write window).
Also not listed (private, new): `FENCE_REARM_BATCH_INTERVAL` const and the
`default_write_fence_ttl_secs`-backing `WRITE_FENCE_TTL_SECS` const itself.
Also not listed (private/`pub(crate)`, #1444 R2): `OVERSIZE_RECHECK_TICKS`
const, the `OversizeBlockCache` type alias, `oversize_block_cache`,
`oversize_cache_key` (private), `record_oversize_block` and
`clear_oversize_block` (`pub(crate)` rather than private so
`reconcile.rs`'s `status_patch` tests can drive the exact cache
`oversize_block_condition` reads, without widening either past
crate-internal visibility), `should_skip_for_oversize` (private; the
mutating cache read/write helpers backing `oversize_block_condition`), and
`detect_oversized_batch` (pre-flight oversize check called from
`apply_reshard_batch`, which now returns the downcastable
`OversizedDocumentBlock` error instead of only a generic `bail!` on both
the pre-flight miss and a live 413 response). Also not listed (private,
#1457 R1/R2, new): `run_migration_pass_impl` (the shared `final_pass: bool`
implementation `run_migration_pass` now wraps), `fetch_all_collection_ids`
(fetches a source shard's complete `GET /collections` id list independently
of the bucket-scoped snapshot, so `snapshot_reshard_prune_chunks` can seed
an empty `keep_ids` scope for a collection a moved bucket emptied entirely
— #1457 R2), and `apply_reshard_prune_chunk` (this driver's `POST
/admin/reshard:prune` caller for one `ReshardPruneChunk`, mirroring
`apply_reshard_batch`'s existing `POST /admin/reshard:apply` caller).

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

/// `"<namespace>/<name>" -> (block, ticks skipped on it so far)`, written by
/// [`run_migration_pass_impl`] and consumed by [`advance_catching_up`]
/// (mutating, to decide/count a skip) and [`oversize_block_condition`]
/// (read-only, for `reconcile.rs`'s `status_patch`) — #1444 R2. Mirrors
/// `reconcile.rs`'s own `ShardUsageCache` pattern: a synchronous status
/// projection reads a cache a background loop writes, rather than doing I/O
/// itself.
type OversizeBlockCache = std::sync::Mutex<BTreeMap<String, (OversizedDocumentBlock, u32)>>;

fn oversize_block_cache() -> &'static OversizeBlockCache {
    static CACHE: std::sync::OnceLock<OversizeBlockCache> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::Mutex::new(BTreeMap::new()))
}

fn oversize_cache_key(namespace: &str, name: &str) -> String {
    format!("{namespace}/{name}")
}

/// Record (or refresh) a discovered oversize wedge for `namespace/name`,
/// resetting its skip counter — a fresh discovery, whether this is the first
/// tick to hit it or a periodic recheck (#1444 R2, see
/// [`OVERSIZE_RECHECK_TICKS`]) that hit the same wedge again. `pub(crate)`
/// rather than private so `reconcile.rs`'s `status_patch` tests can drive the
/// exact cache [`oversize_block_condition`] reads, without widening this past
/// crate-internal visibility.
pub(crate) fn record_oversize_block(namespace: &str, name: &str, block: OversizedDocumentBlock) {
    oversize_block_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(oversize_cache_key(namespace, name), (block, 0));
}

/// Clear any recorded oversize wedge for `namespace/name` — called whenever
/// a migration pass for it completes without hitting one, meaning whatever
/// was wedged is resolved. `pub(crate)` for the same test-seam reason as
/// [`record_oversize_block`].
pub(crate) fn clear_oversize_block(namespace: &str, name: &str) {
    oversize_block_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .remove(&oversize_cache_key(namespace, name));
}

/// If `namespace/name` has a recorded oversize wedge AND has not yet used up
/// its [`OVERSIZE_RECHECK_TICKS`] skip budget, bump its skip counter and
/// return it — the caller ([`advance_catching_up`]) should short-circuit to
/// [`DriveOutcome::Blocked`] without arming the write-pause fence. Returns
/// `None` (no skip) once the budget is exhausted, letting the next real
/// attempt either clear the wedge (if fixed) or re-record it with a fresh
/// budget.
fn should_skip_for_oversize(namespace: &str, name: &str) -> Option<OversizedDocumentBlock> {
    let mut cache = oversize_block_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (block, ticks) = cache.get_mut(&oversize_cache_key(namespace, name))?;
    if *ticks >= OVERSIZE_RECHECK_TICKS {
        return None;
    }
    *ticks += 1;
    Some(block.clone())
}

/// The oversized-document block currently recorded for `namespace/name`, if
/// any (#1444 R2) — read-only, does not affect [`should_skip_for_oversize`]'s
/// skip budget. `reconcile.rs`'s `status_patch` calls this to layer a
/// distinct `status.reshard` blocking condition + remediation message onto
/// the policy/usage-derived status.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reshard-driver-rs.md#source
pub fn oversize_block_condition(namespace: &str, name: &str) -> Option<OversizedDocumentBlock> {
    oversize_block_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(&oversize_cache_key(namespace, name))
        .map(|(block, _)| block.clone())
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

/// #1443 R1: re-arm the write fence every this-many applied batches inside a
/// fenced [`run_migration_pass_impl`] loop — a fenced pass with hundreds of
/// sequential byte-capped batch POSTs plus checkpoints can realistically run
/// past [`WRITE_FENCE_TTL_SECS`]/[`ClusterControl::write_fence_ttl_secs`]
/// without this, silently reopening the write window mid-sequence.
const FENCE_REARM_BATCH_INTERVAL: usize = 20;

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
/// [`ClusterControl::write_fence_ttl_secs`] deadline every
/// [`FENCE_REARM_BATCH_INTERVAL`] applied batches/chunks — a re-arm failure
/// aborts the whole pass immediately (propagated as `Err`, which every
/// caller already surfaces as `DriveOutcome::Blocked` before eviction ever
/// runs).
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
    // Counts both applied batches and applied prune chunks — the shared
    // clock [`FENCE_REARM_BATCH_INTERVAL`] re-arms the fence against,
    // independent of which of the two loops below is currently running.
    let mut applied_calls = 0usize;
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
            if let Err(err) = apply_reshard_batch(http, &dest_url, token.as_deref(), batch).await {
                // #1444 R2: record the wedge distinctly before propagating, so
                // callers that turn this `Err` into `DriveOutcome::Blocked`
                // still leave a structured trace behind for `status.reshard`
                // and for `advance_catching_up`'s fence-skip check, even
                // though the `Err` itself stays a generic message.
                if let Some(oversized) = err.downcast_ref::<OversizedDocumentBlock>() {
                    record_oversize_block(namespace, name, oversized.clone());
                }
                return Err(err);
            }
            total_batches += 1;
            applied_calls += 1;
            if let Some(fenced_buckets) = moving_buckets
                .filter(|b| !b.is_empty() && applied_calls % FENCE_REARM_BATCH_INTERVAL == 0)
            {
                set_write_fence(
                    control,
                    http,
                    namespace,
                    name,
                    lumen,
                    &current,
                    fenced_buckets,
                    control.write_fence_ttl_secs(),
                )
                .await
                .context("re-arm write fence mid migration pass")?;
            }
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
                applied_calls += 1;
                if let Some(fenced_buckets) = moving_buckets
                    .filter(|b| !b.is_empty() && applied_calls % FENCE_REARM_BATCH_INTERVAL == 0)
                {
                    set_write_fence(
                        control,
                        http,
                        namespace,
                        name,
                        lumen,
                        &current,
                        fenced_buckets,
                        control.write_fence_ttl_secs(),
                    )
                    .await
                    .context("re-arm write fence mid migration pass")?;
                }
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
    if let Some(block) = should_skip_for_oversize(namespace, name) {
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
/// under periodic in-loop re-arming; the fence is additionally re-armed with
/// a fresh TTL at every phase boundary below (after the migration pass and
/// before the target checkpoint, again before eviction, and again before the
/// sources' checkpoint round) so a real pass — hundreds of sequential
/// batch/checkpoint HTTP round trips — can never silently outlive
/// [`ClusterControl::write_fence_ttl_secs`] mid-sequence. Any re-arm failure
/// aborts to [`DriveOutcome::Blocked`] immediately, always strictly before
/// [`evict_old_shards`] runs.
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
    }

    if let Err(err) = evict_old_shards(control, http, namespace, name, lumen, current, target).await
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
        assert!(
            oversize_block_condition(namespace, name).is_none(),
            "no wedge recorded yet"
        );
        assert!(
            should_skip_for_oversize(namespace, name).is_none(),
            "nothing to skip before a wedge is ever recorded"
        );

        let block = OversizedDocumentBlock {
            collection: "widgets".to_string(),
            external_id: "abc".to_string(),
            bytes: crate::reshard::ADMIN_ROUTE_BODY_LIMIT_BYTES + 1,
        };
        record_oversize_block(namespace, name, block.clone());
        assert_eq!(
            oversize_block_condition(namespace, name),
            Some(block.clone()),
            "the recorded wedge must be readable without affecting the skip budget"
        );

        // The recheck budget is consumed by `should_skip_for_oversize`, not
        // by the read-only `oversize_block_condition` above.
        for _ in 0..OVERSIZE_RECHECK_TICKS {
            assert_eq!(
                should_skip_for_oversize(namespace, name),
                Some(block.clone()),
                "every tick within the recheck budget must skip on the same wedge"
            );
        }
        assert!(
            should_skip_for_oversize(namespace, name).is_none(),
            "once the recheck budget is exhausted, the next tick must be let through"
        );

        clear_oversize_block(namespace, name);
        assert!(
            oversize_block_condition(namespace, name).is_none(),
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
  - path: projects/lumen/src/operator/reshard_driver.rs
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1386 AC1/AC2: added `should_start_split_false_on_stale_pre_cutover_usage`
      and `should_start_split_true_on_fresh_post_cutover_usage_above_urgent`
      unit tests, exercising `should_start_split` against a status built via
      `LumenSpec::reshard_status_with_usage`'s new freshness-generation
      parameter. `should_start_split`/`drive_tick` themselves are
      unchanged — the freshness gate lives entirely in
      `reshard_status_with_usage`'s `blockingConditions` output, which this
      function's existing check already consumes.
  - path: projects/lumen/src/operator/reshard_driver.rs
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1396 R1/R2/R3/R5: pre-land review fixes for cutover correctness.
      R1 re-sequences `advance_catching_up` (now `advance_catching_up` +
      new `advance_catching_up_fenced`) to migrate -> checkpoint the
      target-only -> evict sources -> checkpoint sources -> cutover, so no
      source eviction can become durable, or even be attempted, before the
      target's migrated data for the same pass is durably checkpointed
      (`checkpoint_touched_shards` generalized to `checkpoint_shards` over
      an explicit shard iterator, called twice per tick). R2 adds a
      bounded, status-visible write pause on still-moving buckets during
      the final `CatchingUp` pass via new `WRITE_FENCE_TTL_SECS`,
      `reshard_fence_call`, and `set_write_fence` (arm before the pass,
      unconditional clear after, on every exit path) against the new
      `api.rs` `POST /admin/reshard:fence` seam. R3 rewrites
      `checkpoint_shard` to parse the response body and require
      `persisted == true`; a 200 `{"persisted": false}` now bails with a
      shard-naming error, surfacing as `DriveOutcome::Blocked` rather than
      a satisfied durability gate. R5 adds a freshness re-check to
      `should_start_split`: `status.reshard.usage_measured_at_map_version`
      must equal `spec.shardMap.version`, so a lagging/stale status
      subresource can never start a split on its own. New/updated unit
      tests: `checkpoint_shard_blocked_when_response_reports_persisted_false`,
      `checkpoint_shard_blocked_when_response_omits_persisted_key`,
      `checkpoint_shard_ok_when_response_reports_persisted_true` (AC3),
      `should_start_split_false_on_stale_status_map_version`,
      `should_start_split_true_on_fresh_status_map_version` (AC5), and
      `status_with_blocking`'s fixture now sets
      `usage_measured_at_map_version` to stay fresh under the new R5 gate.
  - path: projects/lumen/src/operator/reshard_driver.rs
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1442 R2: `advance_catching_up` no longer clears the write fence
      unconditionally on every exit path — when the outcome is
      `DriveOutcome::CompletedSplit`, the fence is left armed (bounded by
      `WRITE_FENCE_TTL_SECS`) instead of cleared immediately. A completed
      split just called `ClusterControl::trigger_rolling_restart`, but pods
      only read `SHARD_MAP_*`/`SHARD_COUNT` env at boot, so old-map pods
      keep serving (and would otherwise keep accepting local writes for) the
      just-evicted source buckets until the rolling restart actually reaches
      them; clearing the fence right after triggering the restart would
      reopen exactly that mixed-map window. Every other exit path (success
      without a completed split, or `Blocked`) still clears the fence
      immediately as before. Scope is limited to fence-through-restart
      sequencing only; new test
      `write_fence_stays_armed_immediately_after_completed_split` in
      `tests/reshard_driver_e2e.rs` covers the behavior.
  - path: projects/lumen/src/operator/reshard_driver.rs
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1443 round-2 write-fence hardening (R1/R3/R4; R2's design lives in
      `projects-lumen-src-reshard-rs.md`/`projects-lumen-src-storage-rs.md`).
      R1 (TTL too short for a real pass): new private `FENCE_REARM_BATCH_INTERVAL`
      const plus a refactored `run_migration_pass_impl` re-arms the fence
      every `FENCE_REARM_BATCH_INTERVAL` applied batches during a long
      migration; `advance_catching_up_fenced` additionally re-arms
      immediately before the target checkpoint, before eviction, and before
      the source checkpoint — every step boundary in the fenced sequence now
      starts with a fresh TTL rather than relying on the single arm at the
      top of the pass. A failed re-arm at any of these points aborts the
      whole sequence to `DriveOutcome::Blocked` *before* the next
      (potentially destructive, e.g. evict) step runs — never proceeds on a
      possibly-expired fence. New public seam: `ClusterControl::
      write_fence_ttl_secs()` (default-impl trait method returning
      `WRITE_FENCE_TTL_SECS`) and the new `pub fn
      default_write_fence_ttl_secs()` free function that exposes that same
      default to integration tests building a `fence_ttl_secs: Option<u64>`
      override fixture. R3 (arm overflow panic): `set_write_fence`'s
      deadline computation now uses `Instant::checked_add` instead of `+`,
      returning a clean error instead of panicking on an out-of-range TTL;
      pairs with `api.rs`'s new `MAX_FENCE_TTL_SECS` request-validation gate
      so an invalid `ttl_secs` is rejected as 400 before it ever reaches this
      arithmetic. R4 (partial-arm leak): `set_write_fence` now tracks
      `armed_urls` as it arms each source shard in turn and, on the first
      failure, best-effort clears every shard it already armed before
      returning the error — a shard that failed to arm never leaves a sibling
      shard fenced with no driver retry in flight. New unit test
      `set_write_fence_clears_already_armed_shards_on_partial_failure`
      (AC4) proves this with a two-shard fake where shard A arms
      successfully and shard B is unreachable, asserting shard A receives
      exactly an arm call followed by a clear (empty-buckets) call. New e2e
      test `write_fence_survives_a_tick_longer_than_a_single_ttl` (AC1) in
      `tests/reshard_driver_e2e.rs` proves a fenced pass whose two sequential
      checkpoint calls together exceed a short TTL never lets a write through
      mid-pass.
  - path: "projects/lumen/src/operator/reshard_driver.rs"
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1444 R2: an oversized single-document batch (`byte_cap_chunk`'s
      single-id floor case in `reshard.rs`, over
      `crate::reshard::ADMIN_ROUTE_BODY_LIMIT_BYTES`) no longer wedges the
      split as a silent, permanently-recurring generic `Blocked` failure
      with a fence armed every tick. New `detect_oversized_batch` helper
      pre-flight-checks a batch's real wire size before `apply_reshard_batch`
      sends it (and as a defense-in-depth fallback on a live 413 response),
      returning a new `OversizedDocumentBlock` error (public struct,
      `Display`+`Error`, names the collection/external_id/byte count and
      carries remediation text). `run_migration_pass_impl` downcasts an
      `apply_reshard_batch` failure to `OversizedDocumentBlock` and records
      it (namespace/name-keyed, in-process `OversizeBlockCache`, mirroring
      `reconcile.rs`'s `ShardUsageCache` pattern) before propagating the
      error, and clears any recorded block on a pass that completes without
      hitting one (self-healing once an operator fixes the document).
      `advance_catching_up` now checks `should_skip_for_oversize` before
      arming the write-pause fence: a tick with a currently-recorded block
      short-circuits to `DriveOutcome::Blocked` without arming the fence,
      closing the recurring 503 write-window regression; a bounded
      `OVERSIZE_RECHECK_TICKS` budget still lets a real attempt through
      periodically so a since-fixed document is rediscovered. New public
      `oversize_block_condition(namespace, name)` read accessor lets
      `reconcile.rs`'s `status_patch` layer a distinct
      `"reshardOversizedDocument"` `status.reshard.blockingConditions`
      entry + remediation `message` onto the policy/usage-derived status.
  - path: "projects/lumen/src/operator/reshard_driver.rs"
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1444 R2 AC2 test coverage: `record_oversize_block` and
      `clear_oversize_block` widened from private to `pub(crate)` (still not
      part of the crate's public API) so `reconcile.rs`'s `status_patch`
      tests can drive the exact `oversize_block_condition` cache. New unit
      tests: `detect_oversized_batch_none_when_under_limit` and
      `detect_oversized_batch_some_when_over_limit_names_first_id` prove the
      pre-flight size classification and first-offender selection;
      `apply_reshard_batch_rejects_oversized_batch_without_sending_request`
      proves the pre-flight check short-circuits before any HTTP call
      (`wiremock` `.expect(0)`); `oversize_block_cache_records_skips_then_
      exhausts_recheck_budget` proves the record/read/skip/exhaust/clear
      cache lifecycle end to end; `advance_catching_up_skips_fence_arm_when_
      oversize_wedge_recorded` proves a known-wedged tick reports `Blocked`
      without ever calling the fence route (`wiremock` `.expect(0)` on
      `/admin/reshard:fence`).
```
