---
id: '1870'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: exporter-independent-w3c-server-context
entry: receive_request
nodes:
  receive_request:
    kind: start
    label: "service-http receives an HTTP request in every feature build"
  parse_traceparent:
    kind: process
    label: "parse W3C version 00 traceparent without requiring OTLP"
  parent_valid:
    kind: decision
    label: "version, lengths, lowercase or uppercase hex, and nonzero ids are valid?"
  child_context:
    kind: process
    label: "preserve upstream trace id and create a distinct local server span id with parent_span_id"
  root_context:
    kind: process
    label: "create a fresh nonzero local root trace and server span"
  record_fields:
    kind: process
    label: "record canonical trace_id, span_id, optional parent_span_id, and flags on the request span"
  otlp_enabled:
    kind: decision
    label: "optional OTLP layer installed?"
  attach_otel_parent:
    kind: process
    label: "attach the same parsed context to OpenTelemetry without changing ids"
  route_request:
    kind: terminal
    label: "route request normally; structured logging can read correlation fields"
edges:
  - { from: receive_request, to: parse_traceparent }
  - { from: parse_traceparent, to: parent_valid }
  - { from: parent_valid, to: child_context, label: "yes" }
  - { from: parent_valid, to: root_context, label: "no or absent" }
  - { from: child_context, to: record_fields }
  - { from: root_context, to: record_fields }
  - { from: record_fields, to: otlp_enabled }
  - { from: otlp_enabled, to: attach_otel_parent, label: "yes" }
  - { from: otlp_enabled, to: route_request, label: "no" }
  - { from: attach_otel_parent, to: route_request }
---
flowchart TD
    request[HTTP request] --> parse[parse traceparent without OTLP]
    parse --> valid{valid version 00 context?}
    valid -->|yes| child[preserve trace id; create local child span]
    valid -->|no or absent| root[create fresh local root trace/span]
    child --> fields[record canonical correlation fields]
    root --> fields
    fields --> otlp{OTLP layer installed?}
    otlp -->|yes| attach[attach same parent context]
    otlp -->|no| route([route request])
    attach --> route
```
