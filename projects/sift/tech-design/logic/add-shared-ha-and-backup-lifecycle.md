---
id: "1605"
summary: (fill)
fill_sections: [logic]
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
