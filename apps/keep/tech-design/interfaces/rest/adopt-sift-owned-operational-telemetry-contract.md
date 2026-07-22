---
id: '2414'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: keep-sift-operational-telemetry-applicability
entry: start
nodes:
  start:
    kind: start
    label: Keep process starts with service-owned logging options
  resolve:
    kind: process
    label: Resolve log level, log format, and optional OTLP endpoint from Keep configuration
  shared:
    kind: process
    label: Compose libs/service-observability with identity keep and the package version
  stdout:
    kind: process
    label: JSON mode emits axiom.service.log.v1 lines to stdout with W3C-compatible request correlation
  collector:
    kind: process
    label: Sift-owned collector reads stdout and attaches routing, credentials, and delivery policy outside Keep
  query:
    kind: process
    label: VAT starts a real Keep process, collects records through Sift, and queries durable Sift evidence
  done:
    kind: terminal
    label: Keep remains Sift-agnostic while its operational events are queryable by stable service identity
edges:
  - { from: start, to: resolve }
  - { from: resolve, to: shared }
  - { from: shared, to: stdout }
  - { from: stdout, to: collector }
  - { from: collector, to: query }
  - { from: query, to: done }
---
flowchart TD
  start[Keep process starts with service-owned logging options] --> resolve[Resolve log level, log format, and optional OTLP endpoint]
  resolve --> shared[Compose shared service-observability with identity keep]
  shared --> stdout[JSON mode emits axiom.service.log.v1 stdout records with W3C correlation]
  stdout --> collector[Sift-owned collector performs routing and delivery outside Keep]
  collector --> query[VAT starts Keep, collects with Sift, and queries durable evidence]
  query --> done[Keep stays Sift-agnostic and events remain queryable]
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
