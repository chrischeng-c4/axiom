---
id: '1868'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: versioned-service-jsonl-stdout
entry: init_observability
nodes:
  init_observability:
    kind: start
    label: "service supplies ObservabilityConfig and validated ServiceIdentity"
  format:
    kind: decision
    label: "LogFormat is Json?"
  pretty:
    kind: terminal
    label: "emit explicit human-readable development output; collector compatibility is false"
  json_layer:
    kind: process
    label: "install JsonFields plus ServiceJsonFormatter on stdout"
  event_fields:
    kind: process
    label: "record event values and walk entered spans from root to leaf"
  correlation:
    kind: process
    label: "select nearest valid trace_id, span_id, parent_span_id, trace_flags, and request_id"
  sanitize:
    kind: process
    label: "remove reserved and sensitive keys; keep at most 64 attributes with bounded key and value sizes"
  event:
    kind: process
    label: "construct ServiceLogEventV1 with RFC3339 UTC timestamp, severity, identity, event, message, correlation, and attributes"
  line:
    kind: process
    label: "serde_json serialize one object then append one newline"
  otlp:
    kind: decision
    label: "optional OTLP tracer constructed?"
  logging_only:
    kind: terminal
    label: "stdout event remains complete without exporter"
  shared_identity:
    kind: terminal
    label: "stdout and exporter use the same active trace identity"
edges:
  - { from: init_observability, to: format }
  - { from: format, to: pretty, label: "no" }
  - { from: format, to: json_layer, label: "yes" }
  - { from: json_layer, to: event_fields }
  - { from: event_fields, to: correlation }
  - { from: correlation, to: sanitize }
  - { from: sanitize, to: event }
  - { from: event, to: line }
  - { from: line, to: otlp }
  - { from: otlp, to: logging_only, label: "no" }
  - { from: otlp, to: shared_identity, label: "yes" }
---
flowchart TD
    init[validated config plus service identity] --> format{JSON collector mode?}
    format -->|no| pretty([development-only pretty output])
    format -->|yes| fields[event fields plus root-to-leaf span fields]
    fields --> correlation[validate and select nearest correlation fields]
    correlation --> sanitize[drop secrets; bound 64 attributes]
    sanitize --> event[build axiom.service.log.v1 event]
    event --> line[serialize one object plus one newline]
    line --> otlp{OTLP tracer present?}
    otlp -->|no| stdout([complete stdout correlation])
    otlp -->|yes| shared([stdout and exporter share trace identity])
```

Contract invariants:

- `schema` is exactly `axiom.service.log.v1`; `timestamp` is RFC3339 UTC; `severity` is the tracing level; `service.name` and `service.version` come only from validated `ServiceIdentity`.
- `event` uses a bounded explicit event field when present and otherwise tracing metadata name. `message` is always present and defaults to the event name.
- Entered span fields are merged root-to-leaf, then event fields are considered. Reserved correlation fields are promoted only when valid: trace ids are 32 lowercase hex, span and parent ids are 16 lowercase hex, flags are two lowercase hex, and request ids are non-empty bounded strings. Invalid values are omitted, never copied into attributes.
- `authorization`, `proxy-authorization`, `cookie`, `set-cookie`, `baggage`, and `tracestate` field spellings, including normalized HTTP header attribute paths, are excluded. The formatter keeps at most 64 non-reserved attributes, limits keys to 128 bytes and string/debug values to 4096 bytes, and emits deterministic key order.
- Serialization failure returns `fmt::Error`; it never emits a partial JSON prefix. Each successful event is a single compact JSON object followed by exactly one newline.
- Pretty formatting remains available only through explicit `LogFormat::Pretty`; `LogFormat::Json` is the sole collector-compatible mode. OTLP availability never changes the JSON schema.
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/service-observability/src/jsonl.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: Define ServiceLogEventV1, stable schema constants, correlation validation, sensitive-key exclusion, bounded attributes, and the tracing-subscriber JSONL formatter.
  - path: libs/service-observability/src/logging.rs
    action: modify
    anchor: init_tracing_with_identity
    section: logic
    impl_mode: hand-written
    description: Compose JsonFields and ServiceJsonFormatter for LogFormat::Json while retaining explicit pretty output and optional OTLP layering.
  - path: libs/service-observability/contracts/axiom.service.log.v1.schema.json
    action: create
    section: logic
    impl_mode: hand-written
    description: Define the machine-readable required, optional, nested service, correlation-pattern, attribute-bound, and additional-property rules.
  - path: libs/service-observability/tests/service_log_jsonl.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Capture real tracing output and verify per-line framing, stable schema, inherited correlation, validation, redaction, bounds, static-schema drift, and exporter independence.
```

`libs/service-observability/Cargo.toml` gains the direct workspace `serde` and `serde_json` dependencies required by the public wire type and formatter. Those declarative dependency entries are covered by compiling the generated target set rather than receiving source-ownership markers.
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: versioned-service-jsonl-stdout-verification
requirements:
  composition:
    id: R5
    text: "All service-observability tests pass in logging-only mode and with the optional OTLP feature without changing the JSON contract; pretty remains explicitly non-collector-compatible."
    kind: regression
    risk: medium
    verify: cargo test -p service-observability
  correlation:
    id: R3
    text: "Nearest active span correlation is promoted only when ids are valid and works in the default no-OTLP build."
    kind: functional
    risk: high
    verify: cargo test -p service-observability --test service_log_jsonl active_span_fields_become_valid_correlation -- --exact
  framing_identity:
    id: R2
    text: "Two real tracing events produce two independently parseable compact JSON lines with timestamp, severity, service identity, event, message, and bounded attributes."
    kind: functional
    risk: high
    verify: cargo test -p service-observability --test service_log_jsonl jsonl_lines_parse_independently_with_identity -- --exact
  safety_bounds:
    id: R4
    text: "Sensitive and invalid reserved fields are omitted, no more than 64 attributes survive, values are bounded, and JSON framing stays valid."
    kind: security
    risk: high
    verify: cargo test -p service-observability --test service_log_jsonl sensitive_and_oversized_attributes_are_safe -- --exact
  wire_contract:
    id: R1
    text: "ServiceLogEventV1 and the static schema identify axiom.service.log.v1 and reject drift in required and optional fields."
    kind: contract
    risk: high
    verify: cargo test -p service-observability --test service_log_jsonl schema_contract_matches_rust_event -- --exact
---
flowchart TD
    r1[R1 wire contract] --> cargo_test_p_service_observability_test_service_log_jsonl_schema_contract_matches_rust_event_exact[cargo test -p service-observability --test service_log_jsonl schema_contract_matches_rust_event -- --exact]
    r2[R2 framing identity] --> cargo_test_p_service_observability_test_service_log_jsonl_jsonl_lines_parse_independently_with_identity_exact[cargo test -p service-observability --test service_log_jsonl jsonl_lines_parse_independently_with_identity -- --exact]
    r3[R3 correlation] --> cargo_test_p_service_observability_test_service_log_jsonl_active_span_fields_become_valid_correlation_exact[cargo test -p service-observability --test service_log_jsonl active_span_fields_become_valid_correlation -- --exact]
    r4[R4 safety bounds] --> cargo_test_p_service_observability_test_service_log_jsonl_sensitive_and_oversized_attributes_are_safe_exact[cargo test -p service-observability --test service_log_jsonl sensitive_and_oversized_attributes_are_safe -- --exact]
    r5[R5 composition] --> cargo_test_p_service_observability[cargo test -p service-observability]
```
