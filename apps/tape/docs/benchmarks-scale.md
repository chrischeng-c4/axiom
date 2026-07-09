<!-- HANDWRITE-BEGIN gap="missing-generator:logic:7beae5a1" tracker="pending-tracker" reason="New docs page (mirrors projects/lumen/docs/benchmarks-scale.md shape, scoped to tape's actual gates): documents apps/tape/tests/tape_perf_gate.rs (local append/replay/checkpoint regression budget, no external peer win claims) and apps/tape/tests/tape_vs_nats_jetstream.rs (real nats-server -js 20k-event 128-byte-payload backlog replay, Tape zero-copy replay_refs >=1.5x win), how to reproduce both, and the explicit not-yet-calibrated peer list (Kafka/Redpanda/Pulsar/RabbitMQ Streams). No new benchmark classes invented." -->
# tape — benchmark and scale posture

This page tracks tape's current performance regression gates. Unlike lumen's
multi-size competitive calibration matrix, tape has exactly two executable
performance gates today; this page documents what is real rather than
projecting a bigger benchmark program tape does not yet run.

> **Status:** the only claimed external-peer win is against NATS JetStream
> local backlog replay. Kafka, Redpanda, Pulsar, and RabbitMQ Streams remain
> `not_calibrated` peers in `tape::bench::run_benchmark`'s report — tape does
> not claim a performance win over them. RabbitMQ topic exchange is reported
> `not_a_replay_baseline` (routing/fanout only, not an offset/time replay
> system) and is excluded from performance comparison entirely.

## Benchmark Standard

| axis | standard |
|---|---|
| Engines | tape by default; NATS JetStream for the one calibrated external win |
| Data size | 1,000 events / 128-byte payload (local regression); 20,000 events / 128-byte payload (JetStream replay win) |
| Metric | local append/replay/checkpoint regression budget; full-replay latency ratio vs JetStream |
| Reliability | `verify_report`/`verify_external_replay_win` fail the test on any budget or ratio miss |

## Gate 1 — Local regression budget

`apps/tape/tests/tape_perf_gate.rs` runs `tape::bench::run_benchmark(1_000,
128)` and asserts:

- `report.local_regression_passed` — append/replay/checkpoint stay inside the
  local regression budget.
- `!report.external_peer_win_claim` — the report never claims a win over
  Kafka/Redpanda/Pulsar/JetStream/RabbitMQ Streams from the local-only run.
- Every peer marked `replay_baseline` reports `status == "not_calibrated"` and
  `!win_claim` until a real calibration run exists for that peer.

Reproduce:

```bash
cargo test -p tape --test tape_perf_gate -- --nocapture
```

The `tape-bench` CLI exposes the same report for ad hoc runs:

```bash
cargo run -p tape --bin tape-bench -- run --events 1000 --format json
```

## Gate 2 — Real NATS JetStream replay win

`apps/tape/tests/tape_vs_nats_jetstream.rs` starts a real local `nats-server
-js` subprocess, publishes a 20,000-event, 128-byte-payload backlog to both a
JetStream stream and tape's in-process `TapeJournal`, then replays each from
the beginning:

- tape: `TapeJournal::replay_refs` — zero-copy, no deserialization of stored
  payloads.
- JetStream: a push consumer with `DeliverPolicy::All` draining the full
  backlog.

The gate requires tape's full-replay latency to beat JetStream's by at least
**1.5x** (`REQUIRED_REPLAY_RATIO`); `tape::bench::verify_external_replay_win`
fails the test otherwise.

Reproduce (needs `nats-server` on `PATH`; the test spawns and tears it down
itself — no manual broker setup required):

```bash
cargo test -p tape --test tape_vs_nats_jetstream -- --nocapture
```

## Scale Vocabulary

| class | meaning | current repo posture |
|---|---|---|
| Local regression | 1,000 events | routine gate, every `cargo test -p tape` run |
| JetStream calibration | 20,000 events | routine gate — real `nats-server -js` subprocess, no external service to provision |
| Kafka / Redpanda / Pulsar / RabbitMQ Streams | — | not calibrated; no benchmark exists yet, no win claimed |
| RabbitMQ topic exchange | — | not a replay baseline; routing/fanout comparison only (see Competitor Feature Parity in `README.md`) |

## Known Remaining Work

| work | why it remains |
|---|---|
| Kafka/Redpanda/Pulsar/RabbitMQ Streams calibration gates | No real-service benchmark harness exists yet for these peers; each needs its own `tests/tape_vs_<peer>.rs` following the JetStream gate's shape. |
| isolated perf host | The JetStream gate currently runs co-located with the test process; release-stable throughput claims need an isolated perf host, same open item as lumen's. |
| retention/compaction/backfill perf | Out of scope until the domain feature itself ships (`README.md`'s Retention And Backfill capability is `planned`). |

*Generated as the tape docs+scripts+traits polish handoff (#1331, epic
#1324). Coordinates are sourced from `apps/tape/src/bench.rs`,
`apps/tape/tests/tape_perf_gate.rs`, `apps/tape/tests/
tape_vs_nats_jetstream.rs`.*
<!-- HANDWRITE-END -->
