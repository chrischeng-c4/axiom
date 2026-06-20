---
id: keep-runtime
capability_refs:
  - id: "kv-api"
    role: primary
    gap: "http-key-value-surface"
    claim: "http-key-value-surface"
    coverage: partial
    rationale: "Existing hand-written HTTP API tests cover the KV and claim-check public surface while full EC codegen remains unclaimed."
  - id: "durability"
    role: primary
    gap: "wal-backed-cold-recovery"
    claim: "wal-backed-cold-recovery"
    coverage: partial
    rationale: "Existing hand-written durability tests prove durable-before-ack recovery for the current engine."
  - id: "collections"
    role: primary
    gap: "hash-set-sorted-set-operations"
    claim: "hash-set-sorted-set-operations"
    coverage: partial
    rationale: "Existing hand-written collection API tests cover hash, set, sorted-set, and list routes."
fill_sections: [overview, unit-test, changes]
---

# Keep Runtime Readiness

## Overview
<!-- type: overview lang: markdown -->

Keep is currently standardized as an adopted runtime: the capability contract is
README-owned, the runtime tests are hand-written, and no generated EC wrapper is
claimed until an `external-contracts/` document is authored and `aw ec gen`
materializes the wrapper files.

## Unit Test
<!-- type: unit-test lang: yaml -->

```yaml
unit_tests:
  - id: keep-http-kv-api
    capability_id: kv-api
    command: "cargo test -p keep --test http_api -- --nocapture"
    generated: false
    assertions:
      - HTTP key-value operations roundtrip over the public API
      - OpenAPI, probes, metrics, locks, batches, scans, and claim-check blobs stay callable
  - id: keep-durable-before-ack
    capability_id: durability
    command: "cargo test -p keep --test durability -- --nocapture"
    generated: false
    assertions:
      - mutations are WAL-backed before acknowledgement
      - committed state survives cold recovery
  - id: keep-collections-api
    capability_id: collections
    command: "cargo test -p keep --test collections_api -- --nocapture"
    generated: false
    assertions:
      - hash, set, sorted-set, and list operations remain available
      - collection APIs preserve their public HTTP behavior
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: projects/keep/README.md
    action: claim
    section: overview
    impl_mode: hand-written
    reason: "Canonical capability surface for adopted keep runtime."
  - path: projects/keep/tests/http_api.rs
    action: claim
    section: unit-test
    impl_mode: hand-written
    reason: "Existing hand-written public API conformance gate."
  - path: projects/keep/tests/durability.rs
    action: claim
    section: unit-test
    impl_mode: hand-written
    reason: "Existing hand-written durable-before-ack recovery gate."
  - path: projects/keep/tests/collections_api.rs
    action: claim
    section: unit-test
    impl_mode: hand-written
    reason: "Existing hand-written collection API conformance gate."
```
