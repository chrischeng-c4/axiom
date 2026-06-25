---
id: vat-network-sandbox-v2-transparent-grpc-routing-via-h2-mitm-reve
summary: Teach vat's http-mock CONNECT MITM to negotiate ALPN h2 and stream-reverse-proxy a routed gRPC request to the local emulator's h2c endpoint, so a stock gRPC SDK client to a real GCP host is transparently served by the dual-protocol emulator — completing sandbox transparent routing for gRPC as well as HTTP.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "v1 routes HTTP but the MITM is HTTP/1-only, so a stock gRPC client to a real GCP host escapes; teaching the MITM h2 + a trailer-preserving reverse-proxy to the emulator's h2c port makes gRPC transparently land locally — the second half of the network sandbox's transparent service routing."
---

# vat network sandbox v2: transparent gRPC routing via h2 MITM reverse-proxy

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-network-sandbox-v2-grpc-routing-logic
entry: start
nodes:
  start: { kind: start, label: "runner gRPC client TLS to real GCP host via injected proxy CONNECT" }
  connect: { kind: process, label: "MITM accept CONNECT terminate TLS with CA leaf advertising alpn h2 and http1.1" }
  alpn: { kind: decision, label: "negotiated alpn protocol" }
  h1: { kind: process, label: "http1 path unchanged route stub openapi cassette forward" }
  h2serve: { kind: process, label: "serve hyper http2 over decrypted stream" }
  routed: { kind: decision, label: "host matches a route" }
  proxy: { kind: process, label: "h2c handshake to route target stream request body and trailers verbatim" }
  resp: { kind: process, label: "stream response body and grpc-status trailers back unbuffered" }
  emu: { kind: terminal, label: "request lands on local dual-protocol gRPC emulator dispatch" }
  noroute: { kind: terminal, label: "non-routed h2 returns 502 route-only in v2" }
edges:
  - { from: start, to: connect }
  - { from: connect, to: alpn }
  - { from: alpn, to: h1, label: "http1.1 or none" }
  - { from: alpn, to: h2serve, label: "h2" }
  - { from: h2serve, to: routed }
  - { from: routed, to: proxy, label: "yes" }
  - { from: proxy, to: resp }
  - { from: resp, to: emu }
  - { from: routed, to: noroute, label: "no" }
```
## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-sandbox-v2-grpc-mitm.schema.json"
title: "gRPC MITM reverse-proxy behaviour"
type: object
properties:
  alpn_offered:
    type: array
    items: { type: string }
    description: "MITM leaf ServerConfig ALPN: [h2, http/1.1]."
  h2_routed:
    type: string
    description: "Routed host over h2 → reverse-proxy to the route target via h2c, streaming body + HTTP/2 trailers (grpc-status/grpc-message) verbatim. Never buffer the body."
  h2_unrouted:
    type: string
    description: "Non-routed h2 → 502 (route-only; no h2 record/replay in v2)."
  http1_unchanged:
    type: boolean
    const: true
    description: "ALPN http/1.1 or none keeps the existing route>stub>openapi>cassette>forward path."
additionalProperties: true
```
## Config
<!-- type: config lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-config-sandbox-v2.schema.json"
title: "vat.toml (no new keys; reuses v1 routes)"
type: object
properties:
  note:
    type: string
    description: "v2 adds no vat.toml surface. It reuses the v1 [network].routes table and the preset-auto-derived routes; the route TARGET (the dual-protocol emulator host:port) already serves h2c gRPC, so the same route entry now routes both REST and gRPC."
  cargo_features:
    type: object
    description: "hyper must enable http2 explicitly (v1 relied on tonic feature-unification)."
    properties:
      hyper: { type: array, items: { type: string }, description: "server, client, http1, http2" }
additionalProperties: true
```
## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat emulator http-mock
    behavior:
      - "The MITM leaf now advertises ALPN h2 + http/1.1; after the TLS handshake the proxy branches on the negotiated protocol."
      - "h2 + routed host: reverse-proxy to the route target over h2c, streaming request/response bodies AND HTTP/2 trailers verbatim (so unary/streaming gRPC and grpc-status pass through). h2 + unrouted: 502 (route-only)."
      - "http/1.1 (or no ALPN): unchanged — route > stub > openapi > cassette > forward."
  - name: vat run
    behavior:
      - "No new flags. Declaring a GCP emulator preset + http-mock already seeds the host route (#503); that route now transparently serves the host's gRPC traffic too, so a stock gRPC SDK client (TLS to the real host, trusting vat's CA) reaches the local emulator with no app config."
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-network-sandbox-v2-grpc-routing-unit-tests
---
requirementDiagram
    requirement alpn_h2_negotiated {
      id: UT1
      text: "The MITM ServerConfig advertises ALPN h2+http/1.1 and a client offering h2 negotiates h2."
      risk: medium
      verifymethod: test
    }
    requirement trailers_forwarded {
      id: UT2
      text: "The h2 reverse-proxy streams the body without buffering and forwards HTTP/2 trailers, so a unary gRPC call (grpc-status in trailers) succeeds end-to-end."
      risk: high
      verifymethod: test
    }
    test alpn_h2_negotiated_tests {
      type: functional
      verifies: alpn_h2_negotiated
    }
    test grpc_trailers_forwarded_tests {
      type: functional
      verifies: trailers_forwarded
    }
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-grpc-mitm-routing-smoke
    name: "gRPC client routed through the MITM reaches the local emulator"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat --test vat_emulator_grpc_mitm_routing -- --nocapture"
    assertions:
      - "a generated google.cloud.tasks.v2 gRPC client over TLS to cloudtasks.googleapis.com (trusting vat's CA), routed through the proxy, CreateQueue + CreateTask succeeds and the task is dispatched to a local sink."
      - "HTTP routing (#503) and unmatched-host forwarding still work (no regression)."
  - id: vat-grpc-mitm-build
    name: "default + lean build compile with hyper http2"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo build -p vat --no-default-features"
    assertions:
      - "vat compiles with and without default features; hyper has the http2 feature explicitly."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/emulator/httpmock/ca.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Advertise ALPN h2 + http/1.1 on the MITM leaf ServerConfig (one line)."
  - path: projects/vat/src/emulator/httpmock/mod.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "In handle_connect, branch on the negotiated ALPN; serve hyper http2 over the decrypted stream; for a routed host, stream-reverse-proxy (body + trailers, unbuffered) to the route target over h2c; 502 for non-routed h2. h1 path unchanged."
  - path: projects/vat/Cargo.toml
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Add the http2 feature to hyper (v1 relied on tonic feature-unification)."
  - path: projects/vat/tests/vat_emulator_grpc_mitm_routing.rs
    action: create
    section: e2e-test
    impl_mode: hand-written
    reason: "gRPC client over TLS-to-real-host, routed through the MITM, reaches the local emulator + sink; trailer forwarding proven by the call succeeding."
```
