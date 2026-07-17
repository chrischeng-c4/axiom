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
