---
id: lumen-service-http-convergence
summary: >
  Converge Lumen's k8s-native HTTP shell onto libs/service-http for standard
  probes, request tracing, and graceful drain while preserving Lumen's current
  OpenAPI inventory and OTLP behavior.
fill_sections: [logic, unit-test, e2e-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-service-http-convergence-contract
entry: serve_boot
nodes:
  serve_boot: { kind: start, label: "lumen serve builds Engine and AppState" }
  data_plane: { kind: process, label: "api::router builds auth/body-limited data-plane routes" }
  shared_probes: { kind: process, label: "service_http::standard_probe_routes serves healthz/readyz/metrics/openapi/docs" }
  merge: { kind: process, label: "merge shared probes with Lumen data plane and local admin routes" }
  trace: { kind: process, label: "service_http::trace_layer emits request spans" }
  serve: { kind: process, label: "HTTP/1.1 + h2c listener serves the merged app" }
  signal: { kind: process, label: "service_http::shutdown_with_drain waits for SIGINT/SIGTERM" }
  drain: { kind: process, label: "engine.start_drain flips readiness source" }
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
    serve_boot([lumen serve builds Engine and AppState]) --> data_plane[api::router builds auth/body-limited data-plane routes]
    data_plane --> shared_probes[service_http::standard_probe_routes serves healthz readyz metrics openapi docs]
    shared_probes --> merge[merge shared probes with Lumen data plane and local admin routes]
    merge --> trace[service_http::trace_layer emits request spans]
    trace --> serve[HTTP/1.1 + h2c listener serves merged app]
    serve --> signal[service_http::shutdown_with_drain waits for SIGINT/SIGTERM]
    signal --> drain[engine.start_drain flips readiness source]
    drain --> ready_503[shared readyz reports 503 draining during grace window]
    ready_503 --> stop([grace expires and listener closes])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-service-http-convergence-tests
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
    text: "The shared probe router returns Prometheus text rendered by the supplied metrics provider."
    kind: behavior
    risk: medium
    verify: test
  lumen_openapi_inventory:
    id: R4
    text: "Lumen's served OpenAPI keeps standard admin/probe paths and data-plane paths."
    kind: behavior
    risk: medium
    verify: test
elements:
  service_http_unit_tests:
    kind: test
    path: libs/service-http/src/probes.rs
  lumen_api_e2e:
    kind: test
    path: projects/lumen/tests/api_e2e.rs
  lumen_auth_e2e:
    kind: test
    path: projects/lumen/tests/auth_e2e.rs
relations:
  - { from: service_http_unit_tests, verifies: shared_ready_ok }
  - { from: service_http_unit_tests, verifies: shared_ready_draining }
  - { from: service_http_unit_tests, verifies: shared_metrics }
  - { from: lumen_api_e2e, verifies: lumen_openapi_inventory }
  - { from: lumen_auth_e2e, verifies: shared_ready_ok }
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
    element lumen_api_e2e {
      type: "rs/#[tokio::test]"
    }
    element lumen_auth_e2e {
      type: "rs/#[tokio::test]"
    }
    service_http_unit_tests - verifies -> R1
    service_http_unit_tests - verifies -> R2
    service_http_unit_tests - verifies -> R3
    lumen_api_e2e - verifies -> R4
    lumen_auth_e2e - verifies -> R1
```
## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-service-http-api-contract
    name: "lumen shared service-http probe contract"
    runner: cargo
    path: projects/lumen/tests/api_e2e.rs
    command: "cargo test -p lumen --test api_e2e -- --nocapture"
    verifies:
      - "GET /healthz and GET /readyz stay 200 without authentication."
      - "GET /metrics keeps Prometheus text from the Lumen engine."
      - "GET /openapi.json keeps standard endpoint paths and data-plane paths."
  - id: lumen-service-http-auth-exempt-contract
    name: "lumen shared probes remain auth-exempt"
    runner: cargo
    path: projects/lumen/tests/auth_e2e.rs
    command: "cargo test -p lumen --test auth_e2e -- --nocapture"
    verifies:
      - "Health, readiness, and metrics remain outside the data-plane auth layer."
  - id: lumen-package-regression
    name: "lumen package regression"
    runner: cargo
    path: projects/lumen
    command: "cargo test -p lumen"
    verifies:
      - "The package compiles and the full Lumen regression suite remains green."
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/service-http/src/probes.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Preserve Lumen's readyz body contract while serving readiness from the shared probe router."
  - path: projects/lumen/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add the service-http dependency to Lumen."
  - path: projects/lumen/src/api.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Implement service_http readiness/metrics adapters for Engine, build shared probe routes, keep local OpenAPI path annotations, and use service_http::trace_layer."
  - path: projects/lumen/src/bin/lumen.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Replace the local shutdown signal/drain future with service_http::shutdown_with_drain while keeping OTLP tracing intact."
  - path: projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Synchronize the spec-managed source capture for api.rs service-http adoption."
  - path: projects/lumen/tech-design/semantic/source/projects-lumen-src-bin-lumen-rs.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Synchronize the spec-managed source capture for bin/lumen.rs drain wiring."
```
