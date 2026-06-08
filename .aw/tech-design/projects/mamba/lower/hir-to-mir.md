---
id: hir-to-mir
title: Lower — HIR to MIR
crate: mamba
files:
  - crates/mamba/src/lower/hir_to_mir.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: c2de14e6c
---

# Lower — HIR to MIR

`lower/hir_to_mir.rs` (6298 LOC) takes the HIR module produced by
`ast_to_hir` and emits a CFG-based `MirModule`. Per HIR function it
allocates a `MirBody`, walks statements emitting `MirInst` into the
current block, and opens new blocks at every control-flow boundary
(if / while / for / try / match / break / continue / return).

This is where most JIT-visible decisions are made: which globals are
hot enough to inline-load via `LoadGlobal`, which captures need cells,
how try-blocks fan out to handlers and finally-merge, and how
checked arithmetic ops are emitted vs. plain BinOp.

Three load-bearing invariants:

1. **`Var(sym)` for decorated user funcs emits `LoadGlobal`, not
   `FuncRef`** (commit `7b4b6af4` fix from `closure.md`). Decorators
   replace the bound symbol's value in `GLOBAL_BY_ID` with the wrapper
   instance; `FuncRef` would short-circuit to the raw JIT entry,
   bypassing the wrapper. This affects `@lru_cache`, `@property`,
   `@classmethod`, etc.
2. **Cell allocation lives at function-entry, not at first capture
   site** — every function with captured-and-mutated variables emits
   `MakeCell` instructions in its prologue block, so the cell exists
   before any nested closure tries to capture it. Lazy allocation
   would race with closure construction.
3. **Try-finally emits a "finally-merge" block all paths converge
   on** — return, break, continue, fall-through, and exception all
   route through finally-merge before exiting the try. Skipping any
   path produces a try where finally never runs.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: hir-to-mir-types
types:
  HirToMir:        { kind: struct, label: "lower_hir_to_mir entry + recursive walker" }
  HirNode:         { kind: enum,   label: "from hir::HirModule etc." }
  MirNode:         { kind: enum,   label: "from mir::MirModule etc." }
  CfgBuilder:      { kind: struct, label: "current block + label / fixup table" }
  TypeContext:     { kind: struct, label: "from types — TypeId per VReg" }
  ModuleRegistry:  { kind: struct, label: "from runtime::module — register variadic / kwargs symbol IDs" }
  ClosureRules:    { kind: struct, label: "from runtime::closure — Var(sym)→LoadGlobal for decorated" }
edges:
  - { from: HirToMir,    to: HirNode,     kind: references, label: "input" }
  - { from: HirToMir,    to: MirNode,     kind: owns,       label: "output" }
  - { from: HirToMir,    to: CfgBuilder,  kind: owns,       label: "block management" }
  - { from: HirToMir,    to: TypeContext, kind: references }
  - { from: HirToMir,    to: ModuleRegistry, kind: references, label: "register variadic / kwargs symbols" }
  - { from: HirToMir,    to: ClosureRules,   kind: references, label: "decorated user func emit" }
---
classDiagram
    class HirToMir
    class HirNode
    class MirNode
    class CfgBuilder
    class TypeContext
    class ModuleRegistry
    class ClosureRules
    HirToMir --> HirNode : input
    HirToMir --> MirNode : output
    HirToMir --> CfgBuilder : owns
    HirToMir --> TypeContext : refs
    HirToMir --> ModuleRegistry : register
    HirToMir --> ClosureRules : decorator emit
