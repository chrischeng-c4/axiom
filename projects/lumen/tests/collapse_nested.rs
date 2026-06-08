//! Collapse-on-search + nested `group` (data-table) correctness.
//!
//! * collapse: grouping a keyword field returns one hit per distinct value with
//!   score == max member score (differential vs the engine's own per-doc scores).
//! * group-nested: modelling a group field as child docs (one per element) +
//!   collapse(parent_row_id) returns EXACTLY the parents with a qualifying
//!   element — and NEVER cross-element false-matches (correlation preserved).
//! * enum (子母選單): path eq + per-depth level_match via keyword + set.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use proptest::prelude::*;

use lumen::storage::Engine;
use lumen::types::{
    Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, HasChildQuery, IndexItem,
    IndexRequest, MatchOp, MatchQuery, QueryNode, RangeQuery, SearchRequest, TermQuery,
};

fn spec(t: FieldType, analyzer: Option<Analyzer>) -> FieldSpec {
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

/// Child-collection schema: one doc per group element.
fn child_schema() -> CreateCollectionRequest {
    let mut fields = BTreeMap::new();
    fields.insert("parent".into(), spec(FieldType::Keyword, None));
    fields.insert("sku".into(), spec(FieldType::Keyword, None));
    fields.insert("qty".into(), spec(FieldType::Number, None));
    fields.insert(
        "body".into(),
        spec(FieldType::Text, Some(Analyzer::WhitespaceLower)),
    );
    CreateCollectionRequest { fields }
}

fn search(e: &Engine, req: SearchRequest) -> Vec<(String, f32)> {
    e.search("c", req)
        .unwrap()
        .hits
        .into_iter()
        .map(|h| (h.external_id, h.score))
        .collect()
}

fn base(query: QueryNode) -> SearchRequest {
    SearchRequest {
        query,
        limit: 1_000_000,
        cursor: None,
        sort: None,
        track_total: true,
        collapse: None,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(250))]

    /// collapse(field) == brute-force group-by(field) with score = max member.
    #[test]
    fn collapse_equals_groupby(
        rows in proptest::collection::vec(
            (prop::sample::select(vec!["g0","g1","g2","g3"]), any::<bool>()),
            1..40,
        )
    ) {
        let e = Arc::new(Engine::new());
        e.create_collection("c", child_schema()).unwrap();
        let mut group_of: BTreeMap<String, String> = BTreeMap::new();
        for (i, (g, tok)) in rows.iter().enumerate() {
            let eid = format!("d{i}");
            e.index("c", IndexRequest { items: vec![
                IndexItem { external_id: eid.clone(), field: "parent".into(), value: FieldValue::String(g.to_string()) },
                IndexItem { external_id: eid.clone(), field: "body".into(),
                            value: FieldValue::String(if *tok { "tok x".into() } else { "x".into() }) },
            ], request_id: None }).unwrap();
            group_of.insert(eid, g.to_string());
        }
        let q = QueryNode::Match(MatchQuery { field: "body".into(), text: "tok".into(), op: MatchOp::Or });

        // Per-doc scores (no collapse), then group them by parent → max.
        let per_doc = search(&e, base(q.clone()));
        let mut expected: BTreeMap<String, f32> = BTreeMap::new();
        for (eid, score) in &per_doc {
            let g = group_of[eid].clone();
            let slot = expected.entry(g).or_insert(f32::NEG_INFINITY);
            if *score > *slot { *slot = *score; }
        }

        // Collapsed search.
        let mut req = base(q);
        req.collapse = Some("parent".into());
        let collapsed: BTreeMap<String, f32> = search(&e, req).into_iter().collect();

        prop_assert_eq!(collapsed.keys().cloned().collect::<BTreeSet<_>>(),
                        expected.keys().cloned().collect::<BTreeSet<_>>(), "group set");
        for (g, max) in &expected {
            prop_assert!((collapsed[g] - *max).abs() < 1e-4, "group {} score {} != max {}", g, collapsed[g], max);
        }
    }

    /// group-nested: child docs + collapse(parent) == parents with a qualifying
    /// ELEMENT (sku='S0' AND qty>=5 in the SAME element). No cross-element match.
    #[test]
    fn group_nested_correlation(
        parents in proptest::collection::vec(
            // each parent = a list of elements (sku, qty)
            proptest::collection::vec(
                (prop::sample::select(vec!["S0","S1","S2"]), 0u32..10),
                1..5,
            ),
            1..25,
        )
    ) {
        let e = Arc::new(Engine::new());
        e.create_collection("c", child_schema()).unwrap();
        let mut expected: BTreeSet<String> = BTreeSet::new();
        for (p, elems) in parents.iter().enumerate() {
            let parent = format!("p{p}");
            for (j, (sku, qty)) in elems.iter().enumerate() {
                let cid = format!("{parent}#{j}");
                e.index("c", IndexRequest { items: vec![
                    IndexItem { external_id: cid.clone(), field: "parent".into(), value: FieldValue::String(parent.clone()) },
                    IndexItem { external_id: cid.clone(), field: "sku".into(), value: FieldValue::String(sku.to_string()) },
                    IndexItem { external_id: cid.clone(), field: "qty".into(), value: FieldValue::Number(*qty as f64) },
                ], request_id: None }).unwrap();
                if *sku == "S0" && *qty >= 5 { expected.insert(parent.clone()); }
            }
        }

        // child filter: sku='S0' AND qty>=5, collapse to distinct parent.
        let mut req = base(QueryNode::And(vec![
            QueryNode::Term(TermQuery { field: "sku".into(), value: FieldValue::String("S0".into()) }),
            QueryNode::Range(RangeQuery { field: "qty".into(), gt: None, gte: Some(5.0), lt: None, lte: None }),
        ]));
        req.collapse = Some("parent".into());
        let got: BTreeSet<String> = search(&e, req).into_iter().map(|(p, _)| p).collect();
        prop_assert_eq!(got, expected, "nested group correlation");
    }
}

