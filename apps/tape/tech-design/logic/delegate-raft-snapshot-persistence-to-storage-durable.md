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

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-storage-durable-adoption-verification
requirements:
  failover_keeps_committed_events:
    id: R2
    text: "Shared marker and snapshot writes preserve committed Tape events through leader loss and restart."
    kind: regression
    risk: high
    verify: apps/tape/tests/raft_failover.rs::kill_9_leader_survivors_reelect_with_no_committed_event_loss
  snapshot_recovery_remains_durable:
    id: R1
    text: "Shared atomic persistence preserves Tape Raft snapshot install and recovery semantics."
    kind: regression
    risk: high
    verify: apps/tape/tests/raft_cluster.rs::fresh_node_catches_up_via_install_snapshot
---
flowchart TD
    r1[R1 snapshot recovery remains durable] --> apps_tape_tests_raft_cluster_rs_fresh_node_catches_up_via_install_snapshot[apps/tape/tests/raft_cluster.rs::fresh_node_catches_up_via_install_snapshot]
    r2[R2 failover keeps committed events] --> apps_tape_tests_raft_failover_rs_kill_9_leader_survivors_reelect_with_no_committed_event_loss[apps/tape/tests/raft_failover.rs::kill_9_leader_survivors_reelect_with_no_committed_event_loss]
```
