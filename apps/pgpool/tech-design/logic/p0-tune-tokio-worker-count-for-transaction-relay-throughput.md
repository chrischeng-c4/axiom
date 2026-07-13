---
id: '1622'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-worker-count-contract
entry: runtime
nodes:
  runtime: { kind: start, label: "Tokio multi-thread runtime" }
  tuning: { kind: process, label: "Fixed worker cardinality" }
  invariant: { kind: process, label: "Unchanged proxy protocol and reset state machine" }
  evidence: { kind: terminal, label: "Repeated clean benchmark evidence" }
edges:
  - { from: runtime, to: tuning }
  - { from: tuning, to: invariant }
  - { from: invariant, to: evidence }
---
flowchart LR
  runtime([Tokio multi-thread]) --> tuning[fixed worker cardinality]
  tuning --> invariant[unchanged reset and wire semantics]
  invariant --> evidence([repeat benchmark evidence])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/bin/pgpool.rs
    action: modify
    section: pgpool-worker-count-contract
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
