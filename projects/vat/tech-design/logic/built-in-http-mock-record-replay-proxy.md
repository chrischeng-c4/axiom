---
id: vat-built-in-http-mock-record-replay-proxy
summary: Ship a built-in HTTP stub + record/replay proxy in vat with transparent HTTPS MITM, so a runner's outbound third-party API calls are intercepted with zero app code change — the mock-killer.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "Adds a built-in HTTP stub + record/replay proxy with HTTPS MITM so a runner's outbound third-party API calls are intercepted locally, letting test suites drop hand-rolled HTTP mocks — through vat's run and evidence surface, with no app code change."
---

# Vat Built-in HTTP Mock + Record/Replay Proxy

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-built-in-http-mock-record-replay-proxy-logic
entry: start
nodes:
  start: { kind: start, label: "dispatch builtin preset http-mock" }
  spawn: { kind: process, label: "spawn self vat emulator http-mock host-port ca-path cassette-dir" }
  ca: { kind: process, label: "mint CA write pem; export proxy and CA-trust env" }
  ready: { kind: process, label: "tcp readiness; runner gets HTTP_PROXY + trust env" }
  accept: { kind: decision, label: "request target form" }
  admin: { kind: process, label: "/__admin/* register stub or set mode or list" }
  connect: { kind: process, label: "CONNECT upgrade; tokio-rustls MITM with CA-signed leaf" }
  forward: { kind: process, label: "absolute-form http proxy request" }
  core: { kind: decision, label: "stub match else cassette else mode" }
  stub: { kind: process, label: "return registered stub response" }
  replay: { kind: process, label: "replay recorded cassette response" }
  record: { kind: process, label: "forward to real upstream via reqwest; record cassette" }
  runner: { kind: process, label: "runner makes calls; intercepted transparently" }
  teardown: { kind: process, label: "stop service kills proxy; cassettes persist" }
  done: { kind: terminal, label: "return exit code" }
edges:
  - { from: start, to: spawn }
  - { from: spawn, to: ca }
  - { from: ca, to: ready }
  - { from: ready, to: runner }
  - { from: runner, to: accept }
  - { from: accept, to: admin, label: "origin-form admin" }
  - { from: accept, to: connect, label: "CONNECT https" }
  - { from: accept, to: forward, label: "absolute-form http" }
  - { from: connect, to: core }
  - { from: forward, to: core }
  - { from: core, to: stub, label: "stub" }
  - { from: core, to: replay, label: "cassette hit" }
  - { from: core, to: record, label: "auto or record" }
  - { from: runner, to: teardown }
  - { from: teardown, to: done }
---
flowchart TD
    start([dispatch builtin preset http-mock]) --> spawn[spawn self vat emulator http-mock host-port ca-path cassette-dir]
    spawn --> ca[mint CA write pem; export proxy and CA-trust env]
    ca --> ready[tcp readiness; runner gets HTTP_PROXY + trust env]
    ready --> runner[runner makes calls; intercepted transparently]
    runner --> accept{request target form}
    accept -- origin-form admin --> admin[__admin register stub or set mode or list]
    accept -- CONNECT https --> connect[CONNECT upgrade; tokio-rustls MITM with CA-signed leaf]
    accept -- absolute-form http --> forward[absolute-form http proxy request]
    connect --> core{stub match else cassette else mode}
    forward --> core
    core -- stub --> stub[return registered stub response]
    core -- cassette hit --> replay[replay recorded cassette response]
    core -- auto or record --> record[forward to real upstream via reqwest; record cassette]
    runner --> teardown[stop service kills proxy; cassettes persist]
    teardown --> done([return exit code])
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-http-mock-evidence.schema.json"
title: "Vat HTTP mock proxy evidence"
type: object
description: "Service-evidence shape, exported env, and the stub/cassette records for vat's built-in HTTP mock proxy."
properties:
  preset:
    type: string
    enum: [http-mock]
  prepare_mode:
    type: string
    enum: [builtin_emulator]
  exported_env:
    type: array
    items: { type: string }
    description: "Proxy + CA-trust vars exported to the runner: HTTP_PROXY/HTTPS_PROXY/ALL_PROXY (+lowercase), NO_PROXY, SSL_CERT_FILE, CURL_CA_BUNDLE, REQUESTS_CA_BUNDLE, NODE_EXTRA_CA_CERTS, GIT_SSL_CAINFO, VAT_HTTP_MOCK_HOST."
  stub:
    type: object
    description: "A registered stub: a request matcher and a canned response."
    properties:
      match:
        type: object
        properties:
          method: { type: string }
          host: { type: string }
          path: { type: string }
        additionalProperties: true
      response:
        type: object
        properties:
          status: { type: integer }
          headers: { type: object, additionalProperties: { type: string } }
          body: { type: string }
        additionalProperties: true
    additionalProperties: true
  cassette:
    type: object
    description: "A recorded request/response pair keyed by method+host+path+query+body."
    properties:
      key: { type: string }
      status: { type: integer }
      body_base64: { type: boolean }
    additionalProperties: true
