---
id: tape-competitor-feature-parity-topic-exchange-functional-ec
summary: Competitive feature parity functional correctness for Tape replay-log behavior, including RabbitMQ topic exchange as a routing-only comparison row.
fill_sections: [e2e-test]
---

# EC: Competitor Feature Parity Functional Behavior

Tape's functional competitor contract is replay-log first. Kafka topic logs,
Redpanda, Pulsar topics, NATS JetStream streams, and RabbitMQ Streams are replay
baselines. RabbitMQ topic exchange is still useful EC comparison input, but only
for topic routing/fanout semantics; it is not a replay journal baseline because
it does not provide offset/time replay, durable replay checkpoints, retention
windows, or historical backfill by itself.

The competitor classification is loaded from the committed
`tests/fixtures/competitor_feature_baseline.json` oracle reviewed on
2026-07-20. Every row cites version-scoped official Apache Kafka, Redpanda,
Apache Pulsar, NATS, or RabbitMQ documentation; the Rust test does not define
the competitor capability rows it asserts.

This EC intentionally validates functionality before performance. External
competitor throughput/latency baselines belong in the later competitor
performance contract, like Lumen's pg/OpenSearch split.

## External Contracts
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: tape-competitor-feature-parity-topic-exchange-functional
    capability_id: competitor-feature-parity
    claim_id: topic-replay-competitor-feature-matrix
    contract_id: topic-replay-vs-log-and-exchange-functional-correctness
    category: behavior
    test_path: tests/behavior_tape_claim_competitor_feature_topic_exchange_functional.rs
    command: "cargo test -p tape --test competitor_feature_parity -- --nocapture"
    assertions:
      - "Tape validates ordered append, offset replay, timestamp replay, and durable consumer checkpoints through the local journal core."
      - "The test consumes a versioned fixture whose every competitor row resolves to pinned official upstream provenance."
      - "That external oracle classifies Kafka, Redpanda, Pulsar, JetStream, and RabbitMQ Streams as replay-log baselines for feature parity."
      - "The same oracle classifies RabbitMQ topic exchange as topic-routing/fanout only, not Tape's offset/time replay baseline."
```
