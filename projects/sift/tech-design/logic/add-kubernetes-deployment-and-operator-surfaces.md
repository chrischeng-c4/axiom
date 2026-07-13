---
id: "1606"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-kubernetes-render-flow
entry: command
nodes:
  command: { kind: start, label: "sift deployment CLI command" }
  dockerfile: { kind: process, label: "render source or verified release image Dockerfile" }
  layer: { kind: decision, label: "CRD, operator, or instance layer?" }
  crd: { kind: process, label: "render cluster-scoped Sift CRD" }
  operator: { kind: process, label: "render RBAC, service account, controller deployment, and config" }
  instance: { kind: process, label: "render namespaced Sift workload, service, PVC, probes, and topology env" }
  apply: { kind: terminal, label: "emit artifact with kubectl continuation" }
edges:
  - { from: command, to: dockerfile, label: "dockerfile render" }
  - { from: command, to: layer, label: "k8s" }
  - { from: dockerfile, to: apply }
  - { from: layer, to: crd, label: "crd" }
  - { from: layer, to: operator, label: "operator" }
  - { from: layer, to: instance, label: "instance" }
  - { from: crd, to: apply }
  - { from: operator, to: apply }
  - { from: instance, to: apply }
---
flowchart TD
    command([sift CLI]) --> dockerfile[render Dockerfile]
    command --> layer{Kubernetes layer}
    dockerfile --> apply([artifact plus next step])
    layer -->|crd| crd[cluster CRD]
    layer -->|operator| operator[controller/RBAC]
    layer -->|instance| instance[service workload]
    crd --> apply
    operator --> apply
    instance --> apply
```
