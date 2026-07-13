---
id: "1604"
summary: Adopt the shared runtime, bearer auth, OpenAPI client generation, and JSON-safe agent CLI contract for Sift.
capability_refs:
  - id: security-hardening
    role: primary
    gap: shared-bearer-token-auth
    claim: shared-bearer-token-auth
    coverage: partial
    rationale: Sift must protect data-plane operations without blocking operational probes.
  - id: cli-standard-surface
    role: primary
    gap: offline-llm-topics
    claim: offline-llm-topics
    coverage: partial
    rationale: The Sift CLI must provide offline typed-client generation and valid machine-readable output.
  - id: chainable-output-conformance
    role: contributes
    gap: executable-next-command-validation
    claim: executable-next-command-validation
    coverage: partial
    rationale: JSON output must remain parseable while naming an executable or terminal next step.
fill_sections: [logic, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-runtime-security-flow
entry: request
nodes:
  request: { kind: start, label: "incoming h2c or HTTP/1.1 request" }
  probe: { kind: decision, label: "standard probe route?" }
  standard: { kind: terminal, label: "serve health, readiness, metrics, spec, or docs" }
  auth: { kind: decision, label: "SIFT_AUTH required?" }
  bearer: { kind: decision, label: "bearer token valid and authorized?" }
  denied: { kind: terminal, label: "return shared 401 or 403 error envelope" }
  route: { kind: process, label: "dispatch Sift data-plane route" }
  result: { kind: terminal, label: "return Sift result with shared metrics and error contract" }
edges:
  - { from: request, to: probe }
  - { from: probe, to: standard, label: "yes" }
  - { from: probe, to: auth, label: "no" }
  - { from: auth, to: route, label: "off" }
  - { from: auth, to: bearer, label: "required" }
  - { from: bearer, to: denied, label: "no" }
  - { from: bearer, to: route, label: "yes" }
  - { from: route, to: result }
---
flowchart TD
    request([incoming request]) --> probe{standard route?}
    probe -->|yes| standard([serve probe/admin route])
    probe -->|no| auth{SIFT_AUTH required?}
    auth -->|off| route[dispatch data-plane route]
    auth -->|required| bearer{valid bearer?}
    bearer -->|no| denied([401 or 403])
    bearer -->|yes| route
    route --> result([shared metrics and error result])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/sift/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    gap: sift-shared-service-dependencies
    tracker: "1604"
    description: Compose service-auth, service-metrics, build-stamp, and typed OpenAPI code generation dependencies.
  - path: projects/sift/build.rs
    action: create
    section: logic
    impl_mode: hand-written
    gap: sift-build-provenance
    tracker: "1604"
    description: Stamp Sift source revision, build time, and target through the shared build-stamp crate.
  - path: projects/sift/src/auth.rs
    action: create
    section: logic
    impl_mode: hand-written
    gap: sift-shared-bearer-auth
    tracker: "1604"
    description: Adapt shared bearer-token verification to Sift environment configuration and data-plane middleware.
  - path: projects/sift/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: sift-runtime-security-routing
    tracker: "1604"
    description: Apply shared authorization, metrics, and structured error boundaries to Sift data-plane routes.
  - path: projects/sift/src/bin/sift.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: sift-agent-cli-contract
    tracker: "1604"
    description: Add spec client generation, build metadata, and parseable JSON terminal output to the Sift CLI.
  - path: projects/sift/tests/runtime_security_e2e.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    gap: sift-auth-route-contract-tests
    tracker: "1604"
    description: Verify required bearer auth protects data-plane routes while probes remain reachable.
  - path: projects/sift/tests/cli_contract.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    gap: sift-agent-cli-contract-tests
    tracker: "1604"
    description: Verify JSON CLI output parses and spec generation emits a typed-client entrypoint.
```
