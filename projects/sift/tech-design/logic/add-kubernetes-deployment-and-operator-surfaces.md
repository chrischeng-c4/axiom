---
id: "1606"
summary: Add reproducible Docker image, layered Kubernetes, and operator artifact surfaces for Sift.
capability_refs:
  - id: kubernetes-native-deployment
    role: primary
    gap: crd-operator-instance-render
    claim: crd-operator-instance-render
    coverage: partial
    rationale: Sift needs independently rendered image, CRD, operator, and instance layers.
  - id: developer-and-agent-experience
    role: contributes
    gap: offline-contract
    claim: offline-contract
    coverage: partial
    rationale: Deployment artifacts must be inspectable offline with runnable continuations.
fill_sections: [logic, changes]
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

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/sift/src/deploy.rs
    action: create
    section: logic
    impl_mode: hand-written
    gap: sift-layered-deployment-renderer
    tracker: "1606"
    description: Render Sift Dockerfile, CRD, operator, and instance artifacts from checked-in templates.
  - path: projects/sift/src/operator.rs
    action: create
    section: logic
    impl_mode: hand-written
    gap: sift-shared-operator-controller
    tracker: "1606"
    description: Define the Sift custom-resource type and compose the shared leader-elected operator reconcile loop.
  - path: projects/sift/src/bin/sift.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: sift-deployment-cli-surface
    tracker: "1606"
    description: Expose dockerfile and layered k8s render/operator commands with executable continuations.
  - path: projects/sift/Dockerfile
    action: create
    section: changes
    impl_mode: hand-written
    gap: sift-source-image-artifact
    tracker: "1606"
    description: Provide the source-build Sift image contract.
  - path: projects/sift/Dockerfile.release
    action: create
    section: changes
    impl_mode: hand-written
    gap: sift-release-image-artifact
    tracker: "1606"
    description: Provide the verified release-binary Sift image contract.
  - path: projects/sift/k8s/crd/sift.yaml
    action: create
    section: changes
    impl_mode: hand-written
    gap: sift-crd-artifact
    tracker: "1606"
    description: Define the cluster-scoped Sift custom resource API with YAML-quoted string enum values so `off` remains an auth-mode string at Kubernetes validation time.
  - path: projects/sift/k8s/operator/operator.yaml
    action: create
    section: changes
    impl_mode: hand-written
    gap: sift-operator-artifact
    tracker: "1606"
    description: Define Sift operator service account, RBAC, and controller deployment.
  - path: projects/sift/k8s/instances/dev.yaml
    action: create
    section: changes
    impl_mode: hand-written
    gap: sift-instance-artifact
    tracker: "1606"
    description: Define the development Sift custom resource with standard probes, single-node topology, and a YAML-quoted `auth: "off"` string.
  - path: projects/sift/HA.md
    action: create
    section: changes
    impl_mode: hand-written
    gap: sift-ha-operations-document
    tracker: "1606"
    description: Document Sift single-node and Raft replica deployment, backup, restore, and failure recovery.
  - path: projects/sift/tests/deployment_cli.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    gap: sift-deployment-cli-tests
    tracker: "1606"
    description: Verify all Dockerfile and layered Kubernetes artifact commands render expected contracts, including the Sift auth enum as a string rather than YAML boolean coercion.
```
