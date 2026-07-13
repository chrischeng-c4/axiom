---
id: '1622'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-worker-count-tuning
entry: pgpool
nodes:
  pgpool: { kind: start, label: "pgpool multi-thread CLI runtime" }
  workers: { kind: process, label: "Bounded worker count for 64 client and 16 backend relay topology" }
  relay: { kind: process, label: "Existing concurrent wire relay and pool state machine" }
  benchmark: { kind: terminal, label: "Repeated complete no-error release comparison" }
edges:
  - { from: pgpool, to: workers }
  - { from: workers, to: relay }
  - { from: relay, to: benchmark }
---
flowchart LR
  pgpool([pgpool CLI]) --> workers[bounded Tokio worker count]
  workers --> relay[existing relay and reset semantics]
  relay --> benchmark([repeat complete benchmark])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/bin/pgpool.rs
    action: modify
    section: pgpool-worker-count-tuning
    impl_mode: hand-written
```
