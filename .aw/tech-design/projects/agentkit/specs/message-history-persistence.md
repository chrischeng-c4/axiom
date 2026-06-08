---
id: message-history-persistence
fill_sections: [logic, dependency, test-plan, changes]
---

## Lifecycle
<!-- type: logic lang: mermaid -->

```mermaid
---
id: message_history_persistence_lifecycle
entry: start
nodes:
  start: { kind: start, label: "entry — Message history persistence typed agent-scoped" }
  build: { kind: process, label: "construct typed surface per #2066" }
  decide: { kind: decision, label: "preconditions met?" }
  reject: { kind: terminal, label: "return typed NovaError" }
  execute: { kind: process, label: "execute primary path" }
  done: { kind: terminal, label: "return typed Output" }
edges:
  - { from: start, to: build }
  - { from: build, to: decide }
  - { from: decide, to: reject, label: "no" }
  - { from: decide, to: execute, label: "yes" }
  - { from: execute, to: done }
---
flowchart TD
  start --> build --> decide
  decide -- no --> reject
  decide -- yes --> execute --> done
```

## Surface Map
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: message_history_persistence_surface
nodes:
  Caller: { kind: class, label: "Caller" }
  Surface: { kind: class, label: "Message history persistence typed agent-scoped" }
  CoreTypes: { kind: class, label: "agent::types — Message NovaError NovaResult" }
edges:
  - { from: Caller, to: Surface, label: "invokes" }
  - { from: Surface, to: CoreTypes, label: "depends on" }
---
classDiagram
  Caller --> Surface
  Surface --> CoreTypes
```

## Test Coverage
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: message_history_persistence_tests
nodes:
  T1: { kind: process, label: "T1 [test] happy path — R1 R2" }
  T2: { kind: process, label: "T2 [test] error path returns NovaError — R3" }
  T3: { kind: process, label: "T3 [inspection] trait conformance compile gate — R4" }
  T4: { kind: process, label: "T4 [test] cargo test passes — R5" }
  done: { kind: terminal, label: "all tests pass" }
edges:
  - { from: T1, to: T2 }
  - { from: T2, to: T3 }
  - { from: T3, to: T4 }
  - { from: T4, to: done }
---
flowchart TD
  T1 --> T2 --> T3 --> T4 --> done
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: .aw/tech-design/projects/agentkit/specs/message-history-persistence.md
    action: create
    section: changes
    note: "This TD spec — source of truth for #2066"

  - path: projects/agentkit/core/src/message_history_persistence.rs
    action: create
    section: changes
    note: "Module stub for Message history persistence typed agent-scoped — codegen marker block, implementation lands in this issue's PR"
```
