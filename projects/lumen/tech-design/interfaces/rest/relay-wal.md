---
id: lumen-relay-wal
summary: lumen tails relay's broadcast as its WAL backend (#124) — a derived index, not a source of truth, so it needs no consensus; it folds an ordered broker log into its index. Add a RelayWal WalLog backend (publish via relay, subscribe via relay's broadcast, reusing relay::wire), wired as `--wal relay`, behind a `relay-wal` feature so the default serving binary stays HTTP-client-free. Plaintext h2c (no TLS) preserves the clean cross-compile.
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
  pub: { kind: process, label: "RelayWal.publish: POST relay /v1/{subject}/publish with payload=json(WalRecord); return AppendOutcome.seq + 1 (1-based WAL seq)" }
  tail: { kind: process, label: "each pod RelayWal.subscribe(from): GET relay /v1/{subject}/subscribe?from_seq; decode length-prefixed CBOR LogEntry frames (relay::wire::decode_frames)" }
  fold: { kind: process, label: "map each LogEntry -> (seq+1, WalRecord from payload); the fold loop applies it to the local index (apply_raft_entry)" }
  done: { kind: terminal, label: "every pod tails relay's ordered, raftcore-HA log -> derived index converges; lossable + rebuildable, no lumen-side consensus" }
edges:
  - { from: write, to: pub }
  - { from: pub, to: tail }
  - { from: tail, to: fold }
  - { from: fold, to: done }
---
flowchart TD
    write([publish WalRecord]) --> pub[RelayWal.publish -> relay]
    pub --> tail[subscribe relay broadcast; decode frames]
    tail --> fold[map -> WalRecord; fold into index]
    fold --> done([pods converge; no consensus])
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
edges:
  - { from: suite, to: t_pub }
  - { from: t_pub, to: a_pub }
  - { from: suite, to: t_tail }
  - { from: t_tail, to: a_tail }
---
flowchart TD
    suite([in-process relay]) --> t_pub[publish 3] --> a_pub([seqs 1,2,3])
    suite --> t_tail[subscribe + tail] --> a_tail([3 records in order])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/wal_relay.rs
    action: create
    section: logic
    impl_mode: hand-written
    reason: "RelayWal: a WalLog backed by relay's broadcast. publish POSTs to relay /v1/{subject}/publish (payload=json(WalRecord)); subscribe GETs /v1/{subject}/subscribe and decodes relay's length-prefixed CBOR LogEntry frames (relay::wire::decode_frames), mapping each to (seq+1, WalRecord). Plaintext h2c, no TLS."
  - path: projects/lumen/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Declare the feature-gated module: #[cfg(feature = \"relay-wal\")] pub mod wal_relay;"
  - path: projects/lumen/src/bin/lumen.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Add WalBackend::Relay (feature-gated) + --relay-url/--relay-subject args + a match arm constructing RelayWal."
  - path: projects/lumen/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Optional relay (path) + reqwest deps; relay-wal feature = [dep:relay, dep:reqwest] (off by default to keep the serving binary HTTP-client-free)."
  - path: projects/lumen/tests/wal_relay.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "Integration test (feature relay-wal): publish WalRecords to an in-process relay and tail them back through RelayWal, asserting in-order delivery."
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] RelayWal (WalLog over relay broadcast): publish->relay /publish (seq+1), subscribe->decode_frames(LogEntry)->WalRecord; lumen folds the ordered log (derived index, no consensus). Plaintext h2c, feature-gated. Sound.
- [unit-test] in-process relay round-trip (publish 3, tail in order).
- [changes] wal_relay.rs + lib mod + bin wiring + Cargo feature + test.
