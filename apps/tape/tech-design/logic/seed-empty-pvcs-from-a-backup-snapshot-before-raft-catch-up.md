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
    label: "Construct TapeRaft; state machine restores seed, then raft log/snapshot delta catches up"
  done:
    kind: terminal
    label: "Cold recovery seed is complete; normal live replica synchronization remains raft-runtime owned"
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
    raft --> done([Cold recovery seed is complete; live replica synchronization remains raft-runtime owned])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/src/raft.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add the canonical cold-seed preparation helper for a JournalSnapshot: decode exact snapshot bytes, require an empty data directory, and atomically write the per-node applied marker plus sibling snapshot file that TapeStateMachine::new already restores. Do not add a live restore RPC. generator gap: missing-generator:raft-bootstrap-seed (#1585)."
  - path: apps/tape/src/bin/tape.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add --bootstrap-seed-uri / TAPE_BOOTSTRAP_SEED_URI to tape serve. In replica mode it fetches through service_backup::fetch_backup_object and prepares durable seed state before TapeRaft::from_topology; reject seed configuration outside a fresh durable replica. generator gap: missing-generator:service-bootstrap-cli (#1585)."
  - path: apps/tape/src/operator/crd.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Expose optional bootstrapSeedUri in TapeSpec so declarative empty-PVC recovery can use the same CLI contract without a second snapshot format. generator gap: missing-generator:operator-bootstrap-field (#1585)."
  - path: apps/tape/src/operator/render.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Render TAPE_BOOTSTRAP_SEED_URI only when TapeSpec supplies bootstrapSeedUri; default instances retain no seed and normal PVC restart behavior. generator gap: missing-generator:operator-bootstrap-env (#1585)."
  - path: apps/tape/tests/bootstrap.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Exercise exact file backup seeding, canonical marker/snapshot recovery, empty-directory enforcement, and malformed-seed rejection without a live restore endpoint. generator gap: missing-generator:bootstrap-integration-test (#1585)."
  - path: apps/tape/tests/operator.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Assert an opt-in CR bootstrapSeedUri renders exactly one matching environment variable and an omitted field renders none. generator gap: missing-generator:operator-bootstrap-render-test (#1585)."
  - path: apps/tape/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add the backup-restore and replica-sync-bootstrap capability contracts with an honest cold-seed boundary: external backups seed fresh PVCs before raft catch-up and never replace live replica synchronization. generator gap: missing-generator:capability-backup-restore (#1585)."
```
