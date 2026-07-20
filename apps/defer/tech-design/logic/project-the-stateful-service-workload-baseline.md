---
id: '2170'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: defer-stateful-workload-evidence-contract
entry: root
nodes:
  root:
    kind: start
    label: "stateful-service-workload capability root"
  shared:
    kind: process
    label: "Reference shared raft, backup, auth, and Kubernetes mechanisms"
  local:
    kind: process
    label: "Reference Defer delayed-task lifecycle and existing service evidence"
  validate:
    kind: terminal
    label: "Capability validation reports no missing stateful baseline"
edges:
  - { from: root, to: shared }
  - { from: root, to: local }
  - { from: shared, to: validate }
  - { from: local, to: validate }
---
flowchart TD
    root[Stateful workload root] --> shared[shared library mechanisms]
    root --> local[Defer domain evidence]
    shared --> validate[capability check passes]
    local --> validate
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/defer/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: Add the field-style stateful-service-workload contract and one work root; reference current shared-library paths and existing Defer domain evidence without duplicating their policies.
```
