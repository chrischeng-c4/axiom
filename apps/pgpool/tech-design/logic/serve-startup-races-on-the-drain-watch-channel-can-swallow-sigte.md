---
id: '1884'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: serve-startup-drain-publication
entry: serve_startup
nodes:
  receivers: { kind: process, label: "construct admin and both plane drain receivers" }
  signal_task: { kind: process, label: "spawn SIGTERM/SIGINT watcher after receiver construction" }
  publish: { kind: process, label: "send_replace Draining even with no receiver" }
  wait: { kind: decision, label: "shutdown observes current Draining or waits for transition" }
  stopped: { kind: terminal, label: "both planes stop before serving new work" }
edges:
  - { from: serve_startup, to: receivers }
  - { from: receivers, to: signal_task }
  - { from: signal_task, to: publish, label: signal or drain route }
  - { from: publish, to: wait }
  - { from: wait, to: stopped }
---
flowchart TD
  start["serve startup"] --> receivers["construct admin + TCP + admin receivers"]
  receivers --> signal["spawn signal watcher"]
  signal --> publish["send_replace Draining"]
  publish --> wait{"shutdown sees current drain?"}
  wait -->|yes or changed| stop["both planes stop"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/server-lifecycle/src/drain.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: start_drain
  - path: apps/pgpool/src/bin/pgpool.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: serve
  - path: apps/pgpool/src/admin/state.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: AdminState
  - path: apps/pgpool/src/admin/wiring.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: shutdown_future_resolving_calls_start_drain_on_the_shared_controller
  - path: libs/server-lifecycle/tests/drain_prestart.rs
    action: create
    section: unit-test
    impl_mode: hand-written
```
