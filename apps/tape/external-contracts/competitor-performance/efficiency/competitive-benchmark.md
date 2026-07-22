---
id: tape-competitor-performance-ec
summary: Competitive performance — local Tape replay regression gate plus real NATS JetStream and Kafka replay wins; other replay-log broker wins stay unclaimed until their own calibration gates exist.
fill_sections: [e2e-test]
---

# EC: Competitive Performance

Tape's competitive-performance gate mirrors Lumen's split between a fast local
regression gate and explicit peer calibration. The default local gate runs
Tape's bounded pull/replay and explicit checkpoint-ack path for exactly 1,000
events with 128-byte payloads. Its independent oracle requires append p95 at
or below 5,000 us, full replay at or below 50,000 us, and checkpoint p95 at or
below 5,000 us. The NATS win gate starts real Tape h2c
and `nats-server -js` services, publishes a 20,000-event, 128-byte-payload
durable backlog, takes five complete replay samples across both network
clients, and requires Tape's compact stream p50 to be at least 1.5x faster. It
also reports throughput, p50/p95/p99, child CPU/RSS, durable bytes and disk
amplification, and errors for both services; it fails closed when the real
JetStream prerequisite cannot be started. The
Kafka win gate starts real Tape h2c and single-node Kafka KRaft services (`docker run
apache/kafka:3.9.0`, no ZooKeeper), publishes the same 20,000-event,
128-byte-payload backlog, replays it with a real `rskafka` consumer, and
requires Tape's compact stream to be at least 1.5x faster. The latest
2026-07-18 release calibration measured 2.02x p50 against NATS and 4.07x
against Kafka; the earlier in-process/network asymmetric ratios are invalid.
The Kafka production gate fails closed when Docker, the pinned image, or a
usable broker port is unavailable. Redpanda, Pulsar,
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
      - "The oracle runs exactly 1,000 events with 128-byte payloads and independently requires append p95 <= 5,000 us, full replay <= 50,000 us, and checkpoint p95 <= 5,000 us."
      - "The test computes those limits from observed report fields and fixed EC constants; it does not call Tape's default_baseline or verify_report verdict helpers."
      - "Redpanda, Pulsar, and RabbitMQ Streams performance wins remain unclaimed without mandatory calibrated real-service peer runs; RabbitMQ topic exchange remains routing-only."

  - id: tape-competitor-performance-nats-jetstream-replay-win
    capability_id: competitor-performance
    claim_id: topic-replay-competitor-performance-baseline
    contract_id: topic-replay-nats-jetstream-local-backlog-win
    category: efficiency
    test_path: tests/behavior_tape_claim_competitor_performance_nats_jetstream.rs
    command: "cargo test --release -p tape --test tape_vs_nats_jetstream -- --ignored --nocapture"
    assertions:
      - "The release-only gate must start a real local nats-server with JetStream enabled and fail closed when that prerequisite cannot be started."
      - "Tape and JetStream replay five samples of the same 20,000-event, 128-byte-payload durable backlog from the beginning."
      - "The EC test independently computes nats_p50_us / max(tape_p50_us, 1) and requires the ratio to be >= 1.5 without trusting Tape's external_replay_win or verify_external_replay_win helpers."
      - "The report includes throughput, p50/p95/p99, child-process CPU and RSS, durable bytes and disk amplification, and errors for both services."

  - id: tape-competitor-performance-kafka-replay-win
    capability_id: competitor-performance
    claim_id: topic-replay-competitor-performance-baseline
    contract_id: topic-replay-kafka-local-backlog-win
    category: efficiency
    test_path: tests/efficiency_tape_claim_competitor_performance_kafka_replay_win.rs
    command: "cargo test --release -p tape --test tape_vs_kafka -- --ignored --nocapture"
    assertions:
      - "The release-only production gate must start apache/kafka:3.9.0 in single-node KRaft mode and fail closed if Docker, the pinned image, or a usable broker port is unavailable."
      - "Tape and Kafka replay the same 20,000-event, 128-byte-payload durable backlog from the beginning through real h2c and rskafka clients."
      - "The EC test independently computes kafka_replay_us / max(tape_replay_us, 1) and requires the ratio to be >= 1.5 without trusting Tape's external_replay_win or verify_external_replay_win helpers."
```
