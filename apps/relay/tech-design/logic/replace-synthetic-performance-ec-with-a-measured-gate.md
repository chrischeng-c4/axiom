---
id: '2172'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-measured-performance-contract
entry: parent
nodes:
  parent:
    kind: start
    label: "Parent oracle starts current test binary in report-only child mode"
  child:
    kind: process
    label: "Child creates temp durable Relay with FsyncPolicy Always"
  publish:
    kind: process
    label: "Publish 2000 messages in 100-message batches and record elapsed samples"
  drain:
    kind: process
    label: "Lease and ack every message in 100-message batches and record elapsed samples"
  report:
    kind: process
    label: "Emit one RELAY_PERF_JSON report with counts samples throughput p95 and errors"
  parse:
    kind: decision
    label: "Parent finds and parses exactly one complete report"
  reject:
    kind: terminal
    label: "FAIL missing malformed zero-sample incomplete or error report"
  thresholds:
    kind: decision
    label: "Observed publish and lease-ack throughput >= pinned floors and p95 <= ceiling"
  pass:
    kind: terminal
    label: "PASS measured workload-specific envelope"
edges:
  - { from: parent, to: child }
  - { from: child, to: publish }
  - { from: publish, to: drain }
  - { from: drain, to: report }
  - { from: report, to: parse }
  - { from: parse, to: reject, label: "invalid" }
  - { from: parse, to: thresholds, label: "valid" }
  - { from: thresholds, to: reject, label: "outside envelope" }
  - { from: thresholds, to: pass, label: "inside envelope" }
---
flowchart TD
    parent[parent independent oracle] --> child[durable fsync-always child]
    child --> publish[2000 batched publishes]
    publish --> drain[lease and ack all]
    drain --> report[RELAY_PERF_JSON observations]
    report --> parse{complete non-zero report?}
    parse -->|no| reject[FAIL closed]
    parse -->|yes| thresholds{pinned floors pass?}
    thresholds -->|no| reject
    thresholds -->|yes| pass[PASS]
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/relay/tests/measured_performance.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Define a serde report for workload relay-durable-publish-lease-ack-v1; a report-only ignored child measures 2000 128-byte messages in 100-message batches on temporary FsyncPolicy Always storage, while the ignored parent parses the child stdout and requires both phases to have at least 20 samples, zero errors, complete counts, at least 500 messages per second, and batch p95 no greater than 500000 microseconds.
  - path: apps/relay/vat.toml
    action: modify
    section: config
    impl_mode: hand-written
    description: Build measured_performance in release mode and make meter-perf run only its exact independent parent gate with ignored tests enabled.
  - path: apps/relay/external-contracts/competitor-performance/efficiency/perf-gate.md
    action: modify
    section: e2e-test
    impl_mode: hand-written
    description: Declare behavior through work_queue_throughput and perf_gate, efficiency through the exact release measured parent, and stability through the existing bounded autostart soak; retain the meter tool contract.
  - path: apps/relay/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: List behavior, efficiency, and stability commands and state that external broker wins remain advisory.
  - path: apps/relay/docs/perf-gate.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: Publish the v1 workload and fixed floor semantics plus the latest measured result after calibration.
  - path: apps/relay/aw.toml
    action: modify
    section: e2e-test
    impl_mode: codegen
    description: Regenerate digest-bound EC inventory and claim bindings after independent acceptance.
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-measured-performance-gate-verification
requirements:
  existing_work_queue_behavior_remains_correct:
    id: R4
    text: "The measured-oracle change does not alter publish, lease, acknowledgement, redelivery, or committed-watermark behavior."
    kind: regression
    risk: medium
    verify: cargo test -p relay --test work_queue_throughput --test perf_gate
  measured_gate_fails_closed:
    id: R1
    text: "A release-mode child producer runs the fsync-always durable publish and lease/ack lifecycle, emits non-zero machine-readable observations, and an independent parent oracle rejects missing or zero samples before enforcing pinned throughput and latency floors."
    kind: regression
    risk: high
    verify: cargo test --release -p relay --test measured_performance measured_durable_lifecycle_gate -- --exact --ignored --nocapture
  runtime_tool_dimensions_are_covered:
    id: R2
    text: "The competitor-performance EC supplies executable behavior, efficiency, and stability cases with specific observable assertions."
    kind: functional
    risk: high
    verify: aw ec check --project relay
  vat_meter_runs_measured_gate:
    id: R3
    text: "The vat-isolated meter-perf runner executes the release-mode measured gate and not only synthetic in-memory correctness tests."
    kind: integration
    risk: high
    verify: cd apps/relay && ../../target/debug/vat run meter-perf
---
flowchart TD
    r1[R1 measured gate fails closed] --> cargo_test_release_p_relay_test_measured_performance_measured_durable_lifecycle_gate_exact_ignored_nocapture[cargo test --release -p relay --test measured_performance measured_durable_lifecycle_gate -- --exact --ignored --nocapture]
    r2[R2 runtime tool dimensions are covered] --> aw_ec_check_project_relay[aw ec check --project relay]
    r3[R3 vat meter runs measured gate] --> cd_apps_relay_target_debug_vat_run_meter_perf[cd apps/relay && ../../target/debug/vat run meter-perf]
    r4[R4 existing work queue behavior remains correct] --> cargo_test_p_relay_test_work_queue_throughput_test_perf_gate[cargo test -p relay --test work_queue_throughput --test perf_gate]
```
