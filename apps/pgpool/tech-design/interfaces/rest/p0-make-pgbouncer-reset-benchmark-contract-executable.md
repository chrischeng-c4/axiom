---
id: '1617'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgbouncer-always-reset-contract
entry: benchmark
nodes:
  benchmark: { kind: start, label: "Benchmark runner" }
  pgbouncer: { kind: process, label: "PgBouncer transaction-pool config" }
  reset: { kind: process, label: "DISCARD ALL plus always-reset option" }
  evidence: { kind: process, label: "Hermetic config and profile checks" }
  outcome: { kind: terminal, label: "Comparable reset contract" }
edges:
  - { from: benchmark, to: pgbouncer }
  - { from: pgbouncer, to: reset }
  - { from: reset, to: evidence }
  - { from: evidence, to: outcome }
---
flowchart TD
  benchmark([benchmark runner]) --> pgbouncer[PgBouncer transaction config]
  pgbouncer --> reset[DISCARD ALL and always-reset]
  reset --> evidence[hermetic profile and config checks]
  evidence --> outcome([comparable reset contract])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh
    action: modify
    section: pgbouncer-always-reset-benchmark-contract
    impl_mode: hand-written
  - path: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/README.md
    action: modify
    section: pgbouncer-always-reset-benchmark-contract
    impl_mode: hand-written
  - path: apps/pgpool/tests/pgbouncer_benchmark.rs
    action: modify
    section: pgbouncer-always-reset-benchmark-contract
    impl_mode: hand-written
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgbouncer-always-reset-benchmark-contract-verification
requirements:
  always_reset_config:
    id: R1
    text: "The benchmark-generated PgBouncer configuration explicitly pairs DISCARD ALL with server_reset_query_always = 1."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test pgbouncer_benchmark
  immutable_profile:
    id: R2
    text: "The fairness correction keeps simple protocol, 64 clients, 16 backend connections, and 30-second duration unchanged."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test pgbouncer_benchmark
---
flowchart TD
    r1[R1 always reset config] --> cargo_test_p_pgpool_test_pgbouncer_benchmark[cargo test -p pgpool --test pgbouncer_benchmark]
    r2[R2 immutable profile] --> cargo_test_p_pgpool_test_pgbouncer_benchmark
```
