---
id: projects-tape-tests-competitor-feature-parity-rs
coverage_kind: semantic
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "topic-replay-competitor-feature-matrix"
    gap: "topic-replay-competitor-feature-matrix"
    coverage: partial
    rationale: "The test validates Kafka-style replay-log peers and RabbitMQ topic exchange as routing-only comparison input."
fill_sections: [overview, unit-test, changes]
---

# Tape Competitor Feature Parity Test

## Overview
<!-- type: overview lang: markdown -->

`projects/tape/tests/competitor_feature_parity.rs` is the functional EC backing
for Tape's first competitor feature matrix. It validates replay-log features and
keeps RabbitMQ topic exchange scoped to topic routing/fanout semantics.

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-td-flow
---
flowchart TD
    test["cargo test -p tape --test competitor_feature_parity -- --nocapture"] --> exchange["RabbitMQ topic exchange classified as routing-only"]
    test --> logs["Kafka/Redpanda/Pulsar/JetStream/RabbitMQ Streams are replay-log baselines"]
    test --> tape["Tape local journal validates replay/checkpoint/routing-key behavior"]
    test --> claims["Tape only claims replay feature advantage over RabbitMQ topic exchange"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/tape/tests/competitor_feature_parity.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Functional competitor feature parity test for Tape replay-log semantics."
```