/// Early-termination collapse (track_total=false): with a large limit it must
/// still return the FULL correct distinct-parent set; with a small limit, a
/// valid subset of exactly that size. Every returned parent must genuinely
/// have a qualifying element (no false match under early-term).
#[test]
fn collapse_early_term_correct() {
    let e = Arc::new(Engine::new());
    e.create_collection("c", child_schema()).unwrap();
    let mut expected: BTreeSet<String> = BTreeSet::new();
    for p in 0..200 {
        let parent = format!("p{p}");
        // deterministic: parent p has a qualifying element iff p % 3 == 0
        let (sku, qty) = if p % 3 == 0 { ("S0", 7.0) } else { ("S1", 2.0) };
        e.index(
            "c",
            IndexRequest {
                items: vec![
                    IndexItem {
                        external_id: format!("{parent}#0"),
                        field: "parent".into(),
                        value: FieldValue::String(parent.clone()),
                    },
                    IndexItem {
                        external_id: format!("{parent}#0"),
                        field: "sku".into(),
                        value: FieldValue::String(sku.into()),
                    },
                    IndexItem {
                        external_id: format!("{parent}#0"),
                        field: "qty".into(),
                        value: FieldValue::Number(qty),
                    },
                ],
                request_id: None,
            },
        )
        .unwrap();
        if sku == "S0" && qty >= 5.0 {
            expected.insert(parent);
        }
    }
    let q = || {
        QueryNode::And(vec![
            QueryNode::Term(TermQuery {
                field: "sku".into(),
                value: FieldValue::String("S0".into()),
            }),
            QueryNode::Range(RangeQuery {
                field: "qty".into(),
                gt: None,
                gte: Some(5.0),
                lt: None,
                lte: None,
            }),
        ])
    };
    let run = |limit: u32| -> Vec<String> {
        let req = SearchRequest {
            query: q(),
            limit,
            cursor: None,
            sort: None,
            track_total: false,
            collapse: Some("parent".into()),
        };
        e.search("c", req)
            .unwrap()
            .hits
            .into_iter()
            .map(|h| h.external_id)
            .collect()
    };
    // Large limit → full correct set (early-term never hits the cap).
    let full: BTreeSet<String> = run(100_000).into_iter().collect();
    assert_eq!(full, expected, "early-term full set");
    // Small limit → exactly `limit` valid parents (each really qualifies).
    let page = run(5);
    assert_eq!(page.len(), 5, "early-term page size");
    for p in &page {
        assert!(
            expected.contains(p),
            "early-term returned non-qualifying {p}"
        );
    }
}

