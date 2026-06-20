// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! End-to-end fan-out proof against a real NATS JetStream server.
//!
//! Spin one up with: `nats-server -js` (the project's standard test
//! broker; `brew services start nats-server` also works). The test
//! skips gracefully if no server is reachable, per the repo's
//! "real services, skip if unavailable" pattern.
//!
//! What it proves: publish a mutation stream once; two INDEPENDENT
//! consumers (simulating two serving nodes) each tail the full stream
//! and build identical indexes. That is fan-out — the property GCP
//! Pub/Sub can't give cheaply and the whole data-plane design rests on.

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use std::collections::BTreeMap;

use lumen::log_entry::RaftLogEntry;
use lumen::storage::Engine;
use lumen::types::{
    Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    QueryNode, SearchRequest, TermQuery,
};
use lumen::wal::{WalLog, WalRecord};
use lumen::wal_nats::{NatsWal, NatsWalConfig};

fn nats_url() -> String {
    std::env::var("LUMEN_TEST_NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".into())
}

fn test_config(name: &str) -> NatsWalConfig {
    NatsWalConfig::new(format!("lumen_wal_{name}"), format!("lumen.wal.{name}"))
        .expect("valid test NATS WAL config")
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

fn index_one(coll: &str, eid: &str, field: &str, val: FieldValue) -> RaftLogEntry {
    RaftLogEntry::Index {
        collection_id: coll.into(),
        req: IndexRequest {
            items: vec![IndexItem {
                external_id: eid.into(),
                field: field.into(),
                value: val,
            }],
            request_id: None,
        },
    }
}

/// Probe NATS + JetStream and reset the WAL stream for a clean run.
/// Returns `None` (→ test skips) if no server is reachable OR the
/// server doesn't have JetStream enabled (e.g. a plain `nats-server`
/// without `-js`).
async fn reset_stream(config: &NatsWalConfig) -> Option<()> {
    let client = async_nats::connect(&nats_url()).await.ok()?;
    let js = async_nats::jetstream::new(client);
    // get_or_create_stream fails when JetStream isn't enabled (a plain
    // `nats-server` without `-js`) — `.ok()?` turns that into a skip
    // rather than a failure. Doubles as the clean-slate reset via purge.
    let stream = js
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: config.stream_name.clone(),
            subjects: vec![config.subject.clone()],
            ..Default::default()
        })
        .await
        .ok()?;
    let _ = stream.purge().await;
    Some(())
}

/// Spawn an apply loop: a fresh Engine that folds the full WAL stream.
fn spawn_node(wal: Arc<NatsWal>) -> Arc<Engine> {
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

/// Poll until `engine` reports `expected` hits for the query, or time out.
async fn wait_for_total(engine: &Engine, field: &str, value: &str, expected: u64) -> bool {
    for _ in 0..100 {
        if search_total(engine, field, value).await == Some(expected) {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    false
}

#[tokio::test]
async fn two_nodes_fan_out_from_one_published_stream() {
    let config = test_config("fan_out");
    // Skip if no NATS reachable.
    if reset_stream(&config).await.is_none() {
        eprintln!(
            "skipping: no NATS server at {} (run nats-server -js)",
            nats_url()
        );
        return;
    }

    // Producer side.
    let producer = NatsWal::connect_with_config(&nats_url(), config.clone())
        .await
        .expect("connect producer");

    // Two independent "serving nodes", each its own consumer + engine.
    let node_a = spawn_node(Arc::new(
        NatsWal::connect_with_config(&nats_url(), config.clone())
            .await
            .expect("connect node A"),
    ));
    let node_b = spawn_node(Arc::new(
        NatsWal::connect_with_config(&nats_url(), config.clone())
            .await
            .expect("connect node B"),
    ));

    // Publish the mutation stream ONCE.
    let entries = vec![
        RaftLogEntry::CreateCollection {
            collection_id: "users".into(),
            req: users_schema(),
        },
        index_one("users", "u1", "email", FieldValue::String("a@x.com".into())),
        index_one("users", "u2", "email", FieldValue::String("b@x.com".into())),
        index_one("users", "u3", "email", FieldValue::String("a@x.com".into())),
        index_one(
            "users",
            "u1",
            "bio",
            FieldValue::String("rust engineer".into()),
        ),
    ];
    for e in &entries {
        producer
            .publish(WalRecord::new(e.clone()))
            .await
            .expect("publish");
    }

    // BOTH nodes must independently converge to the same index.
    assert!(
        wait_for_total(&node_a, "email", "a@x.com", 2).await,
        "node A did not converge (email=a@x.com → 2)"
    );
    assert!(
        wait_for_total(&node_b, "email", "a@x.com", 2).await,
        "node B did not converge (email=a@x.com → 2)"
    );

    // Cross-check another query on both — identical results = fan-out.
    assert_eq!(search_total(&node_a, "email", "b@y.com").await, Some(0));
    assert_eq!(search_total(&node_b, "email", "b@x.com").await, Some(1));
}

#[tokio::test]
async fn late_node_replays_backlog_then_sees_live() {
    let config = test_config("late_replay");
    if reset_stream(&config).await.is_none() {
        eprintln!("skipping: no NATS server at {}", nats_url());
        return;
    }
    let producer = NatsWal::connect_with_config(&nats_url(), config.clone())
        .await
        .expect("connect");

    // Publish a backlog BEFORE the node subscribes.
    producer
        .publish(WalRecord::new(RaftLogEntry::CreateCollection {
            collection_id: "users".into(),
            req: users_schema(),
        }))
        .await
        .unwrap();
    producer
        .publish(WalRecord::new(index_one(
            "users",
            "old",
            "email",
            FieldValue::String("a@x.com".into()),
        )))
        .await
        .unwrap();

    // Node starts late — must replay the backlog from seq 0.
    let node = spawn_node(Arc::new(
        NatsWal::connect_with_config(&nats_url(), config.clone())
            .await
            .unwrap(),
    ));
    assert!(
        wait_for_total(&node, "email", "a@x.com", 1).await,
        "late node did not replay backlog"
    );

    // Then a live append must reach it too.
    producer
        .publish(WalRecord::new(index_one(
            "users",
            "new",
            "email",
            FieldValue::String("a@x.com".into()),
        )))
        .await
        .unwrap();
    assert!(
        wait_for_total(&node, "email", "a@x.com", 2).await,
        "late node did not see live append"
    );
}
// CODEGEN-END
