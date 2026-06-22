---
id: lumen-relay-wal
summary: lumen can explicitly tail relay's broadcast as a broker-backed WAL backend (#124). RelayWal publishes compact `WalRecord::encode()` payload envelopes through relay, subscribes with a per-pod subscriber id, uses process-unique message ids to avoid restart dedupe collisions, and reads relay `/len` for latest_seq. The backend is wired as `--wal relay` behind the `relay-wal` feature and uses plaintext h2c. This is the explicit external-broker mode; Lumen primary/replica is the multi-pod auto direction.
capability_refs:
  - id: "long-running-stability"
    role: primary
    gap: "log-fan-out-rebuild-from-log"
    claim: "log-fan-out-rebuild-from-log"
    coverage: full
    rationale: "RelayWal is the feature-gated broker-log backend for lumen's log fan-out and rebuild-from-log capability."
fill_sections: [logic, unit-test, changes]
---

# lumen relay broadcast WAL (#124)

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-relay-wal-flow
entry: write
nodes:
  write: { kind: start, label: "WriteCoordinator publishes a WalRecord to the WAL backend" }
  pub: { kind: process, label: "RelayWal.publish: CBOR POST relay /v1/{subject}/publish with payload=versioned WalRecord::encode() envelope and publisher-unique message_id; return AppendOutcome.seq + 1 (1-based WAL seq)" }
  tail: { kind: process, label: "each pod RelayWal.subscribe(from): GET relay /v1/{subject}/subscribe?from_seq&subscriber_id={pod}; decode length-prefixed CBOR LogEntry frames (relay::wire::decode_frames)" }
  fold: { kind: process, label: "map each LogEntry -> (seq+1, WalRecord decoded from the payload envelope or legacy JSON); the fold loop applies it to the local index (apply_raft_entry)" }
  len: { kind: process, label: "RelayWal.latest_seq: GET relay /v1/{subject}/len and use latest_seq as Lumen WAL position" }
  done: { kind: terminal, label: "explicit broker mode: every pod tails relay's ordered broker log -> derived index converges" }
edges:
  - { from: write, to: pub }
  - { from: pub, to: tail }
  - { from: tail, to: fold }
  - { from: tail, to: len }
  - { from: fold, to: done }
---
flowchart TD
    write([publish WalRecord]) --> pub[RelayWal.publish -> relay]
    pub --> tail[subscribe relay broadcast; decode frames]
    tail --> fold[map -> WalRecord; fold into index]
    tail --> len[latest_seq via /len]
    fold --> done([explicit broker mode converges])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-relay-wal-test
entry: suite
nodes:
  suite: { kind: start, label: "RelayWal against an in-process relay broker (feature relay-wal)" }
  t_pub: { kind: process, label: "publish 3 WalRecords through RelayWal" }
  a_pub: { kind: terminal, label: "assert 1-based seqs 1,2,3 returned by relay" }
  t_tail: { kind: process, label: "subscribe(0) and tail" }
  a_tail: { kind: terminal, label: "assert the 3 records come back in order, decoded from relay's broadcast" }
  t_fanout: { kind: process, label: "two independent subscriber ids tail one published stream into two engines" }
  a_fanout: { kind: terminal, label: "assert both engines converge to the same search results" }
  t_restart: { kind: process, label: "construct a second RelayWal with a reset counter but different publisher id" }
  a_restart: { kind: terminal, label: "assert new writes append seq 2 instead of deduping into seq 1" }
  t_reconnect: { kind: process, label: "drop a stream and subscribe again from last applied seq" }
  a_reconnect: { kind: terminal, label: "assert the next stream resumes at the next WAL seq" }
edges:
  - { from: suite, to: t_pub }
  - { from: t_pub, to: a_pub }
  - { from: suite, to: t_tail }
  - { from: t_tail, to: a_tail }
  - { from: suite, to: t_fanout }
  - { from: t_fanout, to: a_fanout }
  - { from: suite, to: t_restart }
  - { from: t_restart, to: a_restart }
  - { from: suite, to: t_reconnect }
  - { from: t_reconnect, to: a_reconnect }
---
flowchart TD
    suite([in-process relay]) --> t_pub[publish 3] --> a_pub([seqs 1,2,3])
    suite --> t_tail[subscribe + tail] --> a_tail([3 records in order])
    suite --> t_fanout[two subscribers] --> a_fanout([both nodes converge])
    suite --> t_restart[restart publisher] --> a_restart([no dedupe collision])
    suite --> t_reconnect[reconnect from seq] --> a_reconnect([next seq delivered])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/wal_relay.rs
    action: create
    section: logic
    impl_mode: hand-written
    reason: "RelayWal: a WalLog backed by relay's broadcast. publish CBOR POSTs to relay /v1/{subject}/publish (payload=versioned WalRecord::encode() envelope, publisher-unique message_id); subscribe GETs /v1/{subject}/subscribe with a per-pod subscriber_id and decodes relay's length-prefixed CBOR LogEntry frames, mapping each to (seq+1, WalRecord); latest_seq reads relay /len."
  - path: projects/lumen/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Declare the feature-gated module: #[cfg(feature = \"relay-wal\")] pub mod wal_relay;"
  - path: projects/lumen/src/bin/lumen.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Add WalBackend::Relay (feature-gated) + --relay-url/--relay-subject/--relay-subscriber-id args + a match arm constructing RelayWal."
  - path: projects/lumen/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Optional relay (path) + reqwest deps; relay-wal feature = [dep:relay, dep:reqwest]."
  - path: projects/lumen/tests/wal_relay.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "Integration tests (feature relay-wal): publish/tail, latest_seq, two-node fan-out, restart dedupe safety, reconnect from last seq, and invalid payload reporting against an in-process relay."
  - path: projects/relay/src/server.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Expose GET /v1/{subject}/len so RelayWal.latest_seq has a concrete broker source."
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] RelayWal (WalLog over relay broadcast): publish->relay /publish (seq+1), subscribe->decode_frames(LogEntry)->WalRecord with per-pod subscriber ids, latest_seq via relay /len; lumen folds the ordered broker log in explicit Relay mode. Plaintext h2c, feature-gated. Sound.
- [unit-test] in-process relay round-trip plus fan-out, restart dedupe, reconnect, and invalid payload coverage.
- [changes] wal_relay.rs + lib mod + bin wiring + Cargo feature + relay /len + test.
