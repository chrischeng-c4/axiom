---
id: '1809'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-shared-statefulset-projection
entry: cr
nodes:
  cr:
    kind: start
    label: "Tape CRD supplies image, journal PVC tier, auth secret, and Tape/Raft ports"
  map:
    kind: process
    label: "Tape maps only domain and deployment-policy values"
  input:
    kind: process
    label: "Construct service_k8s::render::ServiceStatefulSet input"
  shared:
    kind: process
    label: "service-k8s assembles StatefulSet, topology env, PVC mount, labels, and selector"
  policy:
    kind: process
    label: "Typed common fields carry probes, security, annotations, rollout, tmp, and Secret volumes"
  children:
    kind: process
    label: "Tape emits StatefulSet plus its ServiceAccount, Services, and PDB"
  invariant:
    kind: terminal
    label: "Keep http:7137, raft:7138, one shard, /data PVC, and optional auth mount unchanged"
edges:
  - { from: cr, to: map }
  - { from: map, to: input }
  - { from: input, to: shared }
  - { from: input, to: policy }
  - { from: shared, to: children }
  - { from: policy, to: children }
  - { from: children, to: invariant }
---
flowchart LR
    cr["Tape CRD supplies image, journal PVC tier, auth secret, and Tape/Raft ports"] --> map["Tape maps only domain and deployment-policy values"]
    map --> input["Construct ServiceStatefulSet input"]
    input --> shared["service-k8s assembles common StatefulSet"]
    input --> policy["Typed common workload fields"]
    shared --> children["Tape child objects"]
    policy --> children
    children --> invariant(["Keep ports, PVC, topology, and optional auth mount unchanged"])
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
