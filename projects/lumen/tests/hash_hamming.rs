// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! Hash field + Hamming near-duplicate search: index 64-bit hex hashes, then
//! retrieve every doc within a Hamming-distance threshold, ranked by
//! similarity. Also verifies a `hamming` clause composes inside a boolean AND.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use lumen::storage::Engine;
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, HammingQuery, IndexItem,
    IndexRequest, QueryNode, SearchRequest, TermQuery,
};

fn spec(ft: FieldType) -> FieldSpec {
    FieldSpec {
        field_type: ft,
        analyzer: None,
        multi: None,
        dim: None,
        metric: None,
        backend: None,
        quantize: None,
    }
}

fn schema() -> CreateCollectionRequest {
    let mut f = BTreeMap::new();
    f.insert("ph".into(), spec(FieldType::Hash));
    f.insert("kind".into(), spec(FieldType::Keyword));
    CreateCollectionRequest { fields: f }
}

fn index_doc(e: &Engine, eid: &str, hex: &str, kind: &str) {
    e.index(
        "imgs",
        IndexRequest {
            items: vec![
                IndexItem {
                    external_id: eid.into(),
                    field: "ph".into(),
                    value: FieldValue::String(hex.into()),
                },
                IndexItem {
                    external_id: eid.into(),
                    field: "kind".into(),
                    value: FieldValue::String(kind.into()),
                },
            ],
            request_id: None,
        },
    )
    .unwrap();
}

fn run(e: &Engine, query: QueryNode) -> Vec<(String, f32)> {
    e.search(
        "imgs",
        SearchRequest {
            query,
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
    .map(|h| (h.external_id, h.score))
    .collect()
}

fn hamming(field_hash: &str, max: u32) -> QueryNode {
    QueryNode::Hamming(HammingQuery {
        field: "ph".into(),
        hash: field_hash.into(),
        max_distance: max,
    })
}

fn ids(hits: &[(String, f32)]) -> BTreeSet<String> {
    hits.iter().map(|(id, _)| id.clone()).collect()
}

#[test]
fn hamming_returns_within_threshold_ranked_by_similarity() {
    let e = Arc::new(Engine::new());
    e.create_collection("imgs", schema()).unwrap();
    index_doc(&e, "exact", "0000000000000000", "photo"); // distance 0
    index_doc(&e, "one", "0000000000000001", "photo"); // distance 1
    index_doc(&e, "eight", "00000000000000ff", "photo"); // distance 8
    index_doc(&e, "all", "ffffffffffffffff", "art"); // distance 64

    // threshold 4 → only exact + one (0x-prefix accepted).
    let hits = run(&e, hamming("0x0000000000000000", 4));
    assert_eq!(
        ids(&hits),
        ["exact", "one"].iter().map(|s| s.to_string()).collect()
    );

    // exact (distance 0) ranks above one (distance 1).
    let mut ranked = hits.clone();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    assert_eq!(ranked[0].0, "exact");
    assert!(
        ranked[0].1 > ranked[1].1,
        "closer hash scores higher: {ranked:?}"
    );

    // threshold 8 → adds eight, still excludes all.
    let ids8 = ids(&run(&e, hamming("0000000000000000", 8)));
    assert!(
        ids8.contains("eight") && !ids8.contains("all"),
        "threshold 8: {ids8:?}"
    );

    // threshold 64 → matches everything.
    assert_eq!(run(&e, hamming("0000000000000000", 64)).len(), 4);
}

#[test]
fn hamming_composes_in_boolean_and() {
    let e = Arc::new(Engine::new());
    e.create_collection("imgs", schema()).unwrap();
    index_doc(&e, "exact", "0000000000000000", "photo"); // near + photo
    index_doc(&e, "one", "0000000000000001", "art"); // near + art
    index_doc(&e, "all", "ffffffffffffffff", "photo"); // far + photo

    // near (≤4 of zero) AND kind=photo → only "exact" (one is art, all is far).
    let q = QueryNode::And(vec![
        hamming("0000000000000000", 4),
        QueryNode::Term(TermQuery {
            field: "kind".into(),
            value: FieldValue::String("photo".into()),
        }),
    ]);
    assert_eq!(
        ids(&run(&e, q)),
        ["exact"].iter().map(|s| s.to_string()).collect()
    );
}

#[test]
fn invalid_hex_hash_is_rejected() {
    let e = Arc::new(Engine::new());
    e.create_collection("imgs", schema()).unwrap();
    let bad = e.index(
        "imgs",
        IndexRequest {
            items: vec![IndexItem {
                external_id: "x".into(),
                field: "ph".into(),
                value: FieldValue::String("not-hex-zzz".into()),
            }],
            request_id: None,
        },
    );
    assert!(bad.is_err(), "non-hex hash value must be rejected");
}
// CODEGEN-END
