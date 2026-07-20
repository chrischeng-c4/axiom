// SPEC-MANAGED: apps/tape/tech-design/semantic/source/apps-tape-tests-competitor-feature-parity-rs.md#unit-test
// <HANDWRITE gap="missing-generator:test:tape-competitor-feature-parity" tracker="#768" reason="Initial functional EC comparison for Tape against topic-log and topic-exchange competitors.">
use std::collections::HashSet;

use serde::Deserialize;
use serde_json::json;
use tape::TapeJournal;

#[derive(Debug, Deserialize)]
struct FeatureBaseline {
    schema_version: u32,
    reviewed_at: String,
    sources: Vec<BaselineSource>,
    rows: Vec<FeatureRow>,
}

#[derive(Debug, Deserialize)]
struct BaselineSource {
    id: String,
    product: String,
    version_scope: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct FeatureRow {
    name: String,
    kind: String,
    topic_routing: bool,
    ordered_append: bool,
    offset_replay: bool,
    time_replay: bool,
    consumer_checkpoint: bool,
    retention: bool,
    backfill: bool,
    replay_baseline: bool,
    source_ids: Vec<String>,
}

// <HANDWRITE gap="missing-generator:unit-test" tracker="#2159" reason="unit-test section in competitor_feature_parity.rs is hand-written pending codegen support">
fn baseline() -> FeatureBaseline {
    serde_json::from_str(include_str!("fixtures/competitor_feature_baseline.json"))
        .expect("versioned competitor feature baseline")
}

fn matrix() -> Vec<FeatureRow> {
    baseline().rows
}
// </HANDWRITE>

#[test]
fn competitor_baseline_has_versioned_official_provenance() {
    let baseline = baseline();
    assert_eq!(baseline.schema_version, 1);
    assert_eq!(baseline.reviewed_at, "2026-07-20");
    assert_eq!(baseline.rows.len(), 6);

    let source_ids = baseline
        .sources
        .iter()
        .map(|source| source.id.as_str())
        .collect::<HashSet<_>>();
    assert_eq!(source_ids.len(), baseline.sources.len());
    for source in &baseline.sources {
        assert!(!source.product.is_empty());
        assert!(!source.version_scope.is_empty());
        assert!(
            source.url.starts_with("https://kafka.apache.org/")
                || source.url.starts_with("https://docs.redpanda.com/")
                || source.url.starts_with("https://pulsar.apache.org/")
                || source.url.starts_with("https://docs.nats.io/")
                || source.url.starts_with("https://www.rabbitmq.com/"),
            "competitor provenance must use an official upstream URL: {}",
            source.url
        );
    }
    for row in &baseline.rows {
        assert!(!row.source_ids.is_empty(), "{} needs provenance", row.name);
        for source_id in &row.source_ids {
            assert!(
                source_ids.contains(source_id.as_str()),
                "{} references unknown source {source_id}",
                row.name
            );
        }
    }
}

#[test]
fn rabbitmq_topic_exchange_is_routing_peer_not_replay_baseline() {
    let row = matrix()
        .into_iter()
        .find(|row| row.name == "RabbitMQ topic exchange")
        .expect("RabbitMQ topic exchange comparison row exists");

    assert!(row.topic_routing, "topic exchange is a routing/fanout peer");
    assert_eq!(row.kind, "routing_only");
    assert!(
        !row.replay_baseline,
        "RabbitMQ topic exchange must not be treated as Tape's replay baseline"
    );
    assert!(!row.offset_replay);
    assert!(!row.time_replay);
    assert!(!row.consumer_checkpoint);
    assert!(!row.backfill);
}

#[test]
fn replay_log_peers_cover_the_tape_functional_contract() {
    for name in [
        "Kafka topic log",
        "Redpanda topic log",
        "Pulsar topic",
        "NATS JetStream stream",
        "RabbitMQ Streams",
    ] {
        let row = matrix()
            .into_iter()
            .find(|row| row.name == name)
            .unwrap_or_else(|| panic!("{name} comparison row exists"));
        assert_eq!(row.kind, "replay_log");
        assert!(row.replay_baseline, "{name} is a replay-log baseline");
        assert!(row.ordered_append, "{name} supports ordered append");
        assert!(row.offset_replay, "{name} supports offset replay");
        assert!(row.time_replay, "{name} supports time replay");
        assert!(row.consumer_checkpoint, "{name} supports consumer cursors");
        assert!(row.retention, "{name} supports retention windows");
        assert!(row.backfill, "{name} supports backfill/re-read workflows");
    }
}

#[test]
fn tape_only_claims_replay_feature_win_over_topic_exchange() {
    for row in matrix() {
        let tape_has_replay_advantage = row.name == "RabbitMQ topic exchange";
        if tape_has_replay_advantage {
            assert!(!row.replay_baseline);
            assert!(!row.offset_replay);
            assert!(!row.time_replay);
            assert!(!row.consumer_checkpoint);
        } else if row.name != "Tape" {
            assert!(
                row.replay_baseline,
                "Tape must not claim a blanket feature win over replay-log peer {}",
                row.name
            );
        }
    }
}

#[test]
fn tape_validates_replay_checkpoint_and_routing_key_functionality() {
    let mut journal = TapeJournal::default();
    let first = journal.append(
        "orders.created",
        Some("orders.us.created".into()),
        json!({"id": "o1", "region": "us"}),
        Some(1_000),
    );
    let second = journal.append(
        "orders.created",
        Some("orders.tw.created".into()),
        json!({"id": "o2", "region": "tw"}),
        Some(2_000),
    );

    assert_eq!(first.offset, 0);
    assert_eq!(second.offset, 1);
    assert_eq!(second.key.as_deref(), Some("orders.tw.created"));

    let offset_replay = journal.replay("orders.created", Some(1), None, None);
    assert_eq!(offset_replay, vec![second.clone()]);

    let time_replay = journal.replay("orders.created", None, Some(1_500), None);
    assert_eq!(time_replay, vec![second]);

    let checkpoint = journal
        .put_checkpoint("orders.created", "audit-worker", 2)
        .expect("checkpoint can advance to the topic end offset");
    assert_eq!(checkpoint.offset, 2);
    assert_eq!(
        journal
            .checkpoint("orders.created", "audit-worker")
            .expect("checkpoint is readable")
            .offset,
        2
    );
}
// </HANDWRITE>
