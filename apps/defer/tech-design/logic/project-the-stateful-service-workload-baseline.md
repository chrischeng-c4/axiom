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

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: defer-stateful-workload-contract-verification
requirements:
  contract_composes_existing_evidence:
    id: R1
    text: "Defer's stateful workload capability names shared mechanisms and Defer-local durable scheduling, StatefulSet identity, Raft, backup, security, and lifecycle evidence through links to their authoritative roots."
    kind: regression
    risk: low
    verify: aw capability check --project defer --skip-issue-inventory
  referenced_runtime_evidence_remains_executable:
    id: R2
    text: "The existing Defer Raft scheduler, Kubernetes workload, and shared authorization evidence named by the capability contract remains executable."
    kind: regression
    risk: medium
    verify: cargo test -p defer --test direct_k8s_assets --test raft_scheduler --test service_auth
---
flowchart TD
    r1[R1 contract composes existing evidence] --> aw_capability_check_project_defer_skip_issue_inventory[aw capability check --project defer --skip-issue-inventory]
    r2[R2 referenced runtime evidence remains executable] --> cargo_test_p_defer_test_direct_k8s_assets_test_raft_scheduler_test_service_auth[cargo test -p defer --test direct_k8s_assets --test raft_scheduler --test service_auth]
```
