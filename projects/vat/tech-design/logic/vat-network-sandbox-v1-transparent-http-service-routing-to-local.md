---
id: vat-network-sandbox-v1-transparent-http-service-routing-to-local
summary: Add host-routing to vat's http-mock proxy so a vat run's outbound HTTP(S) request to a known host (e.g. cloudtasks.googleapis.com) is transparently served by the matching local emulator instead of forwarded upstream — the first step of vat's network sandbox, using the existing proxy-env + CONNECT-MITM + CA cooperating-client mechanism.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "vat can intercept outbound HTTP(S) (http-mock proxy + CA MITM) but cannot route a known host to a local emulator; host-routing makes a run's traffic to real GCP hosts land on the local emulators with zero app config — the foundation of the network sandbox."
---

# vat network sandbox v1: transparent HTTP service routing to local emulators

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-network-sandbox-v1-http-routing-logic
entry: start
nodes:
  start: { kind: start, label: "runner outbound HTTP or HTTPS via injected proxy" }
  conn: { kind: decision, label: "CONNECT (https) or absolute-form (http)" }
  mitm: { kind: process, label: "CONNECT terminate TLS with CA leaf then handle https" }
  handle: { kind: process, label: "handle scheme authority req" }
  routed: { kind: decision, label: "host matches a registered route" }
  tolocal: { kind: process, label: "forward to local emulator base plus path_and_query do NOT record" }
  localresp: { kind: terminal, label: "local emulator response returned to runner" }
  stub: { kind: decision, label: "stub then openapi then cassette" }
  hit: { kind: terminal, label: "stub openapi or cassette answer" }
  forward: { kind: process, label: "forward to real upstream and record (auto)" }
  up: { kind: terminal, label: "upstream response recorded" }
  admin: { kind: process, label: "run.rs posts auto-routes to /__admin/routes once all emulator ports known" }
edges:
  - { from: admin, to: start }
  - { from: start, to: conn }
  - { from: conn, to: mitm, label: "connect" }
  - { from: conn, to: handle, label: "http" }
  - { from: mitm, to: handle }
  - { from: handle, to: routed }
  - { from: routed, to: tolocal, label: "yes" }
  - { from: tolocal, to: localresp }
  - { from: routed, to: stub, label: "no" }
  - { from: stub, to: hit, label: "match" }
  - { from: stub, to: forward, label: "miss" }
  - { from: forward, to: up }
```
## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-network-route.schema.json"
title: "Host route"
type: object
description: "A single host-routing rule the proxy resolves before stub/openapi/cassette/forward."
properties:
  host:
    type: string
    description: "Bare hostname to match (e.g. cloudtasks.googleapis.com). Port-agnostic."
  target:
    type: string
    description: "Local base URL the matched request is forwarded to (e.g. http://127.0.0.1:8085). path_and_query is appended; the response is returned verbatim and NOT recorded."
required: [host, target]
```

## Config
<!-- type: config lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-config-network-routes.schema.json"
title: "vat.toml network routes"
type: object
properties:
  network:
    type: object
    description: "Optional. Enables transparent service routing for the run."
    properties:
      routes:
        type: array
        items:
          type: object
          required: [host, target]
          properties:
            host: { type: string }
            target: { type: string, description: "Local base URL, or a service id reference resolved to its host:port." }
    additionalProperties: true
  note:
    type: string
    description: "Declaring a GCP emulator preset (cloud-tasks, pubsub, ...) auto-adds a route from its real googleapis host to the local emulator; explicit [network.routes] entries override/extend."
additionalProperties: true
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat emulator http-mock
    usage: "vat emulator http-mock --host-port <hp> --ca-path <p> --cassette-dir <d> [--route host=base]..."
    behavior:
      - "Accept repeatable --route host=base flags that seed the proxy's host-routing table at startup."
      - "Resolve each request route-first: a host in the routing table is forwarded to its local target base (path_and_query appended) and the response returned verbatim WITHOUT recording; otherwise fall through to stub > openapi > cassette > forward."
      - "Expose POST/DELETE /__admin/routes to register/clear routes at runtime (so vat can add auto-routes once all emulator ports are known)."
  - name: vat run
    behavior:
      - "When a run declares [network.routes] or GCP emulator presets, ensure an http-mock proxy is present, export HTTP(S)_PROXY + NO_PROXY(localhost) + the CA bundle, and POST the resolved routes (explicit + preset-auto-derived: real googleapis host -> local emulator host:port) to the proxy's /__admin/routes before the runner starts."
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-network-sandbox-v1-http-routing-unit-tests
---
requirementDiagram
    requirement route_first_resolution {
      id: UT1
      text: "A request whose host matches a registered route is forwarded to the route target and NOT recorded; an unmatched host falls through to stub/openapi/cassette/forward unchanged."
      risk: high
      verifymethod: test
    }
    requirement route_table_parse {
      id: UT2
      text: "host=base CLI strings and /__admin/routes JSON both populate the routing table; target base + path_and_query compose the local URL correctly."
      risk: medium
      verifymethod: test
    }
    requirement preset_auto_route_map {
      id: UT3
      text: "The built-in GCP host map yields the right real-host->preset pairs (e.g. cloudtasks.googleapis.com for cloud-tasks)."
      risk: medium
      verifymethod: test
    }
    test route_first_resolution_tests {
      type: functional
      verifies: route_first_resolution
    }
    test route_table_parse_tests {
      type: functional
      verifies: route_table_parse
    }
    test preset_auto_route_map_tests {
      type: functional
      verifies: preset_auto_route_map
    }
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-http-mock-host-routing-smoke
    name: "http-mock routes a known host to a local sink"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat --test vat_emulator_httpmock_routing -- --nocapture"
    assertions:
      - "with a route example.test -> http://127.0.0.1:<sink>, a request through the proxy to http://example.test/p (and https://example.test/p via CONNECT MITM) is answered by the local sink, not forwarded upstream."
      - "an unmatched host still forwards/records exactly as before (no regression)."
  - id: vat-http-mock-routing-build
    name: "default + lean build compile"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo build -p vat --no-default-features"
    assertions:
      - "vat compiles with and without default features."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/emulator/httpmock/mod.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Add the routing table to Proxy; resolve route-first in handle(); add POST/DELETE /__admin/routes; seed routes from serve() args."
  - path: projects/vat/src/cli.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Add the repeatable --route host=base flag to the hidden `vat emulator` verb."
  - path: projects/vat/src/commands/emulator.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Thread --route values into httpmock::serve."
  - path: projects/vat/src/config.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Parse the optional [network].routes vat.toml surface."
  - path: projects/vat/src/commands/run.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Ensure an http-mock proxy + proxy/CA env when routes/GCP presets are declared; build the resolved route map (explicit + preset auto-derived) and POST it to /__admin/routes before the runner starts; built-in GCP host map."
  - path: projects/vat/tests/vat_emulator_httpmock_routing.rs
    action: create
    section: e2e-test
    impl_mode: hand-written
    reason: "Route a known host (http + https-MITM) to a local sink and assert it's served locally; unmatched host still forwards."
```
