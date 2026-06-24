// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! Differential test for the query planner: for randomized corpora and every
//! specialized query shape (term / range / boolean AND / filtered ranked search
//! / sort-by-field), the planner's result SET must equal a brute-force
//! evaluation of the query semantics over the corpus, and sort results must be
//! in field order. A faster-but-wrong planner is a failure, not a win.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use proptest::prelude::*;

use lumen::storage::Engine;
use lumen::types::{
    Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    MatchOp, MatchQuery, QueryNode, RangeQuery, SearchRequest, SortOrder, SortSpec, TermQuery,
};

fn fieldspec(t: FieldType, analyzer: Option<Analyzer>) -> FieldSpec {
    FieldSpec {
        field_type: t,
        analyzer,
        multi: None,
        dim: None,
        metric: None,
        backend: None,
        quantize: None,
    }
}

fn schema() -> CreateCollectionRequest {
    let mut fields = BTreeMap::new();
    fields.insert("kw".into(), fieldspec(FieldType::Keyword, None));
    fields.insert("n".into(), fieldspec(FieldType::Number, None));
    fields.insert(
        "body".into(),
        fieldspec(FieldType::Text, Some(Analyzer::WhitespaceLower)),
    );
    CreateCollectionRequest { fields }
}

#[derive(Clone, Copy)]
struct Doc {
    kw: char,
    n: u32,
    tok: bool,
}

fn search_ids(e: &Engine, req: SearchRequest) -> Vec<(String, f32)> {
    e.search("c", req)
        .unwrap()
        .hits
        .into_iter()
        .map(|h| (h.external_id, h.score))
        .collect()
}

fn req(query: QueryNode, sort: Option<Vec<SortSpec>>) -> SearchRequest {
    SearchRequest {
        query,
        limit: 100_000, // larger than any corpus → page == full match set
        cursor: None,
        sort,
        track_total: true,
        collapse: None,
    }
}

