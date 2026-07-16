---
id: '1812'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-storage-durable-adoption
entry: domain
nodes:
  domain:
    kind: start
    label: "Tape serializes its domain JournalSnapshot and applied marker bytes"
  shared:
    kind: process
    label: "storage-durable atomically writes bytes with FsyncPolicy Always and parent-dir sync"
  recovery:
    kind: process
    label: "Tape restores the unchanged domain snapshot and applied floor on restart"
  invariant:
    kind: terminal
    label: "No Tape-local temp-file fsync rename implementation remains"
edges:
  - { from: domain, to: shared }
  - { from: shared, to: recovery }
  - { from: recovery, to: invariant }
---
flowchart LR
  domain["Tape snapshot bytes"] --> shared["storage-durable atomic_write Always"] --> recovery["Tape recovery"] --> invariant(["No local durability mechanism"])
```
