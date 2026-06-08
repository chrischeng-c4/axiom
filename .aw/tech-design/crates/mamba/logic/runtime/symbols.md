---
id: symbols
title: Runtime Symbol Table for JIT Linking
crate: mamba
files:
  - crates/mamba/src/runtime/symbols.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: 146f6c211
---

# Runtime Symbol Table

`runtime/symbols.rs` is the JIT linker's catalog. It enumerates every
`mb_*` function the runtime exposes — name, raw function pointer, MIR
parameter types, MIR return type — so the Cranelift backend can resolve
calls emitted by lowering. Two functions: `runtime_symbols()` returns
the catalog as `Vec<RuntimeSymbol>`; `runtime_externs()` wraps it as
`Vec<MirExtern>` for the MIR pre-link pass.

Three load-bearing invariants:

1. **Every JIT-emitted `mb_*` call must appear here** — symbol lookup
   at link time fails fast if missing, so a runtime function that's
   called but not registered produces a clear MIR-link error rather
   than a delayed unsafe pointer dereference.
2. **Function-pointer addresses are static-lifetime safe** — Rust's
   guarantee that `extern "C" fn` items live for `'static` is what
   makes `addr: *const u8` valid here without retain. If a function
   were ever generated dynamically, its address would have to be kept
   alive separately (see `module.md` `MODULE_JIT_BACKENDS`).
3. **MIR type signatures are part of the contract** — the JIT lowers
   call sites against the declared signature; passing a Rust fn with
   a different signature than what's declared would produce
   well-formed MIR but undefined behavior at runtime. Code-review
   guard: every entry in `runtime_symbols` must match the actual
   `pub fn` it points to.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: symbols-types
types:
  RuntimeSymbol:    { kind: struct, label: "name + addr + params + return_type" }
  MirType:          { kind: enum, label: "from crate::mir" }
  MirExtern:        { kind: struct, label: "from crate::mir (linker view)" }
  RuntimeSymbols:   { kind: struct, label: "fn returning Vec<RuntimeSymbol>" }
  RuntimeExterns:   { kind: struct, label: "fn returning Vec<MirExtern>" }
  CraneliftJit:     { kind: struct, label: "JIT backend (consumes externs)" }
  RuntimeModules:   { kind: struct, label: "every mb_* function across builtins, string_ops, list_ops, dict_ops, tuple_ops, set_ops, exception, class, iter, generator, closure, module, async_rt, file_io, tokio_exec" }
edges:
  - { from: RuntimeSymbol,  to: MirType,        kind: references, label: "params + return" }
  - { from: RuntimeSymbols, to: RuntimeSymbol,  kind: owns,       label: "Vec items" }
  - { from: RuntimeExterns, to: MirExtern,      kind: owns }
  - { from: RuntimeExterns, to: RuntimeSymbols, kind: references, label: "wraps the catalog" }
  - { from: CraneliftJit,   to: RuntimeExterns, kind: references, label: "consumes at link time" }
  - { from: RuntimeSymbols, to: RuntimeModules, kind: references, label: "addresses come from these" }
---
classDiagram
    class RuntimeSymbol
    class MirType
    class MirExtern
    class RuntimeSymbols
    class RuntimeExterns
    class CraneliftJit
    class RuntimeModules
    RuntimeSymbol --> MirType : params + return
    RuntimeSymbols --> RuntimeSymbol : owns
    RuntimeExterns --> MirExtern : owns
    RuntimeExterns --> RuntimeSymbols : wraps
    CraneliftJit --> RuntimeExterns : link
    RuntimeSymbols --> RuntimeModules : addresses from
```

## Symbol shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "symbols-types"
$defs:
  RuntimeSymbol:
    type: object
    x-rust-type: RuntimeSymbol
    properties:
      name:        { type: string, description: "exact mb_* identifier; matches lowering's call-site spelling" }
      addr:        { type: integer, x-rust-type: "*const u8", description: "function pointer cast to byte ptr" }
      params:      { type: array, items: { x-rust-type: MirType } }
      return_type: { x-rust-type: MirType }
    required: [name, addr, params, return_type]
  MirTypeUsedHere:
    description: "Subset of MirType used by runtime symbols (from crate::mir)"
    type: string
    enum: [Int, Float, Bool, Ptr, Str, List, Dict, Tuple, Void, Any]
```

