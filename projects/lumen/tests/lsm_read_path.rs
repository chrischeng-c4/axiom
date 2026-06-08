#![cfg(feature = "experimental")] // unwired subsystem; run with --features experimental
//! End-to-end test that the lumen `Engine` actually reads through the
//! LSM backend when one is attached — not from the in-memory mirror
//! that hangs off [`crate::storage::KeywordIndex`] / `NumberIndex`.
//!
//! The goals matrix:
//!
//! 1. Index O(1000) keyword + number postings, hit the backend on
//!    `term` / `range` / `duplicates` queries, assert correct totals.
//! 2. Drop the `Engine` (its in-memory mirror is discarded with it)
//!    and reopen on the same on-disk LSM root. Re-run the queries —
//!    same answers, no seed pass required. This is the cold-recovery
//!    assertion the spec calls out.
//! 3. Sanity-check that attaching the backend rejects `text`, `set`
//!    and `vector` indexing with the documented error so callers
//!    can't silently lose data on the unsupported field types.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;

use lumen::storage::Engine;
use lumen::storage_backend::Backend;
use lumen::storage_lsm::{FsyncPolicy, Lsm, LsmConfig};
use lumen::types::{
    Analyzer, CreateCollectionRequest, DuplicatesRequest, FieldSpec, FieldType, FieldValue,
    IndexItem, IndexRequest, QueryNode, RangeQuery, SearchRequest, TermQuery,
};
use tempfile::TempDir;

fn lsm_cfg(root: PathBuf) -> LsmConfig {
    LsmConfig {
        root,
        // Force a flush partway through the write workload so the read
        // path exercises both memtable + SST resolution.
        memtable_bytes: 8 * 1024,
        cache_bytes: 4 * 1024 * 1024,
        fsync: FsyncPolicy::PerWrite,
        block_bytes: 8 * 1024,
    }
}

