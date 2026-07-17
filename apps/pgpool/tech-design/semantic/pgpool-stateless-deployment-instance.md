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
  pod_contract:
    id: R2
    text: "The Pgpool Deployment carries probes, preStop drain, termination grace, resources, security, topology spread, and no-surge rollout settings."
    kind: functional
    risk: high
    verify: cargo test -p pgpool k8s::instance
  negative_boundary:
    id: R3
    text: "Rendered YAML contains no StatefulSet, PVC, stable identity, Raft topology environment, or ClientIP affinity contract."
    kind: negative
    risk: high
    verify: cargo test -p pgpool k8s::instance
  cli_artifact:
    id: R4
    text: "pgpool k8s instance render emits deterministic stateless instance YAML for every supported profile."
    kind: regression
    risk: medium
    verify: cargo test -p pgpool --test cli_contract k8s_instance_render_is_a_stateless_deployment
---
flowchart TD
  r1[R1 stateless instance] --> cargo_test[cargo test -p pgpool]
  r2[R2 Pod and rollout contract] --> k8s_test[cargo test -p pgpool k8s::instance]
  r3[R3 negative stateful boundary] --> k8s_test
  r4[R4 CLI artifact] --> cli_test[cargo test -p pgpool --test cli_contract k8s_instance_render_is_a_stateless_deployment]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/Cargo.toml
    action: modify
    impl_mode: hand-written
    section: logic
    description: Depend on the shared operator renderer crate.
  - path: apps/pgpool/src/lib.rs
    action: modify
    impl_mode: hand-written
    section: logic
    description: Export the Pgpool Kubernetes composition module.
  - path: apps/pgpool/src/k8s/mod.rs
    action: create
    impl_mode: hand-written
    section: logic
    description: Publish typed Pgpool Kubernetes instance rendering.
  - path: apps/pgpool/src/k8s/instance.rs
    action: create
    impl_mode: hand-written
    section: logic
    description: Compose common Service and Deployment primitives with the stateless Pgpool Pod contract.
  - path: apps/pgpool/src/bin/pgpool.rs
    action: modify
    impl_mode: hand-written
    section: logic
    description: Add pgpool k8s instance render profile and output handling.
  - path: apps/pgpool/tests/cli_contract.rs
    action: modify
    impl_mode: hand-written
    section: unit-test
    description: Verify the CLI emits Deployment artifacts without stateful or sticky-session fields.
```
