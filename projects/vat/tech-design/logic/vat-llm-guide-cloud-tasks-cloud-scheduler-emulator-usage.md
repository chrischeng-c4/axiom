---
id: vat-llm-guide-cloud-tasks-cloud-scheduler-emulator-usage
summary: Extend the `vat llm` agent usage guide and the README emulator section to document how to wire a Cloud Tasks / Cloud Scheduler client at vat's built-in REST emulator — those SDKs do not auto-read the emulator host var and default to gRPC, so a factory must force the REST transport, an `http://$HOST` endpoint, and anonymous credentials.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "Closes a usability gap in vat's agent-facing usage contract: a consuming agent could not reliably connect a Cloud Tasks / Cloud Scheduler client to vat's built-in REST emulator because, unlike pubsub/firebase-auth/firestore/GCS, those SDKs do not auto-read the emulator host var and default to gRPC."
---

# Vat LLM Guide: Cloud Tasks / Cloud Scheduler Emulator Usage

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-llm-guide-cloud-tasks-cloud-scheduler-emulator-usage-logic
entry: start
nodes:
  start: { kind: start, label: "agent reads vat llm guide to wire a cloud-tasks or cloud-scheduler client" }
  envcheck: { kind: decision, label: "is CLOUD_TASKS or CLOUD_SCHEDULER EMULATOR_HOST set in env" }
  prod: { kind: process, label: "build default client gRPC TLS ADC against GCP" }
  done: { kind: terminal, label: "client talks to real GCP" }
  alt: { kind: decision, label: "use official SDK or call REST directly" }
  factory: { kind: process, label: "factory builds emulator client because SDK does not auto-read host var" }
  rest: { kind: process, label: "force REST transport since vat serves REST not gRPC" }
  endpoint: { kind: process, label: "set api_endpoint to http HOST" }
  anon: { kind: process, label: "use anonymous credentials emulator skips auth" }
  directrest: { kind: process, label: "POST v2 REST API directly per tests vat_emulator_tasks" }
  ready: { kind: terminal, label: "client points at vat REST emulator dispatch round-trips" }
edges:
  - { from: start, to: envcheck }
  - { from: envcheck, to: prod, label: "no" }
  - { from: prod, to: done }
  - { from: envcheck, to: alt, label: "yes" }
  - { from: alt, to: factory, label: "sdk" }
  - { from: factory, to: rest }
  - { from: rest, to: endpoint }
  - { from: endpoint, to: anon }
  - { from: anon, to: ready }
  - { from: alt, to: directrest, label: "direct rest" }
  - { from: directrest, to: ready }
```
## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-llm-guide-cloud-tasks-scheduler-client-wiring.schema.json"
title: "Cloud Tasks / Scheduler emulator client-wiring shape"
type: object
description: "The client-construction override the guide documents for pointing a Cloud Tasks / Cloud Scheduler client at vat's REST emulator."
properties:
  host_env:
    type: string
    enum: [CLOUD_TASKS_EMULATOR_HOST, CLOUD_SCHEDULER_EMULATOR_HOST]
    description: "vat-exported host var (vat-specific; the SDK does NOT auto-read it)."
  auto_read_by_sdk:
    type: boolean
    const: false
    description: "Unlike PUBSUB/FIRESTORE/STORAGE host vars, these are not auto-discovered by the SDK."
  override:
    type: object
    description: "What a factory must set when host_env is present."
    properties:
      transport: { type: string, const: rest, description: "vat serves REST; SDK default gRPC will not connect." }
      api_endpoint: { type: string, description: "http://$HOST (plaintext, not https)." }
      credentials: { type: string, const: anonymous }
    required: [transport, api_endpoint, credentials]
  direct_rest_alternative:
    type: string
    description: "Skip the SDK and POST the v2 REST API directly (tests/vat_emulator_tasks.rs)."
additionalProperties: true
```

## Config
<!-- type: config lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-config-cloud-tasks-scheduler-client.schema.json"
title: "vat.toml (cloud-tasks / cloud-scheduler service for client wiring)"
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
          enum: [cloud-tasks, cloud-scheduler]
          description: >
            Built-in Rust emulator under runtime=auto. Exports
            CLOUD_TASKS_EMULATOR_HOST / CLOUD_SCHEDULER_EMULATOR_HOST into the
            runner; the runner's client factory reads that var and overrides the
            client transport/endpoint/credentials.
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
  - name: vat llm
    behavior:
      - "Continue to print the stable markdown agent usage guide."
      - "Include a client-wiring note for the cloud-tasks and cloud-scheduler presets: their official SDKs do not auto-read the emulator host var and default to gRPC, vat serves REST, so build the client through a factory that forces REST transport, an http://$HOST endpoint, and anonymous credentials."
      - "Distinguish the auto-host-reading SDKs (pubsub, firebase-auth, firestore, GCS) that need no override."
      - "Point agents at the direct-REST alternative in tests/vat_emulator_tasks.rs."
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-llm-guide-cloud-tasks-cloud-scheduler-emulator-usage-unit-tests
---
requirementDiagram
    requirement guide_documents_client_wiring {
      id: UT1
      text: "The vat llm guide string states cloud-tasks/cloud-scheduler SDKs do not auto-read the host var, default to gRPC vs vat REST, and must be wired via a factory (REST transport + http endpoint + anonymous credentials)."
      risk: medium
      verifymethod: test
    }
    requirement guide_keeps_auto_host_sdks_distinct {
      id: UT2
      text: "The guide still names pubsub/firebase-auth/firestore/GCS as auto-host-reading SDKs needing no override."
      risk: low
      verifymethod: test
    }
    test llm_guide_mentions_core_agent_contract {
      type: functional
      verifies: guide_documents_client_wiring
    }
    test llm_guide_distinguishes_auto_host_sdks {
      type: functional
      verifies: guide_keeps_auto_host_sdks_distinct
    }
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-llm-guide-cloud-tasks-scheduler-client-wiring-smoke
    name: "vat llm guide documents cloud-tasks client wiring"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat llm_guide_mentions_core_agent_contract -- --nocapture"
    assertions:
      - "`vat llm` exits successfully and still mentions vat.toml runner mode, direct command mode, and state/diff/logs evidence commands."
      - "the guide mentions the cloud-tasks / cloud-scheduler client-wiring factory (REST transport + http endpoint + anonymous credentials) and the direct-REST alternative."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/commands/llm.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Add the cloud-tasks / cloud-scheduler client-wiring note to the hand-written `vat llm` guide string."
  - path: projects/vat/README.md
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Mirror the client-wiring note in the README emulator section for operator/agent parity."
  - path: projects/vat/tests/vat_toml_runner.rs
    action: validate
    section: e2e-test
    impl_mode: hand-written
    reason: "The existing `vat llm` guide smoke test verifies the new client-wiring note is present."
```
