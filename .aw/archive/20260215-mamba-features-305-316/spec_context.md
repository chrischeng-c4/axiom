---
change_id: mamba-features-305-316
type: spec_context
created_at: 2026-02-14T09:25:16.324101+00:00
updated_at: 2026-02-14T09:25:16.324101+00:00
iteration: 1
complexity: high
stage: spec
scanned_groups:
  - cclab-mamba
  - cclab-orbit
  - cclab-nucleus
  - cclab-core
---

# Spec Context

## Relevant Specs

- **mamba-jit-backend** (group: cclab-mamba)
  - relevance: high
  - reason: Defines current backend architecture (Cranelift), which the LLVM backend will likely mirror or extend.
  - key sections: Overview, JIT Module Initialization, Runtime Symbol Wiring
- **mamba-py312-syntax** (group: cclab-mamba)
  - relevance: high
  - reason: Defines syntax for generics and f-strings, which need implementation/codegen (Features #312, #314).
  - key sections: PEP 695 (Type Parameter Syntax), PEP 701 (f-strings)
- **gil-waker-polling** (group: cclab-orbit)
  - relevance: high
  - reason: Provides the underlying runtime mechanism for async/await (Feature #313).
  - key sections: Waker-driven polling, GIL release on wait
- **architecture** (group: cclab-orbit)
  - relevance: high
  - reason: Event loop foundations for coroutine scheduling.
  - key sections: Orbit Event Loop Architecture
- **mamba-cpython-test-integration** (group: cclab-mamba)
  - relevance: medium
  - reason: Defines how to use CPython tests for verification.
  - key sections: Overview, Requirements

## Dependencies

- cclab-mamba/mamba-jit-backend: Backend architecture foundation
- cclab-mamba/mamba-py312-syntax: Syntax support for generics and f-strings
- cclab-orbit/gil-waker-polling: Foundations for async coroutine polling
- cclab-orbit/architecture: Event loop architecture for async/await

## Gaps

- mamba-llvm-backend: #305 LLVM backend for AOT compilation (currently only Cranelift mentioned)
- mamba-imports: #306 Multi-file import system specification
- mamba-oop: #307 Complete OOP (super, inheritance, operator overloading) specification
- mamba-codegen-comprehensions: #308 Comprehension and generator codegen specification
- mamba-codegen-pattern-matching: #309 Pattern matching (match/case) codegen specification
- mamba-stdlib: #310 Minimal standard library (sys, os, math, json) specification
- mamba-iteration-protocol: #311 For-loop iteration protocol specification
- mamba-string-ops: #312 String operations and f-string interpolation implementation/codegen specification
- mamba-async-await: #313 Async/await and coroutine scheduling high-level integration specification
- mamba-generics-protocols: #314 Generics and protocol types implementation specification
- mamba-gc: #315 Cycle-detecting GC and memory safety specification
- mamba-repl: #316 REPL and interactive mode specification
