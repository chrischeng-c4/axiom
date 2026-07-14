---
id: '1639'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-detached-reset
entry: ready
nodes:
  ready: { kind: start, label: "Backend ReadyForQuery(Idle) validated and forwarded" }
  detach: { kind: process, label: "Remove lease permit into reset reservation and spawn reset task" }
  frontend: { kind: process, label: "Frontend task resumes next client activity" }
  reset: { kind: process, label: "Backend task sends DISCARD ALL and waits for its ReadyForQuery" }
  idle: { kind: process, label: "Park reset-clean stream and permit in idle set then notify" }
  dispose: { kind: process, label: "Drop failed/reset-cancelled stream and permit then notify" }
edges:
  - { from: ready, to: detach }
  - { from: detach, to: frontend }
  - { from: detach, to: reset }
  - { from: reset, to: idle, label: "reset ReadyForQuery" }
  - { from: reset, to: dispose, label: "error EOF timeout cancellation" }
---
flowchart LR
  ready([client sees ReadyForQuery Idle]) --> detach[move permit into reset reservation\nspawn backend-only reset]
  detach --> frontend[resume frontend task]
  detach --> reset[DISCARD ALL]
  reset -->|clean Ready| idle[park idle + notify]
  reset -->|fail/cancel| dispose[drop stream + permit + notify]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: modify
    section: pgpool-detached-reset
    impl_mode: hand-written
  - path: apps/pgpool/src/pool/transaction.rs
    action: modify
    section: pgpool-detached-reset
    impl_mode: hand-written
  - path: apps/pgpool/tests/pool.rs
    action: modify
    section: pgpool-detached-reset
    impl_mode: hand-written
  - path: apps/pgpool/tests/pool_modes.rs
    action: modify
    section: pgpool-detached-reset
    impl_mode: hand-written
```
