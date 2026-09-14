//! Allocation-free upper bound for one decoded owned RaftLogEntry.
//!
//! It counts allocations reachable from this one record. It excludes a separate
//! AOF buffer, transport copies, normalized index state, and vector graphs.

use std::collections::BTreeMap;
use std::mem::size_of;

use crate::log_entry::RaftLogEntry;
use crate::types::{
    BatchUnindexDocsRequest, CreateCollectionRequest, FieldSpec, FieldValue, IndexItem,
    IndexRequest, ReplaceDocItem, ReplaceDocsRequest,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordRamError {
    Overflow,
}

// Rust 1.96 alloc::collections::btree uses B=6 and CAPACITY=11. A one-entry
// map owns a full leaf node. We charge a full worst-case node per entry:
// 11 key slots, 11 value slots, 12 edge pointers, plus a conservative two-word
// allocator header. This intentionally overcounts dense maps but bounds sparse
// nodes, partial nodes, bookkeeping, alignment, and internal-node edges.
const BTREE_CAPACITY: usize = 11;
const BTREE_EDGES: usize = BTREE_CAPACITY + 1;
const ALLOCATION_HEADER: usize = 2 * size_of::<usize>();

fn add(a: usize, b: usize) -> Result<usize, RecordRamError> {
    a.checked_add(b).ok_or(RecordRamError::Overflow)
}
fn mul(a: usize, b: usize) -> Result<usize, RecordRamError> {
    a.checked_mul(b).ok_or(RecordRamError::Overflow)
}
fn allocation(payload: usize) -> Result<usize, RecordRamError> {
    if payload == 0 {
        Ok(0)
    } else {
        add(payload, ALLOCATION_HEADER)
    }
}
fn string(value: &String) -> Result<usize, RecordRamError> {
    allocation(value.capacity())
}
fn vec<T>(value: &Vec<T>) -> Result<usize, RecordRamError> {
    allocation(mul(value.capacity(), size_of::<T>())?)
}

pub fn estimate_record_ram(entry: &RaftLogEntry) -> Result<usize, RecordRamError> {
    let nested = match entry {
        RaftLogEntry::CreateCollection { collection_id, req } => {
            add(string(collection_id)?, create(req)?)?
        }
        RaftLogEntry::Index { collection_id, req } => add(string(collection_id)?, index(req)?)?,
        RaftLogEntry::ReplaceDocs { collection_id, req } => {
            add(string(collection_id)?, replace(req)?)?
        }
        RaftLogEntry::TruncateDocs { collection_id }
        | RaftLogEntry::DropCollection { collection_id, .. } => string(collection_id)?,
        RaftLogEntry::UnindexDocs { collection_id, req } => {
            add(string(collection_id)?, unindex(req)?)?
        }
        RaftLogEntry::Delete {
            collection_id,
            external_id,
            field,
        } => {
            let mut total = add(string(collection_id)?, string(external_id)?)?;
            if let Some(field) = field {
                total = add(total, string(field)?)?;
            }
            total
        }
        RaftLogEntry::AddField {
            collection_id,
            field_name,
            spec,
        } => add(
            add(string(collection_id)?, string(field_name)?)?,
            spec_ram(spec)?,
        )?,
        RaftLogEntry::DropField {
            collection_id,
            field_name,
        } => add(string(collection_id)?, string(field_name)?)?,
    };
    add(size_of::<RaftLogEntry>(), nested)
}

/// Peak for the private CBOR decoder, including its untagged value buffering.
pub fn estimate_record_decode_peak(entry: &RaftLogEntry) -> Result<usize, RecordRamError> {
    let mut total = mul(estimate_record_ram(entry)?, 3)?;
    let content = |value: &FieldValue| -> Result<usize, RecordRamError> {
        let count = match value {
            FieldValue::Vector(values) => values.len(),
            FieldValue::StringList(values) => values.len(),
            _ => return Ok(0),
        };
        if count == 0 {
            return Ok(0);
        }
        // serde1.0.228 first builds Vec<Content>, then tries the owned
        // untagged alternatives. Eight words bound Content's three-word
        // payload, discriminant and alignment. Its cautious size hint can
        // force geometric Vec growth even for a definite CBOR array. Include
        // both allocations during reallocation, independent of final values.
        allocation(mul(mul(count.max(4), 8 * size_of::<usize>())?, 3)?)
    };
    match entry {
        RaftLogEntry::Index { req, .. } => {
            for item in &req.items {
                total = add(total, content(&item.value)?)?;
            }
        }
        RaftLogEntry::ReplaceDocs { req, .. } => {
            for doc in &req.docs {
                for value in doc.fields.values() {
                    total = add(total, content(value)?)?;
                }
            }
        }
        _ => (),
    }
    Ok(total)
}

fn create(req: &CreateCollectionRequest) -> Result<usize, RecordRamError> {
    map(&req.fields, |key, spec| add(string(key)?, spec_ram(spec)?))
}
fn index(req: &IndexRequest) -> Result<usize, RecordRamError> {
    let mut total = vec(&req.items)?;
    if let Some(id) = &req.request_id {
        total = add(total, string(id)?)?;
    }
    for item in &req.items {
        total = add(total, index_item(item)?)?;
    }
    Ok(total)
}
fn index_item(item: &IndexItem) -> Result<usize, RecordRamError> {
    add(
        add(string(&item.external_id)?, string(&item.field)?)?,
        value(&item.value)?,
    )
}
fn replace(req: &ReplaceDocsRequest) -> Result<usize, RecordRamError> {
    let mut total = vec(&req.docs)?;
    for doc in &req.docs {
        total = add(total, replace_doc(doc)?)?;
    }
    Ok(total)
}
fn replace_doc(doc: &ReplaceDocItem) -> Result<usize, RecordRamError> {
    add(
        string(&doc.external_id)?,
        map(&doc.fields, |key, field_value| {
            add(string(key)?, value(field_value)?)
        })?,
    )
}
fn unindex(req: &BatchUnindexDocsRequest) -> Result<usize, RecordRamError> {
    let mut total = vec(&req.external_ids)?;
    for id in &req.external_ids {
        total = add(total, string(id)?)?;
    }
    Ok(total)
}
fn spec_ram(_spec: &FieldSpec) -> Result<usize, RecordRamError> {
    Ok(0)
}
fn value(field_value: &FieldValue) -> Result<usize, RecordRamError> {
    match field_value {
        FieldValue::String(value) => string(value),
        FieldValue::Number(_) => Ok(0),
        FieldValue::Vector(values) => vec(values),
        FieldValue::StringList(values) => {
            let mut total = vec(values)?;
            for value in values {
                total = add(total, string(value)?)?;
            }
            Ok(total)
        }
    }
}
fn full_node_bytes<K, V>() -> Result<usize, RecordRamError> {
    allocation(add(
        mul(BTREE_CAPACITY, add(size_of::<K>(), size_of::<V>())?)?,
        mul(BTREE_EDGES, size_of::<usize>())?,
    )?)
}
fn map<K, V>(
    value: &BTreeMap<K, V>,
    mut entry: impl FnMut(&K, &V) -> Result<usize, RecordRamError>,
) -> Result<usize, RecordRamError> {
    let mut total = mul(value.len(), full_node_bytes::<K, V>()?)?;
    for (key, value) in value {
        total = add(total, entry(key, value)?)?;
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FieldType;

    #[test]
    fn staged_numeric_vector_bound_includes_serde_content_and_output_together() {
        let values = 10_000;
        let entry = RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: "id".into(),
                    field: "v".into(),
                    value: FieldValue::Vector(vec![0.5; values]),
                    version: None,
                }],
                request_id: None,
            },
        };
        // serde1.0.228 buffers each element in Content before attempting the
        // Vec<f32> alternative. Content has a Vec/String-sized variant plus
        // a discriminant, while the final f32 allocation is also live.
        let unavoidable_peak = values * (4 * size_of::<usize>() + size_of::<f32>());
        let bound = estimate_record_decode_peak(&entry).unwrap();
        assert!(
            bound >= unavoidable_peak,
            "staged decoder bound {bound} omits buffered Content; minimum {unavoidable_peak}"
        );
    }
    fn keyword() -> FieldSpec {
        FieldSpec {
            field_type: FieldType::Keyword,
            analyzer: None,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        }
    }

    #[test]
    fn raw_hash_string_counts_even_when_normalized_hash_is_tiny() {
        let mut raw = String::with_capacity(65_536);
        raw.push_str("0000000000000001");
        let entry = RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: "id".into(),
                    field: "hash".into(),
                    value: FieldValue::String(raw),
                    version: None,
                }],
                request_id: None,
            },
        };
        assert!(estimate_record_ram(&entry).unwrap() >= 65_536);
    }
    #[test]
    fn spare_vector_and_string_capacities_are_charged() {
        let mut id = String::with_capacity(1024);
        id.push('x');
        let mut values = Vec::with_capacity(128);
        values.push(1.0);
        let entry = RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: id,
                    field: "vector".into(),
                    value: FieldValue::Vector(values),
                    version: None,
                }],
                request_id: None,
            },
        };
        assert!(estimate_record_ram(&entry).unwrap() >= 1024 + 128 * size_of::<f32>());
    }
    #[test]
    fn index_replace_unindex_and_schema_variants_walk_owned_heaps() {
        let index = RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                items: vec![],
                request_id: Some("r".into()),
            },
        };
        let replace = RaftLogEntry::ReplaceDocs {
            collection_id: "c".into(),
            req: ReplaceDocsRequest {
                docs: vec![ReplaceDocItem {
                    external_id: "id".into(),
                    version: None,
                    fields: BTreeMap::from([("tag".into(), FieldValue::String("v".into()))]),
                }],
            },
        };
        let unindex = RaftLogEntry::UnindexDocs {
            collection_id: "c".into(),
            req: BatchUnindexDocsRequest {
                external_ids: vec!["id".into()],
            },
        };
        let schema = RaftLogEntry::CreateCollection {
            collection_id: "c".into(),
            req: CreateCollectionRequest {
                fields: BTreeMap::from([("tag".into(), keyword())]),
            },
        };
        for entry in [&index, &replace, &unindex, &schema] {
            assert!(estimate_record_ram(entry).unwrap() > size_of::<RaftLogEntry>());
        }
    }

    #[test]
    fn every_log_variant_counts_its_owned_strings_or_containers() {
        let create = RaftLogEntry::CreateCollection {
            collection_id: "create".into(),
            req: CreateCollectionRequest {
                fields: BTreeMap::from([("tag".into(), keyword())]),
            },
        };
        let index = RaftLogEntry::Index {
            collection_id: "index".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: "id".into(),
                    field: "tag".into(),
                    value: FieldValue::StringList(vec!["one".into(), "two".into()]),
                    version: None,
                }],
                request_id: Some("request".into()),
            },
        };
        let replace = RaftLogEntry::ReplaceDocs {
            collection_id: "replace".into(),
            req: ReplaceDocsRequest {
                docs: vec![ReplaceDocItem {
                    external_id: "id".into(),
                    version: None,
                    fields: BTreeMap::from([("tag".into(), FieldValue::String("value".into()))]),
                }],
            },
        };
        let truncate = RaftLogEntry::TruncateDocs {
            collection_id: "truncate".into(),
        };
        let unindex = RaftLogEntry::UnindexDocs {
            collection_id: "unindex".into(),
            req: BatchUnindexDocsRequest {
                external_ids: vec!["id".into()],
            },
        };
        let delete = RaftLogEntry::Delete {
            collection_id: "delete".into(),
            external_id: "id".into(),
            field: Some("tag".into()),
        };
        let drop_collection = RaftLogEntry::DropCollection {
            collection_id: "drop".into(),
            force: false,
        };
        let add_field = RaftLogEntry::AddField {
            collection_id: "add".into(),
            field_name: "tag".into(),
            spec: keyword(),
        };
        let drop_field = RaftLogEntry::DropField {
            collection_id: "drop-field".into(),
            field_name: "tag".into(),
        };
        for entry in [
            &create,
            &index,
            &replace,
            &truncate,
            &unindex,
            &delete,
            &drop_collection,
            &add_field,
            &drop_field,
        ] {
            assert!(estimate_record_ram(entry).unwrap() > size_of::<RaftLogEntry>());
        }
    }
    #[test]
    fn one_entry_map_charges_a_full_sparse_node_not_only_its_live_pair() {
        let fields = BTreeMap::from([("tag".to_owned(), FieldValue::String("v".into()))]);
        assert!(
            map(&fields, |_, _| Ok(0)).unwrap() >= full_node_bytes::<String, FieldValue>().unwrap()
        );
    }
    #[test]
    fn checked_helpers_refuse_overflow() {
        assert_eq!(add(usize::MAX, 1), Err(RecordRamError::Overflow));
        assert_eq!(mul(usize::MAX, 2), Err(RecordRamError::Overflow));
    }
}
