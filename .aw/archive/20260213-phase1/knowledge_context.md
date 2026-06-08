---
change_id: phase1
type: knowledge_context
created_at: 2026-02-12T17:55:54.640102+00:00
updated_at: 2026-02-12T17:55:54.640102+00:00
iteration: 1
complexity: high
stage: knowledge
scanned_categories:
  - changelogs
  - 40-mcp
  - 30-claude
  - 05-titan
  - orbit
  - grid
---

# Knowledge Context

## Relevant Documents

- **orbit/bridge-internals.md**
  - summary: Describes Orbit's Rust-Python bridge with GIL management and FFI patterns. Relevant for taipan's runtime FFI boundary and object model design.
  - relevant sections: GIL batching, Object lifetime
- **changelogs/orbit-architecture.md**
  - summary: Orbit event loop architecture patterns. Reference for async runtime design if needed.
  - relevant sections: task scheduling
- **40-mcp/dynamic-config.md**
  - summary: Dynamic configuration loading. Minor relevance for taipan.toml config.
  - relevant sections: config reload

## Patterns

- **NaN-boxing for value representation** (source: Industry standard (LuaJIT, SpiderMonkey))
  - Pack all values into 64-bit using NaN payload bits. Inline ints/bools/None, pointer for heap objects.
- **Reference counting with cycle collector** (source: CPython implementation)
  - Deterministic refcounting for most objects, periodic cycle collector for reference cycles in containers.
- **setjmp/longjmp exception handling** (source: Lua, early CPython)
  - Use setjmp to set exception handler, longjmp to unwind. Simpler than DWARF but requires manual cleanup.

## Pitfalls

- NaN-boxing limits integer range to 51 bits — need bigint fallback for large numbers
- setjmp/longjmp skips destructors — must emit explicit cleanup before longjmp
- Reference counting cannot detect cycles in isolation — cycle collector must handle list/dict self-references
- File size limit: all files must stay under 500 lines per CLAUDE.md
