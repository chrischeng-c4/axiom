// HANDWRITE-BEGIN gap="missing-generator:unit-test:034badb7" tracker="pending-tracker" reason="Integration test (feature relay-wal): publish WalRecords to an in-process relay and tail them back through RelayWal, asserting in-order delivery."
//! #124 — lumen tails relay's broadcast as its WAL. Only with `--features
//! relay-wal`. Publishes records to an in-process relay broker and tails them
//! back through `RelayWal`.

#![cfg(feature = "relay-wal")]

use std::collections::BTreeMap;
use std::time::Duration;

use futures::StreamExt;

use lumen::log_entry::RaftLogEntry;
use lumen::types::CreateCollectionRequest;
use lumen::wal::{WalLog, WalRecord};
use lumen::wal_relay::RelayWal;

fn entry(coll: &str) -> RaftLogEntry {
    RaftLogEntry::CreateCollection {
        collection_id: coll.into(),
        req: CreateCollectionRequest {
            fields: BTreeMap::new(),
        },
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn relay_wal_publishes_then_tails() {
    // An in-process relay broker.
    let state = relay::AppState::new(relay::RelayServerConfig::ephemeral());
    let app = relay::router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    let wal = RelayWal::new(format!("http://{addr}"), "lumen-wal").unwrap();

    // Publish three records; the WAL seq is 1-based.
    for i in 0..3u64 {
        let seq = wal
            .publish(WalRecord::new(entry(&format!("c{i}"))))
            .await
            .unwrap();
        assert_eq!(seq, i + 1, "1-based WAL seq");
    }

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
// HANDWRITE-END
