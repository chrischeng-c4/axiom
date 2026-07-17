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
