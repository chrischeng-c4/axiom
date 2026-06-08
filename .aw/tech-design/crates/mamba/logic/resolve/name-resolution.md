---
id: name-resolution
title: Name Resolution — Scopes, Symbols, global / nonlocal
crate: mamba
files:
  - crates/mamba/src/resolve/pass.rs
  - crates/mamba/src/resolve/scope.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: a47e76722
---

# Name Resolution

`resolve/scope.rs` defines the symbol-table types (`SymbolId`,
`SymbolInfo`, `SymbolKind`, `VariableClass`, `Scope`, `SymbolTable`).
`resolve/pass.rs` (2328 LOC) walks the AST building scopes and
classifying every name into one of `Local` / `Global` / `Free` / `Cell`.

The classification drives MIR emission: `Local` → ordinary
function-local stack slot; `Global` → `LoadGlobal` / `StoreGlobal`;
`Free` → `LoadCapture` from closure environment; `Cell` → enclosing
function allocates a `MakeCell` so the inner `Free` reference can
read / write through it.

Three load-bearing invariants:

1. **`global X` and `nonlocal X` declarations rebind X's class** —
   without them, an assignment in nested scope creates a new local;
   with them, the existing outer binding is targeted. The pass walks
   declarations *before* assignments so the classification is right
   on the first use.
2. **`Cell` is a downgrade from `Local`** — an outer-function variable
   starts as `Local`; if any inner scope captures-and-mutates it,
   the resolver promotes it to `Cell`. The MIR emitter then knows to
   allocate `MakeCell` in the outer prologue.
3. **`nonlocal_mapping` records inner-Free → outer-Cell** — closures
   capture by Cell handle, not by name; the inner `LoadCapture(idx)`
   indirects through the captured cell list using indices set at
   resolution time.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: name-resolution-types
types:
  ResolvePass:    { kind: struct, label: "pass.rs — AST walker that populates SymbolTable" }
  SymbolTable:    { kind: struct, label: "scopes + symbols + var_classes + nonlocal_mapping" }
  Scope:          { kind: struct, label: "parent + symbols map" }
  SymbolId:       { kind: struct, label: "u32 — primary key" }
  SymbolInfo:     { kind: struct, label: "id + name + kind" }
  SymbolKind:     { kind: enum, label: "Variable / Function / Parameter / Class / Enum / EnumVariant / Module" }
  VariableClass:  { kind: enum, label: "Local / Global / Free / Cell" }
  HirBuilder:     { kind: struct, label: "lower::ast_to_hir consumer" }
  ModuleRegistry: { kind: struct, label: "from runtime::module — alpha-resolved global symbols" }
edges:
  - { from: ResolvePass,  to: SymbolTable, kind: owns }
  - { from: SymbolTable,  to: Scope,       kind: owns }
  - { from: SymbolTable,  to: SymbolInfo,  kind: owns }
  - { from: SymbolInfo,   to: SymbolKind,  kind: owns }
  - { from: SymbolTable,  to: VariableClass, kind: references, label: "var_classes map" }
  - { from: HirBuilder,   to: SymbolTable, kind: references, label: "lookup SymbolId per AST Name" }
  - { from: ResolvePass,  to: ModuleRegistry, kind: references, label: "global names → registry IDs" }
---
classDiagram
    class ResolvePass
    class SymbolTable
    class Scope
    class SymbolId
    class SymbolInfo
    class SymbolKind
    class VariableClass
    class HirBuilder
    class ModuleRegistry
    ResolvePass --> SymbolTable : owns
    SymbolTable --> Scope : owns
    SymbolTable --> SymbolInfo : owns
    SymbolInfo --> SymbolKind : kind
    SymbolTable --> VariableClass : classes
    HirBuilder --> SymbolTable : lookup
    ResolvePass --> ModuleRegistry : globals
```

## Symbol shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "name-resolution-types"
$defs:
  SymbolKind:
    type: string
    enum: [Variable, Function, Parameter, Class, Enum, EnumVariant, Module]
  VariableClass:
    type: string
    enum: [Local, Global, Free, Cell]
    description: "Local: stack slot. Global: GLOBAL_BY_ID. Free: outer-fn capture. Cell: outer-fn local promoted to mutable cell."
  SymbolInfo:
    type: object
    properties:
      id:   { x-rust-type: SymbolId }
      name: { type: string }
      kind: { $ref: "#/$defs/SymbolKind" }
    required: [id, name, kind]
  ResolutionDecision:
    description: "How a name reference is classified"
    type: object
    properties:
      lookup_chain:    { type: array, items: { type: string }, description: "scope ancestors walked" }
      declarator:      { type: string, description: "global / nonlocal / first-use" }
      result_class:    { $ref: "#/$defs/VariableClass" }
      requires_cell:   { type: boolean, description: "outer must MakeCell in prologue" }
    required: [lookup_chain, declarator, result_class, requires_cell]
```

