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

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add the explicit storage-durable dependency required by Tape's domain snapshot adapter."
  - path: apps/tape/src/raft.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: prepare_bootstrap_seed
    description: "Replace Tape-local atomic file persistence with storage_durable::atomic_write while retaining the JournalSnapshot codec and recovery ordering."
```
