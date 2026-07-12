---
id: projects-lumen-src-reshard-rs
capability_refs:
  - id: "dynamic-shard-topology"
    role: primary
    claim: "versioned-virtual-bucket-shard-map"
    coverage: full
    rationale: "This source unit owns bounded SnapshotV1 movement for versioned virtual-bucket shard-map changes."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/reshard.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/reshard.rs` generated from AST during Lumen AW health remediation.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ADMIN_ROUTE_BODY_LIMIT_BYTES` | projects/lumen/src/reshard.rs | const | pub | 28 | #1444 R2: the hard body-size limit `POST /admin/reshard:apply` (and every other admin route) enforces at the HTTP layer via `api.rs`'s `DefaultBodyLimit::max(..)`, shared verbatim so the two can never drift; `crate::operator::reshard_driver`'s oversize-wedge detection also compares against this constant. |
| `MAX_BATCH_BYTES` | projects/lumen/src/reshard.rs | const | pub | 40 | #1396 R4: upper bound on one batch's serialized `snapshot` payload — half of `ADMIN_ROUTE_BODY_LIMIT_BYTES` (#1444 R2, was a hardcoded `4 * 1024 * 1024`). |
| `BucketMove` | projects/lumen/src/reshard.rs | struct | pub | 44 |  |
| `ReshardBatch` | projects/lumen/src/reshard.rs | struct | pub | 68 | #1380: `Serialize`/`Deserialize` make a batch postable to `POST /admin/reshard:apply` as-is. #1457 R1: reverted to purely additive (`from_map_version, to_map_version, bucket, from_shard, to_shard, external_ids, snapshot`) — the #1443 R2 `virtual_bucket_count`/`replace_ids` fields moved to the independently-chunked `ReshardPruneChunk`. |
| `ReshardBatchReplaceScope` | projects/lumen/src/reshard.rs | struct | pub | 90 | #1443 R2 / #1457 R1: the authoritative-subset-replace scope one applying shard must enforce for a `bucket`+collection, now derived from one or more `ReshardPruneChunk`s accumulated by `crate::storage::Engine::apply_reshard_prune_chunk` rather than stamped directly onto a `ReshardBatch`. |
| `ReshardPruneChunk` | projects/lumen/src/reshard.rs | struct | pub | 114 | #1457 R1: one byte-capped chunk of the authoritative "keep" id set for a single `(bucket, collection_id)` pair under the final migration pass's `to` map; receiver accumulates by `(to_map_version, bucket, collection_id, total_chunks)` and prunes once every `chunk_index` has arrived. |
| `bucket_moves` | projects/lumen/src/reshard.rs | function | pub | 128 | bucket_moves(     from: &VirtualBucketShardMap,     to: &VirtualBucketShardMap, ) -> Result<Vec<BucketMove>> |
| `snapshot_reshard_batches` | projects/lumen/src/reshard.rs | function | pub | 170 | #1396 R4: batches are also byte-capped (`MAX_BATCH_BYTES`), splitting an id-chunk further via `byte_cap_chunk` when its serialized snapshot would exceed the cap. #1457 R1: dropped the #1443 R2 `replace_mode: bool` parameter — every pass, including the final `CatchingUp` pass, is now purely additive; the authoritative prune scope is emitted separately by `snapshot_reshard_prune_chunks`. snapshot_reshard_batches(     snapshot: &SnapshotV1,     from: &VirtualBucketShardMap,     to: &VirtualBucketShardMap,     max_external_ids_per_batch: usize, ) -> Result<Vec<ReshardBatch>> |
| `snapshot_reshard_prune_chunks` | projects/lumen/src/reshard.rs | function | pub | 271 | #1457 R1/R2: builds the final migration pass's authoritative "keep" chunks for every `(bucket, collection_id)` pair in `buckets` × `collection_ids`, byte-capped independently per pair via the private `chunk_ids_by_bytes` helper; a pair with zero matching docs still emits exactly one chunk with empty `keep_ids` (closes the #1443-disclosed delete-resurrection edge on an emptied collection). snapshot_reshard_prune_chunks(     snapshot: &SnapshotV1,     to: &VirtualBucketShardMap,     buckets: &BTreeSet<u32>,     collection_ids: &BTreeSet<String>,     max_chunk_bytes: usize, ) -> Result<Vec<ReshardPruneChunk>> |
| `merge_snapshot_delta` | projects/lumen/src/reshard.rs | function | pub | 383 | merge_snapshot_delta(mut base: SnapshotV1, delta: SnapshotV1) -> Result<SnapshotV1> |
| `snapshot_bucket_subset` | projects/lumen/src/reshard.rs | function | pub | 411 | #1380 R2: bucket-scoped export subset, routed with the same `route_document` hash `snapshot_reshard_batches` uses. snapshot_bucket_subset(     snapshot: &SnapshotV1,     virtual_bucket_count: u32,     buckets: &BTreeSet<u32>, ) -> Result<SnapshotV1> |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-reshard-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Snapshot-level resharding primitives.
//!
//! The operator owns when a shard split is allowed; this module owns the
//! data-plane unit that makes a split gradual: compare two versioned
//! virtual-bucket maps, identify moved buckets, and emit bounded SnapshotV1
//! batches containing only the external_ids that now belong to a different
//! physical shard.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::routing::VirtualBucketShardMap;
use crate::storage::{CollectionSnapshot, FieldIndexSnapshot, SnapshotV1};

/// The hard body-size limit `POST /admin/reshard:apply` (and every other
/// admin route) enforces at the HTTP layer — `api.rs`'s
/// `DefaultBodyLimit::max(..)` is built from this exact constant (#1444 R2),
/// so the two can never drift apart: a batch this crate computes as
/// "under the limit" is always actually under the limit the route enforces,
/// and [`crate::operator::reshard_driver`]'s oversize-wedge detection
/// compares a batch's real wire size against this same number rather than a
/// second, hand-copied literal.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-reshard-rs.md#source
pub const ADMIN_ROUTE_BODY_LIMIT_BYTES: usize = 8 * 1024 * 1024;

/// Upper bound on one batch's serialized `snapshot` payload (#1396 R4):
/// [`ADMIN_ROUTE_BODY_LIMIT_BYTES`] is the route's hard 413 cutoff, but
/// [`snapshot_reshard_batches`] used to cap batches only by external-id count
/// (`MAX_EXTERNAL_IDS_PER_BATCH`-style caller constants) — a bucket of large
/// documents (long text fields, vectors, hashes) can still serialize well
/// past the route limit even at a small id count, and a 413 from an
/// oversized batch is deterministically recomputed identically every driver
/// tick, wedging the split forever (the confirmed defect). Half the route
/// limit leaves comfortable headroom for JSON/wire overhead and per-item
/// framing above the raw snapshot bytes measured here.
pub const MAX_BATCH_BYTES: usize = ADMIN_ROUTE_BODY_LIMIT_BYTES / 2;

