---
change_id: taipan-283-294
type: gap_codebase_knowledge
created_at: 2026-02-13T04:14:57.160282+00:00
updated_at: 2026-02-13T04:14:57.160282+00:00
---

# Gap Analysis: Codebase vs Knowledge

## Pattern Mismatches

- **crates/cclab-taipan/src/mir/mod.rs** vs **changelogs/orbit-architecture.md** (Slab Allocator)
  - Gap: The MIR implementation uses standard `Vec<BasicBlock>` and `Vec<MirInst>` without a Slab/Arena allocator. For a high-performance compiler, a Slab allocator is recommended to reduce fragmentation and improve cache locality, a pattern established in the Orbit architecture.
  - Severity: Medium
- **crates/cclab-taipan/src/runtime/rc.rs** vs **orbit/bridge-internals.md** (GIL Release)
  - Gap: The current `tp_retain` and `tp_release` functions do not account for GIL management if they are called from native threads during FFI execution. The Orbit bridge patterns mandate explicit GIL release/acquire strategies for such cases.
  - Severity: High
- **crates/cclab-taipan/src/lower/ast_to_hir.rs** vs **05-titan/architecture-guide.md** (Decoupling)
  - Gap: The lowering passes are tightly coupled in the `lower` module. The Titan architecture guide recommends stronger decoupling between transformation passes to improve testability and maintainability.
  - Severity: Low

## Convention Violations

- **crates/cclab-taipan/** vs **changelogs/orbit-testing-safety.md** (Fuzzing/Miri)
  - Gap: The Taipan crate lacks fuzz testing and Miri validation suites. These are mandatory for core high-performance Rust crates in the Cclab ecosystem to ensure safety and prevent undefined behavior in unsafe code (e.g., NaN-boxing).
  - Severity: High
