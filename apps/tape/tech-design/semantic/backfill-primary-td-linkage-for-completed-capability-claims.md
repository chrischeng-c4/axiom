---
id: '2157'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-completed-claim-primary-linkage
entry: inventory_claims
nodes:
  inventory_claims:
    kind: start
    label: "Load the exact 19 completed Tape capability and claim pairs"
  preserve_history:
    kind: process
    label: "Keep closed implementation work items as historical provenance"
  bind_refs:
    kind: process
    label: "Bind every pair to this TD with primary and full coverage"
  structural_test:
    kind: process
    label: "Check the exact ids, role, coverage, and reference count"
  run_existing_gates:
    kind: process
    label: "Run existing Tape claim oracles and configured runtime gates"
  complete:
    kind: decision
    label: "Does the capability goal advance through runtime verification?"
  revise_metadata:
    kind: process
    label: "Revise only linkage metadata or structural coverage"
  done:
    kind: terminal
    label: "Completed Tape claims have primary verification linkage"
edges:
  - { from: inventory_claims, to: preserve_history }
  - { from: preserve_history, to: bind_refs }
  - { from: bind_refs, to: structural_test }
  - { from: structural_test, to: run_existing_gates }
  - { from: run_existing_gates, to: complete }
  - { from: complete, to: done, label: "yes" }
  - { from: complete, to: revise_metadata, label: "no" }
  - { from: revise_metadata, to: structural_test }
---
flowchart TD
  inventory_claims([Load exact 19 completed claim pairs]) --> preserve_history[Preserve closed WI provenance]
  preserve_history --> bind_refs[Bind primary and full TD refs]
  bind_refs --> structural_test[Verify exact linkage inventory]
  structural_test --> run_existing_gates[Run existing Tape claim oracles and runtime gates]
  run_existing_gates --> complete{Capability goal reaches runtime verification?}
  complete -->|yes| done([Primary verification linkage complete])
  complete -->|no| revise_metadata[Revise linkage metadata only]
  revise_metadata --> structural_test
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/tests/capability_primary_linkage.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Add a deterministic structural regression test for the exact 19 capability refs, including primary role and full coverage. generator gap: missing-generator:test:capability-td-linkage (#2157)."
```
