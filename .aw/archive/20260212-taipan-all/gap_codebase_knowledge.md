---
change_id: taipan-all
type: gap_codebase_knowledge
created_at: 2026-02-12T10:40:20.109072+00:00
updated_at: 2026-02-12T10:40:20.109072+00:00
---

# Gap Analysis: Codebase vs Knowledge

## Convention Violations

- **crates/cclab-taipan** (Severity: Medium)
  - Missing Miri and Fuzz testing infrastructure. According to `changelogs/orbit-testing-safety.md`, a "safety-first" approach using these tools is mandatory for core high-performance crates.
- **crates/cclab-taipan** (Severity: Low)
  - Lack of modular feature flags. `changelogs/orbit-architecture.md` recommends using feature flags for modularity and platform-specific optimizations (e.g., kqueue on macOS), which is relevant given the user's OS is `darwin`.

## Pattern Mismatches

- **crates/cclab-taipan/src/types/context.rs** (Severity: Low)
  - The `TypeContext` interner uses a simple `Vec`. It could benefit from the `Slab Allocator` pattern mentioned in `changelogs/orbit-architecture.md` for better performance and ABA protection in long-lived sessions.
- **crates/cclab-taipan/src/driver/mod.rs** (Severity: Low)
  - No evidence of stage-specific tool/logic filtering as suggested in `40-mcp/dynamic-config.md`. While this doc is about MCP, the principle of minimizing overhead by stage could be applied to the compiler's modular passes.
