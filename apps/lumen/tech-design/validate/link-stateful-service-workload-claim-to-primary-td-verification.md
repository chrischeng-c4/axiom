---
id: '2144'
capability_refs:
  - id: "stateful-service-workload"
    role: primary
    gap: "stateful-service-workload-projection"
    claim: "stateful-service-workload-projection"
    coverage: full
    rationale: "WI #2144 supplies the missing primary TD verification linkage for the already-implemented Lumen stateful-service workload projection without changing runtime behavior."
summary: >
  Link Lumen's existing stateful-service-workload projection to primary TD
  verification while retaining closed WI #1553 as historical provenance.
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-stateful-service-workload-primary-verification
entry: inspect_claim
nodes:
  inspect_claim:
    kind: start
    label: "Inspect the stateful-service-workload-projection capability claim"
  preserve_history:
    kind: process
    label: "Keep closed WI #1553 as historical capability-projection provenance"
  bind_td:
    kind: process
    label: "Bind WI #2144 TD as the primary verification linkage for the existing claim"
  verify_contract:
    kind: process
    label: "Run TD checks and the existing Lumen stateful capability gate without runtime changes"
  linked:
    kind: decision
    label: "Does the capability goal discover the primary TD verification linkage?"
  revise:
    kind: process
    label: "Revise only TD capability metadata or producer-owned provenance"
  complete:
    kind: terminal
    label: "Lumen capability goal advances to runtime verification"
edges:
  - { from: inspect_claim, to: preserve_history }
  - { from: preserve_history, to: bind_td }
  - { from: bind_td, to: verify_contract }
  - { from: verify_contract, to: linked }
  - { from: linked, to: complete, label: "yes" }
  - { from: linked, to: revise, label: "no" }
  - { from: revise, to: verify_contract }
---
flowchart TD
  inspect_claim([Inspect stateful-service-workload-projection]) --> preserve_history[Keep closed WI #1553 as historical provenance]
  preserve_history --> bind_td[Bind WI #2144 TD as primary verification linkage]
  bind_td --> verify_contract[Run TD and existing capability gates]
  verify_contract --> linked{Primary TD linkage discovered?}
  linked -->|yes| complete([Capability goal advances to runtime verification])
  linked -->|no| revise[Revise only TD metadata or producer-owned provenance]
  revise --> verify_contract
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Make WI #2144 the active bounded verification-link root for stateful-service-workload-projection while retaining closed WI #1553 as historical projection provenance; do not change the capability promise or runtime evidence. generator gap: missing-generator:capability-provenance-link (#2144)."
  - path: apps/lumen/tests/capability_stateful_workload_linkage.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Add a deterministic structural regression test that requires the TD primary capability reference, active #2144 linkage, retained #1553 provenance, and the existing stateful capability gate. generator gap: missing-generator:test:capability-td-linkage (#2144)."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-stateful-service-workload-primary-verification
requirements:
  active_and_historical_wi_provenance:
    id: R2
    text: "The capability contract identifies #2144 as the active verification-link work while retaining closed #1553 as historical projection provenance."
    kind: regression
    risk: medium
    verify: capability_stateful_workload_linkage::active_and_historical_provenance_are_distinct
  capability_goal_advances:
    id: R4
    text: "The Lumen capability goal advances past link_claim_verification once the TD lifecycle is complete."
    kind: functional
    risk: high
    verify: aw goal capability --project lumen --non-interactive
  existing_stateful_gate_remains_authoritative:
    id: R3
    text: "The existing Lumen stateful capability gate remains present and no new runtime behavior is claimed."
    kind: regression
    risk: medium
    verify: aw capability check --project lumen --skip-issue-inventory
  primary_td_linkage:
    id: R1
    text: "The TD declares stateful-service-workload-projection as a primary capability verification linkage."
    kind: functional
    risk: high
    verify: capability_stateful_workload_linkage::primary_td_linkage_is_bound
---
flowchart TD
    r1[R1 primary td linkage] --> capability_stateful_workload_linkage_primary_td_linkage_is_bound[capability_stateful_workload_linkage::primary_td_linkage_is_bound]
    r2[R2 active and historical wi provenance] --> capability_stateful_workload_linkage_active_and_historical_provenance_are_distinct[capability_stateful_workload_linkage::active_and_historical_provenance_are_distinct]
    r3[R3 existing stateful gate remains authoritative] --> aw_capability_check_project_lumen_skip_issue_inventory[aw capability check --project lumen --skip-issue-inventory]
    r4[R4 capability goal advances] --> aw_goal_capability_project_lumen_non_interactive[aw goal capability --project lumen --non-interactive]
```