```

## CFG-construction shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "hir-to-mir-types"
$defs:
  CfgPattern:
    description: "How HIR control-flow lowers to MIR blocks"
    type: array
    items:
      type: object
      properties:
        hir_form: { type: string }
        cfg_blocks: { type: array, items: { type: string } }
        terminators: { type: array, items: { type: string } }
      required: [hir_form, cfg_blocks]
    examples:
      - - { hir_form: "if c: T else: E", cfg_blocks: [cond, then, else, merge], terminators: [Branch, Goto, Goto] }
        - { hir_form: "while c: body", cfg_blocks: [header, body, exit], terminators: [Branch, Goto-back-edge] }
        - { hir_form: "for x in iter: body", cfg_blocks: [iter-init, header, body, exit], terminators: [Goto, Branch, Goto-back-edge] }
        - { hir_form: "try: T except: H finally: F", cfg_blocks: [try, handler, finally, merge], terminators: [Goto, Goto, Goto] }
        - { hir_form: "return e", cfg_blocks: [], terminators: [Return(e)], description: "no new block; current block sealed" }
        - { hir_form: "break", cfg_blocks: [], terminators: [Goto(loop-exit)] }
        - { hir_form: "continue", cfg_blocks: [], terminators: [Goto(loop-header)] }
        - { hir_form: "raise e", cfg_blocks: [], terminators: [Raise(e); Unreachable] }
  EmissionFlag:
    description: "Per-function flags affecting emit"
    type: object
    properties:
      has_star_args:    { type: boolean, description: "registers symbol id in module::VARIADIC_SYMBOL_IDS" }
      has_kwargs:       { type: boolean, description: "registers in module::KWARGS_SYMBOL_IDS" }
      requires_cells:   { type: boolean, description: "any captured-and-mutated free vars" }
      has_decorators:   { type: boolean, description: "Var(sym)→LoadGlobal lowering enabled" }
```

## Per-statement lowering logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: hir-to-mir-stmt
entry: enter
nodes:
  enter:        { kind: start,    label: "lower_function(HirFunction)" }
  prologue:     { kind: process,  label: "open block 0; emit MakeCell for any captured-and-mutated vars" }
  for_each:     { kind: process,  label: "for each HirStmt: dispatch by variant" }
  is_assign:    { kind: decision, label: "Assign / VarDecl?" }
  emit_assign:  { kind: process,  label: "lower RHS to VReg; emit Copy / SetAttr / SetItem / StoreCell / StoreGlobal" }
  is_call:      { kind: decision, label: "Expr-stmt with Call?" }
  emit_call:    { kind: process,  label: "lower args to VRegs; emit Call / CallExtern" }
  is_branch:    { kind: decision, label: "If / IfExpr?" }
  emit_branch:  { kind: process,  label: "open then/else/merge blocks; emit Branch terminator" }
  is_loop:      { kind: decision, label: "While / For?" }
  emit_loop:    { kind: process,  label: "open header/body/exit; back-edge from body to header" }
  is_try:       { kind: decision, label: "Try?" }
  emit_try:     { kind: process,  label: "open try / handler / finally / merge; route all paths through finally-merge" }
  is_match:     { kind: decision, label: "Match?" }
  emit_match:   { kind: process,  label: "open one block per arm + merge; sequential pattern test" }
  is_return:    { kind: decision, label: "Return / Yield / YieldFrom?" }
  emit_return:  { kind: process,  label: "Return terminator on current block" }
  is_break:     { kind: decision, label: "Break / Continue?" }
  emit_break:   { kind: process,  label: "Goto loop-exit / loop-header" }
  is_global:    { kind: decision, label: "global / nonlocal access?" }
  emit_global:  { kind: process,  label: "LoadGlobal / StoreGlobal (decorated → LoadGlobal even for direct Var)" }
  done:         { kind: terminal, label: "MirBody.blocks finalized" }
edges:
  - { from: enter,        to: prologue }
  - { from: prologue,     to: for_each }
  - { from: for_each,     to: is_assign }
  - { from: is_assign,    to: emit_assign,    label: "yes" }
  - { from: is_assign,    to: is_call,        label: "no" }
  - { from: is_call,      to: emit_call,      label: "yes" }
  - { from: is_call,      to: is_branch,      label: "no" }
  - { from: is_branch,    to: emit_branch,    label: "yes" }
  - { from: is_branch,    to: is_loop,        label: "no" }
  - { from: is_loop,      to: emit_loop,      label: "yes" }
  - { from: is_loop,      to: is_try,         label: "no" }
  - { from: is_try,       to: emit_try,       label: "yes" }
  - { from: is_try,       to: is_match,       label: "no" }
  - { from: is_match,     to: emit_match,     label: "yes" }
  - { from: is_match,     to: is_return,      label: "no" }
  - { from: is_return,    to: emit_return,    label: "yes" }
  - { from: is_return,    to: is_break,       label: "no" }
  - { from: is_break,     to: emit_break,     label: "yes" }
  - { from: is_break,     to: is_global,      label: "no" }
  - { from: is_global,    to: emit_global,    label: "yes" }
  - { from: emit_assign,  to: for_each,       label: "next" }
  - { from: emit_call,    to: for_each }
  - { from: emit_branch,  to: for_each }
  - { from: emit_loop,    to: for_each }
  - { from: emit_try,     to: for_each }
  - { from: emit_match,   to: for_each }
  - { from: emit_return,  to: for_each }
  - { from: emit_break,   to: for_each }
  - { from: emit_global,  to: for_each }
  - { from: for_each,     to: done,           label: "exhausted" }
