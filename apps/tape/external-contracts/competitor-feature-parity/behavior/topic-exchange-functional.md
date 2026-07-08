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
      - "Kafka, Redpanda, Pulsar, JetStream, and RabbitMQ Streams are classified as replay-log baselines for feature parity."
      - "RabbitMQ topic exchange is included as a topic-routing/fanout comparison row, but is not treated as Tape's offset/time replay baseline."
```
