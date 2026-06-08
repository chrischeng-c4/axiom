---
id: unified-inner-core
fill_sections: [logic, interaction, dependency, test-plan, changes]
---

# Unified Inner Core

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: unified-inner-core-step-execution
entry: start
nodes:
  start:
    kind: start
    label: "Step::run(ctx, input) invoked"
  emit_started:
    kind: process
    label: "emit StepEvent::Started{step_id, input_hash}"
  check_cancellation:
    kind: decision
    label: "ctx.cancellation_token.is_cancelled() ?"
  cancelled_exit:
    kind: terminal
    label: "emit StepEvent::Cancelled; return Err(Cancelled)"
  load_checkpoint:
    kind: decision
    label: "ctx.checkpoint.has(step_id, input) ?"
  resume_from_checkpoint:
    kind: process
    label: "deserialize saved state; jump to saved yield point"
  execute_body:
    kind: process
    label: "run user-supplied body; may yield partial output via ctx.stream(chunk)"
  check_stream:
    kind: decision
    label: "body emitted a chunk ?"
  emit_partial:
    kind: process
    label: "emit StepEvent::PartialOutput{chunk}; checkpoint state; loop back to check_cancellation"
  check_result:
    kind: decision
    label: "body returned Ok(O) ?"
  emit_completed:
    kind: process
    label: "emit StepEvent::Completed{step_id, output_hash}; checkpoint final state"
  ok_exit:
    kind: terminal
    label: "return Ok(O)"
  emit_failed:
    kind: process
    label: "emit StepEvent::Failed{step_id, error}; checkpoint error state"
  err_exit:
    kind: terminal
    label: "return Err(E)"
edges:
  - {from: start, to: emit_started}
  - {from: emit_started, to: check_cancellation}
  - {from: check_cancellation, to: cancelled_exit, label: "yes"}
  - {from: check_cancellation, to: load_checkpoint, label: "no"}
  - {from: load_checkpoint, to: resume_from_checkpoint, label: "yes"}
  - {from: load_checkpoint, to: execute_body, label: "no"}
  - {from: resume_from_checkpoint, to: execute_body}
  - {from: execute_body, to: check_stream}
  - {from: check_stream, to: emit_partial, label: "yes"}
  - {from: emit_partial, to: check_cancellation}
  - {from: check_stream, to: check_result, label: "no"}
  - {from: check_result, to: emit_completed, label: "yes"}
  - {from: emit_completed, to: ok_exit}
  - {from: check_result, to: emit_failed, label: "no"}
  - {from: emit_failed, to: err_exit}
---
flowchart TD
    s[Step::run invoked] --> es[emit Started]
    es --> cc{cancelled?}
    cc -- yes --> ce([Cancelled])
    cc -- no --> lc{checkpoint?}
    lc -- yes --> rc[resume]
    lc -- no --> eb[execute body]
    rc --> eb
    eb --> cs{chunk?}
    cs -- yes --> ep[emit PartialOutput + checkpoint]
    ep --> cc
    cs -- no --> cr{ok?}
    cr -- yes --> ec[emit Completed]
    ec --> ok([Ok])
    cr -- no --> ef[emit Failed]
    ef --> err([Err])
```

## Interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: unified-inner-core-three-frameworks-decompose
actors:
  - {id: User, kind: actor}
  - {id: PyAIAgent, kind: participant}
  - {id: LGGraph, kind: participant}
  - {id: LCChain, kind: participant}
  - {id: Step, kind: system}
  - {id: Runtime, kind: system}
messages:
  - {from: User, to: PyAIAgent, name: "Agent::run(prompt)"}
  - {from: PyAIAgent, to: Step, name: "Step<Prompt, Output>::run(ctx, prompt)"}
  - {from: Step, to: Runtime, name: "execute LLM+tools loop"}
  - {from: Runtime, to: Step, name: "Output", returns: "Ok(O)"}
  - {from: Step, to: PyAIAgent, name: "Ok(O)", returns: "Ok(O)"}
  - {from: User, to: LGGraph, name: "Graph::invoke(state)"}
  - {from: LGGraph, to: Step, name: "traverse: Step_node1::run -> Step_node2::run -> ..."}
  - {from: Step, to: Runtime, name: "execute each node body"}
  - {from: Runtime, to: Step, name: "node Output", returns: "Ok(S')"}
  - {from: Step, to: LGGraph, name: "final state", returns: "Ok(S_final)"}
  - {from: User, to: LCChain, name: "chain.invoke(input)"}
  - {from: LCChain, to: Step, name: "compose: Step_a.then(Step_b).then(Step_c)::run"}
  - {from: Step, to: Runtime, name: "sequential execution"}
  - {from: Runtime, to: Step, name: "final Output", returns: "Ok(O)"}
  - {from: Step, to: LCChain, name: "Ok(O)", returns: "Ok(O)"}
---
sequenceDiagram
    actor User
    participant PyAIAgent
    participant LGGraph
    participant LCChain
    participant Step
    participant Runtime

    User->>PyAIAgent: Agent::run(prompt)
    PyAIAgent->>Step: Step<Prompt, Output>::run(ctx, prompt)
    Step->>Runtime: execute LLM+tools loop
    Runtime-->>Step: Ok(O)
    Step-->>PyAIAgent: Ok(O)

    User->>LGGraph: Graph::invoke(state)
    LGGraph->>Step: traverse nodes
    Step->>Runtime: execute each node
    Runtime-->>Step: Ok(S')
    Step-->>LGGraph: Ok(S_final)

    User->>LCChain: chain.invoke(input)
    LCChain->>Step: compose Step_a.then(Step_b)
    Step->>Runtime: sequential execution
    Runtime-->>Step: Ok(O)
    Step-->>LCChain: Ok(O)
```

