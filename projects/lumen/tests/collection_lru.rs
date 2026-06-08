//! collection-LRU: RAM is a working set, idle collections are snapshotted to
//! disk and dropped, restored on demand. Verifies evict→restore is lossless,
//! the budget is enforced with LRU order, and restore is thread-safe.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::thread;

use lumen::storage::Engine;
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, HasChildQuery, IndexItem,
    IndexRequest, QueryNode, SearchRequest, TermQuery,
};

fn kw_schema() -> CreateCollectionRequest {
    let mut f = BTreeMap::new();
    f.insert(
        "kw".into(),
        FieldSpec {
            field_type: FieldType::Keyword,
            analyzer: None,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        },
    );
    CreateCollectionRequest { fields: f }
}

fn tmpdir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("lumen_lru_{}_{}", std::process::id(), tag));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn index_docs(e: &Engine, coll: &str, n: usize) {
    for i in 0..n {
        e.index(
            coll,
            IndexRequest {
                items: vec![IndexItem {
                    external_id: format!("d{i}"),
                    field: "kw".into(),
                    value: FieldValue::String("shared".into()),
                }],
                request_id: None,
            },
        )
        .unwrap();
    }
}

fn search_count(e: &Engine, coll: &str) -> usize {
    e.search(
        coll,
        SearchRequest {
            query: QueryNode::Term(TermQuery {
                field: "kw".into(),
                value: FieldValue::String("shared".into()),
            }),
            limit: 1000,
            cursor: None,
            sort: None,
            track_total: true,
            collapse: None,
        },
    )
    .unwrap()
    .hits
    .len()
}

/// Evict a collection (snapshot→disk, dropped from RAM) then a search restores
/// it with an IDENTICAL result.
#[test]
fn evict_restore_round_trip_identical() {
    let e = Arc::new(Engine::new_lru(1_500, tmpdir("rt")));
    e.create_collection("a", kw_schema()).unwrap();
    index_docs(&e, "a", 20);
    let before = search_count(&e, "a");
    assert_eq!(before, 20);

    // Fill other collections → "a" (now LRU) is evicted to disk.
    for c in ["b", "c", "d", "e"] {
        e.create_collection(c, kw_schema()).unwrap();
        index_docs(&e, c, 20);
    }
    assert!(
        e.evicted_names().contains(&"a".to_string()),
        "a should be evicted: resident={:?} evicted={:?}",
        e.resident_names(),
        e.evicted_names()
    );

    // Search restores "a" with the same result.
    assert_eq!(search_count(&e, "a"), before, "restored result must match");
    assert!(
        e.resident_names().contains(&"a".to_string()),
        "a resident after restore"
    );
}

/// Budget is enforced (not all collections resident), everything stays queryable
/// (evicted ones restore), and eviction is LRU (the most-recently-accessed
/// collection stays resident).
#[test]
fn budget_enforced_and_lru() {
    let e = Arc::new(Engine::new_lru(6_000, tmpdir("budget")));
    for i in 0..8 {
        let c = format!("c{i}");
        e.create_collection(&c, kw_schema()).unwrap();
        index_docs(&e, &c, 20);
    }
    // eviction happened, and every collection is accounted for.
    let resident = e.resident_names().len();
    assert!(
        resident < 8,
        "some collections must be evicted, resident={resident}"
    );
    assert!(resident >= 1, "always keep at least one resident");
    assert_eq!(resident + e.evicted_names().len(), 8, "all 8 accounted for");

    // Every collection still returns the correct count (restores evicted ones).
    for i in 0..8 {
        assert_eq!(search_count(&e, &format!("c{i}")), 20, "c{i} round-trip");
    }
    // We just searched c0..c7 in order → c7 is most-recently-accessed → resident.
    assert!(
        e.resident_names().contains(&"c7".to_string()),
        "MRU c7 must be resident: {:?}",
        e.resident_names()
    );
}