#[derive(Clone, Debug, PartialEq, Eq)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-reshard-rs.md#source
pub struct BucketMove {
    pub bucket: u32,
    pub from_shard: u32,
    pub to_shard: u32,
}

/// #1380: `Serialize`/`Deserialize` make a batch postable to `POST
/// /admin/reshard:apply` as-is — the wire payload for the admin apply verb
/// is this struct's exact JSON shape, no separate DTO.
///
/// #1457 R1: `ReshardBatch` is now purely additive — it used to also carry
/// an authoritative `replace_ids`/`virtual_bucket_count` pair for the final
/// migration pass (#1443 R2), but stamping the *complete* id set onto every
/// byte-capped chunk of a bucket made the final pass's wire size scale with
/// total bucket population rather than the chunk's own content, so a bucket
/// whose id set alone serialized past the byte cap produced chunks over the
/// route's hard body limit no matter how small `snapshot`/`external_ids`
/// were — and [`crate::operator::reshard_driver::detect_oversized_batch`]
/// wrongly blamed whichever document happened to be first in the chunk. The
/// authoritative-replace concern moved to its own dedicated, independently
/// chunked message: [`ReshardPruneChunk`] /
/// [`snapshot_reshard_prune_chunks`].
#[derive(Clone, Debug, Serialize, Deserialize)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-reshard-rs.md#source
pub struct ReshardBatch {
    pub from_map_version: u64,
    pub to_map_version: u64,
    pub bucket: u32,
    pub from_shard: u32,
    pub to_shard: u32,
    pub external_ids: BTreeMap<String, BTreeSet<String>>,
    pub snapshot: SnapshotV1,
}

/// #1443 R2 / #1457 R1: the authoritative-subset-replace scope one applying
/// shard must enforce for a `bucket`+collection, derived from one or more
/// [`ReshardPruneChunk`]s sharing the same `(to_map_version, bucket,
/// collection_id, total_chunks)` key once every chunk has been received
/// (see [`crate::storage::Engine::apply_reshard_prune_chunk`]'s receiver-side
/// accumulator). Applying this scope after a batch's additive merge closes
/// the delete-resurrection gap #1443 found: a document deleted on the source
/// during the split is absent from the final pass's authoritative id set and
/// is pruned from the target rather than surviving only because an earlier,
/// now-stale copy landed on the target from a prior additive pass.
#[derive(Clone, Debug, PartialEq, Eq)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-reshard-rs.md#source
pub struct ReshardBatchReplaceScope {
    pub bucket: u32,
    pub virtual_bucket_count: u32,
    pub replace_ids: BTreeMap<String, BTreeSet<String>>,
}

/// #1457 R1: one byte-capped chunk of the authoritative "keep" id set for a
/// single `(bucket, collection_id)` pair under the final migration pass's
/// `to` map. Unlike [`ReshardBatch`] (purely additive, chunked by
/// `max_external_ids_per_batch`/[`MAX_BATCH_BYTES`] with no cross-chunk
/// coupling), every chunk of one `(bucket, collection_id)`'s keep set shares
/// the same `to_map_version`/`bucket`/`collection_id`/`total_chunks` and
/// carries only its own slice of `keep_ids` — the receiver
/// ([`crate::storage::Engine::apply_reshard_prune_chunk`]) accumulates
/// chunks by that key and prunes only once every `chunk_index` in
/// `0..total_chunks` has arrived, so re-sending any subset (413 retry) or
/// all chunks (whole-pass retry after a driver restart) converges to the
/// same pruned result rather than pruning against a partial, still-assembling
/// keep set. `keep_ids` may be empty — every moved bucket emits a chunk for
/// every collection that exists on the source shard, even one a batch of
/// deletes emptied entirely (#1457 R2 / #1443's disclosed edge), so the
/// bucket's copies of that collection are still pruned on cutover.
#[derive(Clone, Debug, Serialize, Deserialize)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-reshard-rs.md#source
pub struct ReshardPruneChunk {
    pub to_map_version: u64,
    pub bucket: u32,
    pub virtual_bucket_count: u32,
    pub collection_id: String,
    pub chunk_index: u32,
    pub total_chunks: u32,
    pub keep_ids: BTreeSet<String>,
}

/// Return the virtual buckets whose physical owner changes between two map
/// versions. A shard split keeps the virtual bucket count stable and changes
/// assignments in small increments.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-reshard-rs.md#source
pub fn bucket_moves(
    from: &VirtualBucketShardMap,
    to: &VirtualBucketShardMap,
) -> Result<Vec<BucketMove>> {
    if from.virtual_bucket_count() != to.virtual_bucket_count() {
        bail!(
            "reshard requires stable virtual_bucket_count ({} != {})",
            from.virtual_bucket_count(),
            to.virtual_bucket_count()
        );
    }

    let mut moves = Vec::new();
    for bucket in 0..from.virtual_bucket_count() {
        let from_shard = from
            .assignment_for_bucket(bucket)
            .expect("bucket within from-map range");
        let to_shard = to
            .assignment_for_bucket(bucket)
            .expect("bucket within to-map range");
        if from_shard != to_shard {
            moves.push(BucketMove {
                bucket,
                from_shard,
                to_shard,
            });
        }
    }
    Ok(moves)
}

