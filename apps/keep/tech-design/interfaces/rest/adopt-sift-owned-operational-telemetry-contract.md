---
id: '2414'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: keep-sift-operational-telemetry-contract
entry: parse
nodes:
  parse:
    kind: start
    label: Parse Keep server options and environment
  config:
    kind: process
    label: Map pretty or json log format and optional OTLP endpoint into shared HttpConfig
  identity:
    kind: process
    label: Initialize shared tracing once with service identity keep and package version before startup logs
  request:
    kind: process
    label: Existing service-http trace layer records canonical W3C correlation on HTTP request spans
  jsonl:
    kind: process
    label: Shared JSON formatter writes bounded axiom.service.log.v1 records to stdout
  external:
    kind: process
    label: Deployment-owned Sift collector tails stdout and owns endpoint, auth, batching, retry, and ingestion routing
  proof:
    kind: terminal
    label: Local real-process contract and Sift VAT ingestion query both pass without a Keep to Sift dependency
edges:
  - { from: parse, to: config }
  - { from: config, to: identity }
  - { from: identity, to: request }
  - { from: request, to: jsonl }
  - { from: jsonl, to: external }
  - { from: external, to: proof }
---
flowchart TD
  parse[Parse Keep server options and environment] --> config[Map log format and optional OTLP endpoint into shared HttpConfig]
  config --> identity[Initialize shared tracing with keep identity before startup logs]
  identity --> request[Existing service-http layer records canonical W3C request correlation]
  request --> jsonl[Shared JSON formatter writes axiom.service.log.v1 to stdout]
  jsonl --> external[Deployment-owned Sift collector owns transport and delivery]
  external --> proof[Local producer and Sift VAT ingestion proofs pass without direct coupling]
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/keep/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: Add Keep's opt-in otel feature as a thin forwarding feature to service-http/otlp; the default build stays structured-log-only.
  - path: apps/keep/src/bin/keep.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: ServeArgs
    description: Add KEEP_LOG_FORMAT and KEEP_OTLP_ENDPOINT CLI/environment configuration, map Keep's values to service_http::HttpConfig and ServiceIdentity, and replace the local tracing-subscriber initialization in the default server path with shared observability initialization.
  - path: apps/keep/tests/structured_stdout_traceparent.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Start the compiled Keep server with JSON logging, issue valid, invalid, and absent traceparent HTTP requests, capture stdout, and assert axiom.service.log.v1 identity, W3C correlation behavior, and no Sift dependency.
  - path: apps/keep/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: Document the standard --log-format and optional KEEP_OTLP_ENDPOINT operating controls, while stating that Sift collector endpoint, credentials, and delivery policy remain deployment-owned.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: keep-sift-operational-telemetry-verification
requirements:
  collector_ingest:
    id: R5
    text: "The Sift-owned VAT operational-log integration starts a real Keep process and can query its accepted durable records by the stable keep identity without rejected events."
    kind: e2e
    risk: high
    verify: cargo test -p sift --test vat_service_observability_e2e
  invalid_or_missing_w3c:
    id: R3
    text: "Invalid or missing traceparent input remains available as a fresh nonzero correlation context and does not fail a Keep HTTP request."
    kind: regression
    risk: medium
    verify: cargo test -p keep --test structured_stdout_traceparent
  sift_agnostic:
    id: R4
    text: "Keep exposes standard producer controls only and has no direct Sift crate dependency or Sift endpoint, credential, or transport configuration."
    kind: boundary
    risk: high
    verify: cargo test -p keep --test structured_stdout_traceparent
  structured_stdout:
    id: R1
    text: "A real default Keep server started with --log-format json writes only axiom.service.log.v1 records to stdout with service.name=keep."
    kind: functional
    risk: medium
    verify: cargo test -p keep --test structured_stdout_traceparent
  valid_w3c:
    id: R2
    text: "A valid W3C traceparent sent to Keep's HTTP data plane preserves its trace id, parent span id, and flags in the correlated structured request event."
    kind: functional
    risk: medium
    verify: cargo test -p keep --test structured_stdout_traceparent
---
flowchart TD
    r1[R1 structured stdout] --> cargo_test_p_keep_test_structured_stdout_traceparent[cargo test -p keep --test structured_stdout_traceparent]
    r2[R2 valid w3c] --> cargo_test_p_keep_test_structured_stdout_traceparent
    r3[R3 invalid or missing w3c] --> cargo_test_p_keep_test_structured_stdout_traceparent
    r4[R4 sift agnostic] --> cargo_test_p_keep_test_structured_stdout_traceparent
    r5[R5 collector ingest] --> cargo_test_p_sift_test_vat_service_observability_e2e[cargo test -p sift --test vat_service_observability_e2e]
```
