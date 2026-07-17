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