## Classification lifecycle
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: variable-class-lifecycle
initial: Unbound
nodes:
  Unbound:    { kind: initial,  label: "name first encountered" }
  Local:      { kind: normal,   label: "first use is assignment in current scope; no global / nonlocal" }
  Global:     { kind: normal,   label: "global X declaration in scope; or top-level binding" }
  Free:       { kind: normal,   label: "name found in enclosing scope; not declared local here" }
  Cell:       { kind: normal,   label: "Local later promoted because inner scope captured-and-mutated" }
  ModuleEnd:  { kind: terminal, label: "all classifications committed; SymbolTable frozen" }
edges:
  - { from: Unbound, to: Local,  event: "first-use is assignment without nonlocal/global" }
  - { from: Unbound, to: Global, event: "global X declaration" }
  - { from: Unbound, to: Global, event: "top-level scope assignment" }
  - { from: Unbound, to: Free,   event: "read-only reference to enclosing-scope name" }
  - { from: Local,   to: Cell,   event: "inner scope captures-and-mutates (nonlocal X with assignment)" }
  - { from: Local,   to: ModuleEnd, event: "resolution complete" }
  - { from: Global,  to: ModuleEnd, event: "resolution complete" }
  - { from: Free,    to: ModuleEnd, event: "resolution complete" }
  - { from: Cell,    to: ModuleEnd, event: "resolution complete" }
---
stateDiagram-v2
    [*] --> Unbound
    Unbound --> Local: first-use assign
    Unbound --> Global: global decl / top-level
    Unbound --> Free: outer-scope ref
    Local --> Cell: captured-and-mutated by inner
    Local --> ModuleEnd: complete
    Global --> ModuleEnd: complete
    Free --> ModuleEnd: complete
    Cell --> ModuleEnd: complete
    ModuleEnd --> [*]
```

## Resolution dispatch
<!-- type: logic lang: mermaid -->

```mermaid
---
id: name-resolve-dispatch
entry: enter
nodes:
  enter:        { kind: start,    label: "ResolvePass walks AST" }
  enter_scope:  { kind: process,  label: "FnDef / ClassDef / Lambda → push Scope" }
  collect_decls: { kind: process, label: "Pre-pass body for global / nonlocal declarations" }
  process_stmt: { kind: process,  label: "for each Stmt: handle bindings + references" }
  is_assign:    { kind: decision, label: "Stmt is assignment to Name?" }
  classify_lhs: { kind: process,  label: "global decl → Global; nonlocal → Free + outer Cell; else Local" }
  is_ref:       { kind: decision, label: "Stmt references Name?" }
  walk_chain:   { kind: process,  label: "lookup in current scope; ascend parents" }
  bind_free:    { kind: process,  label: "found in enclosing fn → Free; record nonlocal_mapping" }
  bind_global:  { kind: process,  label: "found at module scope → Global" }
  bind_builtin: { kind: process,  label: "found in builtin scope → Global with builtin marker" }
  unbound_err:  { kind: terminal, label: "NameError" }
  exit_scope:   { kind: process,  label: "FnDef / ClassDef body done → pop Scope" }
  done:         { kind: terminal, label: "all symbols classified" }
edges:
  - { from: enter,        to: enter_scope }
  - { from: enter_scope,  to: collect_decls }
  - { from: collect_decls, to: process_stmt }
  - { from: process_stmt, to: is_assign }
  - { from: is_assign,    to: classify_lhs, label: "yes" }
  - { from: is_assign,    to: is_ref,       label: "no" }
  - { from: is_ref,       to: walk_chain,   label: "yes" }
  - { from: is_ref,       to: process_stmt, label: "next" }
  - { from: walk_chain,   to: bind_free,    label: "found enclosing" }
  - { from: walk_chain,   to: bind_global,  label: "found module" }
  - { from: walk_chain,   to: bind_builtin, label: "found builtin" }
  - { from: walk_chain,   to: unbound_err,  label: "not found" }
  - { from: classify_lhs, to: process_stmt, label: "next" }
  - { from: bind_free,    to: process_stmt }
  - { from: bind_global,  to: process_stmt }
  - { from: bind_builtin, to: process_stmt }
  - { from: process_stmt, to: exit_scope,   label: "body done" }
  - { from: exit_scope,   to: done }
