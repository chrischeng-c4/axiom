---
id: '1698'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-reused-acquire-deadline-timer
entry: deadline
nodes:
  deadline: { kind: start, label: "Fix acquire deadline and pin one Sleep" }
  check: { kind: decision, label: "Idle reuse, replay, or physical permit available" }
  acquire: { kind: terminal, label: "Return existing acquisition result" }
  enable: { kind: process, label: "Create and enable a fresh Notify future" }
  wait: { kind: decision, label: "Select shared deadline Sleep against Notify" }
  retry: { kind: process, label: "Wake and recheck existing acquisition loop" }
  timeout: { kind: terminal, label: "Return existing saturated error" }
edges:
  - { from: deadline, to: check }
  - { from: check, to: acquire, label: "available" }
  - { from: check, to: enable, label: "saturated" }
  - { from: enable, to: wait }
  - { from: wait, to: retry, label: "Notify" }
  - { from: retry, to: check }
  - { from: wait, to: timeout, label: "deadline" }
---
flowchart LR
  start([fix deadline + pin one Sleep]) --> check{capacity or replay available?}
  check -->|yes| granted([existing result])
  check -->|no| notify[enable fresh Notify future]
  notify --> race{Notify or same Sleep?}
  race -->|Notify| check
  race -->|deadline| saturated([existing PoolError::Saturated])
```

### Invariants

- Each invocation fixes one deadline and owns exactly one pinned `Sleep`; Notify wakeups never reset or replace it.
- A fresh enabled `Notify` future retains the current no-missed-wake protocol. A wake only re-enters the existing idle/replay/physical-capacity checks; it grants no slot by itself.
- The deadline race replaces only the current `timeout(remaining, notified)` wrapper. All capacity permits, Notify cardinality, timeout error shape, reset, liveness, and client-visible behavior remain unchanged.
- The timer exists only while saturated acquisition waits; it is not added to the backend relay leg.

### Error handling

If the shared deadline wins, return the existing saturated error. If Notify wins, drop only that one-shot notifier and retry with the same pending Sleep. Connect, bootstrap, reset, liveness, cancellation, and dropped-lease errors preserve their existing paths and permit cleanup.
