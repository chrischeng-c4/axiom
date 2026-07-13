---
id: "1605"
summary: Add shared durable journal, Raft-host replication, snapshots, and backup/restore lifecycle to Sift.
capability_refs:
  - id: replica-sync-and-bootstrap
    role: primary
    gap: sift-raft-host-replication
    claim: sift-raft-host-replication
    coverage: partial
    rationale: Sift must use the shared state-machine host when replica topology is configured.
  - id: backup-and-restore
    role: primary
    gap: sift-shared-backup-restore
    claim: sift-shared-backup-restore
    coverage: partial
    rationale: Sift snapshots must be reusable by the shared backup and bootstrap contracts.
  - id: durability-and-acknowledgment
    role: contributes
    gap: sift-crc-framed-durability
    claim: sift-crc-framed-durability
    coverage: partial
    rationale: The raw journal needs shared CRC-framed append and atomic snapshot mechanics.
fill_sections: [logic, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-ha-backup-flow
entry: ingest
nodes:
  ingest: { kind: start, label: "validated raw event" }
  mode: { kind: decision, label: "raft-host replica mode?" }
  local: { kind: process, label: "append CRC-framed journal with shared fsync policy" }
  propose: { kind: process, label: "propose serialized event to RaftHost" }
  apply: { kind: process, label: "Sift RaftStateMachine applies ordered journal entry" }
  acknowledged: { kind: terminal, label: "acknowledge durable applied cursor" }
  snapshot: { kind: process, label: "serialize journal snapshot and atomically replace local snapshot" }
  backup: { kind: process, label: "service-backup ships snapshot to declared off-node destination and applies retention" }
  restore: { kind: terminal, label: "restore snapshot then replay replicated log" }
edges:
  - { from: ingest, to: mode }
  - { from: mode, to: local, label: "single-node" }
  - { from: mode, to: propose, label: "replica" }
  - { from: propose, to: apply }
  - { from: local, to: acknowledged }
  - { from: apply, to: acknowledged }
  - { from: acknowledged, to: snapshot, label: "backup/checkpoint" }
  - { from: snapshot, to: backup }
  - { from: backup, to: restore, label: "bootstrap or recovery" }
---
flowchart TD
    ingest([validated raw event]) --> mode{replica mode?}
    mode -->|single node| local[shared durable append]
    mode -->|replica| propose[RaftHost propose]
    propose --> apply[ordered state-machine apply]
    local --> acknowledged([acknowledge applied cursor])
    apply --> acknowledged
    acknowledged --> snapshot[atomic journal snapshot]
    snapshot --> backup[shared backup runner]
    backup --> restore([restore then catch up])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/sift/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    gap: sift-ha-shared-dependencies
    tracker: "1605"
    description: Compose raft-core, raft-host, service-durability, and service-backup dependencies for the Sift runtime.
  - path: projects/sift/src/durability.rs
    action: create
    section: logic
    impl_mode: hand-written
    gap: sift-framed-journal-state-machine
    tracker: "1605"
    description: Implement CRC-framed event journal snapshot/restore and the RaftStateMachine adapter.
  - path: projects/sift/src/backup.rs
    action: create
    section: logic
    impl_mode: hand-written
    gap: sift-shared-backup-runner
    tracker: "1605"
    description: Implement Sift snapshot backup/restore composition through service-backup destinations and retention.
  - path: projects/sift/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: sift-ha-runtime-routing
    tracker: "1605"
    description: Route single-node writes through the framed journal and replica writes through RaftHost while exposing snapshot operations.
  - path: projects/sift/src/bin/sift.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: sift-ha-cli-lifecycle
    tracker: "1605"
    description: Add snapshot, restore, and backup commands with shared backup destination and retention arguments.
  - path: projects/sift/tests/ha_backup_e2e.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    gap: sift-ha-backup-contract-tests
    tracker: "1605"
    description: Verify durable recovery, Raft single-node state-machine ordering, snapshot restore, and shared backup output.
```
