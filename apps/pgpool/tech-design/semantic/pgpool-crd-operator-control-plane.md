---
id: '1575'
summary: Reconcile a typed Pgpool custom resource into stateless shared operator artifacts and project readiness plus global endpoint connection-budget status.
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-crd-operator-control-plane
entry: custom_resource
nodes:
  custom_resource: { kind: start, label: "Pgpool CR image replicas backend endpoint budgets" }
  reconcile: { kind: process, label: "Shared ManagedService reconcile" }
  render: { kind: process, label: "Render ServiceAccount Deployment ClusterIP Service and PDB" }
  readiness: { kind: process, label: "Read Deployment ReadyFacts" }
  discovery: { kind: process, label: "Discover runtime endpoint connection facts" }
  admission: { kind: decision, label: "Atomically admit desired and rollout Pod quotas" }
  limit: { kind: process, label: "Set per-Pod PGPOOL_MAX_BACKEND_CONNECTIONS" }
  drain: { kind: process, label: "Remove readiness drain and retain quota" }
  status: { kind: terminal, label: "Patch Pgpool readiness and endpoint budget status" }
edges:
  - { from: custom_resource, to: reconcile }
  - { from: reconcile, to: render }
  - { from: render, to: readiness }
  - { from: custom_resource, to: discovery }
  - { from: discovery, to: admission }
  - { from: admission, to: limit, label: "capacity available" }
  - { from: limit, to: drain }
  - { from: readiness, to: status }
  - { from: admission, to: status, label: "blocked" }
  - { from: drain, to: status }
---
```