/// Build bounded, purely-additive snapshot batches for documents that move
/// under `to`. Batches are grouped by `(bucket, from_shard, to_shard)` and
/// capped by `max_external_ids_per_batch`/[`MAX_BATCH_BYTES`], so an
/// operator can checkpoint progress after every emitted batch instead of
/// blocking on one full-shard copy.
///
/// #1457 R1: every pass — including the reshard driver's final `CatchingUp`
/// pass, run under the write fence — uses this purely-additive form. The
/// final pass's authoritative prune scope is now a separate, independently
/// byte-capped message: see [`snapshot_reshard_prune_chunks`].
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-reshard-rs.md#source
pub fn snapshot_reshard_batches(
    snapshot: &SnapshotV1,
    from: &VirtualBucketShardMap,
    to: &VirtualBucketShardMap,
    max_external_ids_per_batch: usize,
) -> Result<Vec<ReshardBatch>> {
    if max_external_ids_per_batch == 0 {
        bail!("max_external_ids_per_batch must be > 0");
    }

    let moves = bucket_moves(from, to)?;
    if moves.is_empty() {
        return Ok(Vec::new());
    }
    let moves_by_bucket: BTreeMap<u32, BucketMove> =
        moves.into_iter().map(|m| (m.bucket, m)).collect();
    let mut ids_by_move: BTreeMap<(u32, u32, u32), BTreeMap<String, Vec<String>>> = BTreeMap::new();

    for (collection_id, collection) in &snapshot.collections {
        for external_id in collection.eid_fields.keys() {
            let route = from.route_document(collection_id, None, external_id);
            let Some(mv) = moves_by_bucket.get(&route.bucket) else {
                continue;
            };
            let new_route = to.route_document(collection_id, None, external_id);
            if new_route.shard != mv.to_shard {
                bail!(
                    "route for bucket {} changed unexpectedly: assignment={} route={}",
                    mv.bucket,
                    mv.to_shard,
                    new_route.shard
                );
            }
            ids_by_move
                .entry((mv.bucket, mv.from_shard, mv.to_shard))
                .or_default()
                .entry(collection_id.clone())
                .or_default()
                .push(external_id.clone());
        }
    }

    let mut batches = Vec::new();
    for ((bucket, from_shard, to_shard), by_collection) in ids_by_move {
        let mut pending: Vec<(String, String)> = by_collection
            .into_iter()
            .flat_map(|(collection_id, mut ids)| {
                ids.sort();
                ids.into_iter()
                    .map(move |external_id| (collection_id.clone(), external_id))
            })
            .collect();
        pending.sort();

        for chunk in pending.chunks(max_external_ids_per_batch) {
            let mut sub_batches = Vec::new();
            byte_cap_chunk(snapshot, chunk, MAX_BATCH_BYTES, &mut sub_batches)?;
            for (external_ids, partial) in sub_batches {
                batches.push(ReshardBatch {
                    from_map_version: from.version(),
                    to_map_version: to.version(),
                    bucket,
                    from_shard,
                    to_shard,
                    external_ids,
                    snapshot: partial,
                });
            }
        }
    }

    Ok(batches)
}

/// #1457 R1 / R2: build the final migration pass's authoritative "keep"
/// chunks for every `(bucket, collection_id)` pair, `bucket` restricted to
/// `buckets` (the caller's current `from_shard` group — never the whole
/// map's moved buckets, so one from-shard's prune scope can never claim
/// authority over a bucket another from-shard actually owns) and
/// `collection_id` ranging over `collection_ids` (the *full* list of
/// collections that exist on the source shard, fetched independently of
/// `snapshot` — see module docs on why the bucket-scoped snapshot's own
/// collection keys are not sufficient).
///
/// `snapshot` only needs to cover `buckets` (a bucket-scoped export is
/// enough): for each `(bucket, collection_id)` pair, `keep_ids` is exactly
/// the external_ids in `snapshot` that route to that bucket for that
/// collection, computed with `to.route_document` (identical to
/// `from.route_document`'s bucket component, since [`bucket_moves`] already
/// requires a stable `virtual_bucket_count` between the two maps). A
/// collection with **zero** matching docs in a bucket still emits exactly
/// one chunk with an empty `keep_ids` (`total_chunks == 1`) — this is what
/// makes a collection a batch of deletes emptied entirely still get pruned
/// on the target rather than silently keeping its stale copies (#1457 R2,
/// the edge #1443 disclosed).
///
/// Each `(bucket, collection_id)`'s keep set is independently byte-capped by
/// `max_chunk_bytes` via [`chunk_ids_by_bytes`] — unlike [`ReshardBatch`],
/// no *other* pair's chunk count or size is affected by how large one pair's
/// population is.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-reshard-rs.md#source
pub fn snapshot_reshard_prune_chunks(
    snapshot: &SnapshotV1,
    to: &VirtualBucketShardMap,
    buckets: &BTreeSet<u32>,
    collection_ids: &BTreeSet<String>,
    max_chunk_bytes: usize,
) -> Result<Vec<ReshardPruneChunk>> {
    if max_chunk_bytes == 0 {
        bail!("max_chunk_bytes must be > 0");
    }

    let virtual_bucket_count = to.virtual_bucket_count();
    let mut keep: BTreeMap<(u32, String), BTreeSet<String>> = BTreeMap::new();
    for &bucket in buckets {
        for collection_id in collection_ids {
            keep.insert((bucket, collection_id.clone()), BTreeSet::new());
        }
    }
    for (collection_id, collection) in &snapshot.collections {
        if !collection_ids.contains(collection_id) {
            continue;
        }
        for external_id in collection.eid_fields.keys() {
            let bucket = to.route_document(collection_id, None, external_id).bucket;
            if let Some(ids) = keep.get_mut(&(bucket, collection_id.clone())) {
                ids.insert(external_id.clone());
            }
        }
    }

    let mut chunks = Vec::new();
    for ((bucket, collection_id), ids) in keep {
        let pieces = chunk_ids_by_bytes(&ids, max_chunk_bytes);
        let total_chunks = pieces.len() as u32;
        for (chunk_index, keep_ids) in pieces.into_iter().enumerate() {
            chunks.push(ReshardPruneChunk {
                to_map_version: to.version(),
                bucket,
                virtual_bucket_count,
                collection_id: collection_id.clone(),
                chunk_index: chunk_index as u32,
                total_chunks,
                keep_ids,
            });
        }
    }
    Ok(chunks)
}

/// Recursively halve `ids` until each emitted chunk's serialized size is at
/// or under `max_bytes`, or the chunk is down to a single id (mirrors
/// [`byte_cap_chunk`]'s same one-item floor for the same reason: a single id
/// long enough alone to exceed the cap cannot be split further). An empty
/// `ids` still returns exactly one (empty) chunk — [`snapshot_reshard_prune_chunks`]
/// relies on this to always emit at least one chunk per `(bucket,
/// collection_id)` pair, including pairs with nothing left to keep.
fn chunk_ids_by_bytes(ids: &BTreeSet<String>, max_bytes: usize) -> Vec<BTreeSet<String>> {
    if ids.is_empty() {
        return vec![BTreeSet::new()];
    }
    let size = serde_json::to_vec(ids)
        .map(|b| b.len())
        .unwrap_or(usize::MAX);
    if size <= max_bytes || ids.len() == 1 {
        return vec![ids.clone()];
    }
    let mid = ids.len() / 2;
    let first: BTreeSet<String> = ids.iter().take(mid).cloned().collect();
    let rest: BTreeSet<String> = ids.iter().skip(mid).cloned().collect();
    let mut out = chunk_ids_by_bytes(&first, max_bytes);
    out.extend(chunk_ids_by_bytes(&rest, max_bytes));
    out
}

