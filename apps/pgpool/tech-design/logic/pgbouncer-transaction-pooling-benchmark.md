---
id: '1597'
summary: (fill)
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-pgbouncer-transaction-pooling-baseline
entry: benchmark_invocation
nodes:
  benchmark_invocation:
    kind: start
    label: The P0 benchmark runner is invoked with its standard profile (transaction pooling, PostgreSQL simple-query protocol, one configured backend connection cap) or --dry-run.
  dry_run:
    kind: decision
    label: Is --dry-run requested?
  emit_profile:
    kind: terminal
    label: Emit the exact machine-readable profile without binding ports or requiring external tools; this is the hermetic contract test seam.
  prerequisites:
    kind: process
    label: Verify pgpool binary, pgbouncer, initdb, postgres, psql, and pgbench are available; fail with one actionable missing-tool error otherwise.
  temporary_backend:
    kind: process
    label: Create a temporary trust-auth PostgreSQL cluster on loopback, start it on an allocated port, create pgpool_bench, and initialize the standard pgbench workload directly against that backend.
  equal_targets:
    kind: process
    label: Start PgBouncer in transaction pool mode and pgpool serve in transaction mode against the SAME fresh backend. Both targets use the SAME backend connection cap, client count, job count, duration, database, user, and cleartext loopback transport.
  sequential_measurement:
    kind: process
    label: Run pgbench -M simple against PgBouncer and then pgpool sequentially; reject a target that reports command failure or lacks its TPS/average-latency result.
  normalized_report:
    kind: process
    label: Parse both pgbench reports and emit one JSON document containing profile, raw TPS/average latency, pgpool-to-PgBouncer ratios, and observed winner; retain no baseline or ratchet in this P0 slice.
  cleanup:
    kind: terminal
    label: Stop both proxies and the temporary PostgreSQL process and remove the temporary data directory on success, failure, or interruption.
edges:
  - from: benchmark_invocation
    to: dry_run
  - from: dry_run
    to: emit_profile
    label: yes
  - from: dry_run
    to: prerequisites
    label: no
  - from: prerequisites
    to: temporary_backend
  - from: temporary_backend
    to: equal_targets
  - from: equal_targets
    to: sequential_measurement
  - from: sequential_measurement
    to: normalized_report
  - from: normalized_report
    to: cleanup
---
flowchart TD
    benchmark_invocation([Invoke transaction-pooling benchmark]) --> dry_run{--dry-run?}
    dry_run -->|yes| emit_profile([Emit profile JSON without external side effects])
    dry_run -->|no| prerequisites[Verify pgpool, pgbouncer and PostgreSQL benchmark tools]
    prerequisites --> temporary_backend[Start temporary trust-auth PostgreSQL and seed pgbench]
    temporary_backend --> equal_targets[Start equal-capacity PgBouncer transaction pool and pgpool transaction pool]
    equal_targets --> sequential_measurement[Run pgbench -M simple sequentially against both targets]
    sequential_measurement --> normalized_report[Parse TPS and average latency, emit comparison JSON]
    normalized_report --> cleanup([Always stop processes and remove temporary cluster])
```

This is applicable as a P0 competitor-performance baseline. It proves the first necessary condition for a PgBouncer win: a repeatable, protocol-compatible, equal-capacity measurement. It deliberately does not mutate the pooler data plane, introduce a performance ratchet, claim a win before observing one, test TLS/auth variants, or benchmark the currently unsupported extended-query protocol. The harness lives under `apps/pgpool/benchmarks/pgbouncer-transaction-pooling/`; its hermetic `--dry-run` profile is covered by a normal Cargo integration test, while the real benchmark uses actual PgBouncer and PostgreSQL processes and is manually invoked or later run under Vat. A later shared `rig` simple-query transport may replace the pgbench driver without changing this profile's fairness rules.

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-pgbouncer-transaction-pooling-baseline-verification
requirements:
  ac1_live_baseline_emits_both_target_metrics_when_enabled:
    id: AC1
    text: "When PGPOOL_RUN_PGBOUNCER_BENCH=1 and the documented real-tool prerequisites are present, the runner provisions a temporary PostgreSQL backend and emits one parseable comparison JSON document containing nonzero TPS and average-latency values for both PgBouncer and pgpool plus their ratios; otherwise the integration test skips with an actionable prerequisite message."
    kind: integration
    risk: high
    verify: pgbouncer_benchmark::live_transaction_pooling_baseline_emits_comparable_metrics_when_enabled
  r1_dry_run_declares_a_protocol_compatible_equal_capacity_profile:
    id: R1
    text: "The benchmark runner's --dry-run JSON fixes transaction pooling, PostgreSQL simple-query protocol, and the identical backend connection cap/client count/job count/duration for pgpool and PgBouncer before either target is measured."
    kind: functional
    risk: high
    verify: pgbouncer_benchmark::dry_run_profile_declares_equal_transaction_pooling_inputs
  r2_runner_remains_a_parseable_hermetic_entrypoint:
    id: R2
    text: "The shipped benchmark runner passes bash syntax validation and its --dry-run mode exits successfully without requiring PgBouncer, PostgreSQL, open ports, or a compiled pgpool binary."
    kind: regression
    risk: medium
    verify: pgbouncer_benchmark::runner_is_syntax_valid_and_dry_run_is_hermetic
---
flowchart TD
    ac1[AC1 ac1 live baseline emits both target metrics when enabled] --> pgbouncer_benchmark_live_transaction_pooling_baseline_emits_comparable_metrics_when_enabled[pgbouncer_benchmark::live_transaction_pooling_baseline_emits_comparable_metrics_when_enabled]
    r1[R1 r1 dry run declares a protocol compatible equal capacity profile] --> pgbouncer_benchmark_dry_run_profile_declares_equal_transaction_pooling_inputs[pgbouncer_benchmark::dry_run_profile_declares_equal_transaction_pooling_inputs]
    r2[R2 r2 runner remains a parseable hermetic entrypoint] --> pgbouncer_benchmark_runner_is_syntax_valid_and_dry_run_is_hermetic[pgbouncer_benchmark::runner_is_syntax_valid_and_dry_run_is_hermetic]
```
