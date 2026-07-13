---
id: '1561'
summary: Compose Pgpool Kubernetes instance artifacts from the shared stateless Deployment and common Service render modules.
fill_sections: [logic, unit-test]
capability_refs:
  - id: kubernetes-native-deployment
    role: primary
    gap: stateless-deployment-instance
    claim: stateless-deployment-instance
    coverage: full
    rationale: "Adds the typed Pgpool instance composition and deterministic negative boundary for the Deployment workload profile."
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-stateless-deployment-applicability
entry: input
nodes:
  input: { kind: start, label: "Pgpool instance inputs" }
  common: { kind: process, label: "Compose shared common Pod template and ClusterIP Service" }
  deployment: { kind: process, label: "Render shared ServiceDeployment with no-surge rollout" }
  reject_stateful: { kind: process, label: "Reject StatefulSet PVC Raft stable identity and ClientIP affinity fields" }
  done: { kind: terminal, label: "Deterministic stateless instance artifacts" }
edges:
  - { from: input, to: common }
  - { from: common, to: deployment }
  - { from: deployment, to: reject_stateful }
  - { from: reject_stateful, to: done }
---
flowchart TD
  input([Pgpool instance inputs]) --> common[shared common Pod template and ClusterIP Service]
  common --> deployment[shared ServiceDeployment with no-surge rollout]
  deployment --> reject_stateful[exclude StatefulSet PVC Raft stable identity and ClientIP affinity]
  reject_stateful --> done([deterministic stateless instance artifacts])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-stateless-deployment-applicability-test
requirements:
  stateless_instance:
    id: R1
    text: "Pgpool renders a shared Deployment and ClusterIP Service without stateful-only fields."
    kind: functional
    risk: high
    verify: cargo test -p pgpool
---
flowchart TD
  r1[R1 stateless instance] --> cargo_test[cargo test -p pgpool]
```