fn users_schema() -> CreateCollectionRequest {
    let mut fields = BTreeMap::new();
    fields.insert(
        "email".to_string(),
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
    fields.insert(
        "age".to_string(),
        FieldSpec {
            field_type: FieldType::Number,
            analyzer: None,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        },
    );
    CreateCollectionRequest { fields }
}

fn item(eid: &str, field: &str, value: FieldValue) -> IndexItem {
    IndexItem {
        external_id: eid.to_string(),
        field: field.to_string(),
        value,
    }
}

/// 1 000 keyword writes spanning ~200 distinct values, 1 000 number
/// writes across a known range. We compute the expected groups /
/// counts in the same loop so the assertions don't need to keep
/// independent oracle tables.
struct Workload {
    items: Vec<IndexItem>,
    /// "shared@x.com" group → list of eids that wrote it.
    keyword_groups: BTreeMap<String, BTreeSet<String>>,
    /// numeric value → list of eids that wrote it.
    number_groups: BTreeMap<i64, BTreeSet<String>>,
}

fn build_workload() -> Workload {
    let mut items = Vec::with_capacity(2_000);
    let mut keyword_groups: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut number_groups: BTreeMap<i64, BTreeSet<String>> = BTreeMap::new();
    for i in 0..1_000 {
        let eid = format!("u{i:04}");
        // ~200 distinct emails — every email has on average 5 eids.
        let email = format!("user-{}@x.com", i % 200);
        items.push(item(&eid, "email", FieldValue::String(email.clone())));
        keyword_groups.entry(email).or_default().insert(eid.clone());
        // Numbers in [0, 100) — every value has 10 eids on average.
        let n = (i % 100) as i64;
        items.push(item(&eid, "age", FieldValue::Number(n as f64)));
        number_groups.entry(n).or_default().insert(eid);
    }
    Workload {
        items,
        keyword_groups,
        number_groups,
    }
}

fn open_engine(root: &std::path::Path) -> Engine {
    let lsm = Lsm::open(lsm_cfg(root.to_path_buf())).unwrap();
    let backend: Arc<dyn Backend> = Arc::new(lsm);
    Engine::new_with_backend(backend).expect("engine bootstrap")
}

fn term_count(e: &Engine, coll: &str, field: &str, value: FieldValue) -> u64 {
    let resp = e
        .search(
            coll,
            SearchRequest {
                query: QueryNode::Term(TermQuery {
                    field: field.to_string(),
                    value,
                }),
                limit: 10_000,
                cursor: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .unwrap();
    resp.total
}

fn range_count(e: &Engine, coll: &str, field: &str, gte: f64, lt: f64) -> u64 {
    let resp = e
        .search(
            coll,
            SearchRequest {
                query: QueryNode::Range(RangeQuery {
                    field: field.to_string(),
                    gte: Some(gte),
                    lt: Some(lt),
                    gt: None,
                    lte: None,
                }),
                limit: 10_000,
                cursor: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .unwrap();
    resp.total
}

fn run_query_battery(engine: &Engine, workload: &Workload) {
    // -- term query (keyword) --------------------------------------
    for (email, eids) in workload.keyword_groups.iter().take(20) {
        let got = term_count(engine, "users", "email", FieldValue::String(email.clone()));
        assert_eq!(
            got,
            eids.len() as u64,
            "term query on email {email} mismatch"
        );
    }
    // An email that was never written.
    let missing = term_count(
        engine,
        "users",
        "email",
        FieldValue::String("missing@x.com".to_string()),
    );
    assert_eq!(missing, 0, "term query on missing email should yield 0");

    // -- term query (number) ---------------------------------------
    for (n, eids) in workload.number_groups.iter().take(20) {
        let got = term_count(engine, "users", "age", FieldValue::Number(*n as f64));
        assert_eq!(got, eids.len() as u64, "term query on age={n} mismatch");
    }

    // -- range query -----------------------------------------------
    // Spec range [20, 50): every numeric value in that range
    // contributes its full group.
    let expected_range: u64 = workload
        .number_groups
        .range(20..50)
        .map(|(_, eids)| eids.len() as u64)
        .sum();
    assert_eq!(
        range_count(engine, "users", "age", 20.0, 50.0),
        expected_range,
        "range scan on age [20, 50) mismatch"
    );
    // Empty range — well above the data.
    assert_eq!(range_count(engine, "users", "age", 200.0, 500.0), 0);

    // -- duplicates query (keyword) --------------------------------
    let dup = engine
        .duplicates(
            "users",
            DuplicatesRequest {
                field: "email".to_string(),
                min_group_size: 2,
                limit: 1_000,
                offset: 0,
            },
        )
        .unwrap();
    let dup_emails: BTreeSet<&str> = dup
        .groups
        .iter()
        .map(|g| g.value.as_str().expect("string keyword"))
        .collect();
    let expected_emails: BTreeSet<&str> = workload
        .keyword_groups
        .iter()
        .filter(|(_, eids)| eids.len() >= 2)
        .map(|(email, _)| email.as_str())
        .collect();
    assert_eq!(
        dup_emails, expected_emails,
        "duplicates: keyword groups mismatch"
    );
    // Spot-check one group is materialised correctly (member list).
    let first = dup.groups.first().expect("at least one keyword duplicate");
    let email = first.value.as_str().unwrap();
    let expected_eids = workload.keyword_groups.get(email).unwrap();
    let got_eids: BTreeSet<&str> = first.external_ids.iter().map(String::as_str).collect();
    let expected_str: BTreeSet<&str> = expected_eids.iter().map(String::as_str).collect();
    assert_eq!(got_eids, expected_str, "duplicate group members mismatch");

    // -- duplicates query (number) ---------------------------------
    let dup_n = engine
        .duplicates(
            "users",
            DuplicatesRequest {
                field: "age".to_string(),
                min_group_size: 2,
                limit: 1_000,
                offset: 0,
            },
        )
        .unwrap();
    let dup_numbers: BTreeSet<i64> = dup_n
        .groups
        .iter()
        .map(|g| g.value.as_f64().expect("number") as i64)
        .collect();
    let expected_numbers: BTreeSet<i64> = workload
        .number_groups
        .iter()
        .filter(|(_, eids)| eids.len() >= 2)
        .map(|(n, _)| *n)
        .collect();
    assert_eq!(
        dup_numbers, expected_numbers,
        "duplicates: number groups mismatch"
    );
}

#[test]
fn lsm_attached_engine_reads_through_backend_and_recovers() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().to_path_buf();
    let workload = build_workload();

    // --- Phase 1: cold boot, write workload, query through backend ---
    {
        let engine = open_engine(&root);
        engine.create_collection("users", users_schema()).unwrap();
        engine
            .index(
                "users",
                IndexRequest {
                    items: workload.items.clone(),
                    request_id: None,
                },
            )
            .unwrap();
        run_query_battery(&engine, &workload);
    }

    // --- Phase 2: cold restart, no seed indexing, queries must still
    //              return the same answers from disk. ---
    let engine = open_engine(&root);
    // The engine MUST already know about the collection from the
    // persisted schema entry — we explicitly do NOT call
    // `create_collection` again.
    assert!(
        engine
            .list_collections()
            .unwrap()
            .contains(&"users".to_string()),
        "collection schema should have been recovered from disk"
    );
    run_query_battery(&engine, &workload);
}

#[test]
fn backend_attached_engine_rejects_unsupported_field_types() {
    let dir = TempDir::new().unwrap();
    let lsm = Lsm::open(lsm_cfg(dir.path().to_path_buf())).unwrap();
    let backend: Arc<dyn Backend> = Arc::new(lsm);
    let engine = Engine::new_with_backend(backend).unwrap();

    // Schema with one of every backend-unsupported type.
    let mut fields = BTreeMap::new();
    fields.insert(
        "bio".to_string(),
        FieldSpec {
            field_type: FieldType::Text,
            analyzer: Some(Analyzer::WhitespaceLower),
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        },
    );
    fields.insert(
        "tags".to_string(),
        FieldSpec {
            field_type: FieldType::Set,
            analyzer: None,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        },
    );
    engine
        .create_collection("mixed", CreateCollectionRequest { fields })
        .unwrap();

    for (field, value) in [
        ("bio", FieldValue::String("hello world".to_string())),
        (
            "tags",
            FieldValue::StringList(vec!["a".to_string(), "b".to_string()]),
        ),
    ] {
        let err = engine
            .index(
                "mixed",
                IndexRequest {
                    items: vec![item("u1", field, value)],
                    request_id: None,
                },
            )
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("v1 LSM only covers keyword + number"),
            "expected BackendNotSupported error on field `{field}`, got: {err}"
        );
    }
}
