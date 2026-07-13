---
id: '1618'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-multithread-runtime-contract
entry: runtime
nodes:
  runtime: { kind: start, label: "Tokio runtime selection" }
  workers: { kind: process, label: "Multiple executor workers run independent relay tasks" }
  semantics: { kind: process, label: "Existing wire protocol and pool state machine" }
  result: { kind: terminal, label: "Same isolation with higher CPU concurrency" }
edges:
  - { from: runtime, to: workers }
  - { from: workers, to: semantics }
  - { from: semantics, to: result }
---
flowchart LR
  runtime([Tokio runtime]) --> workers[multiple relay workers]
  workers --> semantics[existing protocol and pool state machine]
  semantics --> result([same isolation higher concurrency])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/bin/pgpool.rs
    action: modify
    section: pgpool-multithread-runtime-contract
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
