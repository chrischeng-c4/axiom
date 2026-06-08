---
change_id: taipan-283-294
type: knowledge_context
created_at: 2026-02-13T04:12:40.571266+00:00
updated_at: 2026-02-13T04:12:40.571266+00:00
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

- **orbit/bridge-internals.md**
  - summary: Critical guidance on how to safely bridge Rust native code with the Python runtime, specifically regarding GIL management and exception mapping. Directly applicable to Taipan's runtime FFI.
  - relevant sections: GIL Release Strategy, Error Propagation and Mapping
- **changelogs/orbit-architecture.md**
  - summary: Recommends Slab allocation for high-performance memory management in core crates. Relevant for Taipan's IR and AST object management.
  - relevant sections: Slab Allocator Implementation
- **changelogs/orbit-testing-safety.md**
  - summary: Mandatory safety standards for high-performance Rust crates, including fuzzing and Miri checks. Taipan must adhere to these as a core compiler.
  - relevant sections: Fuzz Testing Requirements, Miri Validation
- **05-titan/architecture-guide.md**
  - summary: General architectural guidance on decoupling system components, useful for designing the compiler's pass architecture.
  - relevant sections: Decoupling Passes

## Patterns

- **GIL Release Strategy** (source: orbit/bridge-internals.md)
  - Explicit strategy for when to hold/release the Python GIL during native execution.
- **Slab Allocator for IR** (source: changelogs/orbit-architecture.md)
  - Using Slab allocators to manage lifecycle of high-volume, short-lived objects.
- **Error Hierarchy Mapping** (source: orbit/bridge-internals.md)
  - Formal mapping between native errors and Python exception types.

## Pitfalls

- GIL deadlocks when calling Python-bound code from native threads without proper acquisition.
- Memory bloat in SSA IR if objects are not pooled or slab-allocated.
- Loss of source location information during multi-stage lowering (Lower -> MIR -> Codegen).
