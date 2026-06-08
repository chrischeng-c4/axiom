---
id: generator
title: Generator Functions and yield
crate: mamba
files:
  - crates/mamba/src/runtime/generator.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: 91213c22f
---

# Generator Functions and yield

Mamba generators are stackful coroutines on the same OS thread as the
caller. Each generator owns a private 64 KiB `mmap`'d stack with a
guard page; `next()` / `send()` / `throw()` swap CPU register context
and stack pointer between the caller and the generator body. Yield
overhead is one register save plus a stack-pointer swap (~10 ns) — no
channel hops, no cross-thread sync.

Three load-bearing invariants:

1. **Same-thread coroutines** — generators run on their creator's
   thread. `GENERATORS` is thread-local. Crossing a thread boundary
   with a generator handle would dereference an invalid registry slot.
2. **Coroutine context pointer stability** — `GenEntry.coro_ctx` is
   `Box<CoroContext>` so its address survives `HashMap` resizes;
   `swap_context` is called with that raw pointer. Inlining
   `CoroContext` directly into `GenEntry` would corrupt resumption
   after any registry growth.
3. **Generator IDs disjoint from iter IDs** — `NEXT_GEN_ID` is global
   atomic starting at 1; iterator IDs start at `0x1_0000_0000` (see
   `iter.md`). The disjoint ranges let `mb_iter` and `mb_next_raise`
   tell which registry to look at given just an `i64` handle.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: generator-types
types:
  GenEntry:        { kind: struct }
  GenState:        { kind: enum, label: "Created / Suspended / Completed" }
  CoroContext:     { kind: struct, label: "callee-saved regs + SP + LR/RIP" }
  CoroStack:       { kind: struct, label: "mmap'd stack + guard page" }
  CallerCtxStack:  { kind: struct, label: "fixed 16-slot caller ctx stack (yield from nesting)" }
  Generators:      { kind: struct, label: "thread_local HashMap<u64, GenEntry>" }
  ActiveGenId:     { kind: struct, label: "thread_local Cell<Option<u64>>" }
  ActiveGenCtx:    { kind: struct, label: "thread_local Cell<*mut CoroContext> (hot-path cache)" }
  XferSlots:       { kind: struct, label: "YIELD_XFER / SEND_XFER / THROW_XFER (thread_local Cell<u64>)" }
  IterModule:      { kind: struct, label: "from runtime::iter (mb_next_raise dispatches generators)" }
edges:
  - { from: GenEntry,        to: GenState,        kind: owns }
  - { from: GenEntry,        to: CoroContext,     kind: owns,       label: "Box for stable address" }
  - { from: GenEntry,        to: CoroStack,       kind: owns }
  - { from: Generators,      to: GenEntry,        kind: owns }
  - { from: ActiveGenCtx,    to: CoroContext,     kind: references, label: "raw ptr cached for yield" }
  - { from: CallerCtxStack,  to: CoroContext,     kind: owns,       label: "16 pre-allocated slots" }
  - { from: XferSlots,       to: GenEntry,        kind: references, label: "value transfer between yield/resume" }
  - { from: IterModule,      to: Generators,      kind: references, label: "advance_generator_if_applicable" }
---
classDiagram
    class GenEntry
    class GenState
    class CoroContext
    class CoroStack
    class CallerCtxStack
    class Generators
    class ActiveGenId
    class ActiveGenCtx
    class XferSlots
    class IterModule
    GenEntry --> GenState : state
    GenEntry --> CoroContext : Box
    GenEntry --> CoroStack : mmap
    Generators --> GenEntry : owns
    ActiveGenCtx --> CoroContext : cached raw ptr
    CallerCtxStack --> CoroContext : 16 slots
    XferSlots --> GenEntry : value transfer
    IterModule --> Generators : dispatch
```

## Generator entry shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "generator-types"
$defs:
  GenState:
    type: string
    enum: [Created, Suspended, Completed]
  GenEntry:
    type: object
    x-rust-type: GenEntry
    properties:
      coro_ctx:        { x-rust-type: "Box<CoroContext>" }
      coro_stack:      { x-rust-type: CoroStack, description: "64 KiB usable + 16 KiB guard" }
      state:           { $ref: "#/$defs/GenState" }
      body_fn_addr:    { type: integer, x-rust-type: u64, description: "NaN-boxed FUNC tag bits, 48-bit code address" }
      args:            { type: array, items: { x-rust-type: MbValue } }
      name:            { type: string }
      yielded_value:   { x-rust-type: MbValue, description: "set by yield_value, read by next" }
      sent_value:      { x-rust-type: MbValue, description: "set by send, read after yield" }
      return_value:    { x-rust-type: MbValue, description: "set when body returns; surfaces in StopIteration.value" }
      throw_request:
        oneOf:
          - { type: "null" }
          - type: object
            properties:
              exc_type: { type: string }
              message:  { type: string }
            required: [exc_type, message]
      close_request:   { type: boolean }
    required: [coro_ctx, coro_stack, state, body_fn_addr, args, name, yielded_value, sent_value, return_value, throw_request, close_request]
  CoroContext:
    type: object
    x-rust-type: CoroContext
    description: "Saved CPU regs — 21 u64 on aarch64 (x19-x28, x29, x30, SP, d8-d15), 8 u64 on x86_64 (rbx, rbp, r12-r15, rsp, rip)"
    properties:
      regs: { type: array, items: { type: integer, x-rust-type: u64 } }
    required: [regs]
  CoroStack:
    type: object
    x-rust-type: CoroStack
    properties:
      base:       { type: integer, x-rust-type: "*mut u8", description: "mmap base; guard page at base" }
      total_size: { type: integer, minimum: 1, description: "guard + usable, 16-byte aligned at top" }
    required: [base, total_size]
```

