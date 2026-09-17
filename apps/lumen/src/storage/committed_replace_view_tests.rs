//! Colocated unit fixtures for `committed_replace_view.rs` integration.

use super::*;
use crate::log_entry::RaftLogEntry;
use crate::types::{FieldValue, IndexItem, IndexRequest};
use crate::wal::fast_index_scanner::{FastIndexScanner, FastIndexValue};
use crate::wal::WalRecord;
use std::collections::BTreeSet;

fn source_set(values: Vec<&str>) -> Vec<u8> {
    WalRecord::new(RaftLogEntry::Index {
        collection_id: "docs".into(),
        req: IndexRequest {
            request_id: None,
            items: vec![IndexItem {
                external_id: "id".into(),
                field: "set".into(),
                version: None,
                value: FieldValue::StringList(values.into_iter().map(str::to_owned).collect()),
            }],
        },
    })
    .encode()
    .unwrap()
}

#[test]
fn keyword_reads_dense_and_sparse_tail_without_a_string_clone() {
    let mut index = KeywordIndex::default();
    index.dense_forward.resize(8, None);
    index.dense_forward[7] = Some("dense".into());
    index.forward.insert(9, "sparse".into());
    assert!(keyword_equal(&index, 7, "dense"));
    assert!(!keyword_equal(&index, 7, "other"));
    assert!(keyword_equal(&index, 9, "sparse"));
    assert!(!keyword_equal(&index, 9, "other"));
}

#[test]
fn set_compares_logical_unique_source_members_and_distinguishes_empty_from_missing() {
    let source = source_set(vec!["b", "a", "b"]);
    let scan = FastIndexScanner::parse(&source).unwrap();
    let FastIndexValue::StringList(source) = scan.items().next().unwrap().value else {
        panic!()
    };
    let mut index = SetIndex::default();
    index
        .forward
        .insert(5, BTreeSet::from(["a".into(), "b".into()]));
    assert!(set_equal(&index, 5, source));
    let empty_source = source_set(vec![]);
    let empty_scan = FastIndexScanner::parse(&empty_source).unwrap();
    let FastIndexValue::StringList(empty) = empty_scan.items().next().unwrap().value else {
        panic!()
    };
    index.forward.insert(6, BTreeSet::new());
    assert!(set_equal(&index, 6, empty));
    assert!(!set_equal(&index, 99, empty));
}

// Integration fixtures still required: sealed segment Keyword/Set rows,
// checksum/hash branches through `View`, and a FieldCoverage byte-sum
// overflow seam. They require the root-owned Collection construction helper.
