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
