---
id: real-kafka-calibrated-performance-benchmark-kraft-docker-mirrors
summary: >
  Real Kafka calibrated performance benchmark for apps/tape (WI #1482,
  epic #768), adding the first calibrated Kafka peer to the
  competitor-performance capability. Mirrors the existing
  apps/tape/tests/tape_vs_nats_jetstream.rs shape: spawns a real
  single-node Kafka broker in KRaft mode via `docker run
  apache/kafka:3.9.0`, publishes the same 20,000-event / 128-byte-payload
  backlog, replays it from the beginning with a real Kafka consumer
  (pure-Rust rskafka client), and compares against Tape's zero-copy
  replay_refs path via the existing tape::bench::external_replay_win /
  verify_external_replay_win helpers. Domain logic
  (append/replay/checkpoint/retention semantics) is untouched.
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-vs-kafka-flow
entry: route
nodes:
  route:
    kind: start
    label: "apps/tape gains a calibrated Kafka competitor-performance peer, mirroring tests/tape_vs_nats_jetstream.rs"
  spawn_kafka:
    kind: process
    label: "docker run apache/kafka:3.9.0 (KRaft single-node, no ZooKeeper) on a free local port; skip test gracefully if Docker is unavailable"
  publish_backlog:
    kind: process
    label: "publish a 20000-event / 128-byte-payload backlog to a Kafka topic via a real pure-Rust rskafka producer, same shape as the JetStream test"
  replay_kafka:
    kind: process
    label: "replay the full backlog from the beginning with a real rskafka consumer, measuring wall-clock replay latency"
  replay_tape:
    kind: process
    label: "TapeJournal::replay_refs replays the same backlog locally, measuring wall-clock replay latency (zero-copy path)"
  compare:
    kind: process
    label: "tape::bench::external_replay_win / verify_external_replay_win compares tape_replay_us vs kafka_replay_us against a calibrated required_ratio picked from a real measured run"
  no_baseline_change:
    kind: decision
    label: "bench.rs default_baseline() local peer list is NOT changed: Kafka stays uncalibrated_peer there by design, same as NATS JetStream today"
  done:
    kind: terminal
    label: "cargo test -p tape --test tape_vs_kafka passes with an honestly measured win ratio; cargo build/test -p tape stay green"
edges:
  - { from: route, to: spawn_kafka }
  - { from: spawn_kafka, to: publish_backlog }
  - { from: publish_backlog, to: replay_kafka }
  - { from: route, to: replay_tape }
  - { from: replay_kafka, to: compare }
  - { from: replay_tape, to: compare }
  - { from: compare, to: no_baseline_change }
  - { from: no_baseline_change, to: done }
---
flowchart TD
    route[apps/tape gains a calibrated Kafka peer] --> spawn_kafka[docker run apache/kafka:3.9.0 KRaft single-node]
    spawn_kafka --> publish_backlog[publish 20000-event/128B backlog via rskafka producer]
    publish_backlog --> replay_kafka[replay backlog via rskafka consumer, measure latency]
    route --> replay_tape[TapeJournal::replay_refs replays same backlog, measure latency]
    replay_kafka --> compare[external_replay_win compares ratio vs calibrated threshold]
    replay_tape --> compare
    compare --> no_baseline_change[bench.rs default_baseline unchanged: Kafka stays uncalibrated_peer]
    no_baseline_change --> done[test passes with honestly measured win ratio]
```
