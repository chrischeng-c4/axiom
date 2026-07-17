---
id: '1571'
summary: Enforce a global held-Pod quota invariant independently for each remote PostgreSQL endpoint.
fill_sections: [logic, unit-test]
capability_refs:
  - id: kubernetes-native-deployment
    role: primary
    gap: global-endpoint-quota-allocation
    claim: global-endpoint-quota-allocation
    coverage: full
    rationale: "Prevents Deployment scale and rollout overlap from exceeding usable remote connection capacity."
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-global-endpoint-quota-allocation
entry: facts
nodes:
  facts: { kind: start, label: "Effective limit reserve non-pgpool usage and headroom" }
  usable: { kind: process, label: "Compute saturating usable capacity" }
  request: { kind: process, label: "Atomically evaluate requested Pod quotas" }
  fits: { kind: decision, label: "Existing plus requested held quota fits" }
  reserve: { kind: process, label: "Insert all Pending allocations" }
  blocked: { kind: terminal, label: "Reject all requested allocations with reason" }
  transition: { kind: process, label: "Pending Ready Draining retain quota" }
  release: { kind: terminal, label: "Release only after drain completion" }
edges:
  - { from: facts, to: usable }
  - { from: usable, to: request }
  - { from: request, to: fits }
  - { from: fits, to: reserve, label: "yes" }
  - { from: fits, to: blocked, label: "no" }
  - { from: reserve, to: transition }
  - { from: transition, to: release }
---
flowchart TD
  facts([capacity facts]) --> usable[saturating usable capacity]
  usable --> request[atomic Pod quota request]
  request --> fits{held plus requested fits?}
  fits -->|yes| reserve[insert all Pending allocations]
  fits -->|no| blocked([blocked without partial allocation])
  reserve --> transition[Pending Ready Draining hold quota]
  transition --> release([release after drain complete])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-global-endpoint-quota-allocation-tests
requirements:
  invariant:
    id: R1
    text: "Held Pending Ready and Draining quotas never exceed usable endpoint capacity."
    kind: functional
    risk: high
    verify: cargo test -p pgpool k8s::budget
  atomic_scale:
    id: R2
    text: "Insufficient multi-Pod scale requests fail without partial allocation."
    kind: negative
    risk: high
    verify: cargo test -p pgpool k8s::budget
  drain_hold:
    id: R3
    text: "Draining allocations cannot release before drain completion."
    kind: negative
    risk: high
    verify: cargo test -p pgpool k8s::budget
  isolation:
    id: R4
    text: "Endpoint allocation maps are isolated by endpoint key."
    kind: regression
    risk: high
    verify: cargo test -p pgpool k8s::budget
---
flowchart TD
  r1[R1 invariant] --> unit[cargo test -p pgpool k8s::budget]
  r2[R2 atomic scale] --> unit
  r3[R3 drain hold] --> unit
  r4[R4 endpoint isolation] --> unit
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/k8s/budget.rs
    action: create
    impl_mode: hand-written
    section: logic
    anchor: EndpointAllocator
    description: Implement atomic endpoint reservation and drain-safe release transitions.
```
