---
id: '1555'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-stateful-workload-evidence-contract
entry: root
nodes:
  root:
    kind: start
    label: "stateful-service-workload capability root"
  shared:
    kind: process
    label: "Reference shared mechanisms, not copied implementation prose"
  local:
    kind: process
    label: "Reference Relay policy and existing domain evidence"
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
    root --> local[Relay domain evidence]
    shared --> validate[capability check passes]
    local --> validate
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/relay/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: Add the field-style stateful-service-workload contract and one work root; keep shared-library links and Relay evidence as references to their existing authoritative roots.
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-stateful-service-workload-verification
requirements:
  baseline_links_authoritative_evidence:
    id: R2
    text: "The projected root links existing shared mechanisms and Relay-local durable, topology, backup, security, and deployment evidence without duplicating domain capability prose."
    kind: regression
    risk: low
    verify: apps/relay/README.md stateful-service-workload capability contract
  stateful_workload_root_is_projected:
    id: R1
    text: "Relay exposes the stateful-service-workload capability derived from its stateful_storage trait."
    kind: functional
    risk: low
    verify: aw capability check --project relay --skip-issue-inventory
---
flowchart TD
    r1[R1 stateful workload root is projected] --> aw_capability_check_project_relay_skip_issue_inventory[aw capability check --project relay --skip-issue-inventory]
    r2[R2 baseline links authoritative evidence] --> apps_relay_readme_md_stateful_service_workload_capability_contract[apps/relay/README.md stateful-service-workload capability contract]
```
