---
id: tape-competitor-performance-ec
summary: Competitive performance — local Tape replay regression gate plus real NATS JetStream and Kafka replay wins; other replay-log broker wins stay unclaimed until their own calibration gates exist.
fill_sections: [e2e-test]
---

# EC: Competitive Performance

Tape's competitive-performance gate mirrors Lumen's split between a fast local
regression gate and explicit peer calibration. The default local gate runs
Tape's bounded pull/replay and explicit checkpoint-ack path and verifies that it does not regress
against conservative local budgets. The NATS win gate starts real Tape h2c
and `nats-server -js` services, publishes a 20,000-event, 128-byte-payload
durable backlog, takes five complete replay samples across both network
clients, and requires Tape's compact stream p50 to be at least 1.5x faster. It
also reports throughput, p50/p95/p99, child CPU/RSS, durable bytes and disk
amplification, and errors for both services. The
Kafka win gate starts real Tape h2c and single-node Kafka KRaft services (`docker run
apache/kafka:3.9.0`, no ZooKeeper), publishes the same 20,000-event,
128-byte-payload backlog, replays it with a real `rskafka` consumer, and
requires Tape's compact stream to be at least 1.5x faster. The latest
2026-07-18 release calibration measured 2.02x p50 against NATS and 4.07x
against Kafka; the earlier in-process/network asymmetric ratios are invalid. The Kafka
test skips gracefully when Docker is unavailable. Redpanda, Pulsar,
and RabbitMQ Streams remain unclaimed until their own real-service calibration
gates exist. RabbitMQ topic exchange is recorded as routing-only, not a replay
performance baseline.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: tape-competitor-performance-local-regression-and-calibration-ledger
    capability_id: competitor-performance
    claim_id: topic-replay-competitor-performance-baseline
    contract_id: topic-replay-local-performance-and-peer-calibration
    category: efficiency
    test_path: tests/behavior_tape_claim_competitor_performance.rs
    command: "cargo test -p tape --test tape_perf_gate -- --nocapture"
    assertions:
      - "Tape's local bounded pull/replay and explicit checkpoint-ack benchmark stays inside conservative regression budgets."
      - "Redpanda, Pulsar, and RabbitMQ Streams performance wins are not claimed without calibrated real-service peer runs."
      - "RabbitMQ topic exchange remains routing-only and is not treated as a replay performance baseline."

  - id: tape-competitor-performance-nats-jetstream-replay-win
    capability_id: competitor-performance
    claim_id: topic-replay-competitor-performance-baseline
    contract_id: topic-replay-nats-jetstream-local-backlog-win
    category: efficiency
    test_path: tests/behavior_tape_claim_competitor_performance_nats_jetstream.rs
    command: "cargo test --release -p tape --test tape_vs_nats_jetstream -- --nocapture"
    assertions:
      - "The test starts a real local nats-server with JetStream enabled."
      - "Tape and JetStream replay the same 20,000-event, 128-byte-payload backlog workload from the beginning."
      - "Tape's real h2c replay-stream latency is at least 1.5x faster than NATS JetStream for the symmetric local backlog workload."
      - "The five-sample report includes throughput, p50/p95/p99, child-process CPU and RSS, durable bytes and disk amplification, and errors for both services."

  - id: tape-competitor-performance-kafka-replay-win
    capability_id: competitor-performance
    claim_id: topic-replay-competitor-performance-baseline
    contract_id: topic-replay-kafka-local-backlog-win
    category: efficiency
    test_path: tests/efficiency_tape_claim_competitor_performance_kafka_replay_win.rs
    command: "cargo test --release -p tape --test tape_vs_kafka -- --nocapture"
    assertions:
      - "The test starts a real single-node apache/kafka:3.9.0 broker in KRaft mode (no ZooKeeper) via docker, skipping gracefully when Docker is unavailable."
      - "Tape and Kafka replay the same 20,000-event, 128-byte-payload backlog workload from the beginning, using a real rskafka consumer for the Kafka side."
      - "Tape's real h2c replay-stream latency is at least 1.5x faster than Kafka for the symmetric local backlog workload."
```
