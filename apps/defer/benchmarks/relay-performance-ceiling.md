# Defer / Relay scheduler ceiling

## Contract

Defer may spend more work maintaining ETA ordering, queue controls, permits,
attempts, retry/DLQ transitions, and terminal task records, but its scheduler
must retain at least 80% of Relay's throughput under the same durable lifecycle.
Google Cloud Tasks is Defer's external feature-contract peer; this local
performance ceiling intentionally compares the two Axiom state machines so
network-region effects do not dominate scheduler overhead.

```yaml
benchmark_contract:
  performance_comparator: relay
  scope: same-host-sibling-implementation-ceiling
  rss_measurement: process-shared-not-component-isolated
  dated_observation_authoritative: false
  cloud_tasks_performance_claim: false
  universal_superiority_claim: false
  messages: 1000
  batches: 10
  batch_size: 100
  payload_serialized_bytes: 128
  minimum_ratio: 0.8
```

## Reproduce

```bash
cargo test --release -p defer --test relay_performance_ceiling -- --ignored --nocapture
```

The gate uses 1,000 messages in ten batches of 100, a 128-byte serialized JSON payload, one
Raft voter, `fsync=always`, and the same durable enqueue -> committed lease ->
committed ack lifecycle. Defer additionally maintains its queue and task
lifecycle state. A result passes when `defer_to_relay_ratio >= 0.8`.

## 2026-07-17 local run

Host class: Darwin arm64 (`T6000`), otherwise idle release build.

| Metric | Defer | Relay |
|---|---:|---:|
| Throughput | 1,261.25 msg/s | 134.15 msg/s |
| p50 batch lifecycle | 72,082 us | 738,921 us |
| p95 batch lifecycle | 105,369 us | 969,254 us |
| p99 batch lifecycle | 105,369 us | 969,254 us |
| CPU during measured section | 100.00 ms | 1,080.00 ms |
| Process-shared RSS sample | 28,770,304 bytes | 38,584,320 bytes |
| Durable bytes | 1,576,158 | 1,442,101 |
| Disk amplification | 12.3137x | 11.2664x |
| Errors | 0 | 0 |

Observed `Defer / Relay` throughput ratio: **9.4019x**; required minimum:
**0.8x**. RSS is explicitly process-shared because both cases run in one test
process; it is a real point-in-time process sample, not a component-isolated
memory comparison. The executable assertion, not this dated observation, is
the regression gate.
