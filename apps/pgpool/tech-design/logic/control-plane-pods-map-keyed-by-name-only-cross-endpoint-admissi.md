---
id: '1889'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-endpoint-scoped-pod-control
entry: admit_scale
nodes:
  admit: { kind: start, label: "Admit static allocation for one endpoint and Pod name." }
  key: { kind: process, label: "Store control status under the endpoint plus Pod composite key." }
  endpoint_action: { kind: process, label: "Ready, observe, drain, and release receive the same endpoint and Pod identity." }
  allocator: { kind: process, label: "Route allocator operation only to the keyed endpoint." }
  released: { kind: terminal, label: "Every endpoint allocation can be drained and released without orphaning quota." }
edges:
  - { from: admit, to: key }
  - { from: key, to: endpoint_action }
  - { from: endpoint_action, to: allocator }
  - { from: allocator, to: released }
---
flowchart TD
    admit([Admit endpoint and Pod allocation]) --> key[Key control state by endpoint plus Pod]
    key --> endpoint_action[Use endpoint plus Pod for ready observe drain release]
    endpoint_action --> allocator[Route to the matching endpoint allocator]
    allocator --> released([No endpoint retains orphaned quota])
```

A Pod name is reusable across endpoints because a single Deployment Pod may serve several remote endpoints. `PgpoolControlPlane` therefore keys its internal control record by `(endpoint, pod)`, and every Pod lifecycle API accepts that composite identity. Same-endpoint re-admission remains an `AllocationError::DuplicatePod`; cross-endpoint admission is intentional. The control plane never searches by Pod name alone, so drain and release target exactly the allocator that admitted the allocation.
## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: apps/pgpool/src/k8s/control.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: admit_scale
    reason: Key pod control state by endpoint plus Pod, require endpoint-scoped lifecycle routing, and test cross-endpoint release.
  - path: apps/pgpool/tests/operator.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: status_projects_global_budget_and_managed_readiness
    reason: Migrate operator control-plane status coverage to the explicit endpoint-scoped Pod lifecycle API.
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-endpoint-scoped-pod-control-verification
requirements:
  cross_endpoint_pod_drain_and_release:
    id: R1
    text: "A Pod name admitted on two endpoints can be drained and released from both endpoint allocators without orphaned quota."
    kind: regression
    risk: high
    verify: endpoint_scoped_pod_lifecycle_releases_cross_endpoint_allocations
  duplicate_semantics_are_deliberate:
    id: R2
    text: "Same-endpoint re-admission remains duplicate while cross-endpoint admission uses distinct control keys."
    kind: functional
    risk: medium
    verify: endpoint_scoped_pod_lifecycle_releases_cross_endpoint_allocations
---
flowchart TD
    r1[R1 cross endpoint pod drain and release] --> endpoint_scoped_pod_lifecycle_releases_cross_endpoint_allocations[endpoint_scoped_pod_lifecycle_releases_cross_endpoint_allocations]
    r2[R2 duplicate semantics are deliberate] --> endpoint_scoped_pod_lifecycle_releases_cross_endpoint_allocations
```
