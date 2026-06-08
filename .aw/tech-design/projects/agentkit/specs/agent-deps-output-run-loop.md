---
id: agent-deps-output-run-loop
fill_sections: [logic, dependency, test-plan, changes]
---

## Run Loop Lifecycle
<!-- type: logic lang: mermaid -->

```mermaid
---
id: agent-2030-lifecycle
entry: start
nodes:
  start: { kind: start, label: "Agent.run(input)" }
  prompt: { kind: process, label: "build prompt from template + Deps" }
  llm: { kind: process, label: "LLM call (async, cancel-safe)" }
  has_tool: { kind: decision, label: "tool_call in reply?" }
  tool: { kind: process, label: "dispatch tool — Deps injected" }
  validate: { kind: process, label: "parse + validate against Output schema" }
  ok: { kind: decision, label: "validation OK?" }
  err: { kind: terminal, label: "return NovaError" }
  done: { kind: terminal, label: "return Output" }
edges:
  - { from: start, to: prompt }
  - { from: prompt, to: llm }
  - { from: llm, to: has_tool }
  - { from: has_tool, to: tool, label: "yes" }
  - { from: tool, to: llm }
  - { from: has_tool, to: validate, label: "no" }
  - { from: validate, to: ok }
  - { from: ok, to: err, label: "no" }
  - { from: ok, to: done, label: "yes" }
---
flowchart TD
  start --> prompt --> llm --> has_tool
  has_tool -- yes --> tool --> llm
  has_tool -- no --> validate --> ok
  ok -- no --> err
  ok -- yes --> done
```

## Surface Map
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: agent-2030-surface
nodes:
  Caller: { kind: class, label: "Caller" }
  Agent: { kind: class, label: "Agent<Deps, Output>" }
  Deps: { kind: class, label: "Deps (DI bag #2034)" }
  Output: { kind: class, label: "AgentOutput (#2032)" }
  Llm: { kind: class, label: "LlmRpc (#2029)" }
  Tools: { kind: class, label: "ToolsRpc (#2029)" }
  NovaError: { kind: class, label: "agent::error::NovaError" }
edges:
  - { from: Caller, to: Agent, label: "run(input)" }
  - { from: Agent, to: Deps, label: "uses" }
  - { from: Agent, to: Output, label: "produces" }
  - { from: Agent, to: Llm, label: "calls" }
  - { from: Agent, to: Tools, label: "dispatches" }
  - { from: Agent, to: NovaError, label: "returns on error" }
---
classDiagram
  Caller --> Agent
  Agent --> Deps
  Agent --> Output
  Agent --> Llm
  Agent --> Tools
  Agent --> NovaError
```

## Test Coverage
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: agent-2030-tests
nodes:
  T1: { kind: process, label: "T1 [test] happy path — direct reply matches Output schema — R1, R3" }
  T2: { kind: process, label: "T2 [test] tool-call path — Deps reachable inside handler — R2, R3" }
  T3: { kind: process, label: "T3 [test] validation failure returns NovaError::Validation — R4" }
  T4: { kind: process, label: "T4 [test] cancellation token aborts mid-call — R2" }
  T5: { kind: process, label: "T5 [inspection] cargo test -p agent passes — R5" }
  done: { kind: terminal, label: "all tests pass" }
edges:
  - { from: T1, to: T2 }
  - { from: T2, to: T3 }
  - { from: T3, to: T4 }
  - { from: T4, to: T5 }
  - { from: T5, to: done }
---
flowchart TD
  T1 --> T2 --> T3 --> T4 --> T5 --> done
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: .aw/tech-design/projects/agentkit/specs/agent-deps-output-run-loop.md
    action: create
    section: changes
    note: "This TD spec — source of truth for #2030"

  - path: projects/agentkit/core/src/agent.rs
    action: create
    section: changes
    note: "Agent<Deps, Output> struct + builder + run loop — codegen marker block"
```
