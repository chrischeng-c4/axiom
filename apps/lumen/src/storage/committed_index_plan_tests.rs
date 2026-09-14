//! Unit tests to place in `committed_index_plan.rs` after integration.

use super::*;
use crate::log_entry::RaftLogEntry;
use crate::types::{FieldValue, IndexItem, IndexRequest};
use crate::wal::WalRecord;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct View {
    fields: BTreeMap<String, FieldType>,
    vector_dimensions: BTreeMap<String, u32>,
    ids: BTreeMap<String, u32>,
    cells: BTreeSet<(u32, String)>,
    versions: BTreeMap<(u32, String), u64>,
    duplicate: Option<(String, Instant)>,
}
impl PlanView for View {
    fn engine_epoch(&self) -> u64 {
        3
    }
    fn collection_generation(&self) -> u64 {
        4
    }
    fn schema_version(&self) -> u32 {
        5
    }
    fn data_version(&self) -> u64 {
        6
    }
    fn revision(&self) -> u64 {
        7
    }
    fn interner_len(&self) -> usize {
        11
    }
    fn is_live(&self) -> bool {
        true
    }
    fn field_type(&self, f: &str) -> Option<FieldType> {
        self.fields.get(f).copied()
    }
    fn vector_dimension(&self, f: &str) -> Option<u32> {
        self.vector_dimensions.get(f).copied()
    }
    fn id(&self, e: &str) -> Option<u32> {
        self.ids.get(e).copied()
    }
    fn has_cell(&self, id: u32, f: &str) -> bool {
        self.cells.contains(&(id, f.to_owned()))
    }
    fn cell_version(&self, id: u32, f: &str) -> Option<u64> {
        self.versions.get(&(id, f.to_owned())).copied()
    }
    fn request_deadline(&self, r: &str) -> Option<Instant> {
        self.duplicate
            .as_ref()
            .filter(|(id, _)| id == r)
            .map(|(_, d)| *d)
    }
}
fn now() -> Instant {
    Instant::now()
}
fn scalar_view() -> View {
    View {
        fields: BTreeMap::from([
            ("kw".into(), FieldType::Keyword),
            ("n".into(), FieldType::Number),
            ("tags".into(), FieldType::Set),
        ]),
        ..View::default()
    }
}
fn item(id: &str, field: &str, value: FieldValue, version: Option<u64>) -> IndexItem {
    IndexItem {
        external_id: id.into(),
        field: field.into(),
        value,
        version,
    }
}
fn encode(items: Vec<IndexItem>, request: Option<&str>) -> Vec<u8> {
    WalRecord::new(RaftLogEntry::Index {
        collection_id: "docs".into(),
        req: IndexRequest {
            request_id: request.map(str::to_owned),
            items,
        },
    })
    .encode()
    .unwrap()
}
fn planned(bytes: &[u8], view: &View) -> ScalarPlan {
    let s = FastIndexScanner::parse(bytes).unwrap();
    let PlanResult::Planned(p) = plan(&s, view, now(), |_| Ok(())).unwrap() else {
        panic!("fixture must be scalar")
    };
    p
}

