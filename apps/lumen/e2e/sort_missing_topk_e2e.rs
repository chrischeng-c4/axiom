//! #3997 HTTP oracle for sparse high-cardinality keyword sorting.
//!
//! The fixture is sealed with the production segment writer before every read.
//! Its independent oracle covers the complete single-key keyword sort contract:
//! order direction, missing policy, page sizes, native offsets, cursor walks,
//! and both total modes. A final write also proves exact response-cache
//! invalidation still wins over the planner optimisation.

use std::sync::Arc;

use axum_test::TestServer;
use lumen::api::{router, AppState};
use lumen::storage::Engine;
use serde_json::{json, Value};

#[derive(Clone)]
struct Row {
    external_id: String,
    keyword: Option<String>,
}

fn server() -> (TestServer, Arc<Engine>, tempfile::TempDir) {
    let engine = Arc::new(Engine::new());
    let server = TestServer::new(router(AppState::open(engine.clone()))).expect("test server");
    let segment_dir = tempfile::tempdir().expect("segment tempdir");
    (server, engine, segment_dir)
}

fn ids(response: &Value) -> Vec<String> {
    response["hits"]
        .as_array()
        .expect("hits array")
        .iter()
        .map(|hit| {
            hit["external_id"]
                .as_str()
                .expect("hit external_id")
                .to_string()
        })
        .collect()
}

fn scores(response: &Value) -> Vec<f64> {
    response["hits"]
        .as_array()
        .expect("hits array")
        .iter()
        .map(|hit| hit["score"].as_f64().expect("hit score"))
        .collect()
}

fn sort_body(
    order: &str,
    missing: &str,
    limit: u32,
    cursor: Option<String>,
    offset: u32,
    track_total: bool,
) -> Value {
    json!({
        // `exists` is an exact field-presence predicate, not a text/match
        // shortcut. Every fixture row has `group`, so it is the complete set.
        "query":{"exists":{"field":"group"}},
        "sort":[{"field":"kw","order":order,"missing":missing}],
        "limit":limit,
        "cursor":cursor,
        "offset":offset,
        "track_total":track_total,
    })
}

fn expected(rows: &[Row], order: &str, missing: &str) -> Vec<String> {
    let mut rows: Vec<Row> = rows
        .iter()
        .filter(|row| missing != "exclude" || row.keyword.is_some())
        .cloned()
        .collect();
    rows.sort_by(|left, right| {
        let key_order = match (&left.keyword, &right.keyword) {
            (Some(left), Some(right)) => {
                let cmp = left.cmp(right);
                if order == "desc" {
                    cmp.reverse()
                } else {
                    cmp
                }
            }
            (None, Some(_)) if missing == "first" => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) if missing == "first" => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, None) => std::cmp::Ordering::Equal,
        };
        key_order.then_with(|| left.external_id.cmp(&right.external_id))
    });
    rows.into_iter().map(|row| row.external_id).collect()
}

fn expected_total(expected: &[String], missing: &str, limit: u32, track_total: bool) -> u64 {
    // The existing exclude-only sorted planner may stop at a full page with
    // `track_total:false`. Missing-inclusive sorts deliberately retain their
    // historical exact total because their materialized contract included it.
    if !track_total && missing == "exclude" {
        // Its lower-bound planner reads one sentinel past a full page, even
        // for an empty requested page.
        expected.len().min(limit as usize + 1) as u64
    } else {
        expected.len() as u64
    }
}

