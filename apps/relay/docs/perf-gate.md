# relay competitor perf-gate (#125)

A **permanent regression gate**, mirroring lumen's `perf_gate_vs_db`: every
iteration relay must hold its ratio versus the competitors (ratchet) and must
beat the primary bar (RabbitMQ) where claimed, or the gate fails the build.

## Cell (one workload fanned across N targets)

| Cell | Competitors | Metric | Must-beat |
|------|-------------|--------|-----------|
| work-queue | RabbitMQ (quorum/classic), NATS JetStream WorkQueuePolicy, Redis Streams, Dragonfly Streams | publish qps + lease/ack qps (higher better) | RabbitMQ quorum |

Primary bar = **RabbitMQ** — the closest direct single-cast work-queue broker.
NATS JetStream is the Axiom ecosystem replacement target, Redis Streams is the
pragmatic job-queue baseline, and Dragonfly is the multi-core Redis-compatible
variant. Redpanda/Kafka-class topic replay belongs to tape's gate, not relay's.

## How it runs

- **engine** is relay's in-process durable core baseline: disk-backed log with
  the default relay fsync policy, not an in-memory ceiling.
- **relay** is driven over its HTTP/2 service (`publish-batch` / `lease-batch` /
  `ack-batch`) with CBOR.
- **RabbitMQ** uses a durable quorum queue by default, persistent messages,
  publisher confirms, competing consumers, and manual ack.
- **NATS JetStream** uses WorkQueue retention, file storage, explicit ack,
  AckWait as the lease TTL analogue, and MaxDeliver for redelivery/DLQ posture.
- **Redis / Dragonfly** use Streams consumer groups (`XADD` / `XREADGROUP` /
  `XACK` / `XDEL`) with append-only persistence enabled; the harness refuses to
  run them when AOF/appendonly is off. The measured posture includes
  delete-on-ack.
- arena reduces each cell to a `ratio` normalized so **higher is better for
  relay**, then applies the gate (see [`perf_gate`](../src/perf_gate.rs)):
  - **ratchet** — `ratio >= baseline_ratio * ratchet` (no regression since the
    last passing run; default `ratchet = 0.95`);
  - **must-beat** — on claimed cells, `ratio >= 1.0` (relay is actually ahead).

The gate passes only when no cell regresses and no must-beat cell is lost; a
passing run records new baselines.

## Artifacts

- Closed-loop harness: [`examples/bench_compare.rs`](../examples/bench_compare.rs).
- Gate spec: [`projects/arena/examples/relay-vs-rabbitmq-nats-redis.toml`](../../arena/examples/relay-vs-rabbitmq-nats-redis.toml)
- EC binding: `ec.benchmark` under the `relay` project in `.aw/config.toml`
  (`aw health --verify-ec` drives it).
- relay-side measurement: `cargo bench -p relay` (criterion;
  [`benches/relay_bench.rs`](../benches/relay_bench.rs)) — the competitor-free
  local baseline for publish / lease+ack.
- Gate rule + workload smoke: [`tests/perf_gate.rs`](../tests/perf_gate.rs).