## Registration logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: symbols-registration
entry: enter
nodes:
  enter:        { kind: start,    label: "runtime_symbols() | runtime_externs()" }
  is_externs:   { kind: decision, label: "externs vs symbols?" }
  build_syms:   { kind: process,  label: "alloc Vec; rt_sym! macro per fn" }
  walk_modules: { kind: process,  label: "for each runtime module: register all mb_* with declared MIR types" }
  return_syms:  { kind: terminal, label: "return Vec<RuntimeSymbol>" }
  call_syms:    { kind: process,  label: "let syms = runtime_symbols()" }
  wrap_externs: { kind: process,  label: "for each: MirExtern { name, params, return_type }" }
  return_ext:   { kind: terminal, label: "return Vec<MirExtern>" }
edges:
  - { from: enter,        to: is_externs }
  - { from: is_externs,   to: build_syms,   label: "symbols" }
  - { from: is_externs,   to: call_syms,    label: "externs" }
  - { from: build_syms,   to: walk_modules }
  - { from: walk_modules, to: return_syms }
  - { from: call_syms,    to: wrap_externs }
  - { from: wrap_externs, to: return_ext }
---
flowchart TD
    enter([symbols / externs]) --> is_externs{which?}
    is_externs -->|symbols| build_syms[Vec; rt_sym! macro]
    is_externs -->|externs| call_syms[runtime_symbols]
    build_syms --> walk_modules[per module: register]
    walk_modules --> return_syms([Vec RuntimeSymbol])
    call_syms --> wrap_externs[wrap as MirExtern]
    wrap_externs --> return_ext([Vec MirExtern])
```

## Link-time interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: jit-link-flow
actors:
  - { id: Lowering, kind: system, label: "MIR lowering — emits 'call mb_X' instructions" }
  - { id: Symbols,  kind: system, label: "runtime/symbols.rs" }
  - { id: Cranelift, kind: system, label: "CraneliftJitBackend" }
  - { id: Module,   kind: system, label: "compiled module body" }
messages:
  - { from: Lowering,  to: Lowering, name: "emit MIR with call to 'mb_iter' / 'mb_add' / ..." }
  - { from: Cranelift, to: Symbols,  name: "runtime_externs()" }
  - { from: Symbols,   to: Cranelift, name: "Vec<MirExtern> with name / params / return_type" }
  - { from: Cranelift, to: Cranelift, name: "declare each extern in module" }
  - { from: Cranelift, to: Symbols,  name: "for each call site: addr = lookup(name).addr" }
  - { from: Symbols,   to: Cranelift, name: "*const u8 fn pointer" }
  - { from: Cranelift, to: Module,   name: "patch call to absolute address; finalize" }
  - { from: Module,    to: Module,   name: "module body now callable; calls mb_* directly" }
---
sequenceDiagram
    participant Lowering
    participant Symbols
    participant Cranelift
    participant Module
    Lowering->>Lowering: emit call mb_X
    Cranelift->>Symbols: runtime_externs
    Symbols-->>Cranelift: Vec MirExtern
    Cranelift->>Cranelift: declare externs
    Cranelift->>Symbols: lookup addr per name
    Symbols-->>Cranelift: fn pointer
    Cranelift->>Module: patch + finalize
    Module->>Module: callable now
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->
```yaml
scenarios:
  - id: registered-runtime-symbol
    given: a contributor adds mb_foo and an rt_sym entry with declared MirType params
    when: the JIT module declares and links mb_foo
    then: symbol lookup returns the static function pointer and module calls succeed
  - id: missing-runtime-symbol
    given: lowering emits a call to mb_foo without a runtime_symbols entry
    when: the JIT links the module
    then: MIR-link fails loudly before runtime execution
```

## Tests
<!-- type: test-plan lang: mermaid -->
```mermaid
---
id: runtime-symbols-test-plan
title: Runtime Symbol Table Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test runtime_tests --release -- {name} --test-threads=1"]
    Runner --> Complete["runtime_symbols_test"]
    Runner --> TypeMatch["runtime_symbols_type_match"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/symbols.rs
    action: modify
    impl_mode: hand-written
    description: "RuntimeSymbol struct, runtime_symbols() catalog (~all mb_* across runtime), runtime_externs() MIR-link wrapper, rt_sym! macro for entry construction. Hand-written; every new mb_* fn requires an rt_sym! entry."
```
