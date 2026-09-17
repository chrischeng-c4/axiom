//! Colocated fixtures for `committed_replace_plan.rs` after root integration.
//!
//! These tests use the existing fast Index encoder only as the private
//! flattened source image. The production Replace adapter supplies the same
//! image and the descriptors from its CBOR scanner.

use super::*;
use crate::log_entry::RaftLogEntry;
use crate::types::{FieldValue, IndexItem, IndexRequest};
use crate::wal::WalRecord;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct View {
    fields: BTreeMap<String, FieldType>,
    ids: BTreeMap<String, u32>,
    coverage: BTreeMap<u32, Vec<String>>,
    versions: BTreeMap<u32, u64>,
    unchanged: BTreeSet<(u32, String)>,
}
impl PlanView for View {
    fn engine_epoch(&self) -> u64 {
        1
    }
    fn collection_generation(&self) -> u64 {
        2
    }
    fn schema_version(&self) -> u32 {
        3
    }
    fn data_version(&self) -> u64 {
        4
    }
    fn revision(&self) -> u64 {
        5
    }
    fn interner_len(&self) -> usize {
        10
    }
    fn is_live(&self) -> bool {
        true
    }
    fn field_type(&self, f: &str) -> Option<FieldType> {
        self.fields.get(f).copied()
    }
    fn vector_dimension(&self, f: &str) -> Option<u32> {
        (f == "vec").then_some(2)
    }
    fn id(&self, e: &str) -> Option<u32> {
        self.ids.get(e).copied()
    }
    fn has_cell(&self, id: u32, f: &str) -> bool {
        self.coverage
            .get(&id)
            .is_some_and(|v| v.iter().any(|x| x == f))
    }
    fn cell_version(&self, _: u32, _: &str) -> Option<u64> {
        None
    }
    fn request_deadline(&self, _: &str) -> Option<Instant> {
        None
    }
}
impl ReplacePlanView for View {
    fn doc_version(&self, id: u32) -> Option<u64> {
        self.versions.get(&id).copied()
    }
    fn old_fields(&self, id: u32) -> Option<&[String]> {
        self.coverage.get(&id).map(Vec::as_slice)
    }
    fn old_fields_bound(&self, id: u32) -> OldFieldsBound {
        let fields = self.old_fields(id).unwrap_or_default();
        OldFieldsBound {
            count: fields.len(),
            copied_bytes: fields.iter().map(String::len).sum(),
        }
    }
    fn existing_unchanged(&self, id: u32, f: &str, _: FieldType, _: usize, _: Option<u64>) -> bool {
        self.unchanged.contains(&(id, f.into()))
    }
}
fn wire(items: Vec<IndexItem>) -> Vec<u8> {
    WalRecord::new(RaftLogEntry::Index {
        collection_id: "docs".into(),
        req: IndexRequest {
            request_id: None,
            items,
        },
    })
    .encode()
    .unwrap()
}
fn item(id: &str, field: &str, value: FieldValue) -> IndexItem {
    IndexItem {
        external_id: id.into(),
        field: field.into(),
        value,
        version: None,
    }
}
fn view() -> View {
    View {
        fields: BTreeMap::from([
            ("kw".into(), FieldType::Keyword),
            ("text".into(), FieldType::Text),
            ("vec".into(), FieldType::Vector),
        ]),
        ids: BTreeMap::from([("old".into(), 2)]),
        coverage: BTreeMap::from([(2, vec!["kw".into(), "text".into(), "vec".into()])]),
        ..View::default()
    }
}
fn parsed() -> ParsedValues {
    BTreeMap::from([
        (
            0,
            ParsedValue {
                hash: None,
                checksum: Some(11),
            },
        ),
        (
            1,
            ParsedValue {
                hash: None,
                checksum: Some(12),
            },
        ),
        (
            2,
            ParsedValue {
                hash: None,
                checksum: Some(13),
            },
        ),
    ])
}

