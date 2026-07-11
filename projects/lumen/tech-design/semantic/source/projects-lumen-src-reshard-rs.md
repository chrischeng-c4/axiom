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
| `MAX_BATCH_BYTES` | projects/lumen/src/reshard.rs | const | pub | 29 | #1396 R4: upper bound on one batch's serialized `snapshot` payload, well under `api.rs`'s 8 MiB `DefaultBodyLimit` on `/admin/reshard:apply`. |
| `BucketMove` | projects/lumen/src/reshard.rs | struct | pub | 33 |  |
| `ReshardBatch` | projects/lumen/src/reshard.rs | struct | pub | 44 | #1380: `Serialize`/`Deserialize` make a batch postable to `POST /admin/reshard:apply` as-is. |
| `bucket_moves` | projects/lumen/src/reshard.rs | function | pub | 58 | bucket_moves(     from: &VirtualBucketShardMap,     to: &VirtualBucketShardMap, ) -> Result<Vec<BucketMove>> |
| `snapshot_reshard_batches` | projects/lumen/src/reshard.rs | function | pub | 94 | #1396 R4: batches are now also byte-capped (`MAX_BATCH_BYTES`), splitting an id-chunk further via `byte_cap_chunk` when its serialized snapshot would exceed the cap. snapshot_reshard_batches(     snapshot: &SnapshotV1,     from: &VirtualBucketShardMap,     to: &VirtualBucketShardMap,     max_external_ids_per_batch: usize, ) -> Result<Vec<ReshardBatch>> |
| `merge_snapshot_delta` | projects/lumen/src/reshard.rs | function | pub | 204 | merge_snapshot_delta(mut base: SnapshotV1, delta: SnapshotV1) -> Result<SnapshotV1> |
| `snapshot_bucket_subset` | projects/lumen/src/reshard.rs | function | pub | 232 | #1380 R2: bucket-scoped export subset, routed with the same `route_document` hash `snapshot_reshard_batches` uses. snapshot_bucket_subset(     snapshot: &SnapshotV1,     virtual_bucket_count: u32,     buckets: &BTreeSet<u32>, ) -> Result<SnapshotV1> |
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

/// Upper bound on one batch's serialized `snapshot` payload (#1396 R4):
/// `api.rs`'s `/admin/reshard:apply` route sits behind an 8 MiB
/// `DefaultBodyLimit`, but [`snapshot_reshard_batches`] used to cap batches
/// only by external-id count (`MAX_EXTERNAL_IDS_PER_BATCH`-style caller
/// constants) — a bucket of large documents (long text fields, vectors,
/// hashes) can still serialize well past 8 MiB even at a small id count, and
/// a 413 from an oversized batch is deterministically recomputed identically
/// every driver tick, wedging the split forever (the confirmed defect).
/// 4 MiB — half the route limit — leaves comfortable headroom for JSON/wire
/// overhead and per-item framing above the raw snapshot bytes measured here.
pub const MAX_BATCH_BYTES: usize = 4 * 1024 * 1024;

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

/// Build bounded snapshot batches for documents that move under `to`.
/// Batches are grouped by `(bucket, from_shard, to_shard)` and capped by
/// `max_external_ids_per_batch`, so an operator can checkpoint progress after
/// every emitted batch instead of blocking on one full-shard copy.
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

/// Recursively halve `chunk` (already `<= max_external_ids_per_batch` ids)
/// until each emitted `(external_ids, snapshot)` pair's serialized snapshot
/// is at or under `max_batch_bytes`, or the chunk is down to a single
/// external_id — one oversized document cannot be split further, so it is
/// emitted as its own (over-budget) batch rather than looping forever; a
/// single document that alone exceeds the route's 8 MiB body limit is a
/// data-modeling problem this splitter cannot solve, not a batching bug.
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
                wire_bytes < 8 * 1024 * 1024,
                "batch serialized to {wire_bytes} bytes, over the route's 8 MiB body limit"
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
```