/// Explicit cross-element false-match guard: parent with element A (sku=S0,qty=1)
/// and element B (sku=S9,qty=9). sku=S0 AND qty>=5 must NOT match (no single
/// element has both).
#[test]
fn no_cross_element_false_match() {
    let e = Arc::new(Engine::new());
    e.create_collection("c", child_schema()).unwrap();
    let put = |cid: &str, sku: &str, qty: f64| {
        e.index(
            "c",
            IndexRequest {
                items: vec![
                    IndexItem {
                        external_id: cid.into(),
                        field: "parent".into(),
                        value: FieldValue::String("p1".into()),
                    },
                    IndexItem {
                        external_id: cid.into(),
                        field: "sku".into(),
                        value: FieldValue::String(sku.into()),
                    },
                    IndexItem {
                        external_id: cid.into(),
                        field: "qty".into(),
                        value: FieldValue::Number(qty),
                    },
                ],
                request_id: None,
            },
        )
        .unwrap();
    };
    put("p1#0", "S0", 1.0); // matches sku, not qty
    put("p1#1", "S9", 9.0); // matches qty, not sku
    let mut req = base(QueryNode::And(vec![
        QueryNode::Term(TermQuery {
            field: "sku".into(),
            value: FieldValue::String("S0".into()),
        }),
        QueryNode::Range(RangeQuery {
            field: "qty".into(),
            gt: None,
            gte: Some(5.0),
            lt: None,
            lte: None,
        }),
    ]));
    req.collapse = Some("parent".into());
    let got: Vec<(String, f32)> = e
        .search("c", req)
        .unwrap()
        .hits
        .into_iter()
        .map(|h| (h.external_id, h.score))
        .collect();
    assert!(got.is_empty(), "cross-element false match: {got:?}");

    // Sanity: a single element with both DOES match.
    put("p1#2", "S0", 7.0);
    let mut req2 = base(QueryNode::And(vec![
        QueryNode::Term(TermQuery {
            field: "sku".into(),
            value: FieldValue::String("S0".into()),
        }),
        QueryNode::Range(RangeQuery {
            field: "qty".into(),
            gt: None,
            gte: Some(5.0),
            lt: None,
            lte: None,
        }),
    ]));
    req2.collapse = Some("parent".into());
    let got2: Vec<String> = e
        .search("c", req2)
        .unwrap()
        .hits
        .into_iter()
        .map(|h| h.external_id)
        .collect();
    assert_eq!(
        got2,
        vec!["p1".to_string()],
        "qualifying element should match"
    );
}

