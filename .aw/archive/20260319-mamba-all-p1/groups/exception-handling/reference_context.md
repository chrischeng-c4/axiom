---
change: mamba-all-p1
group: exception-handling
date: 2026-03-19
written_by: claude-revision
review_verdict: ADDRESSED
---

# Reference Context

| Spec | Relevance | Key Requirements |
|------|-----------|------------------|
| `cclab-mamba/runtime/exception.md` | high | Directly covers exception.rs (419 LOC, all three pre-clarification answers): R5 (ExceptionGroup wrapping Vec<MbValue> with mb_exception_group_new, split, subgroup methods for PEP 654 — Q1 implementation at exception.rs:417-456), R6 (except* AST variant with is_star parsing, HIR/MIR multi-handler splitting — Q1 mb_except_star at exception.rs:458-528), R2 (mb_exception_new with cause/traceback fields for exception instantiation — Q2 MbException chaining at exception.rs:10-24), R4 (thread-local state mb_raise, mb_get_exception, mb_clear_exception — Q2 helpers with_cause/with_context at exception.rs:38-46), R1 (exception class hierarchy for traceback display — Q3 chained display). Primary runtime spec for all three pre-clarifications. |
| `cclab-mamba/parser/statements.md` | high | Directly covers stmt_compound.rs (explicitly cited in Q1 at lines 289-295 for is_star flag): R2 (Compound Statement Parsing covering try/except with ExceptHandler including is_star flag for PEP 654 except* syntax). Parsing contract for except* desugaring unspecified without this spec. The is_star flag — which triggers except* semantics — lives at stmt_compound.rs:289-295, exact location cited in Q1. |
| `cclab-mamba/hir/hir.md` | high | Directly covers hir/mod.rs: R1 (HirStmt variants including Try exception handling blocks with HirExceptHandler). Pre-clarification Q1 explicitly states 'HIR propagates is_star in HirExceptHandler.' The HirExceptHandler with is_star is the HIR representation of except* — bridge between parser's is_star flag and MIR multi-handler splitting logic. HIR data-model contract for except* desugaring unspecified without this spec. |
| `cclab-mamba/stdlib/diagnostics-utils.md` | medium | Covers stdlib traceback and contextlib modules for exception handling integration (runtime/stdlib/traceback_mod.rs, runtime/stdlib/contextlib_mod.rs): R5 (traceback.format_exc — format current exception traceback with file/line/function per frame for Q3), R6 (traceback.print_exc — stderr output for Q3 rendering), R1 (contextlib.suppress — context manager suppressing listed exception types, depends on exception hierarchy). Pre-clarification Q3 explicitly states traceback rendering 'could benefit from integration with the diagnostic module for richer ANSI colored output and code snippets.' These specs cover the stdlib integration layer. |
| `cclab-mamba/README.md` | low | Authoritative spec index for cclab-mamba mapping source files to specifications. Shows exception.rs→runtime/exception.md, stmt_compound.rs→parser/statements.md, hir/mod.rs→hir/hir.md, traceback_mod.rs→stdlib/diagnostics-utils.md. Value is spec discovery only, not implementable requirements. All indexed specs now directly included above. |

## Coverage Analysis

**Pre-Clarifications Scope Coverage:**

| Question | Area | Files | Covered By | Status |
|----------|------|-------|-----------|--------|
| Q1: Exception Groups and except* | ExceptionGroup wrapping, except* AST variant, is_star parsing and propagation through parser/HIR/MIR, multi-handler splitting | exception.rs:417-528, stmt_compound.rs:289-295, hir/mod.rs | runtime/exception.md R5-R6 (ExceptionGroup, except* syntax desugaring), parser/statements.md R2 (is_star parsing in try/except), hir/hir.md R1 (HirExceptHandler with is_star) | ✅ Fully covered: Parser AST, HIR, and runtime implementation all specified |
| Q2: Exception Chaining | MbException struct with cause/context/suppress_context fields, with_cause/with_context helpers, mb_raise_from/mb_raise_with_context | exception.rs:10-46 | runtime/exception.md R2 (exception instantiation with chaining), R4 (thread-local state for raise/get_exception) | ✅ Fully covered: Exception chaining struct and helper functions specified |
| Q3: Chained Traceback Display | Traceback rendering showing file/line/function stack, chaining messages ('The above exception was the direct cause...'), potential diagnostic integration | exception.rs (chained display), stdlib/traceback_mod.rs | runtime/exception.md R1 (exception hierarchy structure used in chained display), stdlib/diagnostics-utils.md R5-R6 (traceback.format_exc, traceback.print_exc for rendering) | ✅ Fully covered: Exception hierarchy, traceback formatting, and stdlib integration specified |

**Gap Analysis:**

