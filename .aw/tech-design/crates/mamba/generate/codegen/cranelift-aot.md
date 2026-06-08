---
id: cranelift-aot
title: Cranelift AOT — Object File Output
crate: mamba
files:
  - crates/mamba/src/codegen/cranelift/aot.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: 5236629f5
---

# Cranelift AOT

`codegen/cranelift/aot.rs` is the AOT (ahead-of-time) variant of the
Cranelift backend. Same MIR-to-clir lowering as `cranelift.md` but
emits ELF / Mach-O object bytes via `cranelift_object` instead of
loading machine code into the JIT in-process. AOT output is consumed
by `mamba build` to produce shareable binaries.

Three load-bearing invariants:

1. **AOT and JIT share one MIR pipeline** — they differ only in
   `cranelift_object::ObjectModule` vs `cranelift_jit::JITModule`
   construction. Lowering rules in `cranelift/mod.rs` are identical.
2. **Object output is platform-native** — Mach-O on macOS, ELF on
   Linux. `cranelift_object` picks the right format from
   `target_lexicon::Triple`.
3. **Runtime symbols become unresolved externs in the object** — the
   AOT object has unresolved references to every `mb_*` symbol; a
   subsequent link step resolves them against `libmamba_runtime.a`
   (or equivalent).

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: cranelift-aot-types
types:
  AotBackend:    { kind: struct }
  ObjectModule:  { kind: struct, label: "cranelift_object::ObjectModule" }
  TargetTriple:  { kind: struct, label: "target_lexicon::Triple" }
  MIR:           { kind: struct, label: "from mir/mir" }
  RuntimeSymbols: { kind: struct, label: "from runtime/symbols (declared as externs)" }
edges:
  - { from: AotBackend,   to: ObjectModule, kind: owns }
  - { from: AotBackend,   to: TargetTriple, kind: owns }
  - { from: AotBackend,   to: MIR,          kind: references }
  - { from: AotBackend,   to: RuntimeSymbols, kind: references, label: "declare unresolved externs" }
---
classDiagram
    class AotBackend
    class ObjectModule
    class TargetTriple
    class MIR
    class RuntimeSymbols
    AotBackend --> ObjectModule : owns
    AotBackend --> TargetTriple : owns
    AotBackend --> MIR : refs
    AotBackend --> RuntimeSymbols : externs
```

## Output shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "cranelift-aot-types"
$defs:
  AotOutput:
    type: object
    properties:
      bytes:    { type: array, items: { type: integer, minimum: 0, maximum: 255 } }
      target:   { type: string, description: "target triple (e.g., aarch64-apple-darwin)" }
      format:   { type: string, enum: [ELF, MachO, COFF] }
      externs:  { type: array, items: { type: string }, description: "unresolved symbol names" }
    required: [bytes, target, format, externs]
```

## AOT compilation logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: aot-compile
entry: enter
nodes:
  enter:        { kind: start,    label: "compile_aot(MirModule, target)" }
  init_obj:    { kind: process,  label: "ObjectModule::new(target_triple, builder)" }
  declare_externs: { kind: process, label: "for each runtime extern: declare as Linkage::Import" }
  per_body:    { kind: process,  label: "for each MirBody: define Function (Linkage::Export)" }
  lower:       { kind: process,  label: "shared MIR-to-clir lowering (per cranelift.md)" }
  finalize:    { kind: process,  label: "module.finish() — produce ObjectProduct" }
  emit_bytes:  { kind: terminal, label: "ObjectProduct.emit() → Vec<u8>" }
edges:
  - { from: enter,           to: init_obj }
  - { from: init_obj,        to: declare_externs }
  - { from: declare_externs, to: per_body }
  - { from: per_body,        to: lower }
  - { from: lower,           to: per_body, label: "next body" }
  - { from: per_body,        to: finalize, label: "all done" }
  - { from: finalize,        to: emit_bytes }
---
flowchart TD
    enter([compile_aot]) --> init_obj[ObjectModule]
    init_obj --> declare_externs[Linkage::Import]
    declare_externs --> per_body[per MirBody]
    per_body --> lower[shared MIR-to-clir]
    lower --> per_body
    per_body --> finalize[module.finish]
    finalize --> emit_bytes([Vec u8])
```

## AOT build interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: aot-build-flow
actors:
  - { id: Driver,    kind: system, label: "compiler driver / mamba build" }
  - { id: Aot,       kind: system, label: "cranelift/aot.rs" }
  - { id: Linker,    kind: system, label: "system linker (ld / cc)" }
  - { id: Runtime,   kind: system, label: "libmamba_runtime.a" }
messages:
  - { from: Driver, to: Aot,     name: "compile_aot(MirModule, target)" }
  - { from: Aot,    to: Aot,     name: "lower MIR + finalize ObjectModule" }
  - { from: Aot,    to: Driver,  name: "object bytes" }
  - { from: Driver, to: Driver,  name: "write target/<name>.o" }
  - { from: Driver, to: Linker,  name: "ld -o exe my.o + libmamba_runtime.a" }
  - { from: Linker, to: Runtime, name: "resolve mb_* extern symbols" }
  - { from: Linker, to: Driver,  name: "executable produced" }
---
sequenceDiagram
    participant Driver
    participant Aot
    participant Linker
    participant Runtime
    Driver->>Aot: compile_aot
    Aot->>Aot: lower + finalize
    Aot-->>Driver: object bytes
    Driver->>Driver: write .o
    Driver->>Linker: ld with runtime.a
    Linker->>Runtime: resolve mb_*
    Linker-->>Driver: executable
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: aot-hello
    given: hello.py is compiled with `mamba build hello.py -o hello`
    when: the AOT backend emits and links an object file
    then: the hello executable is produced with runtime symbols resolved
  - id: aot-execution
    given: the generated hello executable exists
    when: the user runs `./hello`
    then: it prints hello
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
runner: "cargo test -p mamba --test runtime_tests --release -- {name} --test-threads=1"
fixtures:
  - id: aot_hello
    name: "test_aot_compile_hello"
    description: "compile_aot produces valid object bytes for a hello-world module"
  - id: aot_externs_unresolved
    name: "test_aot_externs_marked_import"
    description: "every mb_* in the catalog is declared Linkage::Import in the object"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/codegen/cranelift/aot.rs
    action: modify
    impl_mode: hand-written
    description: "cranelift_object::ObjectModule wrapper; shared MIR-to-clir lowering; emit Vec<u8> object bytes for system linker. Hand-written."
```
