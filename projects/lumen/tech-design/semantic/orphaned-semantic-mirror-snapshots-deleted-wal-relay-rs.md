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
id: lumen-orphaned-semantic-mirror-cleanup
entry: start
nodes:
  start: { kind: start, label: "sweep projects/lumen/tech-design/semantic/source/*.md" }
  extract: { kind: process, label: "extract '# Standardized <path>' target from each semantic source mirror" }
  exists: { kind: decision, label: "target exists on disk?" }
  keep: { kind: terminal, label: "keep mirror" }
  delete: { kind: process, label: "delete orphaned mirror snapshot" }
  lock: { kind: terminal, label: "refresh projects/lumen/tech-design/td.lock" }
edges:
  - { from: start, to: extract }
  - { from: extract, to: exists }
  - { from: exists, to: keep, label: "yes" }
  - { from: exists, to: delete, label: "no" }
  - { from: delete, to: lock }
---
flowchart TD
    start([semantic/source sweep]) --> extract[read Standardized target path]
    extract --> exists{target exists?}
    exists -->|yes| keep([keep mirror])
    exists -->|no| delete[remove orphaned mirror]
    delete --> lock([refresh td.lock])
```
