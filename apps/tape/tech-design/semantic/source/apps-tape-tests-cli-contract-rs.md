---
id: apps-tape-tests-cli-contract-rs
coverage_kind: semantic
capability_refs:
  - id: "cli-interface"
    role: primary
    claim: "tape-cli-convention-and-replay-verbs"
    gap: "tape-cli-convention-and-replay-verbs"
    coverage: partial
    rationale: "The test proves the first CLI replay/admin workflow."
  - id: "chainable-output-conformance"
    role: primary
    claim: "local-replay-command-next-markers"
    gap: "local-replay-command-next-markers"
    coverage: partial
    rationale: "The test exercises commands that print next markers."
  - id: "subscription-delivery-resources"
    role: primary
    claim: "topic-subscription-resource-contract"
    gap: "topic-subscription-resource-contract"
    coverage: partial
    rationale: "The integration test proves local subscription CLI forms and offline spec inventory."
fill_sections: [overview, unit-test, changes]
---

# Tape CLI Contract Test

## Overview
<!-- type: overview lang: markdown -->

`apps/tape/tests/cli_contract.rs` verifies the binary-visible contract for
the bootstrap Tape CLI and offline spec.
It also verifies the topic subscription resource slice: pull checkpoint
compatibility, absence of push/mode flags, and declared API inventory.
Pull reads are bounded and do not advance their cursor until the matching
explicit ack command succeeds.

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-td-flow
---
flowchart TD
    test["cargo test -p tape --test cli_contract -- --nocapture"] --> help["compiled tape --help exposes command groups"]
    test --> spec["spec routes list API inventory"]
    test --> workflow["append/replay/checkpoint commands round-trip temp store"]
    test --> subscriptions["pull-only subscription create/list/show/delete round-trip temp store; push rejected"]
    test --> subscription_spec["subscription routes/OpenAPI/JSON Schema are declared"]
    test --> pullack["bounded pull then explicit ack continues at the next checkpoint offset"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/tests/cli_contract.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Binary smoke tests for Tape CLI and spec route inventory."
  - path: apps/tape/tests/cli_contract.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Add subscription CLI, local lifecycle, and offline contract inventory tests (#1254)."
  - path: apps/tape/tests/cli_contract.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Add bounded pull/ack round-trip and pull/ack inventory coverage (#1255)."
```
