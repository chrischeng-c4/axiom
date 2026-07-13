---
id: "1607"
summary: Add service-level operational contracts, stability evidence, and governance validation for Sift.
capability_refs:
  - id: ec-gates-configured
    role: primary
    gap: behavior-and-claim-closure-manifest
    claim: behavior-and-claim-closure-manifest
    coverage: partial
    rationale: Sift needs executable security, CLI, and resilience contracts rather than source-only claims.
  - id: long-running-stability
    role: primary
    gap: ingest-query-replay-soak
    claim: ingest-query-replay-soak
    coverage: partial
    rationale: Readiness, drain, bounded ingestion, and process lifecycle need regression proof.
  - id: developer-and-agent-experience
    role: contributes
    gap: interactive-tooling
    claim: interactive-tooling
    coverage: partial
    rationale: Agents need a supported local-cluster connection lifecycle and clean producer configuration.
fill_sections: [logic, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-operational-evidence-flow
entry: local-or-cluster-service
nodes:
  service: { kind: start, label: "Sift real process or Kubernetes service" }
  connect: { kind: process, label: "shared cli-std resolves secret, port-forwards, and scopes command environment" }
  auth: { kind: process, label: "security contract exercises required-token protected data plane" }
  stability: { kind: process, label: "repeatable readiness, drain, bounded ingest, and resource scenarios" }
  runner: { kind: process, label: "vat runner coordinates local real service evidence" }
  policy: { kind: process, label: "guard, meter, and rig configurations describe policy and measurement" }
  ec: { kind: process, label: "AW external contracts generate and verify executable tests" }
  health: { kind: terminal, label: "clean config, EC, test, and readiness gates" }
edges:
  - { from: service, to: connect, label: "cluster access" }
  - { from: service, to: auth, label: "security behavior" }
  - { from: service, to: stability, label: "operational behavior" }
  - { from: connect, to: runner }
  - { from: auth, to: runner }
  - { from: stability, to: runner }
  - { from: runner, to: policy }
  - { from: policy, to: ec }
  - { from: ec, to: health }
---
flowchart TD
    service([Sift process or service]) --> connect[shared connect lifecycle]
    service --> auth[authenticated data-plane contract]
    service --> stability[readiness drain bounded-ingest scenarios]
    connect --> runner[vat real-service runner]
    auth --> runner
    stability --> runner
    runner --> policy[guard meter rig policy]
    policy --> ec[AW EC generation and verification]
    ec --> health([clean governance and health gates])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/sift/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    gap: sift-shared-connect-dependency
    tracker: "1607"
    description: Enable cli-std Kubernetes connection lifecycle support for the Sift CLI.
  - path: projects/sift/src/bin/sift.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: sift-connect-command
    tracker: "1607"
    description: Expose the supported Sift cluster connect command using the shared port-forward and token-resolution lifecycle.
  - path: projects/sift/tests/operational_cli.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    gap: sift-cli-operational-contract
    tracker: "1607"
    description: Verify CLI standard surfaces, connect help, and parseable terminal output contracts.
  - path: projects/sift/tests/stability_e2e.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    gap: sift-stability-evidence
    tracker: "1607"
    description: Exercise readiness, drain, bounded ingestion, and journal recovery through the Sift service router.
  - path: projects/sift/vat.toml
    action: create
    section: changes
    impl_mode: hand-written
    gap: sift-vat-real-service-runner
    tracker: "1607"
    description: Define the COW workspace and real Sift service runner used by operational evidence.
  - path: projects/sift/guard.toml
    action: create
    section: changes
    impl_mode: hand-written
    gap: sift-guard-security-contract
    tracker: "1607"
    description: Bind Sift bearer-auth and probe-exemption evidence to the security guard surface.
  - path: projects/sift/meter-stability.toml
    action: create
    section: changes
    impl_mode: hand-written
    gap: sift-meter-stability-contract
    tracker: "1607"
    description: Bind repeatable Sift stability tests to the meter gate surface.
  - path: projects/sift/external-contracts/security-hardening/sift-auth.md
    action: create
    section: changes
    impl_mode: hand-written
    gap: sift-auth-external-contract
    tracker: "1607"
    description: Declare the bearer-auth and operational-probe external contract.
  - path: projects/sift/external-contracts/long-running-stability/sift-resilience.md
    action: create
    section: changes
    impl_mode: hand-written
    gap: sift-resilience-external-contract
    tracker: "1607"
    description: Declare bounded-ingest, drain, and recovery evidence for the stability gate.
```
