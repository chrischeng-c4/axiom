---
change: mamba-all-p1
group: async-generators
date: 2026-03-19
written_by: artifact_cli
review_verdict: REVISED
---

# Reference Context

| Spec | Relevance | Key Requirements |
|------|-----------|------------------|
| `cclab-mamba/runtime/generator.md` | high | Generator state machine implementation (R1-R5): compilation to state machine with Created/Suspended/Running/Completed states, yield expression suspension and resumption preserving local variables and instruction pointer, generator iterator protocol (__iter__ returns self, __next__ resumes), send(value) method for resumption with value, throw(exception) and close() methods with GeneratorExit handling. Foundational for Q1 (sync generators) and Q2 (async generator architecture). |
| `cclab-mamba/runtime/async.md` | high | Async/await runtime integration (R1-R4): async function compilation to state machine objects with suspension/resumption at await points, Orbit event loop integration (mb_coroutine_suspend, mb_coroutine_resume, asyncio.run, asyncio.gather), GIL-safe scheduling with release during suspension and re-acquisition on resumption, future interoperability with Tokio futures. Critical for Q2 (async generators need async runtime foundation). |
| `cclab-mamba/runtime/exception.md` | high | Exception hierarchy and protocols (includes StopAsyncIteration): builtin exception class tree, exception creation and raising, StopIteration exception with value attribute for generator returns. StopAsyncIteration exception variant must be added for async generator protocol. Needed for Q2 (async generator termination) and exception handling in async contexts. |
| `cclab-mamba/runtime/iter.md` | high | Iteration protocol implementation (__iter__, __next__, StopIteration): async iteration counterpart dunder methods (__aiter__, __anext__) must be added for async generators. Async for/with desugaring depends on async iteration protocol. Covers Q3 (async for/with implementation). |
| `cclab-mamba/runtime/class.md` | high | Magic method dispatch system (R5 especially): operators and protocol method dispatch to dunder methods. Specifically: __aiter__, __anext__, __aenter__, __aexit__ async protocol methods must be added and dispatched. R8 (context manager protocol) covers __enter__/__exit__ which must be extended to async context manager (__aenter__/__aexit__). Critical for Q3 (async for/with context manager protocol). |
| `cclab-mamba/hir/hir.md` | high | HIR node definitions and async construct support: HirStmt variants for AsyncFor and AsyncWith statements must be added to AST-to-HIR output. Currently async constructs are parsed but dropped/collapsed in lowering. Defines the intermediate representation that bridges AST to code generation. Covers Q3 (async for/with HIR representation). |
| `cclab-mamba/lower/ast-to-hir.md` | high | AST to HIR desugaring (R3: With-statement desugaring): async for/with desugaring rules. R3 covers with-statement desugaring to __enter__/__exit__ protocol calls; async with must be extended to __aenter__/__aexit__ calls. AsyncFor desugaring to __aiter__/__anext__ protocol calls must be added (currently AsyncFor is silently dropped in ast_to_hir.rs). Covers Q3 (async for/with desugaring and HirStmt variant generation). |
| `cclab-mamba/testing/mamba-py312-conformance.md` | high | Python 3.12 conformance testing (R6.1-R6.7): R6 Generator & Iterator Protocol establishes acceptance criteria with test scenarios for yield, yield from, send(), throw(), close(), StopIteration.value. R6.7 explicitly marks "Async generators — xfail" defining the conformance obligation for this change group. R6.1-R6.6 cover sync generator protocol tests that underpin Q1. Directly specifies the test fixtures and expected golden-file outputs (CPython 3.12 reference). Maps to all three pre-clarification areas: Q1 (R6.1-R6.6 generators), Q2 (R6.7 async generators with xfail), Q3 (R6.6 iterator protocol for for/with loops). |

## Coverage Analysis

**Pre-Clarifications Scope Coverage:**

| Area | Files | Covered By |
|------|-------|-----------|
| Q1: Sync Generators | generator.rs, class.rs, exception.rs | runtime/generator.md (R1-R5 state machine, send/throw/close), runtime/class.md (R5 method dispatch), runtime/exception.md (StopIteration), mamba-py312-conformance.md (R6.1-R6.6 test criteria) |
| Q2: Async Generators | tokio_exec.rs, generator.rs | runtime/async.md (R1-R4 async runtime, coroutine scheduling), runtime/generator.md (R1 state machine foundation), runtime/exception.md (StopAsyncIteration), mamba-py312-conformance.md (R6.7 async generator test spec) |
| Q3: Async For/With | ast_to_hir.rs, class.rs, hir/ | runtime/iter.md (R __aiter__/__anext__ protocol), runtime/class.md (R5 __aenter__/__aexit__ dispatch, R8 context manager), hir/hir.md (async construct HIR nodes), lower/ast-to-hir.md (R3 desugaring rules for async with, new rules for async for), mamba-py312-conformance.md (R6.6 iterator protocol scenarios) |
| Exception Handling | exception.rs | runtime/exception.md (exception hierarchy, StopAsyncIteration), mamba-py312-conformance.md (exception scenarios) |