#[test]
fn repeated_cell_lww_version_none_then_older_or_equal_keeps_one_winner() {
    let mut view = scalar_view();
    view.ids.insert("old".into(), 2);
    view.versions.insert((2, "kw".into()), 7);
    let p = planned(
        &encode(
            vec![
                item("old", "kw", FieldValue::String("arrival".into()), None),
                item("old", "kw", FieldValue::String("older".into()), Some(6)),
                item("old", "kw", FieldValue::String("equal".into()), Some(7)),
            ],
            None,
        ),
        &view,
    );
    assert_eq!(p.applied, 1);
    assert_eq!(p.winners.values().copied().collect::<Vec<_>>(), vec![0]);
    assert_eq!(
        p.bytes_by_field["kw"],
        ("arrival".len() + "old".len()) as u64
    );
}
#[test]
fn request_dedup_has_no_actions() {
    let mut view = scalar_view();
    view.duplicate = Some(("once".into(), now() + std::time::Duration::from_secs(60)));
    let p = planned(
        &encode(
            vec![item("x", "kw", FieldValue::String("v".into()), None)],
            Some("once"),
        ),
        &view,
    );
    assert!(matches!(p.request, RequestOutcome::Duplicate { .. }));
    assert_eq!(p.applied, 0);
    assert!(p.actions.is_empty() && p.winners.is_empty());
}
#[test]
fn wrong_type_replaces_old_cell_with_deletion_after_valid_prefix() {
    let mut view = scalar_view();
    view.ids.insert("old".into(), 2);
    view.cells.insert((2, "kw".into()));
    let p = planned(
        &encode(
            vec![
                item("old", "kw", FieldValue::String("valid".into()), None),
                item("old", "kw", FieldValue::Number(2.), None),
            ],
            None,
        ),
        &view,
    );
    assert_eq!(p.applied, 1);
    assert!(p.winners.is_empty());
    assert!(matches!(
        p.actions.last(),
        Some(ScalarAction::Drop { ordinal: 1, .. })
    ));
    assert!(matches!(
        p.business_error,
        Some(BusinessError::TypeMismatch { .. })
    ));
}
#[test]
fn unknown_field_keeps_stable_id_and_valid_prefix() {
    let p = planned(
        &encode(
            vec![
                item("sparse-new", "kw", FieldValue::String("ok".into()), None),
                item(
                    "sparse-new",
                    "missing",
                    FieldValue::String("bad".into()),
                    None,
                ),
            ],
            None,
        ),
        &scalar_view(),
    );
    assert_eq!(p.new_external_ids, ["sparse-new"]);
    assert_eq!(p.winners.values().copied().collect::<Vec<_>>(), vec![0]);
    assert!(matches!(
        p.business_error,
        Some(BusinessError::UnknownField { .. })
    ));
}
#[test]
fn value_is_borrowed_and_plan_has_only_ordinal() {
    let bytes = encode(
        vec![item("id", "kw", FieldValue::String("value".into()), None)],
        None,
    );
    let s = FastIndexScanner::parse(&bytes).unwrap();
    let FastIndexValue::String(value) = s.items().next().unwrap().value else {
        panic!()
    };
    let base = bytes.as_ptr() as usize;
    assert!(
        base <= value.as_ptr() as usize
            && value.as_ptr() as usize + value.len() <= base + bytes.len()
    );
    let PlanResult::Planned(p) = plan(&s, &scalar_view(), now(), |_| Ok(())).unwrap() else {
        panic!()
    };
    assert_eq!(p.winners.values().copied().collect::<Vec<_>>(), vec![0]);
}
#[test]
fn over_256_mib_borrowed_value_reserves_only_planner_metadata() {
    const VALUE_BYTES: usize = 264 * 1024 * 1024;
    let bytes = encode(
        vec![item(
            "id",
            "kw",
            FieldValue::String("x".repeat(VALUE_BYTES)),
            None,
        )],
        None,
    );
    assert!(bytes.len() > 256 * 1024 * 1024);
    let s = FastIndexScanner::parse(&bytes).unwrap();
    let FastIndexValue::String(value) = s.items().next().unwrap().value else {
        panic!()
    };
    let base = bytes.as_ptr() as usize;
    assert!(
        base <= value.as_ptr() as usize
            && value.as_ptr() as usize + value.len() <= base + bytes.len()
    );
    let PlanResult::Planned(p) = plan(&s, &scalar_view(), now(), |reserved| {
        assert!(reserved <= 256 * 1024 * 1024);
        Ok(())
    })
    .unwrap() else {
        panic!()
    };
    assert_eq!(p.winners.values().copied().collect::<Vec<_>>(), vec![0]);
}
#[test]
fn nan_number_keeps_current_invalid_number_error_text() {
    let p = planned(
        &encode(
            vec![item("id", "n", FieldValue::Number(f64::NAN), None)],
            None,
        ),
        &scalar_view(),
    );
    assert!(
        matches!(p.business_error,Some(BusinessError::InvalidNumber(ref message)) if message=="NaN is not a valid number value")
    );
    assert!(p
        .actions
        .iter()
        .any(|action| matches!(action, ScalarAction::Drop { ordinal: 0, .. })));
}
#[test]
fn supports_full_ten_thousand_item_limit_with_sparse_and_existing_ids() {
    let mut view = scalar_view();
    view.ids.insert("old".into(), 9);
    let items = (0..MAX_INDEX_ITEMS)
        .map(|n| {
            let id = if n % 2 == 0 {
                "old".to_owned()
            } else {
                format!("new-{n}")
            };
            item(&id, "kw", FieldValue::String(n.to_string()), None)
        })
        .collect();
    let p = planned(&encode(items, None), &view);
    assert_eq!(p.source_items, MAX_INDEX_ITEMS);
    assert_eq!(p.applied as usize, MAX_INDEX_ITEMS);
    assert_eq!(p.new_external_ids.len(), MAX_INDEX_ITEMS / 2);
    assert_eq!(p.winners.len(), MAX_INDEX_ITEMS / 2 + 1);
}