## Dependency
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: unified-inner-core-type-graph
types:
  Step:            { kind: trait }
  RunContext:      { kind: struct }
  StepEvent:       { kind: enum }
  Checkpoint:      { kind: trait }
  CancellationToken: { kind: struct }
  EventSink:       { kind: trait }
  Sequential:      { kind: struct, implements: [Step] }
  Conditional:     { kind: struct, implements: [Step] }
  Parallel:        { kind: struct, implements: [Step] }
  Cycle:           { kind: struct, implements: [Step] }
edges:
  - { from: Sequential, to: Step, kind: implements }
  - { from: Conditional, to: Step, kind: implements }
  - { from: Parallel, to: Step, kind: implements }
  - { from: Cycle, to: Step, kind: implements }
  - { from: Step, to: RunContext, kind: references, label: "uses" }
  - { from: RunContext, to: Checkpoint, kind: owns, label: "checkpoint" }
  - { from: RunContext, to: CancellationToken, kind: owns, label: "token" }
  - { from: RunContext, to: EventSink, kind: owns, label: "sink" }
  - { from: Step, to: StepEvent, kind: references, label: "emits" }
---
classDiagram
    class Step~I,O,D~ {
      <<trait>>
      +run(ctx: RunContext~D~, input: I) Result~O~
      +id() StepId
    }
    class RunContext~D~ {
      +deps: D
      +cancellation_token: CancellationToken
      +checkpoint: Arc~dyn Checkpoint~
      +event_sink: Arc~dyn EventSink~
      +stream(chunk: Bytes)
    }
    class StepEvent {
      <<enum>>
      Started
      PartialOutput
      Completed
      Failed
      Cancelled
    }
    class Checkpoint~S~ {
      <<trait>>
      +save(id: StepId, state: S) Result
      +load(id: StepId) Result~Option~S~~
      +list() Result~Vec~StepId~~
      +delete(id: StepId) Result
    }
    class CancellationToken {
      +is_cancelled() bool
      +cancel()
    }
    class EventSink {
      <<trait>>
      +emit(event: StepEvent)
    }
    class Sequential~A,B,C~ {
      +first: Step~A,B~
      +second: Step~B,C~
    }
    class Conditional~A,B,C,D~ {
      +body: Step~A,B~
      +branch: fn(B) Choice~C,D~
    }
    class Parallel~A,B,C~ {
      +left: Step~A,B~
      +right: Step~A,C~
    }
    class Cycle~A~ {
      +body: Step~A,A~
      +stop: fn(A) bool
      +max_iter: usize
    }
    Sequential ..|> Step : implements
    Conditional ..|> Step : implements
    Parallel ..|> Step : implements
    Cycle ..|> Step : implements
    Step --> RunContext : uses
    RunContext --> Checkpoint : holds
    RunContext --> CancellationToken : holds
    RunContext --> EventSink : holds
    Step ..> StepEvent : emits
