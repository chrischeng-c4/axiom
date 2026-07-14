---
id: '1637'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-vectored-buffered-relay
entry: read
nodes:
  read: { kind: start, label: "Read first complete validated backend frame" }
  drain: { kind: process, label: "Drain only complete buffered frames until ReadyForQuery or incomplete suffix" }
  segments: { kind: process, label: "Retain ordered Bytes segments without concatenation" }
  writev: { kind: process, label: "writev segments; advance partial write cursor" }
  ready: { kind: process, label: "Apply ReadyForQuery status after all prior bytes are sent" }
  prefix: { kind: process, label: "Forward valid prefix once then terminate on malformed suffix" }
  wait: { kind: terminal, label: "Await next backend frame" }
edges:
  - { from: read, to: drain }
  - { from: drain, to: segments, label: "complete frames" }
  - { from: segments, to: writev }
  - { from: writev, to: ready, label: "batch contains ReadyForQuery" }
  - { from: writev, to: wait, label: "no ReadyForQuery" }
  - { from: drain, to: prefix, label: "malformed suffix after valid prefix" }
---
flowchart LR
  read([first validated frame]) --> drain[drain complete buffered frames]
  drain --> segments[ordered Bytes segments\nno concatenation]
  segments --> writev[writev + partial cursor]
  writev -->|ReadyForQuery| ready[apply ownership status]
  writev -->|no Ready| wait([await next frame])
  drain -->|malformed suffix| prefix[forward valid prefix then end]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/proxy/relay.rs
    action: modify
    section: pgpool-vectored-buffered-relay
    impl_mode: hand-written
  - path: apps/pgpool/src/pool/transaction.rs
    action: modify
    section: pgpool-vectored-buffered-relay
    impl_mode: hand-written
  - path: apps/pgpool/tests/pool_modes.rs
    action: modify
    section: pgpool-vectored-buffered-relay
    impl_mode: hand-written
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-vectored-buffered-relay-verification
requirements:
  release_comparison:
    id: R4
    text: "The immutable PgBouncer comparison retains all 64 clients and no pgbench client errors; meter sampling is diagnostic only."
    kind: e2e
    risk: high
    verify: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh
  transaction_isolation:
    id: R3
    text: "Transaction lease boundaries, pipelined frontend safety, reset isolation, and the backend capacity cap remain unchanged."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test pool --test pool_modes
  vectored_relay:
    id: R1
    text: "A multi-frame backend batch retains exact ordered validated segments and forwards them losslessly, including a partial vectored write, without concatenating into BytesMut."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --lib proxy::relay::tests
---
flowchart TD
    r1[R1 vectored relay] --> cargo_test_p_pgpool_lib_proxy_relay_tests[cargo test -p pgpool --lib proxy::relay::tests]
    r3[R3 transaction isolation] --> cargo_test_p_pgpool_test_pool_test_pool_modes[cargo test -p pgpool --test pool --test pool_modes]
    r4[R4 release comparison] --> apps_pgpool_benchmarks_pgbouncer_transaction_pooling_run_sh[apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh]
```
