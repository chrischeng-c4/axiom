---
id: apps-tape-src-spec-rs
coverage_kind: semantic
capability_refs:
  - id: "http2-api-list"
    role: primary
    claim: "h2c-openapi-route-list"
    gap: "h2c-openapi-route-list"
    coverage: partial
    rationale: "The offline spec exposes Tape's service route inventory before the h2c server lands."
  - id: "standard-operational-endpoints"
    role: primary
    claim: "standard-service-route-inventory"
    gap: "standard-service-route-inventory"
    coverage: partial
    rationale: "The route inventory includes health, readiness, metrics, OpenAPI, and docs endpoints."
fill_sections: [overview, schema, unit-test, changes]
---

# Tape Offline Spec Surface

## Overview
<!-- type: overview lang: markdown -->

`apps/tape/src/spec.rs` is the offline machine-readable contract for the
first Tape service slice. It emits OpenAPI-shaped JSON/YAML, JSON schemas, a
route inventory, and LLM topic bodies.

## Schema
<!-- type: schema lang: yaml -->

```yaml
schemas:
  - title: AppendEventRequest
    type: object
    required: [payload]
  - title: TapeEvent
    type: object
    required: [topic, offset, timestamp_ms, payload]
  - title: ReplayResponse
    type: object
    required: [events]
  - title: CheckpointRequest
    type: object
    required: [offset]
  - title: ConsumerCheckpoint
    type: object
    required: [topic, consumer, offset, updated_at_ms]
  - title: RetentionPolicy
    type: object
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-td-flow
---
flowchart TD
    test["cargo test -p tape --test cli_contract spec_routes_list_topic_contract -- --exact --nocapture"] --> routes["tape spec --format routes"]
    routes --> topic["append/replay/checkpoint routes present"]
    routes --> ops["/healthz /readyz /metrics /openapi.json /docs present"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/src/spec.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: "Initial offline route, OpenAPI-shaped, schema, and LLM topic contract."
  - path: apps/tape/tests/cli_contract.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Spec route inventory test through the Tape binary."
```
