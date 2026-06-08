---
id: benchmark-suite
fill_sections: [logic, dependency, test-plan, changes]
---

## Lifecycle
<!-- type: logic lang: mermaid -->

```mermaid
---
id: benchmark_suite_lifecycle
entry: start
nodes:
  start: { kind: start, label: "entry — Benchmark suite agentkit vs PydanticAI vs LangGraph" }
  build: { kind: process, label: "construct typed surface per #2075" }
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
id: benchmark_suite_surface
nodes:
  Caller: { kind: class, label: "Caller" }
  Surface: { kind: class, label: "Benchmark suite agentkit vs PydanticAI vs LangGraph" }
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
id: benchmark_suite_tests
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
  - path: .aw/tech-design/projects/agentkit/specs/benchmark-suite.md
    action: create
    section: changes
    note: "This TD spec — source of truth for #2075"

  - path: projects/agentkit/core/src/benchmark_suite.rs
    action: create
    section: changes
    note: "Module stub for Benchmark suite agentkit vs PydanticAI vs LangGraph — codegen marker block, implementation lands in this issue's PR"
```
