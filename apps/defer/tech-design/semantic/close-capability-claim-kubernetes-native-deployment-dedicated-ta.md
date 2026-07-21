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

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: defer-kubernetes-topology-verification
requirements:
  composed_direct_topology:
    id: R2
    text: "Both the composed direct base and production overlay contain the exact connected StatefulSet, headless and client Services, PDB, probes, PVC, security, monitoring, and network-policy invariants, with no voter HPA."
    kind: regression
    risk: high
    verify: cargo test -p defer --features operator --test direct_k8s_assets --test operator -- --nocapture
  generated_ec_inventory:
    id: R6
    text: "The accepted CLI, rendered-topology, and real Kind recovery external-contract cases remain generated as production-required fail-closed runners bound to dedicated-task-service-topology."
    kind: regression
    risk: medium
    verify: aw ec check --project defer
  kind_pvc_recovery:
    id: R4
    text: "A disposable Kind cluster runs the current image and operator, verifies the reconciled topology and bound PVC, recovers queue and task state after a different pod UID takes over, accepts post-recovery mutations, and is deleted on success."
    kind: stability
    risk: high
    verify: bash apps/defer/scripts/kind-e2e.sh
  layered_cli_artifacts:
    id: R1
    text: "The Defer CLI independently renders version-bound source and release Dockerfiles, a structural CRD, a namespaced operator, and a production instance with three replicas per shard and backup policy."
    kind: functional
    risk: medium
    verify: cargo test -p defer --test cli_contract deploy_artifacts_render_by_lifecycle_layer -- --nocapture
  operator_resource_graph:
    id: R3
    text: "The feature-enabled operator renders exactly six production objects with a three-replica StatefulSet, exact peer and client services, PDB, PVC and probes, connected token signing and peer-TLS secrets, and an exact scheduled backup job."
    kind: functional
    risk: high
    verify: cargo test -p defer --features operator --test direct_k8s_assets --test operator -- --nocapture
  shared_reconciliation_boundary:
    id: R5
    text: "Defer contributes only its CR schema, domain defaults, and policy while the generic StatefulSet, Service, PDB, probe, storage, security, and backup composition remains provided by service-k8s primitives."
    kind: architecture
    risk: medium
    verify: cargo test -p defer --features operator --test operator production_render_composes_shared_stateful_primitives -- --nocapture
---
flowchart TD
    r1[R1 layered cli artifacts] --> cargo_test_p_defer_test_cli_contract_deploy_artifacts_render_by_lifecycle_layer_nocapture[cargo test -p defer --test cli_contract deploy_artifacts_render_by_lifecycle_layer -- --nocapture]
    r2[R2 composed direct topology] --> cargo_test_p_defer_features_operator_test_direct_k8s_assets_test_operator_nocapture[cargo test -p defer --features operator --test direct_k8s_assets --test operator -- --nocapture]
    r3[R3 operator resource graph] --> cargo_test_p_defer_features_operator_test_direct_k8s_assets_test_operator_nocapture
    r4[R4 kind pvc recovery] --> bash_apps_defer_scripts_kind_e2e_sh[bash apps/defer/scripts/kind-e2e.sh]
    r5[R5 shared reconciliation boundary] --> cargo_test_p_defer_features_operator_test_operator_production_render_composes_shared_stateful_primitives_nocapture[cargo test -p defer --features operator --test operator production_render_composes_shared_stateful_primitives -- --nocapture]
    r6[R6 generated ec inventory] --> aw_ec_check_project_defer[aw ec check --project defer]
```
