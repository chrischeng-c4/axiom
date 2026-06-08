---
id: repl
title: REPL — Interactive Read / Eval / Print Loop
crate: mamba
files:
  - crates/mamba/src/driver/repl.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: bf867ab85
---

# REPL

`driver/repl.rs` (445 LOC) implements the `mamba` interactive shell.
Each input cell is parsed as a partial module, lowered through the
same pipeline as `mamba run`, JIT-compiled, and executed. Definitions
persist across cells via `runtime/closure::GLOBAL_BY_ID` and
`runtime/module::MODULES`.

Commit `f383369e0` (#838) added `rustyline` for proper readline
support — line editing, history, `Ctrl-R` search.

Three load-bearing invariants:

1. **REPL state is module-shaped** — each cell looks like a top-level
   `Module.stmts` to the rest of the pipeline. Function and class
   definitions register into the persistent module namespace; later
   cells can reference them by ordinary name lookup.
2. **JIT backends per cell are retained in `MODULE_JIT_BACKENDS`** —
   cell N defines `def f(): ...`; cell N+1 calls `f()`. The fn pointer
   from cell N's JIT compile must outlive cell N's frame, so the
   backend is pushed into the global retention vector (per `module.md`).
3. **`__main__` is the cell namespace** — bare assignments and
   imports go into `__main__` like a normal Python script. This
   matches CPython's REPL semantics.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: repl-types
types:
  Repl:           { kind: struct, label: "repl.rs entry — main loop" }
  Rustyline:      { kind: struct, label: "rustyline crate (readline)" }
  ReplState:      { kind: struct, label: "history + cell counter + persistent module" }
  ParserMod:      { kind: struct, label: "from parser/mod" }
  Lowerer:        { kind: struct, label: "from lower" }
  Codegen:        { kind: struct, label: "from codegen/cranelift/jit" }
  Runtime:        { kind: struct, label: "from runtime — global namespace persists" }
edges:
  - { from: Repl,      to: Rustyline,  kind: references }
  - { from: Repl,      to: ReplState,  kind: owns }
  - { from: Repl,      to: ParserMod,  kind: references }
  - { from: Repl,      to: Lowerer,    kind: references }
  - { from: Repl,      to: Codegen,    kind: references }
  - { from: Repl,      to: Runtime,    kind: references, label: "module + globals persist" }
---
classDiagram
    class Repl
    class Rustyline
    class ReplState
    class ParserMod
    class Lowerer
    class Codegen
    class Runtime
    Repl --> Rustyline : refs
    Repl --> ReplState : owns
    Repl --> ParserMod : refs
    Repl --> Lowerer : refs
    Repl --> Codegen : refs
    Repl --> Runtime : globals persist
```

## REPL state shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "repl-types"
$defs:
  ReplState:
    type: object
    properties:
      history:        { type: array, items: { type: string }, description: "rustyline history buffer" }
      cell_index:     { type: integer, minimum: 0 }
      module_name:    { type: string, const: __main__ }
      jit_backends:
        type: array
        items: { x-rust-type: "Box<CraneliftJitBackend>" }
        description: "retained per cell"
    required: [history, cell_index, module_name, jit_backends]
```

## REPL session lifecycle
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: repl-session
initial: Idle
nodes:
  Idle:        { kind: initial, label: "rustyline waiting for input" }
  Reading:     { kind: normal,  label: "consuming multi-line cell" }
  Parsing:     { kind: normal,  label: "parser attempts module parse" }
  Compiling:   { kind: normal,  label: "lower + codegen + JIT compile" }
  Executing:   { kind: normal,  label: "transmute + call entry fn" }
  Printing:    { kind: normal,  label: "captured stdout + last expr value" }
  Exiting:     { kind: terminal, label: "EOF / quit; cleanup_all_runtime_state" }
edges:
  - { from: Idle,      to: Reading,   event: "first character of cell" }
  - { from: Reading,   to: Reading,   event: "indented line continuation" }
  - { from: Reading,   to: Parsing,   event: "blank line / EOF marks cell end" }
  - { from: Parsing,   to: Compiling, event: "parse ok" }
  - { from: Parsing,   to: Idle,      event: "syntax error; print + reset" }
  - { from: Compiling, to: Executing }
  - { from: Compiling, to: Idle,      event: "type error; print + reset" }
  - { from: Executing, to: Printing }
  - { from: Executing, to: Idle,      event: "runtime exception; print traceback" }
  - { from: Printing,  to: Idle }
  - { from: Idle,      to: Exiting,   event: "Ctrl-D / quit()" }
---
stateDiagram-v2
    [*] --> Idle
    Idle --> Reading: input
    Reading --> Reading: continuation
    Reading --> Parsing: cell end
    Parsing --> Compiling: ok
    Parsing --> Idle: syntax error
    Compiling --> Executing
    Compiling --> Idle: type error
    Executing --> Printing
    Executing --> Idle: runtime exception
    Printing --> Idle
    Idle --> Exiting: EOF
    Exiting --> [*]
```

## Cell pipeline logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: repl-cell-pipeline
entry: enter
nodes:
  enter:       { kind: start,    label: "Repl::run_cell(input)" }
  parse:       { kind: process,  label: "parser::parse_module(input) into Module" }
  is_partial:  { kind: decision, label: "incomplete input (eg unclosed bracket)?" }
  prompt_more: { kind: terminal, label: "secondary prompt; accumulate next line" }
  type_check:  { kind: process,  label: "TypeChecker::check_module" }
  lower:       { kind: process,  label: "lower_hir_to_mir_repl — REPL-specific lowerer that preserves cell-state symbols" }
  codegen:     { kind: process,  label: "JIT compile cell module" }
  retain:      { kind: process,  label: "ReplState.jit_backends.push(boxed); MODULE_JIT_BACKENDS.push too" }
  execute:     { kind: process,  label: "transmute entry; call; capture stdout" }
  display:     { kind: process,  label: "print captured stdout; if last stmt is Expr, also print its repr" }
  done:        { kind: terminal, label: "back to Idle" }
edges:
  - { from: enter,       to: parse }
  - { from: parse,       to: is_partial }
  - { from: is_partial,  to: prompt_more, label: "yes" }
  - { from: is_partial,  to: type_check,  label: "no" }
  - { from: type_check,  to: lower }
  - { from: lower,       to: codegen }
  - { from: codegen,     to: retain }
  - { from: retain,      to: execute }
  - { from: execute,     to: display }
  - { from: display,     to: done }
---
flowchart TD
    enter([run_cell]) --> parse[parser parse_module]
    parse --> is_partial{incomplete?}
    is_partial -->|yes| prompt_more([secondary prompt])
    is_partial -->|no| type_check[check_module]
    type_check --> lower[lower_hir_to_mir_repl]
    lower --> codegen[JIT compile]
    codegen --> retain[push backend]
    retain --> execute[transmute + call]
    execute --> display[print captured]
    display --> done([Idle])
```

## Cell-to-cell interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: repl-cell-flow
actors:
  - { id: User,     kind: actor }
  - { id: Repl,     kind: system }
  - { id: Pipeline, kind: system }
  - { id: Globals,  kind: system, label: "GLOBAL_BY_ID / GLOBAL_NAMES" }
  - { id: Backends, kind: system, label: "MODULE_JIT_BACKENDS" }
messages:
  - { from: User,     to: Repl,     name: "Cell 1: def f(): return 42" }
  - { from: Repl,     to: Pipeline, name: "compile cell" }
  - { from: Pipeline, to: Globals,  name: "GLOBAL_BY_ID[f] = wrapper closure" }
  - { from: Pipeline, to: Backends, name: "retain JIT backend" }
  - { from: Repl,     to: User,     name: "(no output)" }
  - { from: User,     to: Repl,     name: "Cell 2: print(f())" }
  - { from: Repl,     to: Pipeline, name: "compile cell" }
  - { from: Pipeline, to: Globals,  name: "look up f → call it" }
  - { from: Pipeline, to: Repl,     name: "captured stdout '42'" }
  - { from: Repl,     to: User,     name: "42" }
---
sequenceDiagram
    actor User
    participant Repl
    participant Pipeline
    participant Globals
    participant Backends
    User->>Repl: cell 1 def f
    Repl->>Pipeline: compile
    Pipeline->>Globals: f wrapper
    Pipeline->>Backends: retain
    Repl-->>User: (silent)
    User->>Repl: cell 2 print f()
    Repl->>Pipeline: compile
    Pipeline->>Globals: lookup f
    Pipeline-->>Repl: 42
    Repl-->>User: 42
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->
```yaml
scenarios:
  - id: repl-start
    given: the user starts mamba without a script
    when: the REPL initializes
    then: it displays the primary prompt and waits for input
  - id: persistent-variable
    given: a prior cell assigns x = 1
    when: a later cell evaluates x + 1
    then: the REPL prints 2 using the persistent __main__ namespace
  - id: persistent-function
    given: a prior cell defines def f(): return 42
    when: a later cell evaluates f()
    then: the retained JIT backend and globals allow the call to return 42
  - id: repl-exit
    given: the REPL is idle
    when: the user sends Ctrl-D or quit()
    then: it exits and runs cleanup_all_runtime_state
```

## Tests
<!-- type: test-plan lang: mermaid -->
```mermaid
---
id: repl-test-plan
title: REPL Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test runtime_tests --release -- {name} --test-threads=1"]
    Runner --> Basic["test_repl_basic_cells"]
    Runner --> DefCall["test_repl_def_then_call"]
    Runner --> Partial["test_repl_secondary_prompt"]
    Runner --> History["test_repl_history"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/driver/repl.rs
    action: modify
    impl_mode: hand-written
    description: "Repl entry + ReplState + cell pipeline + rustyline integration. Hand-written; cross-cell namespace persistence is the contract."
```
