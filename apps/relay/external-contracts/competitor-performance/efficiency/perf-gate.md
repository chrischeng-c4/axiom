---
id: relay-competitor-performance-ec
summary: Relay performance evidence combines a vat-isolated measured durable lifecycle gate with explicit behavior and stability cases; external broker comparisons remain advisory calibration.
fill_sections: [e2e-test, tool-contract]
---

# EC: Competitive Broker Performance

Relay's production performance contract separates behavior, efficiency, and
stability. The efficiency case measures an fsync-always durable publish then
lease/ack lifecycle in a report-only child process; an independent parent
parses the observations and applies fixed workload-specific floors. Vat keeps
the workspace isolated and meter captures runtime evidence. External RabbitMQ,
NATS JetStream, Redis Streams, and Dragonfly comparisons remain advisory until
equivalent real-service calibration is explicitly promoted into a required
gate.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: relay-competitor-performance-behavior
    capability_id: competitor-performance
    claim_id: normalized-win-ratchet-decision-model
    contract_id: relay-work-queue-performance-workload-behavior
    category: behavior
    command: "bash apps/relay/scripts/ec-evidence.sh performance-behavior"
    assertions:
      - "The publish, ordered lease, batch acknowledgement, committed-watermark, redelivery, and per-subject isolation workloads complete with every message acknowledged exactly once."
      - "The normalized decision model fails both a pinned-ratio regression and loss of a must-beat cell; this behavior case does not claim that its hard-coded inputs are measured performance."
      - "The outer oracle requires all eleven named behavior tests and a non-zero execution count in each test binary, so a renamed or removed test cannot pass as a zero-match Cargo filter."

  - id: relay-competitor-performance-measured-durable-lifecycle
    capability_id: competitor-performance
    claim_id: normalized-win-ratchet-decision-model
    contract_id: relay-measured-durable-lifecycle-envelope
    category: efficiency
    test_path: apps/relay/tests/efficiency_relay_competitor_performance_measured_durable_lifecycle.rs
    command: "bash apps/relay/scripts/ec-evidence.sh performance-efficiency"
    assertions:
      - "A report-only child process uses temporary disk storage with FsyncPolicy::Always to publish and then lease/ack exactly 2,000 128-byte payloads in 100-message batches."
      - "The child emits one machine-readable report containing non-zero elapsed time, at least 20 samples per phase, throughput, batch p95, acknowledgement counts, and error count; missing, malformed, zero-sample, incomplete, or error reports fail closed."
      - "An independent parent parser requires publish and lease/ack throughput >= 500 messages/second and batch p95 <= 500,000 microseconds without calling Relay's perf_gate verdict helper."
      - "A test-owned outer oracle first proves its own zero-test and missing-marker rejection, requires both ignored test names, then accepts only exactly one executed gate and exactly one relay_perf_gate report marker before Meter records the same release invocation."
      - "RabbitMQ, NATS JetStream, Redis Streams, and Dragonfly results remain advisory calibration; this local envelope does not assert an external-broker win."

  - id: relay-competitor-performance-bounded-soak
    capability_id: competitor-performance
    claim_id: normalized-win-ratchet-decision-model
    contract_id: relay-performance-workload-stability
    category: stability
    command: "RELAY_SOAK_AUTOSTART=1 bash apps/relay/scripts/soak.sh"
    assertions:
      - "The bounded fixed-state publish, lease, heartbeat, and inspect workload completes for 60 seconds with a non-zero operation count and zero HTTP or lifecycle errors."
      - "The second observation window stays within the pinned RSS, file-descriptor, thread/task, and p99 latency growth ceilings."
```
## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: relay-meter-performance
    tool: meter
    manifest: meter-relay-performance.toml
    category: efficiency
    command: "cd apps/relay && ../../target/debug/vat run meter-perf"
    native:
      version: 1
      project: relay
      source_contract: relay-competitor-performance-measured-durable-lifecycle
      delegate_command: "cd apps/relay && ../../target/debug/vat run meter-perf"
```