---
flowchart TD
    enter([resolve walk]) --> enter_scope[push Scope]
    enter_scope --> collect_decls[pre-pass global / nonlocal]
    collect_decls --> process_stmt[per Stmt]
    process_stmt --> is_assign{assign?}
    is_assign -->|yes| classify_lhs[Local / Global / Cell]
    is_assign -->|no| is_ref{ref?}
    is_ref -->|yes| walk_chain[scope ascent]
    is_ref -->|no| process_stmt
    walk_chain -->|enclosing| bind_free[Free + nonlocal_mapping]
    walk_chain -->|module| bind_global[Global]
    walk_chain -->|builtin| bind_builtin[Global builtin]
    walk_chain -->|none| unbound_err([NameError])
    classify_lhs --> process_stmt
    bind_free --> process_stmt
    bind_global --> process_stmt
    bind_builtin --> process_stmt
    process_stmt --> exit_scope[pop]
    exit_scope --> done([symbols classified])
```

## Free + Cell interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: free-cell-flow
actors:
  - { id: Outer,    kind: system, label: "outer fn body" }
  - { id: Inner,    kind: system, label: "nested fn body" }
  - { id: Resolver, kind: system, label: "resolve pass" }
  - { id: Symbols,  kind: system, label: "SymbolTable" }
messages:
  - { from: Resolver, to: Symbols, name: "outer fn defines x → Local" }
  - { from: Resolver, to: Symbols, name: "inner fn ascends scope; finds x in outer → Free" }
  - { from: Resolver, to: Symbols, name: "inner fn assigns x (or has nonlocal x); record nonlocal_mapping[inner_x] = outer_x" }
  - { from: Resolver, to: Symbols, name: "promote outer_x: Local → Cell" }
  - { from: Symbols,  to: Outer,   name: "MakeCell at outer prologue" }
  - { from: Symbols,  to: Inner,   name: "LoadCapture / StoreCell with cell index" }
---
sequenceDiagram
    participant Resolver
    participant Symbols
    participant Outer
    participant Inner
    Resolver->>Symbols: outer defines x = Local
    Resolver->>Symbols: inner finds x = Free
    Resolver->>Symbols: nonlocal_mapping
    Resolver->>Symbols: outer x → Cell
    Symbols-->>Outer: MakeCell prologue
    Symbols-->>Inner: LoadCapture / StoreCell
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: global-declaration
    given: scope_modifiers/global_basic.py declares global x inside a function
    when: name resolution classifies assignments to x
    then: the function binding is Global and updates the module binding
  - id: nonlocal-declaration
    given: scope_modifiers/nonlocal_basic.py mutates x from a nested function
    when: name resolution processes nonlocal x
    then: the outer x is promoted to Cell and the inner x is Free
  - id: closure-capture-loop
    given: functional/closure_capture_loop.py captures a loop variable in a closure
    when: the resolver records captures
    then: the loop variable is captured through a cell rather than copied by value
  - id: undefined-name
    given: language/name_error.py references an undefined name
    when: resolution walks the scope chain
    then: it emits a NameError before runtime lowering
```

## Tests
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: name-resolution-test-plan
title: Name Resolution Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"]
    Runner --> GlobalDecl["scope_modifiers/global_basic.py / .expected"]
    Runner --> NonlocalDecl["scope_modifiers/nonlocal_basic.py / .expected"]
    Runner --> ClosureCapture["functional/closure_capture_loop.py / .expected"]
    Runner --> NameError["language/name_error.py / .expected"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/resolve/pass.rs
    action: modify
    impl_mode: hand-written
    description: "AST walker — pre-pass collects global/nonlocal decls; classifies each binding into Local / Global / Free / Cell; populates SymbolTable. Hand-written; classification rules are the contract for cell allocation in lower::hir_to_mir."
  - file: crates/mamba/src/resolve/scope.rs
    action: modify
    impl_mode: hand-written
    description: "SymbolId / SymbolInfo / SymbolKind / VariableClass / Scope / SymbolTable. Hand-written."
```