/// Recursively halve `chunk` (already `<= max_external_ids_per_batch` ids)
/// until each emitted `(external_ids, snapshot)` pair's serialized snapshot
/// is at or under `max_batch_bytes`, or the chunk is down to a single
/// external_id — one oversized document cannot be split further, so it is
/// emitted as its own (over-budget) batch rather than looping forever; a
/// single document that alone exceeds [`ADMIN_ROUTE_BODY_LIMIT_BYTES`] is a
/// data-modeling problem this splitter cannot solve, not a batching bug —
/// [`crate::operator::reshard_driver`] detects and surfaces exactly this
/// batch shape (#1444 R2) rather than retrying it forever as a generic 413.
fn byte_cap_chunk(
    snapshot: &SnapshotV1,
    chunk: &[(String, String)],
    max_batch_bytes: usize,
    out: &mut Vec<(BTreeMap<String, BTreeSet<String>>, SnapshotV1)>,
) -> Result<()> {
    if chunk.is_empty() {
        return Ok(());
    }
    let external_ids = ids_map_from_pairs(chunk.iter().cloned());
    let partial = snapshot_subset(snapshot, &external_ids)?;
    let size = serde_json::to_vec(&partial)
        .map(|bytes| bytes.len())
        .unwrap_or(usize::MAX);
    if size <= max_batch_bytes || chunk.len() == 1 {
        out.push((external_ids, partial));
        return Ok(());
    }
    let mid = chunk.len() / 2;
    byte_cap_chunk(snapshot, &chunk[..mid], max_batch_bytes, out)?;
    byte_cap_chunk(snapshot, &chunk[mid..], max_batch_bytes, out)?;
    Ok(())
}

/// Merge a reshard delta snapshot into an existing target snapshot. This is
/// the wire-level primitive an operator can use between batches: fetch target
/// snapshot, merge one moved-bucket batch, restore the merged snapshot, then
/// checkpoint the batch as complete.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-reshard-rs.md#source
pub fn merge_snapshot_delta(mut base: SnapshotV1, delta: SnapshotV1) -> Result<SnapshotV1> {
    if base.version != delta.version {
        bail!(
            "snapshot version mismatch: base={} delta={}",
            base.version,
            delta.version
        );
    }
    for (collection_id, delta_collection) in delta.collections {
        match base.collections.get_mut(&collection_id) {
            Some(base_collection) => merge_collection_delta(base_collection, delta_collection)?,
            None => {
                base.collections.insert(collection_id, delta_collection);
            }
        }
    }
    Ok(base)
}

/// Restrict a snapshot to only the external_ids routed to one of `buckets`,
/// computed with the exact same `route_document` hash
/// `snapshot_reshard_batches` uses (`route_hash(collection_id, external_id) %
/// virtual_bucket_count`). Backs the bucket-scoped export admin verb (`POST
/// /admin/backup:scoped`, #1380 R2) so an export and the batches later
/// computed against the same map can never disagree about bucket
/// membership. `physical_shard_count` is irrelevant to bucket selection, so
/// callers only need to agree on `virtual_bucket_count`.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-reshard-rs.md#source
pub fn snapshot_bucket_subset(
    snapshot: &SnapshotV1,
    virtual_bucket_count: u32,
    buckets: &BTreeSet<u32>,
) -> Result<SnapshotV1> {
    let map = VirtualBucketShardMap::balanced(0, virtual_bucket_count, 1)?;
    let mut external_ids: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (collection_id, collection) in &snapshot.collections {
        for external_id in collection.eid_fields.keys() {
            let bucket = map.route_document(collection_id, None, external_id).bucket;
            if buckets.contains(&bucket) {
                external_ids
                    .entry(collection_id.clone())
                    .or_default()
                    .insert(external_id.clone());
            }
        }
    }
    snapshot_subset(snapshot, &external_ids)
}

fn ids_map_from_pairs(
    pairs: impl IntoIterator<Item = (String, String)>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (collection_id, external_id) in pairs {
        out.entry(collection_id).or_default().insert(external_id);
    }
    out
}

fn snapshot_subset(
    snapshot: &SnapshotV1,
    external_ids: &BTreeMap<String, BTreeSet<String>>,
) -> Result<SnapshotV1> {
    let mut collections = BTreeMap::new();
    for (collection_id, wanted) in external_ids {
        let Some(collection) = snapshot.collections.get(collection_id) else {
            bail!("snapshot missing collection `{collection_id}`");
        };
        let subset = collection_subset(collection, wanted);
        if !subset.eid_fields.is_empty() {
            collections.insert(collection_id.clone(), subset);
        }
    }
    Ok(SnapshotV1 {
        version: snapshot.version,
        collections,
    })
}

fn collection_subset(
    collection: &CollectionSnapshot,
    wanted: &BTreeSet<String>,
) -> CollectionSnapshot {
    CollectionSnapshot {
        schema: collection.schema.clone(),
        version: collection.version,
        eid_fields: collection
            .eid_fields
            .iter()
            .filter(|(external_id, _)| wanted.contains(*external_id))
            .map(|(external_id, fields)| (external_id.clone(), fields.clone()))
            .collect(),
        fields: collection
            .fields
            .iter()
            .map(|(field, index)| (field.clone(), field_index_subset(index, wanted)))
            .collect(),
    }
}

fn merge_collection_delta(base: &mut CollectionSnapshot, delta: CollectionSnapshot) -> Result<()> {
    if base.schema != delta.schema {
        bail!("cannot merge reshard snapshots with different collection schemas");
    }
    base.version = base.version.max(delta.version);
    base.eid_fields.extend(delta.eid_fields);
    for (field, delta_index) in delta.fields {
        match base.fields.get_mut(&field) {
            Some(base_index) => merge_field_index_delta(base_index, delta_index)?,
            None => {
                base.fields.insert(field, delta_index);
            }
        }
    }
    Ok(())
}

