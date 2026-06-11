// <HANDWRITE gap="standardize:claim-code" tracker="projects-lumen-tests-properties-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
//! Property-based tests — Contract 3 of the coverage goal.
//!
//! Each invariant is fuzzed with proptest over randomized
//! (external_id, field, value) inputs. Example-based tests can pass by
//! luck of the chosen example; these assert the invariant holds across
//! the whole input space (≥ 256 cases each, proptest default, with
//! shrinking on failure).
//!
//! Invariants (mirrors the goal doc):
//!   P1 index(eid,field,v)  ⇒ term/match(v) contains eid
//!   P2 delete(eid)         ⇒ no query returns eid
//!   P3 snapshot→restore    ⇒ identical query results
//!   P4 re-index(eid,f,v2)  ⇒ v1 gone, v2 present
//!   P5 index N distinct    ⇒ stats.documents_indexed == N
//!   P6 duplicates groups   ⇒ every eid in a group really shares the value

use std::collections::BTreeMap;
use std::sync::Arc;

use proptest::prelude::*;

use lumen::storage::Engine;
use lumen::types::{
    Analyzer, CreateCollectionRequest, DuplicatesRequest, FieldSpec, FieldType, FieldValue,
    IndexItem, IndexRequest, MatchOp, MatchQuery, QueryNode, SearchRequest, TermQuery,
};

// ---------------------------------------------------------------------------
// Generators
// ---------------------------------------------------------------------------

/// External ids drawn from a small alphabet so collisions (re-index of
/// the same eid) happen often — that exercises the replace path.
fn eid_strategy() -> impl Strategy<Value = String> {
    proptest::collection::vec(prop::sample::select(vec!['a', 'b', 'c', 'd', 'e']), 1..4)
        .prop_map(|cs| cs.into_iter().collect())
}

/// Keyword values from a small alphabet so duplicate groups form.
fn keyword_value() -> impl Strategy<Value = String> {
    prop::sample::select(vec!["red", "green", "blue", "amber", "violet"])
        .prop_map(|s| s.to_string())
}

/// Free text drawn from a handful of tokens.
fn text_value() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        prop::sample::select(vec!["alpha", "beta", "gamma", "delta", "epsilon"]),
        1..6,
    )
    .prop_map(|ts| ts.join(" "))
}

