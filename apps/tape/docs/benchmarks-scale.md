<!-- HANDWRITE-BEGIN gap="missing-generator:logic:7beae5a1" tracker="pending-tracker" reason="Release real-service replay calibration and scale posture for Tape, NATS JetStream, and Kafka KRaft." -->
# tape — benchmark and scale posture

This page tracks Tape's current performance regression gates. Unlike Lumen's
multi-size competitive calibration matrix, Tape has one local gate and two
real-service replay gates; this page documents measured claims only.

> **Status:** release Tape h2c replay is calibrated against NATS JetStream and
> Kafka KRaft. Redpanda, Pulsar, and RabbitMQ Streams remain uncalibrated.
> RabbitMQ topic exchange is reported
> `not_a_replay_baseline` (routing/fanout only, not an offset/time replay
> system) and is excluded from performance comparison entirely.

## Benchmark Standard

| axis | standard |
|---|---|
| Engines | Tape local regression; release Tape h2c service versus NATS JetStream and Kafka KRaft |
| Data size | 1,000 events / 128-byte payload (local regression); 20,000 events / 128-byte payload (real-service replay) |
| Metric | local append/replay/checkpoint budget; symmetric full-replay network latency ratio |
| Reliability | `verify_report`/`verify_external_replay_win` fail the test on any budget or ratio miss |

## Gate 1 — Local regression budget

`apps/tape/tests/tape_perf_gate.rs` runs `tape::bench::run_benchmark(1_000,
128)` and asserts:

- `report.local_regression_passed` — append/replay/checkpoint stay inside the
  local regression budget.
- `!report.external_peer_win_claim` — the report never claims a win over
  Kafka/Redpanda/Pulsar/JetStream/RabbitMQ Streams from the local-only run.
- Every peer stays `!win_claim` in the local-only report. Kafka and NATS report
  `calibrated_separate_gate`; Redpanda, Pulsar, and RabbitMQ Streams remain
  `not_calibrated`. Only the release real-service tests may emit named wins.

Reproduce:

```bash
cargo test -p tape --test tape_perf_gate -- --nocapture
```

The `tape-bench` CLI exposes the same report for ad hoc runs:

```bash
cargo run -p tape --bin tape-bench -- run --events 1000 --format json
```

## Gate 2 — Real NATS JetStream replay win

`apps/tape/tests/tape_vs_nats_jetstream.rs` starts real release Tape h2c and
local `nats-server -js` processes, prepares the same durable 20,000-event,
128-byte-payload backlog, then takes five validated full-replay samples from
each service over warm connections:

- Tape: compact read-only h2c replay frames carrying offset, timestamp, key,
  and payload bytes; the client validates the complete body.
- JetStream: a push consumer with `DeliverPolicy::All` draining the full
  backlog.

The gate requires Tape's p50 full-replay latency to beat JetStream's by at
least **1.5x** (`REQUIRED_REPLAY_RATIO`);
`tape::bench::verify_external_replay_win` fails the test otherwise. The JSON
evidence also reports throughput, p50/p95/p99, child-process CPU and RSS,
durable bytes/amplification, and errors for both services.

Reproduce (needs `nats-server` on `PATH`; the test spawns and tears it down
itself — no manual broker setup required):

```bash
cargo test --release -p tape --test tape_vs_nats_jetstream -- --ignored --nocapture
```

The latest 2026-07-18 calibration produced:

| Metric | Tape | NATS JetStream |
|---|---:|---:|
| Throughput | 967,614 events/s | 748,828 events/s |
| p50 full replay | 13,586 us | 27,384 us |
| p95 / p99 | 45,457 / 45,457 us | 32,980 / 32,980 us |
| CPU during samples | 40 ms | 160 ms |
| RSS | 31,981,568 bytes | 51,920,896 bytes |
| Durable bytes | 5,066,792 | 3,842,369 |
| Disk amplification | 1.9792x | 1.5009x |
| Errors | 0 | 0 |

The p50 ratio was **2.02x**, above the 1.5x floor. Broker setup and backlog
publication are outside the latency samples; complete transfer and validation
are inside them.

## Gate 3 — Real Kafka KRaft replay win

`apps/tape/tests/tape_vs_kafka.rs` starts release Tape h2c plus a pinned
`apache/kafka:3.9.0` single-node KRaft broker in Docker. A real `rskafka`
consumer and the Tape frame client download and validate the same durable
20,000-event backlog. The symmetric gate also requires 1.5x.

```bash
cargo test --release -p tape --test tape_vs_kafka -- --ignored --nocapture
```

The latest corrected 2026-07-18 calibration measured Tape at 13,575 us and
Kafka at 55,243 us: **4.07x**. The retired 20x/110x numbers compared in-process Tape
memory with a network Kafka client and are not valid evidence.

## Scale Vocabulary

| class | meaning | current repo posture |
|---|---|---|
| Local regression | 1,000 events | routine gate, every `cargo test -p tape` run |
| JetStream calibration | 20,000 events | release real-service gate — Tape h2c + `nats-server -js` |
| Kafka calibration | 20,000 events | release real-service gate — Tape h2c + pinned KRaft Docker broker |
| Redpanda / Pulsar / RabbitMQ Streams | — | not calibrated; no benchmark exists yet, no win claimed |
| RabbitMQ topic exchange | — | not a replay baseline; routing/fanout comparison only (see Competitor Feature Parity in `README.md`) |

## Known Remaining Work

| work | why it remains |
|---|---|
| Redpanda/Pulsar/RabbitMQ Streams calibration gates | Each needs its own symmetric real-service `tests/tape_vs_<peer>.rs` harness. |
| isolated perf host | The NATS/Kafka gates are release-mode and co-located; cross-hardware publication still needs a dedicated perf host. |
| retention/compaction/backfill perf | The domain behavior is implemented; dedicated retention/backfill efficiency cells remain unclaimed. |

*Generated as the tape docs+scripts+traits polish handoff (#1331, epic
#1324). Coordinates are sourced from `apps/tape/src/bench.rs`,
`apps/tape/tests/tape_perf_gate.rs`, `apps/tape/tests/
tape_vs_nats_jetstream.rs`.*
<!-- HANDWRITE-END -->
