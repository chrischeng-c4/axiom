---
id: lumen-orphaned-wal-relay-semantic-mirrors
summary: >
  Remove semantic/source mirror snapshots for deleted relay WAL files so Lumen's
  semantic inventory no longer preserves stale source text or references to the
  retired HA.md relay-WAL path. The cleanup is documentation/ownership hygiene
  only; runtime WAL behavior remains raft-host based.
capability_refs:
  - id: "long-running-stability"
    role: primary
    gap: "log-fan-out-rebuild-from-log"
    claim: "log-fan-out-rebuild-from-log"
    coverage: partial
    rationale: >
      The orphaned mirrors describe the retired relay-backed WAL path under the
      long-running rebuild-from-log claim; removing them keeps the semantic
      inventory aligned with the current raft-host source tree.
fill_sections: [logic, unit-test, changes]
---

# TD: Orphaned semantic mirror snapshots for deleted wal_relay files

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-orphaned-semantic-mirror-contract
entry: sweep
nodes:
  sweep: { kind: start, label: "run semantic/source missing-target sweep" }
  result: { kind: process, label: "orphan set = wal_relay source mirror + wal_relay test mirror" }
  remove: { kind: process, label: "delete only those orphaned mirror TD files" }
  refresh: { kind: process, label: "run aw td lock --project lumen" }
  verify: { kind: terminal, label: "sweep is empty; cleanup TD check passes" }
edges:
  - { from: sweep, to: result }
  - { from: result, to: remove }
  - { from: remove, to: refresh }
  - { from: refresh, to: verify }
---
flowchart TD
    sweep([missing-target sweep]) --> result[orphan set: source/test wal_relay mirrors]
    result --> remove[delete only orphaned mirror TDs]
    remove --> refresh[refresh td.lock]
    refresh --> verify([sweep empty and TD check clean])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-orphaned-semantic-mirror-contract-tests
requirements:
  sweep_empty:
    id: R1
    text: "The semantic/source sweep prints no missing target entries after cleanup."
    kind: regression
    risk: medium
    verify: command
  deleted_files_absent:
    id: R2
    text: "`projects-lumen-src-wal_relay-rs.md` and `projects-lumen-tests-wal_relay-rs.md` are absent."
    kind: functional
    risk: medium
    verify: command
  no_ha_reference:
    id: R3
    text: "The deleted semantic mirror no longer carries the stale `see HA.md` reference."
    kind: documentation
    risk: low
    verify: command
  td_check:
    id: R4
    text: "The cleanup TD validates with `aw td check`."
    kind: governance
    risk: low
    verify: command
---
flowchart TD
    r1[R1 sweep empty] --> shell[semantic/source sweep]
    r2[R2 deleted files absent] --> absent[test ! -e orphan files]
    r3[R3 no stale HA mirror] --> absent
    r4[R4 TD check] --> check[aw td check cleanup TD]
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md
    action: delete
    section: logic
    impl_mode: hand-written
    description: "Delete semantic source snapshot for removed projects/lumen/src/wal_relay.rs."
  - path: projects/lumen/tech-design/semantic/source/projects-lumen-tests-wal_relay-rs.md
    action: delete
    section: logic
    impl_mode: hand-written
    description: "Delete semantic test snapshot for removed projects/lumen/tests/wal_relay.rs."
  - path: projects/lumen/tech-design/td.lock
    action: modify
    section: changes
    impl_mode: hand-written
    description: "Regenerate Lumen TD lock after deleting orphaned semantic TDs."
```
