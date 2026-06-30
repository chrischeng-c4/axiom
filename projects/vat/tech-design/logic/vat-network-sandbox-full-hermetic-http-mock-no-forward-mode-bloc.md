---
id: vat-network-sandbox-full-hermetic-http-mock-no-forward-mode-bloc
summary: Add a no-forward (hermetic) mode to vat's http-mock proxy so an unmatched request returns a clear blocked error instead of forwarding to the real upstream — auto-enabled when [network].egress is localhost-only/deny, so the runner's seatbelt egress confinement plus a non-forwarding proxy make a vat run fully hermetic (fail-closed, can't reach the real internet) while routes/stubs/openapi/cassette-replays still work.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: full-hermetic-http-mock-no-forward-mode
    claim: full-hermetic-http-mock-no-forward-mode
    coverage: partial
    rationale: "Egress confinement (#518/#527) confines the runner but the un-sandboxed http-mock proxy still forwards unmatched requests to the internet; a no-forward mode closes that escape hatch, completing the hermetic sandbox."
---

# vat network sandbox: full-hermetic http-mock no-forward mode

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-network-sandbox-hermetic-logic
entry: start
nodes:
  start: { kind: start, label: "vat run egress not open spawns http-mock with --no-forward" }
  req: { kind: process, label: "proxied request route stub openapi cassette" }
  match: { kind: decision, label: "local match found" }
  serve: { kind: terminal, label: "serve route stub openapi or cassette replay local" }
  fwd: { kind: decision, label: "forward policy on" }
  upstream: { kind: terminal, label: "forward to real upstream and record default open" }
  block: { kind: terminal, label: "502 hermetic no local match forwarding disabled no upstream no record" }
edges:
  - { from: start, to: req }
  - { from: req, to: match }
  - { from: match, to: serve, label: "hit" }
  - { from: match, to: fwd, label: "miss" }
  - { from: fwd, to: upstream, label: "on" }
  - { from: fwd, to: block, label: "off hermetic" }
```
## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-hermetic-proxy.schema.json"
title: "http-mock forward policy"
type: object
properties:
  forward:
    type: boolean
    default: true
    description: "true = current (forward unmatched to real upstream + record). false = hermetic: unmatched returns 502 blocked, no upstream connection, no cassette write."
  blocked_response:
    type: object
    description: "The hermetic miss response."
    properties:
      status: { type: integer, const: 502 }
      body: { type: string, description: "{\"error\":\"hermetic: no local match for <host><path>; forwarding disabled\"}" }
  unaffected:
    type: array
    items: { type: string }
    description: "route, stub, openapi, cassette-replay still serve in no-forward mode; only forward+record is disabled. Non-routed h2/gRPC already 502s."
additionalProperties: true
```
## Config
<!-- type: config lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-config-hermetic.schema.json"
title: "egress-derived hermetic proxy"
type: object
properties:
  derivation:
    type: string
    description: "vat run passes --no-forward to the http-mock spawn when [network].egress is localhost-only or deny; open keeps forwarding. No separate vat.toml key — one knob (egress) yields full hermeticity."
additionalProperties: true
```
## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat emulator http-mock
    usage: "vat emulator http-mock --host-port <hp> --ca-path <p> --cassette-dir <d> [--route ...] [--no-forward]"
    behavior:
      - "--no-forward puts the proxy in hermetic mode: an unmatched request (after route/stub/openapi/cassette miss) returns 502 {\"error\":\"hermetic: ...; forwarding disabled\"} instead of forwarding to the real upstream or writing a cassette."
      - "Without --no-forward, forwarding+recording is unchanged."
  - name: vat run
    behavior:
      - "Spawns the http-mock proxy with --no-forward when [network].egress is localhost-only or deny — so the runner (seatbelt-confined to localhost) plus a non-forwarding proxy make the run hermetic. egress=open keeps the proxy forwarding."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-network-sandbox-hermetic-unit-tests
---
requirementDiagram
    requirement no_forward_blocks_miss {
      id: UT1
      text: "With forward=off, an unmatched request returns 502 with the hermetic error and makes no upstream connection / no cassette write; a registered stub still serves."
      risk: high
      verifymethod: test
    }
    requirement forward_default_unchanged {
      id: UT2
      text: "With forward=on (default), forwarding+recording behaviour is unchanged."
      risk: medium
      verifymethod: test
    }
    test hermetic_no_forward_tests {
      type: functional
      verifies: no_forward_blocks_miss
    }
    test forward_default_unchanged_tests {
      type: functional
      verifies: forward_default_unchanged
    }
```
## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-hermetic-no-forward-smoke
    name: "http-mock --no-forward blocks unmatched, serves stub"
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: full-hermetic-http-mock-no-forward-mode
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat --test vat_emulator_httpmock_hermetic -- --nocapture"
    assertions:
      - "a proxy started with --no-forward returns 502 hermetic for an unmatched host (no upstream reached) while a registered stub on the same proxy returns 200; an unmatched request on a default (forwarding) proxy still forwards."
  - id: vat-hermetic-build
    name: "default + lean build compile"
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: full-hermetic-http-mock-no-forward-mode
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
    reason: "Add forward: bool to Proxy (seeded from serve()); gate the handle() forward step (4) — when off, return the 502 hermetic error instead of connecting upstream / writing a cassette; the h2 non-grpc fallthrough that reaches handle() inherits it."
  - path: projects/vat/src/emulator/mod.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Thread a no_forward flag into Kind::HttpMock and httpmock::serve."
  - path: projects/vat/src/cli.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Add the --no-forward flag to the hidden `vat emulator` verb (http-mock only)."
  - path: projects/vat/src/commands/emulator.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Pass --no-forward through to httpmock::serve."
  - path: projects/vat/src/commands/run.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Spawn the http-mock proxy with --no-forward when [network].egress is localhost-only or deny."
  - path: projects/vat/tests/vat_emulator_httpmock_hermetic.rs
    action: create
    section: e2e-test
    impl_mode: hand-written
    reason: "--no-forward proxy: unmatched → 502 hermetic (no upstream), stub still served; default proxy still forwards."
```
