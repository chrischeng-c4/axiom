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

use crate::routing::VirtualBucketShardMap;
use crate::storage::{CollectionSnapshot, FieldIndexSnapshot, SnapshotV1};

#[derive(Clone, Debug, PartialEq, Eq)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-reshard-rs.md#source
pub struct BucketMove {
    pub bucket: u32,
    pub from_shard: u32,
    pub to_shard: u32,
}

#[derive(Clone, Debug)]
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
            let external_ids = ids_map_from_pairs(chunk.iter().cloned());
            let partial = snapshot_subset(snapshot, &external_ids)?;
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

    Ok(batches)
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