fn set_of(ids: &[(String, f32)]) -> BTreeSet<String> {
    ids.iter().map(|(e, _)| e.clone()).collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn planner_matches_bruteforce(
        writes in proptest::collection::vec(
            (
                proptest::sample::select(vec!["a", "b", "c", "d", "e", "f"]),
                prop::sample::select(vec!['a', 'b', 'c', 'd']),
                0u32..20,
                any::<bool>(),
            ),
            1..50,
        )
    ) {
        let e = Arc::new(Engine::new());
        e.create_collection("c", schema()).unwrap();

        // Apply writes; last write per eid wins (full-doc replace each time).
        let mut state: BTreeMap<String, Doc> = BTreeMap::new();
        for (eid, kw, n, tok) in &writes {
            let eid = eid.to_string();
            e.index("c", IndexRequest {
                items: vec![
                    IndexItem { external_id: eid.clone(), field: "kw".into(),
                                value: FieldValue::String(kw.to_string()), version: None, },
                    IndexItem { external_id: eid.clone(), field: "n".into(),
                                value: FieldValue::Number(*n as f64), version: None, },
                    IndexItem { external_id: eid.clone(), field: "body".into(),
                                value: FieldValue::String(
                                    if *tok { "tok filler".into() } else { "filler".into() }), version: None, },
                ],
                request_id: None,
            }).unwrap();
            state.insert(eid, Doc { kw: *kw, n: *n, tok: *tok });
        }

        // Brute-force predicates over the final corpus state.
        let bf = |pred: &dyn Fn(&Doc) -> bool| -> BTreeSet<String> {
            state.iter().filter(|(_, d)| pred(d)).map(|(e, _)| e.clone()).collect()
        };

        // 1. term kw == 'c'
        let got = search_ids(&e, req(QueryNode::Term(TermQuery {
            field: "kw".into(), value: FieldValue::String("c".into()) }), None));
        prop_assert_eq!(set_of(&got), bf(&|d| d.kw == 'c'), "term");

        // 2. range n in [5,15)
        let range = || QueryNode::Range(RangeQuery {
            field: "n".into(), gt: None, gte: Some(5.0), lt: Some(15.0), lte: None });
        let got = search_ids(&e, req(range(), None));
        prop_assert_eq!(set_of(&got), bf(&|d| d.n >= 5 && d.n < 15), "range");

        // 3. AND[term kw=='c', range n[5,15)]
        let and2 = QueryNode::And(vec![
            QueryNode::Term(TermQuery { field: "kw".into(), value: FieldValue::String("c".into()) }),
            range(),
        ]);
        let got = search_ids(&e, req(and2, None));
        prop_assert_eq!(set_of(&got), bf(&|d| d.kw == 'c' && d.n >= 5 && d.n < 15), "and2");

        // 4. filtered search: AND[match body 'tok', term kw=='c', range n[5,15)]
        let fs = QueryNode::And(vec![
            QueryNode::Match(MatchQuery { field: "body".into(), text: "tok".into(), op: MatchOp::And }),
            QueryNode::Term(TermQuery { field: "kw".into(), value: FieldValue::String("c".into()) }),
            range(),
        ]);
        let got = search_ids(&e, req(fs, None));
        prop_assert_eq!(set_of(&got), bf(&|d| d.tok && d.kw == 'c' && d.n >= 5 && d.n < 15), "filtered_search");

        // 5. sort by n asc, filter kw=='c': set must match, and n non-decreasing.
        let got = search_ids(&e, req(
            QueryNode::Term(TermQuery { field: "kw".into(), value: FieldValue::String("c".into()) }),
            Some(vec![SortSpec { field: "n".into(), order: SortOrder::Asc }])));
        prop_assert_eq!(set_of(&got), bf(&|d| d.kw == 'c'), "sort set");
        // scores carry the sort value → must be non-decreasing for asc.
        for w in got.windows(2) {
            prop_assert!(w[0].1 <= w[1].1, "sort asc not ordered: {:?}", got);
        }

        // 6. sort by kw asc, filter n[5,15]: set must match, and keyword non-decreasing.
        let got = search_ids(&e, req(
            range(),
            Some(vec![SortSpec { field: "kw".into(), order: SortOrder::Asc }])));
        prop_assert_eq!(set_of(&got), bf(&|d| d.n >= 5 && d.n < 15), "keyword sort set");
        for w in got.windows(2) {
            let a = state.get(&w[0].0).unwrap();
            let b = state.get(&w[1].0).unwrap();
            prop_assert!(
                a.kw <= b.kw,
                "keyword sort asc not ordered: {:?}",
                got
            );
        }

        // 7. composite sort by kw asc, n desc: full set + lexicographic order.
        let got = search_ids(&e, req(
            QueryNode::Range(RangeQuery { field: "n".into(), gt: None, gte: None, lt: None, lte: None }),
            Some(vec![
                SortSpec { field: "kw".into(), order: SortOrder::Asc },
                SortSpec { field: "n".into(), order: SortOrder::Desc },
            ])));
        prop_assert_eq!(set_of(&got), bf(&|_| true), "composite sort set");
        for w in got.windows(2) {
            let a = state.get(&w[0].0).unwrap();
            let b = state.get(&w[1].0).unwrap();
            prop_assert!(
                a.kw < b.kw || (a.kw == b.kw && a.n >= b.n),
                "composite keyword+number sort not ordered: {:?}",
                got
            );
        }

        // 8. sort by n desc (no filter via unbounded range): full set + non-increasing.
        let got = search_ids(&e, req(
            QueryNode::Range(RangeQuery { field: "n".into(), gt: None, gte: None, lt: None, lte: None }),
            Some(vec![SortSpec { field: "n".into(), order: SortOrder::Desc }])));
        prop_assert_eq!(set_of(&got), bf(&|_| true), "sort desc set");
        for w in got.windows(2) {
            prop_assert!(w[0].1 >= w[1].1, "sort desc not ordered");
        }
    }
}
// CODEGEN-END
