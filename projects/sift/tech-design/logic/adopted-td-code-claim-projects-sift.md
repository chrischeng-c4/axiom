---
id: "1619"
summary: Extend Sift's canonical semantic TD with all service-archetype sources and their executable evidence.
capability_refs:
  - id: ec-gates-configured
    role: primary
    gap: behavior-and-claim-closure-manifest
    claim: behavior-and-claim-closure-manifest
    coverage: partial
    rationale: Non-domain implementation sources need a capability-resolvable semantic ownership edge.
  - id: kubernetes-native-deployment
    role: contributes
    gap: crd-operator-instance-render
    claim: crd-operator-instance-render
    coverage: partial
    rationale: Image, Kustomize, and operator sources must close back to the deployed-service capability.
  - id: long-running-stability
    role: contributes
    gap: ingest-query-replay-soak
    claim: ingest-query-replay-soak
    coverage: partial
    rationale: Operational test sources are the executable proof of bounded ingestion and durable recovery.
fill_sections: [logic, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-semantic-ownership-flow
entry: service-archetype-source
nodes:
  source: { kind: start, label: "runtime, deployment, and operational-evidence source" }
  domain: { kind: process, label: "canonical semantic TD records source unit and ownership section" }
  capability: { kind: process, label: "semantic TD resolves to README service capability claims" }
  proof: { kind: process, label: "tests and EC runners remain executable evidence" }
  health: { kind: terminal, label: "semantic and traceability health closes without orphan sources" }
edges:
  - { from: source, to: domain }
  - { from: domain, to: capability }
  - { from: capability, to: proof }
  - { from: proof, to: health }
---
flowchart TD
    source([service-archetype source]) --> domain[canonical semantic TD]
    domain --> capability[README capability claim]
    capability --> proof[tests and EC evidence]
    proof --> health([no orphan source or traceability block])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/sift/tech-design/semantic/sift-projects-sift.md
    action: modify
    section: logic
    impl_mode: hand-written
    gap: sift-service-archetype-semantic-ownership
    tracker: "1619"
    description: Add semantic source and evidence edges for every implemented Sift service-archetype runtime, deployment, and operational test artifact.
```