| Gap | Severity | Details | Recommendation |
|-----|----------|---------|-----------------|
| Missing HIGH-relevance primary specs for Q1/Q2/Q3 | CRITICAL (RESOLVED) | Original context was missing runtime/exception.md, parser/statements.md, hir/hir.md which directly implement Q1/Q2/Q3. all-mamba-p0.md and pattern-matching.md were included at low/low despite being entirely irrelevant (no connection to exception handling, ExceptionGroup, chaining, or traceback). | RESOLVED: Added three HIGH-relevance primary specs (runtime/exception.md, parser/statements.md, hir/hir.md). Removed all-mamba-p0.md and pattern-matching.md entirely. |
| Missing stdlib/diagnostics-utils.md for Q3 integration | HIGH (RESOLVED) | Q3 explicitly mentions potential integration with diagnostic module for richer output. traceback_mod.rs with R5-R6 (format_exc, print_exc) and contextlib.suppress (R1) were absent. | RESOLVED: Added stdlib/diagnostics-utils.md at MEDIUM relevance covering traceback formatting and contextlib integration. |
| all-mamba-p0.md incorrectly included at LOW | MEDIUM (RESOLVED) | all-mamba-p0.md R1-R5 (import aliases, PEP 634 patterns, BigInt, benchmarks, builtins conformance) have zero connection to exception handling, ExceptionGroup, chaining, or traceback rendering. None cited in any pre-clarification. Inclusion misguides implementers about in-scope features. | RESOLVED: Removed all-mamba-p0.md entirely from this group's context. Builtins conformance (R5) is unrelated to exception handling specifics. |
| pattern-matching.md incorrectly included at LOW | MEDIUM (RESOLVED) | pattern-matching.md covers PEP 634 match/case structural dispatch (R1-R8: parser patterns, AS patterns, type narrowing, HIR nodes, MIR decision trees, codegen). Zero connection to except*, exception hierarchy, or chaining. Pre-clarifications never mention match/case. | RESOLVED: Removed pattern-matching.md entirely from this group's context. Orthogonal feature should not be included at any relevance level. |
| README.md rated MEDIUM but has no implementable requirements | LOW (RESOLVED) | README.md is a spec-index table with no R1-Rn requirements, no acceptance criteria, no behavior specification. class-features review rated README.md LOW with note 'Value is spec discovery only, not implementable requirements' — MEDIUM was inconsistent. | RESOLVED: Downgraded README.md from MEDIUM to LOW, consistent with class-features review pattern. |

## Verification Summary

**Pre-clarifications Coverage:**
- ✅ Q1 (ExceptionGroup and except*): Covered by runtime/exception.md R5-R6 (ExceptionGroup wrapping, except* syntax splitting), parser/statements.md R2 (is_star parsing), hir/hir.md R1 (HirExceptHandler with is_star) — Complete parser AST, HIR, and runtime coverage
- ✅ Q2 (Exception chaining): Covered by runtime/exception.md R2 (mb_exception_new with cause/context/suppress_context), R4 (thread-local state mb_raise/mb_get_exception/mb_clear_exception) — Complete exception chaining struct and state management
- ✅ Q3 (Chained traceback display): Covered by runtime/exception.md R1 (exception hierarchy structure), stdlib/diagnostics-utils.md R5-R6 (traceback.format_exc, traceback.print_exc) — Complete exception hierarchy and traceback rendering
- ✅ All affected crates/source files covered: exception.rs, stmt_compound.rs, hir/mod.rs, traceback_mod.rs, contextlib_mod.rs

**Relevance Scores:**
- ✅ runtime/exception.md rated 'high' — directly implements Q1 (ExceptionGroup/except*/is_star), Q2 (MbException chaining), Q3 (exception hierarchy for display)
- ✅ parser/statements.md rated 'high' — directly implements Q1 is_star parsing in try/except (stmt_compound.rs:289-295)
- ✅ hir/hir.md rated 'high' — directly implements Q1 HirExceptHandler with is_star propagation, HIR data-model for except* desugaring
- ✅ stdlib/diagnostics-utils.md rated 'medium' — stdlib integration for Q3 traceback rendering and contextlib exception suppression
- ✅ README.md rated 'low' — spec index only, no implementable requirements

**Key Requirements Accuracy:**
- ✅ runtime/exception.md lists correct key requirements: R5 (ExceptionGroup), R6 (except* syntax), R2 (exception chaining), R4 (thread-local state), R1 (exception hierarchy)
- ✅ parser/statements.md lists correct key requirement: R2 (compound statement parsing including try/except with is_star)
- ✅ hir/hir.md lists correct key requirement: R1 (HirStmt variants including Try with HirExceptHandler)
- ✅ stdlib/diagnostics-utils.md lists correct key requirements: R5-R6 (traceback formatting), R1 (contextlib.suppress)