#[test]
fn stale_is_dropped_before_bad_schema_and_id_is_still_interned() {
    let raw = wire(vec![item(
        "new",
        "missing",
        FieldValue::String("bad".into()),
    )]);
    let scan = FastIndexScanner::parse(&raw).unwrap();
    let mut v = view();
    v.versions.insert(10, 9);
    // `new` is allocated at watermark 10 before stale/schema work.
    let p = plan(
        &scan,
        &[ReplaceDocDescriptor {
            external_id: "new",
            version: Some(8),
            fields_start: 0,
            fields_end: 1,
        }],
        &parsed(),
        &v,
        Instant::now(),
        |_| Ok(()),
    )
    .unwrap();
    assert_eq!(p.scalar.new_external_ids, ["new"]);
    assert!(matches!(
        p.outcomes.as_slice(),
        [ReplaceItemOutcome::Dropped { current_version: 9 }]
    ));
}

#[test]
fn invalid_doc_keeps_old_coverage_and_later_sibling_runs() {
    let raw = wire(vec![
        item("old", "kw", FieldValue::Number(3.)),
        item("new", "kw", FieldValue::String("ok".into())),
    ]);
    let scan = FastIndexScanner::parse(&raw).unwrap();
    let p = plan(
        &scan,
        &[
            ReplaceDocDescriptor {
                external_id: "old",
                version: None,
                fields_start: 0,
                fields_end: 1,
            },
            ReplaceDocDescriptor {
                external_id: "new",
                version: None,
                fields_start: 1,
                fields_end: 2,
            },
        ],
        &parsed(),
        &view(),
        Instant::now(),
        |_| Ok(()),
    )
    .unwrap();
    assert!(matches!(&p.outcomes[0], ReplaceItemOutcome::Error { .. }));
    assert!(matches!(
        &p.outcomes[1],
        ReplaceItemOutcome::Ok {
            fields_written: 1,
            ..
        }
    ));
    assert!(p.final_docs.iter().all(|doc| doc.id != 2));
}

#[test]
fn duplicate_unversioned_docs_follow_source_order_and_omit_old_fields() {
    let raw = wire(vec![
        item("old", "kw", FieldValue::String("first".into())),
        item("old", "text", FieldValue::String("second".into())),
    ]);
    let scan = FastIndexScanner::parse(&raw).unwrap();
    let p = plan(
        &scan,
        &[
            ReplaceDocDescriptor {
                external_id: "old",
                version: None,
                fields_start: 0,
                fields_end: 1,
            },
            ReplaceDocDescriptor {
                external_id: "old",
                version: None,
                fields_start: 1,
                fields_end: 2,
            },
        ],
        &parsed(),
        &view(),
        Instant::now(),
        |_| Ok(()),
    )
    .unwrap();
    assert_eq!(p.final_docs[0].fields.as_slice(), ["text"]);
    assert!(p
        .scalar
        .actions
        .iter()
        .any(|a| matches!(a,ScalarAction::Drop{ordinal:usize::MAX,cell,..}if cell.field=="kw")));
}

#[test]
fn text_and_vector_need_a_prior_replace_checksum_before_equal_skip() {
    let raw = wire(vec![item("old", "text", FieldValue::String("same".into()))]);
    let scan = FastIndexScanner::parse(&raw).unwrap();
    let mut v = view(); // false: a merge write or first replace has no checksum
    let first = plan(
        &scan,
        &[ReplaceDocDescriptor {
            external_id: "old",
            version: None,
            fields_start: 0,
            fields_end: 1,
        }],
        &parsed(),
        &v,
        Instant::now(),
        |_| Ok(()),
    )
    .unwrap();
    assert!(matches!(
        &first.outcomes[0],
        ReplaceItemOutcome::Ok {
            fields_written: 1,
            ..
        }
    ));
    v.unchanged.insert((2, "text".into()));
    let second = plan(
        &scan,
        &[ReplaceDocDescriptor {
            external_id: "old",
            version: None,
            fields_start: 0,
            fields_end: 1,
        }],
        &parsed(),
        &v,
        Instant::now(),
        |_| Ok(()),
    )
    .unwrap();
    assert!(matches!(
        &second.outcomes[0],
        ReplaceItemOutcome::Ok {
            fields_skipped: 1,
            ..
        }
    ));
}

