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
## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: apps/pgpool/src/k8s/budget.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: EndpointAllocator::reserve_many
    reason: Admit static Pod quota against the allocator quota plus externally-held reserve capacity, preserving the allocator's atomic error and blocked-scale status behavior.
  - path: apps/pgpool/src/k8s/control.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: PgpoolControlPlane::admit_scale
    reason: Pass outstanding reserve-grant units into static scale admission, reject instead of implicitly reclaiming grants, and cover admit/grant/release sequences.
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-reserve-aware-static-admission-verification
requirements:
  combined_capacity_invariant:
    id: R3
    text: "Admit, grant, drain-release, and reserve-release sequences preserve held total at or below usable capacity."
    kind: invariant
    risk: high
    verify: reserve_and_static_sequence_never_exceeds_usable_capacity
  reject_precedence_preserves_physical_reserves:
    id: R2
    text: "A scale-up that would over-commit rejects without reclaiming idle, draining, or expired reserve grants."
    kind: functional
    risk: high
    verify: reserve_aware_static_scale_admission_rejects_overcommit_without_mutation
  static_admission_counts_outstanding_reserves:
    id: R1
    text: "Static scale admission includes every outstanding reserve grant in the endpoint capacity check."
    kind: regression
    risk: high
    verify: reserve_aware_static_scale_admission_rejects_overcommit_without_mutation
---
flowchart TD
    r1[R1 static admission counts outstanding reserves] --> reserve_aware_static_scale_admission_rejects_overcommit_without_mutation[reserve_aware_static_scale_admission_rejects_overcommit_without_mutation]
    r2[R2 reject precedence preserves physical reserves] --> reserve_aware_static_scale_admission_rejects_overcommit_without_mutation
    r3[R3 combined capacity invariant] --> reserve_and_static_sequence_never_exceeds_usable_capacity[reserve_and_static_sequence_never_exceeds_usable_capacity]
```