## Generator lifecycle
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: generator-lifecycle
initial: Created
nodes:
  Created:    { kind: initial,  label: "mb_generator_create; body not yet entered" }
  Suspended:  { kind: normal,   label: "yielded; coro_ctx holds saved regs" }
  Resuming:   { kind: transient, label: "swap_context active; body running" }
  Completed:  { kind: terminal, label: "body returned or close run; subsequent next raises StopIteration" }
edges:
  - { from: Created,   to: Resuming,  event: "mb_generator_next (with sent_value=None) | mb_generator_send(None)" }
  - { from: Created,   to: Completed, event: "mb_generator_send(non-None) → TypeError before swap" }
  - { from: Suspended, to: Resuming,  event: "mb_generator_next | mb_generator_send | mb_generator_throw" }
  - { from: Resuming,  to: Suspended, event: "mb_generator_yield_value (yield) — swap back" }
  - { from: Resuming,  to: Completed, event: "body returns | exception bubbles past body | close request honored" }
  - { from: Suspended, to: Completed, event: "mb_generator_close (suspended path)" }
---
stateDiagram-v2
    [*] --> Created
    Created --> Resuming: next [send=None]
    Created --> Completed: send [non-None] TypeError
    Suspended --> Resuming: next / send / throw
    Resuming --> Suspended: yield (swap back)
    Resuming --> Completed: body returns / exc bubbles / close
    Suspended --> Completed: close request
    Completed --> [*]
```

## Resume / yield dispatch
<!-- type: logic lang: mermaid -->

```mermaid
---
id: generator-dispatch
entry: enter
nodes:
  enter:        { kind: start,    label: "next / send / throw / close" }
  classify:     { kind: decision, label: "method?" }
  set_send:     { kind: process,  label: "send: stage SEND_XFER = value" }
  set_throw:    { kind: process,  label: "throw: stage THROW_XFER = ptr (exc_type, msg)" }
  set_close:    { kind: process,  label: "close: stage THROW_XFER = 1 (close marker)" }
  state_check:  { kind: decision, label: "current GenState?" }
  send_to_created: { kind: decision, label: "send with non-None on Created?" }
  type_err:     { kind: terminal, label: "TypeError; mark Completed" }
  push_caller:  { kind: process,  label: "CALLER_CTX_STACK.push() — pre-allocated slot" }
  swap_in:      { kind: process,  label: "swap_context(caller_ctx, gen_ctx); ACTIVE_GEN_ID = id" }
  body_runs:    { kind: process,  label: "body executes until yield / return / exc" }
  read_xfer:    { kind: process,  label: "read YIELD_XFER → MbValue" }
  pop_caller:   { kind: process,  label: "CALLER_CTX_STACK.pop()" }
  done_yield:   { kind: terminal, label: "return yielded value (Suspended)" }
  done_stop:    { kind: terminal, label: "return None; mark Completed; iter::signal_stop_iteration" }
edges:
  - { from: enter,         to: classify }
  - { from: classify,      to: set_send,    label: "send" }
  - { from: classify,      to: set_throw,   label: "throw" }
  - { from: classify,      to: set_close,   label: "close" }
  - { from: classify,      to: state_check, label: "next" }
  - { from: set_send,      to: send_to_created }
  - { from: send_to_created, to: type_err,  label: "yes" }
  - { from: send_to_created, to: state_check, label: "no" }
  - { from: set_throw,     to: state_check }
  - { from: set_close,     to: state_check }
  - { from: state_check,   to: done_stop,   label: "Completed" }
  - { from: state_check,   to: push_caller, label: "Created | Suspended" }
  - { from: push_caller,   to: swap_in }
  - { from: swap_in,       to: body_runs }
  - { from: body_runs,     to: read_xfer,   label: "yield" }
  - { from: body_runs,     to: pop_caller,  label: "return / exc" }
  - { from: read_xfer,     to: pop_caller }
  - { from: pop_caller,    to: done_yield,  label: "yielded" }
  - { from: pop_caller,    to: done_stop,   label: "completed" }