#[test]
fn reserve_refuses_before_descriptor_or_action_allocation() {
    let raw = wire(vec![item("new", "kw", FieldValue::String("v".into()))]);
    let scan = FastIndexScanner::parse(&raw).unwrap();
    assert!(plan(
        &scan,
        &[ReplaceDocDescriptor {
            external_id: "new",
            version: None,
            fields_start: 0,
            fields_end: 1
        }],
        &parsed(),
        &view(),
        Instant::now(),
        |_| Err(anyhow::anyhow!("refused"))
    )
    .is_err());
}

#[test]
fn outer_collection_failure_is_err_not_a_per_item_result() {
    let raw = wire(vec![item("new", "kw", FieldValue::String("v".into()))]);
    let scan = FastIndexScanner::parse(&raw).unwrap();
    struct Gone(View);
    impl PlanView for Gone {
        fn engine_epoch(&self) -> u64 {
            1
        }
        fn collection_generation(&self) -> u64 {
            2
        }
        fn schema_version(&self) -> u32 {
            3
        }
        fn data_version(&self) -> u64 {
            4
        }
        fn revision(&self) -> u64 {
            5
        }
        fn interner_len(&self) -> usize {
            10
        }
        fn is_live(&self) -> bool {
            false
        }
        fn field_type(&self, _: &str) -> Option<FieldType> {
            None
        }
        fn vector_dimension(&self, _: &str) -> Option<u32> {
            None
        }
        fn id(&self, _: &str) -> Option<u32> {
            None
        }
        fn has_cell(&self, _: u32, _: &str) -> bool {
            false
        }
        fn cell_version(&self, _: u32, _: &str) -> Option<u64> {
            None
        }
        fn request_deadline(&self, _: &str) -> Option<Instant> {
            None
        }
    }
    impl ReplacePlanView for Gone {
        fn doc_version(&self, _: u32) -> Option<u64> {
            None
        }
        fn old_fields(&self, _: u32) -> Option<&[String]> {
            None
        }
        fn old_fields_bound(&self, _: u32) -> OldFieldsBound {
            OldFieldsBound::default()
        }
        fn existing_unchanged(
            &self,
            _: u32,
            _: &str,
            _: FieldType,
            _: usize,
            _: Option<u64>,
        ) -> bool {
            false
        }
    }
    assert!(plan(
        &scan,
        &[ReplaceDocDescriptor {
            external_id: "new",
            version: None,
            fields_start: 0,
            fields_end: 1
        }],
        &parsed(),
        &Gone(view()),
        Instant::now(),
        |_| Ok(())
    )
    .is_err());
}

#[test]
fn replace_does_not_consult_index_cell_versions() {
    let raw = wire(vec![item("old", "kw", FieldValue::String("v".into()))]);
    let scan = FastIndexScanner::parse(&raw).unwrap();
    let p = plan(
        &scan,
        &[ReplaceDocDescriptor {
            external_id: "old",
            version: None,
            fields_start: 0,
            fields_end: 1,
        }],
        &parsed(),
        &view(),
        Instant::now(),
        |_| Ok(()),
    )
    .unwrap();
    assert!(matches!(&p.outcomes[0], ReplaceItemOutcome::Ok { .. }));
}

#[test]
fn old_coverage_reserve_refuses_before_omitted_drop_metadata_exists() {
    let raw = wire(vec![]);
    let scan = FastIndexScanner::parse(&raw).unwrap();
    let mut v = view();
    v.coverage
        .insert(2, (0..20_000).map(|n| format!("old_{n}")).collect());
    assert!(plan(
        &scan,
        &[ReplaceDocDescriptor {
            external_id: "old",
            version: None,
            fields_start: 0,
            fields_end: 0
        }],
        &parsed(),
        &v,
        Instant::now(),
        |bytes| {
            assert!(bytes > 20_000 * 256);
            Err(anyhow::anyhow!("refused"))
        }
    )
    .is_err());
}

