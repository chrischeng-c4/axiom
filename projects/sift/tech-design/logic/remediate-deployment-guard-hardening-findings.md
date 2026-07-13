---
id: "1616"
summary: Remediate Sift deployment image and pod-security findings reported by Guard.
capability_refs:
  - id: security-hardening
    role: primary
    gap: shared-bearer-token-auth
    claim: shared-bearer-token-auth
    coverage: partial
    rationale: Service hardening evidence includes the deployment boundary that carries the authenticated data plane.
  - id: kubernetes-native-deployment
    role: contributes
    gap: crd-operator-instance-render
    claim: crd-operator-instance-render
    coverage: partial
    rationale: The rendered operator and instance artifacts must satisfy the native Guard posture scan.
fill_sections: [logic, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-deployment-guard-remediation-flow
entry: rendered-deployment-artifact
nodes:
  artifact: { kind: start, label: "Sift Dockerfile or Kubernetes render" }
  ownership: { kind: process, label: "set explicit non-root COPY ownership" }
  image: { kind: process, label: "replace mutable tags with a versioned image reference" }
  pod: { kind: process, label: "set pod and container non-root restricted security contexts" }
  render: { kind: process, label: "verify static and operator-generated manifests remain aligned" }
  guard: { kind: terminal, label: "native Guard scan has zero findings" }
edges:
  - { from: artifact, to: ownership }
  - { from: artifact, to: image }
  - { from: artifact, to: pod }
  - { from: ownership, to: render }
  - { from: image, to: render }
  - { from: pod, to: render }
  - { from: render, to: guard }
---
flowchart TD
    artifact([Sift deployment render]) --> ownership[explicit COPY ownership]
    artifact --> image[versioned image reference]
    artifact --> pod[restricted non-root pod]
    ownership --> render[static and generated rendering align]
    image --> render
    pod --> render
    render --> guard([Guard clean])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/sift/Dockerfile
    action: modify
    section: changes
    impl_mode: hand-written
    gap: sift-dockerfile-copy-ownership
    tracker: "1616"
    description: Apply explicit non-root ownership to build and runtime COPY operations.
  - path: projects/sift/k8s/operator/operator.yaml
    action: modify
    section: changes
    impl_mode: hand-written
    gap: sift-static-operator-security-context
    tracker: "1616"
    description: Pin the operator image and declare restricted non-root pod and container security context.
  - path: projects/sift/k8s/instances/dev.yaml
    action: modify
    section: changes
    impl_mode: hand-written
    gap: sift-dev-image-pin
    tracker: "1616"
    description: Use a versioned development image reference.
  - path: projects/sift/src/deploy.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: sift-rendered-image-pin
    tracker: "1616"
    description: Keep staging and production instance rendering pinned to a versioned Sift image.
  - path: projects/sift/src/operator.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: sift-operator-workload-security-context
    tracker: "1616"
    description: Render restricted non-root security context for Sift workload and backup containers.
  - path: projects/sift/tests/deployment_cli.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    gap: sift-deployment-security-render-tests
    tracker: "1616"
    description: Verify deployment rendering keeps versioned images and non-root security controls.
```