---
flowchart TD
    enter([lower_function]) --> prologue[block 0 + MakeCell]
    prologue --> for_each[per HirStmt]
    for_each --> is_assign{Assign / VarDecl?}
    is_assign -->|yes| emit_assign[Copy/SetAttr/SetItem/Store*]
    is_assign -->|no| is_call{Call?}
    is_call -->|yes| emit_call[Call / CallExtern]
    is_call -->|no| is_branch{If?}
    is_branch -->|yes| emit_branch[then/else/merge + Branch]
    is_branch -->|no| is_loop{loop?}
    is_loop -->|yes| emit_loop[header/body/exit]
    is_loop -->|no| is_try{Try?}
    is_try -->|yes| emit_try[try/handler/finally/merge]
    is_try -->|no| is_match{Match?}
    is_match -->|yes| emit_match[arms + merge]
    is_match -->|no| is_return{Return?}
    is_return -->|yes| emit_return[Return terminator]
    is_return -->|no| is_break{Break/Continue?}
    is_break -->|yes| emit_break[Goto loop]
    is_break -->|no| is_global{global/nonlocal?}
    is_global -->|yes| emit_global[LoadGlobal/StoreGlobal]
    emit_assign --> for_each
    emit_call --> for_each
    emit_branch --> for_each
    emit_loop --> for_each
    emit_try --> for_each
    emit_match --> for_each
    emit_return --> for_each
    emit_break --> for_each
    emit_global --> for_each
    for_each --> done([MirBody finalized])
```

## Function lowering interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: hir-to-mir-flow
actors:
  - { id: HIR,         kind: system, label: "hir::HirModule" }
  - { id: HirToMir,    kind: system, label: "lower::hir_to_mir" }
  - { id: ModuleReg,   kind: system, label: "runtime::module variadic / kwargs registries" }
  - { id: MIR,         kind: system, label: "mir::MirModule" }
messages:
  - { from: HIR,         to: HirToMir, name: "lower_hir_to_mir(HirModule, TypeContext)" }
  - { from: HirToMir,    to: HirToMir, name: "for each HirFunction: alloc MirBody; open block 0" }
  - { from: HirToMir,    to: ModuleReg, name: "if has_star_args: register_variadic_symbol(sym)" }
  - { from: HirToMir,    to: ModuleReg, name: "if has_kwargs: register_kwargs_symbol(sym)" }
  - { from: HirToMir,    to: HirToMir, name: "emit prologue MakeCell instructions" }
  - { from: HirToMir,    to: HirToMir, name: "walk HirStmts; emit MirInst per stmt; open new blocks at control flow" }
  - { from: HirToMir,    to: HirToMir, name: "for decorated user fn refs in non-call position: emit LoadGlobal not FuncRef (commit 7b4b6af4)" }
  - { from: HirToMir,    to: MIR,      name: "MirBody finalized" }
  - { from: HirToMir,    to: HirToMir, name: "for each HirClass: lower methods recursively" }
  - { from: HirToMir,    to: MIR,      name: "MirModule.bodies + MirModule.externs" }
---
sequenceDiagram
    participant HIR
    participant HirToMir
    participant ModuleReg
    participant MIR
    HIR->>HirToMir: lower_hir_to_mir
    HirToMir->>HirToMir: alloc MirBody
    HirToMir->>ModuleReg: register variadic / kwargs
    HirToMir->>HirToMir: prologue MakeCell
    HirToMir->>HirToMir: walk HirStmts; emit MirInst
    HirToMir->>HirToMir: decorated → LoadGlobal
    HirToMir->>MIR: MirBody finalized
    HirToMir->>HirToMir: lower class methods
    HirToMir->>MIR: MirModule
```

