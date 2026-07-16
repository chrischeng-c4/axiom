---
id: '1812'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-storage-durable-adoption-contract
entry: snapshot
nodes:
  snapshot:
    kind: start
    label: "Tape owns JournalSnapshot JSON and marker value bytes"
  write:
    kind: process
    label: "storage_durable::atomic_write writes each file with FsyncPolicy Always"
  order:
    kind: process
    label: "Tape preserves snapshot-before-marker bootstrap ordering and best-effort runtime warnings"
  result:
    kind: terminal
    label: "Restart reads unchanged Tape domain state without a local atomic helper"
edges:
  - { from: snapshot, to: write }
  - { from: write, to: order }
  - { from: order, to: result }
---
flowchart TD
  snapshot["Tape snapshot and marker bytes"] --> write["storage-durable atomic write Always"] --> order["Preserve Tape ordering"] --> result(["Unchanged recovery"])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: dependencies
    description: "Declare storage-durable as Tape's direct shared durability dependency."
  - path: apps/tape/src/raft.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: prepare_bootstrap_seed
    description: "Delegate bootstrap, applied-marker, and journal-snapshot atomic writes to storage_durable::atomic_write with FsyncPolicy Always. generator gap: missing-generator:storage-durable-adoption (#1812)."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-storage-durable-adoption-contract
requirements:
  shared_atomic_write_preserves_failover:
    id: R2
    text: "Leader loss and restart must retain all committed events after durable-write delegation."
    kind: regression
    risk: high
    verify: apps/tape/tests/raft_failover.rs::kill_9_leader_survivors_reelect_with_no_committed_event_loss
  shared_atomic_write_preserves_snapshot_recovery:
    id: R1
    text: "Tape must recover the same replicated journal state when every marker and snapshot replace delegates to storage-durable."
    kind: regression
    risk: high
    verify: apps/tape/tests/raft_cluster.rs::fresh_node_catches_up_via_install_snapshot
---
flowchart TD
    r1[R1 shared atomic write preserves snapshot recovery] --> apps_tape_tests_raft_cluster_rs_fresh_node_catches_up_via_install_snapshot[apps/tape/tests/raft_cluster.rs::fresh_node_catches_up_via_install_snapshot]
    r2[R2 shared atomic write preserves failover] --> apps_tape_tests_raft_failover_rs_kill_9_leader_survivors_reelect_with_no_committed_event_loss[apps/tape/tests/raft_failover.rs::kill_9_leader_survivors_reelect_with_no_committed_event_loss]
```
