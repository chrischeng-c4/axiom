---
id: relay-perf-gate
summary: A permanent regression gate (mirrors lumen perf_gate_vs_db) — a single work-queue cell vs RabbitMQ quorum/classic queues, NATS JetStream WorkQueuePolicy, Redis Streams, and Dragonfly. Primary bar = RabbitMQ (the single-cast work-queue broker relay replaces). Redpanda/Kafka-class replay journals are tape competitors, not relay competitors. Gate = no-regression ratchet + must-beat where claimed. relay-side benches, the bench_compare harness, and the ratchet rule are standalone; arena remains the advisory ratio wrapper.
capability_refs:
  - id: competitor-performance
    role: primary
    gap: normalized-win-ratchet-decision-model
    claim: normalized-win-ratchet-decision-model
    coverage: full
    rationale: "Defines the normalized ratio, ratchet, and must-beat decision model used by the relay perf gate."
  - id: competitor-performance
    role: primary
    gap: external-broker-comparison
    claim: external-broker-comparison
    coverage: partial
    rationale: "Defines the external competitor comparison contract while adapter execution remains tied to the arena dogfood path."
fill_sections: [logic, config, unit-test, changes]
---

# relay competitor perf-gate — vs RabbitMQ / NATS JetStream / Redis Streams (work queue ratchet)

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
    label: "Cell: work-queue (publish -> lease/reserve/read -> ack/delete) vs RabbitMQ / JetStream / Redis Streams / Dragonfly"
  measure:
    kind: process
    label: "Measure publish throughput and lease/ack throughput per target"
  ratio:
    kind: process
    label: "ratio = peer / relay; with lower-is-better latency, ratio > 1 means relay wins"
  ratchet:
    kind: decision
    label: "Ratchet: is relay's ratio still >= baseline * ratchet (no regression since last run)?"
  mustbeat:
    kind: decision
    label: "On cells where relay claims to win (primary bar = RabbitMQ): is relay actually faster?"
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
    run([arena compare-N]) --> cells[1 work-queue cell x N targets]
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
# The executable comparison is `cargo run -p relay --release --example
# bench_compare -- --backend <target>`, with one closed-loop workload and one
# measurement implementation. Every backend is durable-only: relay engine uses a
# disk-backed data dir and the default fsync policy; RabbitMQ uses durable queues
# + persistent messages + confirms; JetStream uses file storage; Redis/Dragonfly
# must have append-only persistence enabled or the harness fails fast. arena wraps
# those runs for normalized ratio + ratchet reporting; it is not the daily
# production EC gate.

base: relay            # ratios divide by relay
ratchet: 0.95          # relay may not drop below 95% of its recorded baseline ratio
primary_bar: rabbitmq  # the thing relay replaces; must-beat where claimed

cells:
  work_queue:
    competitors: [rabbitmq-quorum, nats-jetstream, redis-streams, dragonfly-streams]
    metrics: [publish_qps, lease_ack_qps]      # higher is better
    must_beat: [rabbitmq-quorum]
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
    label: "run each benched workload (publish, lease+ack) at small scale"
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
  - path: apps/relay/Cargo.toml
    action: modify
    section: config
    impl_mode: hand-written
    reason: "Add criterion, h2c, Redis, NATS, and RabbitMQ dev dependencies plus the relay_bench and bench_compare targets."
  - path: apps/relay/src/perf_gate.rs
    action: create
    section: logic
    impl_mode: hand-written
    reason: "The ratchet gate rule: evaluate per-cell ratios against the recorded baseline (no-regression) plus must-beat, returning a pass/fail verdict."
  - path: apps/relay/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Declare and re-export the perf_gate module."
  - path: apps/relay/benches/relay_bench.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "criterion benchmarks for the relay-side publish and work-queue lease+ack cycle."
  - path: apps/relay/examples/bench_compare.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "Closed-loop durable-only executable harness for engine, relay h2c, RabbitMQ, NATS JetStream, Redis Streams, and Dragonfly using the same publish then lease/ack workload."
  - path: apps/relay/tests/perf_gate.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "Tests for the ratchet rule (holds / regresses / must-beat lost) and a small-scale smoke of the benched workloads."
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] The gate is sound: per-cell ratio vs base, ratchet (no-regression vs recorded baseline), then must-beat on claimed cells; either failing condition fails the build. Mirrors lumen perf_gate_vs_db.
- [config] base/ratchet/primary_bar + one work-queue cell with competitors, metric direction, and must-beat, matching the single-cast work-queue broker (no broadcast or tape-style replay cell).
- [unit-test] Pure ratchet-rule cases (hold / regress / must-beat lost) are deterministic; the workload smoke keeps the benched cells honest in CI without competitors.
- [changes] relay-side benches + ratchet rule + tests + the bench_compare harness; external broker clients are dev-dependencies only. arena is the advisory ratio wrapper, keeping the production EC gate standalone.
