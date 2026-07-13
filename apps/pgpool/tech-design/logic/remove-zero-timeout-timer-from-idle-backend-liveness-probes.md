---
id: '1626'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-immediate-idle-liveness-probe
entry: acquire_idle
nodes:
  acquire_idle: { kind: start, label: "Acquire pops an idle backend" }
  immediate_probe: { kind: process, label: "Poll TcpStream peek once without a timer" }
  pending: { kind: decision, label: "No readability event yet?" }
  live: { kind: process, label: "Hand out unchanged stream" }
  dead: { kind: process, label: "Drop dead backend and retry acquire" }
  readable: { kind: process, label: "Leave bytes unconsumed for relay" }
edges:
  - { from: acquire_idle, to: immediate_probe }
  - { from: immediate_probe, to: pending }
  - { from: pending, to: live, label: "yes" }
  - { from: pending, to: readable, label: "readable bytes" }
  - { from: readable, to: live }
  - { from: immediate_probe, to: dead, label: "EOF or I/O error" }
---
flowchart LR
  acquire_idle([pop idle backend]) --> immediate_probe[poll peek once: no timer]
  immediate_probe --> pending{pending?}
  pending -->|yes| live[reuse unchanged stream]
  pending -->|readable bytes| readable[leave bytes queued]
  readable --> live
  immediate_probe -->|EOF or error| dead[drop and retry]
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: modify
    section: pgpool-immediate-idle-liveness-probe
    impl_mode: hand-written
  - path: apps/pgpool/tests/pool.rs
    action: modify
    section: pgpool-immediate-idle-liveness-probe
    impl_mode: hand-written
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-immediate-idle-liveness-probe-verification
requirements:
  dead_backend:
    id: R2
    text: "EOF and I/O failure from an idle backend discard it before it is leased, while queued readable bytes are never consumed by the liveness check."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test pool --test pool_modes
  timer_free_pending:
    id: R1
    text: "An idle backend with no readability event remains reusable after one immediate non-consuming poll without constructing a zero-timeout timer."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test pool
---
flowchart TD
    r1[R1 timer free pending] --> cargo_test_p_pgpool_test_pool[cargo test -p pgpool --test pool]
    r2[R2 dead backend] --> cargo_test_p_pgpool_test_pool_test_pool_modes[cargo test -p pgpool --test pool --test pool_modes]
```