## Review Feedback Resolutions

### [HIGH] runtime/exception.md is absent
- **Status**: ✅ RESOLVED
- **Action**: Added runtime/exception.md at HIGH relevance. Key requirements: R5 (ExceptionGroup), R6 (except* syntax), R2 (exception instantiation with chaining), R4 (thread-local state).
- **Details**: This spec directly covers exception.rs:417-528, which is cited in all three pre-clarification answers. Q1 ExceptionGroup (R5: mb_exception_group_new, split, subgroup, exceptions) and mb_except_star (R6: except* desugaring) map directly to exception.rs:417-528. Q2 MbException chaining (R2: cause/context/suppress_context fields, R4: mb_raise/mb_get_exception/mb_clear_exception) maps to exception.rs:10-46. Q3 exception hierarchy (R1) provides the class structure for chained traceback rendering. Without this spec, implementers have no contract for the exception.rs implementation.

### [HIGH] parser/statements.md is absent
- **Status**: ✅ RESOLVED
- **Action**: Added parser/statements.md at HIGH relevance. Key requirement: R2 (compound statement parsing including try/except with is_star flag).
- **Details**: This spec covers stmt_compound.rs, which is explicitly cited in Q1 at lines 289-295 for the is_star flag on except handlers. R2 specifies compound statement parsing including the try/except statement and ExceptHandler with is_star. The parsing contract for except* syntax is entirely unspecified without this spec. The is_star flag that triggers except* semantics lives at the exact location cited in Q1.

### [HIGH] hir/hir.md is absent
- **Status**: ✅ RESOLVED
- **Action**: Added hir/hir.md at HIGH relevance. Key requirement: R1 (HirStmt variants including Try with HirExceptHandler).
- **Details**: This spec covers hir/mod.rs. Pre-clarification Q1 explicitly states 'HIR propagates is_star in HirExceptHandler.' R1 defines HirStmt variants including Try for exception handling blocks. HirExceptHandler with is_star is the bridge between the parser's is_star flag and the MIR multi-handler splitting logic. The HIR data-model contract for except* desugaring is entirely unspecified without this spec.

### [MEDIUM] stdlib/diagnostics-utils.md is absent
- **Status**: ✅ RESOLVED
- **Action**: Added stdlib/diagnostics-utils.md at MEDIUM relevance. Key requirements: R5 (traceback.format_exc), R6 (traceback.print_exc), R1 (contextlib.suppress).
- **Details**: This spec covers stdlib traceback and contextlib modules. Pre-clarification Q3 explicitly states traceback rendering 'could benefit from integration with the diagnostic module for richer ANSI colored output and code snippets.' R5-R6 cover traceback formatting with file/line/function per frame. R1 covers contextlib.suppress for suppressing exception types, which depends on exception hierarchy. These specs provide the stdlib integration contract for Q3's traceback rendering.

### [MEDIUM] all-mamba-p0.md is irrelevant
- **Status**: ✅ RESOLVED
- **Action**: Removed all-mamba-p0.md from this group's reference context entirely.
- **Details**: all-mamba-p0.md R1-R5 (import aliases/relative imports, PEP 634 pattern matching, BigInt overflow, benchmark suite, builtins conformance) have zero intersection with exception handling, ExceptionGroup, exception chaining, or traceback rendering. None of the three pre-clarification questions mention module systems, BigInt arithmetic, benchmark suites, or structural pattern matching. Including it misguides implementers about what features are in scope for exception-handling work.

### [LOW] pattern-matching.md is irrelevant
- **Status**: ✅ RESOLVED
- **Action**: Removed pattern-matching.md from this group's reference context entirely.
- **Details**: pattern-matching.md covers PEP 634 match/case structural dispatch (R1-R8: parser pattern types, AS patterns, mapping **rest, case-branch type narrowing, HIR pattern nodes, AST→HIR lowering, nested patterns in MIR, Cranelift codegen). None of these requirements address except*, exception hierarchy, ExceptionGroup, or traceback rendering. While both patterns and exceptions involve type-based dispatch, exception matching and pattern matching are orthogonal language features. Pre-clarifications do not reference match/case or HirPattern. Including it inverted correct priority.

### [LOW] README.md is rated MEDIUM but has no implementable requirements
- **Status**: ✅ RESOLVED
- **Action**: Downgraded README.md from MEDIUM to LOW, consistent with class-features review.
- **Details**: README.md is a spec-index table with no R1-Rn requirements, no acceptance criteria, no behavioral contracts. The class-features review for this same change rated README.md LOW with note 'Value is spec discovery only, not implementable requirements' — MEDIUM was inconsistent. README.md's value lies in pointing to exception.md, statements.md, hir.md, and diagnostics-utils.md, but these must be directly included (which they now are) rather than discovered through README indirection.
