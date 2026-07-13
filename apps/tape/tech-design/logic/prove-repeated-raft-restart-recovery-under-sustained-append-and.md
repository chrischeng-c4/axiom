---
id: "1589"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-restart-endurance
entry: cycle
nodes:
  cycle: { kind: start, label: "Start real TapeRaft from durable dir" }
  write: { kind: process, label: "Commit bounded appends and checkpoint" }
  restart: { kind: process, label: "Reopen fresh journal from same directory" }
  verify: { kind: decision, label: "Events and checkpoint preserved?" }
  fail: { kind: terminal, label: "Fail on loss, duplicate, or regression" }
  next: { kind: terminal, label: "Repeat bounded recovery cycles" }
edges:
  - { from: cycle, to: write }
  - { from: write, to: restart }
  - { from: restart, to: verify }
  - { from: verify, to: fail, label: "no" }
  - { from: verify, to: next, label: "yes" }
---
flowchart TD
 cycle[Start TapeRaft] --> write[Commit appends and checkpoint] --> restart[Restart]
 restart --> verify{State preserved?}
 verify -->|no| fail([Fail])
 verify -->|yes| next([Repeat cycles])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/tests/long_running_stability.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Run several real TapeRaft restart cycles against one durable directory, advancing append batches and checkpoints, then assert full replay and durable checkpoint state after every reopen. generator gap: missing-generator:raft-endurance-test (#1589)."
  - path: apps/tape/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Record the bounded repeated-restart conformance gate under Long-Running Stability without claiming retention or unbounded soak. generator gap: missing-generator:stability-capability (#1589)."
```
