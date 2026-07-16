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
    description: Replace the ShardedStatefulSet-plus-harden JSON mutation path with one ServiceStatefulSet input that declares Tape-owned values and shared workload fields.
  - path: apps/tape/tests/operator.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Extend structural render evidence for the typed common workload fields while preserving Tape port, PVC, probe, security, and token-registry behavior.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-shared-statefulset-projection-verification
requirements:
  optional_auth_secret_remains_tape_policy:
    id: R2
    text: "The shared projection retains Tape's opt-in token-registry Secret mount and never enables it for an incomplete auth configuration."
    kind: functional
    risk: medium
    verify: apps/tape/tests/operator.rs::token_registry_secret_wiring_is_opt_in
  shared_projection_preserves_tape_workload_contract:
    id: R1
    text: "Tape's typed ServiceStatefulSet adoption preserves the public and peer ports, durable PVC, standard probes, secure non-root/read-only pod settings, rollout policy, and shared topology environment contract."
    kind: regression
    risk: medium
    verify: apps/tape/tests/operator.rs::render_emits_expected_child_objects
---
flowchart TD
    r1[R1 shared projection preserves tape workload contract] --> apps_tape_tests_operator_rs_render_emits_expected_child_objects[apps/tape/tests/operator.rs::render_emits_expected_child_objects]
    r2[R2 optional auth secret remains tape policy] --> apps_tape_tests_operator_rs_token_registry_secret_wiring_is_opt_in[apps/tape/tests/operator.rs::token_registry_secret_wiring_is_opt_in]
```