## Acceptance scenarios
<!-- type: overview lang: markdown -->

```mermaid
---
id: hir-to-mir-acceptance
actors:
  - { id: User,    kind: actor }
  - { id: Mamba,   kind: system }
  - { id: Fixture, kind: system }
messages:
  - { from: User,    to: Mamba,   name: "run stdlib/functools_lru_cache.py" }
  - { from: Mamba,   to: Fixture, name: "@lru_cache def fib(n): if n<2: return n; return fib(n-1) + fib(n-2)" }
  - { from: Fixture, to: Mamba,   name: "fib references inside body emit LoadGlobal — wrapper hit (commit 7b4b6af4)" }
  - { from: User,    to: Mamba,   name: "run scope_modifiers/nonlocal_basic.py" }
  - { from: Mamba,   to: Fixture, name: "outer x; nested fn assigns to x" }
  - { from: Fixture, to: Mamba,   name: "MakeCell at outer prologue; nested fn emits LoadCapture / StoreCell" }
  - { from: User,    to: Mamba,   name: "run exceptions/try_finally.py" }
  - { from: Mamba,   to: Fixture, name: "try ... return ... finally ..." }
  - { from: Fixture, to: Mamba,   name: "return routes through finally-merge before exiting" }
  - { from: User,    to: Mamba,   name: "run async_await/gather.py" }
  - { from: Mamba,   to: Fixture, name: "async def f(): ..." }
  - { from: Fixture, to: Mamba,   name: "AsyncFnDef emits coroutine body with Await suspension points" }
---
sequenceDiagram
    actor User
    participant Mamba
    participant Fixture
    User->>Mamba: stdlib/functools_lru_cache.py
    Mamba->>Fixture: @lru_cache fib
    Fixture-->>Mamba: LoadGlobal hits wrapper
    User->>Mamba: scope_modifiers/nonlocal_basic.py
    Mamba->>Fixture: nonlocal write
    Fixture-->>Mamba: MakeCell + LoadCapture + StoreCell
    User->>Mamba: exceptions/try_finally.py
    Mamba->>Fixture: try return finally
    Fixture-->>Mamba: finally-merge route
    User->>Mamba: async_await/gather.py
    Mamba->>Fixture: async def
    Fixture-->>Mamba: coroutine MIR
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
runner: "cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"
fixtures:
  - id: lru_cache_var_load
    name: "stdlib/functools_lru_cache.py"
    paired: "stdlib/functools_lru_cache.expected"
    verifies: ["Var(sym) for decorated fn emits LoadGlobal (commit 7b4b6af4)"]
  - id: nonlocal_cells
    name: "scope_modifiers/nonlocal_basic.py"
    paired: "scope_modifiers/nonlocal_basic.expected"
    verifies: ["MakeCell prologue + LoadCapture / StoreCell in nested fn"]
  - id: try_finally_path
    name: "exceptions/try_finally.py"
    paired: "exceptions/try_finally.expected"
    verifies: ["all exit paths route through finally-merge"]
  - id: async_def
    name: "async_await/gather.py"
    paired: "async_await/gather.expected"
    verifies: ["AsyncFnDef emits coroutine body shape"]
  - id: while_loop
    name: "control_flow/while_basic.py"
    paired: "control_flow/while_basic.expected"
    verifies: ["while emits header/body/exit with back-edge"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/lower/hir_to_mir.rs
    action: modify
    impl_mode: hand-written
    description: "lower_hir_to_mir + lower_function + lower_stmt + lower_expr + CFG construction (if / while / for / try / match); decorated-fn LoadGlobal rule (commit 7b4b6af4); cell prologue allocation; finally-merge path. Hand-written; the lowering contract is the load-bearing surface for codegen."
```
