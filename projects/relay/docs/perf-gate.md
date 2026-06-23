# relay competitor perf-gate (#125)

A **permanent regression gate**, mirroring lumen's `perf_gate_vs_db`: every
iteration relay must hold its ratio versus the competitors (ratchet) and must
beat the primary bar (NATS) where claimed, or the gate fails the build.

## Cells (one workload fanned across N targets)

| Cell | Competitors | Metric | Must-beat |
|------|-------------|--------|-----------|
| broadcast / pub-sub | NATS core, Redis pub/sub | fan-out p99 (lower better) | NATS core |
| work-queue | RabbitMQ (quorum), NATS JetStream, Redis Streams | lease/ack qps (higher better) | NATS JetStream |
| durable log / replay | Redpanda, Pulsar | append qps + `subscribe(from_seq)` replay (higher better) | report-only |

Primary bar = **NATS** — the thing relay replaces.

## How it runs

- **relay** is driven over its HTTP/2 service (`publish` / `lease` / `ack` /
  `subscribe`).
- **competitors** speak native (non-HTTP) protocols, so arena drives them with
  the `command` flavor — a small per-target adapter that emits one metric. The
  adapters and brokers run in **CI** (compose), not in unit tests.
- arena reduces each cell to a `ratio` normalized so **higher is better for
  relay**, then applies the gate (see [`perf_gate`](../src/perf_gate.rs)):
  - **ratchet** — `ratio >= baseline_ratio * ratchet` (no regression since the
    last passing run; default `ratchet = 0.95`);
  - **must-beat** — on claimed cells, `ratio >= 1.0` (relay is actually ahead).

The gate passes only when no cell regresses and no must-beat cell is lost; a
passing run records new baselines.

## Artifacts

- Gate spec: [`projects/arena/examples/relay-vs-nats-rabbitmq-redpanda.toml`](../../arena/examples/relay-vs-nats-rabbitmq-redpanda.toml)
- EC binding: `ec.benchmark` under the `relay` project in `.aw/config.toml`
  (`aw health --verify-ec` drives it).
- relay-side measurement: `cargo bench -p relay` (criterion;
  [`benches/relay_bench.rs`](../benches/relay_bench.rs)) — the competitor-free
  local baseline for append / fan-out / lease+ack.
- Gate rule + workload smoke: [`tests/perf_gate.rs`](../tests/perf_gate.rs).
