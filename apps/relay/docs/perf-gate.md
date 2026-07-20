# relay competitor perf-gate (#125)

A **permanent regression gate**, mirroring lumen's `perf_gate_vs_db`: every
iteration relay must hold its ratio versus the competitors (ratchet) and must
beat the primary bar (RabbitMQ) where claimed, or the gate fails the build.

## Cell (one workload fanned across N targets)

| Cell | Competitors | Metric | Must-beat |
|------|-------------|--------|-----------|
| work-queue (batch 100) | RabbitMQ (quorum/classic), NATS JetStream WorkQueuePolicy, Redis Streams, Dragonfly Streams | publish qps + lease/ack qps (higher better) | none; regression ratchet only |
| bulk work-queue (batch 1,000) | RabbitMQ quorum, NATS JetStream WorkQueuePolicy | full publish + lease/ack lifecycle throughput (higher better) | RabbitMQ quorum |

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
passing run records new baselines. The must-beat claim is deliberately scoped
to Relay's bulk batch API. The interactive batch-100 cell remains visible and
ratcheted, but is not presented as a current broker win.

## 2026-07-17 release-local calibration

Host: Darwin arm64 T6000. Services ran one at a time on loopback with one
durable replica, 100,000 messages, 128-byte payloads, concurrency 10, and
batch size 1,000. Relay used its release h2c/CBOR service; RabbitMQ 3.13 used a
durable quorum queue, persistent messages, publisher confirms, and manual ack;
NATS Server 2.12.6 used JetStream file storage, WorkQueue retention, and
explicit ack. Connections and setup were outside the timed phases.

| Backend | Publish msg/s | Publish p50/p95/p99 | Lease+ack msg/s | Lease+ack p50/p95/p99 | Full lifecycle msg/s |
|---|---:|---:|---:|---:|---:|
| Relay | 141,985 | 69,956 / 77,057 / 77,329 us | 97,617 | 99,319 / 123,332 / 138,562 us | 57,847 |
| RabbitMQ quorum | 67,947 | 127,139 / 171,234 / 334,211 us | 48,486 | 175,983 / 203,625 / 291,586 us | 28,295 |
| NATS JetStream | 64,402 | 148,944 / 215,615 / 219,289 us | 75,642 | 100,330 / 108,878 / 108,931 us | 34,785 |

The full-lifecycle rate is `1 / (1/publish_rate + 1/lease_ack_rate)`: Relay is
2.04x RabbitMQ and 1.66x NATS for this bulk workload, with zero command or
delivery errors.

Resource evidence was captured around the same run. Client/server CPU was
3.22 s for Relay, 15.11 s for RabbitMQ, and 7.43 s for NATS. Client maximum RSS
was 30,605,312 / 126,042,112 / 10,141,696 bytes respectively; server RSS moved
203,712->282,784 / 341,092->481,944 / 85,792->109,616 KiB. Incremental retained
durable bytes were 33,280,000 / 48,553,984 / 1,146,880 bytes, or 2.60x / 3.79x /
0.09x the logical payload bytes. This last number is retained-footprint
amplification after ack, not device-write amplification; JetStream eagerly
reclaims WorkQueue data, so the two must not be conflated.

At batch size 100, Relay did not beat either broker in the same calibration.
That is why the arena claim is split rather than hiding the losing cell.

## Artifacts

- Closed-loop harness: [`examples/bench_compare.rs`](../examples/bench_compare.rs).
- Gate spec: [`apps/arena/examples/relay-vs-rabbitmq-nats-redis.toml`](../../arena/examples/relay-vs-rabbitmq-nats-redis.toml)
- EC binding: `ec.benchmark` under the `relay` project in `.aw/config.toml`
  (`aw health --verify-ec` drives it).
- relay-side measurement: `cargo bench -p relay` (criterion;
  [`benches/relay_bench.rs`](../benches/relay_bench.rs)) — the competitor-free
  local baseline for publish / lease+ack.
- Gate rule + workload smoke: [`tests/perf_gate.rs`](../tests/perf_gate.rs).
