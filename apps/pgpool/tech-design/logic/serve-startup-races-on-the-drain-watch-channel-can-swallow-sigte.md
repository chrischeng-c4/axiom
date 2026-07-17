---
id: '1884'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: serve-startup-drain-publication-contract
entry: serve_startup
nodes:
  subscriptions: { kind: process, label: "AdminState, TCP, and admin subscriptions exist before signal task" }
  durable_publish: { kind: process, label: "DrainController send_replace persists Draining without receivers" }
  tolerant_wait: { kind: decision, label: "shutdown checks current state before waiting for changed" }
  shutdown: { kind: terminal, label: "both serving planes terminate their shutdown futures" }
edges:
  - { from: serve_startup, to: subscriptions }
  - { from: subscriptions, to: durable_publish, label: signal or POST drain }
  - { from: durable_publish, to: tolerant_wait }
  - { from: tolerant_wait, to: shutdown }
---
flowchart TD
  startup["serve startup"] --> receivers["construct all drain receivers"]
  receivers --> publish["send_replace Draining"]
  publish --> wait{"currently draining?"}
  wait -->|yes| shutdown["both planes stop"]
  wait -->|no then changed| shutdown
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
id: serve-startup-drain-publication-contract-verification
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
