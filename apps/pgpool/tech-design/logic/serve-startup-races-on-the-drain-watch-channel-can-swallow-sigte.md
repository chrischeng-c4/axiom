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

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: serve-startup-drain-publication-verification
requirements:
  admin_and_cli_regressions:
    id: R3
    text: "Existing admin-plane and CLI surface behavior remains unchanged by serve-startup ordering."
    kind: regression
    risk: medium
    verify: cargo test -p pgpool --test admin_plane && cargo test -p pgpool --test cli_contract
  both_planes_observe_prestart_drain:
    id: R2
    text: "TCP and admin shutdown signals created before serving both complete when drain flips before either plane polls its shutdown future."
    kind: functional
    risk: high
    verify: cargo test -p server-lifecycle --test drain_prestart both_plane_signals_observe_prestart_drain
  receiverless_publish_persists:
    id: R1
    text: "Draining published before any receiver exists persists in DrainController and is observed by subscriptions created later."
    kind: regression
    risk: high
    verify: cargo test -p server-lifecycle --test drain_prestart receiverless_drain_persists_for_late_subscriber
---
flowchart TD
    r1[R1 receiverless publish persists] --> cargo_test_p_server_lifecycle_test_drain_prestart_receiverless_drain_persists_for_late_subscriber[cargo test -p server-lifecycle --test drain_prestart receiverless_drain_persists_for_late_subscriber]
    r2[R2 both planes observe prestart drain] --> cargo_test_p_server_lifecycle_test_drain_prestart_both_plane_signals_observe_prestart_drain[cargo test -p server-lifecycle --test drain_prestart both_plane_signals_observe_prestart_drain]
    r3[R3 admin and cli regressions] --> cargo_test_p_pgpool_test_admin_plane_cargo_test_p_pgpool_test_cli_contract[cargo test -p pgpool --test admin_plane && cargo test -p pgpool --test cli_contract]
```
