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
id: pgpool-worker-count-contract-verification
requirements:
  runtime_isolation:
    id: R1
    text: "Worker-count tuning leaves all protocol, pool capacity, and reset isolation behavior unchanged."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --lib --test pool --test pool_modes
  stable_comparison:
    id: R2
    text: "Every retained release comparison has all 64 clients and no pgbench errors; stable win requires multiple runs rather than one outlier."
    kind: integration
    risk: high
    verify: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --pgpool-bin target/release/pgpool
---
flowchart TD
    r1[R1 runtime isolation] --> cargo_test_p_pgpool_lib_test_pool_test_pool_modes[cargo test -p pgpool --lib --test pool --test pool_modes]
    r2[R2 stable comparison] --> apps_pgpool_benchmarks_pgbouncer_transaction_pooling_run_sh_pgpool_bin_target_release_pgpool[apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --pgpool-bin target/release/pgpool]
```
