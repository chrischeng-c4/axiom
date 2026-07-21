---
id: '2220'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: defer-kubernetes-topology-verification
entry: render_layers
nodes:
  render_layers: { kind: start, label: "render Dockerfile CRD operator and instance layers through Defer CLI" }
  compose_direct: { kind: process, label: "compose direct base and production Kustomize resources" }
  direct_ok: { kind: decision, label: "exact StatefulSet Services PDB PVC probes and security connected?" }
  render_operator: { kind: process, label: "render operator-owned production resource graph" }
  operator_ok: { kind: decision, label: "exact three-replica topology secrets and backup wiring?" }
  deploy_kind: { kind: process, label: "deploy current image CRD operator and Defer CR to disposable Kind" }
  mutate_state: { kind: process, label: "create queue and tasks through public API" }
  replace_pod: { kind: process, label: "delete pod and wait for different ready UID on bound PVC" }
  recover_ok: { kind: decision, label: "state recovered and post-recovery mutations commit?" }
  cleanup_ok: { kind: decision, label: "disposable cluster deleted and absent?" }
  fail: { kind: terminal, label: "Kubernetes topology claim fails closed" }
  verified: { kind: terminal, label: "dedicated task service topology verified" }
  shared: { kind: terminal, label: "generic reconciliation remains service-k8s owned" }
edges:
  - { from: render_layers, to: compose_direct }
  - { from: compose_direct, to: direct_ok }
  - { from: direct_ok, to: render_operator, label: "yes" }
  - { from: direct_ok, to: fail, label: "no" }
  - { from: render_operator, to: operator_ok }
  - { from: operator_ok, to: deploy_kind, label: "yes" }
  - { from: operator_ok, to: fail, label: "no" }
  - { from: deploy_kind, to: mutate_state }
  - { from: mutate_state, to: replace_pod }
  - { from: replace_pod, to: recover_ok }
  - { from: recover_ok, to: cleanup_ok, label: "yes" }
  - { from: recover_ok, to: fail, label: "no" }
  - { from: cleanup_ok, to: verified, label: "yes" }
  - { from: cleanup_ok, to: fail, label: "no" }
  - { from: render_layers, to: shared, label: "ownership boundary" }
---
flowchart TD
    render_layers([render CLI deployment layers]) --> compose_direct[compose direct base and prod]
    compose_direct --> direct_ok{exact connected direct topology?}
    direct_ok -->|yes| render_operator[render operator production graph]
    direct_ok -->|no| fail([claim fails closed])
    render_operator --> operator_ok{exact operator graph and secrets?}
    operator_ok -->|yes| deploy_kind[deploy current image and operator to Kind]
    operator_ok -->|no| fail
    deploy_kind --> mutate_state[create queue and scheduled tasks]
    mutate_state --> replace_pod[replace pod on bound PVC]
    replace_pod --> recover_ok{state recovered and mutable?}
    recover_ok -->|yes| cleanup_ok{cluster deleted and absent?}
    recover_ok -->|no| fail
    cleanup_ok -->|yes| verified([topology claim verified])
    cleanup_ok -->|no| fail
    render_layers -->|ownership boundary| shared([service-k8s owns generic reconciliation])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/defer/tests/direct_k8s_assets.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: prod_profile_renders_the_connected_security_boundary
    reason: "Own exact named Service and PDB invariants from both composed direct base and production Kustomize resource sets, so disconnected resources fail."
  - path: apps/defer/tests/operator.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: production_render_composes_shared_stateful_primitives
    reason: "Own the exact six-object production graph, three-replica StatefulSet relationships, connected security secrets, and backup CronJob oracle."
  - path: apps/defer/scripts/kind-e2e.sh
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: assert_operator_topology
    reason: "Own the real disposable Kind topology, PVC-backed pod replacement, recovered task state, post-recovery mutation, and cleanup journey."
```
