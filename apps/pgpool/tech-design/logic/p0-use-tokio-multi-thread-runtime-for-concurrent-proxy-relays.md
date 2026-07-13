---
id: '1618'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-multithread-runtime
entry: pgpool_cli
nodes:
  pgpool_cli: { kind: start, label: "pgpool CLI process" }
  runtime: { kind: process, label: "Tokio multi-thread runtime" }
  relays: { kind: process, label: "Concurrent frontend and backend relay tasks" }
  invariant: { kind: terminal, label: "Unchanged wire and reset isolation semantics" }
edges:
  - { from: pgpool_cli, to: runtime }
  - { from: runtime, to: relays }
  - { from: relays, to: invariant }
---
flowchart LR
  pgpool_cli([pgpool CLI]) --> runtime[Tokio multi-thread runtime]
  runtime --> relays[concurrent proxy relay tasks]
  relays --> invariant([unchanged wire and reset isolation])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/bin/pgpool.rs
    action: modify
    section: pgpool-multithread-runtime
    impl_mode: hand-written
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-multithread-runtime-verification
requirements:
  release_benchmark:
    id: R2
    text: "The unchanged corrected release benchmark completes all 64 clients without pgbench errors."
    kind: integration
    risk: high
    verify: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --pgpool-bin target/release/pgpool
  runtime_behavior:
    id: R1
    text: "The multi-thread runtime build preserves transaction-pool isolation and proxy behavior."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --lib --test pool_modes
---
flowchart TD
    r1[R1 runtime behavior] --> cargo_test_p_pgpool_lib_test_pool_modes[cargo test -p pgpool --lib --test pool_modes]
    r2[R2 release benchmark] --> apps_pgpool_benchmarks_pgbouncer_transaction_pooling_run_sh_pgpool_bin_target_release_pgpool[apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --pgpool-bin target/release/pgpool]
```
