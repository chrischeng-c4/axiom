---
id: '2415'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: loom-telemetry-contract
entry: input
nodes:
  input:
    kind: start
    label: long-running role and optional W3C request context
  valid:
    kind: decision
    label: logging configuration valid
  record:
    kind: process
    label: schema-valid JSONL request completion record
  reject:
    kind: terminal
    label: startup error names invalid LOOM_LOG_FORMAT
  done:
    kind: terminal
    label: Sift-independent producer contract satisfied
edges:
  - { from: input, to: valid }
  - { from: valid, to: record, label: valid }
  - { from: valid, to: reject, label: invalid }
  - { from: record, to: done }
---
flowchart TD
  input[service role plus optional traceparent] --> valid{valid LOOM_LOG_FORMAT}
  valid -->|json or pretty| record[schema-valid correlated JSONL]
  valid -->|other value| reject[startup error]
  record --> done[producer remains Sift agnostic]
```

The contract is stable across Loom roles. JSON mode produces axiom.service.log.v1 records with service.name=loom; every HTTP response yields one shared http_request_complete event after the response. Valid W3C input preserves trace_id, parent_span_id, and trace_flags while creating a local span id; missing or malformed input yields a valid root record. Pretty mode retains human local logs. Invalid logging format is rejected before a role starts. Optional OTLP is configured only through the shared initializer.

The negative contract is equally important: Loom has no Sift crate dependency and accepts no Sift URL, token, header, or collector configuration. JSONL stays on normal process stdout for a collector supplied and operated by Sift. Replica count, raft state, workflow dispatch, payload ownership, and controller routes are unchanged.
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/service-http/src/transport.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: pub fn trace_layer
    description: Emit one W3C-correlated http_request_complete JSONL event through the shared response hook so all shared HTTP services have the same generic request evidence.
  - path: libs/service-http/Cargo.toml
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: Add the existing formatter test dependency needed to decode the shared completion event contract.
  - path: libs/service-http/tests/request_completion_event.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Exercise the shared router and JSON formatter for valid, absent, and malformed W3C request context.
  - path: apps/loom/src/main.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: fn main
    description: Initialize shared service tracing only for long-running Loom roles from LOOM_LOG_FORMAT, LOOM_LOG_LEVEL, and optional LOOM_OTLP_ENDPOINT.
  - path: apps/loom/src/operator/render.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: fn extra_env
    description: Set collector-ready LOOM_LOG_FORMAT=json in the operator-rendered service workload environment.
  - path: apps/loom/k8s/base/statefulset.yaml
    action: modify
    section: logic
    impl_mode: hand-written
    description: Set LOOM_LOG_FORMAT=json in the checked-in controller workload base.
  - path: apps/loom/tests/structured_stdout_traceparent.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Spawn the real controller, exercise valid invalid and absent traceparent requests, and assert schema identity W3C correlation and no direct Sift linkage.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: loom-sift-agnostic-telemetry-verification
requirements:
  collector_boundary:
    id: R3
    text: "Loom adds neither a Sift dependency nor Sift endpoint, token, header, route, or collector configuration; collection remains external."
    kind: negative
    risk: medium
    verify: apps/loom/tests/structured_stdout_traceparent.rs::loom_remains_sift_agnostic
  existing_controller_surface:
    id: R5
    text: "The existing shared standard endpoint surface remains available after tracing initialization is moved to the process boundary."
    kind: regression
    risk: medium
    verify: apps/loom/src/controller.rs::tests::standard_endpoints_served
  real_controller_jsonl:
    id: R2
    text: "A real Loom controller under LOOM_LOG_FORMAT=json emits a nonzero service.name=loom JSONL completion record for valid, missing, and malformed traceparent requests."
    kind: functional
    risk: high
    verify: apps/loom/tests/structured_stdout_traceparent.rs::real_loom_controller_correlates_structured_stdout
  shared_completion:
    id: R1
    text: "The shared HTTP trace layer emits exactly one axiom.service.log.v1 http_request_complete event with method, URI, response status, latency, and inherited W3C context."
    kind: functional
    risk: high
    verify: libs/service-http/tests/request_completion_event.rs::completion_event_is_schema_valid_and_w3c_correlated
  workload_configuration:
    id: R4
    text: "Both checked-in and operator-rendered controller workloads request JSON service logging without changing replica or workflow semantics."
    kind: regression
    risk: medium
    verify: apps/loom/tests/structured_stdout_traceparent.rs::loom_workloads_request_json_logging
---
flowchart TD
    r1[R1 shared completion] --> libs_service_http_tests_request_completion_event_rs_completion_event_is_schema_valid_and_w3c_correlated[libs/service-http/tests/request_completion_event.rs::completion_event_is_schema_valid_and_w3c_correlated]
    r2[R2 real controller jsonl] --> apps_loom_tests_structured_stdout_traceparent_rs_real_loom_controller_correlates_structured_stdout[apps/loom/tests/structured_stdout_traceparent.rs::real_loom_controller_correlates_structured_stdout]
    r3[R3 collector boundary] --> apps_loom_tests_structured_stdout_traceparent_rs_loom_remains_sift_agnostic[apps/loom/tests/structured_stdout_traceparent.rs::loom_remains_sift_agnostic]
    r4[R4 workload configuration] --> apps_loom_tests_structured_stdout_traceparent_rs_loom_workloads_request_json_logging[apps/loom/tests/structured_stdout_traceparent.rs::loom_workloads_request_json_logging]
    r5[R5 existing controller surface] --> apps_loom_src_controller_rs_tests_standard_endpoints_served[apps/loom/src/controller.rs::tests::standard_endpoints_served]
```
