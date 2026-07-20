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
  - id: "subscription-delivery-resources"
    role: primary
    claim: "topic-subscription-resource-contract"
    gap: "topic-subscription-resource-contract"
    coverage: partial
    rationale: "The offline API inventory declares topic delivery resource collection/item routes and schemas."
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
  - title: SubscriptionCreateRequest
    type: object
    required: [name]
  - title: Subscription
    type: object
    required: [topic, name]
  - title: SubscriptionListResponse
    type: object
    required: [subscriptions]
  - title: PullSubscriptionRequest
    type: object
  - title: PullSubscriptionBatch
    type: object
    required: [topic, subscription, cursor, limit, next_offset, events]
  - title: PullSubscriptionAckRequest
    type: object
    required: [offset]
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
    routes --> subscriptions["subscription collection and item routes present"]
    routes --> pullack["bounded pull and explicit ack routes present"]
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
  - path: apps/tape/src/spec.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: "Declare subscription routes and pull-only delivery schemas in offline routes, OpenAPI, JSON Schema, and LLM API wording (#1254); push/lease/consumer-group modes are absent."
  - path: apps/tape/src/spec.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: "Declare bounded pull and explicit ack inventory schemas without adding live h2c delivery handlers (#1255)."
```