additionalProperties: true
```

## Config
<!-- type: config lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-config-http-mock.schema.json"
title: "vat.toml (HTTP mock preset addition)"
type: object
properties:
  services:
    type: array
    items:
      type: object
      required: [id]
      properties:
        preset:
          type: string
          enum: [postgres, redis, nats, rabbitmq, mysql, mongo, firestore, pubsub, datastore, bigtable, spanner, firebase, firebase-auth, cloud-tasks, cloud-scheduler, cloud-workflows, cloud-storage, http-mock]
          description: >
            http-mock runs vat's built-in HTTP stub + record/replay proxy under
            runtime=auto (built-in only). It exports a SET of env vars (not a
            single host var): HTTP_PROXY/HTTPS_PROXY/ALL_PROXY point at the proxy,
            NO_PROXY=localhost,127.0.0.1 keeps vat's other loopback emulators
            direct, and CA-trust vars (SSL_CERT_FILE, CURL_CA_BUNDLE,
            REQUESTS_CA_BUNDLE, NODE_EXTRA_CA_CERTS, GIT_SSL_CAINFO) point at the
            vat-minted CA so HTTPS MITM is trusted. The runner needs no code
            change; tests register stubs via VAT_HTTP_MOCK_HOST/__admin. Built-in
            only: runtime must stay auto.
        runtime:
          type: string
          enum: [auto, native, docker]
          default: auto
        export:
          type: object
          additionalProperties: { type: string }
      additionalProperties: true
additionalProperties: true
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat emulator
    usage: "vat emulator http-mock --host-port 127.0.0.1:<PORT> --ca-path <pem> --cassette-dir <dir>"
    behavior:
      - "Hidden verb: vat spawns itself as the service process for the http-mock preset; --ca-path and --cassette-dir are optional args used only by this kind."
      - "Runs a forward proxy: absolute-form http requests and CONNECT (HTTPS MITM via a vat-minted CA + per-host leaf certs) are intercepted; origin-form /__admin/* is the control API (register stubs, set mode, list recordings)."
      - "Each request resolves stub > cassette replay > (auto/record) forward-and-record to the real upstream; cassettes persist under --cassette-dir across runs."
      - "Built without the emulator feature, the verb errors cleanly (no panic); a malformed request never panics the proxy."
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-built-in-http-mock-record-replay-proxy-unit-tests
---
requirementDiagram
    requirement preset_parses_builtin {
      id: UT1
      text: "ServicePreset round-trips http-mock and classifies as built-in and built-in-only; prepare_builtin_service exports the proxy + CA-trust env set including NO_PROXY."
      risk: high
      verifymethod: test
    }
    requirement ca_mints_trusted_leaf {
      id: UT2
      text: "ca.rs mints a CA and a per-host leaf signed by it that parses as a rustls certificate."
      risk: high
      verifymethod: test
    }
    requirement cassette_roundtrip {
      id: UT3
      text: "cassette.rs keys a request and round-trips a stored response (including non-UTF8 body) on disk."
      risk: medium
      verifymethod: test
    }
    requirement stub_matcher {
      id: UT4
      text: "stub.rs matches by method/host/path and returns the registered response, else no match."
      risk: medium
      verifymethod: test
    }
    requirement stub_over_proxy {
      id: UT5
      text: "A stubbed plain-HTTP request through the proxy returns the stub body with no network."
      risk: high
      verifymethod: test
    }
    requirement https_mitm_stub {
      id: UT6
      text: "A client trusting the minted CA and using the proxy GETs a stubbed https URL via CONNECT+MITM and receives the stub (no real upstream)."
      risk: high
      verifymethod: test
    }
    requirement record_then_replay {
      id: UT7
      text: "With no stub, a first request to a local upstream records+forwards; a second request replays from the cassette with the upstream down."
      risk: high
      verifymethod: test
    }
    test config_http_mock_tests {
      type: functional
      verifies: preset_parses_builtin
    }
    test ca_mint_tests {
      type: functional
      verifies: ca_mints_trusted_leaf
    }
    test cassette_store_tests {
      type: functional
      verifies: cassette_roundtrip
    }
    test stub_matcher_tests {
      type: functional
      verifies: stub_matcher
    }
    test stub_over_proxy_tests {
      type: functional
      verifies: stub_over_proxy
    }
    test https_mitm_stub_tests {
      type: functional
      verifies: https_mitm_stub
    }
    test record_replay_tests {
      type: functional
      verifies: record_then_replay
    }
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-http-mock-stub-and-mitm-smoke
    name: "HTTP mock proxy stubs plain and HTTPS-MITM requests"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat --test vat_emulator_httpmock -- --nocapture"
    assertions:
      - "a stubbed GET http://api.test/v1/x through the proxy returns the stub body with no network."
      - "a client trusting the minted CA and using the proxy GETs a stubbed https://api.test/v1/y via CONNECT+MITM and receives the stub (no real upstream)."
      - "with no stub, a first GET to a local upstream records+forwards and a second GET replays from the cassette with the upstream down."
  - id: vat-http-mock-preset-run-smoke
    name: "http-mock preset wires the runner's proxy + CA trust"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat http_mock_preset_exports_proxy_env -- --nocapture --ignored"
    assertions:
      - "a preset = http-mock vat.toml run exports HTTP(S)_PROXY + NO_PROXY + the CA-trust vars; the runner curls a stubbed https URL with no code change and gets the stub."
  - id: vat-http-mock-lean-build
    name: "lean build still compiles"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo build -p vat --no-default-features"
    assertions:
      - "vat compiles without the emulator feature; the http-mock emulator verb then errors cleanly, never a panic."
```