#[test]
fn versioned_then_unversioned_keeps_effective_version_for_later_stale_item() {
    let raw = wire(vec![
        item("old", "kw", FieldValue::String("a".into())),
        item("old", "text", FieldValue::String("b".into())),
        item("old", "kw", FieldValue::String("c".into())),
    ]);
    let scan = FastIndexScanner::parse(&raw).unwrap();
    let mut v = view();
    v.versions.insert(2, 10);
    let p = plan(
        &scan,
        &[
            ReplaceDocDescriptor {
                external_id: "old",
                version: Some(30),
                fields_start: 0,
                fields_end: 1,
            },
            ReplaceDocDescriptor {
                external_id: "old",
                version: None,
                fields_start: 1,
                fields_end: 2,
            },
            ReplaceDocDescriptor {
                external_id: "old",
                version: Some(20),
                fields_start: 2,
                fields_end: 3,
            },
        ],
        &parsed(),
        &v,
        Instant::now(),
        |_| Ok(()),
    )
    .unwrap();
    assert_eq!(p.final_docs[0].version, Some(30));
    assert!(matches!(
        &p.outcomes[2],
        ReplaceItemOutcome::Dropped {
            current_version: 30
        }
    ));
}

#[test]
fn malformed_descriptor_proof_fails_before_stale_shortcut() {
    let raw = wire(vec![item("old", "kw", FieldValue::String("x".into()))]);
    let scan = FastIndexScanner::parse(&raw).unwrap();
    let mut v = view();
    v.versions.insert(2, 9);
    assert!(plan(
        &scan,
        &[ReplaceDocDescriptor {
            external_id: "old",
            version: Some(8),
            fields_start: 1,
            fields_end: 1
        }],
        &parsed(),
        &v,
        Instant::now(),
        |_| Ok(())
    )
    .is_err());
}

#[test]
fn invalid_hash_keeps_source_ordinal_for_exact_late_rendering() {
    let raw = wire(vec![item(
        "new",
        "hash",
        FieldValue::String("not-a-hash".into()),
    )]);
    let scan = FastIndexScanner::parse(&raw).unwrap();
    let mut v = view();
    v.fields.insert("hash".into(), FieldType::Hash);
    let p = plan(
        &scan,
        &[ReplaceDocDescriptor {
            external_id: "new",
            version: None,
            fields_start: 0,
            fields_end: 1,
        }],
        &parsed(),
        &v,
        Instant::now(),
        |_| Ok(()),
    )
    .unwrap();
    assert!(matches!(
        &p.outcomes[0],
        ReplaceItemOutcome::Error {
            error: ReplaceItemError::InvalidHash { ordinal: 0 }
        }
    ));
}

#[test]
fn thirty_two_docs_can_plan_more_than_one_thousand_flattened_fields() {
    let mut items = Vec::new();
    let mut docs = Vec::new();
    let mut v = View::default();
    for d in 0..32 {
        let start = items.len();
        let id = format!("doc-{d:02}");
        for f in 0..33 {
            let field = format!("f-{f:02}");
            v.fields.insert(field.clone(), FieldType::Keyword);
            items.push(item(&id, &field, FieldValue::String("v".into())));
        }
        docs.push(ReplaceDocDescriptor {
            external_id: Box::leak(id.into_boxed_str()),
            version: None,
            fields_start: start,
            fields_end: items.len(),
        });
    }
    let raw = wire(items);
    let scan = FastIndexScanner::parse(&raw).unwrap();
    let p = plan(
        &scan,
        &docs,
        &ParsedValues::new(),
        &v,
        Instant::now(),
        |_| Ok(()),
    )
    .unwrap();
    assert_eq!(p.outcomes.len(), 32);
    assert!(p.scalar.source_items > 1000);
}