fn merge_field_index_delta(base: &mut FieldIndexSnapshot, delta: FieldIndexSnapshot) -> Result<()> {
    match (base, delta) {
        (
            FieldIndexSnapshot::Text {
                tokens,
                forward,
                doc_count,
                total_doc_len,
                bytes,
                ..
            },
            FieldIndexSnapshot::Text {
                tokens: delta_tokens,
                forward: delta_forward,
                bytes: delta_bytes,
                ..
            },
        ) => {
            for (token, postings) in delta_tokens {
                tokens.entry(token).or_default().extend(postings);
            }
            forward.extend(delta_forward);
            *doc_count = forward.len() as u64;
            *total_doc_len = forward.values().map(|(_, len)| u64::from(*len)).sum();
            *bytes = bytes.saturating_add(delta_bytes);
        }
        (
            FieldIndexSnapshot::Keyword {
                terms,
                forward,
                bytes,
            },
            FieldIndexSnapshot::Keyword {
                terms: delta_terms,
                forward: delta_forward,
                bytes: delta_bytes,
            },
        ) => {
            for (term, ids) in delta_terms {
                terms.entry(term).or_default().extend(ids);
            }
            forward.extend(delta_forward);
            *bytes = bytes.saturating_add(delta_bytes);
        }
        (
            FieldIndexSnapshot::Number { forward, bytes },
            FieldIndexSnapshot::Number {
                forward: delta_forward,
                bytes: delta_bytes,
            },
        ) => {
            forward.extend(delta_forward);
            *bytes = bytes.saturating_add(delta_bytes);
        }
        (
            FieldIndexSnapshot::Set {
                elements,
                forward,
                bytes,
            },
            FieldIndexSnapshot::Set {
                elements: delta_elements,
                forward: delta_forward,
                bytes: delta_bytes,
            },
        ) => {
            for (element, ids) in delta_elements {
                elements.entry(element).or_default().extend(ids);
            }
            forward.extend(delta_forward);
            *bytes = bytes.saturating_add(delta_bytes);
        }
        (
            FieldIndexSnapshot::Vector {
                spec,
                vectors,
                codebook,
                bytes,
            },
            FieldIndexSnapshot::Vector {
                spec: delta_spec,
                vectors: delta_vectors,
                codebook: delta_codebook,
                bytes: delta_bytes,
            },
        ) => {
            if *spec != delta_spec {
                bail!("cannot merge vector snapshots with different specs");
            }
            let delta_ids: BTreeSet<String> = delta_vectors
                .iter()
                .map(|(external_id, _)| external_id.clone())
                .collect();
            vectors.retain(|(external_id, _)| !delta_ids.contains(external_id));
            vectors.extend(delta_vectors);
            if codebook.is_none() {
                *codebook = delta_codebook;
            }
            *bytes = bytes.saturating_add(delta_bytes);
        }
        (
            FieldIndexSnapshot::Hash { forward, bytes },
            FieldIndexSnapshot::Hash {
                forward: delta_forward,
                bytes: delta_bytes,
            },
        ) => {
            forward.extend(delta_forward);
            *bytes = bytes.saturating_add(delta_bytes);
        }
        _ => bail!("cannot merge snapshots with different field index types"),
    }
    Ok(())
}

