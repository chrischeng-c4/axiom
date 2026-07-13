---
id: "1585"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-empty-pvc-bootstrap-contract
entry: seed
nodes:
  seed:
    kind: start
    label: "Replica-mode Tape starts with optional TAPE_BOOTSTRAP_SEED_URI"
  guard:
    kind: decision
    label: "Is TAPE_DATA_DIR configured and empty before raft state exists?"
  reject:
    kind: terminal
    label: "Abort before listener and before any local state is replaced"
  fetch:
    kind: process
    label: "Fetch exact file:// or backup-enabled s3:// object through service-backup"
  decode:
    kind: decision
    label: "Does the object decode as Tape's JournalSnapshot?"
  persist:
    kind: process
    label: "Atomically prepare the canonical applied marker and snapshot files for this raft node"
  raft:
    kind: process
    label: "Construct TapeRaft; TapeStateMachine restores the seed, then raft log/snapshot delta catches up"
  done:
    kind: terminal
    label: "Cold recovery seed is complete; normal live replica synchronization remains raft-host owned"
edges:
  - { from: seed, to: guard }
  - { from: guard, to: reject, label: "no" }
  - { from: guard, to: fetch, label: "yes" }
  - { from: fetch, to: decode }
  - { from: decode, to: reject, label: "no" }
  - { from: decode, to: persist, label: "yes" }
  - { from: persist, to: raft }
  - { from: raft, to: done }
---
flowchart TD
    seed[Replica-mode Tape starts with optional TAPE_BOOTSTRAP_SEED_URI] --> guard{Is TAPE_DATA_DIR configured and empty before raft state exists?}
    guard -->|no| reject([Abort before listener and before any local state is replaced])
    guard -->|yes| fetch[Fetch exact file or backup-enabled S3 object through service-backup]
    fetch --> decode{Does the object decode as Tape's JournalSnapshot?}
    decode -->|no| reject
    decode -->|yes| persist[Atomically prepare canonical applied marker and snapshot files for this raft node]
    persist --> raft[Construct TapeRaft; state machine restores seed, then raft delta catches up]
    raft --> done([Cold recovery seed is complete; live replica synchronization remains raft-host owned])
```