#[test]
fn text_is_planned_in_the_unified_ledger_and_keeps_text_drop_kind() {
    let mut view = scalar_view();
    view.fields.insert("body".into(), FieldType::Text);
    view.ids.insert("id".into(), 2);
    view.cells.insert((2, "body".into()));
    let plan = planned(
        &encode(
            vec![
                item(
                    "id",
                    "body",
                    FieldValue::String("valid prefix".into()),
                    Some(1),
                ),
                item("id", "body", FieldValue::Number(3.0), Some(2)),
            ],
            None,
        ),
        &view,
    );
    assert!(plan.has_text());
    assert_eq!(
        plan.text_actions()
            .map(|(_, ordinal)| ordinal)
            .collect::<Vec<_>>(),
        vec![0]
    );
    assert!(matches!(
        plan.actions.last(),
        Some(ScalarAction::Drop {
            kind: FieldType::Text,
            ordinal: 1,
            ..
        })
    ));
    assert!(plan.business_error.is_some());
}

#[test]
fn vector_is_planned_without_decoding_its_borrowed_values() {
    let mut view = scalar_view();
    view.fields.insert("vector".into(), FieldType::Vector);
    view.vector_dimensions.insert("vector".into(), 3);
    let plan = planned(
        &encode(
            vec![item(
                "id",
                "vector",
                FieldValue::Vector(vec![1.0, -2.0, 3.0]),
                None,
            )],
            None,
        ),
        &view,
    );
    assert_eq!(plan.applied, 1);
    assert_eq!(plan.winners.values().copied().collect::<Vec<_>>(), vec![0]);
    assert!(matches!(
        plan.actions.as_slice(),
        [ScalarAction::Apply {
            kind: FieldType::Vector,
            ordinal: 0,
            bytes,
            ..
        }] if *bytes == 3 * std::mem::size_of::<f32>() as u64 + 2
    ));
}

#[test]
fn vector_dimension_error_keeps_valid_prefix_then_drops_the_bad_cell() {
    let mut view = scalar_view();
    view.fields.insert("vector".into(), FieldType::Vector);
    view.vector_dimensions.insert("vector".into(), 3);
    view.ids.insert("old".into(), 2);
    view.cells.insert((2, "vector".into()));
    let plan = planned(
        &encode(
            vec![
                item(
                    "old",
                    "vector",
                    FieldValue::Vector(vec![1.0, 2.0, 3.0]),
                    Some(1),
                ),
                item("old", "vector", FieldValue::Vector(vec![4.0]), Some(2)),
            ],
            None,
        ),
        &view,
    );
    assert_eq!(plan.applied, 1);
    assert!(matches!(
        plan.actions.last(),
        Some(ScalarAction::Drop {
            kind: FieldType::Vector,
            ordinal: 1,
            ..
        })
    ));
    let error = plan
        .business_error
        .expect("wrong vector dimension is a business error")
        .into_error(
            "docs",
            &FastIndexScanner::parse(&encode(
                vec![item(
                    "old",
                    "vector",
                    FieldValue::Vector(vec![4.0]),
                    Some(2),
                )],
                None,
            ))
            .unwrap(),
        );
    assert_eq!(
        error.to_string(),
        "vector field `vector` declared dim=3 but got vector of length 1"
    );
}

#[test]
fn stale_vector_dimension_is_skipped_before_validation() {
    let mut view = scalar_view();
    view.fields.insert("vector".into(), FieldType::Vector);
    view.vector_dimensions.insert("vector".into(), 3);
    view.ids.insert("old".into(), 2);
    view.versions.insert((2, "vector".into()), 5);
    let plan = planned(
        &encode(
            vec![item(
                "old",
                "vector",
                FieldValue::Vector(vec![1.0]),
                Some(5),
            )],
            None,
        ),
        &view,
    );
    assert_eq!(plan.applied, 0);
    assert!(plan.actions.is_empty());
    assert!(plan.business_error.is_none());
}
