// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-tests-wal_relay-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! #124 — lumen tails relay's broadcast as its WAL. Only with `--features
//! relay-wal`. Publishes records to an in-process relay broker and tails them
//! back through `RelayWal`.

#![cfg(feature = "relay-wal")]

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;

use lumen::log_entry::RaftLogEntry;
use lumen::storage::Engine;
use lumen::types::{
    Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    QueryNode, SearchRequest, TermQuery,
};
use lumen::wal::{WalLog, WalRecord};
use lumen::wal_relay::RelayWal;

async fn start_relay() -> String {
    let state = relay::AppState::new(relay::RelayServerConfig::ephemeral());
    let app = relay::router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    format!("http://{addr}")
}

fn users_schema() -> CreateCollectionRequest {
    let mut fields = BTreeMap::new();
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
    fields.insert(
        "bio".into(),
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

fn create_collection(coll: &str) -> RaftLogEntry {
    RaftLogEntry::CreateCollection {
        collection_id: coll.into(),
        req: users_schema(),
    }
}

fn index_one(coll: &str, eid: &str, field: &str, val: FieldValue) -> RaftLogEntry {
    RaftLogEntry::Index {
        collection_id: coll.into(),
        req: IndexRequest {
            items: vec![IndexItem {
                external_id: eid.into(),
                field: field.into(),
                value: val,
                version: None,
            }],
            request_id: None,
        },
    }
}

fn spawn_node(wal: Arc<RelayWal>) -> Arc<Engine> {
    let engine = Arc::new(Engine::new());
    let e = engine.clone();
    tokio::spawn(async move {
        let mut sub = match wal.subscribe(0).await {
            Ok(s) => s,
            Err(_) => return,
        };
        while let Some(item) = sub.next().await {
            if let Ok((_seq, rec)) = item {
                let _ = e.apply_raft_entry(rec.entry);
            }
        }
    });
    engine
}

async fn search_total(engine: &Engine, field: &str, value: &str) -> Option<u64> {
    engine
        .search(
            "users",
            SearchRequest {
                query: QueryNode::Term(TermQuery {
                    field: field.into(),
                    value: FieldValue::String(value.into()),
                }),
                limit: 10,
                cursor: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .ok()
        .map(|r| r.total)
}

async fn wait_for_total(engine: &Engine, field: &str, value: &str, expected: u64) -> bool {
    for _ in 0..100 {
        if search_total(engine, field, value).await == Some(expected) {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    false
}

/// @spec projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn relay_wal_publishes_then_tails() {
    let base = start_relay().await;

    let wal = RelayWal::new_with_ids(base, "lumen-wal", "tailer", "producer").unwrap();

    // Publish three records; the WAL seq is 1-based.
    for i in 0..3u64 {
        let seq = wal
            .publish(WalRecord::new(create_collection(&format!("c{i}"))))
            .await
            .unwrap();
        assert_eq!(seq, i + 1, "1-based WAL seq");
    }
    assert_eq!(wal.latest_seq().await.unwrap(), 3);

    // Tail from the start and fold the ordered log back.
    let mut stream = wal.subscribe(0).await.unwrap();
    let mut seqs = Vec::new();
    for _ in 0..3 {
        let (seq, rec) = tokio::time::timeout(Duration::from_secs(5), stream.next())
            .await
            .expect("stream item before timeout")
            .expect("stream not ended")
            .expect("ok item");
        assert!(matches!(rec.entry, RaftLogEntry::CreateCollection { .. }));
        seqs.push(seq);
    }
    assert_eq!(seqs, vec![1, 2, 3], "tailed in order");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn two_nodes_fan_out_from_one_published_stream() {
    let base = start_relay().await;
    let producer = RelayWal::new_with_ids(&base, "lumen-wal", "producer-sub", "producer").unwrap();
    let node_a = spawn_node(Arc::new(
        RelayWal::new_with_ids(&base, "lumen-wal", "node-a", "node-a-pub").unwrap(),
    ));
    let node_b = spawn_node(Arc::new(
        RelayWal::new_with_ids(&base, "lumen-wal", "node-b", "node-b-pub").unwrap(),
    ));

    for entry in [
        create_collection("users"),
        index_one("users", "u1", "email", FieldValue::String("a@x.com".into())),
        index_one("users", "u2", "email", FieldValue::String("b@x.com".into())),
        index_one("users", "u3", "email", FieldValue::String("a@x.com".into())),
    ] {
        producer
            .publish(WalRecord::new(entry))
            .await
            .expect("publish");
    }

    assert!(
        wait_for_total(&node_a, "email", "a@x.com", 2).await,
        "node A did not receive the full relay broadcast log"
    );
    assert!(
        wait_for_total(&node_b, "email", "a@x.com", 2).await,
        "node B did not receive the full relay broadcast log"
    );
    assert_eq!(search_total(&node_a, "email", "b@x.com").await, Some(1));
    assert_eq!(search_total(&node_b, "email", "b@x.com").await, Some(1));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn restarted_publisher_does_not_dedupe_new_writes() {
    let base = start_relay().await;
    let before_restart =
        RelayWal::new_with_ids(&base, "lumen-wal", "node-a", "publisher-before").unwrap();
    let after_restart =
        RelayWal::new_with_ids(&base, "lumen-wal", "node-a", "publisher-after").unwrap();

    assert_eq!(
        before_restart
            .publish(WalRecord::new(create_collection("before")))
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        after_restart
            .publish(WalRecord::new(create_collection("after")))
            .await
            .unwrap(),
        2,
        "a restarted process resets its counter but must use a distinct publisher id"
    );
    assert_eq!(after_restart.latest_seq().await.unwrap(), 2);

    let mut stream = after_restart.subscribe(0).await.unwrap();
    let mut collections = Vec::new();
    for _ in 0..2 {
        let (_seq, rec) = tokio::time::timeout(Duration::from_secs(5), stream.next())
            .await
            .expect("stream item before timeout")
            .expect("stream not ended")
            .expect("ok item");
        if let RaftLogEntry::CreateCollection { collection_id, .. } = rec.entry {
            collections.push(collection_id);
        }
    }
    assert_eq!(collections, vec!["before", "after"]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn subscriber_can_reconnect_from_last_applied_seq() {
    let base = start_relay().await;
    let wal = RelayWal::new_with_ids(base, "lumen-wal", "node-a", "producer").unwrap();

    wal.publish(WalRecord::new(create_collection("first")))
        .await
        .unwrap();
    let mut first_stream = wal.subscribe(0).await.unwrap();
    let (seq, _) = tokio::time::timeout(Duration::from_secs(5), first_stream.next())
        .await
        .expect("first item before timeout")
        .expect("stream not ended")
        .expect("ok item");
    assert_eq!(seq, 1);
    drop(first_stream);

    wal.publish(WalRecord::new(create_collection("second")))
        .await
        .unwrap();
    let mut second_stream = wal.subscribe(seq).await.unwrap();
    let (next_seq, rec) = tokio::time::timeout(Duration::from_secs(5), second_stream.next())
        .await
        .expect("second item before timeout")
        .expect("stream not ended")
        .expect("ok item");
    assert_eq!(next_seq, 2);
    assert!(matches!(
        rec.entry,
        RaftLogEntry::CreateCollection { collection_id, .. } if collection_id == "second"
    ));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn invalid_relay_payload_is_reported_not_silently_skipped() {
    let base = start_relay().await;
    let wal = RelayWal::new_with_ids(&base, "lumen-wal", "node-a", "producer").unwrap();
    let client = reqwest::Client::builder()
        .http2_prior_knowledge()
        .build()
        .unwrap();
    client
        .post(format!("{base}/v1/lumen-wal/publish"))
        .json(&serde_json::json!({
            "message_id": "invalid-wal-payload",
            "payload": { "not": "a WalRecord" }
        }))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let mut stream = wal.subscribe(0).await.unwrap();
    let err = tokio::time::timeout(Duration::from_secs(5), stream.next())
        .await
        .expect("stream item before timeout")
        .expect("stream not ended")
        .expect_err("payload decode should fail");
    assert!(
        err.to_string().contains("not a WalRecord"),
        "unexpected error: {err}"
    );
}
// CODEGEN-END
