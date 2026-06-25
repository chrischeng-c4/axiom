---
id: pool-the-grpc-mitm-reverse-proxy-h2c-upstream-connections
summary: Pool vat's gRPC MITM reverse-proxy upstream h2c connections per target so many gRPC requests multiplex over one reused HTTP/2 connection (recreated only when closed), instead of handshaking a fresh connection per request as #509 currently does.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "The #509 gRPC reverse-proxy reconnects per request; reusing a multiplexed h2c connection per upstream removes redundant handshakes and socket churn under load while keeping behaviour identical."
---

# Pool the gRPC MITM reverse-proxy h2c upstream connections

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-grpc-reverse-proxy-conn-pool-logic
entry: start
nodes:
  start: { kind: start, label: "routed gRPC request needs upstream send" }
  lookup: { kind: decision, label: "pool has a SendRequest for host:port" }
  health: { kind: decision, label: "pooled sender is ready not closed" }
  reuse: { kind: process, label: "clone the SendRequest multiplex on existing conn" }
  dial: { kind: process, label: "tcp connect plus http2 handshake spawn conn task" }
  store: { kind: process, label: "insert sender into pool keyed by host:port" }
  send: { kind: process, label: "send_request stream body and trailers" }
  done: { kind: terminal, label: "response streamed back" }
edges:
  - { from: start, to: lookup }
  - { from: lookup, to: health, label: "hit" }
  - { from: lookup, to: dial, label: "miss" }
  - { from: health, to: reuse, label: "healthy" }
  - { from: health, to: dial, label: "dead evict" }
  - { from: dial, to: store }
  - { from: store, to: send }
  - { from: reuse, to: send }
  - { from: send, to: done }
```
## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-grpc-conn-pool.schema.json"
title: "h2c upstream connection pool"
type: object
properties:
  key:
    type: string
    description: "Upstream host:port (the route target's authority)."
  entry:
    type: string
    description: "A hyper http2 SendRequest (clonable, multiplexes); the driving Connection runs on a spawned task."
  health_check:
    type: string
    description: "Before reuse, verify the sender is usable (ready / not is_closed); a dead entry is evicted and re-dialed."
  concurrency:
    type: string
    description: "Pool guarded by a Mutex/RwLock; the request multiplexes on a cloned SendRequest rather than holding the lock for the whole call."
additionalProperties: true
```
## Config
<!-- type: config lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-grpc-conn-pool-config.schema.json"
title: "no config surface"
type: object
properties:
  note:
    type: string
    description: "Internal optimization — no vat.toml or CLI surface. One reused connection per upstream target; no tunables in this increment."
additionalProperties: true
```
## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat emulator http-mock
    behavior:
      - "Unchanged external behaviour. Internally the gRPC reverse-proxy reuses a pooled per-upstream h2c connection (multiplexed) instead of handshaking per request; a dead pooled connection is transparently re-established."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-grpc-reverse-proxy-conn-pool-unit-tests
---
requirementDiagram
    requirement reuse_across_requests {
      id: UT1
      text: "Two sequential routed gRPC calls to the same upstream reuse one h2c connection (upstream accepts once)."
      risk: medium
      verifymethod: test
    }
    requirement recover_after_close {
      id: UT2
      text: "After the pooled connection closes, the next call evicts it, re-dials, and succeeds."
      risk: medium
      verifymethod: test
    }
    test grpc_pool_reuse_tests {
      type: functional
      verifies: reuse_across_requests
    }
    test grpc_pool_recover_tests {
      type: functional
      verifies: recover_after_close
    }
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-grpc-pool-reuse-smoke
    name: "pooled gRPC reverse-proxy reuses one upstream connection"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat --test vat_emulator_grpc_mitm_routing -- --nocapture"
    assertions:
      - "two sequential gRPC calls through the MITM to the same emulator both succeed (reuse path), and the #509 single-call e2e still passes."
  - id: vat-grpc-pool-build
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
    reason: "Add a per-upstream h2c SendRequest pool on Proxy (Mutex<HashMap<String, SendRequest>>); grpc_reverse_proxy gets-or-dials a healthy multiplexed sender from it, evicting+re-dialing a dead entry, instead of handshaking per request."
  - path: projects/vat/tests/vat_emulator_grpc_mitm_routing.rs
    action: modify
    section: e2e-test
    impl_mode: hand-written
    reason: "Add a second sequential gRPC call asserting connection reuse; keep the #509 single-call proof."
```
