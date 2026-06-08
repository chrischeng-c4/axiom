---
id: stdlib-native-implementations
title: stdlib Native Implementation Architecture
crate: mamba
files:
  - crates/mamba/src/runtime/stdlib
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: 770cb0f94
---

# stdlib native-implementations architecture

Cross-cutting spec describing how the ~97 stdlib modules in
`runtime/stdlib/*_mod.rs` plug into the runtime. Common pattern:

1. Each module exposes a `pub fn register()` that builds a
   `HashMap<String, MbValue>` of attrs (function pointers as
   FUNC-tagged MbValue, or class Instances) and calls
   `super::register_module(name, attrs)` to add to `MODULES`.
2. Each `mb_*` function uses either an `extern "C" fn` ABI for
   native dispatch (per `runtime/symbols.md` + `runtime/module.md`
   `NATIVE_FUNC_ADDRS`) or a `pub fn(args) -> MbValue` form callable
   via the `runtime_symbols.rs` registration table.
3. Per-module `register()` is called once at startup from
   `runtime::module::mb_register_native_modules`.

Three load-bearing invariants:

1. **Native module attrs are bound at startup, not first import** —
   the module appears in `MODULES` immediately; first `import X`
   just retrieves the cached `MbModule`.
2. **Function pointer + class Instance attrs are equivalent** — the
   user-side experience (`math.sin(x)`, `re.compile(p)`,
   `collections.deque()`) is the same whether the attr is a callable
   FUNC tag or a class Instance with `__call__`.
3. **`extern "C" fn dispatch_X` wrappers go through `mb_call_spread`
   ABI** — required for callable-as-value patterns
   (`f = math.sin; f(x)`). Pure `pub fn` symbols would not be
   reach via `mb_call_spread` and break alias support.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: native-impl-types
types:
  StdlibModule:    { kind: struct, label: "any *_mod.rs file" }
  RegisterFn:      { kind: struct, label: "pub fn register() — runs once at startup" }
  ModuleRegistry:  { kind: struct, label: "from runtime::module MODULES" }
  RuntimeSymbols:  { kind: struct, label: "from runtime::symbols (rt_sym! catalog)" }
  NativeFuncAddrs: { kind: struct, label: "from runtime::module" }
  ImportSystem:    { kind: struct }
edges:
  - { from: StdlibModule,    to: RegisterFn,      kind: owns }
  - { from: RegisterFn,      to: ModuleRegistry,  kind: references, label: "register_module(name, attrs)" }
  - { from: RegisterFn,      to: NativeFuncAddrs, kind: references, label: "register dispatch_X addrs" }
  - { from: StdlibModule,    to: RuntimeSymbols,  kind: references, label: "mb_X registered for JIT link" }
  - { from: ImportSystem,    to: ModuleRegistry,  kind: references, label: "import X retrieves cached MbModule" }
---
classDiagram
    class StdlibModule
    class RegisterFn
    class ModuleRegistry
    class RuntimeSymbols
    class NativeFuncAddrs
    class ImportSystem
    StdlibModule --> RegisterFn : owns
    RegisterFn --> ModuleRegistry : register_module
    RegisterFn --> NativeFuncAddrs : addrs
    StdlibModule --> RuntimeSymbols : rt_sym!
    ImportSystem --> ModuleRegistry : import
```

## Module-registration shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "native-impl-types"
$defs:
  ModuleRegistration:
    description: "Pattern every *_mod.rs follows"
    type: object
    properties:
      module_name:    { type: string, description: "Python-visible name (e.g. math, re, collections)" }
      attrs_built_in: { type: array, items: { type: string }, description: "list of attr names" }
      register_called_from: { type: string, const: "runtime::module::mb_register_native_modules" }
      rt_sym_entries: { type: array, items: { type: string }, description: "names in rt_sym! catalog" }
      native_dispatch_wrappers:
        type: array
        items: { type: string }
        description: "extern C fn dispatch_X for mb_call_spread compatibility"
    required: [module_name, attrs_built_in, register_called_from]
```

## Registration logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: native-mod-register
entry: enter
nodes:
  enter:        { kind: start,    label: "mb_register_native_modules() at runtime startup" }
  iter_mods:    { kind: process,  label: "for each *_mod.rs: call register()" }
  build_attrs:  { kind: process,  label: "register() builds HashMap<String, MbValue> of attrs" }
  add_native:   { kind: process,  label: "for each callable attr: register addr in NATIVE_FUNC_ADDRS" }
  call_register: { kind: process, label: "super::register_module(name, attrs) — pushes into MODULES" }
  done:         { kind: terminal, label: "all native modules ready for import" }
edges:
  - { from: enter,         to: iter_mods }
  - { from: iter_mods,     to: build_attrs,  label: "per module" }
  - { from: build_attrs,   to: add_native }
  - { from: add_native,    to: call_register }
  - { from: call_register, to: iter_mods,    label: "next" }
  - { from: iter_mods,     to: done,         label: "all done" }
---
flowchart TD
    enter([startup]) --> iter_mods[per *_mod.rs]
    iter_mods --> build_attrs[build attrs map]
    build_attrs --> add_native[NATIVE_FUNC_ADDRS]
    add_native --> call_register[register_module]
    call_register --> iter_mods
    iter_mods --> done([all ready])
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
runner: "cargo test -p mamba --test runtime_tests --release -- {name} --test-threads=1"
fixtures:
  - id: native_module_import
    name: "test_native_module_import_succeeds"
    description: "every registered native module is importable; attrs callable"
  - id: native_alias_callable
    name: "test_native_alias_callable"
    description: "f = math.sin; f(x) works (extern C ABI)"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/stdlib
    action: modify
    impl_mode: hand-written
    description: "Cross-cutting registration pattern across ~97 stdlib *_mod.rs files. Hand-written; the register() + extern C dispatch pattern is the contract — adding a new stdlib module = a new file following the pattern + entry in mb_register_native_modules."
```
