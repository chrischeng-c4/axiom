---
id: '2149'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: beam-single-node-durable-state
entry: start
nodes:
  start: { kind: start, label: "Process startup / write request" }
  load_check: { kind: decision, label: "Does valid snapshot exist?" }
  init_registry: { kind: process, label: "Initialize serve state in registry" }
  reject_corrupt: { kind: terminal, label: "Abort startup and reject corrupt snapshot" }
  write_request: { kind: process, label: "Write payload/mutation to memory" }
  save_state: { kind: process, label: "Call libs/storage-durable atomic replace" }
  fsync_data: { kind: process, label: "Perform fsync on file and parent directory" }
  done: { kind: terminal, label: "Acknowledge write success" }
edges:
  - { from: start, to: load_check }
  - { from: load_check, to: init_registry, label: "Yes" }
  - { from: load_check, to: reject_corrupt, label: "No (Corrupt)" }
  - { from: init_registry, to: write_request }
  - { from: write_request, to: save_state }
  - { from: save_state, to: fsync_data }
  - { from: fsync_data, to: done }
---
flowchart TD
    start([Start]) --> load_check{Valid snapshot?}
    load_check -->|Yes| init_registry[Initialize registry]
    load_check -->|No| reject_corrupt([Abort startup])
    init_registry --> write_request[Process mutation request]
    write_request --> save_state[Atomic temp-write & replace]
    save_state --> fsync_data[Fsync file & parent dir]
    fsync_data --> done([Ack success])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/beam/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
  - path: apps/beam/src/main.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: "struct ServeArgs"
  - path: apps/beam/src/persist.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: "pub fn save_framed"
  - path: apps/beam/src/service.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: "struct AppState"
  - path: apps/beam/tests/persistence.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: "fn flat_round_trip_identity_cpu_and_gpu"
  - path: apps/beam/tests/restart_recovery.rs
    action: create
    section: unit-test
    impl_mode: hand-written
```
