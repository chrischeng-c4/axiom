---
id: '2152'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: replace-placeholder-kubernetes-operator-with-real-reconciliation
entry: start
nodes:
  start: { kind: start, label: "Reconcile Request" }
  fetch_cr: { kind: process, label: "Fetch Beam CR" }
  is_deleted: { kind: decision, label: "Is CR marked for deletion?" }
  cleanup: { kind: process, label: "Cleanup owned resources" }
  remove_finalizer: { kind: process, label: "Remove finalizer" }
  ensure_finalizer: { kind: process, label: "Ensure finalizer is present" }
  apply_resources: { kind: process, label: "Apply Deployment and Service" }
  publish_status: { kind: process, label: "Update status and observedGeneration" }
  done: { kind: terminal, label: "End Reconcile" }
edges:
  - { from: start, to: fetch_cr }
  - { from: fetch_cr, to: is_deleted }
  - { from: is_deleted, to: cleanup, label: "Yes" }
  - { from: cleanup, to: remove_finalizer }
  - { from: remove_finalizer, to: done }
  - { from: is_deleted, to: ensure_finalizer, label: "No" }
  - { from: ensure_finalizer, to: apply_resources }
  - { from: apply_resources, to: publish_status }
  - { from: publish_status, to: done }
---
flowchart TD
    start([Reconcile Request]) --> fetch_cr[Fetch Beam CR]
    fetch_cr --> is_deleted{Is CR marked for deletion?}
    is_deleted -->|Yes| cleanup[Cleanup owned resources]
    cleanup --> remove_finalizer[Remove finalizer]
    remove_finalizer --> done([End Reconcile])
    is_deleted -->|No| ensure_finalizer[Ensure finalizer is present]
    ensure_finalizer --> apply_resources[Apply Deployment and Service]
    apply_resources --> publish_status[Update status and observedGeneration]
    publish_status --> done
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/beam/src/main.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: K8sOperatorCmd::Run
  - path: apps/beam/src/dx.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: pub fn render_instance_yaml
  - path: apps/beam/src/operator/mod.rs
    action: create
    section: logic
    impl_mode: hand-written
  - path: apps/beam/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: "[dependencies]"
  - path: apps/beam/k8s/operator/rbac.yaml
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: rules
  - path: apps/beam/tests/operator_reconcile.rs
    action: create
    section: unit-test
    impl_mode: hand-written
```
