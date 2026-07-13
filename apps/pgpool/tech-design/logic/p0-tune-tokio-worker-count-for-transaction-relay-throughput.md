---
id: '1622'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-worker-count-tuning
entry: pgpool
nodes:
  pgpool: { kind: start, label: "pgpool multi-thread CLI runtime" }
  workers: { kind: process, label: "Bounded worker count for 64 client and 16 backend relay topology" }
  relay: { kind: process, label: "Existing concurrent wire relay and pool state machine" }
  benchmark: { kind: terminal, label: "Repeated complete no-error release comparison" }
edges:
  - { from: pgpool, to: workers }
  - { from: workers, to: relay }
  - { from: relay, to: benchmark }
---
flowchart LR
  pgpool([pgpool CLI]) --> workers[bounded Tokio worker count]
  workers --> relay[existing relay and reset semantics]
  relay --> benchmark([repeat complete benchmark])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/bin/pgpool.rs
    action: modify
    section: pgpool-worker-count-tuning
    impl_mode: hand-written
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-worker-count-tuning-verification
requirements:
  comparison:
    id: R2
    text: "Unchanged release comparisons complete all 64 clients without pgbench errors and retain every result for stable-win analysis."
    kind: integration
    risk: high
    verify: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --pgpool-bin target/release/pgpool
  isolation:
    id: R1
    text: "A bounded Tokio worker count preserves transaction lease reuse, reset-between-owners isolation, and capacity behavior."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --lib --test pool --test pool_modes
---
flowchart TD
    r1[R1 isolation] --> cargo_test_p_pgpool_lib_test_pool_test_pool_modes[cargo test -p pgpool --lib --test pool --test pool_modes]
    r2[R2 comparison] --> apps_pgpool_benchmarks_pgbouncer_transaction_pooling_run_sh_pgpool_bin_target_release_pgpool[apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --pgpool-bin target/release/pgpool]
```
