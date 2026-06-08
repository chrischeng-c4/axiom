---
id: mamba-features-305-316
type: proposal
version: 2
created_at: 2026-02-14T09:29:56.582606+00:00
updated_at: 2026-02-14T09:29:56.582606+00:00
iteration: 1
scope: major
spec_plan:
  - id: mamba-llvm-backend
    title: "LLVM Backend for AOT Compilation (#305)"
    depends: []
    context_refs:
      codebase: ["crates/mamba/src/codegen/mod.rs"]
      spec: ["mamba-jit-backend"]
    affected_code: ["crates/mamba/src/codegen/llvm/", "crates/mamba/src/codegen/mod.rs"]
  - id: mamba-gc-runtime
    title: "Cycle-Detecting GC and Memory Safety (#315)"
    depends: []
    context_refs:
      knowledge: ["orbit/performance-tuning.md: Memory Ownership"]
    affected_code: ["crates/mamba/src/runtime/rc.rs", "crates/mamba/src/runtime/gc.rs"]
  - id: mamba-string-runtime
    title: "String Operations and f-string Interpolation (#312)"
    depends: []
    context_refs:
      spec: ["mamba-py312-syntax: PEP 701 f-strings"]
    affected_code: ["crates/mamba/src/runtime/string_ops.rs"]
  - id: mamba-iteration-protocol
    title: "For-loop Iteration Protocol (#311)"
    depends: []
    affected_code: ["crates/mamba/src/runtime/iter.rs"]
  - id: mamba-import-system
    title: "Multi-file Import System (#306)"
    depends: []
    affected_code: ["crates/mamba/src/resolve/import.rs", "crates/mamba/src/runtime/module.rs"]
  - id: mamba-oop-model
    title: "Complete OOP Model: Inheritance, super(), and Dunder Methods (#307)"
    depends: []
    context_refs:
      codebase: ["crates/mamba/src/runtime/class.rs"]
    affected_code: ["crates/mamba/src/runtime/class.rs"]
  - id: mamba-type-system
    title: "Generics and Protocol Types (#314)"
    depends: [mamba-oop-model]
    context_refs:
      spec: ["mamba-py312-syntax: PEP 695 Type Parameter Syntax"]
    affected_code: ["crates/mamba/src/types/generic.rs", "crates/mamba/src/types/protocol.rs"]
  - id: mamba-codegen-logic
    title: "Comprehension, Generator, and Pattern Matching Codegen (#308, #309)"
    depends: [mamba-iteration-protocol, mamba-oop-model]
    context_refs:
      codebase: ["crates/mamba/src/parser/pattern.rs", "crates/mamba/src/parser/expr_compound.rs"]
    affected_code: ["crates/mamba/src/lower/comprehension.rs", "crates/mamba/src/lower/match.rs"]
  - id: mamba-async-runtime
    title: "Async/Await and Coroutine Scheduling (#313)"
    depends: [mamba-codegen-logic]
    context_refs:
      knowledge: ["orbit/bridge-internals.md: Core Architecture"]
    affected_code: ["crates/mamba/src/runtime/async_rt.rs"]
  - id: mamba-stdlib-core
    title: "Minimal Standard Library (#310)"
    depends: [mamba-import-system, mamba-async-runtime]
    affected_code: ["crates/mamba/src/runtime/builtins.rs"]
  - id: mamba-repl-tool
    title: "REPL and Interactive Mode (#316)"
    depends: [mamba-stdlib-core]
    affected_code: ["crates/mamba/src/driver/repl.rs"]
history:
  - timestamp: 2026-02-14T09:29:56.582606+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
---

<proposal>

# Spec Navigation Map: mamba-features-305-316

## Scope Overview (Mindmap)