fn schema() -> CreateCollectionRequest {
    let mut fields = BTreeMap::new();
    fields.insert(
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
    fields.insert(
        "body".into(),
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
    CreateCollectionRequest { fields }
}

fn fresh() -> Arc<Engine> {
    let e = Arc::new(Engine::new());
    e.create_collection("c", schema()).unwrap();
    e
}

fn term_search(e: &Engine, field: &str, value: &str) -> Vec<String> {
    e.search(
        "c",
        SearchRequest {
            query: QueryNode::Term(TermQuery {
                field: field.into(),
                value: FieldValue::String(value.into()),
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
    .into_iter()
    .map(|h| h.external_id)
    .collect()
}

fn match_search(e: &Engine, field: &str, text: &str) -> Vec<String> {
    e.search(
        "c",
        SearchRequest {
            query: QueryNode::Match(MatchQuery {
                field: field.into(),
                text: text.into(),
                op: MatchOp::Or,
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
    .into_iter()
    .map(|h| h.external_id)
    .collect()
}

proptest! {
    // Goal Contract 3 requires ≥ 1000 random cases per invariant (the
    // proptest default is 256). 1024 keeps each invariant well above
    // that floor while staying fast (the engine ops are in-memory).
    #![proptest_config(ProptestConfig::with_cases(1024))]

    // P1 — after indexing (eid, kw, v), an exact term query for v must
    // return eid. The "last write wins" model means we apply writes in
    // order and check the final value for each eid.
    #[test]
    fn p1_indexed_keyword_is_findable(
        writes in proptest::collection::vec((eid_strategy(), keyword_value()), 1..40)
    ) {
        let e = fresh();
        let mut final_value: BTreeMap<String, String> = BTreeMap::new();
        for (eid, v) in &writes {
            e.index("c", IndexRequest {
                items: vec![IndexItem {
                    external_id: eid.clone(),
                    field: "kw".into(),
                    value: FieldValue::String(v.clone()),
                }],
                request_id: None,
            }).unwrap();
            final_value.insert(eid.clone(), v.clone());
        }
        for (eid, v) in &final_value {
            let hits = term_search(&e, "kw", v);
            prop_assert!(hits.contains(eid), "eid {eid} with kw={v} not found in {hits:?}");
        }
    }

    // P2 — after deleting an eid, no term query returns it.
    #[test]
    fn p2_deleted_eid_disappears(
        writes in proptest::collection::vec((eid_strategy(), keyword_value()), 1..40),
        victim_idx in 0usize..40,
    ) {
        let e = fresh();
        let mut eids = vec![];
        for (eid, v) in &writes {
            e.index("c", IndexRequest {
                items: vec![IndexItem {
                    external_id: eid.clone(),
                    field: "kw".into(),
                    value: FieldValue::String(v.clone()),
                }],
                request_id: None,
            }).unwrap();
            eids.push(eid.clone());
        }
        if eids.is_empty() { return Ok(()); }
        let victim = eids[victim_idx % eids.len()].clone();
        e.delete("c", &victim, None).unwrap();
        // No keyword value query may return the victim.
        for v in ["red", "green", "blue", "amber", "violet"] {
            let hits = term_search(&e, "kw", v);
            prop_assert!(!hits.contains(&victim), "deleted {victim} still in kw={v}: {hits:?}");
        }
    }

    // P3 — snapshot then restore into a fresh engine yields identical
    // query results for every keyword value and text token.
    #[test]
    fn p3_snapshot_restore_preserves_queries(
        writes in proptest::collection::vec((eid_strategy(), keyword_value(), text_value()), 1..30)
    ) {
        let e = fresh();
        for (eid, kw, body) in &writes {
            e.index("c", IndexRequest {
                items: vec![
                    IndexItem { external_id: eid.clone(), field: "kw".into(),   value: FieldValue::String(kw.clone()) },
                    IndexItem { external_id: eid.clone(), field: "body".into(), value: FieldValue::String(body.clone()) },
                ],
                request_id: None,
            }).unwrap();
        }
        let snap = e.snapshot().unwrap();
        let restored = Engine::new();
        restored.restore(snap).unwrap();

        for v in ["red", "green", "blue", "amber", "violet"] {
            let mut a = term_search(&e, "kw", v);
            let mut b = term_search(&restored, "kw", v);
            a.sort(); b.sort();
            prop_assert_eq!(a, b, "kw={} diverged after restore", v);
        }
        for tok in ["alpha", "beta", "gamma", "delta", "epsilon"] {
            let mut a = match_search(&e, "body", tok);
            let mut b = match_search(&restored, "body", tok);
            a.sort(); b.sort();
            prop_assert_eq!(a, b, "body match {} diverged after restore", tok);
        }
    }

    // P4 — re-indexing (eid, kw) with a new value removes the old value
    // entirely and installs the new one.
    #[test]
    fn p4_reindex_replaces_value(
        eid in eid_strategy(),
        v1 in keyword_value(),
        v2 in keyword_value(),
    ) {
        prop_assume!(v1 != v2);
        let e = fresh();
        let put = |v: &str| e.index("c", IndexRequest {
            items: vec![IndexItem { external_id: eid.clone(), field: "kw".into(), value: FieldValue::String(v.into()) }],
            request_id: None,
        }).unwrap();
        put(&v1);
        put(&v2);
        prop_assert!(!term_search(&e, "kw", &v1).contains(&eid), "old value {v1} survived re-index");
        prop_assert!(term_search(&e, "kw", &v2).contains(&eid), "new value {v2} missing after re-index");
    }

    // P5 — documents_indexed equals the count of distinct external_ids
    // written (last-write-wins; same eid written twice counts once).
    #[test]
    fn p5_documents_indexed_counts_distinct_eids(
        writes in proptest::collection::vec((eid_strategy(), keyword_value()), 1..50)
    ) {
        let e = fresh();
        let mut distinct = std::collections::BTreeSet::new();
        for (eid, v) in &writes {
            e.index("c", IndexRequest {
                items: vec![IndexItem { external_id: eid.clone(), field: "kw".into(), value: FieldValue::String(v.clone()) }],
                request_id: None,
            }).unwrap();
            distinct.insert(eid.clone());
        }
        let stats = e.stats("c").unwrap();
        prop_assert_eq!(stats.documents_indexed, distinct.len() as u64);
    }

    // P6 — every eid reported in a duplicate group genuinely carries the
    // group's value on that field.
    #[test]
    fn p6_duplicate_groups_are_truthful(
        writes in proptest::collection::vec((eid_strategy(), keyword_value()), 1..60)
    ) {
        let e = fresh();
        // last-write-wins truth table
        let mut truth: BTreeMap<String, String> = BTreeMap::new();
        for (eid, v) in &writes {
            e.index("c", IndexRequest {
                items: vec![IndexItem { external_id: eid.clone(), field: "kw".into(), value: FieldValue::String(v.clone()) }],
                request_id: None,
            }).unwrap();
            truth.insert(eid.clone(), v.clone());
        }
        let resp = e.duplicates("c", DuplicatesRequest {
            field: "kw".into(),
            min_group_size: 2,
            limit: 1000,
            offset: 0,
        }).unwrap();
        for group in &resp.groups {
            let gv = group.value.as_str().unwrap();
            prop_assert!(group.external_ids.len() >= 2, "group below min_group_size");
            for eid in &group.external_ids {
                prop_assert_eq!(
                    truth.get(eid).map(|s| s.as_str()),
                    Some(gv),
                    "eid {} reported in group {} but its real value is {:?}",
                    eid, gv, truth.get(eid)
                );
            }
        }
    }
}

// </HANDWRITE>
