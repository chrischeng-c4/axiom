//! End-to-end test of [`lumen::client::Client`] against the real axum
//! router, hosted over real HTTP via `axum-test`'s HttpRandomPort
//! transport. Doubles as a wire-format sanity check that the typed
//! client and the real handlers agree on every JSON shape.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum_test::{TestServer, TestServerConfig, Transport};
use lumen::api::{router, AppState};
use lumen::client::Client;
use lumen::storage::Engine;
use lumen::types::{
    DuplicatesRequest, FieldSpec, FieldType, FieldValue, IndexItem, MatchOp, MatchQuery, QueryNode,
    SearchRequest, TermQuery,
};

fn boot_server() -> TestServer {
    let engine = Arc::new(Engine::new());
    let app = router(AppState::open(engine));
    let config = TestServerConfig {
        transport: Some(Transport::HttpRandomPort),
        ..TestServerConfig::default()
    };
    TestServer::new_with_config(app, config).expect("test server")
}

fn schema() -> BTreeMap<String, FieldSpec> {
    let mut fields = BTreeMap::new();
    fields.insert(
        "bio".into(),
        FieldSpec {
            field_type: FieldType::Text,
            analyzer: None,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        },
    );
    fields.insert(
        "email".into(),
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
    fields
}

#[tokio::test]
async fn client_round_trip_create_index_search_duplicates_delete() {
    let server = boot_server();
    let base = server
        .server_address()
        .expect("HttpRandomPort transport must surface an address")
        .to_string();
    // server_address() returns a trailing-slash URL; Client::collection_base
    // strips it, so either form is fine.
    let client = Client::new(base);

    // -- create_collection --------------------------------------------------
    let created = client
        .create_collection("users", schema())
        .await
        .expect("create_collection");
    assert_eq!(created.collection_id, "users");
    assert_eq!(created.version, 1);
    assert_eq!(created.fields_count, 2);

    // -- list_collections ---------------------------------------------------
    let listed = client.list_collections().await.expect("list_collections");
    assert_eq!(listed, vec!["users".to_string()]);

    // -- index --------------------------------------------------------------
    let items = vec![
        IndexItem {
            external_id: "u1".into(),
            field: "bio".into(),
            value: FieldValue::String("senior rust engineer".into()),
        },
        IndexItem {
            external_id: "u1".into(),
            field: "email".into(),
            value: FieldValue::String("a@x.com".into()),
        },
        IndexItem {
            external_id: "u2".into(),
            field: "bio".into(),
            value: FieldValue::String("junior rust engineer".into()),
        },
        IndexItem {
            external_id: "u2".into(),
            field: "email".into(),
            value: FieldValue::String("a@x.com".into()),
        },
        IndexItem {
            external_id: "u3".into(),
            field: "email".into(),
            value: FieldValue::String("b@y.com".into()),
        },
    ];
    let indexed = client
        .index("users", items, Some("req-1".into()))
        .await
        .expect("index");
    assert_eq!(indexed.indexed, 5);
    assert!(indexed.bytes_written.contains_key("bio"));
    assert!(indexed.bytes_written.contains_key("email"));

    // -- search (match) -----------------------------------------------------
    let search = client
        .search(
            "users",
            SearchRequest {
                query: QueryNode::Match(MatchQuery {
                    field: "bio".into(),
                    text: "rust engineer".into(),
                    op: MatchOp::And,
                }),
                limit: 10,
                cursor: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .await
        .expect("search");
    assert_eq!(search.total, 2);
    let eids: Vec<&str> = search.hits.iter().map(|h| h.external_id.as_str()).collect();
    assert!(eids.contains(&"u1"));
    assert!(eids.contains(&"u2"));

    // -- duplicates ---------------------------------------------------------
    let dups = client
        .duplicates(
            "users",
            DuplicatesRequest {
                field: "email".into(),
                min_group_size: 2,
                limit: 100,
                offset: 0,
            },
        )
        .await
        .expect("duplicates");
    assert_eq!(dups.groups.len(), 1);
    assert_eq!(dups.groups[0].external_ids.len(), 2);

    // -- stats --------------------------------------------------------------
    let stats = client.stats("users").await.expect("stats");
    assert_eq!(stats.documents_indexed, 3);

    // -- delete one external_id --------------------------------------------
    client.delete("users", "u1", None).await.expect("delete");
    let after = client
        .search(
            "users",
            SearchRequest {
                query: QueryNode::Term(TermQuery {
                    field: "email".into(),
                    value: FieldValue::String("a@x.com".into()),
                }),
                limit: 10,
                cursor: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .await
        .expect("search after delete");
    assert_eq!(after.total, 1);
    assert_eq!(after.hits[0].external_id, "u2");

    // -- drop_collection ----------------------------------------------------
    client
        .drop_collection("users")
        .await
        .expect("drop_collection");
    let listed = client.list_collections().await.expect("list_collections");
    assert!(listed.is_empty());
}