#[tokio::test]
async fn sealed_sparse_keyword_sort_preserves_full_contract_and_cache() {
    const DOCS: usize = 120;
    let (server, engine, segment_dir) = server();
    server
        .put("/collections/docs")
        .json(&json!({
            "fields": {
                "group": {"type":"keyword"},
                "kw": {"type":"keyword"}
            }
        }))
        .await
        .assert_status_ok();

    let mut items = Vec::new();
    let mut rows = Vec::new();
    for i in 0..DOCS {
        let external_id = format!("d{i:03}");
        items.push(json!({"external_id":external_id,"field":"group","value":"g"}));
        let keyword = (i % 10 != 0).then(|| format!("k{:03}", DOCS - i));
        if let Some(value) = &keyword {
            items.push(json!({"external_id":format!("d{i:03}"),"field":"kw","value":value}));
        }
        rows.push(Row {
            external_id: format!("d{i:03}"),
            keyword,
        });
    }
    server
        .post("/collections/docs/index")
        .json(&json!({"items":items}))
        .await
        .assert_status_ok();

    // Production segment writer: every assertion below uses the sealed reader
    // and its ordinal posting stream, not an in-RAM BTreeMap driver.
    engine
        .flush_to_segments(segment_dir.path(), 17)
        .expect("seal sort fixture with production segment writer");

    for order in ["asc", "desc"] {
        for missing in ["first", "last", "exclude"] {
            let oracle = expected(&rows, order, missing);
            for limit in [0u32, 1, 80, 2_000] {
                for track_total in [true, false] {
                    let response: Value = server
                        .post("/collections/docs/search")
                        .json(&sort_body(order, missing, limit, None, 0, track_total))
                        .await
                        .json();
                    let expected_page = oracle
                        .iter()
                        .take(limit as usize)
                        .cloned()
                        .collect::<Vec<_>>();
                    assert_eq!(
                        ids(&response),
                        expected_page,
                        "{order}/{missing}/limit={limit}/track_total={track_total}"
                    );
                    let expected_scores = expected_page
                        .iter()
                        .map(|external_id| {
                            if rows
                                .iter()
                                .find(|row| &row.external_id == external_id)
                                .expect("oracle id belongs to fixture")
                                .keyword
                                .is_some()
                            {
                                1.0
                            } else {
                                0.0
                            }
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(
                        scores(&response),
                        expected_scores,
                        "{order}/{missing}/limit={limit}/track_total={track_total} scores"
                    );
                    assert_eq!(
                        response["total"].as_u64(),
                        Some(expected_total(&oracle, missing, limit, track_total)),
                        "{order}/{missing}/limit={limit}/track_total={track_total} total"
                    );
                }
            }

            let offset_response: Value = server
                .post("/collections/docs/search")
                .json(&sort_body(order, missing, 7, None, 50, true))
                .await
                .json();
            assert_eq!(
                ids(&offset_response),
                oracle[50..57],
                "{order}/{missing} offset page"
            );
            assert_eq!(offset_response["total"].as_u64(), Some(oracle.len() as u64));

            let mut cursor = None;
            let mut paged = Vec::new();
            loop {
                let response: Value = server
                    .post("/collections/docs/search")
                    .json(&sort_body(order, missing, 13, cursor.take(), 0, true))
                    .await
                    .json();
                paged.extend(ids(&response));
                cursor = response["cursor"].as_str().map(str::to_owned);
                if cursor.is_none() {
                    break;
                }
            }
            assert_eq!(paged, oracle, "{order}/{missing} cursor walk");
        }
    }

    // Populate the exact request cache, then mutate this collection. The
    // response cannot retain the old first page after the successful write.
    let cache_seed: Value = server
        .post("/collections/docs/search")
        .json(&sort_body("asc", "last", 13, None, 0, true))
        .await
        .json();
    assert_eq!(ids(&cache_seed).len(), 13);
    server
        .post("/collections/docs/index")
        .json(&json!({"items":[
            {"external_id":"new","field":"group","value":"g"},
            {"external_id":"new","field":"kw","value":"k000"}
        ]}))
        .await
        .assert_status_ok();
    let after_write: Value = server
        .post("/collections/docs/search")
        .json(&sort_body("asc", "last", 13, None, 0, true))
        .await
        .json();
    assert_eq!(ids(&after_write).first().map(String::as_str), Some("new"));
    assert_eq!(after_write["total"].as_u64(), Some((DOCS + 1) as u64));
}