```mermaid
mindmap
  root((mamba-features-305-316))  
    Backend Architecture
      LLVM integration
      AOT compilation
      Pluggable backends
    Language Core
      OOP inheritance
      super()
      Operator overloading
      Multi-file imports
      Generics
      Protocols
    Runtime & Memory
      Cycle-detecting GC
      Memory safety
      Coroutine scheduling
      Orbit integration
      Iteration protocol
    Syntactic Features
      Comprehensions
      Generator codegen
      Pattern matching
      f-strings
    Tooling & Stdlib
      Minimal stdlib (sys, os, math, json)
      Interactive REPL
```

## Spec Dependency Graph (Block Diagram)

```mermaid
block-beta
  columns 3

  mamba_llvm_backend["mamba-llvm-backend\n codebase: crates/mamba/src/codegen/mod.rs"]
  mamba_gc_runtime["mamba-gc-runtime"]
  mamba_string_runtime["mamba-string-runtime"]
  mamba_iteration_protocol["mamba-iteration-protocol"]
  mamba_import_system["mamba-import-system"]
  mamba_oop_model["mamba-oop-model\n codebase: crates/mamba/src/runtime/class.rs"]
  mamba_type_system["mamba-type-system"]
  mamba_codegen_logic["mamba-codegen-logic\n codebase: crates/mamba/src/parser/pattern.rs, crates/mamba/src/parser/expr_compound.rs"]
  mamba_async_runtime["mamba-async-runtime"]
  mamba_stdlib_core["mamba-stdlib-core"]
  mamba_repl_tool["mamba-repl-tool"]

  mamba_oop_model --> mamba_type_system
  mamba_iteration_protocol --> mamba_codegen_logic
  mamba_oop_model --> mamba_codegen_logic
  mamba_codegen_logic --> mamba_async_runtime
  mamba_import_system --> mamba_stdlib_core
  mamba_async_runtime --> mamba_stdlib_core
  mamba_stdlib_core --> mamba_repl_tool
```

## Spec Execution Order

1. **mamba-gc-runtime** — Cycle-Detecting GC and Memory Safety (#315)
   - code: crates/mamba/src/runtime/rc.rs, crates/mamba/src/runtime/gc.rs
2. **mamba-import-system** — Multi-file Import System (#306)
   - code: crates/mamba/src/resolve/import.rs, crates/mamba/src/runtime/module.rs
3. **mamba-iteration-protocol** — For-loop Iteration Protocol (#311)
   - code: crates/mamba/src/runtime/iter.rs
4. **mamba-llvm-backend** — LLVM Backend for AOT Compilation (#305)
   - code: crates/mamba/src/codegen/llvm/, crates/mamba/src/codegen/mod.rs
5. **mamba-oop-model** — Complete OOP Model: Inheritance, super(), and Dunder Methods (#307)
   - code: crates/mamba/src/runtime/class.rs
6. **mamba-codegen-logic** — Comprehension, Generator, and Pattern Matching Codegen (#308, #309)
   - depends: mamba-iteration-protocol, mamba-oop-model
   - code: crates/mamba/src/lower/comprehension.rs, crates/mamba/src/lower/match.rs
7. **mamba-async-runtime** — Async/Await and Coroutine Scheduling (#313)
   - depends: mamba-codegen-logic
   - code: crates/mamba/src/runtime/async_rt.rs
8. **mamba-stdlib-core** — Minimal Standard Library (#310)
   - depends: mamba-import-system, mamba-async-runtime
   - code: crates/mamba/src/runtime/builtins.rs
9. **mamba-repl-tool** — REPL and Interactive Mode (#316)
   - depends: mamba-stdlib-core
   - code: crates/mamba/src/driver/repl.rs
10. **mamba-string-runtime** — String Operations and f-string Interpolation (#312)
   - code: crates/mamba/src/runtime/string_ops.rs
11. **mamba-type-system** — Generics and Protocol Types (#314)
   - depends: mamba-oop-model
   - code: crates/mamba/src/types/generic.rs, crates/mamba/src/types/protocol.rs

</proposal>
