---
id: '1809'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-shared-statefulset-projection-contract
entry: input
nodes:
  input:
    kind: start
    label: "Tape supplies ServiceStatefulSet values; no post-render JSON mutation is allowed"
  shared:
    kind: process
    label: "service-k8s projects standard topology env, labels, selector, service account, affinity, resources, and PVC mount"
  typed:
    kind: process
    label: "Typed inputs project Tape-selected probes, pod/container security, annotations, rollout, tmp, and optional Secret mount"
  tape:
    kind: process
    label: "Tape retains CRD fields, image, ports, TAPE env names, journal storage tier, and auth-secret policy"
  render:
    kind: process
    label: "Render StatefulSet with existing ServiceAccount, headless/client Services, and PDB"
  verify:
    kind: terminal
    label: "Operator test proves byte-level child-shape invariants and source no longer contains harden or ShardedStatefulSet"
edges:
  - { from: input, to: shared }
  - { from: input, to: typed }
  - { from: input, to: tape }
  - { from: shared, to: render }
  - { from: typed, to: render }
  - { from: tape, to: render }
  - { from: render, to: verify }
---
flowchart TD
    input["Typed ServiceStatefulSet input only"] --> shared["Shared StatefulSet projection"]
    input --> typed["Typed generic workload fields"]
    input --> tape["Tape domain/deployment values"]
    shared --> render["Tape children"]
    typed --> render
    tape --> render
    render --> verify(["Structural and source-boundary verification"])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/src/operator/render.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: statefulset
    description: "Compose render::ServiceStatefulSet directly with typed Tape ports, environment, PVC, security, probe, annotation, rollout, volume, and affinity values; delete harden and ShardedStatefulSet usage. generator gap: missing-generator:kubernetes-statefulset-adoption (#1809)."
  - path: apps/tape/tests/operator.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: render_emits_expected_child_objects
    description: "Assert the shared typed projection preserves Tape's StatefulSet fields, including rollout metadata, pod security, tmp volume, and existing auth-secret behavior. generator gap: missing-generator:kubernetes-statefulset-adoption-test (#1809)."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-shared-statefulset-projection-contract
requirements:
  token_secret_is_still_opt_in:
    id: R2
    text: "Tape alone selects token-registry Secret usage, and the shared projection must render no auth environment or Secret mount for an incomplete configuration."
    kind: functional
    risk: medium
    verify: apps/tape/tests/operator.rs::token_registry_secret_wiring_is_opt_in
  typed_shared_statefulset_is_behavior_preserving:
    id: R1
    text: "Refactoring Tape to ServiceStatefulSet preserves the rendered runtime contract: Raft topology env, http/raft ports, durable journal PVC, health probes, secure pod/container context, update policy, and Prometheus annotations."
    kind: regression
    risk: high
    verify: apps/tape/tests/operator.rs::render_emits_expected_child_objects
---
flowchart TD
    r1[R1 typed shared statefulset is behavior preserving] --> apps_tape_tests_operator_rs_render_emits_expected_child_objects[apps/tape/tests/operator.rs::render_emits_expected_child_objects]
    r2[R2 token secret is still opt in] --> apps_tape_tests_operator_rs_token_registry_secret_wiring_is_opt_in[apps/tape/tests/operator.rs::token_registry_secret_wiring_is_opt_in]
```