```

## Test Plan
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: unified-inner-core-test-plan
---
requirementDiagram

requirement T1 {
  id: "T1"
  text: "PydanticAI Agent.run() port: tool-calling agent runs end-to-end as a single Step."
  risk: medium
  verifymethod: test
}

requirement T2 {
  id: "T2"
  text: "LangGraph StateGraph port: three-node graph with conditional edge executes correctly as Sequential + Conditional composition."
  risk: medium
  verifymethod: test
}

requirement T3 {
  id: "T3"
  text: "LangChain chain port: linear three-Step composition produces same output as hand-wired sequence."
  risk: low
  verifymethod: test
}

requirement T4 {
  id: "T4"
  text: "Compile-time type mismatch: composing Step<A,B> with Step<C,D> where B != C fails to compile (trybuild)."
  risk: low
  verifymethod: test
}

requirement T5 {
  id: "T5"
  text: "Cancellation: a Step running a 10-second sleep returns Err(Cancelled) within 100ms of token.cancel()."
  risk: medium
  verifymethod: test
}

requirement T6 {
  id: "T6"
  text: "Checkpoint resume: a Step interrupted at a yield point resumes from the same point with identical state."
  risk: high
  verifymethod: test
}

requirement T7 {
  id: "T7"
  text: "Streaming: a Step that yields N chunks emits exactly N StepEvent::PartialOutput events in order."
  risk: medium
  verifymethod: test
}

requirement T8 {
  id: "T8"
  text: "Parallel composition: two Steps running on the same input complete concurrently (wall-time less than sum of individual times)."
  risk: medium
  verifymethod: test
}

requirement T9 {
  id: "T9"
  text: "Cycle bounded iteration: a Step<A,A> with stop_predicate returning false always terminates at max_iter."
  risk: low
  verifymethod: test
}

requirement T10 {
  id: "T10"
  text: "Overhead benchmark: Step::run vs hand-rolled equivalent latency delta is under 5 percent on tool-calling benchmark."
  risk: high
  verifymethod: test
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentkit/core/src/step/mod.rs
    action: create
    section: dependency
    note: "Step<I, O, D> trait + StepId + impl helpers. Foundation for everything in Epic 2/3/4."

  - path: projects/agentkit/core/src/step/context.rs
    action: create
    section: dependency
    note: "RunContext<D> + CancellationToken + EventSink + Checkpoint hookup."

  - path: projects/agentkit/core/src/step/event.rs
    action: create
    section: dependency
    note: "StepEvent enum (Started, PartialOutput, Completed, Failed, Cancelled)."

  - path: projects/agentkit/core/src/step/checkpoint.rs
    action: create
    section: dependency
    note: "Checkpoint<S> trait surface. Backends ship as Epic 3 issues, not here."

  - path: projects/agentkit/core/src/step/combinator/sequential.rs
    action: create
    section: dependency
    note: "Sequential<A, B, C> = Step<A, B>.then(Step<B, C>) -> Step<A, C>."

  - path: projects/agentkit/core/src/step/combinator/conditional.rs
    action: create
    section: dependency
    note: "Conditional<A, B, C, D> = Step<A, B> + predicate -> Step<A, Either<C, D>>."

  - path: projects/agentkit/core/src/step/combinator/parallel.rs
    action: create
    section: dependency
    note: "Parallel<A, B, C> = (Step<A, B>, Step<A, C>) -> Step<A, (B, C)>."

  - path: projects/agentkit/core/src/step/combinator/cycle.rs
    action: create
    section: dependency
    note: "Cycle<A> = Step<A, A> + stop_predicate + max_iter -> Step<A, A>."

  - path: projects/agentkit/core/src/lib.rs
    action: update
    section: dependency
    note: "Re-export Step + combinators + RunContext + Checkpoint at crate root."

  - path: .aw/tech-design/projects/agentkit/logic/architecture.md
    action: update
    section: changes
    note: "Point the high-level architecture at unified-inner-core.md as the authoritative inner-core spec."

  - path: .aw/tech-design/projects/agentkit/README.md
    action: update
    section: changes
    note: "Add unified-inner-core to the spec index."
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Execution flowchart covers the four critical paths (cancelled, resumed-from-checkpoint, streaming chunk, completed/failed). Decision: approved — though a follow-up TD will need to specify the async yield-point semantics inside `execute_body` more precisely once the trait surface is being implemented (currently the node label is intentionally opaque).
- [interaction] Three-framework decomposition diagram correctly shows that `Agent.run`, `Graph.invoke`, and `chain.invoke` all reduce to traversals of `Step`. This is the load-bearing artifact of the spec — its presence validates the unification thesis.
- [dependency] classDiagram captures the core trait surface (`Step`, `RunContext`, `Checkpoint`, `EventSink`) plus the four combinator structs and their `implements: [Step]` relationship. Adequate for downstream issues to start from.
- [test-plan] T1–T3 cover the three reduction proofs (PydanticAI / LangGraph / LangChain ports). T4 is the compile-time wiring contract. T5–T9 cover cancellation, checkpoint resume, streaming, parallel concurrency, and bounded cycle. T10 is the overhead benchmark. Coverage matrix is complete versus R1–R16.
- [changes] The 11-file change list creates the `step/` module + four combinator files + updates the architecture spec index. Each path is concrete; no `(fill)` placeholders.

### Review 2
**Verdict:** approved

- [logic] Re-verified: state machine semantics are consistent with the trait surface in `dependency`. No new findings.
- [interaction] Re-verified: framework-reduction sequenceDiagram remains the load-bearing artifact. Approved.
- [dependency] Re-verified: trait surface stable. Approved.
- [test-plan] Re-verified: T1–T10 coverage matrix remains complete versus R1–R16. Approved.
- [changes] Re-verified: 11-file change list is concrete and bounded to the inner core. Approved.
