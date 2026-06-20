---
id: keep-runtime
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    gap: "http-key-value-surface"
    claim: "http-key-value-surface"
    coverage: partial
    rationale: "Existing hand-written HTTP API tests cover the KV and claim-check public surface while full EC codegen remains unclaimed."
  - id: "long-running-stability"
    role: primary
    gap: "wal-backed-cold-recovery"
    claim: "wal-backed-cold-recovery"
    coverage: partial
    rationale: "Existing hand-written durability tests prove durable-before-ack recovery for the current engine."
  - id: "competitor-feature-parity"
    role: primary
    gap: "hash-set-sorted-set-operations"
    claim: "hash-set-sorted-set-operations"
    coverage: partial
    rationale: "Existing hand-written collection API tests cover hash, set, sorted-set, and list routes."
  - id: "cli-interface"
    role: primary
    gap: "openapi-probe-metrics-surface"
    claim: "openapi-probe-metrics-surface"
    coverage: partial
    rationale: "Existing HTTP API tests cover the binary-served OpenAPI, health, readiness, and metrics surface."
  - id: "security-hardening"
    role: primary
    gap: "body-limit-and-public-route-boundary"
    claim: "body-limit-and-public-route-boundary"
    coverage: enabling
    rationale: "Router body-limit and public route boundaries are the current implemented security baseline before negative gates are authored."
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
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: keep-runtime-unit-test
coverage_kind: semantic
strategy: preserve observed runtime behavior while EC codegen remains unclaimed
tests:
  - id: keep-http-kv-api
    capability_id: competitor-feature-parity
    command: "cargo test -p keep --test http_api -- --nocapture"
    generated: false
  - id: keep-durable-before-ack
    capability_id: long-running-stability
    command: "cargo test -p keep --test durability -- --nocapture"
    generated: false
  - id: keep-collections-api
    capability_id: competitor-feature-parity
    command: "cargo test -p keep --test collections_api -- --nocapture"
    generated: false
---
requirementDiagram

requirement KEEP_HTTP_KV_API {
  id: keep-http-kv-api
  text: HTTP key-value operations, OpenAPI, probes, metrics, locks, batches, scans, and claim-check blobs stay callable.
  risk: medium
  verifymethod: test
}

requirement KEEP_DURABLE_BEFORE_ACK {
  id: keep-durable-before-ack
  text: Mutations are WAL-backed before acknowledgement and committed state survives cold recovery.
  risk: high
  verifymethod: test
}

requirement KEEP_COLLECTIONS_API {
  id: keep-collections-api
  text: Hash, set, sorted-set, and list operations preserve their public HTTP behavior.
  risk: medium
  verifymethod: test
}
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
