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
fill_sections: [overview, unit-test, changes]
---

# Tape CLI Contract Test

## Overview
<!-- type: overview lang: markdown -->

`apps/tape/tests/cli_contract.rs` verifies the binary-visible contract for
the bootstrap Tape CLI and offline spec.

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
```