---
flowchart TD
    enter([next / send / throw / close]) --> classify{method}
    classify -->|send| set_send[stage SEND_XFER]
    classify -->|throw| set_throw[stage THROW_XFER ptr]
    classify -->|close| set_close[stage THROW_XFER=1]
    classify -->|next| state_check{state?}
    set_send --> send_to_created{Created and non-None?}
    send_to_created -->|yes| type_err([TypeError])
    send_to_created -->|no| state_check
    set_throw --> state_check
    set_close --> state_check
    state_check -->|Completed| done_stop([StopIteration])
    state_check -->|Created or Suspended| push_caller[push caller ctx]
    push_caller --> swap_in[swap_context]
    swap_in --> body_runs[body runs]
    body_runs -->|yield| read_xfer[read YIELD_XFER]
    body_runs -->|return / exc| pop_caller[pop caller ctx]
    read_xfer --> pop_caller
    pop_caller -->|yielded| done_yield([return value])
    pop_caller -->|completed| done_stop
```

## yield_value / resume interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: generator-yield-resume
actors:
  - { id: Caller,      kind: system, label: "JIT-compiled next() emit" }
  - { id: GenRuntime,  kind: system, label: "generator.rs" }
  - { id: Body,        kind: system, label: "JIT-compiled body fn (running on coro stack)" }
  - { id: Xfer,        kind: system, label: "YIELD_XFER / SEND_XFER cells" }
messages:
  - { from: Caller,     to: GenRuntime, name: mb_generator_next(handle) }
  - { from: GenRuntime, to: Xfer,       name: "SEND_XFER = None.bits()" }
  - { from: GenRuntime, to: GenRuntime, name: "ACTIVE_GEN_CTX = ctx of handle" }
  - { from: GenRuntime, to: Body,       name: "swap_context(caller_ctx, gen_ctx)" }
  - { from: Body,       to: Body,       name: "execute until yield" }
  - { from: Body,       to: Xfer,       name: "YIELD_XFER = value.bits()" }
  - { from: Body,       to: GenRuntime, name: "swap_context back" }
  - { from: GenRuntime, to: Xfer,       name: "read YIELD_XFER" }
  - { from: GenRuntime, to: Caller,     name: yielded_value, returns: MbValue }
  - { from: Caller,     to: GenRuntime, name: "mb_generator_send(handle, v)" }
  - { from: GenRuntime, to: Xfer,       name: "SEND_XFER = v.bits()" }
  - { from: GenRuntime, to: Body,       name: "swap_context back to body" }
  - { from: Body,       to: Xfer,       name: "yield expression reads SEND_XFER" }
  - { from: Body,       to: Body,       name: "continues until next yield / return" }
---
sequenceDiagram
    participant Caller
    participant GenRuntime
    participant Body
    participant Xfer
    Caller->>GenRuntime: mb_generator_next(handle)
    GenRuntime->>Xfer: SEND_XFER = None
    GenRuntime->>GenRuntime: ACTIVE_GEN_CTX = ctx
    GenRuntime->>Body: swap_context (enter body)
    Body->>Body: run until yield
    Body->>Xfer: YIELD_XFER = value
    Body->>GenRuntime: swap_context back
    GenRuntime->>Xfer: read YIELD_XFER
    GenRuntime-->>Caller: yielded_value
    Caller->>GenRuntime: mb_generator_send(handle, v)
    GenRuntime->>Xfer: SEND_XFER = v
    GenRuntime->>Body: swap_context back
    Body->>Xfer: yield reads SEND_XFER
    Body->>Body: continues
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->
```yaml
scenarios:
  - id: basic-yield
    given: generators/expr_deep_broad.py yields two values
    when: for-loop iteration advances the generator
    then: both values are returned before StopIteration
  - id: send-created
    given: a newly created generator has not been started
    when: send receives a non-None value
    then: TypeError is raised before context swap
  - id: yield-from-chain
    given: generators/yield_from_chain.py nests yield from three levels deep
    when: values yield through the chain
    then: CALLER_CTX_STACK preserves resumption and bubbles values to the outer caller
  - id: throw-close
    given: a suspended generator receives throw or close
    when: the runtime stages the request
    then: the request is raised at the suspended yield or closes the generator
```

## Tests
<!-- type: test-plan lang: mermaid -->
```mermaid
---
id: runtime-generator-test-plan
title: Generator Runtime Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"]
    Runner --> Basic["generators/expr_deep_broad.py"]
    Runner --> Send["generators/send_edge_cases.py"]
    Runner --> YieldFrom["generators/yield_from_chain.py"]
    Runner --> ThrowClose["generators/throw_close.py"]
    Runner --> Frame["generators/gi_frame_introspection.py"]
    Runner --> StopValue["generators/stop_iter_value.py"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/generator.rs
    action: modify
    impl_mode: hand-written
    description: "Stackful coroutines: GenEntry + CoroContext + CoroStack (mmap + guard page); thread-local GENERATORS registry, ACTIVE_GEN_ID/CTX, YIELD_XFER/SEND_XFER/THROW_XFER cells, CALLER_CTX_STACK (16 slots). Hand-written; the swap_context contract is target-arch dependent."
```
