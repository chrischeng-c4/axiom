---
change_id: mamba-features-305-316
type: gap_codebase_spec
created_at: 2026-02-14T09:28:00.856995+00:00
updated_at: 2026-02-14T09:28:00.856995+00:00
---

# Gap Analysis: Codebase vs Spec

## Missing Specifications for Existing Code

- **OOP System (runtime/class.rs)**
  - Code: Implements MRO, instance creation, attribute access, and operator overloading.
  - Spec: NO matching spec found in `cclab/specs/`.
  - Severity: **High** (Core feature with complex logic).

- **Async Runtime (runtime/async_rt.rs)**
  - Code: Implements basic coroutine lifecycle and `mb_await`.
  - Spec: NO matching spec found. Existing `gil-waker-polling` is general Orbit logic, not Mamba-specific.
  - Severity: **High** (Critical for coroutine scheduling #313).

- **Pattern Matching Parser (parser/pattern.rs)**
  - Code: Advanced pattern parsing for sequence, mapping, and class patterns.
  - Spec: NO matching spec.
  - Severity: **Medium**.

- **Comprehension Parser (parser/expr_compound.rs)**
  - Code: Support for list, set, dict comprehensions and generator expressions.
  - Spec: NO matching spec.
  - Severity: **Medium**.

- **Memory Management (runtime/rc.rs)**
  - Code: Reference counting implementation.
  - Spec: NO matching spec.
  - Severity: **High** (Foundation for #315 GC).

## Missing Implementation for Planned Features

- **LLVM Backend (#305)**
  - Spec: NO spec found (only Cranelift JIT mentioned).
  - Code: NO LLVM implementation in `codegen/`.
  - Severity: **High**.

- **Multi-file Import System (#306)**
  - Spec: NO spec found.
  - Code: NO visible module resolution logic.
  - Severity: **High**.

- **Minimal Standard Library (#310)**
  - Spec: NO spec found.
  - Code: `runtime/builtins.rs` exists but is empty/minimal.
  - Severity: **Medium**.

- **Cycle-detecting GC (#315)**
  - Spec: NO spec found.
  - Code: Current implementation is simple RC.
  - Severity: **High**.

- **REPL and Interactive Mode (#316)**
  - Spec: NO spec found.
  - Code: NO REPL implementation.
  - Severity: **Low**.