/// has_child: nested condition as a first-class boolean clause. A query on the
/// PARENT collection referencing a child sub-query must compose under AND / NOT
/// exactly like a brute-force "parent has a qualifying child element".
#[test]
fn has_child_composes_in_boolean_tree() {
    let e = Arc::new(Engine::new());
    let mut pf = BTreeMap::new();
    pf.insert("city".into(), spec(FieldType::Keyword, None));
    e.create_collection("parents", CreateCollectionRequest { fields: pf })
        .unwrap();
    e.create_collection("items", child_schema()).unwrap();

    let mut city_of: BTreeMap<String, &str> = BTreeMap::new();
    let mut qualifies: BTreeSet<String> = BTreeSet::new();
    for p in 0..60 {
        let parent = format!("p{p}");
        let city = if p % 2 == 0 { "taipei" } else { "tokyo" };
        e.index(
            "parents",
            IndexRequest {
                items: vec![IndexItem {
                    external_id: parent.clone(),
                    field: "city".into(),
                    value: FieldValue::String(city.into()),
                }],
                request_id: None,
            },
        )
        .unwrap();
        city_of.insert(parent.clone(), city);
        // qualifying element (sku=S0 AND qty>=5) iff p%3==0
        let (sku, qty) = if p % 3 == 0 { ("S0", 7.0) } else { ("S1", 1.0) };
        e.index(
            "items",
            IndexRequest {
                items: vec![
                    IndexItem {
                        external_id: format!("{parent}#0"),
                        field: "parent".into(),
                        value: FieldValue::String(parent.clone()),
                    },
                    IndexItem {
                        external_id: format!("{parent}#0"),
                        field: "sku".into(),
                        value: FieldValue::String(sku.into()),
                    },
                    IndexItem {
                        external_id: format!("{parent}#0"),
                        field: "qty".into(),
                        value: FieldValue::Number(qty),
                    },
                ],
                request_id: None,
            },
        )
        .unwrap();
        if sku == "S0" && qty >= 5.0 {
            qualifies.insert(parent);
        }
    }

    let hc = || {
        QueryNode::HasChild(HasChildQuery {
            collection: "items".into(),
            field: "parent".into(),
            query: Box::new(QueryNode::And(vec![
                QueryNode::Term(TermQuery {
                    field: "sku".into(),
                    value: FieldValue::String("S0".into()),
                }),
                QueryNode::Range(RangeQuery {
                    field: "qty".into(),
                    gt: None,
                    gte: Some(5.0),
                    lt: None,
                    lte: None,
                }),
            ])),
        })
    };
    let on_parents = |q: QueryNode| -> BTreeSet<String> {
        e.search(
            "parents",
            SearchRequest {
                query: q,
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
    };

    // standalone has_child → every parent with a qualifying element.
    assert_eq!(on_parents(hc()), qualifies, "standalone has_child");

    // AND[city=taipei, has_child] → taipei ∩ qualifies.
    let expect_and: BTreeSet<String> = qualifies
        .iter()
        .filter(|p| city_of[*p] == "taipei")
        .cloned()
        .collect();
    assert_eq!(
        on_parents(QueryNode::And(vec![
            QueryNode::Term(TermQuery {
                field: "city".into(),
                value: FieldValue::String("taipei".into())
            }),
            hc(),
        ])),
        expect_and,
        "AND[city, has_child]"
    );

    // AND[city=tokyo, NOT has_child] → tokyo parents WITHOUT a qualifying element.
    let expect_andnot: BTreeSet<String> = city_of
        .iter()
        .filter(|(p, c)| **c == "tokyo" && !qualifies.contains(*p))
        .map(|(p, _)| p.clone())
        .collect();
    assert_eq!(
        on_parents(QueryNode::And(vec![
            QueryNode::Term(TermQuery {
                field: "city".into(),
                value: FieldValue::String("tokyo".into())
            }),
            QueryNode::Not(Box::new(hc())),
        ])),
        expect_andnot,
        "AND[city, NOT has_child]"
    );
}

/// CJK `has`/contains: a `text` field with the Ngram analyzer matches a Chinese
/// substring with no whitespace. `match name "手機殼"` (default op=And over the
/// query's char-ngrams) returns exactly the docs containing that substring.
#[test]
fn ngram_cjk_substring() {
    let mut fields = BTreeMap::new();
    fields.insert("name".into(), spec(FieldType::Text, Some(Analyzer::Ngram)));
    let e = Arc::new(Engine::new());
    e.create_collection("c", CreateCollectionRequest { fields })
        .unwrap();
    let put = |eid: &str, name: &str| {
        e.index(
            "c",
            IndexRequest {
                items: vec![IndexItem {
                    external_id: eid.into(),
                    field: "name".into(),
                    value: FieldValue::String(name.into()),
                }],
                request_id: None,
            },
        )
        .unwrap();
    };
    put("a", "藍色手機殼保護套"); // contains 手機殼
    put("b", "透明手機殼"); // contains 手機殼
    put("c", "手機架"); // 手機 but NOT 手機殼
    put("d", "筆記型電腦包"); // unrelated
    let ids: BTreeSet<String> = e
        .search(
            "c",
            base(QueryNode::Match(MatchQuery {
                field: "name".into(),
                text: "手機殼".into(),
                op: MatchOp::And,
            })),
        )
        .unwrap()
        .hits
        .into_iter()
        .map(|h| h.external_id)
        .collect();
    assert_eq!(
        ids,
        BTreeSet::from(["a".to_string(), "b".to_string()]),
        "Ngram CJK substring should match only docs containing 手機殼"
    );
}

/// enum (子母選單): full path as keyword + per-depth tokens as a set →
/// path eq and level_match (depth N component == X) both work.
#[test]
fn enum_path_and_level_match() {
    let mut fields = BTreeMap::new();
    fields.insert("enum_path".into(), spec(FieldType::Keyword, None));
    let mut set_spec = spec(FieldType::Keyword, None);
    set_spec.multi = Some(true); // keyword + multi → set
    fields.insert("enum_levels".into(), set_spec);
    let e = Arc::new(Engine::new());
    e.create_collection("c", CreateCollectionRequest { fields })
        .unwrap();

    // Encode "A|B|C" → path + levels {"1|A","2|B","3|C"}.
    let put = |eid: &str, path: &[&str]| {
        let full = path.join("|");
        let levels: Vec<String> = path
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{}|{}", i + 1, c))
            .collect();
        e.index(
            "c",
            IndexRequest {
                items: vec![
                    IndexItem {
                        external_id: eid.into(),
                        field: "enum_path".into(),
                        value: FieldValue::String(full),
                    },
                    IndexItem {
                        external_id: eid.into(),
                        field: "enum_levels".into(),
                        value: FieldValue::StringList(levels),
                    },
                ],
                request_id: None,
            },
        )
        .unwrap();
    };
    put("a", &["A", "B", "C"]);
    put("b", &["A", "B", "D"]); // shares level 1,2 with a; differs at 3
    put("c", &["A", "X", "C"]); // differs at 2

    let ids = |req: SearchRequest| -> BTreeSet<String> {
        e.search("c", req)
            .unwrap()
            .hits
            .into_iter()
            .map(|h| h.external_id)
            .collect()
    };

    // path eq: "A|B|C" → only a.
    let eq = ids(base(QueryNode::Term(TermQuery {
        field: "enum_path".into(),
        value: FieldValue::String("A|B|C".into()),
    })));
    assert_eq!(eq, BTreeSet::from(["a".to_string()]));

    // level_match depth 2 == "B" → a and b (both A|B|*), not c (A|X|*).
    let lvl2b = ids(base(QueryNode::Term(TermQuery {
        field: "enum_levels".into(),
        value: FieldValue::String("2|B".into()),
    })));
    assert_eq!(lvl2b, BTreeSet::from(["a".to_string(), "b".to_string()]));

    // level_match depth 3 == "C" → a and c, not b (ends in D).
    let lvl3c = ids(base(QueryNode::Term(TermQuery {
        field: "enum_levels".into(),
        value: FieldValue::String("3|C".into()),
    })));
    assert_eq!(lvl3c, BTreeSet::from(["a".to_string(), "c".to_string()]));
}