**Gap Analysis:**

All 8 critical specs are now directly included at 'high' relevance. Each spec's requirements explicitly define what must be extended for async generator support:
- runtime/generator.md defines the state machine foundation that async generators must inherit
- runtime/async.md defines the event loop and GIL-safe execution model required for async generators
- runtime/exception.md defines exception classes including StopAsyncIteration for async termination
- runtime/iter.md defines __aiter__/__anext__ protocol methods for async iteration
- runtime/class.md defines magic method dispatch for async dunder methods
- hir/hir.md defines HIR node structure for async statements
- lower/ast-to-hir.md defines desugaring rules that must be extended for async for/with
- mamba-py312-conformance.md defines the test acceptance criteria and explicitly marks async generators as xfail

# Reviews

## Review: reviewer (Iteration 1) — REVISED

**Change ID**: mamba-all-p1

**Verdict**: REVIEWED → REVISED

### Summary

All 8 high-priority issues from Iteration 1 have been addressed:
1. ✅ All 7 directly relevant specs now included at 'high' relevance (removed README.md indirection)
2. ✅ mamba-py312-conformance.md added at 'high' relevance with R6.1-R6.7 test criteria
3. ✅ all-mamba-p0.md removed (key requirements were factually incorrect)
4. ✅ README.md removed (index document, not a spec)
5. ✅ pattern-matching.md removed (entirely orthogonal to async generators)
6. ✅ Key requirements now accurately reflect actual spec content
7. ✅ Coverage analysis updated to map each pre-clarification area to supporting specs
8. ✅ Gap analysis replaced with complete spec inclusion statement

### Revision Checklist

- ✅ All affected crates/areas from pre-clarifications are covered by at least one spec
  - Q1 (sync generators: generator.rs) covered by runtime/generator.md + mamba-py312-conformance.md
  - Q2 (async generators: tokio_exec.rs, generator.rs) covered by runtime/async.md + runtime/generator.md + runtime/exception.md + mamba-py312-conformance.md
  - Q3 (async for/with: ast_to_hir.rs, class.rs, hir/) covered by runtime/iter.md + runtime/class.md + hir/hir.md + lower/ast-to-hir.md + mamba-py312-conformance.md

- ✅ Relevance scores are reasonable
  - All 8 specs rated 'high' — each directly implements or defines requirements for async generators, async runtime, or async protocol
  - Removed 3 low-value specs: README.md (index only), pattern-matching.md (orthogonal), all-mamba-p0.md (incorrect requirements)

- ✅ Key requirements listed per spec are accurate
  - runtime/generator.md: R1-R5 correctly reflect state machine, yield/resume, send/throw/close
  - runtime/async.md: R1-R4 correctly reflect async compilation, event loop integration, GIL safety, Tokio interop
  - runtime/exception.md: correctly includes StopAsyncIteration for async generators
  - runtime/iter.md: correctly includes __aiter__/__anext__ async protocol methods
  - runtime/class.md: R5 correctly reflects dunder dispatch for __aiter__/__anext__/__aenter__/__aexit__; R8 correctly covers async context manager protocol
  - hir/hir.md: correctly requires AsyncFor/AsyncWith HIR variants
  - lower/ast-to-hir.md: R3 correctly describes async with desugaring; implicitly covers async for desugaring requirement
  - mamba-py312-conformance.md: R6.1-R6.7 correctly define test acceptance criteria with R6.7 explicitly marking async generators as xfail

- ✅ No irrelevant specs included
  - All 8 specs directly address async generators, async runtime, or async protocol
  - Removed pattern-matching.md per feedback

### Issues Resolved

- **[HIGH] Removed** All 7 directly relevant specs were absent → Now included at 'high' relevance
- **[HIGH] Added** mamba-py312-conformance.md with R6.1-R6.7 test criteria → Now the most directly relevant spec for acceptance bar
- **[HIGH] Corrected** all-mamba-p0.md key requirements were factually wrong → Removed entirely
- **[MEDIUM] Removed** README.md is an index document, not a specification → Replaced with direct spec inclusions
- **[LOW] Removed** pattern-matching.md covers PEP 634 match/case, entirely orthogonal → Removed

### Confirmation

Pre-clarifications scope is now fully covered by concrete, directly relevant specs with accurate key requirements and proper relevance ratings.
