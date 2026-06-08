---
change_id: mamba-p3
type: gap_codebase_knowledge
created_at: 2026-02-23T01:13:05.759842+00:00
updated_at: 2026-02-23T01:13:05.759842+00:00
---

# Gap Analysis: Codebase vs Knowledge

## Convention Violations
None identified. All existing stdlib modules follow the documented stdlib-module-pattern (register() fn, HashMap attrs, wired in mod.rs).

## Pattern Mismatches
1. **No knowledge doc for Mamba runtime patterns** (medium) — The knowledge base has no Mamba-specific documentation. All patterns (NaN-boxing, ObjData, rt_sym!, etc.) are only documented in code comments and the CLAUDE.md. P3 implementation depends on tribal knowledge from P1/P2 sessions.
2. **External crate dependency pattern undocumented** (low) — P3 introduces first external crate dependencies (rusqlite, flate2, zip, tar). No knowledge doc covers the pattern for wrapping external Rust crates into Mamba stdlib modules.
3. **Threading safety pattern undocumented** (medium) — The thread-local registry pattern is documented but cross-thread access for threading module (#417) has no precedent in the knowledge base.

## Summary
No convention violations in existing code. Main gap: no Mamba-specific knowledge docs exist. P3 introduces two new patterns (external crate wrapping, multi-threaded access) not covered by existing knowledge.