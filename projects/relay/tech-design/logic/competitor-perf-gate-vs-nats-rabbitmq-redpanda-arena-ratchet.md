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
(fill)
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-perf-gate-test-plan
entry: start
nodes:
  start:
    kind: start
    label: "pending"
edges: []
---
flowchart TD
    start([pending])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
(fill)
```
