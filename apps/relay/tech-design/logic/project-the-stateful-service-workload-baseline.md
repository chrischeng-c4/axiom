---
id: '1555'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-stateful-service-workload-projection
entry: trait
nodes:
  trait:
    kind: start
    label: "Relay aw.toml declares stateful_storage"
  project:
    kind: process
    label: "AW derives the stateful-service-workload baseline"
  root:
    kind: process
    label: "README root links shared mechanisms with Relay-local evidence"
  check:
    kind: decision
    label: "capability check finds a complete root?"
  ready:
    kind: terminal
    label: "Relay baseline is present without duplicating domain capability prose"
  missing:
    kind: terminal
    label: "Report the missing root and the remediation command"
edges:
  - { from: trait, to: project }
  - { from: project, to: root }
  - { from: root, to: check }
  - { from: check, to: ready, label: "yes" }
  - { from: check, to: missing, label: "no" }
---
flowchart LR
    trait[Relay stateful_storage trait] --> project[AW baseline projection]
    project --> root[README stateful-service-workload root]
    root --> check{capability check complete?}
    check -->|yes| ready[shared mechanisms and Relay policy are linked]
    check -->|no| missing[emit root-specific remediation]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/relay/aw.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: Declare the stateful_storage trait so the shared stateful-service-workload baseline is required for Relay.
  - path: apps/relay/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: Add the canonical stateful-service-workload capability root that links existing durable acknowledgement, stable StatefulSet identity, raft topology, backup and restore, peer security, and deployment evidence without copying the domain roots.
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
