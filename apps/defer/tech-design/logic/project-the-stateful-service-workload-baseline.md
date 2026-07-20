---
id: '2170'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: defer-stateful-workload-contract
entry: profile
nodes:
  profile:
    kind: start
    label: "stateful_storage profile requires baseline"
  shared:
    kind: process
    label: "Compose raft-runtime, service-backup, service-auth, and service-k8s"
  domain:
    kind: process
    label: "Reference delayed-task, dispatch, rate-limit, HA, and stability roots"
  contract:
    kind: process
    label: "Publish one non-duplicative capability root"
  verify:
    kind: terminal
    label: "AW capability check and targeted Defer evidence pass"
edges:
  - { from: profile, to: shared }
  - { from: profile, to: domain }
  - { from: shared, to: contract }
  - { from: domain, to: contract }
  - { from: contract, to: verify }
---
flowchart TD
    profile[stateful_storage profile] --> shared[shared mechanisms]
    profile --> domain[Defer authoritative roots]
    shared --> contract[stateful-service-workload]
    domain --> contract
    contract --> verify[capability and targeted gates]
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
