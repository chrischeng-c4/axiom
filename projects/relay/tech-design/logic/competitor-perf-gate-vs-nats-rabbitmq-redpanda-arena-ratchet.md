---
id: relay-perf-gate
summary: A permanent regression gate (mirrors lumen perf_gate_vs_db) — arena compare-N + ratchet across three cells (broadcast, work-queue, durable log) vs NATS / RabbitMQ / Redpanda. Primary bar = NATS. Gate = no-regression ratchet + must-beat where claimed. relay-side benches + ratchet rule are standalone; competitor adapters run in CI.
fill_sections: [logic, config, unit-test, changes]
---

# relay competitor perf-gate — vs NATS / RabbitMQ / Redpanda (arena, ratchet)

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-perf-gate-flow
entry: run
nodes:
  run:
    kind: start
    label: "arena runs each cell as one workload fanned across N targets (relay + competitors)"
  cells:
    kind: process
    label: "Cells: broadcast (vs NATS core / Redis pub-sub), work-queue (vs RabbitMQ / JetStream / Redis Streams), durable log (vs Redpanda / Kafka / Pulsar)"
  measure:
    kind: process
    label: "Measure the metric per target (fan-out latency, lease/ack throughput, append + replay-from-seq)"
  ratio:
    kind: process
    label: "ratio = peer / relay; with lower-is-better latency, ratio > 1 means relay wins"
  ratchet:
    kind: decision
    label: "Ratchet: is relay's ratio still >= baseline * ratchet (no regression since last run)?"
  mustbeat:
    kind: decision
    label: "On cells where relay claims to win (primary bar = NATS): is relay actually faster?"
  pass:
    kind: terminal
    label: "Gate passes; record new baselines"
  fail:
    kind: terminal
    label: "Gate FAILS the build (regression, or lost a must-beat cell)"
edges:
  - { from: run, to: cells }
  - { from: cells, to: measure }
  - { from: measure, to: ratio }
  - { from: ratio, to: ratchet }
  - { from: ratchet, to: fail, label: "regressed" }
  - { from: ratchet, to: mustbeat, label: "held" }
  - { from: mustbeat, to: fail, label: "lost a claimed cell" }
  - { from: mustbeat, to: pass, label: "won where claimed" }
---
flowchart TD
    run([arena compare-N]) --> cells[3 cells x N targets]
    cells --> measure[measure metric per target]
    measure --> ratio[ratio = peer / relay]
    ratio --> ratchet{>= baseline * ratchet?}
    ratchet -->|regressed| fail([FAIL build])
    ratchet -->|held| mustbeat{won must-beat cells?}
    mustbeat -->|no| fail
    mustbeat -->|yes| pass([PASS, record baselines])
```
## Config
<!-- type: config lang: yaml -->

```yaml
# relay perf-gate (arena compare-N + ratchet); mirrors lumen perf_gate_vs_db.
# The live comparison runs via arena against the spec referenced by the relay
# project's ec.benchmark binding in .aw/config.toml. Competitor adapters run in
# CI: NATS / RabbitMQ / Redpanda speak native (non-HTTP) protocols, so they use
# arena's command flavor, while relay is driven over its HTTP/2 service.

base: relay            # ratios divide by relay
ratchet: 0.95          # relay may not drop below 95% of its recorded baseline ratio
primary_bar: nats      # the thing relay replaces; must-beat where claimed

cells:
  broadcast:
    competitors: [nats-core, redis-pubsub]
    metric: fanout_p99_ms      # lower is better
    must_beat: [nats-core]
  work_queue:
    competitors: [rabbitmq-quorum, nats-jetstream, redis-streams]
    metric: lease_ack_qps      # higher is better
    must_beat: [nats-jetstream]
  durable_log:
    competitors: [redpanda, pulsar]
    metric: append_qps         # higher is better
    must_beat: []              # report-only vs Kafka-class for now
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-perf-gate-test-plan
entry: suite
nodes:
  suite:
    kind: start
    label: "perf-gate rule + bench-workload smoke"
  t_hold:
    kind: process
    label: "current ratio >= baseline * ratchet"
  a_hold:
    kind: terminal
    label: "assert the ratchet PASSES (no regression)"
  t_regress:
    kind: process
    label: "current ratio < baseline * ratchet"
  a_regress:
    kind: terminal
    label: "assert the ratchet FAILS the gate"
  t_mustbeat:
    kind: process
    label: "a must-beat cell where relay is slower (ratio < 1)"
  a_mustbeat:
    kind: terminal
    label: "assert the gate FAILS even if the ratchet held"
  t_bench:
    kind: process
    label: "run each benched workload (append, fan-out, lease+ack) at small scale"
  a_bench:
    kind: terminal
    label: "assert each completes and the work-queue cycle is exactly-once (gate workloads are valid)"
edges:
  - { from: suite, to: t_hold, label: "case: ratchet holds" }
  - { from: t_hold, to: a_hold }
  - { from: suite, to: t_regress, label: "case: ratchet regress" }
  - { from: t_regress, to: a_regress }
  - { from: suite, to: t_mustbeat, label: "case: must-beat lost" }
  - { from: t_mustbeat, to: a_mustbeat }
  - { from: suite, to: t_bench, label: "case: workload smoke" }
  - { from: t_bench, to: a_bench }
---
flowchart TD
    suite([perf-gate suite]) --> t_hold[ratio >= baseline*ratchet]
    t_hold --> a_hold([ratchet passes])
    suite --> t_regress[ratio < baseline*ratchet]
    t_regress --> a_regress([gate fails])
    suite --> t_mustbeat[must-beat cell, relay slower]
    t_mustbeat --> a_mustbeat([gate fails])
    suite --> t_bench[run workloads small scale]
    t_bench --> a_bench([complete, exactly-once])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/relay/Cargo.toml
    action: modify
    section: config
    impl_mode: hand-written
    reason: "Add criterion dev-dependency and the relay_bench benchmark target."
  - path: projects/relay/src/perf_gate.rs
    action: create
    section: logic
    impl_mode: hand-written
    reason: "The ratchet gate rule: evaluate per-cell ratios against the recorded baseline (no-regression) plus must-beat, returning a pass/fail verdict."
  - path: projects/relay/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Declare and re-export the perf_gate module."
  - path: projects/relay/benches/relay_bench.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "criterion benchmarks for the three gate cells: append throughput, broadcast fan-out, work-queue lease+ack cycle (the relay-side measurement)."
  - path: projects/relay/tests/perf_gate.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "Tests for the ratchet rule (holds / regresses / must-beat lost) and a small-scale smoke of the benched workloads."
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] compare-N -> measure -> ratio -> ratchet (no-regression) -> must-beat -> pass/fail. Mirrors lumen's perf gate; primary bar NATS. Applicable.
- [config] base/ratchet/primary_bar + the three cells with competitors, metric direction, and must-beat sets. Applicable.
- [unit-test] Ratchet holds/regresses/must-beat-lost + a small-scale smoke of the benched workloads. Applicable.
- [changes] relay-side: criterion benches + ratchet-rule module + tests; the arena spec and EC binding are repo/cross-project infra added alongside. Applicable.