fn field_index_subset(index: &FieldIndexSnapshot, wanted: &BTreeSet<String>) -> FieldIndexSnapshot {
    match index {
        FieldIndexSnapshot::Text {
            analyzer,
            tokens,
            forward,
            bytes,
            ..
        } => {
            let forward: BTreeMap<String, (BTreeSet<String>, u32)> = forward
                .iter()
                .filter(|(external_id, _)| wanted.contains(*external_id))
                .map(|(external_id, value)| (external_id.clone(), value.clone()))
                .collect();
            let doc_count = forward.len() as u64;
            let total_doc_len = forward.values().map(|(_, len)| u64::from(*len)).sum();
            let tokens = tokens
                .iter()
                .filter_map(|(token, postings)| {
                    let postings: BTreeMap<String, u32> = postings
                        .iter()
                        .filter(|(external_id, _)| wanted.contains(*external_id))
                        .map(|(external_id, tf)| (external_id.clone(), *tf))
                        .collect();
                    (!postings.is_empty()).then(|| (token.clone(), postings))
                })
                .collect();
            FieldIndexSnapshot::Text {
                analyzer: *analyzer,
                tokens,
                forward: forward.into_iter().collect(),
                doc_count,
                total_doc_len,
                bytes: *bytes,
            }
        }
        FieldIndexSnapshot::Keyword {
            terms,
            forward,
            bytes,
        } => {
            let forward = forward
                .iter()
                .filter(|(external_id, _)| wanted.contains(*external_id))
                .map(|(external_id, value)| (external_id.clone(), value.clone()))
                .collect();
            let terms = terms
                .iter()
                .filter_map(|(term, ids)| {
                    let ids: BTreeSet<String> = ids
                        .iter()
                        .filter(|external_id| wanted.contains(*external_id))
                        .cloned()
                        .collect();
                    (!ids.is_empty()).then(|| (term.clone(), ids))
                })
                .collect();
            FieldIndexSnapshot::Keyword {
                terms,
                forward,
                bytes: *bytes,
            }
        }
        FieldIndexSnapshot::Number { forward, bytes } => FieldIndexSnapshot::Number {
            forward: forward
                .iter()
                .filter(|(external_id, _)| wanted.contains(*external_id))
                .map(|(external_id, value)| (external_id.clone(), *value))
                .collect(),
            bytes: *bytes,
        },
        FieldIndexSnapshot::Set {
            elements,
            forward,
            bytes,
        } => {
            let forward = forward
                .iter()
                .filter(|(external_id, _)| wanted.contains(*external_id))
                .map(|(external_id, value)| (external_id.clone(), value.clone()))
                .collect();
            let elements = elements
                .iter()
                .filter_map(|(element, ids)| {
                    let ids: BTreeSet<String> = ids
                        .iter()
                        .filter(|external_id| wanted.contains(*external_id))
                        .cloned()
                        .collect();
                    (!ids.is_empty()).then(|| (element.clone(), ids))
                })
                .collect();
            FieldIndexSnapshot::Set {
                elements,
                forward,
                bytes: *bytes,
            }
        }
        FieldIndexSnapshot::Vector {
            spec,
            vectors,
            codebook,
            bytes,
        } => FieldIndexSnapshot::Vector {
            spec: *spec,
            vectors: vectors
                .iter()
                .filter(|(external_id, _)| wanted.contains(external_id))
                .cloned()
                .collect(),
            codebook: *codebook,
            bytes: *bytes,
        },
        FieldIndexSnapshot::Hash { forward, bytes } => FieldIndexSnapshot::Hash {
            forward: forward
                .iter()
                .filter(|(external_id, _)| wanted.contains(*external_id))
                .map(|(external_id, value)| (external_id.clone(), *value))
                .collect(),
            bytes: *bytes,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::storage::Engine;
    use crate::types::{
        CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
        MatchOp, MatchQuery, QueryNode, SearchRequest,
    };

    use super::*;

    #[test]
    fn bucket_moves_reports_only_reassigned_buckets() {
        let from = VirtualBucketShardMap::new(1, vec![0, 0, 1, 1], 2).unwrap();
        let to = VirtualBucketShardMap::new(2, vec![0, 1, 1, 0], 2).unwrap();

        assert_eq!(
            bucket_moves(&from, &to).unwrap(),
            vec![
                BucketMove {
                    bucket: 1,
                    from_shard: 0,
                    to_shard: 1,
                },
                BucketMove {
                    bucket: 3,
                    from_shard: 1,
                    to_shard: 0,
                },
            ]
        );
    }

    #[test]
    fn snapshot_reshard_batches_emit_bounded_restorable_partials() {
        let collection_id = "users";
        let source = Engine::new();
        source
            .create_collection(
                collection_id,
                CreateCollectionRequest {
                    fields: BTreeMap::from([
                        ("body".into(), field(FieldType::Text)),
                        ("email".into(), field(FieldType::Keyword)),
                        ("age".into(), field(FieldType::Number)),
                        ("tags".into(), field(FieldType::Set)),
                        ("phash".into(), field(FieldType::Hash)),
                    ]),
                },
            )
            .unwrap();
        for i in 0..24 {
            let external_id = format!("doc-{i:02}");
            source
                .index(
                    collection_id,
                    IndexRequest {
                        request_id: None,
                        items: vec![
                            item(&external_id, "body", FieldValue::String("engineer".into())),
                            item(
                                &external_id,
                                "email",
                                FieldValue::String(format!("{external_id}@example.com")),
                            ),
                            item(&external_id, "age", FieldValue::Number(i as f64)),
                            item(
                                &external_id,
                                "tags",
                                FieldValue::StringList(vec!["blue".into(), "green".into()]),
                            ),
                            item(
                                &external_id,
                                "phash",
                                FieldValue::String(format!("{i:016x}")),
                            ),
                        ],
                    },
                )
                .unwrap();
        }

        let snapshot = source.snapshot().unwrap();
        let from = VirtualBucketShardMap::new(1, vec![0, 0, 0, 0], 1).unwrap();
        let to = VirtualBucketShardMap::new(2, vec![0, 1, 0, 1], 2).unwrap();
        let batches = snapshot_reshard_batches(&snapshot, &from, &to, 3).unwrap();

        assert!(!batches.is_empty());
        assert!(batches
            .iter()
            .all(|b| b.external_ids.values().map(BTreeSet::len).sum::<usize>() <= 3));
        assert!(batches.iter().any(|b| b.to_shard == 1));

        let mut target_snapshot = SnapshotV1 {
            version: snapshot.version,
            collections: BTreeMap::new(),
        };
        for batch in &batches {
            target_snapshot =
                merge_snapshot_delta(target_snapshot, batch.snapshot.clone()).unwrap();
        }
        let moved = Engine::new();
        moved.restore(target_snapshot).unwrap();
        let hits = moved
            .search(
                collection_id,
                SearchRequest {
                    query: QueryNode::Match(MatchQuery {
                        field: "body".into(),
                        text: "engineer".into(),
                        op: MatchOp::And,
                    }),
                    limit: 100,
                    cursor: None,
                    routing_key: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap()
            .hits
            .len();
        let moved_ids: usize = batches
            .iter()
            .map(|batch| {
                batch
                    .external_ids
                    .values()
                    .map(BTreeSet::len)
                    .sum::<usize>()
            })
            .sum();
        assert_eq!(hits, moved_ids);
        assert!(moved_ids < 24, "split should move only reassigned buckets");
    }

    #[test]
    fn snapshot_bucket_subset_matches_route_document_membership() {
        let collection_id = "users";
        let source = Engine::new();
        source
            .create_collection(
                collection_id,
                CreateCollectionRequest {
                    fields: BTreeMap::from([("email".into(), field(FieldType::Keyword))]),
                },
            )
            .unwrap();
        for i in 0..16 {
            let external_id = format!("doc-{i:02}");
            source
                .index(
                    collection_id,
                    IndexRequest {
                        request_id: None,
                        items: vec![item(
                            &external_id,
                            "email",
                            FieldValue::String(format!("{external_id}@example.com")),
                        )],
                    },
                )
                .unwrap();
        }
        let snapshot = source.snapshot().unwrap();
        let virtual_bucket_count = 8;
        let buckets = BTreeSet::from([0u32, 3]);
        let scoped = snapshot_bucket_subset(&snapshot, virtual_bucket_count, &buckets).unwrap();

        let map = VirtualBucketShardMap::balanced(0, virtual_bucket_count, 1).unwrap();
        let expected: BTreeSet<String> = snapshot.collections[collection_id]
            .eid_fields
            .keys()
            .filter(|external_id| {
                buckets.contains(&map.route_document(collection_id, None, external_id).bucket)
            })
            .cloned()
            .collect();

        let got: BTreeSet<String> = scoped
            .collections
            .get(collection_id)
            .map(|c| c.eid_fields.keys().cloned().collect())
            .unwrap_or_default();
        assert_eq!(got, expected);
        assert!(!expected.is_empty(), "test buckets should select some docs");
        assert!(
            expected.len() < 16,
            "bucket-scoped export should not return everything"
        );
    }

    /// AC4 (#1396 R4): a moved bucket's delta whose full snapshot serializes
    /// well over 8 MiB (large per-doc text bodies, not merely a high id
    /// count) still migrates completely via byte-capped batches — the
    /// fixture picks a document count/size that would collapse to a single
    /// oversized batch under the old id-count-only cap
    /// (`max_external_ids_per_batch` set high enough that byte size, not id
    /// count, is the binding constraint).
    #[test]
    fn snapshot_reshard_batches_splits_oversized_bucket_delta_by_bytes() {
        let collection_id = "docs";
        let source = Engine::new();
        source
            .create_collection(
                collection_id,
                CreateCollectionRequest {
                    fields: BTreeMap::from([("body".into(), field(FieldType::Text))]),
                },
            )
            .unwrap();

        // A text field's snapshot wire size is driven by its *inverted
        // index* (`FieldIndexSnapshot::Text`'s `forward`: per-doc unique
        // token set, and `tokens`: per-term postings) — repeating one word
        // many times within a doc collapses to a single unique token and
        // stays tiny on the wire, so this fixture instead gives every doc a
        // large, shared vocabulary of distinct tokens: ~2500 unique tokens
        // per doc across 200 docs comes to well over 8 MiB serialized
        // (~25 KiB/doc of forward token strings alone, plus per-term
        // postings), comfortably over both `MAX_BATCH_BYTES` (4 MiB) and
        // the route's 8 MiB body limit if emitted as one batch.
        const VOCAB_SIZE: usize = 2500;
        let vocab: Vec<String> = (0..VOCAB_SIZE).map(|i| format!("tok{i}")).collect();
        let big_body = vocab.join(" ");
        let ids: Vec<String> = (0..200).map(|i| format!("d-{i:04}")).collect();
        for id in &ids {
            source
                .index(
                    collection_id,
                    IndexRequest {
                        request_id: None,
                        items: vec![item(id, "body", FieldValue::String(big_body.clone()))],
                    },
                )
                .unwrap();
        }

        let snapshot = source.snapshot().unwrap();
        // Single physical shard -> two, everything in one bucket moves.
        let from = VirtualBucketShardMap::new(1, vec![0], 1).unwrap();
        let to = VirtualBucketShardMap::new(2, vec![1], 2).unwrap();
        // A generous id-count cap so byte size, not id count, is what
        // forces the split.
        let batches = snapshot_reshard_batches(&snapshot, &from, &to, 10_000).unwrap();

        assert!(
            batches.len() > 1,
            "expected the oversized delta to split into more than one batch, got {}",
            batches.len()
        );
        for batch in &batches {
            let wire_bytes = serde_json::to_vec(batch).unwrap().len();
            assert!(
                wire_bytes < ADMIN_ROUTE_BODY_LIMIT_BYTES,
                "batch serialized to {wire_bytes} bytes, over the route's {ADMIN_ROUTE_BODY_LIMIT_BYTES} byte body limit"
            );
        }

        // Every id made it into exactly one batch, and merging them back
        // together restores every document.
        let moved_ids: BTreeSet<String> = batches
            .iter()
            .flat_map(|b| b.external_ids.values().flat_map(|s| s.iter().cloned()))
            .collect();
        assert_eq!(moved_ids, ids.iter().cloned().collect::<BTreeSet<_>>());

        let mut target_snapshot = SnapshotV1 {
            version: snapshot.version,
            collections: BTreeMap::new(),
        };
        for batch in &batches {
            target_snapshot =
                merge_snapshot_delta(target_snapshot, batch.snapshot.clone()).unwrap();
        }
        let moved = Engine::new();
        moved.restore(target_snapshot).unwrap();
        let hits = moved
            .search(
                collection_id,
                SearchRequest {
                    query: QueryNode::Match(MatchQuery {
                        field: "body".into(),
                        text: "tok0".into(),
                        op: MatchOp::And,
                    }),
                    limit: 1000,
                    cursor: None,
                    routing_key: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap()
            .hits
            .len();
        assert_eq!(
            hits,
            ids.len(),
            "every moved document should be restorable from the byte-capped batches"
        );
    }

    /// AC1 (#1457 R1): a bucket group whose id set ALONE serializes past a
    /// small byte cap must still complete via multiple independently
    /// bounded prune chunks (every chunk under the route's hard body
    /// limit), and the union of every chunk's `keep_ids` for a
    /// `(bucket, collection_id)` group reconstructs the full authoritative
    /// set regardless of how many chunks it split into.
    #[test]
    fn snapshot_reshard_prune_chunks_splits_large_keep_set_by_bytes() {
        let collection_id = "docs";
        let source = Engine::new();
        source
            .create_collection(
                collection_id,
                CreateCollectionRequest {
                    fields: BTreeMap::from([("email".into(), field(FieldType::Keyword))]),
                },
            )
            .unwrap();
        let ids: Vec<String> = (0..20_000)
            .map(|i| format!("doc-with-a-fairly-long-external-id-{i:06}"))
            .collect();
        for id in &ids {
            source
                .index(
                    collection_id,
                    IndexRequest {
                        request_id: None,
                        items: vec![item(
                            id,
                            "email",
                            FieldValue::String(format!("{id}@example.com")),
                        )],
                    },
                )
                .unwrap();
        }
        let snapshot = source.snapshot().unwrap();
        let to = VirtualBucketShardMap::new(2, vec![0, 1], 2).unwrap();
        let buckets = BTreeSet::from([0u32, 1u32]);
        let collection_ids = BTreeSet::from([collection_id.to_string()]);
        const SMALL_CAP: usize = 64 * 1024;
        let chunks =
            snapshot_reshard_prune_chunks(&snapshot, &to, &buckets, &collection_ids, SMALL_CAP)
                .unwrap();

        assert!(
            chunks.len() > 2,
            "expected the large keep set to split into multiple chunks, got {}",
            chunks.len()
        );
        for chunk in &chunks {
            let wire_bytes = serde_json::to_vec(chunk).unwrap().len();
            assert!(
                wire_bytes < ADMIN_ROUTE_BODY_LIMIT_BYTES,
                "chunk serialized to {wire_bytes} bytes, over the route's {ADMIN_ROUTE_BODY_LIMIT_BYTES} byte body limit"
            );
        }

        let mut by_group: BTreeMap<(u32, String), Vec<&ReshardPruneChunk>> = BTreeMap::new();
        for chunk in &chunks {
            by_group
                .entry((chunk.bucket, chunk.collection_id.clone()))
                .or_default()
                .push(chunk);
        }
        let mut reconstructed: BTreeSet<String> = BTreeSet::new();
        for group in by_group.values() {
            let total = group[0].total_chunks;
            let mut seen: BTreeSet<u32> = BTreeSet::new();
            for c in group {
                assert_eq!(c.total_chunks, total);
                assert!(seen.insert(c.chunk_index), "duplicate chunk_index");
                reconstructed.extend(c.keep_ids.iter().cloned());
            }
            assert_eq!(seen, (0..total).collect::<BTreeSet<_>>());
        }
        assert_eq!(reconstructed, ids.iter().cloned().collect::<BTreeSet<_>>());
    }

    /// AC2 (#1457 R2, the edge #1443 disclosed): a `(bucket, collection_id)`
    /// pair with zero matching docs in the snapshot — modeling a collection
    /// whose every moved-bucket document was deleted before the final pass —
    /// still gets exactly one chunk carrying an empty `keep_ids`, so a
    /// receiver still prunes any stale copies it holds rather than the pair
    /// being silently omitted because the snapshot has nothing to say about
    /// it.
    #[test]
    fn snapshot_reshard_prune_chunks_emits_empty_scope_for_emptied_collection() {
        let empty_collection_id = "emptied";
        let populated_collection_id = "populated";
        let source = Engine::new();
        for collection_id in [empty_collection_id, populated_collection_id] {
            source
                .create_collection(
                    collection_id,
                    CreateCollectionRequest {
                        fields: BTreeMap::from([("email".into(), field(FieldType::Keyword))]),
                    },
                )
                .unwrap();
        }
        source
            .index(
                populated_collection_id,
                IndexRequest {
                    request_id: None,
                    items: vec![item(
                        "doc-1",
                        "email",
                        FieldValue::String("doc-1@example.com".into()),
                    )],
                },
            )
            .unwrap();
        // `empty_collection_id` stays empty: models a collection whose only
        // moved-bucket docs were all deleted before the final pass.
        let snapshot = source.snapshot().unwrap();
        let to = VirtualBucketShardMap::new(1, vec![0], 1).unwrap();
        let buckets = BTreeSet::from([0u32]);
        let collection_ids = BTreeSet::from([
            empty_collection_id.to_string(),
            populated_collection_id.to_string(),
        ]);
        let chunks = snapshot_reshard_prune_chunks(
            &snapshot,
            &to,
            &buckets,
            &collection_ids,
            MAX_BATCH_BYTES,
        )
        .unwrap();

        let empty_chunks: Vec<&ReshardPruneChunk> = chunks
            .iter()
            .filter(|c| c.collection_id == empty_collection_id)
            .collect();
        assert_eq!(
            empty_chunks.len(),
            1,
            "an emptied collection must still get exactly one (empty) chunk"
        );
        assert_eq!(empty_chunks[0].total_chunks, 1);
        assert!(empty_chunks[0].keep_ids.is_empty());
        assert_eq!(empty_chunks[0].bucket, 0);

        let populated_chunks: Vec<&ReshardPruneChunk> = chunks
            .iter()
            .filter(|c| c.collection_id == populated_collection_id)
            .collect();
        assert_eq!(populated_chunks.len(), 1);
        assert_eq!(
            populated_chunks[0].keep_ids,
            BTreeSet::from(["doc-1".to_string()])
        );
    }

    fn field(field_type: FieldType) -> FieldSpec {
        FieldSpec {
            field_type,
            analyzer: None,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        }
    }

    fn item(external_id: &str, field: &str, value: FieldValue) -> IndexItem {
        IndexItem {
            external_id: external_id.into(),
            field: field.into(),
            value,
            version: None,
        }
    }
}
// CODEGEN-END
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: "projects/lumen/src/reshard.rs"
    action: modify
    section: rust-source-unit
    description: |
      Existing reshard source is captured as a per-file rust-source-unit so
      dynamic shard topology has semantic and codegen traceability.
    impl_mode: codegen
  - path: "projects/lumen/src/reshard.rs"
    action: modify
    section: rust-source-unit
    description: |
      #1396 R4: cap `snapshot_reshard_batches`'s emitted batches by
      serialized byte size (`MAX_BATCH_BYTES`, 4 MiB), not only by
      external-id count, so a bucket of large documents can no longer
      serialize past `api.rs`'s 8 MiB `/admin/reshard:apply` body limit
      and wedge a split forever on a deterministic 413. A new private
      `byte_cap_chunk` helper recursively halves an id-chunk until each
      emitted batch's snapshot is at or under the cap (or down to one
      document, which is emitted as-is since it cannot be split further).
    impl_mode: hand-written
  - path: "projects/lumen/src/reshard.rs"
    action: modify
    section: rust-source-unit
    description: |
      #1443 R2: close a delete-resurrection gap in the write-fence
      hardening pass. `snapshot_reshard_batches` gained a `replace_mode:
      bool` parameter; when `true` (the reshard driver's final fenced
      `CatchingUp` pass only), the complete authoritative external-id set
      routed to each `(bucket, from_shard, to_shard)` group is captured
      once, before that group is consumed/chunked, and stamped as new
      `ReshardBatch.virtual_bucket_count`/`replace_ids` fields onto every
      byte-capped chunk of that group — so a bucket spanning multiple
      chunks still converges correctly regardless of application order.
      New `ReshardBatchReplaceScope` struct is the applying shard's
      typed view of that scope (bucket + virtual_bucket_count +
      replace_ids), consumed by
      `crate::storage::Engine::apply_reshard_batch`'s new pruning step.
      Every non-final pass still uses purely-additive merge
      (`replace_mode: false`, `replace_ids: None`), unchanged.
    impl_mode: hand-written
  - path: "projects/lumen/src/reshard.rs"
    action: modify
    section: rust-source-unit
    description: |
      #1444 R2: named `ADMIN_ROUTE_BODY_LIMIT_BYTES` as the single source
      of truth for the route's body-size limit, shared verbatim with
      `api.rs`'s `DefaultBodyLimit::max(..)` (previously a hand-copied `8
      MiB` literal in `api.rs` and a `4 MiB` (`ADMIN_ROUTE_BODY_LIMIT_BYTES
      / 2`) literal here) so they can never drift apart. `MAX_BATCH_BYTES`
      is now defined as `ADMIN_ROUTE_BODY_LIMIT_BYTES / 2` instead of a
      separate hardcoded `4 * 1024 * 1024`.
      `crate::operator::reshard_driver` compares a batch's real wire size
      against this same constant to distinguish the oversize-single-document
      wedge (`byte_cap_chunk`'s single-id floor case) from any other apply
      failure.
    impl_mode: hand-written
  - path: "projects/lumen/src/reshard.rs"
    action: modify
    section: rust-source-unit
    description: |
      #1457 R1: reverted `ReshardBatch` to purely additive — the #1443 R2
      `virtual_bucket_count`/`replace_ids` fields were removed. Stamping
      the complete authoritative id set onto every byte-capped chunk of a
      moving bucket made the final pass's wire size scale with total
      bucket population rather than the chunk's own content: a bucket
      whose id set alone serialized past `MAX_BATCH_BYTES` produced
      chunks over the route's hard body limit no matter how small
      `snapshot`/`external_ids` were, and
      `operator::reshard_driver::detect_oversized_batch` wrongly blamed
      whichever document happened to be first in the chunk. The
      authoritative-replace concern moved to a new, independently
      byte-capped message type: `ReshardPruneChunk`, built by the new
      `snapshot_reshard_prune_chunks` (replacing `snapshot_reshard_batches`'s
      removed `replace_mode: bool` parameter). Each chunk carries one
      `(bucket, collection_id)` pair's `keep_ids` slice plus
      `chunk_index`/`total_chunks`; the receiver
      (`crate::storage::Engine::apply_reshard_prune_chunk`) accumulates by
      `(to_map_version, bucket, collection_id, total_chunks)` and prunes
      only once every chunk has arrived, so a 413 retry on any subset
      converges. #1457 R2: `snapshot_reshard_prune_chunks` is pre-seeded
      with an empty `keep_ids` entry for every `(bucket, collection_id)`
      pair in `buckets` × the caller-supplied full `collection_ids` list
      (not derived from the bucket-scoped snapshot, which omits a
      collection entirely when it has zero matching docs) — so a
      collection a batch of deletes emptied entirely inside a moved
      bucket still gets a chunk pruning its stale copies on the target,
      closing the #1443-disclosed delete-resurrection edge (#1443's own
      fix only prunes ids that *do* appear in the final snapshot).
    impl_mode: hand-written
```
