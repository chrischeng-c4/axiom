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
    label: "service supplies ObservabilityConfig and stable ServiceIdentity"
  format:
    kind: decision
    label: "collector-compatible JSON or explicit development pretty output?"
  pretty:
    kind: terminal
    label: "human-readable output; not a collector contract"
  json_layer:
    kind: process
    label: "install axiom.service.log.v1 formatter on stdout"
  capture_event:
    kind: process
    label: "record event fields and entered span fields"
  sanitize:
    kind: process
    label: "extract reserved correlation fields; drop sensitive propagation keys; bound remaining attributes"
  encode:
    kind: process
    label: "serialize one ServiceLogEventV1 object and append exactly one newline"
  otlp:
    kind: decision
    label: "valid optional OTLP exporter available?"
  export:
    kind: process
    label: "attach trace exporter without changing stdout schema"
  logging_only:
    kind: terminal
    label: "structured stdout remains complete without exporter"
  correlated:
    kind: terminal
    label: "structured stdout and optional exported span share trace identity"
edges:
  - { from: init_observability, to: format }
  - { from: format, to: pretty, label: "pretty" }
  - { from: format, to: json_layer, label: "json" }
  - { from: json_layer, to: capture_event }
  - { from: capture_event, to: sanitize }
  - { from: sanitize, to: encode }
  - { from: encode, to: otlp }
  - { from: otlp, to: export, label: "yes" }
  - { from: otlp, to: logging_only, label: "no" }
  - { from: export, to: correlated }
---
flowchart TD
    init[config plus service identity] --> format{output format}
    format -->|pretty| dev([human-readable development output])
    format -->|json| capture[capture event plus active span fields]
    capture --> safe[extract correlation; redact and bound attributes]
    safe --> line[serialize axiom.service.log.v1 plus newline]
    line --> otlp{OTLP available?}
    otlp -->|no| local([complete structured stdout])
    otlp -->|yes| exported([same trace identity in stdout and exporter])
```

The collector contract is exporter-independent. Reserved top-level fields are `schema`, `timestamp`, `severity`, `service`, `event`, `message`, `trace_id`, `span_id`, `parent_span_id`, `trace_flags`, `request_id`, and `attributes`. Correlation is inherited from the nearest entered span, with event fields taking precedence only when they pass the documented lowercase-hex validation. Authorization, cookie, baggage, and tracestate-like keys are excluded before bounded attributes are serialized.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/service-observability/src/jsonl.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: Define the public axiom.service.log.v1 event model, correlation validation, sensitive-field exclusion, bounded attributes, and tracing-subscriber JSONL formatter.
  - path: libs/service-observability/src/logging.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Install the versioned JSONL formatter for collector-compatible mode while preserving explicit pretty output and optional OTLP composition.
  - path: libs/service-observability/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Export the versioned event contract and formatter surfaces for service adopters and collectors.
  - path: libs/service-observability/contracts/axiom.service.log.v1.schema.json
    action: create
    section: logic
    impl_mode: hand-written
    description: Publish the machine-readable versioned stdout event schema and its required and prohibited-by-default fields.
  - path: libs/service-observability/tests/service_log_jsonl.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Verify independent-line parsing, service identity, span correlation inheritance, validation, redaction, bounds, no-context behavior, and no-OTLP operation.
```

The bounded hand-authored implementation also adds direct `serde` and `serde_json` dependencies to `libs/service-observability/Cargo.toml`; the contract test makes the static schema and Rust event surface drift together.
