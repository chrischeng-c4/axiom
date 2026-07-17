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
    label: "version, delimiter positions, lowercase hex lengths, and nonzero ids are valid?"
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
    label: "attach the same parsed context to OpenTelemetry without changing upstream trace identity"
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
    parse --> valid{valid W3C version 00 context?}
    valid -->|yes| child[preserve trace id; create local child span]
    valid -->|no or absent| root[create fresh local root trace/span]
    child --> fields[record canonical correlation fields]
    root --> fields
    fields --> otlp{OTLP layer installed?}
    otlp -->|yes| attach[attach same upstream context]
    otlp -->|no| route([route request])
    attach --> route
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/service-http/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: Add the small OS-random dependency used to create nonzero trace and span ids when no OpenTelemetry layer exists.
  - path: libs/service-http/src/trace_context.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: Parse and canonicalize W3C version 00 traceparent, reject malformed or zero ids, and create exporter-independent root or child request correlation.
  - path: libs/service-http/src/transport.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Replace the feature-gated-only make-span path with an always-on request context and record canonical correlation fields while preserving optional OTLP parent attachment.
  - path: libs/service-http/src/lib.rs
    action: modify
    section: contract
    impl_mode: hand-written
    description: Export the stable request trace-context types and parsing surface for service adopters and tests.
  - path: libs/service-http/README.md
    action: modify
    section: contract
    impl_mode: hand-written
    description: Document always-on W3C server context, safe invalid-header fallback, field shape, and the independent OTLP exporter boundary.
  - path: libs/service-http/external-contracts/behavior/shared-http-service-scaffold-contract.md
    action: modify
    section: contract
    impl_mode: hand-written
    description: Close the shared request-correlation claim for logging-only and OTLP-enabled builds.
  - path: libs/service-http/tests/request_trace_context.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Prove valid parent preservation, local id generation, malformed and zero-id fallback, request routing, and no-OTLP behavior.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: exporter-independent-w3c-server-context-verification
requirements:
  invalid_or_missing_context_is_safe:
    id: R2
    text: "Missing, malformed, unsupported-version, wrong-length, invalid-hex, or all-zero traceparent values create a fresh valid root trace/span and do not reject the HTTP request."
    kind: negative
    risk: high
    verify: cargo test -p service-http --test request_trace_context invalid_or_missing_traceparent_creates_safe_root -- --exact
  otlp_is_additive:
    id: R4
    text: "Enabling the OTLP feature attaches valid propagated context but preserves the same upstream trace identity and the structured request correlation contract."
    kind: regression
    risk: high
    verify: cargo test -p service-http --features otlp --test request_trace_context otlp_feature_preserves_request_trace_identity -- --exact
  request_span_exposes_correlation:
    id: R3
    text: "The shared trace layer records canonical trace_id, span_id, optional parent_span_id, and trace flags on the request span while preserving normal routing in a logging-only build."
    kind: contract
    risk: high
    verify: cargo test -p service-http --test request_trace_context trace_layer_records_context_and_routes_without_otlp -- --exact
  shared_scaffold_remains_green:
    id: R5
    text: "The existing shared HTTP scaffold and prior optional OTLP behavior remain compatible after context parsing moves outside the exporter feature."
    kind: regression
    risk: medium
    verify: cargo test -p service-http
  valid_parent_creates_local_child:
    id: R1
    text: "A valid W3C version 00 traceparent preserves the upstream trace id, records its parent span id, and creates a distinct nonzero local server span id without compiling OTLP."
    kind: functional
    risk: high
    verify: cargo test -p service-http --test request_trace_context valid_traceparent_preserves_trace_and_creates_child_span -- --exact
---
flowchart TD
    r1[R1 valid parent creates local child] --> cargo_test_p_service_http_test_request_trace_context_valid_traceparent_preserves_trace_and_creates_child_span_exact[cargo test -p service-http --test request_trace_context valid_traceparent_preserves_trace_and_creates_child_span -- --exact]
    r2[R2 invalid or missing context is safe] --> cargo_test_p_service_http_test_request_trace_context_invalid_or_missing_traceparent_creates_safe_root_exact[cargo test -p service-http --test request_trace_context invalid_or_missing_traceparent_creates_safe_root -- --exact]
    r3[R3 request span exposes correlation] --> cargo_test_p_service_http_test_request_trace_context_trace_layer_records_context_and_routes_without_otlp_exact[cargo test -p service-http --test request_trace_context trace_layer_records_context_and_routes_without_otlp -- --exact]
    r4[R4 otlp is additive] --> cargo_test_p_service_http_features_otlp_test_request_trace_context_otlp_feature_preserves_request_trace_identity_exact[cargo test -p service-http --features otlp --test request_trace_context otlp_feature_preserves_request_trace_identity -- --exact]
    r5[R5 shared scaffold remains green] --> cargo_test_p_service_http[cargo test -p service-http]
```
