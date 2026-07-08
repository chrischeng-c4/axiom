---
id: tape-competitor-performance-ec
summary: Competitive performance — local Tape replay regression gate plus a real NATS JetStream replay win; other replay-log broker wins stay unclaimed until their own calibration gates exist.
fill_sections: [e2e-test]
---

# EC: Competitive Performance

Tape's competitive-performance gate mirrors Lumen's split between a fast local
regression gate and explicit peer calibration. The default local gate runs
Tape's append/replay/checkpoint path and verifies that it does not regress
against conservative local budgets. The external win gate starts a real
`nats-server -js`, publishes a 20,000-event, 128-byte-payload backlog into
JetStream, replays the backlog, and requires Tape's zero-copy `replay_refs`
path to replay the same local backlog at least 1.5x faster. Kafka, Redpanda,
Pulsar, and RabbitMQ Streams remain unclaimed until their own real-service
calibration gates exist. RabbitMQ topic exchange is recorded as routing-only,
not a replay performance baseline.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: tape-competitor-performance-local-regression-and-calibration-ledger
    capability_id: competitor-performance
    claim_id: topic-replay-competitor-performance-baseline
    contract_id: topic-replay-local-performance-and-peer-calibration
    category: efficiency
    test_path: projects/tape/tests/behavior_tape_claim_competitor_performance.rs
    command: "cargo test -p tape --test tape_perf_gate -- --nocapture"
    assertions:
      - "Tape's local append/replay/checkpoint benchmark stays inside conservative regression budgets."
      - "Kafka, Redpanda, Pulsar, and RabbitMQ Streams performance wins are not claimed without calibrated real-service peer runs."
      - "RabbitMQ topic exchange remains routing-only and is not treated as a replay performance baseline."

  - id: tape-competitor-performance-nats-jetstream-replay-win
    capability_id: competitor-performance
    claim_id: topic-replay-competitor-performance-baseline
    contract_id: topic-replay-nats-jetstream-local-backlog-win
    category: efficiency
    test_path: projects/tape/tests/behavior_tape_claim_competitor_performance_nats_jetstream.rs
    command: "cargo test -p tape --test tape_vs_nats_jetstream -- --nocapture"
    assertions:
      - "The test starts a real local nats-server with JetStream enabled."
      - "Tape and JetStream replay the same 20,000-event, 128-byte-payload backlog workload from the beginning."
      - "Tape's zero-copy full-replay latency is at least 1.5x faster than NATS JetStream for the local backlog replay workload."
```
