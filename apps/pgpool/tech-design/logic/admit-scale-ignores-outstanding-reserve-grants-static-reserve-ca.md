---
id: '1888'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-reserve-aware-static-admission
entry: admit_scale
nodes:
  admit_scale: { kind: start, label: "Scale admission receives desired and surge Pods." }
  preflight: { kind: decision, label: "Do static held, outstanding reserve, and requested static quota fit usable capacity?" }
  reject: { kind: process, label: "Reject without reclaiming reserve grants or mutating allocator, ledger, or Pods." }
  reserve: { kind: process, label: "Atomically reserve all static Pods in the endpoint allocator." }
  refresh: { kind: process, label: "Refresh ledger base held and record pending Pod status." }
  invariant: { kind: terminal, label: "Release builds and sequence tests preserve held total at or below usable capacity." }
edges:
  - { from: admit_scale, to: preflight }
  - { from: preflight, to: reject, label: "no" }
  - { from: preflight, to: reserve, label: "yes" }
  - { from: reserve, to: refresh }
  - { from: refresh, to: invariant }
---
flowchart TD
    admit_scale([admit_scale receives desired and surge Pods]) --> preflight{static held + outstanding reserve + request <= usable?}
    preflight -->|no| reject[Reject without reclaiming reserve or mutating state]
    preflight -->|yes| reserve[Atomically reserve static Pods]
    reserve --> refresh[Refresh ledger base and record pending Pods]
    refresh --> invariant([Preserve held_total <= usable])
```

Static Pod admission has precedence only over new Pods: outstanding reserve grants remain held until their normal close/release transition. The control plane rejects a scale-up that would exceed `usable`; it does not implicitly reclaim idle, draining, or expired reserves, because each may still represent a physical backend connection. The preflight is mutation-free, then `reserve_many` remains the atomic static-allocation operation.
