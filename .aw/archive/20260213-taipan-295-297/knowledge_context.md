---
change_id: taipan-295-297
type: knowledge_context
created_at: 2026-02-13T07:21:29.272539+00:00
updated_at: 2026-02-13T07:21:29.272539+00:00
iteration: 1
complexity: high
stage: knowledge
scanned_categories:
  - 05-titan
  - 30-claude
  - 40-mcp
  - changelogs
  - grid
  - orbit
  - spec-to-code
---

# Knowledge Context

## Relevant Documents

- **cclab/README.md**
  - summary: Overview of Genesis SDD process and document categorization.
  - relevant sections: Directory Structure, Three Document Types
- **orbit/bridge-internals.md**
  - summary: Best practices for Python-Rust FFI, specifically regarding GIL releasing and exception translation.
  - relevant sections: GIL Management, Error Propagation, Reference Counting
- **spec-to-code/code-generator-contract.md**
  - summary: Guidance on how generators should infer implementation details from spec semantics.
  - relevant sections: Principle, Inference Rules

## Patterns

- **GIL Release Pattern** (source: orbit/bridge-internals.md)
  - Use py.allow_threads to release GIL during long-running or blocking Rust execution.
- **Generator Inference Pattern** (source: spec-to-code/code-generator-contract.md)
  - Map high-level IR instructions to specific runtime FFI calls via automated inference.
- **FFI Error Handling Pattern** (source: orbit/bridge-internals.md)
  - Convert Rust panics and Errors to Python exceptions at the FFI boundary.

## Pitfalls

- Holding the GIL while waiting for cross-thread synchronization can lead to deadlocks.
- Incorrect reference counting (Py<T>) between Python and Rust leads to memory leaks or use-after-free.
- Cranelift JITModule requires explicit symbol registration for all external functions (tp_*) before compilation.
