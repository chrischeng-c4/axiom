---
id: '1890'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-truthful-reconcile-status
entry: reconcile
nodes:
  list: { kind: start, label: "List actual Deployment Pods selected for the Pgpool instance." }
  observe: { kind: process, label: "Extract each observed Pod name, readiness, and termination request." }
  admission: { kind: process, label: "Calculate the safe Deployment replica target and held static capacity." }
  project: { kind: process, label: "Project only observed Pod identities and readiness into every endpoint status." }
  reserve: { kind: decision, label: "Is a live reserve ledger reconciled?" }
  omit: { kind: process, label: "Omit reserve counters and set reserve accounting available false." }
  status: { kind: terminal, label: "Publish truthful status without fabricated Pods or unenacted drain states." }
edges:
  - { from: list, to: observe }
  - { from: observe, to: admission }
  - { from: admission, to: project }
  - { from: project, to: reserve }
  - { from: reserve, to: omit, label: "no" }
  - { from: reserve, to: status, label: "yes" }
  - { from: omit, to: status }
---
flowchart TD
    list([List actual selected Pods]) --> observe[Observe name and readiness]
    observe --> admission[Calculate safe replica target]
    admission --> project[Project only observed Pod identities]
    project --> reserve{Live reserve ledger?}
    reserve -->|no| omit[Omit reserve counters and mark unavailable]
    reserve -->|yes| status([Publish live reserve accounting])
    omit --> status
```

Reconcile status is observational: only Pods returned by the Deployment selector are named in status, and a Pod is `Ready` only when its Kubernetes Ready condition is true; all other observed Pods are `Pending`. The capacity plan continues to hold quota for the current target and observed Pods during a scale-in, but it no longer fabricates per-index Pod records or claims a `Draining` phase without runtime confirmation. The live operator has no reserve-ledger reconciliation yet, so CR reserve counters are omitted and `reserveAccountingAvailable` is false. The pure control-plane model remains able to project real reserve values when it owns a ledger.

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: apps/pgpool/src/operator/reconcile.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: plan_capacity
    reason: Project only selected Pod names and observed readiness into plan context; remove fabricated drain records and explicitly mark reserve accounting unavailable.
  - path: apps/pgpool/src/k8s/control.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: status
    reason: Distinguish control-plane reserve ledger availability from endpoint discovery availability in the shared status context.
  - path: apps/pgpool/src/operator/crd.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: from_control_plane
    reason: Omit reserve counters when no live reserve ledger exists and expose whether reserve accounting is available.
  - path: apps/pgpool/tests/reconcile_planning.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: context_aware_status_projects_capacity_plan
    reason: Verify CR status omits unsupported reserve counters and exposes unavailable reserve accounting.
  - path: apps/pgpool/tests/operator.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: concurrent_pods_cannot_overgrant_reserve_capacity
    reason: Preserve coverage for reserve counters when the pure control plane does own a reserve ledger.
```
