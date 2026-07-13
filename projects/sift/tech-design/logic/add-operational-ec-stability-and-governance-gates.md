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
