---
id: '1642'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: opaque-key-admission-control
entry: request
nodes:
  request: { kind: start, label: "Receive request with caller-selected endpoint class and opaque key" }
  enabled: { kind: decision, label: "Admission policy configured for endpoint class?" }
  bypass: { kind: process, label: "Allow without allocating limiter state" }
  fingerprint: { kind: process, label: "Hash opaque key; never retain or emit raw input" }
  bucket: { kind: process, label: "Refill matching token bucket from monotonic caller time" }
  available: { kind: decision, label: "At least one token available?" }
  allow: { kind: process, label: "Consume token and emit typed allow decision" }
  deny: { kind: process, label: "Emit typed deny with bounded Retry-After" }
  bound: { kind: process, label: "Evict least-recently-observed buckets until max_keys is respected" }
  done: { kind: terminal, label: "Return allow or standard 429 response without exposing key bytes" }
edges:
  - { from: request, to: enabled }
  - { from: enabled, to: bypass, label: "no" }
  - { from: enabled, to: fingerprint, label: "yes" }
  - { from: fingerprint, to: bucket }
  - { from: bucket, to: available }
  - { from: available, to: allow, label: "yes" }
  - { from: available, to: deny, label: "no" }
  - { from: allow, to: bound }
  - { from: deny, to: bound }
  - { from: bypass, to: done }
  - { from: bound, to: done }
---
flowchart TD
    request([Request]) --> enabled{Policy configured?}
    enabled -->|no| bypass[Allow without state]
    enabled -->|yes| fingerprint[Hash opaque key]
    fingerprint --> bucket[Refill bucket]
    bucket --> available{Token available?}
    available -->|yes| allow[Consume and allow]
    available -->|no| deny[429 plus Retry-After]
    allow --> bound[Enforce max_keys]
    deny --> bound
    bypass --> done([Credential-free result])
    bound --> done
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/service-http/src/admission.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "Bounded deterministic opaque-key token buckets, redacted decision hooks, standard Retry-After rejection, and reusable axum middleware."
  - path: libs/service-http/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Expose the shared admission module and public contract."
  - path: libs/service-http/tech-design/semantic/source/libs-service-http-src-lib-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Keep the semantic source mirror aligned with the admission export."
  - path: libs/service-http/Cargo.toml
    action: modify
    section: changes
    impl_mode: hand-written
    description: "Add the SHA-256 implementation used to fingerprint opaque keys before retention."
  - path: libs/service-http/README.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: "Publish the shared opaque-key admission capability rooted at #1642."
  - path: apps/lumen/src/api.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Expose an opt-in router_with_admission boundary so Lumen chooses endpoint classes and policies without owning enforcement."
  - path: apps/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Keep Lumen API semantic source aligned with the optional shared admission layer."
  - path: apps/lumen/tests/admission_e2e.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Verify Lumen-selected collection-read policy rejects excess requests while the default router remains unchanged."
  - path: apps/tape/src/server.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Expose the same opt-in router boundary with Tape-selected route classes and policy values."
  - path: apps/tape/tests/service_admission.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Verify Tape-selected append policy uses shared enforcement and default routing stays disabled."
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: opaque-key-admission-control-verification
requirements:
  app_adoption:
    id: R4
    text: "Lumen and Tape can each select their own endpoint classes and policies through the same optional middleware boundary; their existing router entrypoints remain admission-disabled."
    kind: integration
    risk: high
    verify: cargo test -p lumen --test admission_e2e && cargo test -p tape --test service_admission
  bounded_redaction:
    id: R2
    text: "The controller never stores raw opaque keys, event schemas cannot carry them, and per-policy state evicts deterministically to remain at or below max_keys."
    kind: security
    risk: high
    verify: cargo test -p service-http admission
  deterministic_bucket:
    id: R1
    text: "Configured endpoint classes consume deterministic token-bucket capacity, deny excess requests, and refill from caller-controlled monotonic time with a bounded Retry-After value."
    kind: functional
    risk: high
    verify: cargo test -p service-http admission
  standard_http:
    id: R3
    text: "The reusable axum middleware returns the shared error envelope with HTTP 429 and Retry-After while an unconfigured endpoint class passes without allocating state."
    kind: integration
    risk: high
    verify: cargo test -p service-http admission
---
flowchart TD
    r1[R1 deterministic bucket] --> cargo_test_p_service_http_admission[cargo test -p service-http admission]
    r2[R2 bounded redaction] --> cargo_test_p_service_http_admission
    r3[R3 standard http] --> cargo_test_p_service_http_admission
    r4[R4 app adoption] --> cargo_test_p_lumen_test_admission_e2e_cargo_test_p_tape_test_service_admission[cargo test -p lumen --test admission_e2e && cargo test -p tape --test service_admission]
```
