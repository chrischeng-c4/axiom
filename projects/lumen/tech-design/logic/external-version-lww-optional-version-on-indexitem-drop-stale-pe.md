---
id: lumen-external-version-lww
summary: >
  Add an optional `version: u64` to IndexItem. lumen keeps the highest version
  per (external_id, field) and drops strictly-older writes (external-version
  last-write-wins, like Elasticsearch version_type=external). When `version` is
  absent the write applies in arrival order exactly as today.
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "schema-and-metadata-breadth"
    coverage: partial
    rationale: >
      Lets callers offload out-of-order / stale-write suppression to lumen via a
      monotonic per-cell version, a standard search-index ingest primitive.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: external-version-apply
entry: start
nodes:
  start:  { kind: start,    label: "apply index item (id, field, value, version?)" }
  check:  { kind: decision, label: "version present AND stored >= version?" }
  skip:   { kind: terminal, label: "drop stale write (not applied, not counted)" }
  apply:  { kind: process,  label: "drop_eid + apply_value" }
  record: { kind: process,  label: "if versioned: store max version for (id, field)" }
  done:   { kind: terminal, label: "indexed += 1" }
edges:
  - { from: start,  to: check }
  - { from: check,  to: skip,   label: "yes (stale)" }
  - { from: check,  to: apply,  label: "no" }
  - { from: apply,  to: record }
  - { from: record, to: done }
---
flowchart TD
    start([apply index item]) --> check{version present AND stored >= version?}
    check -->|yes stale| skip([drop stale write])
    check -->|no| apply[drop_eid + apply_value]
    apply --> record[if versioned: store max version]
    record --> done([indexed += 1])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: external-version-lww-verification
requirements:
  stale_dropped:
    id: R1
    text: "a versioned write whose version < the stored version for (external_id, field) is dropped and the value is unchanged"
    kind: functional
    risk: high
    verify: test
  newer_wins:
    id: R2
    text: "a versioned write whose version > the stored version advances the cell value"
    kind: functional
    risk: high
    verify: test
  unversioned_arrival_order:
    id: R3
    text: "a write without a version applies in arrival order, unchanged from today"
    kind: functional
    risk: medium
    verify: test
elements:
  test_stale_version_dropped:
    kind: test
    type: "rs/#[test]"
  test_newer_version_wins:
    kind: test
    type: "rs/#[test]"
  test_unversioned_arrival_order:
    kind: test
    type: "rs/#[test]"
relations:
  - { from: test_stale_version_dropped,      verifies: stale_dropped }
  - { from: test_newer_version_wins,         verifies: newer_wins }
  - { from: test_unversioned_arrival_order,  verifies: unversioned_arrival_order }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "stale versioned write is dropped"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "newer versioned write wins"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "unversioned write keeps arrival order"
      risk: medium
      verifymethod: test
    }
    element test_stale_version_dropped {
      type: "rs/#[test]"
    }
    test_stale_version_dropped - verifies -> R1
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/types.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Expose optional per-item version fields for last-write-wins stale write suppression."
  - path: projects/lumen/src/storage.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Apply external-version checks during index writes and ignore stale per-cell updates."
  - path: projects/lumen/tests/api_e2e.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    reason: "Verify stale versioned writes do not overwrite newer indexed values."
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Correct contract: stale check (version present AND stored >= version) drops the write before drop_eid+apply_value; otherwise apply and, when versioned, store the max version per (id, field). Absent version = arrival order.
- [unit-test] R1–R3 cover stale-drop, newer-wins, and the unversioned arrival-order regression, each bound to a concrete test.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/types.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Expose optional external version fields on index payloads."
  - path: projects/lumen/src/storage.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Apply last-write-wins external version checks during index writes."
  - path: projects/lumen/src/storage.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Cover stale version drops, newer version wins, and unversioned arrival-order behavior."
```
