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
fill_sections: []
---
