---
id: '1644'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: shared-stateful-workload-projection
entry: app_policy
nodes:
  app_policy: { kind: start, label: "App-owned CRD and workload policy" }
  typed_input: { kind: process, label: "Build service-k8s typed projection input" }
  statefulset: { kind: process, label: "Render common StatefulSet with downward API, storage, probes, and security defaults" }
  services: { kind: process, label: "Render headless/client Services and PDB" }
  hpa: { kind: decision, label: "App policy enables HPA?" }
  render_hpa: { kind: process, label: "Render optional HPA without changing durable shard topology" }
  preserve: { kind: terminal, label: "Lumen and Tape retain distinct CRDs and domain policy" }
edges:
  - { from: app_policy, to: typed_input }
  - { from: typed_input, to: statefulset }
  - { from: statefulset, to: services }
  - { from: services, to: hpa }
  - { from: hpa, to: render_hpa, label: "yes" }
  - { from: hpa, to: preserve, label: "no" }
  - { from: render_hpa, to: preserve }
---
flowchart TD
    app_policy([App-owned CRD and workload policy]) --> typed_input[Build service-k8s typed projection input]
    typed_input --> statefulset[Render common StatefulSet]
    statefulset --> services[Render Services and PDB]
    services --> hpa{App policy enables HPA?}
    hpa -->|yes| render_hpa[Render optional HPA]
    hpa -->|no| preserve([Preserve distinct CRDs and domain policy])
    render_hpa --> preserve
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/service-k8s/src/render.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Adopt the already-shared typed StatefulSet, Services, PDB, secure pod defaults, resource-request-only defaults, and optional HPA projection as the canonical service-k8s contract."
  - path: apps/lumen/src/operator/render.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Record Lumen as a consumer of ServiceStatefulSet while retaining its CRD, shard policy, probes, storage, and optional autoscaling decisions."
  - path: apps/tape/src/operator/render.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Record Tape as a consumer of ShardedStatefulSet while retaining its distinct Tape CRD and topic-service policy."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: shared-stateful-workload-projection-verification
requirements:
  distinct_app_adoption:
    id: R3
    text: "Lumen and Tape both consume the shared workload renderer while retaining distinct CRDs, app-specific environment, storage, probes, and domain policy."
    kind: integration
    risk: high
    verify: cargo test -p lumen --features operator --test operator_render && cargo test -p tape --features operator --test operator
  optional_hpa:
    id: R2
    text: "The shared HPA projection is opt-in and remains separate from the durable shard and replica topology encoded by the StatefulSet input."
    kind: regression
    risk: high
    verify: cargo test -p service-k8s
  typed_projection:
    id: R1
    text: "service-k8s renders a typed StatefulSet with the raft-runtime downward-API contract, app-provided storage and probes, secure pod/container defaults, and resource requests without mandatory limits."
    kind: functional
    risk: high
    verify: cargo test -p service-k8s
---
flowchart TD
    r1[R1 typed projection] --> cargo_test_p_service_k8s[cargo test -p service-k8s]
    r2[R2 optional hpa] --> cargo_test_p_service_k8s
    r3[R3 distinct app adoption] --> cargo_test_p_lumen_features_operator_test_operator_render_cargo_test_p_tape_features_operator_test_operator[cargo test -p lumen --features operator --test operator_render && cargo test -p tape --features operator --test operator]
```