/// Many threads searching the same evicted collection concurrently must all
/// restore-safely and get the correct result (no corruption / deadlock; the
/// read→write re-lock guards against a double-restore race).
#[test]
fn concurrent_restore_thread_safe() {
    let e = Arc::new(Engine::new_lru(1_500, tmpdir("conc")));
    e.create_collection("hot", kw_schema()).unwrap();
    index_docs(&e, "hot", 20);
    for c in ["x", "y", "z"] {
        e.create_collection(c, kw_schema()).unwrap();
        index_docs(&e, c, 20);
    }
    assert!(
        !e.resident_names().contains(&"hot".to_string()),
        "hot should be evicted first"
    );

    let mut handles = vec![];
    for _ in 0..16 {
        let e2 = e.clone();
        handles.push(thread::spawn(move || search_count(&e2, "hot")));
    }
    for h in handles {
        assert_eq!(h.join().unwrap(), 20, "concurrent restore must be correct");
    }
}

/// A keyword schema with two named fields (for a child collection:
/// `parent_row_id` + a sub-field).
fn two_kw_schema(a: &str, b: &str) -> CreateCollectionRequest {
    let mut f = BTreeMap::new();
    for name in [a, b] {
        f.insert(
            name.to_string(),
            FieldSpec {
                field_type: FieldType::Keyword,
                analyzer: None,
                multi: None,
                dim: None,
                metric: None,
                backend: None,
                quantize: None,
            },
        );
    }
    CreateCollectionRequest { fields: f }
}

/// A `has_child` query whose child collection has been EVICTED must restore it
/// on demand and return the SAME parents as when the child was resident — the
/// nested-group path is correct under collection-LRU, not just the flat path.
#[test]
fn has_child_restores_evicted_child_collection() {
    let e = Arc::new(Engine::new_lru(1_500, tmpdir("haschild")));
    e.create_collection("parent", kw_schema()).unwrap();
    e.create_collection("child", two_kw_schema("parent", "sku"))
        .unwrap();

    // parent p0, p1; child c0->p0(sku S0), c1->p1(sku S1).
    for p in ["p0", "p1"] {
        e.index(
            "parent",
            IndexRequest {
                items: vec![IndexItem {
                    external_id: p.into(),
                    field: "kw".into(),
                    value: FieldValue::String("shared".into()),
                }],
                request_id: None,
            },
        )
        .unwrap();
    }
    for (cid, par, sku) in [("c0", "p0", "S0"), ("c1", "p1", "S1")] {
        e.index(
            "child",
            IndexRequest {
                items: vec![
                    IndexItem {
                        external_id: cid.into(),
                        field: "parent".into(),
                        value: FieldValue::String(par.into()),
                    },
                    IndexItem {
                        external_id: cid.into(),
                        field: "sku".into(),
                        value: FieldValue::String(sku.into()),
                    },
                ],
                request_id: None,
            },
        )
        .unwrap();
    }

    // has_child: parents with a child element where sku == "S0" → {p0}.
    let run = |e: &Engine| -> Vec<String> {
        let mut v: Vec<String> = e
            .search(
                "parent",
                SearchRequest {
                    query: QueryNode::HasChild(HasChildQuery {
                        collection: "child".into(),
                        field: "parent".into(),
                        query: Box::new(QueryNode::Term(TermQuery {
                            field: "sku".into(),
                            value: FieldValue::String("S0".into()),
                        })),
                    }),
                    limit: 100,
                    cursor: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap()
            .hits
            .into_iter()
            .map(|h| h.external_id)
            .collect();
        v.sort();
        v
    };

    let resident = run(&e);
    assert_eq!(
        resident,
        vec!["p0".to_string()],
        "resident has_child baseline"
    );

    // Force the child collection out to disk by exercising other collections.
    for c in ["f0", "f1", "f2", "f3", "f4"] {
        e.create_collection(c, kw_schema()).unwrap();
        index_docs(&e, c, 20);
    }
    assert!(
        e.evicted_names().contains(&"child".to_string()),
        "child must be evicted: resident={:?} evicted={:?}",
        e.resident_names(),
        e.evicted_names()
    );

    // The same query now restores the evicted child and returns identical parents.
    assert_eq!(
        run(&e),
        resident,
        "has_child over an evicted child must match the resident result"
    );
    assert!(
        e.resident_names().contains(&"child".to_string()),
        "child resident after has_child restore"
    );
}
