---
id: keep-service-http-adoption
summary: >
  Converge Keep's k8s-native HTTP shell onto libs/service-http for standard
  probes, request tracing, and graceful drain while preserving Keep's current
  OpenAPI inventory and probe/auth behavior.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: keep-service-http-adoption-contract
entry: serve_boot
nodes:
  serve_boot: { kind: start, label: "keep serve builds KvEngine and AppState" }
  data_plane: { kind: process, label: "http::routes::router builds auth/body-limited data-plane and admin routes" }
  shared_probes: { kind: process, label: "service_http::standard_probe_routes serves healthz/readyz/metrics/openapi/docs" }
  merge: { kind: process, label: "merge shared probes with keep data plane and local info/cluster admin routes" }
  trace: { kind: process, label: "service_http::trace_layer emits request spans" }
  serve: { kind: process, label: "HTTP/1.1 + h2c listener serves the merged app" }
  signal: { kind: process, label: "service_http::shutdown_with_drain waits for SIGINT/SIGTERM" }
  drain: { kind: process, label: "AppState.start_drain flips readiness source" }
  ready_503: { kind: process, label: "shared readyz reports 503 draining during grace window" }
  stop: { kind: terminal, label: "grace expires and listener closes" }
edges:
  - { from: serve_boot, to: data_plane }
  - { from: data_plane, to: shared_probes }
  - { from: shared_probes, to: merge }
  - { from: merge, to: trace }
  - { from: trace, to: serve }
  - { from: serve, to: signal }
  - { from: signal, to: drain }
  - { from: drain, to: ready_503 }
  - { from: ready_503, to: stop }
---
flowchart TD
    serve_boot([keep serve builds KvEngine and AppState]) --> data_plane[http::routes::router builds auth/body-limited data-plane and admin routes]
    data_plane --> shared_probes[service_http::standard_probe_routes serves healthz readyz metrics openapi docs]
    shared_probes --> merge[merge shared probes with keep data plane and local info/cluster admin routes]
    merge --> trace[service_http::trace_layer emits request spans]
    trace --> serve[HTTP/1.1 + h2c listener serves merged app]
    serve --> signal[service_http::shutdown_with_drain waits for SIGINT/SIGTERM]
    signal --> drain[AppState.start_drain flips readiness source]
    drain --> ready_503[shared readyz reports 503 draining during grace window]
    ready_503 --> stop([grace expires and listener closes])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: keep-service-http-adoption-tests
requirements:
  shared_ready_ok:
    id: R1
    text: "The shared probe router returns 200 ok for readyz while the readiness source is not draining."
    kind: behavior
    risk: medium
    verify: test
  shared_ready_draining:
    id: R2
    text: "The shared probe router returns 503 draining once the readiness source is draining."
    kind: behavior
    risk: medium
    verify: test
  shared_metrics:
    id: R3
    text: "The shared probe router returns Keep's Prometheus metrics text rendered by the supplied metrics provider."
    kind: behavior
    risk: medium
    verify: test
  keep_openapi_inventory:
    id: R4
    text: "Keep's served OpenAPI and probe surface keep standard probe paths and data-plane paths."
    kind: behavior
    risk: medium
    verify: test
elements:
  service_http_unit_tests:
    kind: test
    path: libs/service-http/src/probes.rs
  keep_http_api:
    kind: test
    path: projects/keep/tests/http_api.rs
relations:
  - { from: service_http_unit_tests, verifies: shared_ready_ok }
  - { from: service_http_unit_tests, verifies: shared_ready_draining }
  - { from: service_http_unit_tests, verifies: shared_metrics }
  - { from: keep_http_api, verifies: keep_openapi_inventory }
  - { from: keep_http_api, verifies: shared_ready_draining }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "readyz ok"
      risk: medium
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "readyz draining"
      risk: medium
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "metrics provider"
      risk: medium
      verifymethod: test
    }
    requirement R4 {
      id: R4
      text: "OpenAPI inventory"
      risk: medium
      verifymethod: test
    }
    element service_http_unit_tests {
      type: "rs/#[test]"
    }
    element keep_http_api {
      type: "rs/#[tokio::test]"
    }
    service_http_unit_tests - verifies -> R1
    service_http_unit_tests - verifies -> R2
    service_http_unit_tests - verifies -> R3
    keep_http_api - verifies -> R4
    keep_http_api - verifies -> R2
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/keep/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add the service-http dependency to Keep."
  - path: projects/keep/src/http/mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Implement service_http::ReadinessHook and MetricsProvider for AppState so the shared probe router reports Keep readiness and Prometheus metrics."
  - path: projects/keep/src/http/routes.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Build the router from service_http::standard_probe_routes for healthz/readyz/metrics/openapi/docs, merged with Keep's auth/body-limited data plane and local info/cluster admin routes; drop the hand-rolled probe routes."
  - path: projects/keep/src/bin/keep.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Serve the merged app and drain through service_http::serve, trace_layer, and shutdown_with_drain instead of the local hyper auto::Builder, GracefulShutdown, and signal handling."
  - path: projects/keep/tests/http_api.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Assert the shared probe contract: healthz/readyz/metrics/openapi inventory preserved and readyz draining returns 503."
```
