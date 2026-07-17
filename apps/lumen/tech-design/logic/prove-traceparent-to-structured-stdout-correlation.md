---
id: '1871'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-structured-stdout-traceparent-conformance
entry: start_lumen
nodes:
  start_lumen:
    kind: start
    label: "spawn real lumen serve with embedded WAL, JSON format, and no OTLP endpoint"
  ready:
    kind: process
    label: "wait for shared health endpoint while continuously draining stdout"
  request_kind:
    kind: decision
    label: "valid, invalid, or missing traceparent?"
  valid:
    kind: process
    label: "PUT a collection with fixed W3C trace and parent span ids"
  invalid:
    kind: process
    label: "PUT a collection with malformed traceparent"
  missing:
    kind: process
    label: "PUT a collection without traceparent"
  audit:
    kind: process
    label: "Lumen collection audit event executes inside the shared request span"
  capture:
    kind: process
    label: "stop process and parse every captured stdout line as axiom.service.log.v1"
  assert_valid:
    kind: terminal
    label: "audit event preserves inbound trace id and parent while creating a distinct local span id"
  assert_local:
    kind: terminal
    label: "request succeeds and audit event carries valid locally generated correlation"
edges:
  - { from: start_lumen, to: ready }
  - { from: ready, to: request_kind }
  - { from: request_kind, to: valid, label: "valid" }
  - { from: request_kind, to: invalid, label: "invalid" }
  - { from: request_kind, to: missing, label: "missing" }
  - { from: valid, to: audit }
  - { from: invalid, to: audit }
  - { from: missing, to: audit }
  - { from: audit, to: capture }
  - { from: capture, to: assert_valid, label: "valid request" }
  - { from: capture, to: assert_local, label: "invalid or missing" }
---
flowchart TD
    start[spawn lumen serve: JSON, embedded WAL, no OTLP] --> ready[wait for health; drain stdout]
    ready --> request{traceparent case}
    request -->|valid| valid[fixed trace and parent ids]
    request -->|invalid| invalid[malformed header]
    request -->|missing| missing[no header]
    valid --> audit[Lumen audit event inside request span]
    invalid --> audit
    missing --> audit
    audit --> parse[parse every stdout line]
    parse --> correlated([valid input preserves trace and parent])
    parse --> local([invalid or missing gets safe local root])
```

Lumen remains independent of Sift. The existing outer router owns the shared `service_http::trace_layer()`, and the existing `collection_create_or_extend` audit event provides a real domain event inside that request span. The conformance process explicitly selects collector mode, removes `LUMEN_OTLP_ENDPOINT` and `RUST_LOG`, sends three independent collection requests, and treats any non-JSON stdout line as failure.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/tests/structured_stdout_traceparent.rs
    action: create
    section: e2e-test
    impl_mode: hand-written
    description: Spawn the real Lumen binary without OTLP, drive valid, invalid, and missing traceparent requests, drain stdout, and assert versioned JSONL correlation.
```

No Lumen-local formatter, collector client, or request middleware is added. The adopter relies on the existing `service_http::init_tracing_with_identity` mapping, outer `service_http::trace_layer()`, and audit event; this WI is a process-level conformance proof for shared WIs #1868 and #1870.

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-structured-stdout-traceparent-verification
requirements:
  independence:
    id: R5
    text: "The conformance process has no OTLP endpoint or Sift dependency and produces stdout consumable as independent JSONL records."
    kind: integration
    risk: medium
    verify: cargo test -p lumen --test structured_stdout_traceparent real_lumen_process_correlates_structured_stdout -- --exact
  invalid_missing:
    id: R4
    text: "Malformed and missing traceparent requests both succeed and emit valid locally rooted correlation."
    kind: regression
    risk: high
    verify: cargo test -p lumen --test structured_stdout_traceparent real_lumen_process_correlates_structured_stdout -- --exact
  outer_trace_layer:
    id: R2
    text: "The public collection route audit event runs inside the shared request span rather than a Lumen-local propagation implementation."
    kind: contract
    risk: high
    verify: cargo test -p lumen --test structured_stdout_traceparent real_lumen_process_correlates_structured_stdout -- --exact
  structured_mode:
    id: R1
    text: "A real Lumen process uses the shared JSON formatter and every captured operational stdout line conforms to axiom.service.log.v1."
    kind: functional
    risk: high
    verify: cargo test -p lumen --test structured_stdout_traceparent real_lumen_process_correlates_structured_stdout -- --exact
  valid_parent:
    id: R3
    text: "A fixed valid traceparent preserves its trace and parent span ids while Lumen creates a distinct nonzero local span id."
    kind: functional
    risk: high
    verify: cargo test -p lumen --test structured_stdout_traceparent real_lumen_process_correlates_structured_stdout -- --exact
---
flowchart TD
    r1[R1 structured mode] --> cargo_test_p_lumen_test_structured_stdout_traceparent_real_lumen_process_correlates_structured_stdout_exact[cargo test -p lumen --test structured_stdout_traceparent real_lumen_process_correlates_structured_stdout -- --exact]
    r2[R2 outer trace layer] --> cargo_test_p_lumen_test_structured_stdout_traceparent_real_lumen_process_correlates_structured_stdout_exact
    r3[R3 valid parent] --> cargo_test_p_lumen_test_structured_stdout_traceparent_real_lumen_process_correlates_structured_stdout_exact
    r4[R4 invalid missing] --> cargo_test_p_lumen_test_structured_stdout_traceparent_real_lumen_process_correlates_structured_stdout_exact
    r5[R5 independence] --> cargo_test_p_lumen_test_structured_stdout_traceparent_real_lumen_process_correlates_structured_stdout_exact
```
