---
change_id: cclab-taipan
type: gap_codebase_knowledge
created_at: 2026-02-12T07:38:34.987571+00:00
updated_at: 2026-02-12T07:38:34.987571+00:00
---

# Gap Analysis: Codebase vs Knowledge

## Summary
The primary gap is the total lack of knowledge documentation and codebase implementation for the 'Taipan' language itself, which is the subject of this change. While general performance and architectural patterns are documented, their application to the new compiler is entirely missing.

## Pattern Mismatches
- **Performance-Centric Development** (Ref: `orbit/performance-tuning.md`): The codebase currently lacks any benchmarking infrastructure (`benches/`) or performance targets for the upcoming Taipan compiler, which violates the "Performance First" and "Performance-centric development" patterns. (Severity: MEDIUM)
- **Modular CLI Pattern**: The `cclab-cli` uses a sophisticated `linkme`-based distributed registration pattern (see `registry.rs`), but this pattern is not documented in the knowledge base, leading to a knowledge-codebase mismatch for future contributors. (Severity: MEDIUM)

## Convention Violations
- **GIL Management** (Ref: Pitfalls - GIL contention): `crates/cclab-cli/src/main.rs` creates a multi-threaded tokio runtime for the `Server` command and uses `pyo3` for various actions. There is no explicit evidence of GIL management strategy (e.g., `allow_threads`) to avoid the documented contention pitfall in this multi-threaded context. (Severity: MEDIUM)

## Knowledge Gaps
- **Taipan Language Specification**: There is no knowledge document defining the Taipan language, its syntax, or its intended use cases, despite the change ID. (Severity: HIGH)
- **Compiler Architecture**: No knowledge documentation exists for the intended compiler pipeline (Frontend -> IR -> Backend) mentioned as a pattern in specs but missing from knowledge. (Severity: HIGH)
