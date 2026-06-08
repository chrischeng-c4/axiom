---
change_id: taipan-295-297
type: gap_codebase_knowledge
created_at: 2026-02-13T07:24:47.666139+00:00
updated_at: 2026-02-13T07:24:47.666139+00:00
---

# Gap Analysis: Codebase vs Knowledge

## Pattern Mismatches

- **Generator Inference Pattern Mismatch**: `crates/cclab-taipan/src/codegen/cranelift/mod.rs` currently uses explicit placeholders (iconst 0) for object operations. It does not follow the `Generator Inference Pattern` from `spec-to-code/code-generator-contract.md`, which suggests that high-level semantics (like GetAttr) should automatically infer the corresponding runtime FFI call (tp_getattr). (Severity: High)
- **FFI Error Handling Pattern Mismatch**: The `Raise` instruction in `crates/cclab-taipan/src/codegen/cranelift/mod.rs` is implemented as a simple `trap`. This contradicts the `FFI Error Handling Pattern` in `orbit/bridge-internals.md`, which recommends converting errors to structured exceptions at the FFI boundary (e.g., calling a runtime raise function). (Severity: Medium)

## Convention Violations

- **JIT Symbol Wiring**: The `cranelift-jit` requirement for explicit symbol registration (identified in `knowledge_context.md` pitfalls) is not yet addressed in the codebase, although this is the primary goal of the upcoming change. (Severity: High)
